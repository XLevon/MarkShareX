mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};

fn ids(response: &axum_test::TestResponse) -> Vec<i64> {
    response.json::<Value>()["data"]
        .as_array()
        .expect("list response should contain data array")
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect()
}

fn assert_hidden(response: &axum_test::TestResponse, secret_title: &str) {
    response.assert_status(StatusCode::NOT_FOUND);
    assert!(!response.text().contains(secret_title));
}

#[tokio::test]
async fn public_post_reads_hide_drafts_from_every_non_owner() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("draft-visitor", "visitor").await?;
    let owner = app.create_user("draft-owner", "author").await?;
    let other_author = app.create_user("draft-other", "author").await?;
    let sub_admin = app.create_user("draft-sub-admin", "sub_admin").await?;
    let admin = app.create_user("draft-admin", "admin").await?;

    let published = app
        .create_post(&owner, "public integration article", "published")
        .await?;
    let draft = app
        .create_post(&owner, "secret integration draft", "draft")
        .await?;

    app.server
        .get(&format!("/api/v1/posts/{}", published.id))
        .await
        .assert_status_ok();
    app.server
        .get(&format!("/api/v1/posts/slug/{}", published.slug))
        .await
        .assert_status_ok();

    let anonymous_by_id = app.server.get(&format!("/api/v1/posts/{}", draft.id)).await;
    assert_hidden(&anonymous_by_id, &draft.title);
    let anonymous_by_slug = app
        .server
        .get(&format!("/api/v1/posts/slug/{}", draft.slug))
        .await;
    assert_hidden(&anonymous_by_slug, &draft.title);

    for user in [&visitor, &other_author] {
        let response = app
            .server
            .get(&format!("/api/v1/posts/{}", draft.id))
            .authorization_bearer(&user.token)
            .await;
        assert_hidden(&response, &draft.title);
    }

    for user in [&owner, &sub_admin, &admin] {
        let response = app
            .server
            .get(&format!("/api/v1/posts/{}", draft.id))
            .authorization_bearer(&user.token)
            .await;
        response.assert_status_ok();
        assert_eq!(
            response.json::<Value>()["data"]["title"].as_str(),
            Some(draft.title.as_str())
        );
    }

    let crafted_list = app
        .server
        .get("/api/v1/posts?status=draft&page_size=100")
        .await;
    crafted_list.assert_status_ok();
    let visible_ids = ids(&crafted_list);
    assert!(visible_ids.contains(&(published.id as i64)));
    assert!(!visible_ids.contains(&(draft.id as i64)));
    assert!(!crafted_list.text().contains(&draft.title));

    Ok(())
}

#[tokio::test]
async fn draft_related_public_endpoints_return_not_found_without_side_effects() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let owner = app.create_user("surface-owner", "author").await?;
    let visitor = app.create_user("surface-visitor", "visitor").await?;
    let draft = app
        .create_post(&owner, "secret related surface draft", "draft")
        .await?;

    app.server
        .get(&format!("/api/v1/posts/{}/adjacent", draft.id))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .get(&format!("/api/v1/posts/{}/like-status", draft.id))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .post(&format!("/api/v1/posts/{}/like", draft.id))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .get(&format!("/api/v1/posts/{}/comments", draft.id))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .post(&format!("/api/v1/posts/{}/comments", draft.id))
        .json(&json!({
            "content": "must not persist",
            "parent_id": null,
            "author_name": "anonymous",
            "author_email": null
        }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .post("/api/v1/read-logs")
        .json(&json!({
            "post_id": draft.id,
            "duration_seconds": 12,
            "referrer": "integration-test"
        }))
        .await
        .assert_status(StatusCode::NOT_FOUND);

    let counts = app.db_counts_for_post(draft.id).await?;
    assert_eq!(counts.likes, 0);
    assert_eq!(counts.comments, 0);
    assert_eq!(counts.read_logs, 0);

    Ok(())
}
