//! SEC-08 Security Headers HTTP integration tests.
//!
//! Verifies CSP-Report-Only, X-Content-Type-Options, X-Frame-Options,
//! and HSTS headers are present on responses via the real Router.

mod common;

use axum::{
    body::Bytes,
    http::{header, Method, StatusCode},
};
use common::TestApp;

fn header_str(resp: &axum_test::TestResponse, name: &str) -> String {
    resp.header(name).to_str().unwrap_or("").to_string()
}

#[tokio::test]
async fn any_response_includes_csp_report_only_header() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let resp = app.server.get("/api/v1/health").await;
    resp.assert_status_ok();

    let csp = header_str(&resp, "content-security-policy-report-only");
    assert!(!csp.is_empty(), "CSP-Report-Only header must be present");
    for directive in [
        "default-src 'self'",
        "object-src 'none'",
        "base-uri 'self'",
        "frame-ancestors 'self'",
        "form-action 'self'",
        "report-uri /api/v1/csp-report",
    ] {
        assert!(
            csp.contains(directive),
            "missing CSP directive: {directive}"
        );
    }
    assert!(!csp.contains("'unsafe-eval'"));
    assert!(resp.headers().get("content-security-policy").is_none());
    Ok(())
}

#[tokio::test]
async fn cors_allows_only_configured_origins_with_minimal_preflight_surface() -> anyhow::Result<()>
{
    let allowed_origin = "https://admin.example.test";
    let app = TestApp::new_with_cors_allowed_origins(vec![allowed_origin.to_string()]).await?;

    let same_origin = app.server.get("/api/v1/health").await;
    same_origin.assert_status_ok();
    assert!(same_origin
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());

    let default_app = TestApp::new().await?;
    let default_rejected = default_app
        .server
        .get("/api/v1/health")
        .add_header(header::ORIGIN, "https://evil.example.test")
        .await;
    default_rejected.assert_status_ok();
    assert!(default_rejected
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());

    let allowed = app
        .server
        .get("/api/v1/health")
        .add_header(header::ORIGIN, allowed_origin)
        .await;
    allowed.assert_status_ok();
    assert_eq!(
        allowed.header(header::ACCESS_CONTROL_ALLOW_ORIGIN),
        allowed_origin
    );

    let rejected = app
        .server
        .get("/api/v1/health")
        .add_header(header::ORIGIN, "https://evil.example.test")
        .await;
    rejected.assert_status_ok();
    assert!(rejected
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());

    let preflight = app
        .server
        .method(Method::OPTIONS, "/api/v1/posts")
        .add_header(header::ORIGIN, allowed_origin)
        .add_header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .add_header(
            header::ACCESS_CONTROL_REQUEST_HEADERS,
            "authorization,content-type,x-api-key",
        )
        .await;
    preflight.assert_status_ok();
    let methods = header_str(&preflight, "access-control-allow-methods");
    let headers = header_str(&preflight, "access-control-allow-headers");
    let method_set = methods
        .split(',')
        .map(|method| method.trim().to_ascii_uppercase())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        method_set,
        ["DELETE", "GET", "POST", "PUT"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );
    let header_set = headers
        .split(',')
        .map(|name| name.trim().to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        header_set,
        ["authorization", "content-type", "x-api-key"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );
    assert_eq!(header_str(&preflight, "x-content-type-options"), "nosniff");

    let rejected_preflight = app
        .server
        .method(Method::OPTIONS, "/api/v1/posts")
        .add_header(header::ORIGIN, "https://evil.example.test")
        .add_header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .await;
    assert_eq!(rejected_preflight.status_code(), StatusCode::OK);
    assert!(rejected_preflight
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());

    for invalid_origin in [
        "*",
        "https://example.test/not-an-origin",
        "https://example.test?query=1",
        "https://example.test#fragment",
        "https://user@example.test",
        "ftp://example.test",
    ] {
        let invalid =
            TestApp::new_with_cors_allowed_origins(vec![invalid_origin.to_string()]).await;
        assert!(
            invalid.is_err(),
            "invalid CORS origin must fail startup: {invalid_origin}"
        );
    }

    let canonical_app =
        TestApp::new_with_cors_allowed_origins(vec!["HTTPS://ADMIN.EXAMPLE.TEST:443".to_string()])
            .await?;
    let canonical = canonical_app
        .server
        .get("/api/v1/health")
        .add_header(header::ORIGIN, allowed_origin)
        .await;
    canonical.assert_status_ok();
    assert_eq!(
        canonical.header(header::ACCESS_CONTROL_ALLOW_ORIGIN),
        allowed_origin
    );

    let rejected_method = app
        .server
        .method(Method::OPTIONS, "/api/v1/posts")
        .add_header(header::ORIGIN, allowed_origin)
        .add_header(header::ACCESS_CONTROL_REQUEST_METHOD, "PATCH")
        .await;
    assert!(
        !header_str(&rejected_method, "access-control-allow-methods")
            .split(',')
            .any(|method| method.trim() == "PATCH")
    );
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
    let body = serde_json::to_vec(&serde_json::json!({
        "csp-report": {
            "document-uri": "https://example.com/page",
            "violated-directive": "script-src",
            "blocked-uri": "https://evil.com/xss.js"
        }
    }))?;

    let resp = app
        .server
        .post("/api/v1/csp-report")
        .content_type("application/csp-report")
        .bytes(Bytes::from(body))
        .await;

    resp.assert_status_ok();

    let mixed_case = app
        .server
        .post("/api/v1/csp-report")
        .content_type("Application/CSP-Report; charset=UTF-8")
        .bytes(Bytes::from(serde_json::to_vec(&serde_json::json!({
            "csp-report": { "blocked-uri": "inline" }
        }))?))
        .await;
    mixed_case.assert_status_ok();

    app.server
        .post("/api/v1/csp-report")
        .content_type("application/csp-report")
        .bytes(Bytes::from_static(b"{invalid"))
        .await
        .assert_status_bad_request();
    app.server
        .post("/api/v1/csp-report")
        .json(&serde_json::json!({ "blocked-uri": "inline" }))
        .await
        .assert_status_bad_request();
    Ok(())
}

#[tokio::test]
async fn csp_report_endpoint_rejects_oversized_reports() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let response = app
        .server
        .post("/api/v1/csp-report")
        .json(&serde_json::json!({
            "csp-report": {
                "document-uri": "https://example.com/page",
                "violated-directive": "script-src",
                "blocked-uri": "x".repeat(20 * 1024)
            }
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(header_str(&response, "x-content-type-options"), "nosniff");
    Ok(())
}

#[tokio::test]
async fn hsts_is_present_on_all_https_responses_and_absent_on_http() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let expected = "max-age=31536000";

    let https_ok = app
        .server
        .get("/api/v1/health")
        .add_header("x-forwarded-proto", "https")
        .await;
    https_ok.assert_status_ok();
    assert_eq!(header_str(&https_ok, "strict-transport-security"), expected);

    let https_not_found = app
        .server
        .get("/api/v1/definitely-missing")
        .add_header("x-forwarded-proto", "https")
        .await;
    https_not_found.assert_status_not_found();
    assert_eq!(
        header_str(&https_not_found, "strict-transport-security"),
        expected
    );

    let ambiguous_forwarding = app
        .server
        .get("/api/v1/health")
        .add_header("x-forwarded-proto", "https,http")
        .await;
    ambiguous_forwarding.assert_status_ok();
    assert!(ambiguous_forwarding
        .headers()
        .get("strict-transport-security")
        .is_none());

    let http = app.server.get("/api/v1/health").await;
    http.assert_status_ok();
    assert!(http.headers().get("strict-transport-security").is_none());

    let untrusted_app = TestApp::new_with_trusted_proxies(Vec::new()).await?;
    let spoofed = untrusted_app
        .server
        .get("/api/v1/health")
        .add_header("x-forwarded-proto", "https")
        .await;
    spoofed.assert_status_ok();
    assert!(spoofed.headers().get("strict-transport-security").is_none());
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
