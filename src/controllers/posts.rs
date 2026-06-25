use axum::{extract::{State, Path, Query, ConnectInfo}, Json};
use std::net::SocketAddr;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::utils::{AppState, AppError, ApiResponse};
use crate::middleware::auth::AuthUser;
use crate::models::entity::{posts, users, article_types, article_statuses};
use sea_orm::*;

/// Strip any http(s)://host/uploads/ prefix to ./uploads/ for storage normalization.
/// Example: "https://www.xlevon.cn/uploads/a.png" → "./uploads/a.png"
fn normalize_uploads_url(s: &str) -> String {
    let re = regex::Regex::new(r"https?://[^/]+/uploads/").unwrap();
    re.replace(s, "./uploads/").to_string()
}

#[derive(Serialize, ToSchema)]
pub struct AdjacentPost {
    pub id: i32,
    pub title: String,
    pub slug: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdjacentPostsResponse {
    pub prev: Option<AdjacentPost>,
    pub next: Option<AdjacentPost>,
}

#[derive(Deserialize)]
pub struct ListPostsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<String>,
    pub category_id: Option<i32>,
    /// Comma-separated category IDs for multi-select: "1,2,3"
    pub category_ids: Option<String>,
    pub tag_id: Option<i32>,
    /// Fuzzy search posts by tag name (matches posts that have a tag whose name contains this)
    pub tag_search: Option<String>,
    pub author_id: Option<i32>,
    pub search: Option<String>,
    pub article_type: Option<String>,
    /// Comma-separated article types for multi-select: "tutorial,news"
    pub article_types: Option<String>,
    pub article_status: Option<String>,
    /// Comma-separated article statuses for multi-select: "draft,reviewed"
    pub article_statuses: Option<String>,
    pub is_pinned: Option<bool>,
    /// When set to true, category_id queries include direct children (sub-categories).
    pub include_subcategories: Option<bool>,
}

/// Parse comma-separated string into Vec<i32> (e.g. "1,2,3")
fn parse_comma_i32(s: Option<&str>) -> Option<Vec<i32>> {
    s.filter(|v| !v.is_empty())
        .map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect())
}

/// Parse comma-separated string into Vec<String> (e.g. "tutorial,news")
fn parse_comma_string(s: Option<&str>) -> Option<Vec<String>> {
    s.filter(|v| !v.is_empty())
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

#[derive(Serialize, ToSchema)]
pub struct AuthorInfo {
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub content_html: Option<String>,
    pub cover_image: Option<String>,
    pub status: String,
    pub post_type: String,
    pub is_pinned: bool,
    pub allow_comment: bool,
    pub view_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub article_type: Option<String>,
    pub article_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_status_name: Option<String>,
    pub category_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_cover_image: Option<String>,
    pub user_id: i32,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<AuthorInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    pub pin_order: i32,
}

impl From<posts::Model> for PostResponse {
    fn from(p: posts::Model) -> Self {
        Self {
            id: p.id,
            title: p.title,
            slug: p.slug,
            summary: p.summary,
            content: p.content,
            content_html: p.content_html,
            cover_image: p.cover_image,
            status: p.status,
            post_type: p.post_type,
            is_pinned: p.is_pinned,
            allow_comment: p.allow_comment,
            view_count: p.view_count,
            like_count: p.like_count,
            comment_count: p.comment_count,
            article_type: Some(p.article_type),
            article_status: Some(p.article_status),
            category_id: p.category_id,
            category_cover_image: None,
            user_id: p.user_id,
            published_at: p.published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
            category_name: None,
            tags: None,
            author: None,
            author_name: None,
            pin_order: p.sort_order,
            article_type_name: None,
            article_status_name: None,
        }
    }
}

/// Batch-resolve article_type and article_status display names from knowledge base
async fn fill_kb_names(db: &DatabaseConnection, posts: &mut [PostResponse]) {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    
    // Collect unique codes
    let mut type_codes: Vec<String> = Vec::new();
    let mut status_codes: Vec<String> = Vec::new();
    for p in posts.iter() {
        if let Some(ref code) = p.article_type {
            if !type_codes.contains(code) { type_codes.push(code.clone()); }
        }
        if let Some(ref code) = p.article_status {
            if !status_codes.contains(code) { status_codes.push(code.clone()); }
        }
    }
    
    if type_codes.is_empty() && status_codes.is_empty() { return; }
    
    // Lookup type display_names
    let type_names: std::collections::HashMap<String, String> = if !type_codes.is_empty() {
        article_types::Entity::find()
            .filter(article_types::Column::Code.is_in(type_codes.clone()))
            .all(db).await
            .unwrap_or_default()
            .into_iter()
            .map(|t| (t.code, t.display_name))
            .collect()
    } else { std::collections::HashMap::new() };
    
    // Lookup status display_names
    let status_names: std::collections::HashMap<String, String> = if !status_codes.is_empty() {
        article_statuses::Entity::find()
            .filter(article_statuses::Column::Code.is_in(status_codes.clone()))
            .all(db).await
            .unwrap_or_default()
            .into_iter()
            .map(|s| (s.code, s.display_name))
            .collect()
    } else { std::collections::HashMap::new() };
    
    // Fill into responses
    for p in posts.iter_mut() {
        if let Some(ref code) = p.article_type {
            p.article_type_name = type_names.get(code).cloned();
        }
        if let Some(ref code) = p.article_status {
            p.article_status_name = status_names.get(code).cloned();
        }
    }
}

#[derive(Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub slug: Option<String>,
    pub category_id: Option<i32>,
    pub status: Option<String>,
    pub post_type: Option<String>,
    pub cover_image: Option<String>,
    pub is_pinned: Option<bool>,
    pub allow_comment: Option<bool>,
    pub article_type: Option<String>,
    pub article_status: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_helpers::double_option::deserialize")]
    pub category_id: Option<Option<i32>>,
    pub status: Option<String>,
    pub post_type: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_helpers::double_option::deserialize")]
    pub cover_image: Option<Option<String>>,
    pub is_pinned: Option<bool>,
    pub allow_comment: Option<bool>,
    pub article_type: Option<String>,
    pub article_status: Option<String>,
    pub author_id: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
}
/// GET /api/v1/posts — List published posts (public)

