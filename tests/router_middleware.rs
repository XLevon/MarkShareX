mod common;

use axum::http::StatusCode;
use common::TestApp;

#[tokio::test]
async fn production_router_enforces_ip_blacklist_from_connect_info() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    app.set_setting("ip_blacklist_enabled", "true").await?;
    app.set_setting("ip_blacklist", r#"["127.0.0.1","::1"]"#)
        .await?;

    app.server
        .get("/api/v1/health")
        .await
        .assert_status(StatusCode::FORBIDDEN);

    Ok(())
}
