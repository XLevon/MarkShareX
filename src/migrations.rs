use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};

/// 文件迁移表的建表语句
const CREATE_MIGRATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS _migrations (
    name TEXT PRIMARY KEY,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
)"#;

fn sql_tokens_outside_literals_and_comments(sql: &str) -> Vec<String> {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        SingleQuote,
        DoubleQuote,
        Backtick,
        Bracket,
        LineComment,
        BlockComment,
    }

    let chars: Vec<char> = sql.chars().collect();
    let mut state = State::Normal;
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut index = 0;

    let flush = |token: &mut String, tokens: &mut Vec<String>| {
        if !token.is_empty() {
            tokens.push(std::mem::take(token).to_ascii_uppercase());
        }
    };

    while index < chars.len() {
        let current = chars[index];
        let next = chars.get(index + 1).copied();
        match state {
            State::Normal => match (current, next) {
                ('-', Some('-')) => {
                    flush(&mut token, &mut tokens);
                    state = State::LineComment;
                    index += 1;
                }
                ('/', Some('*')) => {
                    flush(&mut token, &mut tokens);
                    state = State::BlockComment;
                    index += 1;
                }
                ('\'', _) => {
                    flush(&mut token, &mut tokens);
                    state = State::SingleQuote;
                }
                ('"', _) => {
                    flush(&mut token, &mut tokens);
                    state = State::DoubleQuote;
                }
                ('`', _) => {
                    flush(&mut token, &mut tokens);
                    state = State::Backtick;
                }
                ('[', _) => {
                    flush(&mut token, &mut tokens);
                    state = State::Bracket;
                }
                _ if current.is_ascii_alphanumeric() || current == '_' => token.push(current),
                _ => {
                    flush(&mut token, &mut tokens);
                    if matches!(current, ';' | '(' | ')' | ',') {
                        tokens.push(current.to_string());
                    }
                }
            },
            State::SingleQuote => {
                if current == '\'' {
                    if next == Some('\'') {
                        index += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::DoubleQuote => {
                if current == '"' {
                    if next == Some('"') {
                        index += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::Backtick => {
                if current == '`' {
                    state = State::Normal;
                }
            }
            State::Bracket => {
                if current == ']' {
                    state = State::Normal;
                }
            }
            State::LineComment => {
                if current == '\n' {
                    state = State::Normal;
                }
            }
            State::BlockComment => {
                if current == '*' && next == Some('/') {
                    state = State::Normal;
                    index += 1;
                }
            }
        }
        index += 1;
    }
    flush(&mut token, &mut tokens);
    tokens
}

pub(crate) fn reject_transaction_control(sql: &str) -> anyhow::Result<()> {
    let tokens = sql_tokens_outside_literals_and_comments(sql);
    let mut index = 0;
    let mut statement_start = true;
    let mut in_trigger = false;
    let mut trigger_body_started = false;
    let mut trigger_case_depth = 0usize;

    while index < tokens.len() {
        let token = tokens[index].as_str();

        if in_trigger {
            if token == "BEGIN" && !trigger_body_started {
                trigger_body_started = true;
            } else if trigger_body_started && token == "CASE" {
                trigger_case_depth += 1;
            } else if trigger_body_started && token == "END" {
                if trigger_case_depth > 0 {
                    trigger_case_depth -= 1;
                } else if tokens.get(index + 1).map(String::as_str) == Some(";") {
                    in_trigger = false;
                    trigger_body_started = false;
                    statement_start = true;
                    index += 2;
                    continue;
                }
            }
            index += 1;
            continue;
        }

        if token == ";" {
            statement_start = true;
            index += 1;
            continue;
        }

        if statement_start {
            let creates_trigger = token == "CREATE"
                && (tokens.get(index + 1).map(String::as_str) == Some("TRIGGER")
                    || (matches!(
                        tokens.get(index + 1).map(String::as_str),
                        Some("TEMP") | Some("TEMPORARY")
                    ) && tokens.get(index + 2).map(String::as_str) == Some("TRIGGER")));
            if creates_trigger {
                in_trigger = true;
                statement_start = false;
                index += 1;
                continue;
            }

            if matches!(
                token,
                "BEGIN" | "COMMIT" | "END" | "ROLLBACK" | "SAVEPOINT" | "RELEASE"
            ) {
                anyhow::bail!("迁移脚本不得包含事务控制语句: {token}");
            }
            statement_start = false;
        }

        index += 1;
    }
    Ok(())
}

async fn migration_was_recorded(
    db: &impl ConnectionTrait,
    file_name: &str,
) -> anyhow::Result<bool> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?)",
            [file_name.into()],
        ))
        .await?;
    Ok(row
        .and_then(|row| row.try_get_by_index::<i64>(0).ok())
        .is_some_and(|exists| exists == 1))
}

async fn apply_migration(
    db: &DatabaseConnection,
    file_name: &str,
    sql: &str,
) -> anyhow::Result<()> {
    reject_transaction_control(sql)
        .map_err(|error| anyhow::anyhow!("迁移 {file_name} 失败: {error}"))?;
    let txn = db.begin().await?;
    if !sql.trim().is_empty() {
        if let Err(error) = txn.execute_unprepared(sql).await {
            txn.rollback().await.ok();
            return Err(anyhow::anyhow!("迁移 {file_name} 失败: {error}"));
        }
    }
    if let Err(error) = txn
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO _migrations (name) VALUES (?)",
            [file_name.into()],
        ))
        .await
    {
        txn.rollback().await.ok();
        return Err(anyhow::anyhow!("记录迁移 {file_name} 失败: {error}"));
    }
    txn.commit()
        .await
        .map_err(|error| anyhow::anyhow!("提交迁移 {file_name} 失败: {error}"))?;
    Ok(())
}

