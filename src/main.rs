use sea_orm::ConnectionTrait;
use std::net::SocketAddr;
use tracing_subscriber::prelude::*;

use marksharex::{build_router, config::AppConfig, migrations, models, services, utils};

/// 编译时嵌入的默认 Tera 模板
const BUILTIN_TEMPLATES: &[(&str, &str)] = &[
    (
        "default/base.html",
        include_str!("../templates/default/base.html"),
    ),
    (
        "default/index.html",
        include_str!("../templates/default/index.html"),
    ),
    (
        "default/post.html",
        include_str!("../templates/default/post.html"),
    ),
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载.env文件
    let _ = dotenvy::dotenv();

    // 环境变量调试信息 — 仅 debug 模式输出
    #[cfg(debug_assertions)]
    {
        println!("\n📝 环境变量调试:");
        for var in &[
            "MARKSHAREX_DATA_DIR",
            "MARKSHAREX_SERVER_HOST",
            "MARKSHAREX_SERVER_PORT",
            "MARKSHAREX_STORAGE_UPLOAD_DIR",
            "MARKSHAREX_DATABASE_URL",
        ] {
            println!("  {}: {:?}", var, std::env::var(var).ok());
        }
        println!();
    }

    // 启动字符画 — FIGlet standard 字体
    if let Ok(big) = figlet_rs::FIGlet::standard() {
        if let Some(fig) = big.convert("MarkShareX") {
            print!("{}\n", fig.to_string().trim_end());
        }
    }
    println!(
        "Lightweight Self-hosted Markdown Blog System v{} by XLevon\n",
        env!("CARGO_PKG_VERSION")
    );

    // 日志级别：debug 模式输出详细信息，release 模式仅 info+
    let default_filter = if cfg!(debug_assertions) {
        "marksharex=debug,tower_http=debug"
    } else {
        "marksharex=info,tower_http=info"
    };

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| default_filter.into());

    // ── 初始化日志环形缓冲区（必须在 tracing init 前创建）──
    let log_buffer = services::logs::LogBuffer::new(5000);
    let capture_layer = services::logs::LogCaptureLayer::new(log_buffer.clone());

    // 记录进程启动时间
    services::logs::mark_started();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(capture_layer)
        .with(env_filter)
        .init();

    let config = AppConfig::load()?;
    tracing::info!("MarkShareX v{} 启动", env!("CARGO_PKG_VERSION"));

    // 从 config.toml 注入加密密钥到环境变量（crypto.rs 通过 env var 读取）
    if !config.auth.encrypt_key.is_empty() && std::env::var("MARKSHAREX_ENCRYPT_KEY").is_err() {
        std::env::set_var("MARKSHAREX_ENCRYPT_KEY", &config.auth.encrypt_key);
    }

    // 配置值详情 — 仅 debug 模式
    #[cfg(debug_assertions)]
    {
        tracing::debug!("📝 配置详情:");
        tracing::debug!("  数据目录: {}", config.data_dir);
        tracing::debug!("  服务器: {}:{}", config.server.host, config.server.port);
        tracing::debug!("  数据库URL: {}", config.database.url);
        tracing::debug!("  最大连接数: {}", config.database.max_connections);
        tracing::debug!("  最小连接数: {}", config.database.min_connections);
        tracing::debug!("  JWT过期: {}s", config.auth.jwt_expire_seconds);
        tracing::debug!("  刷新令牌过期: {}s", config.auth.refresh_expire_seconds);
        tracing::debug!("  上传目录: {}", config.storage.upload_dir);
        tracing::debug!("  最大文件大小: {}字节", config.storage.max_file_size);
        tracing::debug!("  允许文件类型: {:?}", config.storage.allowed_types);
    }

    std::fs::create_dir_all(&config.data_dir)?;
    std::fs::create_dir_all(&config.storage.upload_dir)?;

    // ── 启动时用内嵌模板覆盖运行时目录 ──
    let templates_target = format!("{}/templates", config.data_dir);
    for (rel_path, content) in BUILTIN_TEMPLATES {
        let dest = std::path::Path::new(&templates_target).join(rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, content)?;
    }
    tracing::info!("默认模板已更新到 {}", templates_target);

    let db = models::init_db(&config.database).await?;
    tracing::info!("数据库连接成功");

    models::run_migrations(&db).await?;
    tracing::info!("数据库迁移完成");

    // ── 文件迁移：执行 migrations/ 目录下所有待执行的 SQL ──
    let file_migration_count = migrations::run(&db).await?;
    if file_migration_count > 0 {
        tracing::info!("文件迁移完成（{} 个新迁移）", file_migration_count);
    }

    let search_engine = services::search::init_index(&config.data_dir)?;
    tracing::info!("搜索引擎初始化完成");

    // Compare DB published post count vs index document count
    let db_count: i64 = db
        .query_all(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "SELECT COUNT(*) FROM posts WHERE status = 'published' AND deleted_at IS NULL"
                .to_string(),
        ))
        .await?
        .first()
        .map(|row| row.try_get_by_index::<i64>(0).unwrap_or(0))
        .unwrap_or(0);

    let reader = search_engine.index.reader()?;
    let searcher = reader.searcher();
    let index_count = searcher.num_docs() as i64;

    if index_count == 0 {
        tracing::info!("索引为空，从数据库重建 (DB 有 {} 篇)...", db_count);
        services::search::reindex_all_posts(&search_engine, &db).await?;
    } else if db_count != index_count {
        tracing::info!(
            "索引与数据库不一致 (索引 {} 篇, DB {} 篇)，自动重建...",
            index_count,
            db_count
        );
        services::search::reindex_all_posts(&search_engine, &db).await?;
    } else {
        tracing::info!("索引与数据库一致 ({} 篇)，跳过重建", index_count);
    }

    // ── 启动时迁移 IP 设置数据格式 ──
    utils::ip_migration::migrate_ip_settings_format(&db).await;

    let state = utils::AppState::new(db, config.clone(), search_engine, log_buffer);

    // ── 启动 AI 定时调度器 ──
    let scheduler = services::ai_scheduler::AiScheduler::new(std::sync::Arc::new(state.clone()));
    tokio::spawn(async move { scheduler.start().await });

    let app = build_router(state)?;

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("🚀 MarkShareX 运行在 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
