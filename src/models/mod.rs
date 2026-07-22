pub mod entity;

use crate::config::DatabaseConfig;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
    TransactionTrait,
};

pub async fn init_db(config: &DatabaseConfig) -> anyhow::Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(&config.url);
    opt.max_connections(config.max_connections);
    opt.min_connections(config.min_connections);
    let db = Database::connect(opt).await?;
    // SQLite 默认不启用外键约束，需要显式开启以保证 CASCADE 删除生效
    db.execute_unprepared("PRAGMA foreign_keys = ON").await?;
    Ok(db)
}

#[cfg(test)]
async fn table_exists(db: &impl ConnectionTrait, name: &str) -> anyhow::Result<bool> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?)",
            [name.into()],
        ))
        .await?;
    Ok(row
        .and_then(|row| row.try_get_by_index::<i64>(0).ok())
        .is_some_and(|exists| exists == 1))
}

async fn database_is_empty(db: &impl ConnectionTrait) -> anyhow::Result<bool> {
    let row = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT NOT EXISTS(
                 SELECT 1 FROM sqlite_master
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
             )"
            .to_string(),
        ))
        .await?;
    Ok(row
        .and_then(|row| row.try_get_by_index::<i64>(0).ok())
        .is_some_and(|empty| empty == 1))
}

async fn apply_initial_schema(
    db: &DatabaseConnection,
    file_name: &str,
    sql: &str,
    migration_names: &[&str],
) -> anyhow::Result<()> {
    crate::migrations::reject_transaction_control(sql)
        .map_err(|error| anyhow::anyhow!("初始化 schema {file_name} 失败: {error}"))?;
    let txn = db.begin().await?;
    if let Err(error) = txn.execute_unprepared(sql).await {
        txn.rollback().await.ok();
        return Err(anyhow::anyhow!("初始化 schema {file_name} 失败: {error}"));
    }
    if let Err(error) = crate::migrations::record_migration_names(&txn, migration_names).await {
        txn.rollback().await.ok();
        return Err(anyhow::anyhow!("记录初始迁移基线失败: {error}"));
    }
    txn.commit()
        .await
        .map_err(|error| anyhow::anyhow!("提交初始 schema 事务失败: {error}"))?;
    Ok(())
}

