pub mod client_info;
mod error;
pub mod ip_migration;
pub mod ip_utils;
mod response;
pub mod safe_url;
pub mod serde_helpers;

pub use error::AppError;
pub use response::{ApiResponse, Pagination};

/// 返回服务器本地时区的 NaiveDateTime（不转换）
pub fn now_local() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

use crate::config::AppConfig;
use crate::services::logs::LogBuffer;
use crate::services::search::SearchEngine;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub search_engine: SearchEngine,
    pub tera: Arc<tera::Tera>,
    pub log_buffer: Arc<LogBuffer>,
    pub ip_guard_rules_cache: Arc<RwLock<IpGuardRulesCache>>,
}

/// Parsed IP guard configuration, cacheable.
#[derive(Clone, Default)]
pub struct IpGuardRules {
    pub blacklist_enabled: bool,
    pub blacklist: Vec<String>,
    pub whitelist_enabled: bool,
    pub whitelist: Vec<String>,
}

#[derive(Default)]
pub struct IpGuardRulesCache {
    generation: u64,
    entry: Option<(Instant, IpGuardRules)>,
}

impl IpGuardRulesCache {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn entry(&self) -> Option<&(Instant, IpGuardRules)> {
        self.entry.as_ref()
    }

    pub fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.entry = None;
    }

    pub fn store_if_current(&mut self, generation: u64, rules: IpGuardRules) -> bool {
        if self.generation != generation {
            return false;
        }
        self.entry = Some((Instant::now(), rules));
        true
    }
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        config: AppConfig,
        search_engine: SearchEngine,
        log_buffer: Arc<LogBuffer>,
    ) -> Self {
        let templates_dir = format!("{}/templates/default", config.data_dir);

        let tera = tera::Tera::new(&format!("{}/**/*", templates_dir)).unwrap_or_else(|e| {
            tracing::error!(
                "Failed to load templates from {}: {}. Using empty engine.",
                templates_dir,
                e
            );
            tera::Tera::default()
        });

        tracing::info!("Templates loaded from {}", templates_dir);

        Self {
            db,
            config,
            search_engine,
            tera: Arc::new(tera),
            log_buffer,
            ip_guard_rules_cache: Arc::new(RwLock::new(IpGuardRulesCache::default())),
        }
    }

    pub async fn invalidate_ip_guard_rules_cache(&self) {
        self.ip_guard_rules_cache.write().await.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidation_generation_rejects_stale_ip_guard_cache_fill() {
        let mut cache = IpGuardRulesCache::default();
        let stale_generation = cache.generation();
        cache.invalidate();

        assert!(!cache.store_if_current(
            stale_generation,
            IpGuardRules {
                blacklist_enabled: true,
                ..Default::default()
            }
        ));
        assert!(cache.entry().is_none());
    }
}
