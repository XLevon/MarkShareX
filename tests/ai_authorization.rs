mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};

fn data_id(response: &axum_test::TestResponse) -> i32 {
    response.json::<Value>()["data"]["id"]
        .as_i64()
        .expect("response data should contain integer id") as i32
}

fn chat_session_id(response: &axum_test::TestResponse) -> i32 {
    response.json::<Value>()["data"]["session_id"]
        .as_i64()
        .expect("chat response should contain integer session_id") as i32
}

fn session_ids(response: &axum_test::TestResponse) -> Vec<i64> {
    response.json::<Value>()["data"]
        .as_array()
        .expect("session list should contain data array")
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect()
}

fn sorted_session_ids(response: &axum_test::TestResponse) -> Vec<i64> {
    let mut ids = session_ids(response);
    ids.sort_unstable();
    ids
}

fn session_items(response: &axum_test::TestResponse) -> Vec<Value> {
    response.json::<Value>()["data"]
        .as_array()
        .expect("session list should contain data array")
        .clone()
}

#[tokio::test]
async fn ai_session_list_uses_current_database_role_and_admin_only_prefixes() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let owner = app.create_user("list-owner", "author").await?;
    let other = app.create_user("list-other", "visitor").await?;
    let sub_admin = app.create_user("list-sub-admin", "sub_admin").await?;
    let admin = app.create_user("list-admin", "admin").await?;

    let mut all_ids = Vec::new();
    for (user, title) in [
        (&owner, "owner session"),
        (&other, "other session"),
        (&sub_admin, "sub admin session"),
    ] {
        let created = app
            .server
            .post("/api/v1/admin/ai/sessions")
            .authorization_bearer(&user.token)
            .json(&json!({"title": title, "agent_config_id": null}))
            .await;
        created.assert_status_ok();
        all_ids.push(data_id(&created) as i64);
    }
    all_ids.sort_unstable();

    let admin_list = app
        .server
        .get("/api/v1/admin/ai/sessions")
        .authorization_bearer(&admin.token)
        .await;
    admin_list.assert_status_ok();
    assert_eq!(sorted_session_ids(&admin_list), all_ids);
    assert!(session_items(&admin_list)
        .iter()
        .all(|item| item["user_display_name"].is_string()));

    for (user, expected_id) in [(&owner, all_ids[0]), (&sub_admin, all_ids[2])] {
        let personal_list = app
            .server
            .get("/api/v1/admin/ai/sessions")
            .authorization_bearer(&user.token)
            .await;
        personal_list.assert_status_ok();
        assert_eq!(session_ids(&personal_list), vec![expected_id]);
        assert!(session_items(&personal_list)
            .iter()
            .all(|item| item["user_display_name"].is_null()));
    }

    app.set_user_role(owner.id, "admin").await?;
    let promoted_with_old_token = app
        .server
        .get("/api/v1/admin/ai/sessions")
        .authorization_bearer(&owner.token)
        .await;
    promoted_with_old_token.assert_status_ok();
    assert_eq!(sorted_session_ids(&promoted_with_old_token), all_ids);

    app.set_user_role(admin.id, "author").await?;
    let demoted_with_old_token = app
        .server
        .get("/api/v1/admin/ai/sessions")
        .authorization_bearer(&admin.token)
        .await;
    demoted_with_old_token.assert_status_ok();
    assert!(session_ids(&demoted_with_old_token).is_empty());

    Ok(())
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
async fn ai_session_actions_follow_the_current_role_matrix_without_chat_side_effects(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let owner = app.create_user("session-owner", "author").await?;
    let other = app.create_user("session-other", "visitor").await?;
    let sub_admin = app.create_user("session-sub-admin", "sub_admin").await?;
    let admin = app.create_user("session-admin", "admin").await?;

    let mut session_ids = Vec::new();
    for title in [
        "read target",
        "delete target",
        "chat target",
        "dynamic target",
    ] {
        let created = app
            .server
            .post("/api/v1/admin/ai/sessions")
            .authorization_bearer(&owner.token)
            .json(&json!({"title": title, "agent_config_id": null}))
            .await;
        created.assert_status_ok();
        assert_eq!(
            created.json::<Value>()["data"]["user_id"].as_i64(),
            Some(owner.id as i64)
        );
        session_ids.push(data_id(&created));
    }
    let read_target = session_ids[0];
    let delete_target = session_ids[1];
    let chat_target = session_ids[2];
    let dynamic_target = session_ids[3];

    app.server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&owner.token)
        .json(&json!({
            "message": "/help",
            "history": [],
            "agent_config_id": null,
            "session_id": delete_target,
            "in_admin": false
        }))
        .await
        .assert_status_ok();
    let delete_session_before = app.ai_session_row(delete_target).await?;
    let delete_messages_before = app.ai_session_messages(delete_target).await?;
    let delete_global_sessions_before = app.ai_session_count().await?;
    let delete_global_messages_before = app.ai_message_count().await?;

    for user in [&other, &sub_admin] {
        app.server
            .get(&format!("/api/v1/admin/ai/sessions/{read_target}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::NOT_FOUND);
        app.server
            .delete(&format!("/api/v1/admin/ai/sessions/{delete_target}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::NOT_FOUND);
    }
    assert_eq!(
        app.ai_session_row(delete_target).await?,
        delete_session_before
    );
    assert_eq!(
        app.ai_session_messages(delete_target).await?,
        delete_messages_before
    );
    assert_eq!(app.ai_session_count().await?, delete_global_sessions_before);
    assert_eq!(app.ai_message_count().await?, delete_global_messages_before);

    app.server
        .get(&format!("/api/v1/admin/ai/sessions/{read_target}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();
    app.server
        .delete(&format!("/api/v1/admin/ai/sessions/{delete_target}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();
    assert!(!app.ai_session_exists(delete_target).await?);

    let chat_session_before = app.ai_session_row(chat_target).await?;
    let chat_messages_before = app.ai_session_messages(chat_target).await?;
    let chat_global_sessions_before = app.ai_session_count().await?;
    let chat_global_messages_before = app.ai_message_count().await?;
    assert!(chat_messages_before.is_empty());
    for user in [&other, &sub_admin, &admin] {
        for message in ["unauthorized continuation", "/new unauthorized"] {
            app.server
                .post("/api/v1/admin/ai/chat")
                .authorization_bearer(&user.token)
                .json(&json!({
                    "message": message,
                    "history": [],
                    "agent_config_id": null,
                    "session_id": chat_target,
                    "in_admin": false
                }))
                .await
                .assert_status(StatusCode::NOT_FOUND);
        }
    }
    assert_eq!(app.ai_session_row(chat_target).await?, chat_session_before);
    assert_eq!(
        app.ai_session_messages(chat_target).await?,
        chat_messages_before
    );
    assert_eq!(app.ai_session_count().await?, chat_global_sessions_before);
    assert_eq!(app.ai_message_count().await?, chat_global_messages_before);

    app.server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&owner.token)
        .json(&json!({
            "message": "/help",
            "history": [],
            "agent_config_id": null,
            "session_id": chat_target,
            "in_admin": false
        }))
        .await
        .assert_status_ok();
    assert_eq!(app.ai_session_message_count(chat_target).await?, 1);

    let dynamic_session_before = app.ai_session_row(dynamic_target).await?;
    let dynamic_messages_before = app.ai_session_messages(dynamic_target).await?;
    let dynamic_global_sessions_before = app.ai_session_count().await?;
    let dynamic_global_messages_before = app.ai_message_count().await?;
    app.set_user_role(admin.id, "author").await?;
    app.server
        .get(&format!("/api/v1/admin/ai/sessions/{dynamic_target}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .delete(&format!("/api/v1/admin/ai/sessions/{dynamic_target}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    assert_eq!(
        app.ai_session_row(dynamic_target).await?,
        dynamic_session_before
    );
    assert_eq!(
        app.ai_session_messages(dynamic_target).await?,
        dynamic_messages_before
    );
    assert_eq!(
        app.ai_session_count().await?,
        dynamic_global_sessions_before
    );
    assert_eq!(
        app.ai_message_count().await?,
        dynamic_global_messages_before
    );

    app.set_user_role(other.id, "admin").await?;
    app.server
        .get(&format!("/api/v1/admin/ai/sessions/{dynamic_target}"))
        .authorization_bearer(&other.token)
        .await
        .assert_status_ok();
    app.server
        .delete(&format!("/api/v1/admin/ai/sessions/{dynamic_target}"))
        .authorization_bearer(&other.token)
        .await
        .assert_status_ok();
    assert!(!app.ai_session_exists(dynamic_target).await?);

    Ok(())
}

#[tokio::test]
async fn new_command_creates_one_owned_session_and_returns_that_session() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let owner = app.create_user("new-command-owner", "author").await?;

    let sessions_before = app.ai_session_count().await?;
    let messages_before = app.ai_message_count().await?;
    let without_existing = app
        .server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&owner.token)
        .json(&json!({
            "message": "/new first session",
            "history": [],
            "agent_config_id": null,
            "session_id": null,
            "in_admin": false
        }))
        .await;
    without_existing.assert_status_ok();
    let first_id = chat_session_id(&without_existing);
    let first = app
        .ai_session_row(first_id)
        .await?
        .expect("returned session should exist");
    assert_eq!(first.user_id, owner.id);
    assert_eq!(first.title, "first session");
    assert_eq!(app.ai_session_count().await?, sessions_before + 1);
    assert_eq!(app.ai_message_count().await?, messages_before + 1);
    assert_eq!(app.ai_session_message_count(first_id).await?, 1);

    let first_before = app.ai_session_row(first_id).await?;
    let first_messages_before = app.ai_session_messages(first_id).await?;
    let sessions_before_second = app.ai_session_count().await?;
    let messages_before_second = app.ai_message_count().await?;
    let with_existing = app
        .server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&owner.token)
        .json(&json!({
            "message": "/new second session",
            "history": [],
            "agent_config_id": null,
            "session_id": first_id,
            "in_admin": false
        }))
        .await;
    with_existing.assert_status_ok();
    let second_id = chat_session_id(&with_existing);
    assert_ne!(second_id, first_id);
    let second = app
        .ai_session_row(second_id)
        .await?
        .expect("new returned session should exist");
    assert_eq!(second.user_id, owner.id);
    assert_eq!(second.title, "second session");
    assert_eq!(app.ai_session_message_count(second_id).await?, 1);
    assert_eq!(app.ai_session_count().await?, sessions_before_second + 1);
    assert_eq!(app.ai_message_count().await?, messages_before_second + 1);
    assert_eq!(app.ai_session_row(first_id).await?, first_before);
    assert_eq!(
        app.ai_session_messages(first_id).await?,
        first_messages_before
    );

    Ok(())
}
