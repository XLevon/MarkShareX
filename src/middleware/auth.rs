use crate::services::auth;
use crate::utils::AppError;
use crate::utils::AppState;
use axum::extract::ConnectInfo;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use sea_orm::EntityTrait;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub role: String,
    pub status: String,
    /// 认证来源：jwt 或 api_key
    #[allow(dead_code)]
    pub auth_source: String,
    /// 用户显示名（jwt 携带，无需额外查表）
    #[allow(dead_code)]
    pub display_name: Option<String>,
}

/// Authenticated administrator. Using this extractor makes admin-only handlers
/// fail closed for JWT and API-key authentication alike.
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl TryFrom<AuthUser> for AdminUser {
    type Error = AppError;

    fn try_from(user: AuthUser) -> Result<Self, Self::Error> {
        if user.is_admin() {
            Ok(Self(user))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        AdminUser::try_from(AuthUser::from_request_parts(parts, state).await?)
    }
}

/// Authenticated administrator or sub-administrator for global content actions.
#[derive(Debug, Clone)]
pub struct PrivilegedUser(pub AuthUser);

impl TryFrom<AuthUser> for PrivilegedUser {
    type Error = AppError;

    fn try_from(user: AuthUser) -> Result<Self, Self::Error> {
        if user.is_privileged() {
            Ok(Self(user))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for PrivilegedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        PrivilegedUser::try_from(AuthUser::from_request_parts(parts, state).await?)
    }
}

/// Authenticated author, sub-administrator, or administrator. This is used for
/// owner-scoped editorial surfaces that visitors must lose immediately after a
/// database role downgrade.
#[derive(Debug, Clone)]
pub struct AuthorOrPrivilegedUser(pub AuthUser);

impl TryFrom<AuthUser> for AuthorOrPrivilegedUser {
    type Error = AppError;

    fn try_from(user: AuthUser) -> Result<Self, Self::Error> {
        if user.role == "author" || user.is_privileged() {
            Ok(Self(user))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl std::ops::Deref for AuthorOrPrivilegedUser {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthorOrPrivilegedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Self::try_from(AuthUser::from_request_parts(parts, state).await?)
    }
}

/// Optional authentication for public endpoints that expose additional data to
/// an authenticated owner or privileged user. Invalid supplied credentials are
/// rejected; only a completely absent credential is treated as anonymous.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[axum::async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let has_api_key = parts.headers.contains_key("X-API-Key");
        let has_bearer = parts.headers.contains_key("Authorization");

        if !has_api_key && !has_bearer {
            return Ok(Self(None));
        }

        AuthUser::from_request_parts(parts, state)
            .await
            .map(|user| Self(Some(user)))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 从 extensions 获取客户端 socket 地址
        let socket_addr = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0);

        // ── 优先检查 X-API-Key ──
        if let Some(value) = parts.headers.get("X-API-Key") {
            let api_key = value.to_str().map_err(|_| AppError::Unauthorized)?;
            if api_key.is_empty() {
                return Err(AppError::Unauthorized);
            }
            return Self::from_api_key(api_key, state, &parts.headers, socket_addr).await;
        }

        // ── 回退到 JWT Bearer Token ──
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = auth::verify_token(token, &state.config.auth.jwt_secret)?;
        let user = auth::current_active_user(state, claims.user_id).await?;
        Ok(AuthUser {
            user_id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
            status: user.status,
            auth_source: "jwt".to_string(),
        })
    }
}

impl AuthUser {
    /// 通过 X-API-Key 认证：在 users 表中查找匹配 api_key 的用户，并记录登录日志
    async fn from_api_key(
        key: &str,
        state: &AppState,
        headers: &HeaderMap,
        socket_addr: Option<SocketAddr>,
    ) -> Result<Self, AppError> {
        use crate::models::entity::{login_logs, users};
        use crate::utils::client_info;
        use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter, Set};

        let ip = client_info::extract_client_ip(
            headers,
            socket_addr,
            &state.config.server.trusted_proxies,
        )
        .ok()
        .flatten();
        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let device_type = user_agent
            .as_deref()
            .map(|ua| client_info::device_label(Some(ua), "api_key"));
        let now = crate::utils::now_local();

        let user = users::Entity::find()
            .filter(users::Column::ApiKey.eq(key))
            .filter(users::Column::Status.eq("active"))
            .filter(users::Column::IsActive.eq(true))
            .filter(users::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("API Key lookup failed: {}", e);
                AppError::Unauthorized
            })?;

        match user {
            Some(u) => {
                tracing::info!("API Key auth: user={} role={}", u.username, u.role);

                // 记录 API Key 登录成功日志
                let success_log = login_logs::ActiveModel {
                    user_id: Set(Some(u.id)),
                    username: Set(u.username.clone()),
                    ip_address: Set(ip),
                    user_agent: Set(user_agent),
                    device_type: Set(device_type),
                    login_method: Set("api_key".to_string()),
                    success: Set(true),
                    created_at: Set(now),
                    ..Default::default()
                };
                success_log.insert(&state.db).await.ok();

                Ok(AuthUser {
                    user_id: u.id,
                    username: u.username,
                    display_name: u.display_name,
                    role: u.role,
                    status: u.status,
                    auth_source: "api_key".to_string(),
                })
            }
            None => {
                // 记录 API Key 登录失败日志
                let key_prefix = if key.len() > 10 { &key[..10] } else { key };
                let fail_log = login_logs::ActiveModel {
                    username: Set(format!("api_key:{}...", key_prefix)),
                    ip_address: Set(ip),
                    user_agent: Set(user_agent),
                    device_type: Set(device_type),
                    login_method: Set("api_key".to_string()),
                    success: Set(false),
                    created_at: Set(now),
                    ..Default::default()
                };
                fail_log.insert(&state.db).await.ok();

                Err(AppError::Unauthorized)
            }
        }
    }
}