// 嵌入所有增量迁移文件（编译时由 build.rs 生成）
include!(concat!(env!("OUT_DIR"), "/embedded_migrations.rs"));

pub(crate) fn embedded_migration_names() -> Vec<&'static str> {
    MIGRATIONS.iter().map(|(name, _)| *name).collect()
}

pub(crate) async fn record_migration_names(
    db: &impl ConnectionTrait,
    names: &[&str],
) -> anyhow::Result<()> {
    db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;
    for name in names {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT OR IGNORE INTO _migrations (name) VALUES (?)",
            [(*name).into()],
        ))
        .await?;
    }
    Ok(())
}

async fn run_migration_set(
    db: &DatabaseConnection,
    migrations: &[(&str, &str)],
) -> anyhow::Result<usize> {
    db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;
    let mut count = 0;

    for (file_name, sql) in migrations {
        if migration_was_recorded(db, file_name).await? {
            continue;
        }

        tracing::info!("🔄 执行迁移: {}", file_name);
        apply_migration(db, file_name, sql).await?;
        tracing::info!("✅ 迁移完成: {}", file_name);
        count += 1;
    }

    Ok(count)
}

/// 执行所有嵌入在二进制中的增量迁移。
///
/// 每个迁移脚本作为完整 SQLite batch 在独立事务中执行，避免错误拆分
/// trigger body 或字符串中的分号。脚本与 `_migrations` 记录原子提交。
pub async fn run(db: &DatabaseConnection) -> anyhow::Result<usize> {
    if MIGRATIONS.is_empty() {
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;
        tracing::info!("📦 没有待执行的迁移文件");
        return Ok(0);
    }

    let count = run_migration_set(db, MIGRATIONS).await?;
    if count > 0 {
        tracing::info!("📦 共执行 {} 个迁移", count);
    } else {
        tracing::info!("📦 所有迁移已是最新");
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};

    #[tokio::test]
    async fn migration_batch_preserves_trigger_bodies_and_semicolons_in_strings(
    ) -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;
        let sql = r#"
            CREATE TABLE source (id INTEGER PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE audit (message TEXT NOT NULL);
            CREATE TRIGGER source_audit AFTER INSERT ON source BEGIN
                INSERT INTO audit(message) VALUES ('first;part');
                INSERT INTO audit(message) VALUES (NEW.value || ';second');
            END;
            INSERT INTO source(value) VALUES ('payload;value');
        "#;

        apply_migration(&db, "complex.sql", sql).await?;

        let rows = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT message FROM audit ORDER BY rowid".to_string(),
            ))
            .await?;
        let messages = rows
            .iter()
            .map(|row| row.try_get_by_index::<String>(0))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(messages, vec!["first;part", "payload;value;second"]);
        assert!(migration_was_recorded(&db, "complex.sql").await?);
        Ok(())
    }

    #[tokio::test]
    async fn failed_migration_rolls_back_every_statement_and_is_not_recorded() -> anyhow::Result<()>
    {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;

        let error = apply_migration(
            &db,
            "broken.sql",
            "CREATE TABLE partial (id INTEGER PRIMARY KEY);\
             INSERT INTO partial VALUES (1);\
             INSERT INTO missing_table VALUES (1);",
        )
        .await
        .expect_err("broken migration must fail closed");

        assert!(error.to_string().contains("broken.sql"));
        let table = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='partial'".to_string(),
            ))
            .await?;
        assert!(table.is_none());
        assert!(!migration_was_recorded(&db, "broken.sql").await?);
        Ok(())
    }

    #[tokio::test]
    async fn recorded_migration_is_skipped_on_repeated_runs() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        let migrations = &[(
            "once.sql",
            "CREATE TABLE once_only (id INTEGER PRIMARY KEY);",
        )];

        assert_eq!(run_migration_set(&db, migrations).await?, 1);
        assert_eq!(run_migration_set(&db, migrations).await?, 0);
        assert!(migration_was_recorded(&db, "once.sql").await?);
        Ok(())
    }

    #[tokio::test]
    async fn transaction_control_cannot_escape_the_migration_transaction() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;

        let error = apply_migration(
            &db,
            "transaction-escape.sql",
            "CREATE TABLE escaped_commit (id INTEGER PRIMARY KEY);\
             COMMIT;\
             INSERT INTO missing_after_commit VALUES (1);",
        )
        .await
        .expect_err("migration transaction-control statements must be rejected");

        assert!(error.to_string().contains("transaction-escape.sql"));
        let table = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='escaped_commit'"
                    .to_string(),
            ))
            .await?;
        assert!(table.is_none());
        assert!(!migration_was_recorded(&db, "transaction-escape.sql").await?);
        Ok(())
    }

    #[test]
    fn transaction_control_guard_allows_trigger_syntax_literals_and_comments() {
        reject_transaction_control(
            "-- COMMIT and ROLLBACK are documentation here
             CREATE TRIGGER guarded_trigger AFTER INSERT ON source BEGIN
                 SELECT RAISE(ROLLBACK, 'COMMIT; is text') WHERE NEW.id < 0;
                 INSERT INTO audit(message) VALUES ('SAVEPOINT; RELEASE; BEGIN;');
             END;",
        )
        .expect("trigger syntax, RAISE(ROLLBACK), comments and literals must remain valid");

        reject_transaction_control(
            "CREATE TABLE allowed_identifiers (
                savepoint TEXT,
                release TEXT,
                rollback TEXT,
                commit_value TEXT
             );
             INSERT INTO allowed_identifiers (savepoint, release, rollback, commit_value)
             VALUES ('one', 'two', 'three', 'four');
             SELECT rollback FROM allowed_identifiers;",
        )
        .expect("transaction keywords used as non-leading identifiers must remain valid");

        reject_transaction_control(
            "CREATE TEMP TRIGGER case_trigger AFTER INSERT ON source BEGIN
                 INSERT INTO audit(message)
                 VALUES (CASE WHEN NEW.id > 0 THEN 'ok' ELSE 'fallback' END);
             END;
             CREATE TABLE after_trigger (savepoint TEXT);",
        )
        .expect("CASE END inside a trigger must not be mistaken for the trigger terminator");
    }

    #[tokio::test]
    async fn temp_trigger_with_nested_case_executes_inside_a_real_migration() -> anyhow::Result<()>
    {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;

        apply_migration(
            &db,
            "temp-trigger-nested-case.sql",
            r#"
            CREATE TABLE source (id INTEGER PRIMARY KEY);
            CREATE TABLE audit (message TEXT NOT NULL);
            CREATE TEMP TRIGGER case_trigger AFTER INSERT ON source BEGIN
                INSERT INTO audit(message)
                VALUES (
                    CASE
                        WHEN NEW.id > 0 THEN
                            CASE WHEN NEW.id = 1 THEN 'one' ELSE 'positive' END
                        ELSE 'fallback'
                    END
                );
            END;
            INSERT INTO source(id) VALUES (1);
            INSERT INTO source(id) VALUES (2);
            INSERT INTO source(id) VALUES (-1);
            "#,
        )
        .await?;

        let messages = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT message FROM audit ORDER BY rowid".to_string(),
            ))
            .await?
            .into_iter()
            .filter_map(|row| row.try_get_by_index::<String>(0).ok())
            .collect::<Vec<_>>();
        assert_eq!(messages, vec!["one", "positive", "fallback"]);
        assert!(migration_was_recorded(&db, "temp-trigger-nested-case.sql").await?);
        Ok(())
    }

    #[test]
    fn transaction_control_guard_rejects_all_sqlite_transaction_commands() {
        for sql in [
            "BEGIN TRANSACTION; SELECT 1;",
            "BEGIN IMMEDIATE; SELECT 1;",
            "ROLLBACK;",
            "SAVEPOINT nested;",
            "RELEASE nested;",
            "END TRANSACTION;",
            "END;",
            "END",
        ] {
            assert!(reject_transaction_control(sql).is_err(), "accepted {sql}");
        }
    }

    #[tokio::test]
    async fn bare_end_cannot_commit_before_a_later_migration_failure() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;

        apply_migration(
            &db,
            "bare-end-escape.sql",
            "CREATE TABLE escaped_end (id INTEGER PRIMARY KEY);
             END;
             INSERT INTO missing_after_end VALUES (1);",
        )
        .await
        .expect_err("bare END must be rejected before execution");

        assert!(db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='escaped_end'"
                    .to_string(),
            ))
            .await?
            .is_none());
        assert!(!migration_was_recorded(&db, "bare-end-escape.sql").await?);
        Ok(())
    }

    #[tokio::test]
    async fn migration_record_failure_rolls_back_schema_changes() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(CREATE_MIGRATIONS_TABLE).await?;
        db.execute_unprepared(
            "CREATE TRIGGER reject_migration_record
             BEFORE INSERT ON _migrations
             BEGIN
                 SELECT RAISE(ABORT, 'migration record rejected');
             END;",
        )
        .await?;

        apply_migration(
            &db,
            "record-failure.sql",
            "CREATE TABLE must_rollback (id INTEGER PRIMARY KEY);",
        )
        .await
        .expect_err("migration record failure must fail the migration");

        let table = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='must_rollback'"
                    .to_string(),
            ))
            .await?;
        assert!(table.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn empty_migration_is_recorded_once() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        let migrations = &[("empty.sql", "   \n\t")];

        assert_eq!(run_migration_set(&db, migrations).await?, 1);
        assert_eq!(run_migration_set(&db, migrations).await?, 0);
        assert!(migration_was_recorded(&db, "empty.sql").await?);
        Ok(())
    }

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
