use axum::{
    extract::{Path, Query, State},
    response::Redirect,
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::middleware::auth::PrivilegedUser;
use crate::models::entity::network_resources;
use crate::utils::{ApiResponse, AppError, AppState, Pagination};

// ── 类型 ──────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct NetworkResourceResponse {
    pub id: i32,
    pub url: String,
    pub label: Option<String>,
    pub source_type: String,
    pub referenced: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
    pub source_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRequest {
    pub url: String,
    pub label: Option<String>,
    #[serde(default = "default_source_type")]
    pub source_type: String,
}
fn default_source_type() -> String {
    "image".to_string()
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub url: Option<String>,
    pub label: Option<String>,
    pub source_type: Option<String>,
}

#[derive(Deserialize)]
pub struct EnsureRequest {
    pub url: String,
}

#[derive(Deserialize)]
pub struct BatchResolveRequest {
    pub ids: Vec<i32>,
}

fn to_response(m: network_resources::Model, referenced: bool) -> NetworkResourceResponse {
    NetworkResourceResponse {
        id: m.id,
        url: m.url,
        label: m.label,
        source_type: m.source_type,
        referenced,
        created_at: m.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: m.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}
/// GET /api/v1/network-resources — List network resources

#[utoipa::path(
    get,
    path = "/api/v1/network-resources",
    responses((status = 200, description = "成功", body = [NetworkResourceResponse])),
    tag = "Network Resources"
)]
pub async fn list_resources(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<NetworkResourceResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(200);

    let mut select = network_resources::Entity::find();

    if let Some(ref q) = query.search {
        if !q.is_empty() {
            select = select.filter(
                sea_orm::Condition::any()
                    .add(network_resources::Column::Url.contains(q))
                    .add(network_resources::Column::Label.contains(q)),
            );
        }
    }
    if let Some(ref t) = query.source_type {
        if !t.is_empty() {
            select = select.filter(network_resources::Column::SourceType.eq(t));
        }
    }

    let total = select.clone().count(&state.db).await?;
    let _pages = if page_size > 0 {
        (total + page_size - 1) / page_size
    } else {
        0
    };

    let items = select
        .order_by_desc(network_resources::Column::Id)
        .paginate(&state.db, page_size)
        .fetch_page(page - 1)
        .await?;

    let mut data: Vec<NetworkResourceResponse> = items
        .iter()
        .map(|m| NetworkResourceResponse {
            id: m.id,
            url: m.url.clone(),
            label: m.label.clone(),
            source_type: m.source_type.clone(),
            referenced: false,
            created_at: m.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: m.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    // 批量查询被引用状态
    if !data.is_empty() {
        let ids: Vec<i32> = data.iter().map(|d| d.id).collect();

        use crate::models::entity::{categories, posts};
        use std::collections::HashSet;

        // 查询被 categories 引用的 ID
        let cat_ids: HashSet<i32> = categories::Entity::find()
            .filter(categories::Column::NetworkResourceId.is_in(ids.clone()))
            .all(&state.db)
            .await?
            .into_iter()
            .filter_map(|c| c.network_resource_id)
            .collect();

        // 查询被 posts 引用的 ID
        let post_ids: HashSet<i32> = posts::Entity::find()
            .filter(posts::Column::CoverNetworkId.is_in(ids))
            .all(&state.db)
            .await?
            .into_iter()
            .filter_map(|p| p.cover_network_id)
            .collect();

        for d in &mut data {
            d.referenced = cat_ids.contains(&d.id) || post_ids.contains(&d.id);
        }
    }

    let pagination = Pagination::new(total, page, page_size);

    Ok(Json(ApiResponse::with_pagination(data, pagination)))
}
/// POST /api/v1/network-resources — Add network resource

#[utoipa::path(
    post,
    path = "/api/v1/network-resources",
    responses((status = 200, description = "成功", body = NetworkResourceResponse)),
    tag = "Network Resources"
)]
pub async fn create_resource(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Json(req): Json<CreateRequest>,
) -> Result<Json<ApiResponse<NetworkResourceResponse>>, AppError> {
    let url = strip_fragment(req.url.trim());
    if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return Err(AppError::BadRequest("请输入有效的 HTTP/HTTPS URL".into()));
    }

    // 查重
    if let Some(existing) = network_resources::Entity::find()
        .filter(network_resources::Column::Url.eq(url.clone()))
        .one(&state.db)
        .await?
    {
        return Ok(Json(ApiResponse::new(to_response(existing, false))));
    }

    let now = crate::utils::now_local();
    let resource = network_resources::ActiveModel {
        url: Set(url),
        label: Set(req.label.filter(|l| !l.trim().is_empty())),
        source_type: Set(req.source_type),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok(Json(ApiResponse::new(to_response(resource, false))))
}
/// PUT /api/v1/network-resources/{id} — Update network resource