#[utoipa::path(
    get,
    path = "/api/v1/posts",
    responses(
        (status = 200, description = "文章列表", body = Vec<PostResponse>)
    ),
    params(
        ("page" = Option<u64>, Query, description = "页码（默认 1）"),
        ("page_size" = Option<u64>, Query, description = "每页数量（默认 20）"),
        ("status" = Option<String>, Query, description = "筛选状态"),
        ("category_id" = Option<i32>, Query, description = "分类 ID"),
        ("tag_id" = Option<i32>, Query, description = "标签 ID"),
        ("author_id" = Option<i32>, Query, description = "作者 ID"),
    ),
    tag = "Posts"
)]
pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<ApiResponse<Vec<PostResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let effective_status = Some(query.status.as_deref().unwrap_or("published"));
    // 排除隐藏分类下的文章
    let hidden_ids = super::categories::get_hidden_category_ids(&state.db).await?;
    let exclude_ids = if hidden_ids.is_empty() { None } else { Some(hidden_ids.as_slice()) };
    let (posts_list, pagination) = crate::services::posts::list_posts(
        &state.db,
        page,
        page_size,
        effective_status,
        query.category_id,
        parse_comma_i32(query.category_ids.as_deref()).as_deref(),
        query.tag_id,
        query.tag_search.as_deref(),
        query.author_id,
        exclude_ids,
        query.article_type.as_deref(),
        parse_comma_string(query.article_types.as_deref()).as_deref(),
        query.article_status.as_deref(),
        parse_comma_string(query.article_statuses.as_deref()).as_deref(),
        query.is_pinned,
        query.include_subcategories,
        query.search.as_deref(),
    )
    .await?;

    // Batch-fetch categories to avoid N+1 queries
    let category_ids: Vec<i32> = posts_list.iter().filter_map(|p| p.category_id).collect();
    let mut category_cover_map: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    let categories = if !category_ids.is_empty() {
        let cats = crate::models::entity::categories::Entity::find()
            .filter(crate::models::entity::categories::Column::Id.is_in(category_ids))
            .all(&state.db)
            .await?;
        for c in &cats {
            if let Some(cover) = super::network_resources::resolve_cover_url(
                &state.db,
                c.network_resource_id,
                c.image_url.as_deref(),
                c.image_filename.as_deref(),
            ).await {
                category_cover_map.insert(c.id, cover);
            }
        }
        cats.into_iter().map(|c| (c.id, c.name)).collect::<std::collections::HashMap<i32, String>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch authors to get display_name
    let user_ids: Vec<i32> = posts_list.iter().map(|p| p.user_id).collect();
    let users = if !user_ids.is_empty() {
        let us = users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .all(&state.db)
            .await?;
        us.into_iter().map(|u| (u.id, u.display_name.unwrap_or(u.username))).collect::<std::collections::HashMap<i32, String>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch real view counts from read_logs table
    let post_ids_str = posts_list.iter().map(|p| p.id.to_string()).collect::<Vec<_>>().join(",");
    let view_counts = if !post_ids_str.is_empty() {
        let sql = format!(
            "SELECT post_id, COUNT(*) as cnt FROM read_logs WHERE post_id IN ({}) GROUP BY post_id",
            post_ids_str
        );
        let rows = state.db.query_all(sea_orm::Statement::from_string(
            state.db.get_database_backend(), sql,
        )).await.unwrap_or_default();
        rows.into_iter().filter_map(|r| {
            let id = r.try_get_by_index::<i32>(0).ok()?;
            let cnt = r.try_get_by_index::<i64>(1).ok()?;
            Some((id, cnt as i32))
        }).collect::<std::collections::HashMap<i32, i32>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch real like counts from likes table
    let like_counts = if !post_ids_str.is_empty() {
        let sql = format!(
            "SELECT post_id, COUNT(*) as cnt FROM likes WHERE post_id IN ({}) GROUP BY post_id",
            post_ids_str
        );
        let rows = state.db.query_all(sea_orm::Statement::from_string(
            state.db.get_database_backend(), sql,
        )).await.unwrap_or_default();
        rows.into_iter().filter_map(|r| {
            let id = r.try_get_by_index::<i32>(0).ok()?;
            let cnt = r.try_get_by_index::<i64>(1).ok()?;
            Some((id, cnt as i32))
        }).collect::<std::collections::HashMap<i32, i32>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch tags for all posts
    let tags_map: std::collections::HashMap<i32, Vec<String>> = if !post_ids_str.is_empty() {
        let sql = format!(
            "SELECT pt.post_id, t.name FROM post_tags pt JOIN tags t ON pt.tag_id = t.id WHERE pt.post_id IN ({}) ORDER BY t.name",
            post_ids_str
        );
        if let Ok(rows) = state.db.query_all(sea_orm::Statement::from_string(
            state.db.get_database_backend(), sql,
        )).await {
            let mut map: std::collections::HashMap<i32, Vec<String>> = std::collections::HashMap::new();
            for row in rows {
                if let (Ok(pid), Ok(name)) = (row.try_get_by_index::<i32>(0), row.try_get_by_index::<String>(1)) {
                    map.entry(pid).or_default().push(name);
                }
            }
            map
        } else {
            std::collections::HashMap::new()
        }
    } else {
        std::collections::HashMap::new()
    };

    let mut data = Vec::new();
    for post in posts_list {
        let cover_network_id = post.cover_network_id;
        let cover_image_url = post.cover_image_url.clone();
        let cover_image_filename = post.cover_image_filename.clone();
        let category_name = post.category_id.and_then(|cid| categories.get(&cid).cloned());
        let category_cover = post.category_id.and_then(|cid| category_cover_map.get(&cid).cloned());
        let author_name = users.get(&post.user_id).cloned();
        let mut resp = PostResponse::from(post);
        
        // 处理cover_image（新字段优先，旧字段兜底）
        resp.cover_image = super::network_resources::resolve_post_cover(
            &state.db,
            cover_network_id,
            cover_image_url.as_deref(),
            cover_image_filename.as_deref(),
            resp.cover_image.as_deref(),
        ).await;
        
        // 设置category_cover_image（resolve_cover_url 已返回相对路径 /uploads/xxx 或外链URL）
        resp.category_cover_image = category_cover.filter(|u| !u.is_empty());
        
        resp.category_name = category_name;
        resp.author_name = author_name;
        resp.view_count = view_counts.get(&resp.id).copied().unwrap_or(0);
        resp.like_count = like_counts.get(&resp.id).copied().unwrap_or(0);
        resp.tags = tags_map.get(&resp.id).cloned().map(|v| if v.is_empty() { None } else { Some(v) }).flatten();
        data.push(resp);
    }

    fill_kb_names(&state.db, &mut data).await;
    Ok(Json(ApiResponse::with_pagination(data, pagination)))
}
/// GET /api/v1/admin/posts — List all posts (admin, author-filtered)

#[utoipa::path(
    get,
    path = "/api/v1/admin/posts",
    responses((status = 200, description = "成功", body = [PostResponse])),
    tag = "Posts"
)]
pub async fn list_admin_posts(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<ApiResponse<Vec<PostResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    // Non-privileged users (authors) can only see their own posts
    let effective_author_id = if !auth.is_privileged() && query.author_id.is_none() {
        Some(auth.user_id)
    } else {
        query.author_id
    };

    let (posts_list, pagination) = crate::services::posts::list_posts(
        &state.db,
        page,
        page_size,
        query.status.as_deref(),
        query.category_id,
        parse_comma_i32(query.category_ids.as_deref()).as_deref(),
        query.tag_id,
        query.tag_search.as_deref(),
        effective_author_id,
        None, // 管理端不过滤隐藏分类
        query.article_type.as_deref(),
        parse_comma_string(query.article_types.as_deref()).as_deref(),
        query.article_status.as_deref(),
        parse_comma_string(query.article_statuses.as_deref()).as_deref(),
        query.is_pinned,
        query.include_subcategories,
        query.search.as_deref(),
    )
    .await?;

    // Batch-fetch categories to avoid N+1 queries
    let category_ids: Vec<i32> = posts_list.iter().filter_map(|p| p.category_id).collect();
    let mut category_cover_map: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    let categories = if !category_ids.is_empty() {
        let cats = crate::models::entity::categories::Entity::find()
            .filter(crate::models::entity::categories::Column::Id.is_in(category_ids))
            .all(&state.db)
            .await?;
        for c in &cats {
            if let Some(cover) = super::network_resources::resolve_cover_url(
                &state.db,
                c.network_resource_id,
                c.image_url.as_deref(),
                c.image_filename.as_deref(),
            ).await {
                category_cover_map.insert(c.id, cover);
            }
        }
        cats.into_iter().map(|c| (c.id, c.name)).collect::<std::collections::HashMap<i32, String>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch tags
    let post_ids: Vec<i32> = posts_list.iter().map(|p| p.id).collect();
    let tags_map = if !post_ids.is_empty() {
        let post_tags = crate::models::entity::post_tags::Entity::find()
            .filter(crate::models::entity::post_tags::Column::PostId.is_in(post_ids.clone()))
            .all(&state.db)
            .await?;
        let tag_ids: Vec<i32> = post_tags.iter().map(|pt| pt.tag_id).collect();
        let tags_data = if !tag_ids.is_empty() {
            crate::models::entity::tags::Entity::find()
                .filter(crate::models::entity::tags::Column::Id.is_in(tag_ids))
                .all(&state.db)
                .await?
        } else { vec![] };
        let tag_name_map: std::collections::HashMap<i32, String> = tags_data.into_iter().map(|t| (t.id, t.name)).collect();
        let mut map: std::collections::HashMap<i32, Vec<String>> = std::collections::HashMap::new();
        for pt in post_tags {
            if let Some(name) = tag_name_map.get(&pt.tag_id) {
                map.entry(pt.post_id).or_default().push(name.clone());
            }
        }
        map
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch authors
    let author_ids: Vec<i32> = posts_list.iter().map(|p| p.user_id).collect();
    let authors = if !author_ids.is_empty() {
        crate::models::entity::users::Entity::find()
            .filter(crate::models::entity::users::Column::Id.is_in(author_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| (u.id, u))
            .collect::<std::collections::HashMap<i32, crate::models::entity::users::Model>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch view counts
    let view_counts = if !post_ids.is_empty() {
        let pv_stmt = sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT post_id, COUNT(*) FROM read_logs WHERE post_id IN ({}) GROUP BY post_id",
                    post_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","))
        );
        state.db.query_all(pv_stmt).await?.iter()
            .filter_map(|row| {
                let id: Option<i32> = row.try_get_by_index(0).ok();
                let cnt: Option<i32> = row.try_get_by_index(1).ok();
                match (id, cnt) {
                    (Some(id), Some(cnt)) => Some((id, cnt)),
                    _ => None,
                }
            })
            .collect::<std::collections::HashMap<i32, i32>>()
    } else {
        std::collections::HashMap::new()
    };

    // Build response
    let mut data = Vec::new();
    for p in posts_list {
        let cover_network_id = p.cover_network_id;
        let cover_image_url = p.cover_image_url.clone();
        let cover_image_filename = p.cover_image_filename.clone();
        let category_name = p.category_id.and_then(|cid| categories.get(&cid).cloned());
        let category_cover = p.category_id.and_then(|cid| category_cover_map.get(&cid).cloned());
        let author = authors.get(&p.user_id);
        let author_name = author.map(|a| a.display_name.clone().unwrap_or_else(|| a.username.clone()));
        let mut resp = PostResponse::from(p);
        // Resolve cover_image（新字段优先，旧字段兜底）
        resp.cover_image = super::network_resources::resolve_post_cover(
            &state.db,
            cover_network_id,
            cover_image_url.as_deref(),
            cover_image_filename.as_deref(),
            resp.cover_image.as_deref(),
        ).await;
        // Set category_cover_image from category fallback（已是相对路径 /uploads/xxx 或外链URL）
        resp.category_cover_image = category_cover.filter(|u| !u.is_empty());
        resp.category_name = category_name;
        resp.author_name = author_name;
        resp.view_count = view_counts.get(&resp.id).copied().unwrap_or(0);
        resp.like_count = 0;
        resp.tags = tags_map.get(&resp.id).cloned().and_then(|v| if v.is_empty() { None } else { Some(v) });
        data.push(resp);
    }

    fill_kb_names(&state.db, &mut data).await;
    Ok(Json(ApiResponse::with_pagination(data, pagination)))
}
/// GET /api/v1/posts/{id} — Get post by ID

#[utoipa::path(
    get,
    path = "/api/v1/posts/{id}",
    responses((status = 200, description = "成功", body = PostResponse)),
    tag = "Posts"
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    let post = crate::services::posts::get_post(&state.db, id).await?;
    let category_name = crate::services::posts::get_category_name(&state.db, post.category_id).await;
    let tags = crate::services::posts::get_post_tags(&state.db, post.id).await?;

    // Real view count from read_logs
    let view_count: i64 = state.db.query_one(
        sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM read_logs WHERE post_id = {}", id),
        ),
    ).await?.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0);

    // Real like count from likes
    let real_likes: i32 = state.db.query_one(
        sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM likes WHERE post_id = {}", id),
        )
    ).await?
    .and_then(|r| r.try_get_by_index::<i64>(0).ok())
    .map(|v| v as i32)
    .unwrap_or(0);

    // Author info
    let author = users::Entity::find_by_id(post.user_id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
        .map(|u| AuthorInfo {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
        });

    let cover_network_id = post.cover_network_id;
    let cover_image_url = post.cover_image_url.clone();
    let cover_image_filename = post.cover_image_filename.clone();
    let mut resp = PostResponse::from(post);
    // Resolve cover_image（新字段优先，旧字段兜底）
    resp.cover_image = super::network_resources::resolve_post_cover(
        &state.db,
        cover_network_id,
        cover_image_url.as_deref(),
        cover_image_filename.as_deref(),
        resp.cover_image.as_deref(),
    ).await;
    // Resolve ./uploads/ relative paths to root-relative URLs
    if let Some(ref content) = resp.content {
        resp.content = Some(content.replace("./uploads/", "/uploads/"));
    }
    if let Some(ref html) = resp.content_html {
        resp.content_html = Some(html.replace("./uploads/", "/uploads/"));
    }

    // 先将 nr:{id} 替换为真实 URL（必须在生成 content_html 之前）
    if let Some(ref content) = resp.content {
        resp.content = Some(super::network_resources::resolve_nr_in_content(&state.db, content).await);
    }

    // Generate content_html from content if missing, empty, or missing referrerpolicy on images
    let needs_render = resp.content_html.as_deref().map_or(true, |s| {
        s.is_empty() || (s.contains("<img") && !s.contains("referrerpolicy"))
    });
    if needs_render {
        if let Some(ref content) = resp.content {
            resp.content_html = Some(crate::services::posts::render_markdown(&state.db, content).await);
        }
    }

    // 如果 content_html 已存在于 DB（含 <img src="nr:N">），也需解析
    if let Some(ref html) = resp.content_html {
        resp.content_html = Some(super::network_resources::resolve_nr_in_content(&state.db, html).await);
    }

    resp.category_name = category_name;
    resp.tags = Some(tags.into_iter().map(|t| t.name).collect());
    resp.view_count = view_count as i32;
    resp.like_count = real_likes;
    resp.author = author;

    // 设置 category_cover_image 作为 fallback：网络URL优先
    if let Some(category_id) = resp.category_id {
        if let Some(cat) = crate::models::entity::categories::Entity::find_by_id(category_id)
            .one(&state.db)
            .await
            .ok()
            .flatten()
        {
            resp.category_cover_image = super::network_resources::resolve_cover_url(
                &state.db,
                cat.network_resource_id,
                cat.image_url.as_deref(),
                cat.image_filename.as_deref(),
            ).await;
        }
    }

    // Fill KB names for single post
    if let Some(ref code) = resp.article_type {
        if let Ok(Some(t)) = article_types::Entity::find()
            .filter(article_types::Column::Code.eq(code.clone()))
            .one(&state.db).await
        {
            resp.article_type_name = Some(t.display_name);
        }
    }
    if let Some(ref code) = resp.article_status {
        if let Ok(Some(s)) = article_statuses::Entity::find()
            .filter(article_statuses::Column::Code.eq(code.clone()))
            .one(&state.db).await
        {
            resp.article_status_name = Some(s.display_name);
        }
    }

    Ok(Json(ApiResponse::new(resp)))
}
/// POST /api/v1/posts — Create a new post

