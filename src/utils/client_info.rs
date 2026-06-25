use axum::http::HeaderMap;
use std::net::SocketAddr;

/// 从请求头中提取客户端 IP（优先 X-Real-IP，其次 X-Forwarded-For，最后 fallback 到 socket addr）
pub fn extract_client_ip(headers: &HeaderMap, socket_addr: Option<SocketAddr>) -> Option<String> {
    if let Some(ip) = headers.get("x-real-ip") {
        return ip.to_str().ok().map(|s| s.to_string());
    }
    if let Some(fwd) = headers.get("x-forwarded-for") {
        if let Ok(val) = fwd.to_str() {
            return val.split(',').next().map(|s| s.trim().to_string());
        }
    }
    socket_addr.map(|a| a.ip().to_string())
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
