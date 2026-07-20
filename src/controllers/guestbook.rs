use crate::middleware::auth::AuthUser;
use crate::models::entity::{guestbook, users};
use crate::utils::{ApiResponse, AppError, AppState, Pagination};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::sea_query::Expr;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct GuestbookEntry {
    pub id: i32,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub nickname: String,
    pub email: String,
    pub content: String,
    pub content_html: String,
    pub reply: Option<String>,
    pub is_replied: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

fn model_to_entry(m: guestbook::Model, username_map: &HashMap<i32, String>) -> GuestbookEntry {
    GuestbookEntry {
        id: m.id,
        username: m.user_id.and_then(|uid| username_map.get(&uid).cloned()),
        nickname: m.nickname,
        email: m.email,
        content: m.content,
        content_html: m.content_html,
        reply: m.reply,
        is_replied: m.is_replied,
        status: m.status,
        user_id: m.user_id,
        created_at: m.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: m.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

#[derive(Deserialize)]
pub struct CreateGuestbookRequest {
    pub nickname: String,
    pub email: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ReplyGuestbookRequest {
    pub reply: String,
}

#[derive(Deserialize)]
pub struct GuestbookQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
}

/// POST /api/v1/guestbook — 提交留言（公开）
pub async fn create_entry(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    Json(req): Json<CreateGuestbookRequest>,
) -> Result<Json<ApiResponse<GuestbookEntry>>, AppError> {
    let nickname = req.nickname.trim().to_string();
    let email = req.email.trim().to_string();

    if nickname.is_empty() || req.content.trim().is_empty() {
        return Err(AppError::BadRequest("昵称和内容不能为空".to_string()));
    }
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("请输入有效的邮箱地址".to_string()));
    }

    let (user_id, final_nickname) = if let Some(ref user) = auth {
        // 登录用户：使用账号昵称，关联 user_id
        (Some(user.user_id), user.username.clone())
    } else {
        (None, nickname)
    };

    let now = crate::utils::now_local();
    let entry = guestbook::ActiveModel {
        user_id: Set(user_id),
        nickname: Set(final_nickname),
        email: Set(email),
        content: Set(req.content.trim().to_string()),
        content_html: Set(String::new()),
        status: Set("approved".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let saved = entry.insert(&state.db).await?;
    Ok(Json(ApiResponse::new(model_to_entry(
        saved,
        &HashMap::new(),
    ))))
}

/// GET /api/v1/guestbook — 留言列表（公开，支持搜索）
pub async fn list_entries(
    State(state): State<AppState>,
    Query(query): Query<GuestbookQuery>,
) -> Result<Json<ApiResponse<Vec<GuestbookEntry>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut condition = Condition::all().add(guestbook::Column::DeletedAt.is_null());

    if let Some(ref q) = query.search {
        if !q.trim().is_empty() {
            let like = format!("%{}%", q.trim().replace('%', "\\%").replace('_', "\\_"));
            condition = condition.add(
                Condition::any()
                    .add(Expr::col(guestbook::Column::Nickname).like(&like))
                    .add(Expr::col(guestbook::Column::Content).like(&like)),
            );
        }
    }

    let total = guestbook::Entity::find()
        .filter(condition.clone())
        .count(&state.db)
        .await?;

    let items = guestbook::Entity::find()
        .filter(condition)
        .order_by_desc(guestbook::Column::CreatedAt)
        .offset(Some((page - 1) * page_size))
        .limit(Some(page_size))
        .all(&state.db)
        .await?;

    // 批量查询关联的用户名
    let user_ids: Vec<i32> = items.iter().filter_map(|e| e.user_id).collect();
    let username_map: HashMap<i32, String> = if !user_ids.is_empty() {
        users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| (u.id, u.username))
            .collect()
    } else {
        HashMap::new()
    };

    let data: Vec<GuestbookEntry> = items
        .into_iter()
        .map(|e| model_to_entry(e, &username_map))
        .collect();
    Ok(Json(ApiResponse::with_pagination(
        data,
        Pagination::new(total, page, page_size),
    )))
}

/// PUT /api/v1/admin/guestbook/{id}/reply — 管理员回复
pub async fn reply_entry(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthUser,
    Json(req): Json<ReplyGuestbookRequest>,
) -> Result<Json<ApiResponse<GuestbookEntry>>, AppError> {
    if !auth.is_privileged() {
        return Err(AppError::Forbidden);
    }
    let entry = guestbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("留言不存在".to_string()))?;

    let mut active: guestbook::ActiveModel = entry.into();
    active.reply = Set(Some(req.reply));
    active.is_replied = Set(true);
    active.updated_at = Set(crate::utils::now_local());
    let updated = active.update(&state.db).await?;

    Ok(Json(ApiResponse::new(model_to_entry(
        updated,
        &HashMap::new(),
    ))))
}

/// DELETE /api/v1/admin/guestbook/{id} — 管理员删除（软删除）
pub async fn delete_entry(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if !auth.is_privileged() {
        return Err(AppError::Forbidden);
    }
    let entry = guestbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("留言不存在".to_string()))?;

    let mut active: guestbook::ActiveModel = entry.into();
    active.deleted_at = Set(Some(crate::utils::now_local()));
    active.update(&state.db).await?;

    Ok(Json(ApiResponse::new("删除成功".to_string())))
}
