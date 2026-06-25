use axum::{
    extract::{State, Path, Query},
    Json,
};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::utils::{AppState, AppError, ApiResponse, Pagination};
use crate::models::entity::{users, author_applications, posts, files, comments, login_logs, read_logs};
use crate::middleware::auth::AuthUser;
use crate::services;
use sea_orm::*;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct ApplicationInfo {
    pub id: i32,
    pub reason: String,
    pub content_description: String,
    pub status: String,
    pub admin_remark: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub status: String,
    pub is_active: bool,
    pub last_login_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub application: Option<ApplicationInfo>,
}

impl From<users::Model> for AdminUserResponse {
    fn from(u: users::Model) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            display_name: u.display_name,
            role: u.role,
            status: u.status,
            is_active: u.is_active,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            application: None,
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateUserStatusRequest {
    pub status: String,
}
/// GET /api/v1/admin/users — List users

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    responses((status = 200, description = "成功", body = [AdminUserResponse])),
    tag = "Admin"
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<PaginatedUsers>>, AppError> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let is_pending_apply = query.status.as_deref() == Some("pending_apply");

    // When filtering by pending applications, first get user IDs with pending apps
    let mut pending_ids: Vec<i32> = Vec::new();
    let mut app_map: HashMap<i32, author_applications::Model> = HashMap::new();

    if is_pending_apply {
        let apps = author_applications::Entity::find()
            .filter(author_applications::Column::Status.eq("pending"))
            .all(&state.db)
            .await?;
        pending_ids = apps.iter().map(|a| a.user_id).collect();
        for app in apps {
            app_map.insert(app.user_id, app);
        }
        if pending_ids.is_empty() {
            return Ok(Json(ApiResponse::new(PaginatedUsers {
                data: vec![],
                pagination: Pagination { total: 0, pages: 0, page, page_size },
            })));
        }
    }

    let mut select = users::Entity::find()
        .filter(users::Column::DeletedAt.is_null());

    if is_pending_apply {
        select = select.filter(users::Column::Id.is_in(pending_ids.clone()));
    } else if let Some(ref status) = query.status {
        select = select.filter(users::Column::Status.eq(status));
    }

    if let Some(ref search) = query.search {
        let pattern = format!("%{}%", search);
        select = select.filter(
            users::Column::Username.like(&pattern)
                .or(users::Column::Email.like(&pattern))
        );
    }

    let paginator = select.paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let users_data: Vec<users::Model> = paginator.fetch_page(page - 1).await?;

    // If not pending_apply filter but we still want to show app info, load all apps for these users
    if !is_pending_apply && !users_data.is_empty() {
        let user_ids: Vec<i32> = users_data.iter().map(|u| u.id).collect();
        let apps = author_applications::Entity::find()
            .filter(author_applications::Column::UserId.is_in(user_ids))
            .all(&state.db)
            .await?;
        for app in apps {
            app_map.insert(app.user_id, app);
        }
    }

    let data: Vec<AdminUserResponse> = users_data
        .into_iter()
        .map(|u| {
            let application = app_map.get(&u.id).map(|app| ApplicationInfo {
                id: app.id,
                reason: app.reason.clone(),
                content_description: app.content_description.clone(),
                status: app.status.clone(),
                admin_remark: app.admin_remark.clone(),
                created_at: app.created_at,
            });
            let mut resp = AdminUserResponse::from(u);
            resp.application = application;
            resp
        })
        .collect();

    Ok(Json(ApiResponse::new(PaginatedUsers {
        data,
        pagination: Pagination {
            total,
            pages: ((total as f64) / (page_size as f64)).ceil() as u64,
            page,
            page_size,
        },
    })))
}
/// PUT /api/v1/admin/users/{id}/status — Update user status

#[utoipa::path(
    put,
    path = "/api/v1/admin/users/{id}/status",
    responses((status = 200, description = "成功")),
    tag = "Admin"
)]
pub async fn update_user_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateUserStatusRequest>,
) -> Result<Json<ApiResponse<AdminUserResponse>>, AppError> {
    let valid_statuses = ["active", "muted", "banned"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest(
            format!("无效状态，可选值: {}", valid_statuses.join(", "))
        ));
    }

    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    // 子管理员无权修改管理员账户
    if !auth.is_admin() && user.role == "admin" {
        return Err(AppError::Forbidden);
    }

    // Don't allow admins to ban themselves
    if user.role == "admin" && req.status == "banned" {
        return Err(AppError::BadRequest("不能拉黑管理员用户".into()));
    }

    let mut active: users::ActiveModel = user.into();
    active.status = Set(req.status.clone());
    active.is_active = Set(req.status != "banned");
    active.updated_at = Set(crate::utils::now_local());
    let updated = active.update(&state.db).await?;

    Ok(Json(ApiResponse::new(AdminUserResponse::from(updated))))
}