#[utoipa::path(
    put,
    path = "/api/v1/network-resources/{id}",
    responses((status = 200, description = "成功", body = NetworkResourceResponse)),
    tag = "Network Resources"
)]
pub async fn update_resource(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateRequest>,
) -> Result<Json<ApiResponse<NetworkResourceResponse>>, AppError> {
    let resource = network_resources::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("网络资源不存在".into()))?;

    let mut active: network_resources::ActiveModel = resource.into();

    if let Some(ref url) = req.url {
        let url = url.trim();
        if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
            return Err(AppError::BadRequest("请输入有效的 HTTP/HTTPS URL".into()));
        }
        active.url = Set(url.to_string());
    }
    if let Some(ref label) = req.label {
        active.label = Set(if label.trim().is_empty() {
            None
        } else {
            Some(label.trim().to_string())
        });
    }
    if let Some(ref t) = req.source_type {
        active.source_type = Set(t.clone());
    }
    active.updated_at = Set(crate::utils::now_local());

    let updated = active.update(&state.db).await?;
    Ok(Json(ApiResponse::new(to_response(updated, false))))
}
/// DELETE /api/v1/network-resources/{id} — Delete network resource

#[utoipa::path(
    delete,
    path = "/api/v1/network-resources/{id}",
    responses((status = 200, description = "成功")),
    tag = "Network Resources"
)]
pub async fn delete_resource(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let resource = network_resources::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("网络资源不存在".into()))?;

    // 检查是否有分类引用
    use crate::models::entity::categories;
    let cat_ref = categories::Entity::find()
        .filter(categories::Column::NetworkResourceId.eq(id))
        .count(&state.db)
        .await?;
    if cat_ref > 0 {
        return Err(AppError::BadRequest(format!(
            "该资源被 {} 个分类引用，无法删除",
            cat_ref
        )));
    }

    // 检查是否有文章封面引用
    use crate::models::entity::posts;
    let post_ref = posts::Entity::find()
        .filter(posts::Column::CoverNetworkId.eq(id))
        .count(&state.db)
        .await?;
    if post_ref > 0 {
        return Err(AppError::BadRequest(format!(
            "该资源被 {} 篇文章作为封面引用，无法删除",
            post_ref
        )));
    }

    resource.delete(&state.db).await?;
    Ok(Json(ApiResponse::new("已删除".to_string())))
}

// ── GET /api/v1/network-resources/:id/references ──────────

#[derive(Serialize, ToSchema)]
pub struct ReferenceItem {
    pub target_type: String,
    pub target_id: i32,
    pub target_name: String,
    pub target_slug: String,
    pub target_description: Option<String>,
}
/// GET /api/v1/network-resources/{id}/references — List references to resource

