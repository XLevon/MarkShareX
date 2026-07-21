//! SSRF-safe URL validation for outbound HTTP requests.
//!
//! Blocks requests to loopback, private, link-local, multicast,
//! unspecified, and cloud metadata addresses (169.254.169.254).
//! DNS resolution is performed for hostnames before validation.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use crate::utils::AppError;

/// Errors returned by SSRF-safe URL validation.
#[derive(Debug, thiserror::Error)]
pub enum SafeUrlError {
    #[error("不支持的 URL 协议: {0}，仅允许 http/https")]
    UnsupportedScheme(String),

    #[error("URL 包含用户信息，拒绝")]
    UserInfoRejected,

    #[error("无法解析主机名: {0}")]
    HostResolutionFailed(String),

    #[error("目标地址为内网/环回/保留地址: {0}，拒绝")]
    PrivateAddress(String),

    #[error("无效的 URL: {0}")]
    InvalidUrl(String),
}

impl From<SafeUrlError> for AppError {
    fn from(error: SafeUrlError) -> Self {
        AppError::BadRequest(error.to_string())
    }
}

#[derive(Debug, Clone)]
struct ResolvedSafeUrl {
    url: url::Url,
    host: String,
    socket_addrs: Vec<SocketAddr>,
}

async fn resolve_safe_url(
    url_str: &str,
    allowed_nets: &[String],
) -> Result<ResolvedSafeUrl, SafeUrlError> {
    let parsed = url::Url::parse(url_str)
        .map_err(|e| SafeUrlError::InvalidUrl(format!("{e}: {url_str}")))?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(SafeUrlError::UnsupportedScheme(scheme.to_string()));
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err(SafeUrlError::UserInfoRejected);
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| SafeUrlError::InvalidUrl("URL 缺少主机名".into()))?
        .to_string();
    let ips = resolve_host(&host)
        .await
        .map_err(|e| SafeUrlError::HostResolutionFailed(format!("{host}: {e}")))?;

    for ip in &ips {
        if is_private_or_special(*ip)
            && !allowed_nets
                .iter()
                .any(|rule| crate::utils::ip_utils::ip_matches_rule(rule, &ip.to_string()))
        {
            return Err(SafeUrlError::PrivateAddress(format!("{host} → {ip}")));
        }
    }

    let socket_addrs = ips.into_iter().map(|ip| SocketAddr::new(ip, 0)).collect();
    Ok(ResolvedSafeUrl {
        url: parsed,
        host,
        socket_addrs,
    })
}

/// Validate that a URL is safe for outbound HTTP requests.
pub async fn validate_safe_url(url_str: &str) -> Result<(), SafeUrlError> {
    resolve_safe_url(url_str, &[]).await.map(|_| ())
}

/// Validate a URL and permit only explicit IP/CIDR allowlist matches.
pub async fn validate_safe_url_with_allowlist(
    url_str: &str,
    allowed_nets: &[String],
) -> Result<(), SafeUrlError> {
    resolve_safe_url(url_str, allowed_nets).await.map(|_| ())
}

fn build_pinned_client(
    target: &ResolvedSafeUrl,
    timeout: Duration,
    user_agent: Option<&str>,
) -> Result<reqwest::Client, AppError> {
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(&target.host, &target.socket_addrs);
    if let Some(user_agent) = user_agent {
        builder = builder.user_agent(user_agent);
    }
    builder
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("创建安全 HTTP 客户端失败: {e}")))
}

/// Send one request with DNS pinned to the addresses that passed validation.
/// Automatic redirects are disabled so authenticated requests never leak headers cross-origin.
pub async fn send_safe_request<F>(
    url_str: &str,
    allowed_nets: &[String],
    timeout: Duration,
    user_agent: Option<&str>,
    build_request: F,
) -> Result<reqwest::Response, AppError>
where
    F: FnOnce(&reqwest::Client, &url::Url) -> reqwest::RequestBuilder,
{
    let target = resolve_safe_url(url_str, allowed_nets).await?;
    let client = build_pinned_client(&target, timeout, user_agent)?;
    build_request(&client, &target.url)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("安全 HTTP 请求失败: {e}")))
}

