use axum::{extract::State, Json, http::HeaderMap, extract::ConnectInfo};
use std::net::SocketAddr;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::utils::{AppState, AppError, ApiResponse, client_info};
use crate::services;
use crate::models::entity::{users, refresh_tokens, login_logs};
use sea_orm::*;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub status: String,
    pub avatar_url: Option<String>,
}

impl From<users::Model> for UserResponse {
    fn from(u: users::Model) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            display_name: u.display_name,
            role: u.role,
            status: u.status.clone(),
            avatar_url: u.avatar_url,
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub remember_me: Option<bool>,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
/// POST /api/v1/auth/register — Register new account

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    responses((status = 200, description = "成功", body = LoginResponse)),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    // Validation
    if req.username.trim().is_empty() || req.email.trim().is_empty() || req.password.len() < 8 {
        return Err(AppError::BadRequest("用户名、邮箱不能为空，密码至少8位".into()));
    }

    // Check existing user
    let existing = users::Entity::find()
        .filter(
            users::Column::Username.eq(&req.username)
                .or(users::Column::Email.eq(&req.email))
        )
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("用户名或邮箱已被注册".into()));
    }

    let password_hash = services::auth::hash_password(&req.password)?;
    let now = crate::utils::now_local();

    let user_model = users::ActiveModel {
        username: Set(req.username.trim().to_string()),
        email: Set(req.email.trim().to_string()),
        password_hash: Set(password_hash),
        display_name: Set(req.display_name.filter(|n| !n.trim().is_empty())),
        role: Set("visitor".to_string()),
        status: Set("active".to_string()),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let user = match user_model.insert(&state.db).await {
        Ok(user) => user,
        Err(e) => {
            // Catch UNIQUE constraint violations (race condition: check passed but insert failed)
            if e.to_string().contains("UNIQUE constraint failed") {
                return Err(AppError::BadRequest("用户名或邮箱已被注册".into()));
            }
            return Err(e.into());
        }
    };

    let token = services::auth::generate_token(
        user.id,
        &user.username,
        user.display_name.clone(),
        &user.role,
        &user.status,
        &state.config.auth,
    )?;

    let refresh_token_str = uuid::Uuid::new_v4().to_string();
    let now = crate::utils::now_local();
    // 记住我：30 天，否则按配置（默认 7 天）
    let refresh_secs = if req.remember_me.unwrap_or(false) {
        2592000  // 30 days
    } else {
        state.config.auth.refresh_expire_seconds
    };
    let expires_at = chrono::Local::now()
        .checked_add_signed(chrono::Duration::seconds(refresh_secs))
        .map(|dt| dt.naive_utc())
        .unwrap_or(now);

    let refresh_model = refresh_tokens::ActiveModel {
        user_id: Set(user.id),
        token: Set(refresh_token_str.clone()),
        expires_at: Set(expires_at),
        revoked: Set(false),
        created_at: Set(now),
        ..Default::default()
    };
    refresh_model.insert(&state.db).await?;

    Ok(Json(ApiResponse::new(LoginResponse {
        access_token: token,
        refresh_token: refresh_token_str,
        user: UserResponse::from(user),
    })))
}
/// POST /api/v1/auth/login — Login and get JWT token

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    responses((status = 200, description = "成功", body = LoginResponse)),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let ip = client_info::extract_client_ip(&headers, Some(socket_addr));
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let device_type = user_agent.as_deref()
        .map(|ua| client_info::device_label(Some(ua), "password"));

    // Try username first, then email
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&req.username))
        .filter(users::Column::IsActive.eq(true))
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?;

    // If not found by username, try email
    let user = match user {
        Some(u) => Some(u),
        None => users::Entity::find()
            .filter(users::Column::Email.eq(&req.username))
            .filter(users::Column::IsActive.eq(true))
            .filter(users::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?,
    };

    let user = match user {
        Some(u) => u,
        None => {
            // 记录失败日志
            let now = crate::utils::now_local();
            let fail_log = login_logs::ActiveModel {
                username: Set(req.username),
                ip_address: Set(ip),
                user_agent: Set(user_agent),
                device_type: Set(device_type),
                login_method: Set("password".to_string()),
                success: Set(false),
                created_at: Set(now),
                ..Default::default()
            };
            fail_log.insert(&state.db).await.ok();
            return Err(AppError::Unauthorized);
        }
    };

    // Check if user is banned
    if user.status == "banned" {
        return Err(AppError::BadRequest("账号已被拉黑，无法登录".into()));
    }

    if !services::auth::verify_password(&req.password, &user.password_hash)? {
        // 记录失败日志
        let now = crate::utils::now_local();
        let fail_log = login_logs::ActiveModel {
            user_id: Set(Some(user.id)),
            username: Set(user.username.clone()),
            ip_address: Set(ip),
            user_agent: Set(user_agent),
            device_type: Set(device_type),
            login_method: Set("password".to_string()),
            success: Set(false),
            created_at: Set(now),
            ..Default::default()
        };
        fail_log.insert(&state.db).await.ok();
        return Err(AppError::Unauthorized);
    }

    let token = services::auth::generate_token(
        user.id,
        &user.username,
        user.display_name.clone(),
        &user.role,
        &user.status,
        &state.config.auth,
    )?;

    // Generate refresh token
    let refresh_token_str = uuid::Uuid::new_v4().to_string();
    let now = crate::utils::now_local();
    // 记住我：30 天，否则按配置（默认 7 天）
    let refresh_secs = if req.remember_me.unwrap_or(false) {
        2592000  // 30 days
    } else {
        state.config.auth.refresh_expire_seconds
    };
    let expires_at = chrono::Local::now()
        .checked_add_signed(chrono::Duration::seconds(refresh_secs))
        .map(|dt| dt.naive_utc())
        .unwrap_or(now);

    let refresh_model = refresh_tokens::ActiveModel {
        user_id: Set(user.id),
        token: Set(refresh_token_str.clone()),
        expires_at: Set(expires_at),
        revoked: Set(false),
        created_at: Set(now),
        ..Default::default()
    };
    refresh_model.insert(&state.db).await?;

    // Update last login
    let mut user_active: users::ActiveModel = user.clone().into();
    user_active.last_login_at = Set(Some(now));
    user_active.update(&state.db).await?;

    // 记录成功日志
    let success_log = login_logs::ActiveModel {
        user_id: Set(Some(user.id)),
        username: Set(user.username.clone()),
        ip_address: Set(ip),
        user_agent: Set(user_agent),
        device_type: Set(device_type),
        login_method: Set("password".to_string()),
        success: Set(true),
        created_at: Set(now),
        ..Default::default()
    };
    success_log.insert(&state.db).await.ok();

    Ok(Json(ApiResponse::new(LoginResponse {
        access_token: token,
        refresh_token: refresh_token_str,
        user: UserResponse::from(user),
    })))
}
/// POST /api/v1/auth/refresh — Refresh JWT token

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    responses((status = 200, description = "成功", body = LoginResponse)),
    tag = "Auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let rt = refresh_tokens::Entity::find()
        .filter(refresh_tokens::Column::Token.eq(&req.refresh_token))
        .filter(refresh_tokens::Column::Revoked.eq(false))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if rt.expires_at < crate::utils::now_local() {
        return Err(AppError::Unauthorized);
    }

    let user = users::Entity::find_by_id(rt.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !user.is_active || user.deleted_at.is_some() {
        return Err(AppError::Unauthorized);
    }

    // Revoke old refresh token
    let mut rt_active: refresh_tokens::ActiveModel = rt.into();
    rt_active.revoked = Set(true);
    rt_active.update(&state.db).await?;

    // Generate new tokens
    let token = services::auth::generate_token(
        user.id,
        &user.username,
        user.display_name.clone(),
        &user.role,
        &user.status,
        &state.config.auth,
    )?;

    let new_refresh = uuid::Uuid::new_v4().to_string();
    let now = crate::utils::now_local();
    let expires_at = chrono::Local::now()
        .checked_add_signed(chrono::Duration::seconds(
            state.config.auth.refresh_expire_seconds,
        ))
        .map(|dt| dt.naive_utc())
        .unwrap_or(now);

    let refresh_model = refresh_tokens::ActiveModel {
        user_id: Set(user.id),
        token: Set(new_refresh.clone()),
        expires_at: Set(expires_at),
        revoked: Set(false),
        created_at: Set(now),
        ..Default::default()
    };
    refresh_model.insert(&state.db).await?;

    Ok(Json(ApiResponse::new(LoginResponse {
        access_token: token,
        refresh_token: new_refresh,
        user: UserResponse::from(user),
    })))
}