#[utoipa::path(
    post,
    path = "/api/v1/posts",
    responses((status = 200, description = "成功", body = PostResponse)),
    tag = "Posts"
)]
pub async fn create_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    let base_slug = req.slug.unwrap_or_else(|| crate::services::posts::generate_slug(&req.title));
    
    // Ensure slug uniqueness (including deleted posts since slug is unique in database)
    let mut slug = base_slug.clone();
    let mut counter = 1;
    loop {
        let existing = posts::Entity::find()
            .filter(posts::Column::Slug.eq(&slug))
            .one(&state.db)
            .await?;
        if existing.is_none() {
            break;
        }
        counter += 1;
        slug = format!("{}-{}", base_slug, counter);
    }
    // Store normalized content (relative paths: ./uploads/...)
    let content_to_store = req.content.as_ref().map(|c| {
        normalize_uploads_url(c)
    });
    // 将已知网络资源 URL 替换为 nr:{id}（保存时反向归一化）
    let content_to_store = match content_to_store {
        Some(c) => Some(super::network_resources::normalize_nr_in_content(&state.db, &c).await),
        None => None,
    };
    let content_html = if let Some(ref content) = req.content {
        let normalized = normalize_uploads_url(content);
        // 先生成 nr:→URL 的解析版本，确保 comrak 能正确渲染 <img src>
        let resolved = super::network_resources::resolve_nr_in_content(&state.db, &normalized).await;
        Some(crate::services::posts::render_markdown(&state.db, &resolved).await)
    } else {
        None
    };
    let now = crate::utils::now_local();
    let status = req.status.unwrap_or_else(|| "draft".to_string());
    let published_at = if status == "published" {
        Some(now)
    } else {
        None
    };

    // 封面有三种来源：本地文件 | nr:{id} 网络资源 | 普通外链 URL（不入库）
    // 自托管 URL → 转为 ./uploads/xxx；外部URL/nr: → 保持原样
    let cover_image = req.cover_image.map(|url| {
        if url.starts_with("nr:") { return url; }
        if url.starts_with("http://") || url.starts_with("https://") {
            normalize_uploads_url(&url)
        } else {
            url
        }
    });
    // 拆分为新字段：URL 类（nr: + http(s)）vs 本地文件名
    let (cover_image_url, cover_image_filename) = match &cover_image {
        Some(url) if url.starts_with("nr:") || url.starts_with("http://") || url.starts_with("https://") => {
            (Some(url.clone()), None)
        }
        Some(name) => (None, Some(name.clone())),
        None => (None, None),
    };
    let cover_network_id = if let Some(ref url) = cover_image {
        if url.starts_with("nr:") {
            super::network_resources::ensure_url(&state.db, url).await?
        } else {
            None // 普通外链或本地文件，不自动注册
        }
    } else {
        None
    };

    let post_model = posts::ActiveModel {
        user_id: Set(auth.user_id),
        category_id: Set(req.category_id),
        title: Set(req.title),
        slug: Set(slug),
        summary: Set(req.summary),
        content: Set(content_to_store),
        content_html: Set(content_html),
        cover_image: Set(cover_image),               // 🔒 旧字段，兼容
        cover_image_url: Set(cover_image_url),        // 🆕 URL 类
        cover_image_filename: Set(cover_image_filename), // 🆕 本地文件名
        cover_network_id: Set(cover_network_id),
        status: Set(status.clone()),
        post_type: Set(req.post_type.unwrap_or_else(|| "post".to_string())),
        is_pinned: Set(req.is_pinned.unwrap_or(false)),
        allow_comment: Set(req.allow_comment.unwrap_or(true)),
        article_type: Set(req.article_type.unwrap_or_else(|| "ai_organized".to_string())),
        article_status: Set(req.article_status.unwrap_or_else(|| "latest".to_string())),
        sort_order: Set(0),
        view_count: Set(0),
        like_count: Set(0),
        published_at: Set(published_at),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let post = post_model.insert(&state.db).await?;

    if !req.tags.is_empty() {
        let tag_ids = crate::services::posts::resolve_tag_names(&state.db, &req.tags).await?;
        crate::services::posts::set_post_tags(&state.db, post.id, &tag_ids).await?;
    }

    // Index for search (only published posts)
    if status == "published" {
        let title = post.title.clone();
        let content = post.content.clone().unwrap_or_default();
        let post_id = post.id as u64;
        let _ = state.search_engine.index_document(post_id, &title, &content);
    }

    Ok(Json(ApiResponse::new(PostResponse::from(post))))
}
/// PUT /api/v1/posts/{id} — Update post

