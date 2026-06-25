use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;
use sea_orm::ConnectionTrait;
use crate::utils::{AppState, AppError, ApiResponse};
use crate::middleware::auth::AuthUser;

// ── Logs ──

#[derive(Serialize, ToSchema)]
pub struct LogQueryResponse {
    pub logs: Vec<crate::services::logs::LogEntry>,
    pub total_cached: usize,
}

#[derive(serde::Deserialize)]
pub struct LogQuery {
    pub level: Option<String>,
    pub limit: Option<usize>,
    pub search: Option<String>,
}

/// GET /api/v1/admin/logs
///
/// 获取应用运行日志（仅主管理员可访问）。
/// 支持 `?level=ERROR&limit=50&search=timeout` 过滤。
#[utoipa::path(
    get,
    path = "/api/v1/admin/logs",
    tag = "Ops",
    params(
        ("level" = Option<String>, Query, description = "日志级别过滤（ERROR/WARN/INFO/DEBUG）"),
        ("limit" = Option<usize>, Query, description = "返回条数（默认 100）"),
        ("search" = Option<String>, Query, description = "关键词搜索")
    ),
    responses((status = 200, body = ApiResponse<LogQueryResponse>)),
    security(("api_key" = []), ("bearer" = []))
)]
pub async fn get_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<LogQuery>,
) -> Result<Json<ApiResponse<LogQueryResponse>>, AppError> {
    require_admin(&auth)?;

    let limit = query.limit.unwrap_or(100).min(1000);
    let logs = state.log_buffer.query(query.level.as_deref(), limit, query.search.as_deref());

    Ok(Json(ApiResponse::new(LogQueryResponse {
        total_cached: {
            // Quick count of all entries
            state.log_buffer.query(None, 10000, None).len()
        },
        logs,
    })))
}

// ── Health ──

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,        // "ok" or "degraded"
    pub version: String,
    pub uptime_seconds: u64,
    pub database: DatabaseHealth,
    pub system: SystemHealth,
}

#[derive(Serialize, ToSchema)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub migration_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct SystemHealth {
    pub disk_used_percent: f64,
    pub disk_total_gb: f64,
    pub disk_free_gb: f64,
    pub memory_used_percent: f64,
    pub memory_total_mb: u64,
    pub memory_available_mb: u64,
}

/// GET /api/v1/admin/health
///
/// 系统健康检查：DB 连接、磁盘、内存、运行时间（仅主管理员可访问）。
#[utoipa::path(
    get,
    path = "/api/v1/admin/health",
    tag = "Ops",
    responses((status = 200, body = ApiResponse<HealthResponse>)),
    security(("api_key" = []), ("bearer" = []))
)]
pub async fn get_health(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<HealthResponse>>, AppError> {
    require_admin(&auth)?;

    // ── DB ping ──
    let db_ok = state.db
        .execute_unprepared("SELECT 1")
        .await
        .is_ok();

    // ── Uptime ──
    let uptime = crate::services::logs::uptime_seconds();

    // ── Disk ──
    let (disk_total_gb, disk_free_gb, disk_used_pct) = get_disk_info();

    // ── Memory ──
    let (mem_total_mb, mem_avail_mb, mem_used_pct) = get_memory_info();

    // ── Migration count ──
    let migration_count = state.db
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT COUNT(*) as cnt FROM _migrations",
            [],
        ))
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get_by::<i64, _>("cnt").ok())
        .unwrap_or(0) as usize;

    let status = if db_ok { "ok" } else { "degraded" };

    Ok(Json(ApiResponse::new(HealthResponse {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        database: DatabaseHealth {
            connected: db_ok,
            migration_count,
        },
        system: SystemHealth {
            disk_used_percent: disk_used_pct,
            disk_total_gb,
            disk_free_gb,
            memory_used_percent: mem_used_pct,
            memory_total_mb: mem_total_mb,
            memory_available_mb: mem_avail_mb,
        },
    })))
}

// ── Stats ──

#[derive(Serialize, ToSchema)]
pub struct StatsResponse {
    pub db_max_connections: u32,
    pub db_min_connections: u32,
    pub recent_errors: Vec<crate::services::logs::LogEntry>,
}

/// GET /api/v1/admin/stats
///
/// 系统统计信息（仅主管理员可访问）。
#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    tag = "Ops",
    responses((status = 200, body = ApiResponse<StatsResponse>)),
    security(("api_key" = []), ("bearer" = []))
)]
pub async fn get_stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<StatsResponse>>, AppError> {
    require_admin(&auth)?;

    let recent_errors = state.log_buffer.recent_errors(10);

    Ok(Json(ApiResponse::new(StatsResponse {
        db_max_connections: state.config.database.max_connections,
        db_min_connections: state.config.database.min_connections,
        recent_errors,
    })))
}

// ── Helpers ──

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if !auth.is_admin() {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

fn get_disk_info() -> (f64, f64, f64) {
    // Use data dir's filesystem — parse df output
    let path = std::env::current_dir().unwrap_or_else(|_| "/".into());
    let path_str = path.to_string_lossy();
    if let Ok(output) = std::process::Command::new("df")
        .arg("-B1")  // bytes
        .arg(path_str.as_ref())
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // df output: Filesystem 1K-blocks Used Available Use% Mounted
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total: f64 = parts[1].parse().unwrap_or(0.0);
                let used: f64 = parts[2].parse().unwrap_or(0.0);
                let free: f64 = parts[3].parse().unwrap_or(0.0);
                let pct = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };
                return (total / 1e9, free / 1e9, (pct * 10.0).round() / 10.0);
            }
        }
    }
    (0.0, 0.0, 0.0)
}

fn get_memory_info() -> (u64, u64, f64) {
    let mut total = 0u64;
    let mut avail = 0u64;

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_kb(line);
            } else if line.starts_with("MemAvailable:") {
                avail = parse_kb(line);
            }
        }
    }

    let used_pct = if total > 0 {
        ((total - avail) as f64 / total as f64 * 1000.0).round() / 10.0
    } else {
        0.0
    };

    (total / 1024, avail / 1024, used_pct)
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}
