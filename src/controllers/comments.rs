use axum::{
    extract::{State, Path, Query},
    Json,
};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::utils::{AppState, AppError, ApiResponse, Pagination};
use crate::middleware::auth::AuthUser;
use crate::models::entity::comments;
use crate::models::entity::users;
use sea_orm::*;
use sea_orm::sea_query::Expr;

// ── Response types ──

#[derive(Serialize, ToSchema)]
pub struct CommentResponse {
    pub id: i32,
    pub post_id: i32,
    pub post_title: Option<String>,
    pub user_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub content_html: String,
    pub status: String,
    pub like_count: i32,
    pub created_at: String,
    pub updated_at: String,
    pub replies: Vec<CommentResponse>,
}

impl From<comments::Model> for CommentResponse {
    fn from(c: comments::Model) -> Self {
        Self {
            id: c.id,
            post_id: c.post_id,
            post_title: None,
            user_id: c.user_id,
            parent_id: c.parent_id,
            author_name: c.author_name,
            author_email: c.author_email,
            content: c.content,
            content_html: c.content_html,
            status: c.status,
            like_count: c.like_count,
            created_at: c.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: c.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            replies: vec![],
        }
    }
}

// ── Request types ──

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub parent_id: Option<i32>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCommentStatusRequest {
    pub status: String, // pending / approved / deleted
}

#[derive(Deserialize)]
pub struct ListCommentsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<String>,
}

// ── Public API ──

/// List comments for a post (flat + nested replies)
/// - Non-admin: approved only (public view)
/// - Admin/sub_admin: all comments including pending (via ?admin=1 query param)
#[derive(Deserialize)]
pub struct PostCommentsQuery {
    pub admin: Option<String>,
}
/// GET /api/v1/posts/{id}/comments — List comments for a post

#[utoipa::path(
    get,
    path = "/api/v1/posts/{id}/comments",
    responses((status = 200, description = "成功", body = [CommentResponse])),
    tag = "Comments"
)]
pub async fn list_post_comments(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    Query(query): Query<PostCommentsQuery>,
    auth: Option<AuthUser>,
) -> Result<Json<ApiResponse<Vec<CommentResponse>>>, AppError> {
    let is_admin_view = query.admin.as_deref() == Some("1")
        && auth.as_ref().map_or(false, |a| matches!(a.role.as_str(), "admin" | "sub_admin"));

    let mut select = comments::Entity::find()
        .filter(comments::Column::PostId.eq(post_id))
        .filter(comments::Column::DeletedAt.is_null());

    if !is_admin_view {
        select = select.filter(comments::Column::Status.eq("approved"));
    }

    let all = select
        .order_by_asc(comments::Column::CreatedAt)
        .all(&state.db)
        .await?;

    // Build tree: separate top-level from replies
    let mut top_level: Vec<_> = all
        .iter()
        .filter(|c| c.parent_id.is_none())
        .map(|c| CommentResponse::from(c.clone()))
        .collect();

    let replies: Vec<_> = all.iter().filter(|c| c.parent_id.is_some()).collect();

    for r in replies {
        let resp = CommentResponse::from(r.clone());
        if let Some(parent) = top_level.iter_mut().find(|p| p.id == r.parent_id.unwrap()) {
            parent.replies.push(resp);
        }
    }

    Ok(Json(ApiResponse::new(top_level)))
}
/// POST /api/v1/posts/{id}/comments — Create a comment

