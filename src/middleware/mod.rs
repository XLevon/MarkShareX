pub mod auth;
pub mod ip_guard;

use axum::{
    extract::{ConnectInfo, Request},
    http::{header, HeaderName, HeaderValue, Method},
    response::Response,
};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Layer;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn stack(server: &crate::config::ServerConfig) -> anyhow::Result<CorsLayer> {
    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-api-key"),
        ]);

    let allowed_origins = server
        .cors_allowed_origins
        .iter()
        .map(|configured_origin| {
            let origin = configured_origin.trim();
            let parsed = url::Url::parse(origin).ok();
            let has_only_authority = origin.split_once("://").is_some_and(|(_, authority)| {
                !authority.is_empty() && !authority.contains('/')
            });
            let is_exact_http_origin = parsed.as_ref().is_some_and(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
                    && has_only_authority
                    && url.path() == "/"
                    && url.query().is_none()
                    && url.fragment().is_none()
            });
            if !is_exact_http_origin {
                anyhow::bail!("无效 CORS origin `{configured_origin}`；必须是精确 http(s) origin，不能使用通配符、路径、查询或片段");
            }
            parsed
                .expect("validated CORS URL must be present")
                .origin()
                .ascii_serialization()
                .parse::<HeaderValue>()
                .map_err(|error| {
                anyhow::anyhow!("CORS origin `{configured_origin}` 无法编码为 HTTP header: {error}")
                })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(if allowed_origins.is_empty() {
        layer
    } else {
        layer.allow_origin(AllowOrigin::list(allowed_origins))
    })
}

// ── Compression Layer ──

pub fn compression() -> tower_http::compression::CompressionLayer {
    tower_http::compression::CompressionLayer::new()
        .gzip(true)
        .br(true)
}

// ── Security Headers Layer ──

#[derive(Clone)]
pub struct SecurityHeadersLayer {
    trusted_proxies: Vec<String>,
}

impl SecurityHeadersLayer {
    pub fn new(server: &crate::config::ServerConfig) -> Self {
        Self {
            trusted_proxies: server.trusted_proxies.clone(),
        }
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersMiddleware {
            inner,
            trusted_proxies: self.trusted_proxies.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersMiddleware<S> {
    inner: S,
    trusted_proxies: Vec<String>,
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
        let trusted_proxies = self.trusted_proxies.clone();

        Box::pin(async move {
            let direct_https = req.uri().scheme_str() == Some("https");
            let peer_is_trusted = req
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .is_some_and(|ConnectInfo(peer)| {
                    crate::utils::client_info::is_trusted_proxy(peer.ip(), &trusted_proxies)
                });
            let mut forwarded_proto = req.headers().get_all("x-forwarded-proto").iter();
            let forwarded_https = peer_is_trusted
                && forwarded_proto
                    .next()
                    .and_then(|value| value.to_str().ok())
                    .is_some_and(|value| value.trim().eq_ignore_ascii_case("https"))
                && forwarded_proto.next().is_none();
            let is_https = direct_https || forwarded_https;

            let mut response: Response = inner.call(req).await?;

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
            if is_https {
                headers.insert(
                    header::STRICT_TRANSPORT_SECURITY,
                    HeaderValue::from_static("max-age=31536000"),
                );
            }

            // CSP Report-Only — monitor violations without breaking functionality.
            // Upgrade to enforced CSP after a monitoring period.
            headers.insert(
                header::HeaderName::from_static("content-security-policy-report-only"),
                HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data: https:; \
                     font-src 'self'; \
                     connect-src 'self'; \
                     object-src 'none'; \
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