/// 真正空数据库原子执行内嵌初始化 schema，并记录当前增量迁移基线。
/// 任何已有用户表的数据库都跳过初始化脚本，由增量迁移执行器升级；这可避免
/// 当前初始化 schema 中引用新列的索引阻断历史数据库的 ALTER 迁移。
pub async fn run_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    if !database_is_empty(db).await? {
        return Ok(());
    }
    apply_initial_schema(
        db,
        "0000000000_init_schema.sql",
        include_str!("../../migrations/0000000000_init_schema.sql"),
        &crate::migrations::embedded_migration_names(),
    )
    .await
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

    const EXPECTED_INCREMENTAL_MIGRATIONS: [&str; 9] = [
        "0000000001_categories_is_visible.sql",
        "0000000002_posts_cover_image_url.sql",
        "0000000003_users_title.sql",
        "0000000004_changelog_status.sql",
        "0000000007_guestbook_add_columns.sql",
        "0000000008_news_topic_type.sql",
        "0000000009_task_max_rounds.sql",
        "0000000010_news_source_url.sql",
        "0000000011_guest_copy_enabled.sql",
    ];
    const EXPECTED_FRESH_SCHEMA_FINGERPRINT: u64 = 5_863_388_009_344_279_435;
    const EXPECTED_UPGRADED_HISTORICAL_SCHEMA_FINGERPRINT: u64 = 210_800_713_160_823_661;
    const EXPECTED_FIXED_SEED_FINGERPRINT: u64 = 3_420_813_530_984_549_862;

    async fn assert_index_definition(
        db: &DatabaseConnection,
        name: &str,
        table: &str,
        unique: bool,
        columns: &[&str],
    ) -> anyhow::Result<()> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT tbl_name, sql FROM sqlite_master WHERE type='index' AND name=?",
                [name.into()],
            ))
            .await?
            .unwrap_or_else(|| panic!("missing fresh index {name}"));
        assert_eq!(
            row.try_get_by_index::<String>(0)?,
            table,
            "wrong table for {name}"
        );
        let definition = row.try_get_by_index::<String>(1)?;
        assert_eq!(
            definition
                .to_ascii_uppercase()
                .starts_with("CREATE UNIQUE INDEX"),
            unique,
            "wrong uniqueness for {name}: {definition}"
        );

        let actual_columns = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("PRAGMA index_info('{name}')"),
            ))
            .await?
            .into_iter()
            .map(|row| row.try_get_by_index::<String>(2))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(actual_columns, columns, "wrong columns/order for {name}");
        Ok(())
    }

    async fn application_schema_fingerprint(db: &DatabaseConnection) -> anyhow::Result<u64> {
        let rows = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT type, name, COALESCE(sql, '') FROM sqlite_master
                 WHERE type IN ('table', 'index', 'view', 'trigger') AND name NOT LIKE 'sqlite_%'
                 ORDER BY type, name"
                    .to_string(),
            ))
            .await?;
        let mut hash = 0xcbf29ce484222325_u64;
        for row in rows {
            let object_type = row.try_get_by_index::<String>(0)?;
            let name = row.try_get_by_index::<String>(1)?;
            let sql = row.try_get_by_index::<String>(2)?;
            let normalized_sql = sql
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase();
            for byte in format!("{object_type}\0{name}\0{normalized_sql}\0").bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        Ok(hash)
    }

    async fn fixed_seed_fingerprint(db: &DatabaseConnection) -> anyhow::Result<u64> {
        let queries = [
            (
                "article_types",
                "SELECT quote(id)||'|'||quote(code)||'|'||quote(display_name)||'|'||quote(color)||'|'||quote(sort_order)||'|'||quote(is_active) FROM article_types ORDER BY id",
            ),
            (
                "article_statuses",
                "SELECT quote(id)||'|'||quote(code)||'|'||quote(display_name)||'|'||quote(color)||'|'||quote(sort_order)||'|'||quote(is_active) FROM article_statuses ORDER BY id",
            ),
            (
                "ai_tools",
                "SELECT quote(id)||'|'||quote(name)||'|'||quote(description)||'|'||quote(function_name)||'|'||quote(parameters_schema)||'|'||quote(enabled)||'|'||quote(config) FROM ai_tools ORDER BY id",
            ),
            (
                "ai_agent_config",
                "SELECT quote(id)||'|'||quote(name)||'|'||quote(system_prompt)||'|'||quote(user_prompt)||'|'||quote(is_default)||'|'||quote(model_id) FROM ai_agent_config ORDER BY id",
            ),
            (
                "settings",
                "SELECT quote(key)||'|'||quote(value) FROM settings ORDER BY key",
            ),
        ];
        let mut hash = 0xcbf29ce484222325_u64;
        for (table, query) in queries {
            for byte in format!("{table}\0").bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
            for row in db
                .query_all(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    query.to_string(),
                ))
                .await?
            {
                let value = row.try_get_by_index::<String>(0)?;
                for byte in format!("{value}\0").bytes() {
                    hash ^= u64::from(byte);
                    hash = hash.wrapping_mul(0x100000001b3);
                }
            }
        }
        Ok(hash)
    }

    #[tokio::test]
    async fn fresh_schema_atomically_records_embedded_migration_baseline() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;

        run_migrations(&db).await?;

        assert_eq!(
            fixed_seed_fingerprint(&db).await?,
            EXPECTED_FIXED_SEED_FINGERPRINT,
            "fixed seed rows changed: review all exact values and update the oracle intentionally"
        );

        for table in [
            "users",
            "categories",
            "tags",
            "posts",
            "post_tags",
            "files",
            "settings",
            "refresh_tokens",
            "likes",
            "comments",
            "author_applications",
            "network_resources",
            "login_logs",
            "read_logs",
            "changelog",
            "article_types",
            "article_statuses",
            "guestbook",
            "_migrations",
            "news",
            "ai_providers",
            "ai_models",
            "ai_tools",
            "ai_agent_config",
            "ai_skills",
            "ai_tasks",
            "ai_task_logs",
            "ai_chat_sessions",
            "ai_chat_messages",
        ] {
            assert!(
                table_exists(&db, table).await?,
                "missing fresh table {table}"
            );
        }
        let application_table_count = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
                    .to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<i64>(0).ok());
        assert_eq!(
            application_table_count,
            Some(29),
            "fresh schema must contain exactly the expected application tables"
        );

        for (name, table, unique, columns) in [
            (
                "idx_categories_deleted",
                "categories",
                false,
                &["deleted_at"][..],
            ),
            ("idx_tags_deleted", "tags", false, &["deleted_at"][..]),
            ("idx_posts_status", "posts", false, &["status"][..]),
            ("idx_posts_category", "posts", false, &["category_id"][..]),
            ("idx_posts_deleted", "posts", false, &["deleted_at"][..]),
            (
                "idx_posts_published",
                "posts",
                false,
                &["status", "published_at"][..],
            ),
            ("idx_post_tags_tag", "post_tags", false, &["tag_id"][..]),
            ("idx_post_tags_post", "post_tags", false, &["post_id"][..]),
            ("idx_files_deleted", "files", false, &["deleted_at"][..]),
            (
                "idx_refresh_tokens_user",
                "refresh_tokens",
                false,
                &["user_id"][..],
            ),
            (
                "idx_refresh_tokens_token",
                "refresh_tokens",
                false,
                &["token"][..],
            ),
            ("idx_comments_post", "comments", false, &["post_id"][..]),
            ("idx_comments_status", "comments", false, &["status"][..]),
            (
                "idx_comments_deleted",
                "comments",
                false,
                &["deleted_at"][..],
            ),
            (
                "idx_network_resources_url",
                "network_resources",
                true,
                &["url"][..],
            ),
            ("idx_login_logs_user", "login_logs", false, &["user_id"][..]),
            (
                "idx_login_logs_created",
                "login_logs",
                false,
                &["created_at"][..],
            ),
            ("idx_read_logs_post", "read_logs", false, &["post_id"][..]),
            ("idx_read_logs_user", "read_logs", false, &["user_id"][..]),
            (
                "idx_read_logs_created",
                "read_logs",
                false,
                &["created_at"][..],
            ),
            ("idx_news_status", "news", false, &["status"][..]),
            ("idx_news_topic_type", "news", false, &["topic_type"][..]),
            ("idx_news_sort_order", "news", false, &["sort_order"][..]),
            ("idx_news_created_at", "news", false, &["created_at"][..]),
            ("idx_news_source_url", "news", false, &["source_url"][..]),
            (
                "idx_ai_tools_function_name",
                "ai_tools",
                true,
                &["function_name"][..],
            ),
            (
                "idx_ai_task_logs_task_id",
                "ai_task_logs",
                false,
                &["task_id"][..],
            ),
        ] {
            assert_index_definition(&db, name, table, unique, columns).await?;
        }
        let application_index_count = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
                    .to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<i64>(0).ok());
        assert_eq!(
            application_index_count,
            Some(27),
            "fresh schema must contain exactly the expected application indexes"
        );
        assert_eq!(
            application_schema_fingerprint(&db).await?,
            EXPECTED_FRESH_SCHEMA_FINGERPRINT,
            "fresh schema definitions changed: review every table/index definition and update the fixed oracle intentionally"
        );

        for (table, column) in [
            ("categories", "is_visible"),
            ("posts", "cover_image_url"),
            ("users", "title"),
            ("changelog", "status"),
            ("guestbook", "deleted_at"),
            ("news", "topic_type"),
            ("news", "source_url"),
            ("ai_tasks", "max_tool_rounds"),
        ] {
            let present = db
                .query_one(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    format!(
                        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('{table}') WHERE name='{column}')"
                    ),
                ))
                .await?
                .and_then(|row| row.try_get_by_index::<i64>(0).ok());
            assert_eq!(present, Some(1), "missing fresh {table}.{column}");
        }

        let guest_copy = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT value FROM settings WHERE key='guest_copy_enabled'".to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<String>(0).ok());
        assert_eq!(guest_copy.as_deref(), Some("true"));

        let article_types = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT code, display_name, color, sort_order, is_active FROM article_types ORDER BY sort_order"
                    .to_string(),
            ))
            .await?
            .into_iter()
            .map(|row| {
                Ok((
                    row.try_get_by_index::<String>(0)?,
                    row.try_get_by_index::<String>(1)?,
                    row.try_get_by_index::<String>(2)?,
                    row.try_get_by_index::<i64>(3)?,
                    row.try_get_by_index::<i64>(4)?,
                ))
            })
            .collect::<Result<Vec<_>, sea_orm::DbErr>>()?;
        assert_eq!(
            article_types,
            vec![
                ("space".into(), "".into(), "#6b7280".into(), 0, 1),
                (
                    "ai_organized".into(),
                    "🤖 AI整理".into(),
                    "#a855f7".into(),
                    1,
                    1
                ),
                ("original".into(), "📝 原创".into(), "#3b82f6".into(), 2, 1),
                ("tutorial".into(), "📚 教程".into(), "#22c55e".into(), 3, 1),
                ("repost".into(), "🔗 转载".into(), "#fb923c".into(), 4, 1),
                (
                    "translation".into(),
                    "🌐 翻译".into(),
                    "#fb923c".into(),
                    5,
                    1
                ),
                (
                    "opinion_essay".into(),
                    "💡 随笔".into(),
                    "#ec4899".into(),
                    6,
                    1
                ),
            ]
        );

        let article_statuses = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT code, display_name, color, sort_order, is_active FROM article_statuses ORDER BY sort_order"
                    .to_string(),
            ))
            .await?
            .into_iter()
            .map(|row| {
                Ok((
                    row.try_get_by_index::<String>(0)?,
                    row.try_get_by_index::<String>(1)?,
                    row.try_get_by_index::<String>(2)?,
                    row.try_get_by_index::<i64>(3)?,
                    row.try_get_by_index::<i64>(4)?,
                ))
            })
            .collect::<Result<Vec<_>, sea_orm::DbErr>>()?;
        assert_eq!(
            article_statuses,
            vec![
                ("space".into(), "".into(), "#6b7280".into(), 1, 1),
                ("latest".into(), "✅ 最新".into(), "#22c55e".into(), 2, 1),
            ]
        );

        let ai_tools = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name, function_name, enabled, config, parameters_schema FROM ai_tools ORDER BY id"
                    .to_string(),
            ))
            .await?
            .into_iter()
            .map(|row| {
                Ok((
                    row.try_get_by_index::<String>(0)?,
                    row.try_get_by_index::<String>(1)?,
                    row.try_get_by_index::<i64>(2)?,
                    row.try_get_by_index::<String>(3)?,
                    row.try_get_by_index::<String>(4)?,
                ))
            })
            .collect::<Result<Vec<_>, sea_orm::DbErr>>()?;
        assert_eq!(
            ai_tools
                .iter()
                .map(|(name, function, _, _, _)| (name.as_str(), function.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("获取当前日期时间", "get_current_datetime"),
                ("站内 API 请求", "api_request"),
                ("网络搜索", "web_search"),
                ("网页抓取", "web_extract"),
                ("创建资讯", "create_news"),
                ("创建文章", "create_post"),
            ]
        );
        assert!(ai_tools
            .iter()
            .all(|(_, _, enabled, config, schema)| *enabled == 1
                && config == "{}"
                && schema.contains("\"type\"")));

        let agent = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT id, name, system_prompt, user_prompt, is_default FROM ai_agent_config"
                    .to_string(),
            ))
            .await?
            .expect("missing default AI agent config");
        assert_eq!(agent.try_get_by_index::<i64>(0)?, 1);
        assert_eq!(agent.try_get_by_index::<String>(1)?, " AI助手");
        assert_eq!(agent.try_get_by_index::<String>(2)?, "");
        assert_eq!(agent.try_get_by_index::<String>(3)?, "");
        assert_eq!(agent.try_get_by_index::<i64>(4)?, 0);

        let expected_migrations = EXPECTED_INCREMENTAL_MIGRATIONS.to_vec();
        assert_eq!(
            crate::migrations::embedded_migration_names(),
            expected_migrations,
            "build.rs must embed the exact ordered incremental migration set"
        );
        let recorded_names = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM _migrations ORDER BY name".to_string(),
            ))
            .await?
            .into_iter()
            .map(|row| row.try_get_by_index::<String>(0))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(recorded_names, expected_migrations);
        assert_eq!(crate::migrations::run(&db).await?, 0);
        run_migrations(&db).await?;
        assert_eq!(crate::migrations::run(&db).await?, 0);
        Ok(())
    }

    #[tokio::test]
    async fn invalid_initial_schema_rolls_back_and_prevents_partial_startup() -> anyhow::Result<()>
    {
        let db = Database::connect("sqlite::memory:").await?;
        let error = apply_initial_schema(
            &db,
            "broken-init.sql",
            "CREATE TABLE partial_init (id INTEGER PRIMARY KEY);\
             INSERT INTO missing_init_table VALUES (1);",
            &[],
        )
        .await
        .expect_err("invalid initial schema must fail startup");

        assert!(error.to_string().contains("broken-init.sql"));
        let table = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='partial_init'"
                    .to_string(),
            ))
            .await?;
        assert!(table.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn initial_baseline_record_failure_rolls_back_schema() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(
            "CREATE TABLE _migrations (
                 name TEXT PRIMARY KEY,
                 executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
             );
             CREATE TRIGGER reject_initial_baseline
             BEFORE INSERT ON _migrations
             BEGIN
                 SELECT RAISE(ABORT, 'baseline rejected');
             END;",
        )
        .await?;

        apply_initial_schema(
            &db,
            "baseline-failure.sql",
            "CREATE TABLE initial_must_rollback (id INTEGER PRIMARY KEY);",
            &["pending.sql"],
        )
        .await
        .expect_err("baseline record failure must roll back the initial schema");

        assert!(!table_exists(&db, "initial_must_rollback").await?);
        Ok(())
    }

    #[tokio::test]
    async fn existing_historical_schema_skips_initial_schema_and_runs_real_incrementals(
    ) -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(include_str!(
            "../../tests/fixtures/pre_incremental_schema.sql"
        ))
        .await?;
        let all_migrations = crate::migrations::embedded_migration_names();
        assert_eq!(all_migrations, EXPECTED_INCREMENTAL_MIGRATIONS);

        run_migrations(&db).await?;
        assert_eq!(
            crate::migrations::run(&db).await?,
            EXPECTED_INCREMENTAL_MIGRATIONS.len()
        );
        assert_eq!(
            application_schema_fingerprint(&db).await?,
            EXPECTED_UPGRADED_HISTORICAL_SCHEMA_FINGERPRINT,
            "historical table constraints/indexes or migrated definitions changed"
        );
        let ai_task_foreign_keys = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT \"from\", \"table\", \"to\", on_delete
                 FROM pragma_foreign_key_list('ai_tasks') ORDER BY \"from\""
                    .to_string(),
            ))
            .await?
            .into_iter()
            .map(|row| {
                Ok((
                    row.try_get_by_index::<String>(0)?,
                    row.try_get_by_index::<String>(1)?,
                    row.try_get_by_index::<String>(2)?,
                    row.try_get_by_index::<String>(3)?,
                ))
            })
            .collect::<Result<Vec<_>, sea_orm::DbErr>>()?;
        assert_eq!(
            ai_task_foreign_keys,
            vec![
                (
                    "agent_config_id".to_string(),
                    "ai_agent_config".to_string(),
                    "id".to_string(),
                    "SET NULL".to_string(),
                ),
                (
                    "model_id".to_string(),
                    "ai_models".to_string(),
                    "id".to_string(),
                    "SET NULL".to_string(),
                ),
                (
                    "provider_id".to_string(),
                    "ai_providers".to_string(),
                    "id".to_string(),
                    "CASCADE".to_string(),
                ),
                (
                    "skill_id".to_string(),
                    "ai_skills".to_string(),
                    "id".to_string(),
                    "CASCADE".to_string(),
                ),
            ],
            "historical ai_tasks foreign-key contract must survive migrations"
        );
        let provider_contract = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT \"notnull\" FROM pragma_table_info('ai_tasks') WHERE name='provider_id'"
                    .to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<i64>(0).ok());
        assert_eq!(provider_contract, Some(1));
        let legacy_provider_id = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT provider_id FROM ai_tasks WHERE id=1".to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<i64>(0).ok());
        assert_eq!(legacy_provider_id, Some(1));

        for (table, column) in [
            ("categories", "is_visible"),
            ("posts", "cover_image"),
            ("posts", "cover_image_url"),
            ("posts", "cover_image_filename"),
            ("posts", "cover_network_id"),
            ("users", "title"),
            ("changelog", "status"),
            ("guestbook", "email"),
            ("guestbook", "user_id"),
            ("guestbook", "content_html"),
            ("guestbook", "status"),
            ("guestbook", "deleted_at"),
            ("news", "topic_type"),
            ("news", "source_url"),
            ("ai_tasks", "max_tool_rounds"),
        ] {
            let present = db
                .query_one(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    format!(
                        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('{table}') WHERE name='{column}')"
                    ),
                ))
                .await?
                .and_then(|row| row.try_get_by_index::<i64>(0).ok());
            assert_eq!(present, Some(1), "missing {table}.{column}");
        }

        for index_name in ["idx_news_topic_type", "idx_news_source_url"] {
            let present = db
                .query_one(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name=?)",
                    [index_name.into()],
                ))
                .await?
                .and_then(|row| row.try_get_by_index::<i64>(0).ok());
            assert_eq!(present, Some(1), "missing index {index_name}");
        }

        for sql in [
            "SELECT id FROM users WHERE id=1 AND username='legacy-user'",
            "SELECT id FROM categories WHERE id=1 AND name='legacy-category' AND is_visible=1",
            "SELECT id FROM posts WHERE id=1 AND title='legacy-post' AND cover_image IS NULL AND cover_image_url IS NULL AND cover_image_filename IS NULL AND cover_network_id IS NULL",
            "SELECT id FROM changelog WHERE id=1 AND version='0.1.0' AND status='published'",
            "SELECT id FROM guestbook WHERE id=1 AND nickname='legacy-guest' AND content='legacy-message' AND email='' AND user_id IS NULL AND content_html='' AND status='approved' AND deleted_at IS NULL",
            "SELECT id FROM news WHERE id=1 AND title='legacy-news' AND topic_type='' AND source_url=''",
            "SELECT id FROM ai_tasks WHERE id=1 AND name='legacy-task' AND max_tool_rounds IS NULL",
        ] {
            assert!(
                db.query_one(Statement::from_string(DatabaseBackend::Sqlite, sql.to_string()))
                    .await?
                    .is_some(),
                "historical row/default was not preserved: {sql}"
            );
        }

        let guest_copy = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT value FROM settings WHERE key='guest_copy_enabled'".to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<String>(0).ok());
        assert_eq!(guest_copy.as_deref(), Some("false"));

        let recorded = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) FROM _migrations".to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<i64>(0).ok());
        assert_eq!(recorded, Some(all_migrations.len() as i64));
        assert_eq!(crate::migrations::run(&db).await?, 0);
        Ok(())
    }

    #[tokio::test]
    async fn partial_database_is_not_misclassified_or_baselined_as_fresh() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared("CREATE TABLE legacy_partial (id INTEGER PRIMARY KEY)")
            .await?;

        run_migrations(&db).await?;

        let error = crate::migrations::run(&db)
            .await
            .expect_err("a partial legacy database must fail closed during real startup migration");

        assert!(!table_exists(&db, "users").await?);
        assert!(table_exists(&db, "_migrations").await?);
        assert!(table_exists(&db, "legacy_partial").await?);
        assert!(error.to_string().contains("迁移"));
        let recorded = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) FROM _migrations".to_string(),
            ))
            .await?
            .and_then(|row| row.try_get_by_index::<i64>(0).ok());
        assert_eq!(recorded, Some(0));
        Ok(())
    }
}
