use crate::middleware::auth::{AdminUser, AuthUser};
use crate::models::entity::files;
use crate::services::files as file_service;
use crate::utils::{ApiResponse, AppError, AppState, Pagination};
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, Response},
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct FileResponse {
    pub id: i32,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub url: String,
    pub created_at: chrono::NaiveDateTime,
}

impl FileResponse {
    fn from_model(f: files::Model) -> Self {
        let url = file_service::get_file_url(&f.filename);

        Self {
            id: f.id,
            filename: f.filename,
            original_name: f.original_name,
            mime_type: f.mime_type,
            size: f.size,
            url,
            created_at: f.created_at,
        }
    }
}

/// GET /uploads/:filename — Serve one validated storage file without following symlinks.
pub async fn serve_upload(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response<Body>, AppError> {
    let upload_dir = state.config.storage.upload_dir.clone();
    let read_filename = filename.clone();
    let read_result = tokio::task::spawn_blocking(move || {
        file_service::read_storage_file(&read_filename, &upload_dir)
    })
    .await
    .map_err(|error| AppError::Internal(error.into()))?;
    let data = match read_result {
        Ok(data) => data,
        Err(AppError::IoError(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::NotFound("文件不存在".to_string()));
        }
        Err(error) => return Err(error),
    };
    let content_type =
        file_service::infer_mime_by_extension(&filename).unwrap_or("application/octet-stream");
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, data.len())
        .body(Body::from(data))
        .map_err(|error| AppError::Internal(error.into()))
}

#[derive(Deserialize)]
pub struct ListFilesQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Deserialize, Default)]
pub struct UploadQuery {
    pub overwrite: Option<bool>,
    pub rename: Option<String>,
}
/// POST /api/v1/files/upload — Upload file

#[utoipa::path(
    post,
    path = "/api/v1/files/upload",
    responses((status = 200, description = "成功", body = FileResponse)),
    tag = "Files"
)]
pub async fn upload_file(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    auth: AdminUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<FileResponse>>, AppError> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        // 处理SVG文件的content_type
        let content_type = if content_type == "image/svg" {
            "image/svg+xml".to_string()
        } else {
            content_type
        };

        let original_name = field.file_name().unwrap_or("unknown").to_string();

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("读取文件失败: {}", e)))?;

        if data.len() as u64 > state.config.storage.max_file_size {
            return Err(AppError::BadRequest(format!(
                "文件过大: {} bytes (最大 {} bytes)",
                data.len(),
                state.config.storage.max_file_size
            )));
        }

        let file = crate::services::files::upload_file(
            &state.db,
            auth.0.user_id,
            &original_name,
            &content_type,
            &data,
            &state.config.storage.upload_dir,
            &state.config.storage.allowed_types,
            state.config.storage.max_file_size,
            query.rename.as_deref(),
            query.overwrite.unwrap_or(false),
        )
        .await?;

        let response = FileResponse::from_model(file);
        return Ok(Json(ApiResponse::new(response)));
    }

    Err(AppError::BadRequest("没有上传文件".to_string()))
}
/// GET /api/v1/files — List files

#[utoipa::path(
    get,
    path = "/api/v1/files",
    responses((status = 200, description = "成功", body = [FileResponse])),
    tag = "Files"
)]
pub async fn list_files(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<ApiResponse<Vec<FileResponse>>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let (items, total) = if auth.is_privileged() {
        crate::services::files::list_files(&state.db, page, page_size).await?
    } else {
        crate::services::files::list_files_by_user(&state.db, auth.user_id, page, page_size).await?
    };
    let pagination = Pagination::new(total, page, page_size);

    // 动态计算每个文件的 URL
    let data: Vec<FileResponse> = items
        .into_iter()
        .map(|f| FileResponse::from_model(f))
        .collect();

    Ok(Json(ApiResponse::with_pagination(data, pagination)))
}
/// DELETE /api/v1/files/{id} — Delete file

#[utoipa::path(
    delete,
    path = "/api/v1/files/{id}",
    responses((status = 200, description = "成功")),
    tag = "Files"
)]
pub async fn delete_file(
    State(state): State<AppState>,
    _auth: AdminUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    crate::services::files::delete_file(&state.db, id, &state.config.storage.upload_dir).await?;
    Ok(Json(ApiResponse::new(())))
}

// 批量删除文件
#[derive(Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct BatchDeleteResult {
    pub deleted: usize,
}
/// DELETE /api/v1/files/batch — Batch delete files