/// GET with explicit, bounded redirect handling. Every Location is resolved,
/// validated and DNS-pinned again before the next connection.
pub async fn safe_get_follow_redirects(
    url_str: &str,
    allowed_nets: &[String],
    timeout: Duration,
    user_agent: Option<&str>,
) -> Result<reqwest::Response, AppError> {
    const MAX_REDIRECTS: usize = 5;
    let mut current = url::Url::parse(url_str)
        .map_err(|e| SafeUrlError::InvalidUrl(format!("{e}: {url_str}")))?;

    for redirect_count in 0..=MAX_REDIRECTS {
        let response = send_safe_request(
            current.as_str(),
            allowed_nets,
            timeout,
            user_agent,
            |client, url| client.get(url.clone()),
        )
        .await?;

        if !response.status().is_redirection() {
            return Ok(response);
        }
        if redirect_count == MAX_REDIRECTS {
            return Err(AppError::BadRequest("安全 HTTP 请求重定向次数过多".into()));
        }
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .ok_or_else(|| AppError::BadRequest("重定向响应缺少 Location".into()))?
            .to_str()
            .map_err(|_| AppError::BadRequest("重定向 Location 非法".into()))?;
        let next = current
            .join(location)
            .map_err(|e| AppError::BadRequest(format!("重定向 Location 非法: {e}")))?;
        if current.scheme() == "https" && next.scheme() != "https" {
            return Err(AppError::BadRequest("拒绝 HTTPS 降级重定向".into()));
        }
        current = next;
    }

    unreachable!("redirect loop always returns or errors")
}

// ── helpers ────────────────────────────────────────────────────────────

/// Resolve a hostname to every IPv4/IPv6 address returned by DNS.  The
/// complete set is validated and then pinned into reqwest before connecting.
async fn resolve_host(host: &str) -> Result<Vec<IpAddr>, std::io::Error> {
    // Try parsing as IP address first (no DNS needed)
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(vec![ip]);
    }

    // Resolve hostname via DNS
    let addrs: Vec<IpAddr> = tokio::net::lookup_host((host, 0))
        .await?
        .map(|sa| sa.ip())
        .collect();

    if addrs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "DNS 解析无结果",
        ));
    }

    Ok(addrs)
}

