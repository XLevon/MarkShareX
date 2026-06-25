use axum::{extract::{State, Query, Path, Multipart}, Json};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use crate::utils::{AppState, AppError, ApiResponse, Pagination};
use crate::middleware::auth::AuthUser;
use crate::models::entity::files;
use crate::services::files as file_service;

/// 根据文件扩展名推断 MIME 类型，作为浏览器 content_type 不可靠时的回退
fn infer_mime_by_extension(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    Some(match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        "zip" => "application/zip",
        "gz" | "tgz" => "application/gzip",
        "tar" => "application/x-tar",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/x-rar-compressed",
        // 视频
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogv" | "ogg" => "video/ogg",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        // 音频
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        _ => return None,
    })
}

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
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<FileResponse>>, AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden);
    }

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

        let original_name = field
            .file_name()
            .unwrap_or("unknown")
            .to_string();

        // 文件类型校验：优先用浏览器 MIME，失败时按扩展名推断
        let is_allowed = state.config.storage.allowed_types.iter().any(|t| t == &content_type)
            || infer_mime_by_extension(&original_name)
                .map(|m| state.config.storage.allowed_types.iter().any(|t| t == m))
                .unwrap_or(false);

        if !is_allowed {
            return Err(AppError::BadRequest(format!(
                "不支持的文件类型: {} (文件: {})", content_type, original_name
            )));
        }

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
            auth.user_id,
            &original_name,
            &content_type,
            &data,
            &state.config.storage.upload_dir,
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
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden);
    }
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
    auth: AuthUser,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<ApiResponse<BatchDeleteResult>>, AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden);
    }
    let deleted = crate::services::files::batch_delete_files(&state.db, &req.ids, &state.config.storage.upload_dir).await?;
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
    _auth: AuthUser,
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
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<Vec<BatchUploadResult>>>, AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden);
    }
    
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

            // 文件类型校验：优先用浏览器 MIME，失败时按扩展名推断
            let is_allowed = state.config.storage.allowed_types.iter().any(|t| t == &content_type)
                || infer_mime_by_extension(&original_name)
                    .map(|m| state.config.storage.allowed_types.iter().any(|t| t == m))
                    .unwrap_or(false);

            if !is_allowed {
                let name_for_error = original_name.clone();
                results.push(BatchUploadResult {
                    original_name,
                    success: false,
                    url: None,
                    file_id: None,
                    error: Some(format!("不支持的文件类型: {} (文件: {})", content_type, name_for_error)),
                });
                continue;
            }
            
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
                auth.user_id,
                &original_name,
                &content_type,
                &data,
                &state.config.storage.upload_dir,
                None,
                false,
            ).await {
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
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<FileResponse>>>, AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden);
    }

    let unreferenced_files = file_service::get_unreferenced_files(&state.db).await?;

    let response = unreferenced_files
        .into_iter()
        .map(|f| FileResponse::from_model(f))
        .collect();

    Ok(Json(ApiResponse::new(response)))
}
