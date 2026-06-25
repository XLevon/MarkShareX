pub mod entity;

use sea_orm::{Database, DatabaseConnection, ConnectOptions, ConnectionTrait};
use crate::config::DatabaseConfig;

pub async fn init_db(config: &DatabaseConfig) -> anyhow::Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(&config.url);
    opt.max_connections(config.max_connections);
    opt.min_connections(config.min_connections);
    let db = Database::connect(opt).await?;
    Ok(db)
}

/// 执行初始化 SQL 脚本（编译时嵌入，无需外部文件）
///
/// 每次启动执行，幂等：所有 DDL 使用 IF NOT EXISTS，
/// ALTER TABLE ADD COLUMN 失败（列已存在）被自动忽略。
pub async fn run_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    let sql = include_str!("../../migrations/0000000000_init_schema.sql");

    for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
        let trimmed = statement.trim();
        // 跳过纯注释块
        if trimmed.starts_with("--") && !trimmed.contains('\n') {
            continue;
        }
        let result = db.execute_unprepared(&format!("{};", trimmed)).await;
        if let Err(e) = result {
            let err_msg = e.to_string().to_lowercase();
            if err_msg.contains("duplicate column") {
                tracing::debug!("  ⚠️ 列已存在（跳过）: {:?}", trimmed.lines().next());
                continue;
            }
            // 其他错误也不阻塞启动（幂等容忍）
            tracing::warn!("  ⚠️ SQL 执行跳过: {} —— {}", trimmed.lines().next().unwrap_or(""), e);
        }
    }

    Ok(())
}
