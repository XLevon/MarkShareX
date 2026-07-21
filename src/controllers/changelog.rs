use crate::middleware::auth::PrivilegedUser;
use crate::models::entity::changelog;
use crate::utils::{ApiResponse, AppError, AppState, Pagination};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── Response ──

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ChangelogResponse {
    pub id: i32,
    pub version: String,
    pub content: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<changelog::Model> for ChangelogResponse {
    fn from(m: changelog::Model) -> Self {
        Self {
            id: m.id,
            version: m.version,
            content: m.content,
            status: m.status.clone(),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// ── Request ──

#[derive(Deserialize)]
pub struct ChangelogQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Deserialize)]
pub struct CreateChangelogRequest {
    pub version: Option<String>,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateChangelogRequest {
    pub version: Option<String>,
    pub content: Option<String>,
}

// ── Admin ──

/// GET /api/v1/changelogs — List all changelog entries (admin, includes drafts)
#[utoipa::path(
    get,
    path = "/api/v1/changelogs",
    responses((status = 200, description = "成功", body = [ChangelogResponse])),
    tag = "Changelog"
)]
pub async fn list_changelogs(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Query(query): Query<ChangelogQuery>,
) -> Result<Json<ApiResponse<Vec<ChangelogResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let paginator = changelog::Entity::find()
        .order_by_desc(changelog::Column::Version)
        .paginate(&state.db, page_size);

    let total = paginator.num_items().await?;
    let mut items: Vec<ChangelogResponse> = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(ChangelogResponse::from)
        .collect();

    // 草稿（version 为空）永远排在最上面
    if let Some(pos) = items.iter().position(|e| e.version.is_empty()) {
        let draft = items.remove(pos);
        items.insert(0, draft);
    }

    let pages = if page_size > 0 {
        (total + page_size - 1) / page_size
    } else {
        0
    };

    Ok(Json(ApiResponse {
        data: items,
        pagination: Some(Pagination {
            page,
            page_size,
            total,
            pages,
        }),
    }))
}

/// POST /api/v1/changelogs — Create changelog entry
/// - 版本号为空 → 草稿（仅一份，重复创建会覆盖旧草稿）
/// - 版本号非空 → 直接发布
#[utoipa::path(
    post,
    path = "/api/v1/changelogs",
    responses((status = 200, description = "成功", body = ChangelogResponse)),
    tag = "Changelog"
)]
pub async fn create_changelog(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Json(req): Json<CreateChangelogRequest>,
) -> Result<Json<ApiResponse<ChangelogResponse>>, AppError> {
    let version = req.version.as_deref().unwrap_or("").trim().to_string();
    let now = crate::utils::now_local();

    if version.is_empty() {
        // 草稿：先删旧草稿再建新的（version UNIQUE 保证只有一份）
        let existing = changelog::Entity::find()
            .filter(changelog::Column::Version.eq(""))
            .one(&state.db)
            .await?;
        if let Some(old) = existing {
            changelog::Entity::delete_by_id(old.id)
                .exec(&state.db)
                .await?;
        }

        let model = changelog::ActiveModel {
            version: Set("".to_string()),
            content: Set(req.content),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        let inserted = model.insert(&state.db).await?;
        return Ok(Json(ApiResponse {
            data: ChangelogResponse::from(inserted),
            pagination: None,
        }));
    }

    // 正式发布：查重
    let existing = changelog::Entity::find()
        .filter(changelog::Column::Version.eq(&version))
        .one(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest("该版本号已存在".into()));
    }

    let model = changelog::ActiveModel {
        version: Set(version),
        content: Set(req.content),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse {
        data: ChangelogResponse::from(inserted),
        pagination: None,
    }))
}

/// PUT /api/v1/changelogs/{id} — Update changelog entry
/// - 版本号从空变为非空 → 先删旧草稿再更新（避免 UNIQUE 冲突）
#[utoipa::path(
    put,
    path = "/api/v1/changelogs/{id}",
    responses((status = 200, description = "成功", body = ChangelogResponse)),
    tag = "Changelog"
)]
pub async fn update_changelog(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateChangelogRequest>,
) -> Result<Json<ApiResponse<ChangelogResponse>>, AppError> {
    let entry = changelog::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("版本记录不存在".into()))?;

    let mut model: changelog::ActiveModel = entry.into();

    if let Some(version) = req.version {
        let v = version.trim().to_string();
        let old_is_draft = model.version.clone().unwrap().is_empty();

        if old_is_draft && !v.is_empty() {
            // 草稿发布：先删掉旧的 version='' 行，避免 UNIQUE 冲突
            // Actually the update will work because we're changing version from '' to v
            // But we need to ensure no duplicate - check first
            let dup = changelog::Entity::find()
                .filter(changelog::Column::Version.eq(&v))
                .one(&state.db)
                .await?;
            if dup.is_some() {
                return Err(AppError::BadRequest("该版本号已存在".into()));
            }
        }
        model.version = Set(v);
    }
    if let Some(content) = req.content {
        model.content = Set(content);
    }
    model.updated_at = Set(crate::utils::now_local());

    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse {
        data: ChangelogResponse::from(updated),
        pagination: None,
    }))
}

/// DELETE /api/v1/changelogs/{id} — Delete changelog entry
#[utoipa::path(
    delete,
    path = "/api/v1/changelogs/{id}",
    responses((status = 200, description = "成功")),
    tag = "Changelog"
)]
pub async fn delete_changelog(
    State(state): State<AppState>,
    _auth: PrivilegedUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    changelog::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse {
        data: (),
        pagination: None,
    }))
}

// ── Public ──

/// GET /api/v1/changelogs/latest — Get latest published version (public)
#[utoipa::path(
    get,
    path = "/api/v1/changelogs/latest",
    responses((status = 200, description = "成功", body = ChangelogResponse)),
    tag = "Changelog"
)]
pub async fn get_latest_version(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Option<ChangelogResponse>>>, AppError> {
    let latest = changelog::Entity::find()
        .filter(changelog::Column::Version.ne(""))
        .order_by_desc(changelog::Column::CreatedAt)
        .one(&state.db)
        .await?
        .map(ChangelogResponse::from);

    Ok(Json(ApiResponse {
        data: latest,
        pagination: None,
    }))
}

/// GET /api/v1/changelogs/public — List published changelogs (public, excludes drafts)
#[utoipa::path(
    get,
    path = "/api/v1/changelogs/public",
    responses((status = 200, description = "成功", body = [ChangelogResponse])),
    tag = "Changelog"
)]
pub async fn list_public_changelogs(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ChangelogResponse>>>, AppError> {
    let items: Vec<ChangelogResponse> = changelog::Entity::find()
        .filter(changelog::Column::Version.ne(""))
        .order_by_desc(changelog::Column::CreatedAt)
        .all(&state.db)
        .await?
        .into_iter()
        .map(ChangelogResponse::from)
        .collect();

    Ok(Json(ApiResponse {
        data: items,
        pagination: None,
    }))
}
