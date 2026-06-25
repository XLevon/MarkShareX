use axum::{extract::{State, Query}, Json};
use serde::{Deserialize, Serialize};
use crate::utils::{AppState, AppError, ApiResponse, Pagination};
use crate::middleware::auth::AuthUser;
use sea_orm::*;

#[derive(Serialize)]
pub struct TrendPoint {
    pub date: String,
    pub views: i64,
}

#[derive(Deserialize)]
pub struct TrendQuery {
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 { 7 }
/// GET /api/v1/analytics/trend — Page view trend data

#[utoipa::path(
    get,
    path = "/api/v1/analytics/trend",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_trend(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<TrendQuery>,
) -> Result<Json<ApiResponse<Vec<TrendPoint>>>, AppError> {
    let days = query.days.min(365);

    let raw_sql = if auth.is_privileged() {
        format!(
            "SELECT DATE(rl.created_at) as date, COUNT(*) as views FROM read_logs rl WHERE rl.created_at >= DATE('now', '-{} days') GROUP BY DATE(rl.created_at) ORDER BY date",
            days
        )
    } else {
        format!(
            "SELECT DATE(rl.created_at) as date, COUNT(*) as views FROM read_logs rl JOIN posts p ON rl.post_id = p.id WHERE p.user_id = {} AND rl.created_at >= DATE('now', '-{} days') GROUP BY DATE(rl.created_at) ORDER BY date",
            auth.user_id, days
        )
    };

    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;

    let mut data_map = std::collections::HashMap::new();
    for row in &results {
        let date: String = row.try_get_by_index::<String>(0)?;
        let views: i64 = row.try_get_by_index::<i64>(1)?;
        data_map.insert(date, views);
    }

    let mut data = Vec::new();
    let now = chrono::Local::now().date_naive();
    for i in (0..days as i64).rev() {
        let date = now - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let views = data_map.get(&date_str).copied().unwrap_or(0);
        data.push(TrendPoint { date: date_str, views });
    }

    Ok(Json(ApiResponse::new(data)))
}
/// GET /api/v1/analytics/total-views — Total page views

#[utoipa::path(
    get,
    path = "/api/v1/analytics/total-views",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_total_views(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<i64>>, AppError> {
    let raw_sql = if auth.is_privileged() {
        "SELECT COUNT(*) FROM read_logs".to_string()
    } else {
        format!(
            "SELECT COUNT(*) FROM read_logs rl JOIN posts p ON rl.post_id = p.id WHERE p.user_id = {}",
            auth.user_id
        )
    };
    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;
    let total: i64 = results
        .first()
        .and_then(|r| r.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);

    Ok(Json(ApiResponse::new(total)))
}
/// GET /api/v1/analytics/total-likes — Total likes

#[utoipa::path(
    get,
    path = "/api/v1/analytics/total-likes",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_total_likes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<i64>>, AppError> {
    let raw_sql = if auth.is_privileged() {
        "SELECT COALESCE(SUM(like_count), 0) FROM posts WHERE deleted_at IS NULL".to_string()
    } else {
        format!(
            "SELECT COALESCE(SUM(like_count), 0) FROM posts WHERE deleted_at IS NULL AND user_id = {}",
            auth.user_id
        )
    };
    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;
    let total: i64 = results
        .first()
        .and_then(|r| r.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);

    Ok(Json(ApiResponse::new(total)))
}
/// GET /api/v1/analytics/total-comments — Total comments

#[utoipa::path(
    get,
    path = "/api/v1/analytics/total-comments",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_total_comments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<i64>>, AppError> {
    let raw_sql = if auth.is_privileged() {
        "SELECT COUNT(*) FROM comments".to_string()
    } else {
        format!(
            "SELECT COUNT(*) FROM comments c JOIN posts p ON c.post_id = p.id WHERE p.user_id = {}",
            auth.user_id
        )
    };
    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;
    let total: i64 = results
        .first()
        .and_then(|r| r.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);

    Ok(Json(ApiResponse::new(total)))
}

/// GET /api/v1/analytics/today-likes — Today's new likes
#[utoipa::path(
    get,
    path = "/api/v1/analytics/today-likes",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_today_likes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<i64>>, AppError> {
    let raw_sql = if auth.is_privileged() {
        "SELECT COUNT(*) FROM likes WHERE DATE(created_at) = DATE('now')".to_string()
    } else {
        format!(
            "SELECT COUNT(*) FROM likes l JOIN posts p ON l.post_id = p.id WHERE p.user_id = {} AND DATE(l.created_at) = DATE('now')",
            auth.user_id
        )
    };
    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;
    let total: i64 = results
        .first()
        .and_then(|r| r.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);

    Ok(Json(ApiResponse::new(total)))
}

// ─── Today's new posts ──────────────────────────────────────
#[derive(Serialize)]
pub struct TodayPosts {
    pub published: i64,
    pub drafts: i64,
}

/// GET /api/v1/analytics/today-posts — Today's new published & draft posts
#[utoipa::path(
    get,
    path = "/api/v1/analytics/today-posts",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_today_posts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<TodayPosts>>, AppError> {
    let user_filter = if auth.is_privileged() {
        String::new()
    } else {
        format!("AND p.user_id = {}", auth.user_id)
    };

    let raw_sql = format!(
        "SELECT 'published' AS status, COUNT(*) AS cnt \
         FROM posts p \
         WHERE p.status = 'published' AND DATE(p.published_at) = DATE('now') AND p.deleted_at IS NULL {} \
         UNION ALL \
         SELECT 'draft', COUNT(*) \
         FROM posts p \
         WHERE p.status = 'draft' AND DATE(p.created_at) = DATE('now') AND p.deleted_at IS NULL {}",
        user_filter, user_filter
    );
    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;

    let mut published: i64 = 0;
    let mut drafts: i64 = 0;
    for row in &results {
        let status: String = row.try_get_by_index::<String>(0)?;
        let cnt: i64 = row.try_get_by_index::<i64>(1)?;
        match status.as_str() {
            "published" => published = cnt,
            "draft" => drafts = cnt,
            _ => {}
        }
    }

    Ok(Json(ApiResponse::new(TodayPosts { published, drafts })))
}

// ─── Per-post views ─────────────────────────────────────────
#[derive(Serialize)]
pub struct PostViews {
    pub post_id: i64,
    pub title: String,
    pub slug: String,
    pub author_name: String,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub published_at: Option<String>,
}
/// GET /api/v1/analytics/post-views — Post view ranking

#[utoipa::path(
    get,
    path = "/api/v1/analytics/post-views",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_post_views(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<LikeQuery>,
) -> Result<Json<ApiResponse<Vec<PostViews>>>, AppError> {
    let page = query.page.max(1);
    let page_size = query.page_size.min(100).max(1);
    let offset = (page - 1) * page_size;

    let db = &state.db;
    let backend = db.get_database_backend();

    let user_filter = if auth.is_privileged() {
        String::new()
    } else {
        format!("AND p.user_id = {}", auth.user_id)
    };

    // Total count
    let total_sql = format!(
        "SELECT COUNT(*) FROM posts p WHERE p.status = 'published' AND p.deleted_at IS NULL {}",
        user_filter
    );
    let total: i64 = db.query_one(
        sea_orm::Statement::from_string(backend, total_sql)
    ).await?
    .and_then(|r| r.try_get_by_index::<i64>(0).ok())
    .unwrap_or(0);

    let raw_sql = format!(
        "SELECT p.id, p.title, p.slug, COALESCE(u.display_name, u.username) AS author_name, \
                p.published_at, \
                COALESCE(pv.cnt, 0) AS view_count, \
                COALESCE(lk.cnt, 0) AS like_count, \
                COALESCE(cm.cnt, 0) AS comment_count \
         FROM posts p \
         LEFT JOIN users u ON p.user_id = u.id \
         LEFT JOIN (SELECT post_id, COUNT(*) AS cnt FROM read_logs GROUP BY post_id) pv ON p.id = pv.post_id \
         LEFT JOIN (SELECT post_id, COUNT(*) AS cnt FROM likes GROUP BY post_id) lk ON p.id = lk.post_id \
         LEFT JOIN (SELECT post_id, COUNT(*) AS cnt FROM comments WHERE status = 'approved' GROUP BY post_id) cm ON p.id = cm.post_id \
         WHERE p.status = 'published' AND p.deleted_at IS NULL {} \
         ORDER BY view_count DESC \
         LIMIT {} OFFSET {}",
        user_filter, page_size, offset
    );

    let stmt = sea_orm::Statement::from_string(state.db.get_database_backend(), raw_sql);
    let results = state.db.query_all(stmt).await?;

    let mut posts = Vec::new();
    for row in &results {
        let post_id: i64 = row.try_get_by_index::<i64>(0)?;
        let title: String = row.try_get_by_index::<String>(1)?;
        let slug: String = row.try_get_by_index::<String>(2)?;
        let author_name: String = row.try_get_by_index::<String>(3)?;
        let published_at: Option<String> = row.try_get_by_index::<Option<String>>(4)?;
        let view_count: i64 = row.try_get_by_index::<i64>(5)?;
        let like_count: i64 = row.try_get_by_index::<i64>(6)?;
        let comment_count: i64 = row.try_get_by_index::<i64>(7)?;

        posts.push(PostViews {
            post_id,
            title,
            slug,
            author_name,
            view_count,
            like_count,
            comment_count,
            published_at,
        });
    }

    Ok(Json(ApiResponse::with_pagination(posts, Pagination::new(total as u64, page, page_size))))
}

// ─── Like records ───────────────────────────────────────────
#[derive(Deserialize)]
pub struct LikeQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 { 1 }
fn default_page_size() -> u64 { 20 }

#[derive(Serialize)]
pub struct LikeRecord {
    pub id: i64,
    pub post_id: i64,
    pub post_title: String,
    pub post_slug: String,
    pub author_name: String,
    pub published_at: Option<String>,
    pub user_name: String,
    pub created_at: String,
}
/// GET /api/v1/admin/likes — List all likes (admin)

#[utoipa::path(
    get,
    path = "/api/v1/admin/likes",
    responses((status = 200, description = "成功")),
    tag = "Analytics"
)]
pub async fn get_like_records(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<LikeQuery>,
) -> Result<Json<ApiResponse<Vec<LikeRecord>>>, AppError> {
    let page = query.page.max(1);
    let page_size = query.page_size.min(100).max(1);
    let offset = (page - 1) * page_size;

    let db = &state.db;
    let backend = db.get_database_backend();

    let user_filter = if auth.is_privileged() {
        String::new()
    } else {
        format!("AND p.user_id = {}", auth.user_id)
    };

    // Total count
    let total_sql = format!(
        "SELECT COUNT(*) FROM likes l JOIN posts p ON l.post_id = p.id WHERE p.deleted_at IS NULL {}",
        user_filter
    );
    let total: i64 = db.query_one(
        sea_orm::Statement::from_string(backend, total_sql)
    ).await?
    .and_then(|r| r.try_get_by_index::<i64>(0).ok())
    .unwrap_or(0);

    // Like records with JOINs
    let raw_sql = format!(
        "SELECT l.id, l.post_id, p.title, p.slug, SUBSTR(p.published_at, 1, 19) AS published_at, \
                COALESCE(u.display_name, u.username) AS author_name, \
                COALESCE(liker.display_name, liker.username) AS user_name, \
                l.created_at \
         FROM likes l \
         JOIN posts p ON l.post_id = p.id \
         LEFT JOIN users u ON p.user_id = u.id \
         LEFT JOIN users liker ON l.user_id = liker.id \
         WHERE p.deleted_at IS NULL {} \
         ORDER BY l.created_at DESC \
         LIMIT {} OFFSET {}",
        user_filter, page_size, offset
    );

    let stmt = sea_orm::Statement::from_string(backend, raw_sql);
    let results = db.query_all(stmt).await?;

    let mut records = Vec::new();
    for row in &results {
        records.push(LikeRecord {
            id: row.try_get_by_index::<i64>(0)?,
            post_id: row.try_get_by_index::<i64>(1)?,
            post_title: row.try_get_by_index::<String>(2)?,
            post_slug: row.try_get_by_index::<String>(3)?,
            published_at: row.try_get_by_index::<Option<String>>(4)?,
            author_name: row.try_get_by_index::<String>(5)?,
            user_name: row.try_get_by_index::<String>(6)?,
            created_at: row.try_get_by_index::<String>(7)?,
        });
    }

    Ok(Json(ApiResponse::with_pagination(records, Pagination::new(total as u64, page, page_size))))
}
