mod common;

use axum::http::StatusCode;
use common::TestApp;
use sea_orm::ConnectionTrait;
use serde_json::{json, Value};

fn post_id(response: &axum_test::TestResponse) -> i32 {
    response.json::<Value>()["data"]["id"]
        .as_i64()
        .expect("post response should contain an integer id") as i32
}

#[tokio::test]
async fn post_pin_authorization_is_enforced_through_http() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("author-one", "author").await?;
    let sub_admin = app.create_user("sub-admin-one", "sub_admin").await?;

    let omitted = app
        .server
        .post("/api/v1/posts")
        .authorization_bearer(&author.token)
        .json(&json!({"title": "omitted pin", "status": "draft"}))
        .await;
    omitted.assert_status_ok();
    assert!(!omitted.json::<Value>()["data"]["is_pinned"]
        .as_bool()
        .unwrap());

    let unpinned = app
        .server
        .post("/api/v1/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "title": "explicit false pin",
            "status": "draft",
            "is_pinned": false
        }))
        .await;
    unpinned.assert_status_ok();
    assert_eq!(
        unpinned.json::<Value>()["data"]["user_id"].as_i64(),
        Some(author.id as i64)
    );
    let post_id = post_id(&unpinned);

    app.server
        .post("/api/v1/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "title": "forbidden true pin",
            "status": "draft",
            "is_pinned": true
        }))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    app.server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&author.token)
        .json(&json!({"title": "same false", "is_pinned": false}))
        .await
        .assert_status_ok();

    app.server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&author.token)
        .json(&json!({"is_pinned": true}))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    let pinned = app
        .server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&sub_admin.token)
        .json(&json!({"is_pinned": true}))
        .await;
    pinned.assert_status_ok();
    assert!(pinned.json::<Value>()["data"]["is_pinned"]
        .as_bool()
        .unwrap());

    app.server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&author.token)
        .json(&json!({"title": "same true", "is_pinned": true}))
        .await
        .assert_status_ok();

    app.server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&author.token)
        .json(&json!({"is_pinned": false}))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    let unpinned = app
        .server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&sub_admin.token)
        .json(&json!({"is_pinned": false}))
        .await;
    unpinned.assert_status_ok();
    assert!(!unpinned.json::<Value>()["data"]["is_pinned"]
        .as_bool()
        .unwrap());

    Ok(())
}

#[tokio::test]
async fn unchanged_author_pin_payload_is_not_written_to_sql() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("author-race", "author").await?;

    let created = app
        .server
        .post("/api/v1/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "title": "pin write detector",
            "status": "draft",
            "is_pinned": false
        }))
        .await;
    created.assert_status_ok();
    let post_id = post_id(&created);

    app.db
        .execute_unprepared(
            "CREATE TRIGGER reject_pin_writes \
             BEFORE UPDATE OF is_pinned ON posts \
             BEGIN SELECT RAISE(ABORT, 'pin write detected'); END;",
        )
        .await?;

    let updated = app
        .server
        .put(&format!("/api/v1/posts/{post_id}"))
        .authorization_bearer(&author.token)
        .json(&json!({
            "title": "ordinary author edit",
            "is_pinned": false
        }))
        .await;
    updated.assert_status_ok();
    assert_eq!(
        updated.json::<Value>()["data"]["title"].as_str(),
        Some("ordinary author edit")
    );

    Ok(())
}
