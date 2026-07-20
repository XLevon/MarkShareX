use sea_orm::{DatabaseConnection, ConnectionTrait, TransactionTrait};

/// 文件迁移表的建表语句
const CREATE_MIGRATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS _migrations (
    name TEXT PRIMARY KEY,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
)"#;

// 嵌入所有增量迁移文件（编译时由 build.rs 生成）
include!(concat!(env!("OUT_DIR"), "/embedded_migrations.rs"));

/// 执行所有嵌入在二进制中的增量迁移。
///
/// - 创建 `_migrations` 追踪表（如果不存在）
/// - 遍历嵌入的 MIGRATIONS 数组（已按文件名排序）
/// - 跳过已记录的迁移
/// - 每个迁移在独立事务中执行，失败回滚
pub async fn run(db: &DatabaseConnection) -> anyhow::Result<usize> {
    // 1. 确保追踪表存在
    db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;

    if MIGRATIONS.is_empty() {
        tracing::info!("📦 没有待执行的迁移文件");
        return Ok(0);
    }

    // 2. 查询已执行的迁移
    let executed: Vec<String> = {
        let stmt = "SELECT name FROM _migrations ORDER BY name";
        let rows = db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                stmt.to_owned(),
            ))
            .await?;
        rows.iter()
            .filter_map(|row| row.try_get_by_index::<String>(0).ok())
            .collect()
    };

    let mut count = 0;

    for (file_name, sql) in MIGRATIONS {
        if executed.iter().any(|e| e == file_name) {
            continue;
        }

        if sql.trim().is_empty() {
            continue;
        }

        tracing::info!("🔄 执行迁移: {}", file_name);

        // 在事务中执行：迁移 SQL + 记录
        let execute_result: Result<(), anyhow::Error> = async {
            let txn = db.begin().await?;
            for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
                let migrated_sql = format!("{};", statement.trim());
                let result = txn.execute_unprepared(&migrated_sql).await;
                if let Err(e) = result {
                    let err_msg = e.to_string().to_lowercase();
                    // Only skip "duplicate column name" errors from
                    // ALTER TABLE ADD COLUMN when re-running on already-migrated DB.
                    if err_msg.contains("duplicate column name") {
                        tracing::warn!(
                            "  ⚠️ 列已存在（幂等跳过）: {}",
                            statement.trim().lines().next().unwrap_or("")
                        );
                        continue;
                    }
                    txn.rollback().await.ok();
                    return Err(anyhow::anyhow!(
                        "迁移 {} 失败: {} —— SQL: {}",
                        file_name,
                        e,
                        migrated_sql
                    ));
                }
            }
            txn.execute_unprepared(&format!(
                "INSERT INTO _migrations (name) VALUES ('{}')",
                file_name.replace('\'', "''")
            ))
            .await?;
            txn.commit().await?;
            Ok(())
        }
        .await;

        execute_result?;

        tracing::info!("✅ 迁移完成: {}", file_name);
        count += 1;
    }

    if count > 0 {
        tracing::info!("📦 共执行 {} 个迁移", count);
    } else {
        tracing::info!("📦 所有迁移已是最新");
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    #[test]
    fn guest_copy_default_is_present_in_initial_and_incremental_migrations() {
        let initial = include_str!("../migrations/0000000000_init_schema.sql");
        let incremental = include_str!("../migrations/0000000011_guest_copy_enabled.sql");

        for sql in [initial, incremental] {
            assert!(sql.contains("guest_copy_enabled"));
            assert!(sql.contains("'true'"));
            assert!(sql.contains("INSERT OR IGNORE INTO settings"));
        }
    }
}
