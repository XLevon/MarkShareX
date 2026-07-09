use axum::{
    extract::{State, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use sea_orm::*;
use crate::utils::{AppState, AppError, ApiResponse, Pagination};
use crate::middleware::auth::AuthUser;
use crate::models::entity::news;

// ── Response ──

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewsResponse {
    pub id: i32,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub content_html: String,
    pub status: String,
    pub sort_order: i32,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub user_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<news::Model> for NewsResponse {
    fn from(m: news::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            summary: m.summary,
            content: m.content,
            content_html: m.content_html,
            status: m.status,
            sort_order: m.sort_order,
            published_at: m.published_at,
            user_id: m.user_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// ── Request ──

#[derive(Deserialize, IntoParams)]
pub struct NewsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    /// 是否包含正文内容（默认 false，列表不返回正文以提升加载效率）
    #[serde(default)]
    pub include_content: bool,
    /// 状态过滤：published（默认）、draft、或 all（不过滤）
    #[serde(default)]
    pub status: Option<String>,
}

fn default_status() -> String { "draft".to_string() }

#[derive(Deserialize, ToSchema)]
pub struct CreateNewsRequest {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub published_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateNewsRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub content_html: Option<String>,
    pub status: Option<String>,
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub published_at: Option<Option<chrono::NaiveDateTime>>,
}

// ── Public ──

/// GET /api/v1/news — 公开咨询列表（仅已发布）
#[utoipa::path(
    get,
    path = "/api/v1/news",
    params(NewsQuery),
    responses((status = 200, description = "成功", body = Vec<NewsResponse>)),
    tag = "News"
)]
pub async fn list_news(
    State(state): State<AppState>,
    Query(query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut select = news::Entity::find();
    match query.status.as_deref() {
        Some("all") => {}, // 不过滤状态
        Some(s) => { select = select.filter(news::Column::Status.eq(s)); }
        None => { select = select.filter(news::Column::Status.eq("published")); }
    }

    let paginator = select
        .order_by_desc(news::Column::SortOrder)
        .order_by_desc(news::Column::CreatedAt)
        .paginate(&state.db, page_size);

    let total = paginator.num_items().await?;
    let items: Vec<NewsResponse> = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(|m| {
            let mut resp = NewsResponse::from(m);
            if !query.include_content {
                resp.content = String::new();
                resp.content_html = String::new();
            }
            resp
        })
        .collect();

    let pages = if page_size > 0 { (total + page_size - 1) / page_size } else { 0 };

    Ok(Json(ApiResponse {
        data: items,
        pagination: Some(Pagination { page, page_size, total, pages }),
    }))
}

/// GET /api/v1/news/{id} — 获取单条咨询详情（含正文）
#[utoipa::path(
    get,
    path = "/api/v1/news/{id}",
    responses((status = 200, description = "成功", body = NewsResponse)),
    tag = "News"
)]
pub async fn get_news(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let item = news::Entity::find_by_id(id)
        .filter(news::Column::Status.eq("published"))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("咨询不存在".into()))?;
    Ok(Json(ApiResponse { data: NewsResponse::from(item), pagination: None }))
}

// ── Admin ──

/// GET /api/v1/admin/news — 管理端咨询列表（含草稿）
#[utoipa::path(
    get,
    path = "/api/v1/admin/news",
    params(NewsQuery),
    responses((status = 200, description = "成功", body = Vec<NewsResponse>)),
    tag = "News"
)]
pub async fn list_admin_news(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let paginator = news::Entity::find()
        .order_by_desc(news::Column::SortOrder)
        .order_by_desc(news::Column::CreatedAt)
        .paginate(&state.db, page_size);

    let total = paginator.num_items().await?;
    let items: Vec<NewsResponse> = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(NewsResponse::from)
        .collect();

    let pages = if page_size > 0 { (total + page_size - 1) / page_size } else { 0 };

    Ok(Json(ApiResponse {
        data: items,
        pagination: Some(Pagination { page, page_size, total, pages }),
    }))
}

/// POST /api/v1/admin/news — 创建咨询
#[utoipa::path(
    post,
    path = "/api/v1/admin/news",
    responses((status = 200, description = "成功", body = NewsResponse)),
    tag = "News"
)]
pub async fn create_news(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateNewsRequest>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let now = crate::utils::now_local();
    let content_html = crate::services::posts::render_markdown(&state.db, &req.content).await;

    let model = news::ActiveModel {
        title: Set(req.title),
        summary: Set(req.summary),
        content: Set(req.content),
        content_html: Set(content_html),
        status: Set(req.status),
        sort_order: Set(req.sort_order),
        published_at: Set(req.published_at),
        user_id: Set(Some(auth.user_id)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: NewsResponse::from(inserted), pagination: None }))
}

/// PUT /api/v1/admin/news/{id} — 更新咨询
#[utoipa::path(
    put,
    path = "/api/v1/admin/news/{id}",
    responses((status = 200, description = "成功", body = NewsResponse)),
    tag = "News"
)]
pub async fn update_news(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateNewsRequest>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let entry = news::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("咨询不存在".into()))?;

    let mut model: news::ActiveModel = entry.into();

    if let Some(v) = req.title { model.title = Set(v); }
    if let Some(v) = req.summary { model.summary = Set(v); }
    if let Some(v) = req.content { 
        model.content = Set(v.clone());
        let html = crate::services::posts::render_markdown(&state.db, &v).await;
        model.content_html = Set(html);
    }
    if let Some(v) = req.content_html { model.content_html = Set(v); }
    if let Some(v) = req.status { model.status = Set(v); }
    if let Some(v) = req.sort_order { model.sort_order = Set(v); }
    if let Some(v) = req.published_at { model.published_at = Set(v); }
    model.updated_at = Set(crate::utils::now_local());

    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: NewsResponse::from(updated), pagination: None }))
}

/// DELETE /api/v1/admin/news/{id} — 删除咨询
#[utoipa::path(
    delete,
    path = "/api/v1/admin/news/{id}",
    responses((status = 200, description = "成功")),
    tag = "News"
)]
pub async fn delete_news(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    news::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}
