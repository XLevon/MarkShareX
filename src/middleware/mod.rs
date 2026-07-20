pub mod auth;
pub mod ip_guard;

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    response::Response,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Layer;
use tower_http::cors::CorsLayer;

pub fn stack() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}

// ── Compression Layer ──

pub fn compression() -> tower_http::compression::CompressionLayer {
    tower_http::compression::CompressionLayer::new()
        .gzip(true)
        .br(true)
}

// ── Security Headers Layer ──

#[derive(Clone)]
pub struct SecurityHeadersLayer;

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersMiddleware<S> {
    inner: S,
}

impl<S, B> tower::Service<Request<B>> for SecurityHeadersMiddleware<S>
where
    S: tower::Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let is_https = req
                .headers()
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                == Some("https");

            let mut response: Response = inner.call(req).await?;

            // Collect status before mutable borrow
            let is_success = response.status().is_success();

            let headers = response.headers_mut();

            // Content-Type sniffing protection
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );

            // Referrer policy: send origin on cross-origin, full URL on same-origin
            headers.insert(
                header::REFERRER_POLICY,
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Prevent clickjacking
            headers.insert(
                header::X_FRAME_OPTIONS,
                HeaderValue::from_static("SAMEORIGIN"),
            );

            // HSTS: only if served over HTTPS (or behind HTTPS proxy)
            if is_https && is_success {
                headers.insert(
                    header::STRICT_TRANSPORT_SECURITY,
                    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
                );
            }

            // CSP Report-Only — monitor violations without breaking functionality.
            // Upgrade to enforced CSP after a monitoring period.
            headers.insert(
                header::HeaderName::from_static("content-security-policy-report-only"),
                HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data: https:; \
                     font-src 'self'; \
                     connect-src 'self'; \
                     frame-ancestors 'self'; \
                     base-uri 'self'; \
                     form-action 'self'; \
                     report-uri /api/v1/csp-report",
                ),
            );

            Ok(response)
        })
    }
}

// ── Cache-Control Layer for hashed static assets ──

#[derive(Clone)]
pub struct AssetCacheLayer;

impl<S> Layer<S> for AssetCacheLayer {
    type Service = AssetCacheMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AssetCacheMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct AssetCacheMiddleware<S> {
    inner: S,
}

impl<S, B> tower::Service<Request<B>> for AssetCacheMiddleware<S>
where
    S: tower::Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let is_asset = req.uri().path().starts_with("/assets/");
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let mut response: Response = inner.call(req).await?;

            if is_asset && response.status().is_success() {
                // Hashed assets can be cached for 1 year
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                );
            }

            Ok(response)
        })
    }
}
