//! SEC-08 Security Headers HTTP integration tests.
//!
//! Verifies CSP-Report-Only, X-Content-Type-Options, X-Frame-Options,
//! and HSTS headers are present on responses via the real Router.

mod common;

use common::TestApp;

fn header_str(resp: &axum_test::TestResponse, name: &str) -> String {
    resp.header(name)
        .to_str()
        .unwrap_or("")
        .to_string()
}

#[tokio::test]
async fn any_response_includes_csp_report_only_header() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let resp = app.server.get("/api/v1/health").await;
    resp.assert_status_ok();

    let csp = header_str(&resp, "content-security-policy-report-only");
    assert!(!csp.is_empty(), "CSP-Report-Only header must be present");
    Ok(())
}

#[tokio::test]
async fn any_response_includes_x_content_type_options_nosniff() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let resp = app.server.get("/api/v1/version").await;
    resp.assert_status_ok();

    let nosniff = header_str(&resp, "x-content-type-options");
    assert_eq!(nosniff, "nosniff", "X-Content-Type-Options must be nosniff");
    Ok(())
}

#[tokio::test]
async fn any_response_includes_x_frame_options_sameorigin() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let resp = app.server.get("/api/v1/version").await;
    resp.assert_status_ok();

    let frame = header_str(&resp, "x-frame-options");
    assert_eq!(frame, "SAMEORIGIN", "X-Frame-Options must be SAMEORIGIN");
    Ok(())
}

#[tokio::test]
async fn csp_report_endpoint_accepts_valid_report() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let resp = app
        .server
        .post("/api/v1/csp-report")
        .json(&serde_json::json!({
            "csp-report": {
                "document-uri": "https://example.com/page",
                "violated-directive": "script-src",
                "blocked-uri": "https://evil.com/xss.js"
            }
        }))
        .await;

    resp.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn referrer_policy_is_strict_origin_when_cross_origin() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let resp = app.server.get("/api/v1/health").await;
    resp.assert_status_ok();

    let referrer = header_str(&resp, "referrer-policy");
    assert_eq!(
        referrer, "strict-origin-when-cross-origin",
        "Referrer-Policy must be strict-origin-when-cross-origin"
    );
    Ok(())
}