#[derive(Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}
/// PUT /api/v1/admin/users/{id}/role — Update user role

#[utoipa::path(
    put,
    path = "/api/v1/admin/users/{id}/role",
    responses((status = 200, description = "成功")),
    tag = "Admin"
)]
pub async fn update_user_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> Result<Json<ApiResponse<AdminUserResponse>>, AppError> {
    let valid_roles = ["admin", "sub_admin", "author", "visitor"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(AppError::BadRequest(
            format!("无效角色，可选值: {}", valid_roles.join(", "))
        ));
    }

    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    // 子管理员无权修改管理员账户
    if !auth.is_admin() && user.role == "admin" {
        return Err(AppError::Forbidden);
    }

    // 子管理员无权将用户提升为管理员
    if !auth.is_admin() && req.role == "admin" {
        return Err(AppError::Forbidden);
    }

    // 不能修改自己的角色（防止管理员误降级）
    if user.id == auth.user_id {
        return Err(AppError::BadRequest("不能修改自己的角色".into()));
    }

    let mut active: users::ActiveModel = user.into();
    active.role = Set(req.role);
    active.updated_at = Set(crate::utils::now_local());
    let updated = active.update(&state.db).await?;

    Ok(Json(ApiResponse::new(AdminUserResponse::from(updated))))
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}
/// POST /api/v1/admin/users — Create user

#[utoipa::path(
    post,
    path = "/api/v1/admin/users",
    responses((status = 200, description = "成功", body = AdminUserResponse)),
    tag = "Admin"
)]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<AdminUserResponse>>, AppError> {
    if req.username.trim().is_empty() || req.email.trim().is_empty() || req.password.len() < 8 {
        return Err(AppError::BadRequest("用户名、邮箱不能为空，密码至少8位".into()));
    }

    let role = req.role.unwrap_or_else(|| "author".to_string());
    let valid_roles = ["admin", "sub_admin", "author", "visitor"];
    if !valid_roles.contains(&role.as_str()) {
        return Err(AppError::BadRequest(
            format!("无效角色，可选值: {}", valid_roles.join(", "))
        ));
    }

    // 子管理员无权创建管理员账户
    if !auth.is_admin() && role == "admin" {
        return Err(AppError::Forbidden);
    }

    let status = req.status.unwrap_or_else(|| "active".to_string());
    let valid_statuses = ["active", "muted", "banned"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(AppError::BadRequest(
            format!("无效状态，可选值: {}", valid_statuses.join(", "))
        ));
    }

    // Check existing user
    let existing = users::Entity::find()
        .filter(
            users::Column::Username.eq(req.username.trim())
                .or(users::Column::Email.eq(req.email.trim()))
        )
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("用户名或邮箱已存在".into()));
    }

    let password_hash = services::auth::hash_password(&req.password)?;
    let now = crate::utils::now_local();

    let user_model = users::ActiveModel {
        username: Set(req.username.trim().to_string()),
        email: Set(req.email.trim().to_string()),
        password_hash: Set(password_hash),
        display_name: Set(req.display_name.filter(|n| !n.trim().is_empty())),
        role: Set(role),
        status: Set(status.clone()),
        is_active: Set(status != "banned"),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let user = user_model.insert(&state.db).await?;
    Ok(Json(ApiResponse::new(AdminUserResponse::from(user))))
}
/// PUT /api/v1/admin/users/{id} — Update user