#[utoipa::path(
    get,
    path = "/api/v1/network-resources/{id}/references",
    responses((status = 200, description = "成功")),
    tag = "Network Resources"
)]
pub async fn get_references(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<ReferenceItem>>>, AppError> {
    // 确认资源存在并获取 URL（用于后续正文扫描）
    let resource = network_resources::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("网络资源不存在".into()))?;
    let resource_url = resource.url.clone();
    let mut seen_post_ids: std::collections::HashSet<i32> = std::collections::HashSet::new();
    let mut refs: Vec<ReferenceItem> = Vec::new();

    // 查询分类引用（FK）
    use crate::models::entity::categories;
    let cats = categories::Entity::find()
        .filter(categories::Column::NetworkResourceId.eq(id))
        .all(&state.db)
        .await?;
    for c in cats {
        refs.push(ReferenceItem {
            target_type: "category".into(),
            target_id: c.id,
            target_name: c.name,
            target_slug: c.slug,
            target_description: c.description,
        });
    }

    // 查询文章封面引用（FK）
    use crate::models::entity::posts;
    let ps = posts::Entity::find()
        .filter(posts::Column::CoverNetworkId.eq(id))
        .all(&state.db)
        .await?;
    for p in ps {
        seen_post_ids.insert(p.id);
        refs.push(ReferenceItem {
            target_type: "post".into(),
            target_id: p.id,
            target_name: p.title,
            target_slug: p.slug,
            target_description: p.summary,
        });
    }

    // 查询文章正文引用（内容中包含资源 URL，不限封面 FK）
    if !resource_url.is_empty() {
        let ps_content = posts::Entity::find()
            .filter(posts::Column::Content.contains(&resource_url))
            .all(&state.db)
            .await?;
        for p in ps_content {
            if !seen_post_ids.contains(&p.id) {
                seen_post_ids.insert(p.id);
                refs.push(ReferenceItem {
                    target_type: "post".into(),
                    target_id: p.id,
                    target_name: p.title,
                    target_slug: p.slug,
                    target_description: p.summary,
                });
            }
        }
    }

    Ok(Json(ApiResponse::new(refs)))
}
/// GET /api/v1/network-resources/{id}/resolve — Resolve resource ID to URL

#[utoipa::path(
    get,
    path = "/api/v1/network-resources/{id}/resolve",
    responses((status = 200, description = "成功")),
    tag = "Network Resources"
)]
pub async fn resolve_resource(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Redirect, AppError> {
    let resource = network_resources::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("网络资源不存在".into()))?;

    Ok(Redirect::temporary(&resource.url))
}
/// POST /api/v1/network-resources/ensure — Ensure URL is registered