#[utoipa::path(
    post,
    path = "/api/v1/posts/{id}/comments",
    responses((status = 200, description = "成功", body = CommentResponse)),
    tag = "Comments"
)]
pub async fn create_comment(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    auth: Option<AuthUser>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<ApiResponse<CommentResponse>>, AppError> {
    let content = req.content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::Validation("评论内容不能为空".to_string()));
    }

    // Determine author details
    let (user_id, author_name, author_email) = if let Some(ref a) = auth {
        // Banned/muted users cannot comment
        if a.status == "banned" {
            return Err(AppError::Validation("您已被禁止评论".to_string()));
        }
        if a.status == "muted" {
            return Err(AppError::Validation("您已被禁言".to_string()));
        }
        // 登录用户：查询 display_name 作为显示名
        let display_name = users::Entity::find_by_id(a.user_id)
            .one(&state.db)
            .await
            .ok()
            .flatten()
            .and_then(|u| u.display_name)
            .unwrap_or_else(|| a.username.clone());
        (Some(a.user_id), display_name, None)
    } else {
        let name = req.author_name.as_deref().unwrap_or("匿名").trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("请填写昵称".to_string()));
        }
        (None, name, req.author_email.clone())
    };

    let content_html = crate::services::posts::render_markdown(&state.db, &content).await;
    let now = crate::utils::now_local();

    // 读取 comment_moderation 设置，决定访客/匿名留言是否需要审核
    let moderation_enabled = get_setting_bool(&state.db, "comment_moderation").await.unwrap_or(false);

    // Determine initial status
    let initial_status = if let Some(ref a) = auth {
        if matches!(a.role.as_str(), "admin" | "sub_admin") {
            "approved".to_string()
        } else if a.role == "author" {
            // 作者在自己文章下评论 → 直接通过；在别人文章下 → 遵循审核设置
            use crate::models::entity::posts;
            let is_own_post = posts::Entity::find_by_id(post_id)
                .one(&state.db)
                .await
                .ok()
                .flatten()
                .map(|p| p.user_id == a.user_id)
                .unwrap_or(false);
            if is_own_post {
                "approved".to_string()
            } else if moderation_enabled {
                "pending".to_string()
            } else {
                "approved".to_string()
            }
        } else {
            // visitor: depends on moderation setting
            if moderation_enabled { "pending".to_string() } else { "approved".to_string() }
        }
    } else {
        // anonymous: depends on moderation setting
        if moderation_enabled { "pending".to_string() } else { "approved".to_string() }
    };

    let model = comments::ActiveModel {
        post_id: Set(post_id),
        user_id: Set(user_id),
        parent_id: Set(req.parent_id),
        author_name: Set(author_name),
        author_email: Set(author_email),
        content: Set(content),
        content_html: Set(content_html),
        status: Set(initial_status),
        like_count: Set(0),
        ip_address: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    };

    let result = model.insert(&state.db).await?;

    // Update post comment_count
    if let Err(e) = update_post_comment_count(&state.db, post_id).await {
        tracing::warn!("Failed to update comment_count: {}", e);
    }

    Ok(Json(ApiResponse::new(CommentResponse::from(result))))
}
/// GET /api/v1/admin/comments — List all comments (admin)

#[utoipa::path(
    get,
    path = "/api/v1/admin/comments",
    responses((status = 200, description = "成功", body = [CommentResponse])),
    tag = "Comments"
)]
pub async fn list_all_comments(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<ApiResponse<Vec<CommentResponse>>>, AppError> {
    use crate::models::entity::posts;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut select = comments::Entity::find()
        .filter(comments::Column::DeletedAt.is_null());

    // Role-based visibility
    let is_privileged = auth.is_privileged();
    if !is_privileged {
        // Authors can see all statuses on their own posts (for moderation)
        let subquery = posts::Entity::find()
            .select_only()
            .column(posts::Column::Id)
            .filter(posts::Column::UserId.eq(auth.user_id))
            .filter(posts::Column::DeletedAt.is_null())
            .into_query();
        select = select.filter(Expr::col(comments::Column::PostId).in_subquery(subquery));
    }

    if let Some(ref status) = query.status {
        select = select.filter(comments::Column::Status.eq(status));
    }

    let paginator = select
        .order_by_desc(comments::Column::CreatedAt)
        .paginate(&state.db, page_size);

    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page - 1).await?;

    // Fetch post titles in one batch
    let post_ids: Vec<i32> = items.iter().map(|c| c.post_id).collect();
    let post_titles: std::collections::HashMap<i32, String> = if !post_ids.is_empty() {
        posts::Entity::find()
            .filter(posts::Column::Id.is_in(post_ids))
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.id, p.title))
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    let data: Vec<CommentResponse> = items.into_iter().map(|c| {
        let mut resp = CommentResponse::from(c);
        resp.post_title = post_titles.get(&resp.post_id).cloned();
        resp
    }).collect();

    let pages = ((total as f64) / (page_size as f64)).ceil() as u64;

    Ok(Json(ApiResponse::with_pagination(
        data,
        Pagination {
            total,
            pages,
            page,
            page_size,
        },
    )))
}
/// PUT /api/v1/admin/comments/{id} — Update comment status