#[utoipa::path(
    put,
    path = "/api/v1/admin/users/{id}",
    responses((status = 200, description = "成功", body = AdminUserResponse)),
    tag = "Admin"
)]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<AdminUserResponse>>, AppError> {
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    // 子管理员无权修改管理员账户
    if !auth.is_admin() && user.role == "admin" {
        return Err(AppError::Forbidden);
    }

    let mut active: users::ActiveModel = user.clone().into();
    let mut status_changed = false;
    let mut new_status = user.status.clone();

    if let Some(ref display_name) = req.display_name {
        active.display_name = Set(if display_name.trim().is_empty() { None } else { Some(display_name.trim().to_string()) });
    }

    if let Some(ref email) = req.email {
        if email.trim().is_empty() {
            return Err(AppError::BadRequest("邮箱不能为空".into()));
        }
        // Check uniqueness
        let dup = users::Entity::find()
            .filter(users::Column::Email.eq(email.trim()))
            .filter(users::Column::Id.ne(id))
            .filter(users::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?;
        if dup.is_some() {
            return Err(AppError::BadRequest("邮箱已被其他用户使用".into()));
        }
        active.email = Set(email.trim().to_string());
    }

    if let Some(ref role) = req.role {
        let valid_roles = ["admin", "sub_admin", "author", "visitor"];
        if !valid_roles.contains(&role.as_str()) {
            return Err(AppError::BadRequest(
                format!("无效角色，可选值: {}", valid_roles.join(", "))
            ));
        }
        // 子管理员无权将用户提升为管理员
        if !auth.is_admin() && role == "admin" {
            return Err(AppError::Forbidden);
        }
        // Can't change own role
        if user.id == auth.user_id {
            return Err(AppError::BadRequest("不能修改自己的角色".into()));
        }
        active.role = Set(role.clone());
    }

    if let Some(ref status) = req.status {
        let valid_statuses = ["active", "muted", "banned"];
        if !valid_statuses.contains(&status.as_str()) {
            return Err(AppError::BadRequest(
                format!("无效状态，可选值: {}", valid_statuses.join(", "))
            ));
        }
        // Don't allow banning admin
        if user.role == "admin" && status == "banned" {
            return Err(AppError::BadRequest("不能拉黑管理员用户".into()));
        }
        new_status = status.clone();
        status_changed = true;
    }

    if status_changed {
        active.status = Set(new_status.clone());
        active.is_active = Set(new_status != "banned");
    }

    active.updated_at = Set(crate::utils::now_local());
    let updated = active.update(&state.db).await?;

    Ok(Json(ApiResponse::new(AdminUserResponse::from(updated))))
}
/// DELETE /api/v1/admin/users/{id} — Soft-delete user

#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{id}",
    responses((status = 200, description = "成功")),
    tag = "Admin"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    // 权限检查：只有管理员和子管理员可以删除用户
    if !auth.is_privileged() {
        return Err(AppError::Forbidden);
    }

    // 防止删除自己
    if id == auth.user_id {
        return Err(AppError::BadRequest("不能删除自己的账号".into()));
    }

    let user = users::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    if user.role == "admin" {
        return Err(AppError::BadRequest("不能删除管理员用户".into()));
    }

    let db = &state.db;

    // 1. 删除该用户的文章及其关联数据
    let user_posts: Vec<i32> = posts::Entity::find()
        .filter(posts::Column::UserId.eq(id))
        .all(db)
        .await?
        .iter()
        .map(|p| p.id)
        .collect();

    for post_id in &user_posts {
        db.execute_unprepared(&format!("DELETE FROM post_tags WHERE post_id = {}", post_id)).await.ok();
        db.execute_unprepared(&format!("DELETE FROM comments WHERE post_id = {}", post_id)).await.ok();
        db.execute_unprepared(&format!("DELETE FROM likes WHERE post_id = {}", post_id)).await.ok();
        db.execute_unprepared(&format!("DELETE FROM read_logs WHERE post_id = {}", post_id)).await.ok();
    }

    posts::Entity::delete_many()
        .filter(posts::Column::UserId.eq(id))
        .exec(db)
        .await?;

    // 2. 删除用户的文件
    files::Entity::delete_many()
        .filter(files::Column::UserId.eq(id))
        .exec(db)
        .await?;

    // 3. 删除用户的评论
    comments::Entity::delete_many()
        .filter(comments::Column::UserId.eq(id))
        .exec(db)
        .await?;

    // 4. 删除用户点赞记录
    db.execute_unprepared(&format!("DELETE FROM likes WHERE user_id = {}", id)).await.ok();

    // 5. 删除用户作者申请
    author_applications::Entity::delete_many()
        .filter(author_applications::Column::UserId.eq(id))
        .exec(db)
        .await?;

    // 5. 删除用户阅读日志、登录日志、刷新令牌
    db.execute_unprepared(&format!("DELETE FROM read_logs WHERE user_id = {}", id)).await.ok();
    db.execute_unprepared(&format!("DELETE FROM login_logs WHERE user_id = {}", id)).await.ok();
    db.execute_unprepared(&format!("DELETE FROM refresh_tokens WHERE user_id = {}", id)).await.ok();
    db.execute_unprepared(&format!("DELETE FROM tags WHERE user_id = {}", id)).await.ok();

    // 5.1 删除用户创建的分类（仅个人分类，系统分类 user_id 为 NULL 不受影响）
    db.execute_unprepared(&format!("DELETE FROM categories WHERE user_id = {}", id)).await.ok();

    // 6. 物理删除用户
    users::Entity::delete_by_id(id).exec(db).await?;

    Ok(Json(ApiResponse::new("用户已删除".to_string())))
}

// ── 登录日志 ──

