mod common;

use axum::http::StatusCode;
use axum_test::multipart::{MultipartForm, Part};
use common::TestApp;
use marksharex::models::entity::{
    ai_agent_config, ai_model, ai_provider, ai_skill, ai_task, ai_task_log, ai_tool,
    article_statuses, article_types, author_applications, categories, changelog, comments, files,
    guestbook, login_logs, network_resources, news, post_tags, posts, read_logs, refresh_tokens,
    tags, users,
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryOrder, Set, Statement,
};
use serde_json::{json, Value};
use std::path::Path;

fn directory_entries(path: &Path) -> anyhow::Result<Vec<String>> {
    let mut entries = std::fs::read_dir(path)?
        .map(|entry| entry.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

async fn table_count(app: &TestApp, table: &str) -> anyhow::Result<i64> {
    let row = app
        .db
        .query_one(Statement::from_string(
            app.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM {table}"),
        ))
        .await?
        .expect("COUNT must return one row");
    Ok(row.try_get_by_index::<i64>(0)?)
}

/// Full HTTP role matrix for six AI admin write endpoints.
/// Assertions: anonymous→401; visitor/author/sub_admin→403; admin→200.
/// After every denied attempt the DB row counts remain unchanged.
async fn row_counts(app: &TestApp) -> anyhow::Result<Vec<u64>> {
    Ok(vec![
        ai_provider::Entity::find().count(&app.db).await?,
        ai_model::Entity::find().count(&app.db).await?,
        ai_agent_config::Entity::find().count(&app.db).await?,
        ai_skill::Entity::find().count(&app.db).await?,
        ai_tool::Entity::find().count(&app.db).await?,
        ai_task::Entity::find().count(&app.db).await?,
        ai_task_log::Entity::find().count(&app.db).await?,
    ])
}

#[tokio::test]
async fn ai_admin_writes_enforce_role_matrix_with_zero_side_effects() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("aiw-visitor", "visitor").await?;
    let author = app.create_user("aiw-author", "author").await?;
    let sub_admin = app.create_user("aiw-sub-admin", "sub_admin").await?;
    let admin = app.create_user("aiw-admin", "admin").await?;

    // ── Create prerequisite: a provider (admin-only) ──
    let counts_before = row_counts(&app).await?;
    let created = app
        .server
        .post("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "name": "matrix-prov",
            "provider_type": "openai",
            "base_url": "https://api.openai.com",
            "api_key": "sk-test"
        }))
        .await;
    created.assert_status_ok();
    let provider_id = created.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let counts_after_prov = row_counts(&app).await?;
    assert_eq!(counts_after_prov[0], counts_before[0] + 1);

    // ── Test each AI write endpoint against every role ──

    // ---- Model ----
    let model_payload = json!({"provider_id": provider_id, "name": "gpt-4o-matrix"});
    let before = row_counts(&app).await?;
    // anonymous
    app.server
        .post("/api/v1/admin/ai/models")
        .json(&model_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .post("/api/v1/admin/ai/models")
            .authorization_bearer(&user.token)
            .json(&model_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(row_counts(&app).await?, before);
    // admin succeeds
    let model_resp = app
        .server
        .post("/api/v1/admin/ai/models")
        .authorization_bearer(&admin.token)
        .json(&model_payload)
        .await;
    model_resp.assert_status_ok();
    let model_id = model_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let after_model = row_counts(&app).await?;
    assert_eq!(after_model[1], before[1] + 1);

    // ---- Agent config ----
    let agent_payload = json!({"name": "matrix-agent", "model_id": model_id});
    let before = row_counts(&app).await?;
    app.server
        .post("/api/v1/admin/ai/agent-configs")
        .json(&agent_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .post("/api/v1/admin/ai/agent-configs")
            .authorization_bearer(&user.token)
            .json(&agent_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(row_counts(&app).await?, before);
    let agent_resp = app
        .server
        .post("/api/v1/admin/ai/agent-configs")
        .authorization_bearer(&admin.token)
        .json(&agent_payload)
        .await;
    agent_resp.assert_status_ok();
    let agent_id = agent_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let after_agent = row_counts(&app).await?;
    assert_eq!(after_agent[2], before[2] + 1);

    // ---- Skill ----
    let skill_payload = json!({
        "name": "matrix-skill",
        "description": "test",
        "content": "echo hello",
        "output_format": "text"
    });
    let before = row_counts(&app).await?;
    app.server
        .post("/api/v1/admin/ai/skills")
        .json(&skill_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .post("/api/v1/admin/ai/skills")
            .authorization_bearer(&user.token)
            .json(&skill_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(row_counts(&app).await?, before);
    let skill_resp = app
        .server
        .post("/api/v1/admin/ai/skills")
        .authorization_bearer(&admin.token)
        .json(&skill_payload)
        .await;
    skill_resp.assert_status_ok();
    let skill_id = skill_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let after_skill = row_counts(&app).await?;
    assert_eq!(after_skill[3], before[3] + 1);

    // ---- Tool ----
    let tool_payload = json!({
        "name": "matrix-tool",
        "function_name": "matrix_tool_fn",
        "parameters_schema": "{}"
    });
    let before = row_counts(&app).await?;
    app.server
        .post("/api/v1/admin/ai/tools")
        .json(&tool_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .post("/api/v1/admin/ai/tools")
            .authorization_bearer(&user.token)
            .json(&tool_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(row_counts(&app).await?, before);
    let tool_resp = app
        .server
        .post("/api/v1/admin/ai/tools")
        .authorization_bearer(&admin.token)
        .json(&tool_payload)
        .await;
    tool_resp.assert_status_ok();
    let tool_id = tool_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let after_tool = row_counts(&app).await?;
    assert_eq!(after_tool[4], before[4] + 1);

    // ---- Task ----
    let task_payload = json!({
        "skill_id": skill_id,
        "cron_expr": "0 0 * * *",
        "name": "matrix-task"
    });
    let before = row_counts(&app).await?;
    app.server
        .post("/api/v1/admin/ai/tasks")
        .json(&task_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .post("/api/v1/admin/ai/tasks")
            .authorization_bearer(&user.token)
            .json(&task_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(row_counts(&app).await?, before);
    let task_resp = app
        .server
        .post("/api/v1/admin/ai/tasks")
        .authorization_bearer(&admin.token)
        .json(&task_payload)
        .await;
    task_resp.assert_status_ok();
    let task_id = task_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let after_task = row_counts(&app).await?;
    assert_eq!(after_task[5], before[5] + 1);

    let task_log = ai_task_log::ActiveModel {
        task_id: Set(task_id),
        status: Set("completed".to_string()),
        steps: Set("[]".to_string()),
        final_reply: Set("existing task log".to_string()),
        error: Set(None),
        created_at: Set(marksharex::utils::now_local()),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    let task_log_before = ai_task_log::Entity::find_by_id(task_log.id)
        .one(&app.db)
        .await?
        .unwrap();
    assert!(marksharex::services::ai_trace::trace_get(task_id).is_none());

    // Every existing AI admin side-effecting route must reject before mutation,
    // external provider calls, task spawning, trace/log writes, or deletion.
    let before_denied = row_counts(&app).await?;
    for user in [&visitor, &author, &sub_admin] {
        let token = &user.token;
        for (path, payload) in [
            (
                format!("/api/v1/admin/ai/providers/{provider_id}"),
                json!({"name":"blocked-provider","provider_type":"openai","base_url":"https://api.openai.com","api_key":"blocked"}),
            ),
            (
                format!("/api/v1/admin/ai/models/{model_id}"),
                model_payload.clone(),
            ),
            (
                format!("/api/v1/admin/ai/agent-configs/{agent_id}"),
                agent_payload.clone(),
            ),
            (
                format!("/api/v1/admin/ai/skills/{skill_id}"),
                skill_payload.clone(),
            ),
            (
                format!("/api/v1/admin/ai/tools/{tool_id}"),
                tool_payload.clone(),
            ),
            (
                format!("/api/v1/admin/ai/tasks/{task_id}"),
                task_payload.clone(),
            ),
        ] {
            app.server
                .put(&path)
                .authorization_bearer(token)
                .json(&payload)
                .await
                .assert_status(StatusCode::FORBIDDEN);
            app.server
                .delete(&path)
                .authorization_bearer(token)
                .await
                .assert_status(StatusCode::FORBIDDEN);
        }

        app.server
            .post(&format!("/api/v1/admin/ai/providers/{provider_id}/test"))
            .authorization_bearer(token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        app.server
            .post(&format!("/api/v1/admin/ai/tasks/{task_id}/run"))
            .authorization_bearer(token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        app.server
            .delete(&format!(
                "/api/v1/admin/ai/tasks/{task_id}/logs/{}",
                task_log.id
            ))
            .authorization_bearer(token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(row_counts(&app).await?, before_denied);
    assert_eq!(
        ai_task_log::Entity::find_by_id(task_log.id)
            .one(&app.db)
            .await?
            .unwrap(),
        task_log_before
    );
    // trace_start precedes tokio::spawn/provider work in run_task; remaining absent
    // after a wait proves the denied request never entered any of those effects.
    assert!(marksharex::services::ai_trace::trace_get(task_id).is_none());

    Ok(())
}

/// Non-AI write endpoints: settings, users, categories, tags, files, comments,
/// network-resources, news — each tested across the full role matrix with
/// zero-side-effect assertions.
#[tokio::test]
async fn admin_resource_writes_enforce_role_matrix_with_zero_side_effects() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("arw-visitor", "visitor").await?;
    let author = app.create_user("arw-author", "author").await?;
    let sub_admin = app.create_user("arw-sub-admin", "sub_admin").await?;
    let admin = app.create_user("arw-admin", "admin").await?;

    // ── Settings (PUT) ──
    let settings_before: Value = app.server.get("/api/v1/settings").await.json();
    let settings_rows_before = marksharex::models::entity::settings::Entity::find()
        .order_by_asc(marksharex::models::entity::settings::Column::Key)
        .all(&app.db)
        .await?;
    app.server
        .put("/api/v1/settings")
        .json(&settings_before["data"])
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .put("/api/v1/settings")
            .authorization_bearer(&user.token)
            .json(&settings_before["data"])
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(
        marksharex::models::entity::settings::Entity::find()
            .order_by_asc(marksharex::models::entity::settings::Column::Key)
            .all(&app.db)
            .await?,
        settings_rows_before
    );
    app.server
        .put("/api/v1/settings")
        .authorization_bearer(&admin.token)
        .json(&settings_before["data"])
        .await
        .assert_status_ok();

    // ── Users list/create — privileged, with admin tier preserved ──
    app.server
        .get("/api/v1/admin/users")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .get("/api/v1/admin/users")
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    for user in [&sub_admin, &admin] {
        app.server
            .get("/api/v1/admin/users")
            .authorization_bearer(&user.token)
            .await
            .assert_status_ok();
    }

    let before_users = users::Entity::find()
        .order_by_asc(users::Column::Id)
        .all(&app.db)
        .await?;
    let denied_user = json!({
        "username": "matrix-denied-user",
        "email": "matrix-denied@example.com",
        "password": "Test1234!",
        "role": "author"
    });
    app.server
        .post("/api/v1/admin/users")
        .json(&denied_user)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .post("/api/v1/admin/users")
            .authorization_bearer(&user.token)
            .json(&denied_user)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            users::Entity::find()
                .order_by_asc(users::Column::Id)
                .all(&app.db)
                .await?,
            before_users
        );
    }
    for (user, username, role) in [
        (&sub_admin, "matrix-created-author", "author"),
        (&admin, "matrix-created-sub-admin", "sub_admin"),
    ] {
        app.server
            .post("/api/v1/admin/users")
            .authorization_bearer(&user.token)
            .json(&json!({
                "username": username,
                "email": format!("{username}@example.com"),
                "password": "Test1234!",
                "role": role
            }))
            .await
            .assert_status_ok();
    }
    assert_eq!(
        users::Entity::find().count(&app.db).await?,
        before_users.len() as u64 + 2
    );

    // ── Files delete — admin-only; denied paths preserve DB row and bytes ──
    let filename = "matrix-protected-file.txt";
    let file_bytes = b"matrix-protected-content";
    let file_id = app
        .insert_file_record(admin.id, filename, "text/plain")
        .await?;
    let file_path = app.upload_dir().join(filename);
    tokio::fs::write(&file_path, file_bytes).await?;
    let before_file = files::Entity::find_by_id(file_id)
        .one(&app.db)
        .await?
        .unwrap();
    app.server
        .delete(&format!("/api/v1/files/{file_id}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .delete(&format!("/api/v1/files/{file_id}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            files::Entity::find_by_id(file_id)
                .one(&app.db)
                .await?
                .unwrap(),
            before_file
        );
        assert_eq!(tokio::fs::read(&file_path).await?, file_bytes);
    }
    app.server
        .delete(&format!("/api/v1/files/{file_id}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();
    assert!(files::Entity::find_by_id(file_id)
        .one(&app.db)
        .await?
        .is_none());
    assert!(!file_path.exists());

    // ── Categories (POST /api/v1/categories) — authenticated owner ──
    let cat_payload = json!({"name": "matrix-cat", "slug": "matrix-cat"});
    let before = categories::Entity::find().count(&app.db).await?;
    app.server
        .post("/api/v1/categories")
        .json(&cat_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin, &admin] {
        app.server
            .post("/api/v1/categories")
            .authorization_bearer(&user.token)
            .json(&cat_payload)
            .await
            .assert_status_ok();
    }
    let after_cat = categories::Entity::find().count(&app.db).await?;
    assert!(after_cat > before);

    // ── Tags (POST /api/v1/tags) — currently AuthUser (any authenticated) ──
    // Finding: tags controller uses AuthUser not PrivilegedUser;
    // any authenticated user can create tags.
    let tag_payload = json!({"name": "matrix-tag", "slug": "matrix-tag"});
    let before = marksharex::models::entity::tags::Entity::find()
        .count(&app.db)
        .await?;
    app.server
        .post("/api/v1/tags")
        .json(&tag_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin, &admin] {
        app.server
            .post("/api/v1/tags")
            .authorization_bearer(&user.token)
            .json(&tag_payload)
            .await
            .assert_status_ok();
    }
    assert!(
        marksharex::models::entity::tags::Entity::find()
            .count(&app.db)
            .await?
            > before
    );

    // ── Comments admin update (PUT /api/v1/admin/comments/:id) — privileged ──
    // First create a post and a comment as an author.
    let post_author = app.create_user("arw-post-author", "author").await?;
    let post = app
        .create_post(&post_author, "matrix-comment-post", "published")
        .await?;
    let comment_resp = app
        .server
        .post(&format!("/api/v1/posts/{}/comments", post.id))
        .authorization_bearer(&post_author.token)
        .json(&json!({"content": "matrix-comment-body"}))
        .await;
    comment_resp.assert_status_ok();
    let comment_id = comment_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let approve = json!({"status": "approved"});
    let before_row = comments::Entity::find_by_id(comment_id)
        .one(&app.db)
        .await?
        .unwrap();
    let before_count = comments::Entity::find().count(&app.db).await?;
    app.server
        .put(&format!("/api/v1/admin/comments/{comment_id}"))
        .json(&approve)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .put(&format!("/api/v1/admin/comments/{comment_id}"))
            .authorization_bearer(&user.token)
            .json(&approve)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            comments::Entity::find_by_id(comment_id)
                .one(&app.db)
                .await?
                .unwrap(),
            before_row
        );
        assert_eq!(comments::Entity::find().count(&app.db).await?, before_count);
    }
    for user in [&sub_admin, &admin] {
        app.server
            .put(&format!("/api/v1/admin/comments/{comment_id}"))
            .authorization_bearer(&user.token)
            .json(&approve)
            .await
            .assert_status_ok();
    }

    // ── Network resources (POST /api/v1/network-resources) — privileged ──
    let nr_payload = json!({
        "url": "https://matrix-test.example.com/denied",
        "label": "matrix-resource"
    });
    let before = network_resources::Entity::find().all(&app.db).await?;
    app.server
        .post("/api/v1/network-resources")
        .json(&nr_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .post("/api/v1/network-resources")
            .authorization_bearer(&user.token)
            .json(&nr_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            network_resources::Entity::find().all(&app.db).await?,
            before
        );
    }
    for (user, suffix) in [(&sub_admin, "sub"), (&admin, "admin")] {
        app.server
            .post("/api/v1/network-resources")
            .authorization_bearer(&user.token)
            .json(&json!({
                "url": format!("https://matrix-test.example.com/{suffix}"),
                "label": suffix
            }))
            .await
            .assert_status_ok();
    }
    assert_eq!(
        network_resources::Entity::find().count(&app.db).await?,
        before.len() as u64 + 2
    );

    // ── News (POST /api/v1/admin/news) — privileged ──
    let news_payload = json!({
        "title": "matrix-news-denied",
        "content": "test news content",
        "topic_type": "bug",
        "status": "published",
        "lang": "zh"
    });
    let before = news::Entity::find().all(&app.db).await?;
    app.server
        .post("/api/v1/admin/news")
        .json(&news_payload)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .post("/api/v1/admin/news")
            .authorization_bearer(&user.token)
            .json(&news_payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(news::Entity::find().all(&app.db).await?, before);
    }
    for (user, suffix) in [(&sub_admin, "sub"), (&admin, "admin")] {
        app.server
            .post("/api/v1/admin/news")
            .authorization_bearer(&user.token)
            .json(&json!({
                "title": format!("matrix-news-{suffix}"),
                "content": "test news content",
                "topic_type": "bug",
                "status": "published",
                "lang": "zh"
            }))
            .await
            .assert_status_ok();
    }
    assert_eq!(
        news::Entity::find().count(&app.db).await?,
        before.len() as u64 + 2
    );

    // ── Remaining high-risk writes: denied before any DB/file/process effect ──
    // Seed every user-delete cascade branch for the target user before the denied request.
    let target_user_id = post_author.id;
    app.server
        .post(&format!("/api/v1/posts/{}/like", post.id))
        .authorization_bearer(&post_author.token)
        .await
        .assert_status_ok();
    app.server
        .post("/api/v1/read-logs")
        .authorization_bearer(&post_author.token)
        .json(&json!({"post_id": post.id, "duration_seconds": 9}))
        .await
        .assert_status_ok();
    app.server
        .post("/api/v1/apply")
        .authorization_bearer(&post_author.token)
        .json(&json!({"reason":"cascade fixture","content":"cascade fixture content"}))
        .await
        .assert_status_ok();
    let cascade_category = app
        .server
        .post("/api/v1/categories")
        .authorization_bearer(&post_author.token)
        .json(&json!({"name":"cascade-user-category"}))
        .await;
    cascade_category.assert_status_ok();
    let cascade_tag = app
        .server
        .post("/api/v1/tags")
        .authorization_bearer(&post_author.token)
        .json(&json!({"name":"cascade-user-tag"}))
        .await;
    cascade_tag.assert_status_ok();
    let cascade_tag_id = cascade_tag.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    post_tags::ActiveModel {
        post_id: Set(post.id),
        tag_id: Set(cascade_tag_id),
    }
    .insert(&app.db)
    .await?;
    let cascade_filename = "cascade-user-file.png";
    let cascade_file_id = app
        .insert_file_record(target_user_id, cascade_filename, "image/png")
        .await?;
    let cascade_file_path = app.upload_dir().join(cascade_filename);
    let cascade_file_bytes = b"cascade-user-file-bytes".to_vec();
    std::fs::write(&cascade_file_path, &cascade_file_bytes)?;
    let now = marksharex::utils::now_local();
    login_logs::ActiveModel {
        user_id: Set(Some(target_user_id)),
        username: Set("arw-post-author".to_string()),
        ip_address: Set(Some("127.0.0.1".to_string())),
        user_agent: Set(Some("cascade-test".to_string())),
        device_type: Set(Some("desktop".to_string())),
        login_method: Set("password".to_string()),
        success: Set(true),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    refresh_tokens::ActiveModel {
        user_id: Set(target_user_id),
        token: Set("cascade-refresh-token".to_string()),
        expires_at: Set(now + chrono::Duration::hours(1)),
        revoked: Set(false),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;

    let users_before_all = users::Entity::find()
        .order_by_asc(users::Column::Id)
        .all(&app.db)
        .await?;
    let posts_before_user_delete = posts::Entity::find().all(&app.db).await?;
    let comments_before_user_delete = comments::Entity::find().all(&app.db).await?;
    let files_before_user_delete = files::Entity::find().all(&app.db).await?;
    let applications_before_user_delete = author_applications::Entity::find().all(&app.db).await?;
    let login_logs_before_user_delete = login_logs::Entity::find().all(&app.db).await?;
    let refresh_tokens_before_user_delete = refresh_tokens::Entity::find().all(&app.db).await?;
    let categories_before_user_delete = categories::Entity::find().all(&app.db).await?;
    let tags_before_user_delete = tags::Entity::find().all(&app.db).await?;
    let post_tags_before_user_delete = post_tags::Entity::find().all(&app.db).await?;
    let read_logs_before_user_delete = read_logs::Entity::find().all(&app.db).await?;
    let likes_before_user_delete = table_count(&app, "likes").await?;
    for (path, payload) in [
        (
            format!("/api/v1/admin/users/{target_user_id}"),
            json!({"display_name": "blocked"}),
        ),
        (
            format!("/api/v1/admin/users/{target_user_id}/status"),
            json!({"status": "banned"}),
        ),
        (
            format!("/api/v1/admin/users/{target_user_id}/role"),
            json!({"role": "admin"}),
        ),
    ] {
        app.server
            .put(&path)
            .authorization_bearer(&visitor.token)
            .json(&payload)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    app.server
        .put(&format!(
            "/api/v1/admin/users/{target_user_id}/reset-password"
        ))
        .authorization_bearer(&sub_admin.token)
        .json(&json!({"password": "Blocked123!"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/admin/users/{target_user_id}"))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        users::Entity::find()
            .order_by_asc(users::Column::Id)
            .all(&app.db)
            .await?,
        users_before_all
    );
    assert_eq!(
        posts::Entity::find().all(&app.db).await?,
        posts_before_user_delete
    );
    assert_eq!(
        comments::Entity::find().all(&app.db).await?,
        comments_before_user_delete
    );
    assert_eq!(
        files::Entity::find().all(&app.db).await?,
        files_before_user_delete
    );
    assert!(files::Entity::find_by_id(cascade_file_id)
        .one(&app.db)
        .await?
        .is_some());
    assert_eq!(std::fs::read(&cascade_file_path)?, cascade_file_bytes);
    assert_eq!(
        author_applications::Entity::find().all(&app.db).await?,
        applications_before_user_delete
    );
    assert_eq!(
        login_logs::Entity::find().all(&app.db).await?,
        login_logs_before_user_delete
    );
    assert_eq!(
        refresh_tokens::Entity::find().all(&app.db).await?,
        refresh_tokens_before_user_delete
    );
    assert_eq!(
        categories::Entity::find().all(&app.db).await?,
        categories_before_user_delete
    );
    assert_eq!(
        tags::Entity::find().all(&app.db).await?,
        tags_before_user_delete
    );
    assert_eq!(
        post_tags::Entity::find().all(&app.db).await?,
        post_tags_before_user_delete
    );
    assert_eq!(
        read_logs::Entity::find().all(&app.db).await?,
        read_logs_before_user_delete
    );
    assert_eq!(table_count(&app, "likes").await?, likes_before_user_delete);

    let nr_target = network_resources::Entity::find()
        .order_by_desc(network_resources::Column::Id)
        .one(&app.db)
        .await?
        .unwrap();
    let nr_before_all = network_resources::Entity::find()
        .order_by_asc(network_resources::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .put(&format!("/api/v1/network-resources/{}", nr_target.id))
        .authorization_bearer(&visitor.token)
        .json(&json!({"url":"https://blocked.example.com","label":"blocked"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/network-resources/{}", nr_target.id))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .post("/api/v1/network-resources/ensure")
        .authorization_bearer(&visitor.token)
        .json(&json!({"url":"https://blocked.example.com"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        network_resources::Entity::find()
            .order_by_asc(network_resources::Column::Id)
            .all(&app.db)
            .await?,
        nr_before_all
    );

    let news_target = news::Entity::find()
        .order_by_desc(news::Column::Id)
        .one(&app.db)
        .await?
        .unwrap();
    let news_before_all = news::Entity::find()
        .order_by_asc(news::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .put(&format!("/api/v1/admin/news/{}", news_target.id))
        .authorization_bearer(&visitor.token)
        .json(&json!({"title":"blocked"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/admin/news/{}", news_target.id))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .post("/api/v1/admin/news/batch-delete")
        .authorization_bearer(&visitor.token)
        .json(&json!({"ids":[news_target.id]}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        news::Entity::find()
            .order_by_asc(news::Column::Id)
            .all(&app.db)
            .await?,
        news_before_all
    );

    let batch_file_id = app
        .insert_file_record(admin.id, "batch-preserve.png", "image/png")
        .await?;
    let batch_file_path = app.upload_dir().join("batch-preserve.png");
    let batch_file_bytes = b"existing-batch-file-bytes".to_vec();
    std::fs::write(&batch_file_path, &batch_file_bytes)?;
    let files_before_all = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;
    let directory_before = directory_entries(app.upload_dir())?;

    for path in ["/api/v1/files/upload", "/api/v1/files/batch"] {
        let form = MultipartForm::new().add_part(
            "file",
            Part::bytes(b"blocked-upload-bytes".as_slice())
                .file_name("blocked.png")
                .mime_type("image/png"),
        );
        app.server
            .post(path)
            .authorization_bearer(&visitor.token)
            .multipart(form)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    app.server
        .delete("/api/v1/files/batch")
        .authorization_bearer(&visitor.token)
        .json(&json!({"ids":[batch_file_id]}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        files::Entity::find()
            .order_by_asc(files::Column::Id)
            .all(&app.db)
            .await?,
        files_before_all
    );
    assert_eq!(directory_entries(app.upload_dir())?, directory_before);
    assert_eq!(std::fs::read(&batch_file_path)?, batch_file_bytes);

    let comment_before_deleted_attempt = comments::Entity::find_by_id(comment_id)
        .one(&app.db)
        .await?
        .unwrap();
    let comment_parent_before_deleted_attempt = posts::Entity::find_by_id(post.id)
        .one(&app.db)
        .await?
        .unwrap();
    app.server
        .put(&format!("/api/v1/admin/comments/{comment_id}"))
        .authorization_bearer(&visitor.token)
        .json(&json!({"status":"deleted"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        comments::Entity::find_by_id(comment_id)
            .one(&app.db)
            .await?
            .unwrap(),
        comment_before_deleted_attempt
    );
    assert_eq!(
        posts::Entity::find_by_id(post.id)
            .one(&app.db)
            .await?
            .unwrap(),
        comment_parent_before_deleted_attempt
    );

    Ok(())
}

#[tokio::test]
async fn category_and_tag_writes_preserve_owner_or_privileged_policy() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let owner = app.create_user("taxonomy-owner", "author").await?;
    let other = app.create_user("taxonomy-other", "visitor").await?;
    let sub_admin = app.create_user("taxonomy-sub-admin", "sub_admin").await?;
    let admin = app.create_user("taxonomy-admin", "admin").await?;

    let category_resp = app
        .server
        .post("/api/v1/categories")
        .authorization_bearer(&owner.token)
        .json(&json!({"name": "owner-category"}))
        .await;
    category_resp.assert_status_ok();
    let category_id = category_resp.json::<Value>()["data"]["id"]
        .as_i64()
        .unwrap() as i32;
    app.server
        .post("/api/v1/categories")
        .authorization_bearer(&owner.token)
        .json(&json!({
            "name": "owner-category-child",
            "parent_id": category_id
        }))
        .await
        .assert_status_ok();
    let category_before = categories::Entity::find_by_id(category_id)
        .one(&app.db)
        .await?
        .unwrap();
    app.server
        .put(&format!("/api/v1/categories/{category_id}"))
        .authorization_bearer(&other.token)
        .json(&json!({"name": "foreign-category-update"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        categories::Entity::find_by_id(category_id)
            .one(&app.db)
            .await?
            .unwrap(),
        category_before
    );
    let categories_before_delete = categories::Entity::find()
        .order_by_asc(categories::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .delete(&format!("/api/v1/categories/{category_id}"))
        .authorization_bearer(&other.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        categories::Entity::find()
            .order_by_asc(categories::Column::Id)
            .all(&app.db)
            .await?,
        categories_before_delete
    );
    app.server
        .put(&format!("/api/v1/categories/{category_id}"))
        .authorization_bearer(&sub_admin.token)
        .json(&json!({"name": "privileged-category-update"}))
        .await
        .assert_status_ok();

    let second_category_resp = app
        .server
        .post("/api/v1/categories")
        .authorization_bearer(&owner.token)
        .json(&json!({"name": "owner-category-second"}))
        .await;
    second_category_resp.assert_status_ok();
    let second_category_id = second_category_resp.json::<Value>()["data"]["id"]
        .as_i64()
        .unwrap() as i32;
    let categories_before_reorder = categories::Entity::find()
        .order_by_asc(categories::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .get("/api/v1/admin/categories")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&other, &owner] {
        app.server
            .get("/api/v1/admin/categories")
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    for user in [&sub_admin, &admin] {
        app.server
            .get("/api/v1/admin/categories")
            .authorization_bearer(&user.token)
            .await
            .assert_status_ok();
    }
    let reorder = json!({"ids": [second_category_id, category_id]});
    app.server
        .put("/api/v1/admin/categories/reorder")
        .authorization_bearer(&other.token)
        .json(&reorder)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        categories::Entity::find()
            .order_by_asc(categories::Column::Id)
            .all(&app.db)
            .await?,
        categories_before_reorder
    );
    app.server
        .put("/api/v1/admin/categories/reorder")
        .authorization_bearer(&sub_admin.token)
        .json(&reorder)
        .await
        .assert_status_ok();

    let tag_resp = app
        .server
        .post("/api/v1/tags")
        .authorization_bearer(&owner.token)
        .json(&json!({"name": "owner-tag"}))
        .await;
    tag_resp.assert_status_ok();
    let tag_id = tag_resp.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;
    let tagged_post = app
        .create_post(&owner, "taxonomy tagged post", "published")
        .await?;
    post_tags::ActiveModel {
        post_id: Set(tagged_post.id),
        tag_id: Set(tag_id),
    }
    .insert(&app.db)
    .await?;
    let tag_before = tags::Entity::find_by_id(tag_id)
        .one(&app.db)
        .await?
        .unwrap();
    app.server
        .put(&format!("/api/v1/tags/{tag_id}"))
        .authorization_bearer(&other.token)
        .json(&json!({"name": "foreign-tag-update"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        tags::Entity::find_by_id(tag_id)
            .one(&app.db)
            .await?
            .unwrap(),
        tag_before
    );
    let tags_before_delete = tags::Entity::find()
        .order_by_asc(tags::Column::Id)
        .all(&app.db)
        .await?;
    let post_tags_before_delete = post_tags::Entity::find()
        .order_by_asc(post_tags::Column::PostId)
        .order_by_asc(post_tags::Column::TagId)
        .all(&app.db)
        .await?;
    app.server
        .delete(&format!("/api/v1/tags/{tag_id}"))
        .authorization_bearer(&other.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        tags::Entity::find()
            .order_by_asc(tags::Column::Id)
            .all(&app.db)
            .await?,
        tags_before_delete
    );
    assert_eq!(
        post_tags::Entity::find()
            .order_by_asc(post_tags::Column::PostId)
            .order_by_asc(post_tags::Column::TagId)
            .all(&app.db)
            .await?,
        post_tags_before_delete
    );
    app.server
        .put(&format!("/api/v1/tags/{tag_id}"))
        .authorization_bearer(&sub_admin.token)
        .json(&json!({"name": "privileged-tag-update"}))
        .await
        .assert_status_ok();

    Ok(())
}

#[tokio::test]
async fn extended_non_ai_admin_resources_enforce_privileged_matrix() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("extended-visitor", "visitor").await?;
    let author = app.create_user("extended-author", "author").await?;
    let sub_admin = app.create_user("extended-sub-admin", "sub_admin").await?;
    let admin = app.create_user("extended-admin", "admin").await?;

    let denied = [&visitor, &author];
    let privileged = [&sub_admin, &admin];

    for path in [
        "/api/v1/admin/article-types",
        "/api/v1/admin/article-statuses",
        "/api/v1/changelogs",
        "/api/v1/admin/applications/pending-count",
    ] {
        app.server
            .get(path)
            .await
            .assert_status(StatusCode::UNAUTHORIZED);
        for user in denied {
            app.server
                .get(path)
                .authorization_bearer(&user.token)
                .await
                .assert_status(StatusCode::FORBIDDEN);
        }
        for user in privileged {
            app.server
                .get(path)
                .authorization_bearer(&user.token)
                .await
                .assert_status_ok();
        }
    }

    let type_before = article_types::Entity::find().all(&app.db).await?;
    let denied_type = json!({"code": "matrix_denied_type", "display_name": "Denied type"});
    for user in denied {
        app.server
            .post("/api/v1/admin/article-types")
            .authorization_bearer(&user.token)
            .json(&denied_type)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            article_types::Entity::find().all(&app.db).await?,
            type_before
        );
    }
    for (index, user) in privileged.into_iter().enumerate() {
        app.server
            .post("/api/v1/admin/article-types")
            .authorization_bearer(&user.token)
            .json(&json!({
                "code": format!("matrix_type_{index}"),
                "display_name": format!("Matrix type {index}")
            }))
            .await
            .assert_status_ok();
    }

    let status_before = article_statuses::Entity::find().all(&app.db).await?;
    let denied_status = json!({"code": "matrix_denied_status", "display_name": "Denied status"});
    for user in denied {
        app.server
            .post("/api/v1/admin/article-statuses")
            .authorization_bearer(&user.token)
            .json(&denied_status)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            article_statuses::Entity::find().all(&app.db).await?,
            status_before
        );
    }
    for (index, user) in privileged.into_iter().enumerate() {
        app.server
            .post("/api/v1/admin/article-statuses")
            .authorization_bearer(&user.token)
            .json(&json!({
                "code": format!("matrix_status_{index}"),
                "display_name": format!("Matrix status {index}")
            }))
            .await
            .assert_status_ok();
    }

    let changelog_before = changelog::Entity::find().all(&app.db).await?;
    for user in denied {
        app.server
            .post("/api/v1/changelogs")
            .authorization_bearer(&user.token)
            .json(&json!({"version": "matrix-denied", "content": "denied"}))
            .await
            .assert_status(StatusCode::FORBIDDEN);
        assert_eq!(
            changelog::Entity::find().all(&app.db).await?,
            changelog_before
        );
    }
    for (index, user) in privileged.into_iter().enumerate() {
        app.server
            .post("/api/v1/changelogs")
            .authorization_bearer(&user.token)
            .json(&json!({
                "version": format!("matrix-privileged-{index}"),
                "content": "allowed"
            }))
            .await
            .assert_status_ok();
    }

    let type_target = article_types::Entity::find()
        .order_by_desc(article_types::Column::Id)
        .one(&app.db)
        .await?
        .unwrap();
    let types_before_denied = article_types::Entity::find()
        .order_by_asc(article_types::Column::Id)
        .all(&app.db)
        .await?;
    for (method, path) in [
        (
            "put",
            format!("/api/v1/admin/article-types/{}", type_target.id),
        ),
        (
            "delete",
            format!("/api/v1/admin/article-types/{}", type_target.id),
        ),
        ("post", "/api/v1/admin/article-types/reorder".to_string()),
    ] {
        let request = match method {
            "put" => app.server.put(&path).json(&json!({})),
            "delete" => app.server.delete(&path),
            _ => app
                .server
                .post(&path)
                .json(&json!({"ids":[type_target.id]})),
        };
        request
            .authorization_bearer(&visitor.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    assert_eq!(
        article_types::Entity::find()
            .order_by_asc(article_types::Column::Id)
            .all(&app.db)
            .await?,
        types_before_denied
    );

    let status_target = article_statuses::Entity::find()
        .order_by_desc(article_statuses::Column::Id)
        .one(&app.db)
        .await?
        .unwrap();
    let statuses_before_denied = article_statuses::Entity::find()
        .order_by_asc(article_statuses::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .put(&format!(
            "/api/v1/admin/article-statuses/{}",
            status_target.id
        ))
        .authorization_bearer(&visitor.token)
        .json(&json!({}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!(
            "/api/v1/admin/article-statuses/{}",
            status_target.id
        ))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .post("/api/v1/admin/article-statuses/reorder")
        .authorization_bearer(&visitor.token)
        .json(&json!({"ids":[status_target.id]}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        article_statuses::Entity::find()
            .order_by_asc(article_statuses::Column::Id)
            .all(&app.db)
            .await?,
        statuses_before_denied
    );

    let changelog_target = changelog::Entity::find()
        .order_by_desc(changelog::Column::Id)
        .one(&app.db)
        .await?
        .unwrap();
    let changelogs_before_denied = changelog::Entity::find()
        .order_by_asc(changelog::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .put(&format!("/api/v1/changelogs/{}", changelog_target.id))
        .authorization_bearer(&visitor.token)
        .json(&json!({}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/changelogs/{}", changelog_target.id))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        changelog::Entity::find()
            .order_by_asc(changelog::Column::Id)
            .all(&app.db)
            .await?,
        changelogs_before_denied
    );

    let guestbook_resp = app
        .server
        .post("/api/v1/guestbook")
        .json(&json!({
            "nickname":"matrix guest",
            "email":"guest@example.com",
            "content":"matrix guest content"
        }))
        .await;
    guestbook_resp.assert_status_ok();
    let guestbook_id = guestbook_resp.json::<Value>()["data"]["id"]
        .as_i64()
        .unwrap() as i32;
    let guestbook_before_denied = guestbook::Entity::find()
        .order_by_asc(guestbook::Column::Id)
        .all(&app.db)
        .await?;
    app.server
        .put(&format!("/api/v1/admin/guestbook/{guestbook_id}/reply"))
        .authorization_bearer(&visitor.token)
        .json(&json!({"reply":"blocked"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/admin/guestbook/{guestbook_id}"))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        guestbook::Entity::find()
            .order_by_asc(guestbook::Column::Id)
            .all(&app.db)
            .await?,
        guestbook_before_denied
    );

    let applicant = app.create_user("extended-applicant", "visitor").await?;
    let application_resp = app
        .server
        .post("/api/v1/apply")
        .authorization_bearer(&applicant.token)
        .json(&json!({"reason": "write", "content": "security content"}))
        .await;
    application_resp.assert_status_ok();
    let application_id = application_resp.json::<Value>()["data"]["id"]
        .as_i64()
        .unwrap() as i32;
    let application_before = author_applications::Entity::find_by_id(application_id)
        .one(&app.db)
        .await?
        .unwrap();
    let applicant_before = users::Entity::find_by_id(applicant.id)
        .one(&app.db)
        .await?
        .unwrap();
    for user in denied {
        for action in ["approve", "reject"] {
            app.server
                .post(&format!(
                    "/api/v1/admin/applications/{application_id}/{action}"
                ))
                .authorization_bearer(&user.token)
                .await
                .assert_status(StatusCode::FORBIDDEN);
        }
        assert_eq!(
            author_applications::Entity::find_by_id(application_id)
                .one(&app.db)
                .await?
                .unwrap(),
            application_before
        );
        assert_eq!(
            users::Entity::find_by_id(applicant.id)
                .one(&app.db)
                .await?
                .unwrap(),
            applicant_before
        );
    }
    app.server
        .post(&format!(
            "/api/v1/admin/applications/{application_id}/approve"
        ))
        .authorization_bearer(&sub_admin.token)
        .await
        .assert_status_ok();
    assert_eq!(
        users::Entity::find_by_id(applicant.id)
            .one(&app.db)
            .await?
            .unwrap()
            .role,
        "author"
    );

    Ok(())
}

#[tokio::test]
async fn admin_news_detail_and_pending_comment_count_require_privileged_user() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let visitor = app.create_user("admin-read-visitor", "visitor").await?;
    let author = app.create_user("admin-read-author", "author").await?;
    let sub_admin = app.create_user("admin-read-sub-admin", "sub_admin").await?;
    let admin = app.create_user("admin-read-admin", "admin").await?;

    let created = app
        .server
        .post("/api/v1/admin/news")
        .authorization_bearer(&sub_admin.token)
        .json(&json!({
            "title": "private draft news",
            "content": "private draft content",
            "topic_type": "security",
            "status": "draft"
        }))
        .await;
    created.assert_status_ok();
    let news_id = created.json::<Value>()["data"]["id"].as_i64().unwrap() as i32;

    app.server
        .get(&format!("/api/v1/news/{news_id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    let public_list = app
        .server
        .get("/api/v1/news?status=draft&include_content=true&page_size=100")
        .await;
    public_list.assert_status_ok();
    let public_list_json = public_list.json::<Value>();
    assert!(public_list_json["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["id"].as_i64() != Some(news_id as i64)));
    app.server
        .get(&format!("/api/v1/news/{news_id}"))
        .authorization_bearer("definitely-invalid")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    app.server
        .get(&format!("/api/v1/admin/news/{news_id}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .get(&format!("/api/v1/admin/news/{news_id}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    for user in [&sub_admin, &admin] {
        app.server
            .get(&format!("/api/v1/admin/news/{news_id}"))
            .authorization_bearer(&user.token)
            .await
            .assert_status_ok();
    }

    let pending_path = "/api/v1/admin/comments/pending-count";
    app.server
        .get(pending_path)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    app.server
        .get(pending_path)
        .authorization_bearer("definitely-invalid")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author] {
        app.server
            .get(pending_path)
            .authorization_bearer(&user.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    for user in [&sub_admin, &admin] {
        app.server
            .get(pending_path)
            .authorization_bearer(&user.token)
            .await
            .assert_status_ok();
    }

    Ok(())
}

#[tokio::test]
async fn supplied_invalid_credentials_do_not_fall_back_to_anonymous_writes() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app
        .create_user("optional-auth-post-author", "author")
        .await?;
    let post = app
        .create_post(&author, "optional auth post", "published")
        .await?;
    let post_id = post.id;

    app.server
        .get(&format!("/api/v1/posts/{post_id}/comments"))
        .authorization_bearer("definitely-invalid")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    let comments_before = comments::Entity::find().all(&app.db).await?;
    app.server
        .post(&format!("/api/v1/posts/{post_id}/comments"))
        .authorization_bearer("definitely-invalid")
        .json(&json!({
            "content": "must not be stored",
            "author_name": "invalid token"
        }))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(
        comments::Entity::find().all(&app.db).await?,
        comments_before
    );

    let guestbook_before = guestbook::Entity::find().all(&app.db).await?;
    app.server
        .post("/api/v1/guestbook")
        .authorization_bearer("definitely-invalid")
        .json(&json!({
            "nickname": "invalid token",
            "email": "invalid@example.com",
            "content": "must not be stored"
        }))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(
        guestbook::Entity::find().all(&app.db).await?,
        guestbook_before
    );

    let read_logs_before = read_logs::Entity::find().all(&app.db).await?;
    app.server
        .post("/api/v1/read-logs")
        .authorization_bearer("definitely-invalid")
        .json(&json!({
            "post_id": post_id,
            "duration_seconds": 12
        }))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(
        read_logs::Entity::find().all(&app.db).await?,
        read_logs_before
    );

    Ok(())
}

#[tokio::test]
async fn visitor_cannot_bypass_post_creation_roles_through_import() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("import-matrix-visitor", "visitor").await?;
    let posts_before = posts::Entity::find()
        .order_by_asc(posts::Column::Id)
        .all(&app.db)
        .await?;
    let files_before = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;
    let categories_before = categories::Entity::find()
        .order_by_asc(categories::Column::Id)
        .all(&app.db)
        .await?;
    let tags_before = tags::Entity::find()
        .order_by_asc(tags::Column::Id)
        .all(&app.db)
        .await?;
    let directory_before = directory_entries(&app.upload_dir())?;

    app.server
        .post("/api/v1/import/posts")
        .authorization_bearer(&visitor.token)
        .json(&json!({
            "items": [{
                "filename": "visitor-bypass.md",
                "content": "---\ntitle: Visitor bypass\nstatus: draft\ncategory: blocked-category\ntags:\n  - blocked-tag\n---\nblocked body",
                "images": []
            }]
        }))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    assert_eq!(
        posts::Entity::find()
            .order_by_asc(posts::Column::Id)
            .all(&app.db)
            .await?,
        posts_before
    );
    assert_eq!(
        files::Entity::find()
            .order_by_asc(files::Column::Id)
            .all(&app.db)
            .await?,
        files_before
    );
    assert_eq!(
        categories::Entity::find()
            .order_by_asc(categories::Column::Id)
            .all(&app.db)
            .await?,
        categories_before
    );
    assert_eq!(
        tags::Entity::find()
            .order_by_asc(tags::Column::Id)
            .all(&app.db)
            .await?,
        tags_before
    );
    assert_eq!(directory_entries(&app.upload_dir())?, directory_before);
    Ok(())
}

#[tokio::test]
async fn privileged_post_management_writes_reject_ordinary_users_without_side_effects(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("post-matrix-author", "author").await?;
    let visitor = app.create_user("post-matrix-visitor", "visitor").await?;
    let post = app
        .create_post(&author, "privileged post matrix", "published")
        .await?;
    let before = posts::Entity::find()
        .order_by_asc(posts::Column::Id)
        .all(&app.db)
        .await?;

    for path in [
        "/api/v1/admin/posts/batch-delete",
        "/api/v1/admin/posts/batch-publish",
        "/api/v1/admin/posts/batch-unpublish",
    ] {
        app.server
            .post(path)
            .authorization_bearer(&visitor.token)
            .json(&json!({"ids":[post.id]}))
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    for action in ["pin", "unpin"] {
        app.server
            .post(&format!("/api/v1/admin/posts/{}/{action}", post.id))
            .authorization_bearer(&visitor.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    app.server
        .put("/api/v1/admin/posts/pin-order")
        .authorization_bearer(&visitor.token)
        .json(&json!({"ids":[post.id]}))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    assert_eq!(
        posts::Entity::find()
            .order_by_asc(posts::Column::Id)
            .all(&app.db)
            .await?,
        before
    );
    Ok(())
}

#[tokio::test]
async fn global_file_md5_lookup_is_admin_only() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("md5-visitor", "visitor").await?;
    let author = app.create_user("md5-author", "author").await?;
    let sub_admin = app.create_user("md5-sub-admin", "sub_admin").await?;
    let admin = app.create_user("md5-admin", "admin").await?;
    let path = "/api/v1/files/check-md5";
    let body = json!({"md5_list": ["00000000000000000000000000000000"]});

    app.server
        .post(path)
        .json(&body)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    for user in [&visitor, &author, &sub_admin] {
        app.server
            .post(path)
            .authorization_bearer(&user.token)
            .json(&body)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    app.server
        .post(path)
        .authorization_bearer(&admin.token)
        .json(&body)
        .await
        .assert_status_ok();

    Ok(())
}
