use crate::utils::{client_info, ip_utils, AppState, IpGuardRules};
use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::EntityTrait;
use std::net::SocketAddr;
use tokio::sync::RwLock;

/// Invalidate the global IP rules cache.  Call after updating
/// ip_blacklist / ip_whitelist settings in tests or after admin changes.
pub async fn invalidate_ip_rules_cache() {
    let now = std::time::Instant::now() - std::time::Duration::from_secs(61);
    let mut cache = IP_RULES_CACHE.write().await;
    *cache = (now, IpGuardRules::default());
}

/// In-memory cache for IP guard rules, refreshed every 60 seconds.
pub static IP_RULES_CACHE: std::sync::LazyLock<RwLock<(std::time::Instant, IpGuardRules)>> =
    std::sync::LazyLock::new(|| {
        RwLock::new((
            std::time::Instant::now() - std::time::Duration::from_secs(61), // expired
            IpGuardRules::default(),
        ))
    });

async fn get_ip_rules(state: &AppState) -> Result<IpGuardRules, (StatusCode, String)> {
    // Check cache first
    {
        let cache = IP_RULES_CACHE.read().await;
        if cache.0.elapsed() < std::time::Duration::from_secs(60) {
            return Ok(cache.1.clone());
        }
    }

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
        blacklist_enabled: get("ip_blacklist_enabled").map(|v| v == "true").unwrap_or(false),
        blacklist: get("ip_blacklist")
            .map(|json| ip_utils::parse_valid_ips(&json))
            .unwrap_or_default(),
        whitelist_enabled: get("ip_whitelist_enabled").map(|v| v == "true").unwrap_or(false),
        whitelist: get("ip_whitelist")
            .map(|json| ip_utils::parse_valid_ips(&json))
            .unwrap_or_default(),
    };

    let mut cache = IP_RULES_CACHE.write().await;
    *cache = (std::time::Instant::now(), rules.clone());
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
    let ip = match client_info::extract_client_ip(request.headers(), Some(socket_addr)) {
        Some(ip) => ip,
        None => return Ok(next.run(request).await),
    };

    let rules = get_ip_rules(&state).await?;

    // ── 黑名单检查 ──
    if rules.blacklist_enabled && rules.blacklist.contains(&ip) {
        return Err((StatusCode::FORBIDDEN, "访问被拒绝".into()));
    }

    // ── 白名单检查（仅限制 X-API-Key 请求）──
    if rules.whitelist_enabled {
        let is_api_key_request = request.headers().get("X-API-Key").is_some();
        if is_api_key_request {
            if rules.whitelist.is_empty() || !rules.whitelist.contains(&ip) {
                return Err((StatusCode::FORBIDDEN, "IP不在白名单中".into()));
            }
        }
    }

    Ok(next.run(request).await)
}
