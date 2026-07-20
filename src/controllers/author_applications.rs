use crate::middleware::auth::AuthUser;
use crate::models::entity::{author_applications, users};
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── Request/Response types ──

#[derive(Deserialize)]
pub struct ApplyRequest {
    pub reason: String,
    pub content: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApplicationResponse {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub reason: String,
    pub content_description: String,
    pub status: String,
    pub admin_remark: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct RejectRequest {
    pub remark: Option<String>,
}
/// POST /api/v1/apply — Submit author application

#[utoipa::path(
    post,
    path = "/api/v1/apply",
    responses((status = 200, description = "成功", body = ApplicationResponse)),
    tag = "Applications"
)]
pub async fn submit_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ApplyRequest>,
) -> Result<Json<ApiResponse<ApplicationResponse>>, AppError> {
    if req.reason.trim().is_empty() {
        return Err(AppError::BadRequest("申请理由不能为空".into()));
    }
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest("分享内容说明不能为空".into()));
    }

    // Check if user already has a pending application
    let existing = author_applications::Entity::find()
        .filter(author_applications::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?;

    if let Some(app) = existing {
        if app.status == "pending" {
            return Err(AppError::BadRequest(
                "你已有待审申请，请等待管理员审核".into(),
            ));
        }
        // If previously rejected, delete old record to allow re-application
        author_applications::Entity::delete_by_id(app.id)
            .exec(&state.db)
            .await?;
    }

    let now = crate::utils::now_local();
    let app = author_applications::ActiveModel {
        user_id: Set(auth.user_id),
        reason: Set(req.reason.trim().to_string()),
        content_description: Set(req.content.trim().to_string()),
        status: Set("pending".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let inserted = app.insert(&state.db).await?;

    // Get user info for response
    let user = users::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    Ok(Json(ApiResponse::new(ApplicationResponse {
        id: inserted.id,
        user_id: inserted.user_id,
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        reason: inserted.reason,
        content_description: inserted.content_description,
        status: inserted.status,
        admin_remark: inserted.admin_remark,
        created_at: inserted.created_at,
    })))
}
/// GET /api/v1/apply/status — Check application status

#[utoipa::path(
    get,
    path = "/api/v1/apply/status",
    responses((status = 200, description = "成功", body = ApplicationResponse)),
    tag = "Applications"
)]
pub async fn get_application_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Option<ApplicationResponse>>>, AppError> {
    let app = author_applications::Entity::find()
        .filter(author_applications::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?;

    if let Some(app) = app {
        let user = users::Entity::find_by_id(app.user_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound("用户不存在".into()))?;

        Ok(Json(ApiResponse::new(Some(ApplicationResponse {
            id: app.id,
            user_id: app.user_id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            reason: app.reason,
            content_description: app.content_description,
            status: app.status,
            admin_remark: app.admin_remark,
            created_at: app.created_at,
        }))))
    } else {
        Ok(Json(ApiResponse::new(None)))
    }
}
/// POST /api/v1/admin/applications/{id}/approve — Approve application

#[utoipa::path(
    post,
    path = "/api/v1/admin/applications/{id}/approve",
    responses((status = 200, description = "成功")),
    tag = "Applications"
)]
pub async fn approve_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<ApplicationResponse>>, AppError> {
    let app = author_applications::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("申请不存在".into()))?;

    if app.status != "pending" {
        return Err(AppError::BadRequest("该申请已被处理".into()));
    }

    let now = crate::utils::now_local();

    // Update application status
    let mut active_app: author_applications::ActiveModel = app.clone().into();
    active_app.status = Set("approved".to_string());
    active_app.reviewed_by = Set(Some(auth.user_id));
    active_app.reviewed_at = Set(Some(now));
    active_app.updated_at = Set(now);
    let updated_app = active_app.update(&state.db).await?;

    // Upgrade user role to author
    let user = users::Entity::find_by_id(app.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    let mut active_user: users::ActiveModel = user.clone().into();
    active_user.role = Set("author".to_string());
    active_user.updated_at = Set(now);
    active_user.update(&state.db).await?;

    Ok(Json(ApiResponse::new(ApplicationResponse {
        id: updated_app.id,
        user_id: updated_app.user_id,
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        reason: updated_app.reason,
        content_description: updated_app.content_description,
        status: updated_app.status,
        admin_remark: updated_app.admin_remark,
        created_at: updated_app.created_at,
    })))
}
/// POST /api/v1/admin/applications/{id}/reject — Reject application

#[utoipa::path(
    post,
    path = "/api/v1/admin/applications/{id}/reject",
    responses((status = 200, description = "成功")),
    tag = "Applications"
)]
pub async fn reject_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<ApiResponse<ApplicationResponse>>, AppError> {
    let app = author_applications::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("申请不存在".into()))?;

    if app.status != "pending" {
        return Err(AppError::BadRequest("该申请已被处理".into()));
    }

    let now = crate::utils::now_local();

    let mut active_app: author_applications::ActiveModel = app.clone().into();
    active_app.status = Set("rejected".to_string());
    active_app.admin_remark = Set(req.remark.filter(|r| !r.trim().is_empty()));
    active_app.reviewed_by = Set(Some(auth.user_id));
    active_app.reviewed_at = Set(Some(now));
    active_app.updated_at = Set(now);
    let updated_app = active_app.update(&state.db).await?;

    let user = users::Entity::find_by_id(app.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    Ok(Json(ApiResponse::new(ApplicationResponse {
        id: updated_app.id,
        user_id: updated_app.user_id,
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        reason: updated_app.reason,
        content_description: updated_app.content_description,
        status: updated_app.status,
        admin_remark: updated_app.admin_remark,
        created_at: updated_app.created_at,
    })))
}
/// GET /api/v1/admin/applications/pending-count — Pending applications count

#[utoipa::path(
    get,
    path = "/api/v1/admin/applications/pending-count",
    responses((status = 200, description = "成功")),
    tag = "Applications"
)]
pub async fn get_pending_count(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<u64>>, AppError> {
    let count = author_applications::Entity::find()
        .filter(author_applications::Column::Status.eq("pending"))
        .count(&state.db)
        .await?;

    Ok(Json(ApiResponse::new(count)))
}