#[utoipa::path(
    put,
    path = "/api/v1/posts/{id}",
    responses((status = 200, description = "成功", body = PostResponse)),
    tag = "Posts"
)]
pub async fn update_post(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    let post = crate::services::posts::get_post(&state.db, id).await?;
    let already_published = post.published_at.is_some();
    let mut post_active: posts::ActiveModel = post.into();

    // Keep existing slug unless title changed
    if let Some(ref title) = req.title {
        // Compare against original title before overwriting
        let title_changed = title != &post_active.title.clone().unwrap();
        post_active.title = Set(title.clone());
        if title_changed {
            // Title changed — regenerate slug with collision avoidance
            let base_slug = crate::services::posts::generate_slug(title);
            let mut slug = base_slug.clone();
            let mut counter = 1;
            loop {
                let existing = posts::Entity::find()
                    .filter(posts::Column::Slug.eq(&slug))
                    .filter(posts::Column::Id.ne(post_active.id.clone().unwrap()))
                    .one(&state.db)
                    .await?;
                if existing.is_none() {
                    break;
                }
                counter += 1;
                slug = format!("{}-{}", base_slug, counter);
            }
            post_active.slug = Set(slug);
        }
    }
    if let Some(content) = req.content {
        let normalized = normalize_uploads_url(&content);
        // 将已知网络资源 URL 替换为 nr:{id}
        let normalized = super::network_resources::normalize_nr_in_content(&state.db, &normalized).await;
        // 先生成 nr:→URL 的解析版本，确保 comrak 能正确渲染 <img src>
        let resolved_for_html = super::network_resources::resolve_nr_in_content(&state.db, &normalized).await;
        let html = crate::services::posts::render_markdown(&state.db, &resolved_for_html).await;
        post_active.content = Set(Some(normalized));
        post_active.content_html = Set(Some(html));
    }
    if let Some(summary) = req.summary {
        post_active.summary = Set(Some(summary));
    }
    if let Some(category_id) = req.category_id {
        post_active.category_id = Set(category_id);
    }
    if let Some(status) = req.status {
        if status == "published" && !already_published {
            let now = crate::utils::now_local();
            post_active.published_at = Set(Some(now));
        }
        post_active.status = Set(status);
    }
    if let Some(post_type) = req.post_type {
        post_active.post_type = Set(post_type);
    }
    if let Some(cover_image) = req.cover_image {
        // 自托管 URL → ./uploads/xxx；外部URL/nr: → 保持原样
        let normalized = cover_image.map(|url| {
            if url.starts_with("nr:") { return url; }
            if url.starts_with("http://") || url.starts_with("https://") {
                normalize_uploads_url(&url)
            } else {
                url
            }
        });
        // 拆分为新字段：URL 类（nr: + http(s)）vs 本地文件名
        let (cover_url, cover_filename) = match &normalized {
            Some(url) if url.starts_with("nr:") || url.starts_with("http://") || url.starts_with("https://") => {
                (Some(url.clone()), None)
            }
            Some(name) => (None, Some(name.clone())),
            None => (None, None),
        };
        // nr:{id} → 验证网络资源；普通外链/本地文件 → 不入库
        let nr_id = if normalized.as_ref().map_or(false, |s| s.starts_with("nr:")) {
            super::network_resources::ensure_url(&state.db, normalized.as_ref().unwrap_or(&"".to_string())).await?
        } else {
            None
        };
        post_active.cover_image = Set(normalized.clone());      // 🔒 旧字段，兼容
        post_active.cover_image_url = Set(cover_url);            // 🆕 URL 类
        post_active.cover_image_filename = Set(cover_filename);  // 🆕 本地文件名
        post_active.cover_network_id = Set(nr_id);
    }
    if let Some(is_pinned) = req.is_pinned {
        post_active.is_pinned = Set(is_pinned);
    }
    if let Some(allow_comment) = req.allow_comment {
        post_active.allow_comment = Set(allow_comment);
    }
    if let Some(article_type) = req.article_type {
        post_active.article_type = Set(article_type);
    }
    if let Some(article_status) = req.article_status {
        post_active.article_status = Set(article_status);
    }
    if let Some(author_id) = req.author_id {
        post_active.user_id = Set(author_id);
    }

    post_active.updated_at = Set(crate::utils::now_local());

    let updated = post_active.update(&state.db).await?;

    if !req.tags.is_empty() {
        let tag_ids = crate::services::posts::resolve_tag_names(&state.db, &req.tags).await?;
        crate::services::posts::set_post_tags(&state.db, updated.id, &tag_ids).await?;
    }

    // Update search index: index if published, remove otherwise
    if updated.status == "published" {
        let title = updated.title.clone();
        let content = updated.content.clone().unwrap_or_default();
        let post_id = updated.id as u64;
        let _ = state.search_engine.index_document(post_id, &title, &content);
    } else {
        let _ = state.search_engine.delete_from_index(updated.id as u64);
    }

    Ok(Json(ApiResponse::new(PostResponse::from(updated))))
}
/// DELETE /api/v1/posts/{id} — Soft-delete post

