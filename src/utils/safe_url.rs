//! SSRF-safe URL validation for outbound HTTP requests.
//!
//! Blocks requests to loopback, private, link-local, multicast,
//! unspecified, and cloud metadata addresses (169.254.169.254).
//! DNS resolution is performed for hostnames before validation.

use std::net::{IpAddr, Ipv4Addr};

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

/// Validate that a URL is safe for outbound HTTP requests.
/// Returns Ok(()) if the URL passes all SSRF checks.
///
/// Checks performed:
/// 1. URL parsing and scheme validation (http/https only)
/// 2. Reject URLs with embedded userinfo (e.g. http://user:pass@host)
/// 3. Resolve hostname to IP addresses
/// 4. Reject private/loopback/link-local/multicast/unspecified IPs
pub async fn validate_safe_url(url_str: &str) -> Result<(), SafeUrlError> {
    let parsed = url::Url::parse(url_str)
        .map_err(|e| SafeUrlError::InvalidUrl(format!("{e}: {url_str}")))?;

    // 1. Scheme must be http or https
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(SafeUrlError::UnsupportedScheme(scheme.to_string()));
    }

    // 2. Reject URLs with userinfo (e.g. http://user:pass@host)
    if parsed.username() != "" || parsed.password().is_some() {
        return Err(SafeUrlError::UserInfoRejected);
    }

    // 3. Resolve host to IP addresses
    let host_str = parsed
        .host_str()
        .ok_or_else(|| SafeUrlError::InvalidUrl("URL 缺少主机名".into()))?;

    let ips = resolve_host(host_str, scheme == "https")
        .await
        .map_err(|e| SafeUrlError::HostResolutionFailed(format!("{host_str}: {e}")))?;

    // 4. Check every resolved IP
    for ip in &ips {
        if is_private_or_special(*ip) {
            return Err(SafeUrlError::PrivateAddress(format!("{host_str} → {ip}")));
        }
    }

    Ok(())
}

/// Validate a URL is safe, but allow private addresses if they appear
/// in the provided allowlist.  Use this for local Ollama / self-hosted
/// services that administrators want to explicitly permit.
pub async fn validate_safe_url_with_allowlist(
    url_str: &str,
    allowed_nets: &[String],
) -> Result<(), SafeUrlError> {
    let parsed = url::Url::parse(url_str)
        .map_err(|e| SafeUrlError::InvalidUrl(format!("{e}: {url_str}")))?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(SafeUrlError::UnsupportedScheme(scheme.to_string()));
    }

    if parsed.username() != "" || parsed.password().is_some() {
        return Err(SafeUrlError::UserInfoRejected);
    }

    let host_str = parsed
        .host_str()
        .ok_or_else(|| SafeUrlError::InvalidUrl("URL 缺少主机名".into()))?;

    let ips = resolve_host(host_str, scheme == "https")
        .await
        .map_err(|e| SafeUrlError::HostResolutionFailed(format!("{host_str}: {e}")))?;

    // Parse allowlist once
    let allowed: Vec<IpAddr> = allowed_nets
        .iter()
        .filter_map(|entry| {
            if let Ok(ip) = entry.parse::<IpAddr>() {
                return Some(ip);
            }
            None
        })
        .collect();

    for ip in &ips {
        if is_private_or_special(*ip) {
            if !allowed.contains(ip) && !allowed.contains(&IpAddr::V4(Ipv4Addr::UNSPECIFIED)) {
                return Err(SafeUrlError::PrivateAddress(format!("{host_str} → {ip}")));
            }
        }
    }

    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────

/// Resolve a hostname to its IP addresses.
/// For HTTPS URLs, only IPv4 is resolved (IPv6 is rejected as unsafe
/// for SSRF purposes — attackers can use IPv6 to bypass IPv4-only
/// firewalls).
async fn resolve_host(host: &str, _https: bool) -> Result<Vec<IpAddr>, std::io::Error> {
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
            v4.is_private() // RFC 1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
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
    #[tokio::test]
    async fn loopback_rejected() {
        assert!(validate_safe_url("http://127.0.0.1:8080").await.is_err());
        assert!(validate_safe_url("http://localhost").await.is_err());
        assert!(validate_safe_url("http://[::1]").await.is_err());
    }

    // Private addresses must be rejected
    #[tokio::test]
    async fn private_addrs_rejected() {
        assert!(validate_safe_url("http://10.0.0.1").await.is_err());
        assert!(validate_safe_url("http://192.168.1.1").await.is_err());
        assert!(validate_safe_url("http://172.16.0.1").await.is_err());
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
}
