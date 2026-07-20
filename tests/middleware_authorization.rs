mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[tokio::test]
async fn api_key_authentication_and_whitelist_branches_use_the_production_router(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("api-key-author", "author").await?;
    let api_key = "qa-api-key-author-secret";
    app.set_user_api_key(author.id, Some(api_key)).await?;

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", "invalid-api-key")
        .json(&json!({"title": "invalid key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 0);

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .json(&json!({"title": "valid api key post", "status": "draft"}))
        .await
        .assert_status_ok();
    assert_eq!(app.post_count().await?, 1);

    app.set_setting("ip_whitelist_enabled", "true").await?;
    app.set_setting("ip_whitelist", r#"["192.0.2.10"]"#).await?;

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .json(&json!({"title": "socket rejected", "status": "draft"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(app.post_count().await?, 1);

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Real-IP", "192.0.2.10")
        .json(&json!({"title": "real ip allowed", "status": "draft"}))
        .await
        .assert_status_ok();

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Forwarded-For", "192.0.2.10, 10.0.0.2")
        .json(&json!({"title": "forwarded ip allowed", "status": "draft"}))
        .await
        .assert_status_ok();

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Real-IP", "198.51.100.20")
        .add_header("X-Forwarded-For", "192.0.2.10")
        .json(&json!({"title": "real ip precedence rejected", "status": "draft"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(app.post_count().await?, 3);

    app.set_user_status(author.id, "disabled").await?;
    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Real-IP", "192.0.2.10")
        .json(&json!({"title": "inactive status key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 3);

    app.set_user_status(author.id, "active").await?;
    app.set_user_active(author.id, false).await?;
    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Real-IP", "192.0.2.10")
        .json(&json!({"title": "inactive flag key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 3);

    app.set_user_active(author.id, true).await?;
    app.set_user_deleted(author.id, true).await?;
    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Real-IP", "192.0.2.10")
        .json(&json!({"title": "deleted user key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 3);

    Ok(())
}

#[tokio::test]
async fn ip_blacklist_uses_real_ip_then_forwarded_for_then_socket_fallback() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting(
        "ip_blacklist",
        r#"["198.51.100.7","203.0.113.9","127.0.0.1"]"#,
    )
    .await?;

    app.server
        .get("/api/v1/health")
        .add_header("X-Real-IP", "198.51.100.7")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    app.server
        .get("/api/v1/health")
        .add_header("X-Forwarded-For", "203.0.113.9, 10.0.0.1")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    app.server
        .get("/api/v1/health")
        .add_header("X-Real-IP", "192.0.2.55")
        .add_header("X-Forwarded-For", "203.0.113.9")
        .await
        .assert_status_ok();

    app.server
        .get("/api/v1/health")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    Ok(())
}