#[utoipa::path(
    delete,
    path = "/api/v1/posts/{id}",
    responses((status = 200, description = "成功")),
    tag = "Posts"
)]
pub async fn delete_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // 作者不能删除已发布的文章
    if !auth.is_privileged() {
        let post = posts::Entity::find_by_id(id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound("文章不存在".into()))?;
        if post.status == "published" && post.user_id == auth.user_id {
            return Err(AppError::Forbidden);
        }
    }

    crate::services::posts::delete_post(&state.db, id).await?;

    // Remove from search index
    let _ = state.search_engine.delete_from_index(id as u64);

    Ok(Json(ApiResponse::new(())))
}
/// GET /api/v1/posts/slug/{slug} — Get post by slug

#[utoipa::path(
    get,
    path = "/api/v1/posts/slug/{slug}",
    responses((status = 200, description = "成功", body = PostResponse)),
    tag = "Posts"
)]
pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    _headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    let post = crate::services::posts::get_post_by_slug(&state.db, &slug).await?;
    let category_name = crate::services::posts::get_category_name(&state.db, post.category_id).await;
    let tags = crate::services::posts::get_post_tags(&state.db, post.id).await?;

    // Real view count from read_logs (recorded by frontend POST /api/v1/read-logs)
    let view_count: i64 = state.db.query_one(
        sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM read_logs WHERE post_id = {}", post.id),
        ),
    ).await?.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0);

    // Real like count from likes
    let real_likes: i32 = state.db.query_one(
        sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM likes WHERE post_id = {}", post.id),
        )
    ).await?
    .and_then(|r| r.try_get_by_index::<i64>(0).ok())
    .map(|v| v as i32)
    .unwrap_or(0);

    // Author info
    let author = users::Entity::find_by_id(post.user_id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
        .map(|u| AuthorInfo {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
        });
    
    let cover_network_id = post.cover_network_id;
    let cover_image_url = post.cover_image_url.clone();
    let cover_image_filename = post.cover_image_filename.clone();
    let mut resp = PostResponse::from(post);
    // Resolve cover_image（新字段优先，旧字段兜底）
    resp.cover_image = super::network_resources::resolve_post_cover(
        &state.db,
        cover_network_id,
        cover_image_url.as_deref(),
        cover_image_filename.as_deref(),
        resp.cover_image.as_deref(),
    ).await;
    // Resolve ./uploads/ relative paths to root-relative URLs
    if let Some(ref content) = resp.content {
        resp.content = Some(content.replace("./uploads/", "/uploads/"));
    }
    if let Some(ref html) = resp.content_html {
        resp.content_html = Some(html.replace("./uploads/", "/uploads/"));
    }

    // 先将 nr:{id} 替换为真实 URL（必须在生成 content_html 之前）
    if let Some(ref content) = resp.content {
        resp.content = Some(super::network_resources::resolve_nr_in_content(&state.db, content).await);
    }

    // Generate content_html from content if missing, empty, or missing referrerpolicy on images
    let needs_render = resp.content_html.as_deref().map_or(true, |s| {
        s.is_empty() || (s.contains("<img") && !s.contains("referrerpolicy"))
    });
    if needs_render {
        if let Some(ref content) = resp.content {
            resp.content_html = Some(crate::services::posts::render_markdown(&state.db, content).await);
        }
    }

    // 如果 content_html 已存在于 DB（含 <img src="nr:N">），也需解析
    if let Some(ref html) = resp.content_html {
        resp.content_html = Some(super::network_resources::resolve_nr_in_content(&state.db, html).await);
    }

    resp.category_name = category_name;
    resp.tags = Some(tags.into_iter().map(|t| t.name).collect());
    resp.view_count = view_count as i32;
    resp.like_count = real_likes;
    resp.author = author;

    // 设置 category_cover_image 作为 fallback：网络URL优先
    if let Some(category_id) = resp.category_id {
        if let Some(cat) = crate::models::entity::categories::Entity::find_by_id(category_id)
            .one(&state.db)
            .await
            .ok()
            .flatten()
        {
            resp.category_cover_image = super::network_resources::resolve_cover_url(
                &state.db,
                cat.network_resource_id,
                cat.image_url.as_deref(),
                cat.image_filename.as_deref(),
            ).await;
        }
    }

    // Fill KB names for single post
    if let Some(ref code) = resp.article_type {
        if let Ok(Some(t)) = article_types::Entity::find()
            .filter(article_types::Column::Code.eq(code.clone()))
            .one(&state.db).await
        {
            resp.article_type_name = Some(t.display_name);
        }
    }
    if let Some(ref code) = resp.article_status {
        if let Ok(Some(s)) = article_statuses::Entity::find()
            .filter(article_statuses::Column::Code.eq(code.clone()))
            .one(&state.db).await
        {
            resp.article_status_name = Some(s.display_name);
        }
    }

    Ok(Json(ApiResponse::new(resp)))
}
/// GET /api/v1/posts/{id}/adjacent — Get adjacent posts (prev/next)

