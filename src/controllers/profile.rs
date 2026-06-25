use axum::{extract::State, Json};
use sea_orm::*;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::AuthUser;
use crate::models::entity::users;
use crate::models::entity::settings;
use crate::services;
use crate::utils::{AppError, AppState, ApiResponse};

// ── GET /api/v1/profile ────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub status: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub title: Option<String>,
    pub created_at: String,
    pub last_login_at: Option<String>,
}
/// GET /api/v1/profile — Get current user profile

#[utoipa::path(
    get,
    path = "/api/v1/profile",
    responses((status = 200, description = "成功", body = ProfileResponse)),
    tag = "Profile"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<ProfileResponse>>, AppError> {
    let user = users::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    Ok(Json(ApiResponse::new(ProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        role: user.role,
        status: user.status,
        avatar_url: user.avatar_url,
        bio: user.bio,
        title: user.title,
        created_at: user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        last_login_at: user.last_login_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    })))
}

// ── PUT /api/v1/profile ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub title: Option<String>,
}
/// PUT /api/v1/profile — Update current user profile

#[utoipa::path(
    put,
    path = "/api/v1/profile",
    responses((status = 200, description = "成功", body = ProfileResponse)),
    tag = "Profile"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<ProfileResponse>>, AppError> {
    let user = users::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    // Validate email uniqueness if changed
    if let Some(ref email) = req.email {
        if email.trim().is_empty() {
            return Err(AppError::BadRequest("邮箱不能为空".into()));
        }
        if email != &user.email {
            let existing = users::Entity::find()
                .filter(users::Column::Email.eq(email))
                .filter(users::Column::Id.ne(auth.user_id))
                .filter(users::Column::DeletedAt.is_null())
                .one(&state.db)
                .await?;
            if existing.is_some() {
                return Err(AppError::BadRequest("邮箱已被使用".into()));
            }
        }
    }

    // Validate display_name uniqueness if changed and not empty
    if let Some(ref name) = req.display_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() && Some(trimmed) != user.display_name.as_deref() {
            let existing = users::Entity::find()
                .filter(users::Column::DisplayName.eq(trimmed))
                .filter(users::Column::Id.ne(auth.user_id))
                .filter(users::Column::DeletedAt.is_null())
                .one(&state.db)
                .await?;
            if existing.is_some() {
                return Err(AppError::BadRequest("昵称已被占用".into()));
            }
        }
    }

    let now = crate::utils::now_local();
    let mut active_model: users::ActiveModel = user.into();

    if let Some(name) = req.display_name {
        active_model.display_name = Set(if name.trim().is_empty() { None } else { Some(name.trim().to_string()) });
    }
    if let Some(email) = req.email {
        active_model.email = Set(email.trim().to_string());
    }
    if let Some(bio) = req.bio {
        active_model.bio = Set(if bio.trim().is_empty() { None } else { Some(bio.trim().to_string()) });
    }
    if let Some(title) = req.title {
        active_model.title = Set(if title.trim().is_empty() { None } else { Some(title.trim().to_string()) });
    }

    active_model.updated_at = Set(now);
    let updated = active_model.update(&state.db).await?;

    Ok(Json(ApiResponse::new(ProfileResponse {
        id: updated.id,
        username: updated.username,
        email: updated.email,
        display_name: updated.display_name,
        role: updated.role,
        status: updated.status,
        avatar_url: updated.avatar_url,
        bio: updated.bio,
        title: updated.title,
        created_at: updated.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        last_login_at: updated.last_login_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    })))
}

// ── GET /api/v1/site/admin-info ──────────────────────────────────────

/// Public endpoint — no auth required.
/// Returns admin user's display_name, avatar_url, and bio for the homepage.
#[derive(Serialize, ToSchema)]
pub struct SiteManagerResponse {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
}
/// GET /api/v1/site/admin-info — Get site manager public info

#[utoipa::path(
    get,
    path = "/api/v1/site/admin-info",
    responses((status = 200, description = "成功", body = SiteManagerResponse)),
    tag = "Profile"
)]
pub async fn get_site_manager_info(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SiteManagerResponse>>, AppError> {
    // Check if a specific webmaster user is configured
    let webmaster_id: Option<i32> = settings::Entity::find()
        .filter(settings::Column::Key.eq("site-manager"))
        .one(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|s| s.value.parse().ok());

    let admin = if let Some(uid) = webmaster_id {
        users::Entity::find_by_id(uid)
            .filter(users::Column::IsActive.eq(true))
            .filter(users::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?
    } else {
        None
    };

    // Fallback: first active admin
    let admin = match admin {
        Some(a) => Some(a),
        None => users::Entity::find()
            .filter(users::Column::Role.eq("admin"))
            .filter(users::Column::IsActive.eq(true))
            .filter(users::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?,
    };

    match admin {
        Some(a) => Ok(Json(ApiResponse::new(SiteManagerResponse {
            display_name: a.display_name,
            avatar_url: a.avatar_url,
            bio: a.bio,
            title: a.title,
            email: Some(a.email),
        }))),
        None => Ok(Json(ApiResponse::new(SiteManagerResponse {
            display_name: None,
            avatar_url: None,
            bio: None,
            title: None,
            email: None,
        }))),
    }
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}
/// PUT /api/v1/profile/password — Change password

#[utoipa::path(
    put,
    path = "/api/v1/profile/password",
    responses((status = 200, description = "成功")),
    tag = "Profile"
)]
pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // Validate new password
    if req.new_password.len() < 8 {
        return Err(AppError::BadRequest("新密码至少8位".into()));
    }

    // Check confirmation
    if req.new_password != req.confirm_password {
        return Err(AppError::BadRequest("两次输入的新密码不一致".into()));
    }

    // Fetch user
    let user = users::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    // Verify old password
    if !services::auth::verify_password(&req.old_password, &user.password_hash)? {
        return Err(AppError::BadRequest("旧密码错误".into()));
    }

    // Hash new password and update
    let new_hash = services::auth::hash_password(&req.new_password)?;
    let now = crate::utils::now_local();

    let mut active_model: users::ActiveModel = user.into();
    active_model.password_hash = Set(new_hash);
    active_model.updated_at = Set(now);
    active_model.update(&state.db).await?;

    Ok(Json(ApiResponse::new(())))
}

// ── GET /api/v1/profile/api-key ──────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub api_key: Option<String>,
}
/// GET /api/v1/profile/api-key — Get API key

#[utoipa::path(
    get,
    path = "/api/v1/profile/api-key",
    responses((status = 200, description = "成功", body = ApiKeyResponse)),
    tag = "Profile"
)]
pub async fn get_api_key(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<ApiKeyResponse>>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let user = users::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    Ok(Json(ApiResponse::new(ApiKeyResponse {
        api_key: user.api_key,
    })))
}
/// PUT /api/v1/profile/api-key — Generate or reset API key

#[utoipa::path(
    put,
    path = "/api/v1/profile/api-key",
    responses((status = 200, description = "成功", body = ApiKeyResponse)),
    tag = "Profile"
)]
pub async fn regenerate_api_key(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<ApiKeyResponse>>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let user = users::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    let new_key = generate_api_key();

    let now = crate::utils::now_local();
    let mut active_model: users::ActiveModel = user.into();
    active_model.api_key = Set(Some(new_key.clone()));
    active_model.updated_at = Set(now);
    active_model.update(&state.db).await?;

    Ok(Json(ApiResponse::new(ApiKeyResponse {
        api_key: Some(new_key),
    })))
}

fn generate_api_key() -> String {
    format!("msx-{}", uuid::Uuid::new_v4())
}