impl AuthUser {
    /// Returns true if the user has full admin privileges.
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Returns true if user can see all data (admin or sub_admin).
    pub fn is_privileged(&self) -> bool {
        self.role == "admin" || self.role == "sub_admin"
    }
}

/// Axum middleware: rejects non-admin users with 403 Forbidden.
/// Accepts JWT via `scalar_token` cookie (set by admin SPA before navigation).
pub async fn require_admin_middleware(
    state: axum::extract::State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    let token = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with("scalar_token="))
                .map(|c| c.trim()["scalar_token=".len()..].to_string())
        });

    let token = if let Some(token) = token {
        token
    } else {
        // Fall back to Bearer header (API calls)
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?
            .to_string()
    };

    let claims = crate::services::auth::verify_token(&token, &state.config.auth.jwt_secret)?;
    if auth::current_active_role(&state, claims.user_id).await? != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth_user(role: &str) -> AuthUser {
        AuthUser {
            user_id: 1,
            username: "tester".to_string(),
            role: role.to_string(),
            status: "active".to_string(),
            auth_source: "test".to_string(),
            display_name: None,
        }
    }

    #[test]
    fn admin_extractor_accepts_only_admin_role() {
        assert!(AdminUser::try_from(auth_user("admin")).is_ok());
        assert!(matches!(
            AdminUser::try_from(auth_user("sub_admin")),
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            AdminUser::try_from(auth_user("author")),
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            AdminUser::try_from(auth_user("visitor")),
            Err(AppError::Forbidden)
        ));
    }

    #[test]
    fn privileged_extractor_accepts_admin_and_sub_admin_only() {
        assert!(PrivilegedUser::try_from(auth_user("admin")).is_ok());
        assert!(PrivilegedUser::try_from(auth_user("sub_admin")).is_ok());
        assert!(matches!(
            PrivilegedUser::try_from(auth_user("author")),
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            PrivilegedUser::try_from(auth_user("visitor")),
            Err(AppError::Forbidden)
        ));
    }
}
