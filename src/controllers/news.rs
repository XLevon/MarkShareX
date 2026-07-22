use crate::middleware::auth::{OptionalAuthUser, PrivilegedUser};
use crate::models::entity::news;
use crate::utils::{ApiResponse, AppError, AppState, Pagination};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// ── Response ──

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewsResponse {
    pub id: i32,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub content_html: String,
    pub status: String,
    pub topic_type: String,
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
            topic_type: m.topic_type,
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
    #[serde(default)]
    pub include_content: bool,
    pub status: Option<String>,
    pub topic_type: Option<String>,
    pub search: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct CreateNewsRequest {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub topic_type: String,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_status() -> String {
    "draft".to_string()
}

#[derive(Deserialize, IntoParams)]
pub struct UpdateNewsRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub topic_type: Option<String>,
    pub sort_order: Option<i32>,
}

fn parse_news_date(value: &str, field: &str) -> Result<chrono::NaiveDate, AppError> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest(format!("{field} 必须使用 YYYY-MM-DD 格式")))
}

fn start_of_news_date(value: &str, field: &str) -> Result<chrono::NaiveDateTime, AppError> {
    parse_news_date(value, field)?
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::BadRequest(format!("{field} 日期无效")))
}

fn start_of_day_after_news_date(
    value: &str,
    field: &str,
) -> Result<chrono::NaiveDateTime, AppError> {
    parse_news_date(value, field)?
        .succ_opt()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .ok_or_else(|| AppError::BadRequest(format!("{field} 日期超出范围")))
}

fn apply_news_filters(
    mut select: Select<news::Entity>,
    q: &NewsQuery,
) -> Result<Select<news::Entity>, AppError> {
    if let Some(status) = &q.status {
        if status != "all" {
            select = select.filter(news::Column::Status.eq(status));
        }
    }
    if let Some(topic_type) = &q.topic_type {
        if !topic_type.is_empty() {
            let types: Vec<&str> = topic_type.split(',').collect();
            select = select.filter(news::Column::TopicType.is_in(types));
        }
    }
    if let Some(date_from) = &q.date_from {
        if !date_from.is_empty() {
            select = select
                .filter(news::Column::CreatedAt.gte(start_of_news_date(date_from, "date_from")?));
        }
    }
    if let Some(date_to) = &q.date_to {
        if !date_to.is_empty() {
            select = select.filter(
                news::Column::CreatedAt.lt(start_of_day_after_news_date(date_to, "date_to")?),
            );
        }
    }
    if let Some(search) = &q.search {
        let term = search.trim();
        if !term.is_empty() {
            let like = format!("%{}%", term.replace('%', "\\%").replace('_', "\\_"));
            select = select.filter(news::Column::Title.like(&like));
        }
    }
    Ok(select)
}

// ── 内部函数 ──

async fn list_news_internal(
    state: &AppState,
    query: NewsQuery,
    default_status: Option<&str>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut select = news::Entity::find();
    if query.status.is_none() {
        if let Some(s) = default_status {
            select = select.filter(news::Column::Status.eq(s));
        }
    }
    select = apply_news_filters(select, &query)?;

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

    Ok(Json(ApiResponse {
        data: items,
        pagination: Some(Pagination {
            page,
            page_size,
            total,
            pages: (total as f64 / page_size as f64).ceil() as u64,
        }),
    }))
}

/// GET /api/v1/news — 公开列表（仅已发布）
#[utoipa::path(
    get,
    path = "/api/v1/news",
    params(NewsQuery),
    responses((status = 200, description = "成功", body = Vec<NewsResponse>)),
    tag = "News"
)]
pub async fn list_news(
    State(state): State<AppState>,
    Query(mut query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    query.status = Some("published".to_string());
    list_news_internal(&state, query, None).await
}

/// GET /api/v1/admin/news — 管理端列表（含草稿）
#[utoipa::path(
    get,
    path = "/api/v1/admin/news",
    params(NewsQuery),
    responses((status = 200, description = "成功", body = Vec<NewsResponse>)),
    tag = "News"
)]
pub async fn list_admin_news(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Query(query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    list_news_internal(&state, query, None).await
}

// ── 单条 ──

/// GET /api/v1/news/{id} — 公开详情（仅 published）
#[utoipa::path(
    get,
    path = "/api/v1/news/{id}",
    responses((status = 200, description = "成功", body = NewsResponse)),
    tag = "News"
)]
pub async fn get_news(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    _auth: OptionalAuthUser,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let item = news::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("资讯不存在".into()))?;

    if item.status != "published" {
        return Err(AppError::NotFound("资讯不存在".into()));
    }

    Ok(Json(ApiResponse {
        data: NewsResponse::from(item),
        pagination: None,
    }))
}

