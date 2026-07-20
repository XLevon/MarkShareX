//! CSP violation report endpoint.
//! Receives CSP Report-Only violation reports from browsers and logs them.
use axum::{extract::State, Json};
use serde_json::Value;

use crate::utils::{AppError, AppState};

/// POST /api/v1/csp-report
///
/// Receives CSP violation reports in the standard `application/csp-report` format.
/// Logs each violation at WARN level for monitoring.
pub async fn csp_report_handler(
    State(_state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let report = &body["csp-report"];

    let document_uri = report["document-uri"].as_str().unwrap_or("?");
    let violated_directive = report["violated-directive"].as_str().unwrap_or("?");
    let blocked_uri = report["blocked-uri"].as_str().unwrap_or("?");

    tracing::warn!(
        document_uri = %document_uri,
        violated_directive = %violated_directive,
        blocked_uri = %blocked_uri,
        "CSP violation reported",
    );

    Ok(Json(serde_json::json!({ "ok": true })))
}