#[utoipa::path(
    delete,
    path = "/api/v1/files/batch",
    responses((status = 200, description = "成功")),
    tag = "Files"
)]
pub async fn batch_delete_files(
    State(state): State<AppState>,
    _auth: AdminUser,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<ApiResponse<BatchDeleteResult>>, AppError> {
    let deleted = crate::services::files::batch_delete_files(
        &state.db,
        &req.ids,
        &state.config.storage.upload_dir,
    )
    .await?;
    Ok(Json(ApiResponse::new(BatchDeleteResult { deleted })))
}

// 新增：检查 MD5 是否已存在
#[derive(Deserialize)]
pub struct CheckMd5Request {
    pub md5_list: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Md5CheckResult {
    pub md5: String,
    pub exists: bool,
    pub url: Option<String>,
    pub file_id: Option<i32>,
}
/// POST /api/v1/files/check-md5 — Check file MD5 hash

#[utoipa::path(
    post,
    path = "/api/v1/files/check-md5",
    responses((status = 200, description = "成功")),
    tag = "Files"
)]
pub async fn check_md5_exists(
    State(state): State<AppState>,
    _auth: AdminUser,
    Json(body): Json<CheckMd5Request>,
) -> Result<Json<ApiResponse<Vec<Md5CheckResult>>>, AppError> {
    let mut results: Vec<Md5CheckResult> = Vec::new();

    for md5 in body.md5_list {
        if let Some(file) = files::Entity::find()
            .filter(
                sea_orm::Condition::all()
                    .add(files::Column::Md5Hash.eq(&md5))
                    .add(files::Column::DeletedAt.is_null()),
            )
            .one(&state.db)
            .await?
        {
            let url = file_service::get_file_url(&file.filename);
            results.push(Md5CheckResult {
                md5: md5.clone(),
                exists: true,
                url: Some(url),
                file_id: Some(file.id),
            });
        } else {
            results.push(Md5CheckResult {
                md5: md5.clone(),
                exists: false,
                url: None,
                file_id: None,
            });
        }
    }

    Ok(Json(ApiResponse::new(results)))
}

// 新增：批量上传文件
#[derive(Serialize, ToSchema)]
pub struct BatchUploadResult {
    pub original_name: String,
    pub success: bool,
    pub url: Option<String>,
    pub file_id: Option<i32>,
    pub error: Option<String>,
}

/// POST /api/v1/files/batch — Batch upload files

#[utoipa::path(
    post,
    path = "/api/v1/files/batch",
    responses((status = 200, description = "成功", body = [FileResponse])),
    tag = "Files"
)]
pub async fn batch_upload(
    State(state): State<AppState>,
    auth: AdminUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<Vec<BatchUploadResult>>>, AppError> {
    let mut results: Vec<BatchUploadResult> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");

        if name == "files" {
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            // 处理SVG文件的content_type
            let content_type = if content_type == "image/svg" {
                "image/svg+xml".to_string()
            } else {
                content_type
            };

            let original_name = field.file_name().unwrap_or("unknown").to_string();

            let data = match field.bytes().await {
                Ok(d) => d,
                Err(e) => {
                    results.push(BatchUploadResult {
                        original_name: original_name.clone(),
                        success: false,
                        url: None,
                        file_id: None,
                        error: Some(format!("读取文件失败: {}", e)),
                    });
                    continue;
                }
            };

            // 检查文件大小
            if data.len() as u64 > state.config.storage.max_file_size {
                results.push(BatchUploadResult {
                    original_name: original_name.clone(),
                    success: false,
                    url: None,
                    file_id: None,
                    error: Some(format!(
                        "文件过大: {} bytes (最大 {} bytes)",
                        data.len(),
                        state.config.storage.max_file_size
                    )),
                });
                continue;
            }

            // 上传文件（已集成 MD5 去重）
            match crate::services::files::upload_file(
                &state.db,
                auth.0.user_id,
                &original_name,
                &content_type,
                &data,
                &state.config.storage.upload_dir,
                &state.config.storage.allowed_types,
                state.config.storage.max_file_size,
                None,
                false,
            )
            .await
            {
                Ok(file) => {
                    let url = file_service::get_file_url(&file.filename);
                    results.push(BatchUploadResult {
                        original_name: original_name.clone(),
                        success: true,
                        url: Some(url),
                        file_id: Some(file.id),
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(BatchUploadResult {
                        original_name: original_name.clone(),
                        success: false,
                        url: None,
                        file_id: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    Ok(Json(ApiResponse::new(results)))
}
/// GET /api/v1/files/unreferenced — List unreferenced files

#[utoipa::path(
    get,
    path = "/api/v1/files/unreferenced",
    responses((status = 200, description = "成功", body = [FileResponse])),
    tag = "Files"
)]
pub async fn list_unreferenced_files(
    State(state): State<AppState>,
    _auth: AdminUser,
) -> Result<Json<ApiResponse<Vec<FileResponse>>>, AppError> {
    let unreferenced_files = file_service::get_unreferenced_files(&state.db).await?;

    let response = unreferenced_files
        .into_iter()
        .map(|f| FileResponse::from_model(f))
        .collect();

    Ok(Json(ApiResponse::new(response)))
}
