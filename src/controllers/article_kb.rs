use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::middleware::auth::AuthUser;
use crate::models::entity::{article_statuses, article_types, posts};
use crate::utils::{ApiResponse, AppError, AppState};
use sea_orm::*;

// ─── Response with post_count ───

#[derive(Debug, Serialize)]
pub struct ArticleTypeWithCount {
    #[serde(flatten)]
    pub item: article_types::Model,
    pub post_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ArticleStatusWithCount {
    #[serde(flatten)]
    pub item: article_statuses::Model,
    pub post_count: i64,
}

// ─── DTOs ───

#[derive(Debug, Deserialize)]
pub struct CreateArticleType {
    pub code: String,
    pub display_name: String,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateArticleStatus {
    pub code: String,
    pub display_name: String,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleType {
    pub display_name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleStatus {
    pub display_name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub ids: Vec<i32>,
}

// ─── Helpers ───

async fn count_posts_by_type(db: &DatabaseConnection, code: &str) -> i64 {
    posts::Entity::find()
        .filter(posts::Column::ArticleType.eq(code))
        .count(db)
        .await
        .unwrap_or(0) as i64
}

async fn count_posts_by_status(db: &DatabaseConnection, code: &str) -> i64 {
    posts::Entity::find()
        .filter(posts::Column::ArticleStatus.eq(code))
        .count(db)
        .await
        .unwrap_or(0) as i64
}

// ─── Article Types ───

/// GET /api/v1/article-types — public list (active only, with post_count)
pub async fn list_article_types(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ArticleTypeWithCount>>>, AppError> {
    let items = article_types::Entity::find()
        .filter(article_types::Column::IsActive.eq(true))
        .order_by_asc(article_types::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;

    let mut results = Vec::new();
    for item in items {
        let post_count = count_posts_by_type(&state.db, &item.code).await;
        results.push(ArticleTypeWithCount { item, post_count });
    }
    Ok(Json(ApiResponse::new(results)))
}

/// GET /api/v1/admin/article-types — admin list (all, with post_count)
pub async fn list_admin_article_types(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ArticleTypeWithCount>>>, AppError> {
    let items = article_types::Entity::find()
        .order_by_asc(article_types::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;

    let mut results = Vec::new();
    for item in items {
        let post_count = count_posts_by_type(&state.db, &item.code).await;
        results.push(ArticleTypeWithCount { item, post_count });
    }
    Ok(Json(ApiResponse::new(results)))
}

/// POST /api/v1/admin/article-types — create
pub async fn create_article_type(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(body): Json<CreateArticleType>,
) -> Result<Json<ApiResponse<article_types::Model>>, AppError> {
    let now = crate::utils::now_local();
    let item = article_types::ActiveModel {
        code: Set(body.code),
        display_name: Set(body.display_name),
        color: Set(body.color.unwrap_or_else(|| "#6b7280".into())),
        sort_order: Set(body.sort_order.unwrap_or(99)),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let inserted = item
        .insert(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;
    Ok(Json(ApiResponse::new(inserted)))
}

/// PUT /api/v1/admin/article-types/:id
pub async fn update_article_type(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateArticleType>,
) -> Result<Json<ApiResponse<article_types::Model>>, AppError> {
    let item = article_types::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?
        .ok_or(AppError::NotFound("文章类型不存在".into()))?;

    let mut active: article_types::ActiveModel = item.into();
    if let Some(v) = body.display_name {
        active.display_name = Set(v);
    }
    if let Some(v) = body.color {
        active.color = Set(v);
    }
    if let Some(v) = body.sort_order {
        active.sort_order = Set(v);
    }
    if let Some(v) = body.is_active {
        active.is_active = Set(v);
    }
    active.updated_at = Set(crate::utils::now_local());

    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;
    Ok(Json(ApiResponse::new(updated)))
}

/// DELETE /api/v1/admin/article-types/:id — only when post_count == 0
pub async fn delete_article_type(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let item = article_types::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?
        .ok_or(AppError::NotFound("文章类型不存在".into()))?;

    let count = count_posts_by_type(&state.db, &item.code).await;
    if count > 0 {
        return Err(AppError::BadRequest(format!(
            "该类型下有 {} 篇文章，无法删除",
            count
        )));
    }

    item.delete(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;
    Ok(Json(ApiResponse::new("删除成功".to_string())))
}

/// POST /api/v1/admin/article-types/reorder
pub async fn reorder_article_types(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<ReorderRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    for (index, id) in req.ids.iter().enumerate() {
        let item = article_types::Entity::find_by_id(*id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::DbError(e))?
            .ok_or(AppError::NotFound("文章类型不存在".into()))?;
        let mut active: article_types::ActiveModel = item.into();
        active.sort_order = Set(index as i32);
        active
            .update(&state.db)
            .await
            .map_err(|e| AppError::DbError(e))?;
    }
    Ok(Json(ApiResponse::new(())))
}

// ─── Article Statuses ───

/// GET /api/v1/article-statuses — public list (active only, with post_count)
pub async fn list_article_statuses(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ArticleStatusWithCount>>>, AppError> {
    let items = article_statuses::Entity::find()
        .filter(article_statuses::Column::IsActive.eq(true))
        .order_by_asc(article_statuses::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;

    let mut results = Vec::new();
    for item in items {
        let post_count = count_posts_by_status(&state.db, &item.code).await;
        results.push(ArticleStatusWithCount { item, post_count });
    }
    Ok(Json(ApiResponse::new(results)))
}

/// GET /api/v1/admin/article-statuses — admin list (all, with post_count)
pub async fn list_admin_article_statuses(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ArticleStatusWithCount>>>, AppError> {
    let items = article_statuses::Entity::find()
        .order_by_asc(article_statuses::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;

    let mut results = Vec::new();
    for item in items {
        let post_count = count_posts_by_status(&state.db, &item.code).await;
        results.push(ArticleStatusWithCount { item, post_count });
    }
    Ok(Json(ApiResponse::new(results)))
}

/// POST /api/v1/admin/article-statuses — create
pub async fn create_article_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(body): Json<CreateArticleStatus>,
) -> Result<Json<ApiResponse<article_statuses::Model>>, AppError> {
    let now = crate::utils::now_local();
    let item = article_statuses::ActiveModel {
        code: Set(body.code),
        display_name: Set(body.display_name),
        color: Set(body.color.unwrap_or_else(|| "#6b7280".into())),
        sort_order: Set(body.sort_order.unwrap_or(99)),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let inserted = item
        .insert(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;
    Ok(Json(ApiResponse::new(inserted)))
}

/// PUT /api/v1/admin/article-statuses/:id
pub async fn update_article_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateArticleStatus>,
) -> Result<Json<ApiResponse<article_statuses::Model>>, AppError> {
    let item = article_statuses::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?
        .ok_or(AppError::NotFound("状态标签不存在".into()))?;

    let mut active: article_statuses::ActiveModel = item.into();
    if let Some(v) = body.display_name {
        active.display_name = Set(v);
    }
    if let Some(v) = body.color {
        active.color = Set(v);
    }
    if let Some(v) = body.sort_order {
        active.sort_order = Set(v);
    }
    if let Some(v) = body.is_active {
        active.is_active = Set(v);
    }
    active.updated_at = Set(crate::utils::now_local());

    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;
    Ok(Json(ApiResponse::new(updated)))
}

/// DELETE /api/v1/admin/article-statuses/:id — only when post_count == 0
pub async fn delete_article_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let item = article_statuses::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?
        .ok_or(AppError::NotFound("状态标签不存在".into()))?;

    let count = count_posts_by_status(&state.db, &item.code).await;
    if count > 0 {
        return Err(AppError::BadRequest(format!(
            "该状态标签下有 {} 篇文章，无法删除",
            count
        )));
    }

    item.delete(&state.db)
        .await
        .map_err(|e| AppError::DbError(e))?;
    Ok(Json(ApiResponse::new("删除成功".to_string())))
}

/// POST /api/v1/admin/article-statuses/reorder
pub async fn reorder_article_statuses(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<ReorderRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    for (index, id) in req.ids.iter().enumerate() {
        let item = article_statuses::Entity::find_by_id(*id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::DbError(e))?
            .ok_or(AppError::NotFound("状态标签不存在".into()))?;
        let mut active: article_statuses::ActiveModel = item.into();
        active.sort_order = Set(index as i32);
        active
            .update(&state.db)
            .await
            .map_err(|e| AppError::DbError(e))?;
    }
    Ok(Json(ApiResponse::new(())))
}
