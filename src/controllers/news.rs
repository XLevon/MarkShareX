
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

fn default_status() -> String { "draft".to_string() }

fn apply_news_filters(mut select: Select<news::Entity>, q: &NewsQuery) -> Select<news::Entity> {
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
            select = select.filter(news::Column::CreatedAt.gte(format!("{}T00:00:00", date_from)));
        }
    }
    if let Some(date_to) = &q.date_to {
        if !date_to.is_empty() {
            select = select.filter(news::Column::CreatedAt.lt(format!("{}T00:00:00", date_to)));
        }
    }
    if let Some(search) = &q.search {
        let term = search.trim();
        if !term.is_empty() {
            let like = format!("%{}%", term.replace('%', "\\%").replace('_', "\\_"));
            select = select.filter(news::Column::Title.like(&like));
        }
    }
    select
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
    select = apply_news_filters(select, &query);

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
    Query(query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    list_news_internal(&state, query, Some("published")).await
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
    _auth: AuthUser,
    Query(query): Query<NewsQuery>,
) -> Result<Json<ApiResponse<Vec<NewsResponse>>>, AppError> {
    list_news_internal(&state, query, None).await
}

// ── 单条（公开 + 管理共用） ──

/// GET /api/v1/news/{id} 或 /api/v1/admin/news/{id}
/// 有 AuthUser → 管理端（不过滤状态），无 AuthUser → 公开（仅 published）
#[utoipa::path(
    get,
    path = "/api/v1/news/{id}",
    responses((status = 200, description = "成功", body = NewsResponse)),
    tag = "News"
)]
pub async fn get_news(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: Option<AuthUser>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let item = news::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("资讯不存在".into()))?;

    // Admin → 不过滤；public → 仅 published
    if auth.is_none() && item.status != "published" {
        return Err(AppError::NotFound("资讯不存在".into()));
    }

    Ok(Json(ApiResponse { data: NewsResponse::from(item), pagination: None }))
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
    use sea_orm::{ConnectionTrait, Statement};

    // Build dynamic SQL
    let mut sql = String::from("SELECT DISTINCT topic_type FROM news WHERE topic_type != '' AND status = 'published'");

    if let Some(ref date_from) = params.date_from {
        if !date_from.is_empty() {
            sql.push_str(&format!(" AND created_at >= '{}T00:00:00'", date_from));
        }
    }
    if let Some(ref date_to) = params.date_to {
        if !date_to.is_empty() {
            sql.push_str(&format!(" AND created_at < '{}T00:00:00'", date_to));
        }
    }
    if let Some(ref search) = params.search {
        let term = search.trim();
        if !term.is_empty() {
            // Escape single quotes in search term
            let escaped = term.replace('\'', "''");
            sql.push_str(&format!(" AND (title LIKE '%{}%' OR summary LIKE '%{}%')", escaped, escaped));
        }
    }
    sql.push_str(" ORDER BY topic_type");

    let rows = state.db
        .query_all(Statement::from_string(state.db.get_database_backend(), sql))
        .await?
        .into_iter()
        .filter_map(|row| row.try_get::<String>("", "topic_type").ok())
        .collect::<Vec<String>>();

    Ok(Json(ApiResponse { data: rows, pagination: None }))
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
    _auth: AuthUser,
    Json(req): Json<CreateNewsRequest>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let now = crate::utils::now_local();
    let status = if req.status.is_empty() { "draft".to_string() } else { req.status.clone() };
    let published_at = if status == "published" { Some(now) } else { None };

    let model = news::ActiveModel {
        title: Set(req.title),
        summary: Set(req.summary),
        content: Set(req.content),
        content_html: Set(String::new()),
        status: Set(status),
        topic_type: Set(req.topic_type),
        sort_order: Set(req.sort_order),
        published_at: Set(published_at),
        user_id: Set(Some(_auth.user_id)),
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
    Json(req): Json<CreateNewsRequest>,
) -> Result<Json<ApiResponse<NewsResponse>>, AppError> {
    let item = news::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("资讯不存在".into()))?;

    let now = crate::utils::now_local();
    let mut model: news::ActiveModel = item.into();

    model.title = Set(req.title);
    model.summary = Set(req.summary);
    model.content = Set(req.content);
    model.topic_type = Set(req.topic_type);
    model.sort_order = Set(req.sort_order);
    model.updated_at = Set(now);

    if req.status == "published" && model.status.as_ref() != "published" {
        model.status = Set("published".to_string());
        model.published_at = Set(Some(now));
    } else if req.status != "published" {
        model.status = Set(req.status);
    }

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
