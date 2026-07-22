use crate::middleware::auth::AuthUser;
use crate::models::entity::tags;
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub user_id: Option<i32>,
    pub post_count: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<tags::Model> for TagResponse {
    fn from(t: tags::Model) -> Self {
        Self {
            id: t.id,
            name: t.name,
            slug: t.slug,
            user_id: t.user_id,
            post_count: None,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

fn can_modify_tag(auth: &AuthUser, tag: &tags::Model) -> bool {
    auth.is_privileged() || tag.user_id == Some(auth.user_id)
}
/// GET /api/v1/tags — List tags

#[utoipa::path(
    get,
    path = "/api/v1/tags",
    responses((status = 200, description = "成功", body = [TagResponse])),
    tag = "Tags"
)]
pub async fn list_tags(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<TagResponse>>>, AppError> {
    let items = tags::Entity::find()
        .filter(tags::Column::DeletedAt.is_null())
        .order_by_asc(tags::Column::Name)
        .all(&state.db)
        .await?;

    // Batch-count posts per tag using a single GROUP BY query
    let tag_ids: Vec<i32> = items.iter().map(|t| t.id).collect();
    let post_counts = if !tag_ids.is_empty() {
        use crate::models::entity::posts;
        let raw_sql = format!(
            "SELECT pt.tag_id, COUNT(*) as cnt FROM post_tags pt JOIN {} p ON pt.post_id = p.id WHERE pt.tag_id IN ({}) AND p.deleted_at IS NULL AND p.status = 'published' GROUP BY pt.tag_id",
            posts::Entity.table_name(),
            tag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            state.db.get_database_backend(),
            &raw_sql,
            tag_ids
                .iter()
                .map(|&id| id.into())
                .collect::<Vec<sea_orm::Value>>(),
        );
        let results = state.db.query_all(stmt).await?;
        let mut map = std::collections::HashMap::new();
        for row in results {
            let tag_id: i32 = row.try_get_by_index::<i32>(0)?;
            let cnt: i64 = row.try_get_by_index::<i64>(1)?;
            map.insert(tag_id, cnt);
        }
        map
    } else {
        std::collections::HashMap::new()
    };

    let mut data: Vec<TagResponse> = Vec::new();
    for item in items {
        let post_count = post_counts.get(&item.id).copied();
        let mut resp = TagResponse::from(item);
        resp.post_count = post_count;
        data.push(resp);
    }
    Ok(Json(ApiResponse::new(data)))
}
/// POST /api/v1/tags — Create tag

#[utoipa::path(
    post,
    path = "/api/v1/tags",
    responses((status = 200, description = "成功", body = TagResponse)),
    tag = "Tags"
)]
pub async fn create_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<ApiResponse<TagResponse>>, AppError> {
    let mut slug = crate::services::posts::generate_slug(&req.name);
    let now = crate::utils::now_local();

    // 检查 slug 唯一性（排除软删除记录），冲突时追加随机后缀
    let existing = tags::Entity::find()
        .filter(tags::Column::Slug.eq(&slug))
        .filter(tags::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?;
    if existing.is_some() {
        slug = format!("{}-{}", slug, &uuid::Uuid::new_v4().to_string()[..8]);
    }

    let model = tags::ActiveModel {
        name: Set(req.name),
        slug: Set(slug),
        user_id: Set(Some(auth.user_id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let result = model.insert(&state.db).await?;
    Ok(Json(ApiResponse::new(TagResponse::from(result))))
}
/// PUT /api/v1/tags/{id} — Update tag

#[utoipa::path(
    put,
    path = "/api/v1/tags/{id}",
    responses((status = 200, description = "成功", body = TagResponse)),
    tag = "Tags"
)]
pub async fn update_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<ApiResponse<TagResponse>>, AppError> {
    let tag = tags::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("标签不存在".to_string()))?;

    if !can_modify_tag(&auth, &tag) {
        return Err(AppError::Forbidden);
    }

    let mut active: tags::ActiveModel = tag.into();
    let name = req.name;
    active.slug = Set(crate::services::posts::generate_slug(&name));
    active.name = Set(name);
    active.updated_at = Set(crate::utils::now_local());

    let updated = active.update(&state.db).await?;
    Ok(Json(ApiResponse::new(TagResponse::from(updated))))
}
/// DELETE /api/v1/tags/{id} — Delete tag

#[utoipa::path(
    delete,
    path = "/api/v1/tags/{id}",
    responses((status = 200, description = "成功")),
    tag = "Tags"
)]
pub async fn delete_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let tag = tags::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("标签不存在".to_string()))?;

    if !can_modify_tag(&auth, &tag) {
        return Err(AppError::Forbidden);
    }

    let txn = state.db.begin().await?;
    let reference_count = crate::models::entity::post_tags::Entity::find()
        .filter(crate::models::entity::post_tags::Column::TagId.eq(id))
        .count(&txn)
        .await?;
    if reference_count > 0 {
        return Err(AppError::BadRequest(format!(
            "该标签已被 {} 篇文章使用，无法删除。请先移除文章关联",
            reference_count
        )));
    }

    // 未被任何文章引用的结构元数据采用硬删除，避免 slug 唯一约束冲突。
    tags::Entity::delete_by_id(id).exec(&txn).await?;
    txn.commit().await?;

    Ok(Json(ApiResponse::new(())))
}
