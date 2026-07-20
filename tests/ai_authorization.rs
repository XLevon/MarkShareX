mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};

fn data_id(response: &axum_test::TestResponse) -> i32 {
    response.json::<Value>()["data"]["id"]
        .as_i64()
        .expect("response data should contain integer id") as i32
}

fn session_ids(response: &axum_test::TestResponse) -> Vec<i64> {
    response.json::<Value>()["data"]
        .as_array()
        .expect("session list should contain data array")
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect()
}

#[tokio::test]
async fn ai_management_requires_the_current_database_admin_role() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("ai-visitor", "visitor").await?;
    let author = app.create_user("ai-author", "author").await?;
    let sub_admin = app.create_user("ai-sub-admin", "sub_admin").await?;
    let admin = app.create_user("ai-admin", "admin").await?;

    app.server
        .get("/api/v1/admin/ai/providers")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .get("/api/v1/admin/ai/providers")
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    app.server
        .get("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();

    app.set_user_role(author.id, "admin").await?;
    app.server
        .get("/api/v1/admin/ai/providers")
        .authorization_bearer(&author.token)
        .await
        .assert_status_ok();

    app.set_user_role(admin.id, "author").await?;
    app.server
        .get("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);

    Ok(())
}

#[tokio::test]
async fn ai_sessions_are_owner_only_through_http_for_every_role() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let owner = app.create_user("session-owner", "author").await?;
    let other = app.create_user("session-other", "visitor").await?;
    let sub_admin = app.create_user("session-sub-admin", "sub_admin").await?;
    let admin = app.create_user("session-admin", "admin").await?;

    let owner_created = app
        .server
        .post("/api/v1/admin/ai/sessions")
        .authorization_bearer(&owner.token)
        .json(&json!({"title": "owner secret session", "agent_config_id": null}))
        .await;
    owner_created.assert_status_ok();
    let owner_session_id = data_id(&owner_created);
    assert_eq!(
        owner_created.json::<Value>()["data"]["user_id"].as_i64(),
        Some(owner.id as i64)
    );

    let other_created = app
        .server
        .post("/api/v1/admin/ai/sessions")
        .authorization_bearer(&other.token)
        .json(&json!({"title": "other session", "agent_config_id": null}))
        .await;
    other_created.assert_status_ok();
    let other_session_id = data_id(&other_created);

    let owner_list = app
        .server
        .get("/api/v1/admin/ai/sessions")
        .authorization_bearer(&owner.token)
        .await;
    owner_list.assert_status_ok();
    assert_eq!(session_ids(&owner_list), vec![owner_session_id as i64]);

    let other_list = app
        .server
        .get("/api/v1/admin/ai/sessions")
        .authorization_bearer(&other.token)
        .await;
    other_list.assert_status_ok();
    assert_eq!(session_ids(&other_list), vec![other_session_id as i64]);

    for user in [&other, &sub_admin, &admin] {
        app.server
            .get(&format!("/api/v1/admin/ai/sessions/{owner_session_id}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::NOT_FOUND);
    }
    app.server
        .get(&format!("/api/v1/admin/ai/sessions/{owner_session_id}"))
        .authorization_bearer(&owner.token)
        .await
        .assert_status_ok();

    app.server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&other.token)
        .json(&json!({
            "message": "/new unauthorized",
            "history": [],
            "agent_config_id": null,
            "session_id": owner_session_id,
            "in_admin": false
        }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    assert!(app.ai_session_exists(owner_session_id).await?);

    for user in [&other, &sub_admin, &admin] {
        app.server
            .delete(&format!("/api/v1/admin/ai/sessions/{owner_session_id}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::NOT_FOUND);
        assert!(app.ai_session_exists(owner_session_id).await?);
    }

    app.server
        .delete(&format!("/api/v1/admin/ai/sessions/{owner_session_id}"))
        .authorization_bearer(&owner.token)
        .await
        .assert_status_ok();
    assert!(!app.ai_session_exists(owner_session_id).await?);

    Ok(())
}