#[utoipa::path(
    put,
    path = "/api/v1/admin/comments/{id}",
    responses((status = 200, description = "成功", body = CommentResponse)),
    tag = "Comments"
)]
pub async fn update_comment_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCommentStatusRequest>,
) -> Result<Json<ApiResponse<CommentResponse>>, AppError> {
    let comment = comments::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("评论不存在".to_string()))?;

    let mut active: comments::ActiveModel = comment.into();
    let now = crate::utils::now_local();

    match req.status.as_str() {
        "deleted" => {
            active.deleted_at = Set(Some(now));
            active.updated_at = Set(now);
            let updated = active.update(&state.db).await?;
            // Update post comment_count
            if let Err(e) = update_post_comment_count(&state.db, updated.post_id).await {
                tracing::warn!("Failed to update comment_count: {}", e);
            }
            return Ok(Json(ApiResponse::new(CommentResponse::from(updated))));
        }
        "approved" | "pending" => {
            active.status = Set(req.status);
            active.updated_at = Set(now);
        }
        _ => return Err(AppError::Validation("无效的状态".to_string())),
    }

    let updated = active.update(&state.db).await?;
    Ok(Json(ApiResponse::new(CommentResponse::from(updated))))
}

// ── Helpers ──

/// Recalculate and update the comment_count for a post
async fn update_post_comment_count(db: &DatabaseConnection, post_id: i32) -> Result<(), DbErr> {
    use crate::models::entity::posts;
    let count = comments::Entity::find()
        .filter(comments::Column::PostId.eq(post_id))
        .filter(comments::Column::DeletedAt.is_null())
        .filter(comments::Column::Status.eq("approved"))
        .count(db)
        .await?;

    let post = posts::Entity::find_by_id(post_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("post not found".to_string()))?;

    let mut active: posts::ActiveModel = post.into();
    active.comment_count = Set(count as i32);
    active.update(db).await?;
    Ok(())
}

/// GET /api/v1/admin/comments/pending-count — Pending comments count
/// Optional query: ?scope=mine — only count pending comments on the auth user's posts
#[derive(Deserialize)]
pub struct PendingCountQuery {
    pub scope: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/comments/pending-count",
    responses((status = 200, description = "成功")),
    tag = "Comments"
)]
pub async fn pending_count(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    Query(query): Query<PendingCountQuery>,
) -> Result<Json<ApiResponse<u64>>, AppError> {
    let mut select = comments::Entity::find()
        .filter(comments::Column::Status.eq("pending"))
        .filter(comments::Column::DeletedAt.is_null());

    // scope=mine: only count pending comments on the authenticated user's posts
    if query.scope.as_deref() == Some("mine") {
        if let Some(ref a) = auth {
            use crate::models::entity::posts;
            // admin/sub_admin: return all pending (they can manage everything)
            if !matches!(a.role.as_str(), "admin" | "sub_admin") {
                let subquery = posts::Entity::find()
                    .select_only()
                    .column(posts::Column::Id)
                    .filter(posts::Column::UserId.eq(a.user_id))
                    .filter(posts::Column::DeletedAt.is_null())
                    .into_query();
                select = select.filter(Expr::col(comments::Column::PostId).in_subquery(subquery));
            }
        }
    }

    let count = select.count(&state.db).await?;
    Ok(Json(ApiResponse::new(count)))
}

/// Read a boolean setting from the settings table
async fn get_setting_bool(db: &DatabaseConnection, key: &str) -> Option<bool> {
    use crate::models::entity::settings;
    settings::Entity::find()
        .filter(settings::Column::Key.eq(key))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.value == "true" || s.value == "1")
}
