use crate::middleware::auth::AuthUser;
use crate::models::entity::categories;
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub image_filename: Option<String>,
    pub is_visible: bool,
    pub parent_id: Option<i32>,
    pub sort_order: i32,
    pub user_id: Option<i32>,
    pub post_count: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<categories::Model> for CategoryResponse {
    fn from(c: categories::Model) -> Self {
        Self {
            id: c.id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            image_url: c.image_url,
            image_filename: c.image_filename,
            is_visible: c.is_visible,
            parent_id: c.parent_id,
            sort_order: c.sort_order,
            user_id: c.user_id,
            post_count: None,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub image_filename: Option<String>,
    pub parent_id: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::double_option::deserialize"
    )]
    pub image_url: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::double_option::deserialize"
    )]
    pub image_filename: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::double_option::deserialize"
    )]
    pub parent_id: Option<Option<i32>>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<bool>,
}
/// GET /api/v1/admin/categories — List ALL categories (admin, no visibility filter)
#[utoipa::path(
    get,
    path = "/api/v1/admin/categories",
    responses((status = 200, description = "成功", body = [CategoryResponse])),
    tag = "Categories"
)]
pub async fn list_admin_categories(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, AppError> {
    let items = categories::Entity::find()
        .filter(categories::Column::DeletedAt.is_null())
        .order_by_asc(categories::Column::SortOrder)
        .all(&state.db)
        .await?;

    // Batch-count posts per category (only non-deleted posts)
    let cat_ids: Vec<i32> = items.iter().map(|c| c.id).collect();
    let post_counts = if !cat_ids.is_empty() {
        use crate::models::entity::posts;
        let raw_sql = format!(
            "SELECT category_id, COUNT(*) as cnt FROM {} WHERE category_id IN ({}) AND deleted_at IS NULL AND status = 'published' GROUP BY category_id",
            posts::Entity.table_name(),
            cat_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            state.db.get_database_backend(),
            &raw_sql,
            cat_ids
                .iter()
                .map(|&id| id.into())
                .collect::<Vec<sea_orm::Value>>(),
        );
        let results = state.db.query_all(stmt).await?;
        let mut map = std::collections::HashMap::new();
        for row in results {
            let cat_id: i32 = row.try_get_by_index::<i32>(0)?;
            let cnt: i64 = row.try_get_by_index::<i64>(1)?;
            map.insert(cat_id, cnt);
        }
        map
    } else {
        std::collections::HashMap::new()
    };

    let mut data: Vec<CategoryResponse> = Vec::new();
    for item in items {
        let post_count = post_counts.get(&item.id).copied();
        let cover_url = resolve_category_cover(&state.db, &item).await;
        let mut resp = CategoryResponse::from(item);
        resp.post_count = post_count;
        resp.image_url = cover_url;
        data.push(resp);
    }
    Ok(Json(ApiResponse::new(data)))
}

/// GET /api/v1/categories — List categories (public, excludes hidden)

#[utoipa::path(
    get,
    path = "/api/v1/categories",
    responses((status = 200, description = "成功", body = [CategoryResponse])),
    tag = "Categories"
)]
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, AppError> {
    // 公开接口：排除隐藏分类 + 隐藏父类的子分类
    let hidden_ids = get_hidden_category_ids(&state.db).await?;

    let mut query = categories::Entity::find().filter(categories::Column::DeletedAt.is_null());
    if !hidden_ids.is_empty() {
        query = query.filter(categories::Column::Id.is_not_in(hidden_ids));
    }
    let items = query
        .order_by_asc(categories::Column::SortOrder)
        .all(&state.db)
        .await?;

    // Batch-count posts per category (only non-deleted posts)
    let cat_ids: Vec<i32> = items.iter().map(|c| c.id).collect();
    let post_counts = if !cat_ids.is_empty() {
        use crate::models::entity::posts;
        let raw_sql = format!(
            "SELECT category_id, COUNT(*) as cnt FROM {} WHERE category_id IN ({}) AND deleted_at IS NULL AND status = 'published' GROUP BY category_id",
            posts::Entity.table_name(),
            cat_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            state.db.get_database_backend(),
            &raw_sql,
            cat_ids
                .iter()
                .map(|&id| id.into())
                .collect::<Vec<sea_orm::Value>>(),
        );
        let results = state.db.query_all(stmt).await?;
        let mut map = std::collections::HashMap::new();
        for row in results {
            let cat_id: i32 = row.try_get_by_index::<i32>(0)?;
            let cnt: i64 = row.try_get_by_index::<i64>(1)?;
            map.insert(cat_id, cnt);
        }
        map
    } else {
        std::collections::HashMap::new()
    };

    let mut data: Vec<CategoryResponse> = Vec::new();
    for item in items {
        let post_count = post_counts.get(&item.id).copied();
        // 先解析封面（避免 move 冲突）
        let cover_url = resolve_category_cover(&state.db, &item).await;
        let mut resp = CategoryResponse::from(item);
        resp.post_count = post_count;
        resp.image_url = cover_url;

        data.push(resp);
    }
    Ok(Json(ApiResponse::new(data)))
}

