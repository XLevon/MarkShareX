use crate::utils::{client_info, ip_utils, AppState, IpGuardRules};
use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::EntityTrait;
use std::net::SocketAddr;

async fn get_ip_rules(state: &AppState) -> Result<IpGuardRules, (StatusCode, String)> {
    // Check cache first
    let cache_generation = {
        let cache = state.ip_guard_rules_cache.read().await;
        if let Some((loaded_at, rules)) = cache.entry() {
            if loaded_at.elapsed() < std::time::Duration::from_secs(60) {
                return Ok(rules.clone());
            }
        }
        cache.generation()
    };

    // Cache miss — load from DB
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

    let rules = IpGuardRules {
        blacklist_enabled: get("ip_blacklist_enabled")
            .map(|v| v == "true")
            .unwrap_or(false),
        blacklist: get("ip_blacklist")
            .map(|json| ip_utils::parse_valid_ips(&json))
            .unwrap_or_default(),
        whitelist_enabled: get("ip_whitelist_enabled")
            .map(|v| v == "true")
            .unwrap_or(false),
        whitelist: get("ip_whitelist")
            .map(|json| ip_utils::parse_valid_ips(&json))
            .unwrap_or_default(),
    };

    let mut cache = state.ip_guard_rules_cache.write().await;
    cache.store_if_current(cache_generation, rules.clone());
    Ok(rules)
}

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
    let ip = match client_info::extract_client_ip(
        request.headers(),
        Some(socket_addr),
        &state.config.server.trusted_proxies,
    ) {
        Ok(Some(ip)) => ip,
        Ok(None) => return Ok(next.run(request).await),
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "无效的 X-Forwarded-For 请求头".into(),
            ));
        }
    };

    let rules = get_ip_rules(&state).await?;

    // ── 黑名单检查 ──
    if rules.blacklist_enabled
        && rules
            .blacklist
            .iter()
            .any(|rule| ip_utils::ip_matches_rule(rule, &ip))
    {
        return Err((StatusCode::FORBIDDEN, "访问被拒绝".into()));
    }

    // ── 白名单检查（仅限制 X-API-Key 请求）──
    if rules.whitelist_enabled {
        let is_api_key_request = request.headers().get("X-API-Key").is_some();
        if is_api_key_request {
            if rules.whitelist.is_empty()
                || !rules
                    .whitelist
                    .iter()
                    .any(|rule| ip_utils::ip_matches_rule(rule, &ip))
            {
                return Err((StatusCode::FORBIDDEN, "IP不在白名单中".into()));
            }
        }
    }

    Ok(next.run(request).await)
}