#[utoipa::path(
    get,
    path = "/api/v1/posts/{id}/adjacent",
    responses((status = 200, description = "成功", body = AdjacentPostsResponse)),
    tag = "Posts"
)]
pub async fn get_adjacent_posts(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<AdjacentPostsResponse>>, AppError> {
    let (prev, next) = crate::services::posts::get_adjacent_posts(&state.db, id).await?;
    Ok(Json(ApiResponse::new(AdjacentPostsResponse {
        prev: prev.map(|(id, title, slug)| AdjacentPost { id, title, slug }),
        next: next.map(|(id, title, slug)| AdjacentPost { id, title, slug }),
    })))
}

// ─── Like endpoints ───────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct LikeStatusResponse {
    pub liked: bool,
    pub like_count: i32,
}
/// POST /api/v1/posts/{id}/like — Toggle like on a post

#[utoipa::path(
    post,
    path = "/api/v1/posts/{id}/like",
    responses((status = 200, description = "成功")),
    tag = "Posts"
)]
pub async fn toggle_like(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(post_id): Path<i32>,
) -> Result<Json<ApiResponse<LikeStatusResponse>>, AppError> {
    let db = &state.db;
    let user_id = auth.user_id;

    // Check if already liked
    let row = db.query_one(
        sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!("SELECT id FROM likes WHERE user_id = {} AND post_id = {}", user_id, post_id),
        )
    ).await?;

    let liked = if row.is_some() {
        // Unlike: remove like, decrement counter
        db.execute_unprepared(&format!(
            "DELETE FROM likes WHERE user_id = {} AND post_id = {}",
            user_id, post_id
        )).await?;
        db.execute_unprepared(&format!(
            "UPDATE posts SET like_count = MAX(0, like_count - 1) WHERE id = {}",
            post_id
        )).await?;
        false
    } else {
        // Like: insert, increment counter
        db.execute_unprepared(&format!(
            "INSERT INTO likes (user_id, post_id) VALUES ({}, {})",
            user_id, post_id
        )).await?;
        db.execute_unprepared(&format!(
            "UPDATE posts SET like_count = like_count + 1 WHERE id = {}",
            post_id
        )).await?;
        true
    };

    // Get updated like_count
    let count: i32 = db.query_one(
        sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!("SELECT like_count FROM posts WHERE id = {}", post_id),
        )
    ).await?
    .and_then(|r| r.try_get_by_index::<i32>(0).ok())
    .unwrap_or(0);

    Ok(Json(ApiResponse::new(LikeStatusResponse {
        liked,
        like_count: count,
    })))
}
/// GET /api/v1/posts/{id}/like-status — Get like status for current user

