mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

#[tokio::test]
async fn malformed_forwarded_chain_is_rejected_at_the_router() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    app.server
        .get("/api/v1/health")
        .add_header("X-Forwarded-For", "203.0.113.20, not-an-ip")
        .await
        .assert_status(StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn invalid_leftmost_forwarded_value_cannot_hide_the_real_client() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting("ip_blacklist", r#"["203.0.113.20"]"#)
        .await?;
    app.state.invalidate_ip_guard_rules_cache().await;

    app.server
        .get("/api/v1/health")
        .add_header("X-Forwarded-For", "not-an-ip, 203.0.113.20")
        .await
        .assert_status(StatusCode::FORBIDDEN);
    Ok(())
}

#[tokio::test]
async fn all_trusted_chain_uses_leftmost_source() -> anyhow::Result<()> {
    let app = TestApp::new_with_trusted_proxies(vec![
        "127.0.0.1".to_string(),
        "10.0.0.2".to_string(),
        "10.0.0.3".to_string(),
    ])
    .await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting("ip_blacklist", r#"["10.0.0.2"]"#).await?;
    app.state.invalidate_ip_guard_rules_cache().await;

    app.server
        .get("/api/v1/health")
        .add_header("X-Forwarded-For", "10.0.0.2, 10.0.0.3")
        .await
        .assert_status(StatusCode::FORBIDDEN);
    Ok(())
}

#[tokio::test]
async fn mapped_ipv6_blacklist_matches_ipv4_client() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting("ip_blacklist", r#"["::ffff:203.0.113.7"]"#)
        .await?;
    app.state.invalidate_ip_guard_rules_cache().await;

    app.server
        .get("/api/v1/health")
        .add_header("X-Forwarded-For", "203.0.113.7")
        .await
        .assert_status(StatusCode::FORBIDDEN);
    Ok(())
}

#[tokio::test]
async fn untrusted_socket_headers_cannot_trigger_blacklist_rules() -> anyhow::Result<()> {
    let app = TestApp::new_with_trusted_proxies(vec![]).await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting("ip_blacklist", r#"["198.51.100.7","203.0.113.9"]"#)
        .await?;
    app.state.invalidate_ip_guard_rules_cache().await;

    app.server
        .get("/api/v1/health")
        .add_header("X-Real-IP", "198.51.100.7")
        .add_header("X-Forwarded-For", "203.0.113.9")
        .await
        .assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn ip_guard_rules_are_isolated_between_apps() -> anyhow::Result<()> {
    let blocked_app = TestApp::new().await?;
    blocked_app
        .set_setting("ip_blacklist_enabled", "true")
        .await?;
    blocked_app
        .set_setting("ip_blacklist", r#"["127.0.0.1"]"#)
        .await?;

    let allowed_app = TestApp::new().await?;
    allowed_app
        .set_setting("ip_blacklist_enabled", "true")
        .await?;
    allowed_app
        .set_setting("ip_blacklist", r#"["192.0.2.1"]"#)
        .await?;

    blocked_app
        .server
        .get("/api/v1/health")
        .await
        .assert_status(StatusCode::FORBIDDEN);
    allowed_app
        .server
        .get("/api/v1/health")
        .await
        .assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn updating_ip_guard_settings_invalidates_the_current_app_cache() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("ip-settings-admin", "admin").await?;

    app.server.get("/api/v1/health").await.assert_status_ok();

    app.server
        .put("/api/v1/settings")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "settings": {
                "ip_blacklist_enabled": "true",
                "ip_blacklist": "[\"127.0.0.1\"]"
            }
        }))
        .await
        .assert_status_ok();

    app.server
        .get("/api/v1/health")
        .await
        .assert_status(StatusCode::FORBIDDEN);
    Ok(())
}

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
    app.state.invalidate_ip_guard_rules_cache().await;

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
        .add_header("X-Real-IP", "198.51.100.200")
        .add_header("X-Forwarded-For", "198.51.100.200, 192.0.2.10")
        .json(&json!({"title": "real ip allowed", "status": "draft"}))
        .await
        .assert_status_ok();

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Forwarded-For", "192.0.2.10")
        .json(&json!({"title": "forwarded ip allowed", "status": "draft"}))
        .await
        .assert_status_ok();

    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Real-IP", "192.0.2.10")
        .add_header("X-Forwarded-For", "192.0.2.10, 198.51.100.20")
        .json(&json!({"title": "real ip precedence rejected", "status": "draft"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(app.post_count().await?, 3);

    app.set_user_status(author.id, "disabled").await?;
    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Forwarded-For", "192.0.2.10")
        .json(&json!({"title": "inactive status key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 3);

    app.set_user_status(author.id, "active").await?;
    app.set_user_active(author.id, false).await?;
    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Forwarded-For", "192.0.2.10")
        .json(&json!({"title": "inactive flag key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 3);

    app.set_user_active(author.id, true).await?;
    app.set_user_deleted(author.id, true).await?;
    app.server
        .post("/api/v1/posts")
        .add_header("X-API-Key", api_key)
        .add_header("X-Forwarded-For", "192.0.2.10")
        .json(&json!({"title": "deleted user key side effect", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(app.post_count().await?, 3);

    Ok(())
}

#[tokio::test]
async fn ip_blacklist_uses_trusted_proxy_chain_then_socket_fallback() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting(
        "ip_blacklist",
        r#"["198.51.100.7","203.0.113.9","127.0.0.1"]"#,
    )
    .await?;
    app.state.invalidate_ip_guard_rules_cache().await;

    app.server
        .get("/api/v1/health")
        .add_header("X-Real-IP", "192.0.2.55")
        .add_header("X-Forwarded-For", "192.0.2.55, 198.51.100.7")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    app.server
        .get("/api/v1/health")
        .add_header("X-Forwarded-For", "192.0.2.55, 203.0.113.9")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    app.server
        .get("/api/v1/health")
        .add_header("X-Real-IP", "203.0.113.9")
        .add_header("X-Forwarded-For", "203.0.113.9, 192.0.2.55")
        .await
        .assert_status_ok();

    app.server
        .get("/api/v1/health")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    Ok(())
}
