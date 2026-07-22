use crate::middleware::auth::AuthUser;
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{
    extract::{Json, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

// ============== 导出功能 ==============

#[derive(Deserialize)]
pub struct ExportRequest {
    pub post_ids: Option<Vec<i32>>,
}

/// POST /api/v1/export/posts — Export posts as ZIP

#[utoipa::path(
    post,
    path = "/api/v1/export/posts",
    responses((status = 200, description = "成功")),
    tag = "Import/Export"
)]
pub async fn export_posts(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let archive = crate::services::import_export::export_posts_archive(
        &state,
        auth.user_id,
        auth.is_privileged(),
        req.post_ids,
    )
    .await?;
    let timestamp = archive.generated_at.format("%Y%m%d_%H%M%S").to_string();
    let file_data = archive.bytes;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/zip".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!(
            "attachment; filename=\"marksharex_export_{}.zip\"",
            timestamp
        )
        .parse()
        .unwrap(),
    );
    headers.insert(
        "Content-Length",
        file_data.len().to_string().parse().unwrap(),
    );

    Ok((headers, file_data))
}

// ============== 导入功能 ==============

#[derive(Deserialize)]
pub struct ImportRequest {
    pub items: Vec<ImportItem>,
}

#[derive(Deserialize)]
pub struct ImportImage {
    pub data: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ImportItem {
    pub filename: String,
    pub content: String,
    pub images: Vec<ImportImage>,
}

impl From<ImportItem> for crate::services::import_export::ImportItemInput {
    fn from(item: ImportItem) -> Self {
        Self {
            filename: item.filename,
            content: item.content,
            images: item
                .images
                .into_iter()
                .map(|image| crate::services::import_export::ImportImageInput {
                    data: image.data,
                    name: image.name,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct ImportResponse {
    pub success: bool,
    pub message: String,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub persisted_with_errors: Vec<i32>,
    pub errors: Vec<String>,
}

impl From<crate::services::import_export::ImportResult> for ImportResponse {
    fn from(result: crate::services::import_export::ImportResult) -> Self {
        Self {
            success: result.success,
            message: result.message,
            imported_count: result.imported_count,
            skipped_count: result.skipped_count,
            persisted_with_errors: result.persisted_with_errors,
            errors: result.errors,
        }
    }
}

/// POST /api/v1/import/posts — Import posts from ZIP
#[utoipa::path(
    post,
    path = "/api/v1/import/posts",
    responses((status = 200, description = "成功")),
    tag = "Import/Export"
)]
pub async fn import_markdown(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ApiResponse<ImportResponse>>, AppError> {
    if !matches!(auth.role.as_str(), "author" | "sub_admin" | "admin") {
        return Err(AppError::Forbidden);
    }

    let items: Vec<_> = req.items.into_iter().map(Into::into).collect();
    let result =
        crate::services::import_export::import_markdown_items(&state, auth.user_id, &items).await?;
    Ok(Json(ApiResponse::new(result.into())))
}