#[derive(Deserialize)]
pub struct LoginLogsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub user_id: Option<i32>,
    pub success: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct LoginLogResponse {
    pub id: i32,
    pub user_id: Option<i32>,
    pub username: String,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub login_method: String,
    pub success: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedLoginLogs {
    pub data: Vec<LoginLogResponse>,
    pub pagination: Pagination,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/login-logs",
    tag = "Admin",
    responses((status = 200, description = "登录日志列表")),
)]
/// GET /api/v1/admin/login-logs
pub async fn list_login_logs(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<LoginLogsQuery>,
) -> Result<Json<ApiResponse<PaginatedLoginLogs>>, AppError> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut select = login_logs::Entity::find();
    if let Some(uid) = query.user_id {
        select = select.filter(login_logs::Column::UserId.eq(uid));
    }
    if let Some(success) = query.success {
        select = select.filter(login_logs::Column::Success.eq(success));
    }
    select = select.order_by_desc(login_logs::Column::CreatedAt);

    let total = select.clone().count(&state.db).await?;
    let pages = ((total as f64) / (page_size as f64)).ceil() as u64;
    let items = select
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size)
        .all(&state.db)
        .await?;

    let data: Vec<LoginLogResponse> = items.into_iter().map(|l| LoginLogResponse {
        id: l.id,
        user_id: l.user_id,
        username: l.username,
        ip_address: l.ip_address,
        device_type: l.device_type,
        login_method: l.login_method,
        success: l.success,
        created_at: l.created_at,
    }).collect();

    Ok(Json(ApiResponse::new(PaginatedLoginLogs {
        data,
        pagination: Pagination { total, pages, page, page_size },
    })))
}

// ── 阅读日志 ──

#[derive(Deserialize)]
pub struct ReadLogsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub post_id: Option<i32>,
    pub user_id: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct ReadLogResponse {
    pub id: i32,
    pub post_id: i32,
    pub post_title: Option<String>,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub referrer: Option<String>,
    pub duration_seconds: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedReadLogs {
    pub data: Vec<ReadLogResponse>,
    pub pagination: Pagination,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/read-logs",
    tag = "Admin",
    responses((status = 200, description = "阅读日志列表")),
)]
/// GET /api/v1/admin/read-logs
pub async fn list_read_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ReadLogsQuery>,
) -> Result<Json<ApiResponse<PaginatedReadLogs>>, AppError> {
    if !auth.is_privileged() {
        return Err(AppError::Forbidden);
    }
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut select = read_logs::Entity::find();
    if let Some(pid) = query.post_id {
        select = select.filter(read_logs::Column::PostId.eq(pid));
    }
    if let Some(uid) = query.user_id {
        select = select.filter(read_logs::Column::UserId.eq(uid));
    }
    select = select.order_by_desc(read_logs::Column::CreatedAt);

    let total = select.clone().count(&state.db).await?;
    let pages = ((total as f64) / (page_size as f64)).ceil() as u64;
    let items = select
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size)
        .all(&state.db)
        .await?;

    // batch-fetch post titles
    let post_ids: Vec<i32> = items.iter().map(|r| r.post_id).collect();
    let titles = if !post_ids.is_empty() {
        let posts_list = posts::Entity::find()
            .filter(posts::Column::Id.is_in(post_ids.clone()))
            .all(&state.db)
            .await?;
        posts_list.into_iter().map(|p| (p.id, p.title)).collect::<std::collections::HashMap<_, _>>()
    } else {
        std::collections::HashMap::new()
    };

    // batch-fetch usernames
    let user_ids: Vec<i32> = items.iter().filter_map(|r| r.user_id).collect();
    let usernames = if !user_ids.is_empty() {
        let users_list = users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids.clone()))
            .all(&state.db)
            .await?;
        users_list.into_iter().map(|u| (u.id, u.username)).collect::<std::collections::HashMap<_, _>>()
    } else {
        std::collections::HashMap::new()
    };

    let data: Vec<ReadLogResponse> = items.into_iter().map(|r| {
        ReadLogResponse {
            id: r.id,
            post_id: r.post_id,
            post_title: titles.get(&r.post_id).cloned(),
            user_id: r.user_id,
            username: r.user_id.and_then(|uid| usernames.get(&uid).cloned()),
            ip_address: r.ip_address,
            device_type: r.device_type,
            referrer: r.referrer,
            duration_seconds: r.duration_seconds,
            created_at: r.created_at,
        }
    }).collect();

    Ok(Json(ApiResponse::new(PaginatedReadLogs {
        data,
        pagination: Pagination { total, pages, page, page_size },
    })))
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedUsers {
    pub data: Vec<AdminUserResponse>,
    pub pagination: Pagination,
}
