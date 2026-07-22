//! CSP violation report endpoint.
//! Receives bounded CSP Report-Only violation reports from browsers and logs safe fields.
use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap},
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use crate::utils::{AppError, AppState};

const MAX_URI_LOG_CHARS: usize = 2_048;
const MAX_DIRECTIVE_LOG_CHARS: usize = 256;
const REPORT_LOG_LIMIT: u32 = 20;
const REPORT_LOG_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Deserialize)]
pub(crate) struct CspReportEnvelope {
    #[serde(rename = "csp-report")]
    report: CspViolationReport,
}

#[derive(Debug, Default, Deserialize)]
struct CspViolationReport {
    #[serde(rename = "document-uri", default)]
    document_uri: String,
    #[serde(rename = "violated-directive", default)]
    violated_directive: String,
    #[serde(rename = "blocked-uri", default)]
    blocked_uri: String,
}

fn sanitize_report_field(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .filter(|character| {
            !character.is_control() && !matches!(character, '\u{2028}' | '\u{2029}')
        })
        .take(max_chars)
        .collect()
}

fn sanitize_report_uri(value: &str) -> String {
    let sanitized = sanitize_report_field(value, MAX_URI_LOG_CHARS);
    if sanitized.is_empty() || matches!(sanitized.as_str(), "inline" | "eval" | "wasm-eval") {
        return sanitized;
    }
    let Ok(mut url) = url::Url::parse(&sanitized) else {
        return "[redacted-uri]".to_string();
    };
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return "[redacted-uri]".to_string();
    }
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

struct ReportLogLimiter {
    window_started: Instant,
    count: u32,
}

impl ReportLogLimiter {
    fn new(now: Instant) -> Self {
        Self {
            window_started: now,
            count: 0,
        }
    }

    fn allow(&mut self, now: Instant) -> bool {
        if now.duration_since(self.window_started) >= REPORT_LOG_WINDOW {
            self.window_started = now;
            self.count = 0;
        }
        if self.count >= REPORT_LOG_LIMIT {
            return false;
        }
        self.count += 1;
        true
    }
}

fn should_log_report() -> bool {
    static LIMITER: LazyLock<Mutex<ReportLogLimiter>> =
        LazyLock::new(|| Mutex::new(ReportLogLimiter::new(Instant::now())));
    LIMITER
        .lock()
        .map(|mut limiter| limiter.allow(Instant::now()))
        .unwrap_or(false)
}

/// POST /api/v1/csp-report
///
/// Receives CSP violation reports in the standard `application/csp-report` format.
/// The route applies a 16 KiB body limit; individual logged fields are bounded,
/// stripped of line-breaking controls, URI-sanitized fail closed, and sampled to
/// at most 20 warning records per process per minute.
pub(crate) async fn csp_report_handler(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, AppError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    let supported_content_type = content_type.is_some_and(|value| {
        value.eq_ignore_ascii_case("application/json")
            || value.eq_ignore_ascii_case("application/csp-report")
    });
    if !supported_content_type {
        return Err(AppError::BadRequest(
            "CSP report Content-Type 必须为 application/json 或 application/csp-report".to_string(),
        ));
    }
    let body: CspReportEnvelope = serde_json::from_slice(&body)
        .map_err(|_| AppError::BadRequest("CSP report JSON 格式无效".to_string()))?;

    if should_log_report() {
        let document_uri = sanitize_report_uri(&body.report.document_uri);
        let violated_directive =
            sanitize_report_field(&body.report.violated_directive, MAX_DIRECTIVE_LOG_CHARS);
        let blocked_uri = sanitize_report_uri(&body.report.blocked_uri);
        tracing::warn!(
            document_uri = %document_uri,
            violated_directive = %violated_directive,
            blocked_uri = %blocked_uri,
            "CSP violation reported",
        );
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::{
        sanitize_report_field, sanitize_report_uri, ReportLogLimiter, REPORT_LOG_LIMIT,
        REPORT_LOG_WINDOW,
    };
    use std::time::Instant;

    #[test]
    fn report_fields_are_bounded_and_control_characters_are_removed() {
        let input = format!(
            "https://example.test/\n\u{2028}\u{2029}{}\0",
            "x".repeat(4_096)
        );
        let sanitized = sanitize_report_field(&input, 128);
        assert!(sanitized.chars().count() <= 128);
        assert!(!sanitized.chars().any(char::is_control));
        assert!(!sanitized.contains(['\u{2028}', '\u{2029}']));
    }

    #[test]
    fn report_uris_do_not_log_credentials_query_or_fragment() {
        assert_eq!(
            sanitize_report_uri("https://user:secret@example.test/path?token=secret#fragment"),
            "https://example.test/path"
        );
        assert_eq!(
            sanitize_report_uri("not-a-uri?token=secret"),
            "[redacted-uri]"
        );
        assert_eq!(
            sanitize_report_uri("data:text/plain,secret"),
            "[redacted-uri]"
        );
        assert_eq!(sanitize_report_uri("inline"), "inline");
        assert_eq!(sanitize_report_uri("eval"), "eval");
        assert_eq!(sanitize_report_uri("wasm-eval"), "wasm-eval");
        assert_eq!(sanitize_report_uri("secret-token"), "[redacted-uri]");
        assert_eq!(sanitize_report_uri("not-a-uri"), "[redacted-uri]");
    }

    #[test]
    fn report_logging_is_sampled_per_time_window() {
        let now = Instant::now();
        let mut limiter = ReportLogLimiter::new(now);
        for _ in 0..REPORT_LOG_LIMIT {
            assert!(limiter.allow(now));
        }
        assert!(!limiter.allow(now));
        assert!(limiter.allow(now + REPORT_LOG_WINDOW));
    }
}
