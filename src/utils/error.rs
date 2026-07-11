use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未授权")]
    Unauthorized,
    #[error("禁止访问")]
    Forbidden,
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("请求错误: {0}")]
    BadRequest(String),
    #[error("验证错误: {0}")]
    Validation(String),
    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("数据库错误")]
    DbError(#[from] sea_orm::DbErr),
    #[error("IO错误")]
    IoError(#[from] std::io::Error),
    #[error("ZIP错误")]
    ZipError(#[from] zip::result::ZipError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the full error details before returning a sanitized response
        if let AppError::Internal(ref e) = self {
            tracing::error!("Internal error: {:#}", e);
        }
        if let AppError::DbError(ref e) = self {
            tracing::error!("Database error: {:#}", e);
        }

        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "未授权".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "禁止访问".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "内部错误".to_string()),
            AppError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误".to_string()),
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO错误".to_string()),
            AppError::ZipError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "压缩包错误".to_string()),
        };
        (status, Json(json!({ "message": message }))).into_response()
    }
}