/// GET /api/v1/admin/news/{id} — 管理详情（包含草稿）
pub async fn get_admin_news(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    _auth: PrivilegedUser,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let item = news::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("资讯不存在".into()))?;

    Ok(Json(ApiResponse {
        data: NewsResponse::from(item),
        pagination: None,
    }))
}

// ── 题材列表（公开） ──

/// GET /api/v1/news/topic-types — 已发布资讯的所有题材（去重，支持日期+搜索筛选）
#[derive(Deserialize, IntoParams)]
pub struct TopicTypeQuery {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
}

pub async fn list_topic_types(
    State(state): State<AppState>,
    Query(params): Query<TopicTypeQuery>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let mut select = news::Entity::find()
        .select_only()
        .column(news::Column::TopicType)
        .distinct()
        .filter(news::Column::TopicType.ne(""))
        .filter(news::Column::Status.eq("published"));

    if let Some(ref date_from) = params.date_from {
        if !date_from.is_empty() {
            select = select
                .filter(news::Column::CreatedAt.gte(start_of_news_date(date_from, "date_from")?));
        }
    }
    if let Some(ref date_to) = params.date_to {
        if !date_to.is_empty() {
            select = select.filter(
                news::Column::CreatedAt.lt(start_of_day_after_news_date(date_to, "date_to")?),
            );
        }
    }
    if let Some(ref search) = params.search {
        let term = search.trim();
        if !term.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(news::Column::Title.contains(term))
                    .add(news::Column::Summary.contains(term)),
            );
        }
    }

    let rows = select
        .order_by_asc(news::Column::TopicType)
        .into_tuple::<String>()
        .all(&state.db)
        .await?;

    Ok(Json(ApiResponse {
        data: rows,
        pagination: None,
    }))
}

// ── Admin CRUD ──

/// POST /api/v1/admin/news — 创建咨询
#[utoipa::path(
    post,
    path = "/api/v1/admin/news",
    responses((status = 200, description = "成功", body = NewsResponse)),
    tag = "News"
)]
pub async fn create_news(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Json(req): Json<CreateNewsRequest>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let now = crate::utils::now_local();
    let status = if req.status.is_empty() {
        "draft".to_string()
    } else {
        req.status.clone()
    };
    let published_at = if status == "published" {
        Some(now)
    } else {
        None
    };

    let model = news::ActiveModel {
        title: Set(req.title),
        summary: Set(req.summary),
        content: Set(req.content),
        content_html: Set(String::new()),
        status: Set(status),
        topic_type: Set(req.topic_type),
        sort_order: Set(req.sort_order),
        published_at: Set(published_at),
        user_id: Set(Some(_auth.0.user_id)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse {
        data: NewsResponse::from(inserted),
        pagination: None,
    }))
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
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateNewsRequest>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let item = news::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("资讯不存在".into()))?;

    let now = crate::utils::now_local();
    let mut model: news::ActiveModel = item.into();

    if let Some(title) = req.title {
        model.title = Set(title);
    }
    if let Some(summary) = req.summary {
        model.summary = Set(summary);
    }
    if let Some(content) = req.content {
        model.content = Set(content);
    }
    if let Some(topic_type) = req.topic_type {
        model.topic_type = Set(topic_type);
    }
    if let Some(sort_order) = req.sort_order {
        model.sort_order = Set(sort_order);
    }
    model.updated_at = Set(now);

    if let Some(status) = req.status {
        if status == "published" && model.status.as_ref() != "published" {
            model.status = Set("published".to_string());
            model.published_at = Set(Some(now));
        } else if status != "published" {
            model.status = Set(status);
        }
    }

    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse {
        data: NewsResponse::from(updated),
        pagination: None,
    }))
}

/// POST /api/v1/admin/news/batch-delete — 批量删除咨询
#[derive(Deserialize, ToSchema)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i32>,
}

pub async fn batch_delete_news(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<ApiResponse<i32>>, AppError> {
    let count = news::Entity::delete_many()
        .filter(news::Column::Id.is_in(req.ids))
        .exec(&state.db)
        .await?
        .rows_affected as i32;
    Ok(Json(ApiResponse {
        data: count,
        pagination: None,
    }))
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
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    news::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse {
        data: (),
        pagination: None,
    }))
}