#[utoipa::path(
    post,
    path = "/api/v1/network-resources/ensure",
    responses((status = 200, description = "成功", body = NetworkResourceResponse)),
    tag = "Network Resources"
)]
pub async fn ensure_resource(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Json(req): Json<EnsureRequest>,
) -> Result<Json<ApiResponse<NetworkResourceResponse>>, AppError> {
    let url = req.url.trim().to_string();
    if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return Err(AppError::BadRequest("无效的 URL".into()));
    }

    // 查重
    if let Some(existing) = network_resources::Entity::find()
        .filter(network_resources::Column::Url.eq(&url))
        .one(&state.db)
        .await?
    {
        return Ok(Json(ApiResponse::new(to_response(existing, false))));
    }

    // 创建
    let now = crate::utils::now_local();
    let resource = network_resources::ActiveModel {
        url: Set(url),
        label: Set(None),
        source_type: Set("image".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok(Json(ApiResponse::new(to_response(resource, false))))
}
/// POST /api/v1/network-resources/batch-resolve — Batch resolve resource IDs

#[utoipa::path(
    post,
    path = "/api/v1/network-resources/batch-resolve",
    responses((status = 200, description = "成功")),
    tag = "Network Resources"
)]
pub async fn batch_resolve(
    State(state): State<AppState>,
    Json(req): Json<BatchResolveRequest>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, AppError> {
    if req.ids.is_empty() {
        return Ok(Json(ApiResponse::new(HashMap::new())));
    }

    let resources = network_resources::Entity::find()
        .filter(network_resources::Column::Id.is_in(req.ids))
        .all(&state.db)
        .await?;

    let map: HashMap<String, String> = resources
        .into_iter()
        .map(|r| (r.id.to_string(), r.url))
        .collect();

    Ok(Json(ApiResponse::new(map)))
}

/// 辅助函数：根据 ID 获取 URL（内部调用）
pub async fn resolve_url(db: &sea_orm::DatabaseConnection, id: Option<i32>) -> Option<String> {
    let id = id?;
    network_resources::Entity::find_by_id(id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|r| r.url)
}

/// 辅助函数：确保网络 URL 已入库，返回 resource ID（内部调用）
/// 支持三种格式：
/// - `nr:{id}` → 直接使用已有的网络资源 ID（不创建）
/// - `http://...` / `https://...` → 查重或创建新的网络资源
/// - 其他字符串（文件名等）→ 返回 None（本地文件）
/// - 包含 /uploads/ 的 URL → 返回 None（本服务器资源，非网络资源）
pub async fn ensure_url(
    db: &sea_orm::DatabaseConnection,
    url: &str,
) -> Result<Option<i32>, AppError> {
    let url = strip_fragment(url.trim());

    // nr:{id} 前缀 → 直接按 ID 查找已有资源
    if let Some(id_str) = url.strip_prefix("nr:") {
        if let Ok(id) = id_str.parse::<i32>() {
            if let Some(resource) = network_resources::Entity::find_by_id(id).one(db).await? {
                return Ok(Some(resource.id));
            }
        }
        return Ok(None);
    }

    if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return Ok(None);
    }

    // Self-hosted URL (contains /uploads/) → not a network resource
    if url.contains("/uploads/") {
        return Ok(None);
    }

    // 查重
    if let Some(existing) = network_resources::Entity::find()
        .filter(network_resources::Column::Url.eq(url.clone()))
        .one(db)
        .await?
    {
        return Ok(Some(existing.id));
    }

    // 创建
    let now = crate::utils::now_local();
    let resource = network_resources::ActiveModel {
        url: Set(url),
        label: Set(None),
        source_type: Set("image".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(Some(resource.id))
}

/// 统一封面 URL 解析
/// 优先级：network_resource_id → image_url(http) → image_filename
/// - `image_url`: 网络 URL 或本地路径（如 posts.cover_image 两种都可能）
/// - `image_filename`: 始终为本地文件名
pub async fn resolve_cover_url(
    db: &sea_orm::DatabaseConnection,
    nr_id: Option<i32>,
    image_url: Option<&str>,
    image_filename: Option<&str>,
) -> Option<String> {
    // 1. network_resource_id → 查表（外部 URL，绝对路径）
    if let Some(url) = resolve_url(db, nr_id).await {
        if !url.is_empty() {
            return Some(url);
        }
    }
    // 2. image_url — 外部 URL 直接返回，本服务器 URL 转相对路径
    if let Some(val) = image_url {
        let val = val.trim();
        if val.starts_with("http://") || val.starts_with("https://") {
            if val.contains("/uploads/") {
                // 自托管 → 提取文件名，返回相对路径
                let clean = val.split("/uploads/").last().unwrap_or("");
                if !clean.is_empty() {
                    return Some(format!("/uploads/{}", clean));
                }
            } else {
                return Some(val.to_string());
            }
        }
    }
    // 3. 本地文件 — 返回相对路径
    let filename = image_filename
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            image_url
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });
    if let Some(ref f) = filename {
        let clean = f
            .trim_start_matches("./uploads/")
            .trim_start_matches("/uploads/")
            .trim_start_matches('/');
        return Some(format!("/uploads/{}", clean));
    }
    None
}

/// 解析 cover_image 字符串中的 nr:{id} 为实际 URL（相对路径，浏览器自动补全）
pub async fn resolve_cover_image_str(
    db: &sea_orm::DatabaseConnection,
    cover_image: Option<&str>,
) -> Option<String> {
    let ci = cover_image?.trim();
    if ci.is_empty() {
        return None;
    }

    // nr:{id} → 查网络资源表获取实际 URL（外部绝对路径）
    if let Some(id_str) = ci.strip_prefix("nr:") {
        if let Ok(id) = id_str.parse::<i32>() {
            if let Ok(Some(resource)) = network_resources::Entity::find_by_id(id).one(db).await {
                return Some(resource.url);
            }
        }
        return None;
    }

    // http(s) URL → 自托管转相对路径，外部直接返回
    if ci.starts_with("http://") || ci.starts_with("https://") {
        if ci.contains("/uploads/") {
            let clean = ci.split("/uploads/").last().unwrap_or("");
            return Some(format!("/uploads/{}", clean));
        }
        return Some(ci.to_string());
    }

    // 本地文件名 → 相对路径
    Some(format!("/uploads/{}", ci.trim_start_matches('/')))
}

