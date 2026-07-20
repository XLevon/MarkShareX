use crate::utils::{client_info, ip_utils, AppState};
use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::EntityTrait;
use std::net::SocketAddr;

/// IP 黑白名单中间件
///
/// 检查顺序：
/// 1. 黑名单优先 — 匹配则返回 403
/// 2. 白名单仅限制 API Key 访问 — X-API-Key 请求但 IP 不在白名单 → 403
pub async fn ip_guard_middleware(
    state: axum::extract::State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let socket_addr = connect_info.0;
    let ip = match client_info::extract_client_ip(request.headers(), Some(socket_addr)) {
        Some(ip) => ip,
        None => return Ok(next.run(request).await), // 无法获取 IP 则放行
    };

    // 从 settings 表读取 IP 规则（读缓存方式，每次都查 DB）
    let settings = crate::models::entity::settings::Entity::find()
        .all(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "配置读取失败".into()))?;

    let get = |key: &str| -> Option<String> {
        settings
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.value.clone())
    };

    // ── 黑名单检查 ──
    let blacklist_enabled = get("ip_blacklist_enabled")
        .map(|v| v == "true")
        .unwrap_or(false);
    if blacklist_enabled {
        if let Some(blacklist_json) = get("ip_blacklist") {
            let blacklist = ip_utils::parse_valid_ips(&blacklist_json);
            if blacklist.contains(&ip) {
                return Err((StatusCode::FORBIDDEN, "访问被拒绝".into()));
            }
        }
    }

    // ── 白名单检查（仅限制 X-API-Key 请求）──
    let whitelist_enabled = get("ip_whitelist_enabled")
        .map(|v| v == "true")
        .unwrap_or(false);
    if whitelist_enabled {
        let is_api_key_request = request.headers().get("X-API-Key").is_some();
        if is_api_key_request {
            if let Some(whitelist_json) = get("ip_whitelist") {
                let whitelist = ip_utils::parse_valid_ips(&whitelist_json);
                if whitelist.is_empty() || !whitelist.contains(&ip) {
                    return Err((StatusCode::FORBIDDEN, "IP不在白名单中".into()));
                }
            } else {
                // 白名单启用但列表为空 → 拒绝所有 API Key 请求
                return Err((StatusCode::FORBIDDEN, "IP不在白名单中".into()));
            }
        }
    }

    Ok(next.run(request).await)
}
