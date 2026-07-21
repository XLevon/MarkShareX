use axum::http::HeaderMap;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIpError {
    InvalidForwardedFor,
    MultipleForwardedForHeaders,
}

fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ipv6) => ipv6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ipv6)),
        ipv4 => ipv4,
    }
}

fn parse_ip(value: &str) -> Option<IpAddr> {
    value.trim().parse::<IpAddr>().ok().map(normalize_ip)
}

fn is_trusted_proxy(ip: IpAddr, trusted_proxies: &[String]) -> bool {
    trusted_proxies
        .iter()
        .filter_map(|trusted| parse_ip(trusted))
        .any(|trusted| trusted == ip)
}

/// 从请求中提取客户端 IP。只有显式可信的代理 socket 才能提供转发头。
///
/// 对可信代理，从 X-Forwarded-For 右向左剥离可信跳点；可信边界内的
/// 非法值或重复 header 会返回错误，由访问控制层 fail closed 拒绝请求。
pub fn extract_client_ip(
    headers: &HeaderMap,
    socket_addr: Option<SocketAddr>,
    trusted_proxies: &[String],
) -> Result<Option<String>, ClientIpError> {
    let peer_ip = socket_addr.map(|addr| normalize_ip(addr.ip()));
    let peer_is_trusted = peer_ip.is_some_and(|ip| is_trusted_proxy(ip, trusted_proxies));

    if !peer_is_trusted {
        return Ok(peer_ip.map(|ip| ip.to_string()));
    }

    let mut forwarded_values = headers.get_all("x-forwarded-for").iter();
    let Some(header) = forwarded_values.next() else {
        return Ok(peer_ip.map(|ip| ip.to_string()));
    };
    if forwarded_values.next().is_some() {
        return Err(ClientIpError::MultipleForwardedForHeaders);
    }

    let value = header
        .to_str()
        .map_err(|_| ClientIpError::InvalidForwardedFor)?;
    if value.trim().is_empty() {
        return Err(ClientIpError::InvalidForwardedFor);
    }

    let mut leftmost = None;
    for raw_ip in value.split(',').rev() {
        let ip = parse_ip(raw_ip).ok_or(ClientIpError::InvalidForwardedFor)?;
        leftmost = Some(ip);
        if !is_trusted_proxy(ip, trusted_proxies) {
            return Ok(Some(ip.to_string()));
        }
    }

    Ok(leftmost.or(peer_ip).map(|ip| ip.to_string()))
}

/// 从 User-Agent 判断设备类型（简要描述）
pub fn parse_device_type(user_agent: &str) -> &'static str {
    let ua = user_agent.to_lowercase();
    // Mobile
    if ua.contains("iphone") || ua.contains("ipod") {
        return "iPhone";
    }
    if ua.contains("android") && ua.contains("mobile") {
        return "Android";
    }
    if ua.contains("mobile") {
        return "Mobile";
    }
    // Tablet
    if ua.contains("ipad") {
        return "iPad";
    }
    if ua.contains("android") && !ua.contains("mobile") {
        return "Android Tablet";
    }
    if ua.contains("tablet") || ua.contains("kindle") {
        return "Tablet";
    }
    // Mac
    if ua.contains("macintosh") || ua.contains("mac os") || ua.contains("macos") {
        return "Mac";
    }
    // PC / Desktop
    if ua.contains("windows") || ua.contains("linux") || ua.contains("x11") {
        return "PC";
    }
    "Desktop"
}

/// 判断是否为 API 请求（无 User-Agent 或包含特定标识）
pub fn is_api_client(user_agent: Option<&str>, login_method: &str) -> bool {
    if login_method == "api_key" {
        return true;
    }
    if let Some(ua) = user_agent {
        let ua_lower = ua.to_lowercase();
        // 常见 HTTP 客户端库
        ua_lower.contains("python-requests")
            || ua_lower.contains("curl")
            || ua_lower.contains("axios")
            || ua_lower.contains("node-fetch")
            || ua_lower.contains("okhttp")
            || ua_lower.contains("go-http-client")
    } else {
        true // 无 UA 通常是 API 调用
    }
}

/// 获取设备类型简短标签（结合 login_method 判断）
pub fn device_label(user_agent: Option<&str>, login_method: &str) -> String {
    if is_api_client(user_agent, login_method) {
        return "API".to_string();
    }
    if let Some(ua) = user_agent {
        parse_device_type(ua).to_string()
    } else {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::net::{IpAddr, Ipv4Addr};

    fn socket(ip: [u8; 4]) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), 8080)
    }

    #[test]
    fn untrusted_peer_cannot_spoof_forwarded_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.10"));
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.20"));

        assert_eq!(
            extract_client_ip(&headers, Some(socket([192, 0, 2, 30])), &[]),
            Ok(Some("192.0.2.30".to_string()))
        );
    }

    #[test]
    fn trusted_peer_ignores_spoofed_leftmost_forwarded_ip_and_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.10"));
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.0.2.99, 203.0.113.20"),
        );

        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &["127.0.0.1".to_string()]
            ),
            Ok(Some("203.0.113.20".to_string()))
        );
    }

    #[test]
    fn trusted_proxy_chain_is_stripped_from_right_to_left() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.0.2.99, 203.0.113.20, 10.0.0.2"),
        );

        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &["127.0.0.1".to_string(), "10.0.0.2".to_string()]
            ),
            Ok(Some("203.0.113.20".to_string()))
        );
    }

    #[test]
    fn invalid_leftmost_value_cannot_poison_known_client_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("not-an-ip, 203.0.113.20"),
        );
        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &["127.0.0.1".to_string()]
            ),
            Ok(Some("203.0.113.20".to_string()))
        );
    }

    #[test]
    fn invalid_value_inside_trusted_boundary_is_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.20, not-an-ip"),
        );
        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &["127.0.0.1".to_string()]
            ),
            Err(ClientIpError::InvalidForwardedFor)
        );
    }

    #[test]
    fn all_trusted_chain_returns_leftmost_source() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("10.0.0.2, 10.0.0.3"),
        );
        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &[
                    "127.0.0.1".to_string(),
                    "10.0.0.2".to_string(),
                    "10.0.0.3".to_string(),
                ]
            ),
            Ok(Some("10.0.0.2".to_string()))
        );
    }

    #[test]
    fn duplicate_forwarded_headers_are_rejected() {
        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", HeaderValue::from_static("203.0.113.20"));
        headers.append("x-forwarded-for", HeaderValue::from_static("198.51.100.10"));
        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &["127.0.0.1".to_string()]
            ),
            Err(ClientIpError::MultipleForwardedForHeaders)
        );
    }

    #[test]
    fn real_ip_only_falls_back_to_peer() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.10"));
        assert_eq!(
            extract_client_ip(
                &headers,
                Some(socket([127, 0, 0, 1])),
                &["127.0.0.1".to_string()]
            ),
            Ok(Some("127.0.0.1".to_string()))
        );
    }

    #[test]
    fn ipv4_mapped_ipv6_peer_matches_ipv4_trusted_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.20"));
        let peer = SocketAddr::new(IpAddr::V6(Ipv4Addr::LOCALHOST.to_ipv6_mapped()), 8080);

        assert_eq!(
            extract_client_ip(&headers, Some(peer), &["127.0.0.1".to_string()]),
            Ok(Some("203.0.113.20".to_string()))
        );
    }
}