/// 统一解析文章封面：新字段优先，旧 cover_image 兜底
pub async fn resolve_post_cover(
    db: &sea_orm::DatabaseConnection,
    cover_network_id: Option<i32>,
    cover_image_url: Option<&str>,
    cover_image_filename: Option<&str>,
    cover_image: Option<&str>, // 🔒 历史兼容
) -> Option<String> {
    // 1. 新字段：复用 resolve_cover_url（与分类封面同一套逻辑）
    let resolved =
        resolve_cover_url(db, cover_network_id, cover_image_url, cover_image_filename).await;
    if resolved.is_some() {
        return resolved;
    }
    // 2. 旧数据 fallback
    resolve_cover_image_str(db, cover_image).await
}

/// 移除 URL 中的片段（#xxx），防止防盗链等干扰
fn strip_fragment(url: &str) -> String {
    if let Some(pos) = url.find('#') {
        url[..pos].to_string()
    } else {
        url.to_string()
    }
}

/// 将内容中的 nr:{id} 替换为真实 URL（用于展示/编辑时渲染图片）
pub async fn resolve_nr_in_content(db: &sea_orm::DatabaseConnection, content: &str) -> String {
    let md_re = regex::Regex::new(r#"!\[([^\]]*)\]\(nr:(\d+)\)"#).unwrap();
    let html_re = regex::Regex::new(r#"src=["']nr:(\d+)["']"#).unwrap();
    let mut result = content.to_string();

    // 处理 Markdown 格式: ![alt](nr:123)
    for cap in md_re.captures_iter(content) {
        let full = cap.get(0).unwrap().as_str();
        let alt = cap.get(1).unwrap().as_str();
        let id_str = cap.get(2).unwrap().as_str();
        if let Ok(id) = id_str.parse::<i32>() {
            if let Some(url) = resolve_url(db, Some(id)).await {
                result = result.replace(full, &format!("![{}]({})", alt, url));
            }
        }
    }

    // 处理 HTML 格式: <img src="nr:123">（content_html 已渲染为 HTML）
    for cap in html_re.captures_iter(content) {
        let full = cap.get(0).unwrap().as_str();
        let id_str = cap.get(1).unwrap().as_str();
        if let Ok(id) = id_str.parse::<i32>() {
            if let Some(url) = resolve_url(db, Some(id)).await {
                result = result.replace(full, &format!(r#"src="{}""#, url));
            }
        }
    }

    result
}

/// 将内容中匹配 network_resources 表的 URL 替换为 nr:{id}（保存时反向归一化）
pub async fn normalize_nr_in_content(db: &sea_orm::DatabaseConnection, content: &str) -> String {
    let re = regex::Regex::new(r#"!\[([^\]]*)\]\((https?://[^)]+)\)"#).unwrap();
    let mut result = content.to_string();
    for cap in re.captures_iter(content) {
        let full = cap.get(0).unwrap().as_str();
        let alt = cap.get(1).unwrap().as_str();
        let url = cap.get(2).unwrap().as_str();
        // 跳过本地上传 URL
        if url.contains("/uploads/") {
            continue;
        }
        // 查 network_resources 表
        if let Some(nr) = crate::models::entity::network_resources::Entity::find()
            .filter(crate::models::entity::network_resources::Column::Url.eq(url))
            .one(db)
            .await
            .ok()
            .flatten()
        {
            result = result.replace(full, &format!("![{}](nr:{})", alt, nr.id));
        }
    }
    result
}
