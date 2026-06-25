mod error;
mod response;
pub mod client_info;
pub mod serde_helpers;
pub mod ip_utils;
pub mod ip_migration;

pub use error::AppError;
pub use response::{ApiResponse, Pagination};

/// 返回服务器本地时区的 NaiveDateTime（不转换）
pub fn now_local() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use crate::config::AppConfig;
use crate::services::search::SearchEngine;
use crate::services::logs::LogBuffer;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub search_engine: SearchEngine,
    pub tera: Arc<tera::Tera>,
    pub log_buffer: Arc<LogBuffer>,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        config: AppConfig,
        search_engine: SearchEngine,
        log_buffer: Arc<LogBuffer>,
    ) -> Self {
        let templates_dir = format!("{}/templates/default", config.data_dir);

        let tera = tera::Tera::new(&format!("{}/**/*", templates_dir))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to load templates from {}: {}. Using empty engine.", templates_dir, e);
                tera::Tera::default()
            });

        tracing::info!("Templates loaded from {}", templates_dir);

        Self {
            db,
            config,
            search_engine,
            tera: Arc::new(tera),
            log_buffer,
        }
    }
}
