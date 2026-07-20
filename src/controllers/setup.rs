use crate::models::entity::users;
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{extract::State, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── 默认管理员和系统设置已移至 migrations/0000000000_init_schema.sql ──

#[derive(Serialize, ToSchema)]
pub struct SetupStatus {
    pub initialized: bool,
}

#[derive(Deserialize)]
pub struct SetupRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SetupResponse {
    pub message: String,
    pub user_id: i32,
}
/// GET /api/v1/setup/status — Check initialization status

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    responses((status = 200, description = "成功", body = SetupResponse)),
    tag = "Setup"
)]
pub async fn setup_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SetupStatus>>, AppError> {
    let admin_count = users::Entity::find()
        .filter(users::Column::Role.eq("admin"))
        .filter(users::Column::DeletedAt.is_null())
        .count(&state.db)
        .await?;

    Ok(Json(ApiResponse::new(SetupStatus {
        initialized: admin_count > 0,
    })))
}
/// POST /api/v1/setup — Initialize system (create admin)

#[utoipa::path(
    post,
    path = "/api/v1/setup",
    responses((status = 200, description = "成功", body = SetupResponse)),
    tag = "Setup"
)]
pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<ApiResponse<SetupResponse>>, AppError> {
    // Check if already initialized
    let admin_count = users::Entity::find()
        .filter(users::Column::Role.eq("admin"))
        .filter(users::Column::DeletedAt.is_null())
        .count(&state.db)
        .await?;

    if admin_count > 0 {
        return Err(AppError::BadRequest(
            "系统已初始化，禁止重复操作".to_string(),
        ));
    }

    // Create admin user
    let password_hash = crate::services::auth::hash_password(&req.password)?;
    let now = crate::utils::now_local();

    let user_model = users::ActiveModel {
        username: Set(req.username),
        email: Set(req.email),
        password_hash: Set(password_hash),
        display_name: Set(req.display_name),
        avatar_url: Set(None),
        role: Set("admin".to_string()),
        bio: Set(req.bio),
        is_active: Set(true),
        last_login_at: Set(None),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let user = user_model.insert(&state.db).await?;

    Ok(Json(ApiResponse::new(SetupResponse {
        message: "初始化成功".to_string(),
        user_id: user.id,
    })))
}