/// Return true if the IP address is in a private, loopback, link-local,
/// multicast, unspecified, or otherwise special-use range that should
/// not be reachable via SSRF.
fn is_private_or_special(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private() // Rust 当前工具链定义的非全局私有/特殊 IPv4 网段
            || v4.is_loopback() // 127.0.0.0/8
            || v4.is_link_local() // 169.254.0.0/16
            || v4.is_unspecified() // 0.0.0.0
            || v4.is_multicast() // 224.0.0.0/4
            || v4.is_broadcast() // 255.255.255.255
            // Check for 169.254.169.254 (cloud metadata) explicitly
            || (v4.octets()[0] == 169 && v4.octets()[1] == 254)
            // RFC 6598: 100.64.0.0/10 (CGNAT / shared address space)
            || (v4.octets()[0] == 100
                && (v4.octets()[1] & 0b1100_0000) == 64)
            // Documentation/test: 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
            || (v4.octets()[0] == 192 && v4.octets()[1] == 0 && v4.octets()[2] == 2)
            || (v4.octets()[0] == 198 && v4.octets()[1] == 51 && v4.octets()[2] == 100)
            || (v4.octets()[0] == 203 && v4.octets()[1] == 0 && v4.octets()[2] == 113)
            // Benchmarking: 198.18.0.0/15
            || (v4.octets()[0] == 198 && (v4.octets()[1] == 18 || v4.octets()[1] == 19))
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_private_or_special(IpAddr::V4(v4));
            }
            v6.is_loopback() // ::1
            || v6.is_unspecified() // ::
            || v6.is_multicast() // ff00::/8
            || v6.is_unique_local() // fc00::/7 (ULA, IPv6 equivalent of private)
            // fe80::/10 — link-local unicast
            || (v6.segments()[0] & 0xffc0 == 0xfe80)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Public addresses that should always pass
    #[tokio::test]
    async fn public_https_urls_pass() {
        assert!(validate_safe_url("https://example.com").await.is_ok());
        assert!(validate_safe_url("https://api.openai.com/v1/models")
            .await
            .is_ok());
        assert!(validate_safe_url("https://www.google.com").await.is_ok());
    }

    // Loopback addresses must be rejected
    #[test]
    fn ipv4_mapped_ipv6_reuses_ipv4_security_classification() {
        for address in [
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:10.0.0.1",
        ] {
            assert!(
                is_private_or_special(address.parse().unwrap()),
                "mapped special address must be rejected: {address}"
            );
        }
        assert!(!is_private_or_special("::ffff:8.8.8.8".parse().unwrap()));
    }

    #[tokio::test]
    async fn loopback_rejected() {
        assert!(validate_safe_url("http://127.0.0.1:8080").await.is_err());
        assert!(validate_safe_url("http://localhost").await.is_err());
        assert!(validate_safe_url("http://[::1]").await.is_err());
        assert!(validate_safe_url("http://[::ffff:127.0.0.1]")
            .await
            .is_err());
        assert!(validate_safe_url("http://[::ffff:169.254.169.254]")
            .await
            .is_err());
    }

    // Private addresses must be rejected
    #[tokio::test]
    async fn private_addrs_rejected() {
        assert!(validate_safe_url("http://10.0.0.1").await.is_err());
        assert!(validate_safe_url("http://192.168.1.1").await.is_err());
        assert!(validate_safe_url("http://172.16.0.1").await.is_err());
    }

    #[tokio::test]
    async fn cgnat_benchmark_and_documentation_ranges_are_rejected() {
        for address in [
            "100.64.0.1",
            "198.18.0.1",
            "192.0.2.1",
            "198.51.100.1",
            "203.0.113.1",
        ] {
            assert!(
                validate_safe_url(&format!("http://{address}"))
                    .await
                    .is_err(),
                "special address must be rejected: {address}"
            );
        }
    }

    // Cloud metadata endpoint
    #[tokio::test]
    async fn cloud_metadata_rejected() {
        assert!(
            validate_safe_url("http://169.254.169.254/latest/meta-data/")
                .await
                .is_err()
        );
    }

    // Unsupported schemes
    #[tokio::test]
    async fn unsupported_schemes_rejected() {
        assert!(validate_safe_url("ftp://example.com").await.is_err());
        assert!(validate_safe_url("file:///etc/passwd").await.is_err());
        assert!(validate_safe_url("gopher://evil.com").await.is_err());
    }

    // Userinfo in URL must be rejected
    #[tokio::test]
    async fn userinfo_rejected() {
        assert!(validate_safe_url("http://user:pass@example.com")
            .await
            .is_err());
        assert!(validate_safe_url("https://admin@evil.com").await.is_err());
    }

    // Invalid URLs
    #[tokio::test]
    async fn invalid_urls_rejected() {
        assert!(validate_safe_url("not-a-url").await.is_err());
        assert!(validate_safe_url("").await.is_err());
    }

    // Allowlist: private IP explicitly permitted
    #[tokio::test]
    async fn allowlist_permits_explicit_private() {
        assert!(validate_safe_url_with_allowlist(
            "http://192.168.1.100:11434",
            &["192.168.1.100".into()]
        )
        .await
        .is_ok());

        // But a different private IP (not in allowlist) still fails
        assert!(
            validate_safe_url_with_allowlist("http://10.0.0.1", &["192.168.1.100".into()])
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn pinned_client_uses_only_the_addresses_selected_during_validation() {
        use axum::{routing::get, Router};
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        let hits = Arc::new(AtomicUsize::new(0));
        let hits_for_handler = hits.clone();
        let app = Router::new().route(
            "/",
            get(move || {
                let hits = hits_for_handler.clone();
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    "pinned"
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = url::Url::parse(&format!("http://rebind.invalid:{}/", address.port())).unwrap();
        let target = ResolvedSafeUrl {
            url: url.clone(),
            host: "rebind.invalid".to_string(),
            socket_addrs: vec![SocketAddr::new(address.ip(), 0)],
        };
        let client = build_pinned_client(&target, Duration::from_secs(2), None).unwrap();
        let response = client.get(url).send().await.unwrap();

        assert_eq!(response.text().await.unwrap(), "pinned");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
        server.abort();
    }

    #[tokio::test]
    async fn allowlist_supports_explicit_cidr_without_unspecified_wildcards() {
        assert!(validate_safe_url_with_allowlist(
            "http://192.168.1.42:11434",
            &["192.168.1.0/24".into()]
        )
        .await
        .is_ok());
        assert!(validate_safe_url_with_allowlist(
            "http://10.0.0.1:11434",
            &["192.168.1.0/24".into()]
        )
        .await
        .is_err());
        assert!(
            validate_safe_url_with_allowlist("http://10.0.0.1:11434", &["0.0.0.0".into()])
                .await
                .is_err()
        );
    }
}