fn can_modify_category(auth: &AuthUser, category: &categories::Model) -> bool {
    auth.is_privileged() || category.user_id == Some(auth.user_id)
}
/// POST /api/v1/categories — Create category

#[utoipa::path(
    post,
    path = "/api/v1/categories",
    responses((status = 200, description = "成功", body = CategoryResponse)),
    tag = "Categories"
)]
pub async fn create_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<ApiResponse<CategoryResponse>>, AppError> {
    let mut slug = crate::services::posts::generate_slug(&req.name);
    let now = crate::utils::now_local();

    // 检查 slug 唯一性（排除软删除记录），冲突时追加随机后缀
    let existing = categories::Entity::find()
        .filter(categories::Column::Slug.eq(&slug))
        .filter(categories::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?;
    if existing.is_some() {
        slug = format!("{}-{}", slug, &uuid::Uuid::new_v4().to_string()[..8]);
    }

    // 封面三种来源：本地文件 | nr:{id} 网络资源 | 普通外链（不入库）
    let network_resource_id = if let Some(ref url) = req.image_url {
        if url.starts_with("nr:") {
            super::network_resources::ensure_url(&state.db, url).await?
        } else {
            None // 普通外链或本地文件
        }
    } else {
        None
    };

    let model = categories::ActiveModel {
        name: Set(req.name),
        slug: Set(slug),
        description: Set(req.description),
        image_url: Set(req.image_url),
        image_filename: Set(req.image_filename),
        network_resource_id: Set(network_resource_id),
        parent_id: Set(req.parent_id),
        sort_order: Set(req.sort_order.unwrap_or(0)),
        is_visible: Set(req.is_visible.unwrap_or(true)),
        user_id: Set(Some(auth.user_id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let result = model.insert(&state.db).await?;

    // 处理图片 URL：network_resource_id > 网络URL > 资源库文件名
    let category_cover = resolve_category_cover(&state.db, &result).await;
    let mut resp = CategoryResponse::from(result);
    resp.image_url = category_cover;

    Ok(Json(ApiResponse::new(resp)))
}
/// PUT /api/v1/categories/{id} — Update category

#[utoipa::path(
    put,
    path = "/api/v1/categories/{id}",
    responses((status = 200, description = "成功", body = CategoryResponse)),
    tag = "Categories"
)]
pub async fn update_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<ApiResponse<CategoryResponse>>, AppError> {
    let category = categories::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("分类不存在".to_string()))?;

    if category.deleted_at.is_some() {
        return Err(AppError::NotFound("分类不存在".to_string()));
    }

    if !can_modify_category(&auth, &category) {
        return Err(AppError::Forbidden);
    }

    let mut active: categories::ActiveModel = category.into();

    if let Some(name) = req.name {
        active.name = Set(name.clone());
        active.slug = Set(crate::services::posts::generate_slug(&name));
    }
    if let Some(description) = req.description {
        active.description = Set(Some(description));
    }
    if let Some(image_url) = req.image_url {
        if image_url.as_ref().map_or(true, |s| s.is_empty()) {
            active.image_url = Set(None);
            active.network_resource_id = Set(None);
        } else {
            let nr_id = if image_url.as_ref().map_or(false, |s| s.starts_with("nr:")) {
                super::network_resources::ensure_url(&state.db, &image_url.as_ref().unwrap())
                    .await?
            } else {
                None
            };
            active.image_url = Set(image_url);
            active.network_resource_id = Set(nr_id);
        }
    }
    if let Some(image_filename) = req.image_filename {
        if image_filename.as_ref().map_or(true, |s| s.is_empty()) {
            active.image_filename = Set(None);
        } else {
            active.image_filename = Set(image_filename);
        }
    }
    if let Some(parent_id) = req.parent_id {
        active.parent_id = Set(parent_id);
    }
    if let Some(sort_order) = req.sort_order {
        active.sort_order = Set(sort_order);
    }
    if let Some(is_visible) = req.is_visible {
        active.is_visible = Set(is_visible);
    }

    active.updated_at = Set(crate::utils::now_local());
    let updated = active.update(&state.db).await?;

    // 处理图片 URL：network_resource_id > 网络URL > 资源库文件名
    let category_cover = resolve_category_cover(&state.db, &updated).await;
    let mut resp = CategoryResponse::from(updated);
    resp.image_url = category_cover;

    Ok(Json(ApiResponse::new(resp)))
}
/// PUT /api/v1/admin/categories/reorder — 拖动排序
#[derive(Deserialize, ToSchema)]
pub struct ReorderRequest {
    pub ids: Vec<i32>,
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/categories/reorder",
    request_body = ReorderRequest,
    responses((status = 200, description = "排序成功")),
    tag = "Categories"
)]
pub async fn reorder_categories(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<ReorderRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    for (index, id) in req.ids.iter().enumerate() {
        let cat = categories::Entity::find_by_id(*id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound("分类不存在".to_string()))?;
        let mut active: categories::ActiveModel = cat.into();
        active.sort_order = Set(index as i32);
        active.update(&state.db).await?;
    }
    Ok(Json(ApiResponse::new(())))
}