#[utoipa::path(
    get,
    path = "/api/v1/posts/{id}/like-status",
    responses((status = 200, description = "成功", body = LikeStatusResponse)),
    tag = "Posts"
)]
pub async fn get_like_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(post_id): Path<i32>,
) -> Result<Json<ApiResponse<LikeStatusResponse>>, AppError> {
    let db = &state.db;

    let row = db.query_one(
        sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!("SELECT id FROM likes WHERE user_id = {} AND post_id = {}", auth.user_id, post_id),
        )
    ).await?;

    let count: i32 = db.query_one(
        sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!("SELECT like_count FROM posts WHERE id = {}", post_id),
        )
    ).await?
    .and_then(|r| r.try_get_by_index::<i32>(0).ok())
    .unwrap_or(0);

    Ok(Json(ApiResponse::new(LikeStatusResponse {
        liked: row.is_some(),
        like_count: count,
    })))
}
/// POST /api/v1/posts/{id}/view — Record a page view
///
/// POST /api/v1/read-logs — Record a read log entry
#[derive(Deserialize)]
pub struct RecordReadLogRequest {
    pub post_id: i32,
    pub duration_seconds: Option<i32>,
    pub referrer: Option<String>,
}

/// 检查 IP 是否在白名单中（白名单启用时）
async fn is_ip_whitelisted(state: &AppState, ip: &str) -> bool {
    use crate::models::entity::settings;
    use crate::utils::ip_utils;
    let items = settings::Entity::find().all(&state.db).await.unwrap_or_default();
    let get = |key: &str| items.iter().find(|s| s.key == key).map(|s| s.value.as_str());

    if get("ip_whitelist_enabled") != Some("true") {
        return false;
    }
    match get("ip_whitelist") {
        Some(json) => ip_utils::parse_valid_ips(json).contains(&ip.to_string()),
        None => false,
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/read-logs",
    tag = "Posts",
    request_body = RecordReadLogRequest,
    responses((status = 200, description = "已记录")),
)]
pub async fn record_read_log(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    auth: Option<crate::middleware::auth::AuthUser>,
    Json(req): Json<RecordReadLogRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    use crate::utils::client_info;
    use crate::models::entity::read_logs;

    let ip = client_info::extract_client_ip(&headers, Some(socket_addr));

    // 检查是否在白名单中 — 白名单 IP 不记录阅读日志
    if let Some(ref ip_str) = ip {
        if is_ip_whitelisted(&state, ip_str).await {
            return Ok(Json(ApiResponse::new(())));
        }
    }

    // Dedup: same IP + post_id within 30s → skip
    let thirty_secs_ago = crate::utils::now_local() - chrono::Duration::seconds(30);
    let recent = read_logs::Entity::find()
        .filter(read_logs::Column::PostId.eq(req.post_id))
        .filter(read_logs::Column::IpAddress.eq(ip.clone()))
        .filter(read_logs::Column::CreatedAt.gt(thirty_secs_ago))
        .count(&state.db)
        .await
        .unwrap_or(0);
    if recent > 0 {
        return Ok(Json(ApiResponse::new(())));
    }

    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let device_type = user_agent.as_deref()
        .map(|ua| client_info::parse_device_type(ua).to_string());

    let now = crate::utils::now_local();
    let log = read_logs::ActiveModel {
        post_id: Set(req.post_id),
        user_id: Set(auth.as_ref().map(|a| a.user_id)),
        ip_address: Set(ip),
        user_agent: Set(user_agent),
        device_type: Set(device_type),
        referrer: Set(req.referrer),
        duration_seconds: Set(req.duration_seconds.unwrap_or(0)),
        created_at: Set(now),
        ..Default::default()
    };
    // 防御式写入：表可能尚未创建（如迁移未执行），避免 500
    log.insert(&state.db).await.ok();
    Ok(Json(ApiResponse::new(())))
}

// ── Authors ──

#[derive(Serialize, ToSchema)]
pub struct AuthorListItem {
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub post_count: i64,
}
/// GET /api/v1/authors — List authors

#[utoipa::path(
    get,
    path = "/api/v1/authors",
    responses((status = 200, description = "成功", body = [AuthorInfo])),
    tag = "Posts"
)]
pub async fn list_authors(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<AuthorListItem>>>, AppError> {
    // Subquery: users who have published posts, with counts
    let sql = "SELECT u.id, u.username, u.display_name, u.avatar_url, u.bio, COUNT(p.id) as post_count \
               FROM users u \
               INNER JOIN posts p ON p.user_id = u.id \
               WHERE p.status = 'published' AND p.deleted_at IS NULL \
               GROUP BY u.id \
               ORDER BY post_count DESC";
    let rows = state.db.query_all(sea_orm::Statement::from_string(
        state.db.get_database_backend(), sql.to_string(),
    )).await?;

    let authors: Vec<AuthorListItem> = rows.into_iter().filter_map(|r| {
        Some(AuthorListItem {
            id: r.try_get_by_index::<i32>(0).ok()?,
            username: r.try_get_by_index::<String>(1).ok()?,
            display_name: r.try_get_by_index::<Option<String>>(2).ok()?,
            avatar_url: r.try_get_by_index::<Option<String>>(3).ok()?,
            bio: r.try_get_by_index::<Option<String>>(4).ok()?,
            post_count: r.try_get_by_index::<i64>(5).ok()?,
        })
    }).collect();

    Ok(Json(ApiResponse::new(authors)))
}

// ── Unified Search ──

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Serialize, ToSchema)]
pub struct SearchArticle {
    pub id: i32,
    pub title: String,
    pub slug: String,
}