/// DELETE /api/v1/categories/{id} — Delete category

#[utoipa::path(
    delete,
    path = "/api/v1/categories/{id}",
    responses((status = 200, description = "成功")),
    tag = "Categories"
)]
pub async fn delete_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let category = categories::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("分类不存在".to_string()))?;

    if !can_modify_category(&auth, &category) {
        return Err(AppError::Forbidden);
    }

    // 检查是否有文章关联
    let post_count: i64 = state
        .db
        .query_one(sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!(
                "SELECT COUNT(*) FROM posts WHERE category_id = {} AND deleted_at IS NULL",
                id
            ),
        ))
        .await?
        .and_then(|r| r.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);

    if post_count > 0 {
        return Err(AppError::BadRequest(format!(
            "该分类下有 {} 篇文章，无法删除。请先清空或移动文章",
            post_count
        )));
    }

    // Unlink child categories (set parent_id to NULL) then hard delete
    let stmt = sea_orm::Statement::from_sql_and_values(
        state.db.get_database_backend(),
        "UPDATE categories SET parent_id = NULL WHERE parent_id = $1",
        vec![id.into()],
    );
    state.db.execute(stmt).await?;
    categories::Entity::delete_by_id(id).exec(&state.db).await?;

    Ok(Json(ApiResponse::new(())))
}

/// 获取前台不可见的分类 ID 集合（含隐藏分类 + 隐藏父类的子分类）
pub async fn get_hidden_category_ids(
    db: &sea_orm::DatabaseConnection,
) -> Result<Vec<i32>, AppError> {
    let all = categories::Entity::find()
        .filter(categories::Column::DeletedAt.is_null())
        .all(db)
        .await?;

    // 收集直接设为隐藏的分类 ID
    let mut hidden: std::collections::HashSet<i32> =
        all.iter().filter(|c| !c.is_visible).map(|c| c.id).collect();

    // 隐藏父类的子分类也视为隐藏（递归处理多层级）
    loop {
        let before = hidden.len();
        for cat in &all {
            if !hidden.contains(&cat.id) {
                if let Some(pid) = cat.parent_id {
                    if hidden.contains(&pid) {
                        hidden.insert(cat.id);
                    }
                }
            }
        }
        if hidden.len() == before {
            break; // 没有新增，停止
        }
    }

    Ok(hidden.into_iter().collect())
}

/// 解析分类封面 URL（委托统一函数）
async fn resolve_category_cover(
    db: &sea_orm::DatabaseConnection,
    cat: &categories::Model,
) -> Option<String> {
    super::network_resources::resolve_cover_url(
        db,
        cat.network_resource_id,
        cat.image_url.as_deref(),
        cat.image_filename.as_deref(),
    )
    .await
}