#[derive(Serialize, ToSchema)]
pub struct SearchTag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub post_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct SearchAuthor {
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SearchResponse {
    pub articles: Vec<SearchArticle>,
    pub tags: Vec<SearchTag>,
    pub authors: Vec<SearchAuthor>,
}
/// GET /api/v1/search — Unified full-text search

#[utoipa::path(
    get,
    path = "/api/v1/search",
    responses((status = 200, description = "成功", body = SearchResponse)),
    tag = "Posts"
)]
pub async fn unified_search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<ApiResponse<SearchResponse>>, AppError> {
    let q = q.q.trim().to_string();
    if q.is_empty() {
        return Ok(Json(ApiResponse::new(SearchResponse {
            articles: vec![],
            tags: vec![],
            authors: vec![],
        })));
    }

    // 1. Tantivy full-text search on articles
    let article_ids = state.search_engine.search(&q, 20).unwrap_or_default();
    let articles: Vec<SearchArticle> = if !article_ids.is_empty() {
        let ids: Vec<i32> = article_ids.into_iter().map(|id| id as i32).collect();
        posts::Entity::find()
            .filter(posts::Column::Id.is_in(ids))
            .filter(posts::Column::Status.eq("published"))
            .filter(posts::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map(|items| {
                items.into_iter().map(|p| SearchArticle {
                    id: p.id,
                    title: p.title,
                    slug: p.slug,
                }).collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    // 2. SQL LIKE search on tags
    let tag_like = format!("%{}%", q.replace('%', "\\%").replace('_', "\\_"));
    let tags: Vec<SearchTag> = {
        let sql = format!(
            "SELECT t.id, t.name, t.slug, COUNT(pt.post_id) as post_count \
             FROM tags t \
             LEFT JOIN post_tags pt ON pt.tag_id = t.id \
             WHERE t.name LIKE ? \
             GROUP BY t.id \
             ORDER BY post_count DESC \
             LIMIT 10"
        );
        state.db.query_all(sea_orm::Statement::from_sql_and_values(
            state.db.get_database_backend(),
            &sql,
            vec![tag_like.into()],
        )).await
        .map(|rows| {
            rows.into_iter().filter_map(|r| {
                Some(SearchTag {
                    id: r.try_get_by_index::<i32>(0).ok()?,
                    name: r.try_get_by_index::<String>(1).ok()?,
                    slug: r.try_get_by_index::<String>(2).ok()?,
                    post_count: r.try_get_by_index::<i64>(3).ok()?,
                })
            }).collect()
        })
        .unwrap_or_default()
    };

    // 3. SQL LIKE search on authors (users with published posts)
    let author_like = format!("%{}%", q.replace('%', "\\%").replace('_', "\\_"));
    let authors: Vec<SearchAuthor> = {
        let sql = format!(
            "SELECT u.id, u.username, u.display_name \
             FROM users u \
             INNER JOIN posts p ON p.user_id = u.id \
             WHERE p.status = 'published' AND p.deleted_at IS NULL \
               AND (u.username LIKE ? OR u.display_name LIKE ?) \
             GROUP BY u.id \
             ORDER BY COUNT(p.id) DESC \
             LIMIT 10"
        );
        state.db.query_all(sea_orm::Statement::from_sql_and_values(
            state.db.get_database_backend(),
            &sql,
            vec![author_like.clone().into(), author_like.into()],
        )).await
        .map(|rows| {
            rows.into_iter().filter_map(|r| {
                Some(SearchAuthor {
                    id: r.try_get_by_index::<i32>(0).ok()?,
                    username: r.try_get_by_index::<String>(1).ok()?,
                    display_name: r.try_get_by_index::<Option<String>>(2).ok()?,
                })
            }).collect()
        })
        .unwrap_or_default()
    };

    Ok(Json(ApiResponse::new(SearchResponse { articles, tags, authors })))
}

// ── 置顶相关 ──

#[derive(Deserialize)]
pub struct UpdatePinOrderRequest {
    pub post_ids: Vec<i32>,
}

/// POST /api/v1/admin/posts/{id}/pin — 置顶文章
pub async fn pin_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    let post = posts::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("文章不存在".to_string()))?;

    // Get current max pin_order
    let max_order: i64 = state.db.query_one(
        sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            "SELECT COALESCE(MAX(sort_order), 0) FROM posts WHERE is_pinned = 1".to_string(),
        ),
    ).await?.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0);

    let mut active: posts::ActiveModel = post.into();
    active.is_pinned = Set(true);
    active.sort_order = Set(max_order as i32 + 1);
    let updated = active.update(&state.db).await?;

    Ok(Json(ApiResponse::new(PostResponse::from(updated))))
}

/// POST /api/v1/admin/posts/{id}/unpin — 取消置顶
pub async fn unpin_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    let post = posts::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("文章不存在".to_string()))?;

    let mut active: posts::ActiveModel = post.into();
    active.is_pinned = Set(false);
    active.sort_order = Set(0);
    let updated = active.update(&state.db).await?;

    Ok(Json(ApiResponse::new(PostResponse::from(updated))))
}

/// PUT /api/v1/admin/posts/pin-order — 更新置顶排序
pub async fn update_pin_order(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<UpdatePinOrderRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    for (idx, post_id) in req.post_ids.iter().enumerate() {
        let post = posts::Entity::find_by_id(*post_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound(format!("文章 {} 不存在", post_id)))?;
        let mut active: posts::ActiveModel = post.into();
        active.sort_order = Set(idx as i32);
        active.update(&state.db).await?;
    }
    Ok(Json(ApiResponse::new("排序已更新".to_string())))
}

/// GET /api/v1/posts/pinned — 获取置顶文章列表（公开）
pub async fn list_pinned_posts(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PostResponse>>>, AppError> {
    let posts_list = posts::Entity::find()
        .filter(posts::Column::IsPinned.eq(true))
        .filter(posts::Column::Status.eq("published"))
        .filter(posts::Column::DeletedAt.is_null())
        .order_by_asc(posts::Column::SortOrder)
        .all(&state.db)
        .await?;

    // Batch-fetch categories + category covers
    let category_ids: Vec<i32> = posts_list.iter().filter_map(|p| p.category_id).collect();
    let mut category_cover_map: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    let categories = if !category_ids.is_empty() {
        let cats = crate::models::entity::categories::Entity::find()
            .filter(crate::models::entity::categories::Column::Id.is_in(category_ids))
            .all(&state.db)
            .await?;
        for c in &cats {
            if let Some(cover) = super::network_resources::resolve_cover_url(
                &state.db,
                c.network_resource_id,
                c.image_url.as_deref(),
                c.image_filename.as_deref(),
            ).await {
                category_cover_map.insert(c.id, cover);
            }
        }
        cats.into_iter().map(|c| (c.id, c.name)).collect::<std::collections::HashMap<i32, String>>()
    } else {
        std::collections::HashMap::new()
    };

    // Batch-fetch authors to get display_name
    let user_ids: Vec<i32> = posts_list.iter().map(|p| p.user_id).collect();
    let users = if !user_ids.is_empty() {
        let us = users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .all(&state.db)
            .await?;
        us.into_iter().map(|u| (u.id, u.display_name.unwrap_or(u.username))).collect::<std::collections::HashMap<i32, String>>()
    } else {
        std::collections::HashMap::new()
    };

    let mut data: Vec<PostResponse> = Vec::new();
    for post in posts_list {
        let cover_network_id = post.cover_network_id;
        let cover_image_url = post.cover_image_url.clone();
        let cover_image_filename = post.cover_image_filename.clone();
        let category_id = post.category_id;
        let user_id = post.user_id;
        let mut resp = PostResponse::from(post);
        let category_name = category_id.and_then(|cid| categories.get(&cid).cloned());
        let category_cover = category_id.and_then(|cid| category_cover_map.get(&cid).cloned());
        let author_name = users.get(&user_id).cloned();

        // Resolve cover image
        resp.cover_image = super::network_resources::resolve_post_cover(
            &state.db,
            cover_network_id,
            cover_image_url.as_deref(),
            cover_image_filename.as_deref(),
            resp.cover_image.as_deref(),
        ).await;

        resp.category_cover_image = category_cover.filter(|u| !u.is_empty());
        resp.category_name = category_name;
        resp.author_name = author_name;
        data.push(resp);
    }

    fill_kb_names(&state.db, &mut data).await;
    Ok(Json(ApiResponse::new(data)))
}
