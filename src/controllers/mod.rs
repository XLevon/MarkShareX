pub mod admin;
pub mod ai;
pub mod analytics;
pub mod article_kb;
pub mod auth;
pub mod author_applications;
pub mod categories;
pub mod changelog;
pub mod comments;
pub mod csp;
pub mod files;
pub mod guestbook;
pub mod import_export;
pub mod network_resources;
pub mod news;
pub mod ops;
pub mod pages;
pub mod posts;
pub mod profile;
pub mod settings;
pub mod setup;
pub mod tags;

use crate::api_doc::ApiDoc;
use crate::middleware::auth::require_admin_middleware;
use crate::utils::{ApiResponse, AppState};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

// ── 端点发现 ──

#[derive(Serialize)]
struct EndpointInfo {
    method: &'static str,
    path: &'static str,
    description: &'static str,
    auth_required: bool,
}

/// GET /api/v1/ — 返回所有 API 路由元数据，供 AI 工具自举发现
async fn list_endpoints() -> Json<ApiResponse<Vec<EndpointInfo>>> {
    let endpoints = vec![
        // Health
        EndpointInfo {
            method: "GET",
            path: "/api/v1/",
            description: "获取所有 API 端点列表",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/health",
            description: "健康检查",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/version",
            description: "获取版本信息",
            auth_required: false,
        },
        // Auth
        EndpointInfo {
            method: "POST",
            path: "/api/v1/auth/login",
            description: "用户登录",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/auth/register",
            description: "用户注册",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/auth/refresh",
            description: "刷新令牌",
            auth_required: false,
        },
        // Posts (public)
        EndpointInfo {
            method: "GET",
            path: "/api/v1/posts",
            description: "获取文章列表（前台公开）",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/posts/slug/:slug",
            description: "根据 slug 获取文章",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/posts/:id",
            description: "根据 ID 获取文章",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/posts/:id/adjacent",
            description: "获取文章的前后篇",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/posts/:id/like",
            description: "点赞文章",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/posts/:id/like-status",
            description: "查询点赞状态",
            auth_required: false,
        },
        // Posts (auth required)
        EndpointInfo {
            method: "POST",
            path: "/api/v1/posts",
            description: "创建文章",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/posts/:id",
            description: "更新文章",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/posts/:id",
            description: "删除文章",
            auth_required: true,
        },
        // Comments
        EndpointInfo {
            method: "GET",
            path: "/api/v1/posts/:id/comments",
            description: "获取文章评论",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/posts/:id/comments",
            description: "发表评论",
            auth_required: false,
        },
        // Categories
        EndpointInfo {
            method: "GET",
            path: "/api/v1/categories",
            description: "获取分类列表",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/categories",
            description: "创建分类",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/categories/:id",
            description: "更新分类",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/categories/:id",
            description: "删除分类",
            auth_required: true,
        },
        // Tags
        EndpointInfo {
            method: "GET",
            path: "/api/v1/tags",
            description: "获取标签列表",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/tags",
            description: "创建标签",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/tags/:id",
            description: "更新标签",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/tags/:id",
            description: "删除标签",
            auth_required: true,
        },
        // Files
        EndpointInfo {
            method: "GET",
            path: "/api/v1/files",
            description: "获取文件列表",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/files/upload",
            description: "上传文件",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/files/batch",
            description: "批量上传文件",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/files/batch",
            description: "批量删除文件",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/files/check-md5",
            description: "检查文件 MD5 是否存在",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/files/:id",
            description: "删除文件",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/files/unreferenced",
            description: "获取未引用的文件",
            auth_required: true,
        },
        // Settings
        EndpointInfo {
            method: "GET",
            path: "/api/v1/settings",
            description: "获取站点设置",
            auth_required: false,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/settings",
            description: "更新站点设置（含 api_key）",
            auth_required: true,
        },
        // Setup
        EndpointInfo {
            method: "GET",
            path: "/api/v1/setup/status",
            description: "检查初始化状态",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/setup",
            description: "执行初始化",
            auth_required: false,
        },
        // Authors / Applications
        EndpointInfo {
            method: "GET",
            path: "/api/v1/authors",
            description: "获取作者列表",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/apply",
            description: "申请成为作者",
            auth_required: false,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/apply/status",
            description: "查询申请状态",
            auth_required: false,
        },
        // Search
        EndpointInfo {
            method: "GET",
            path: "/api/v1/search",
            description: "全文搜索",
            auth_required: false,
        },
        // Profile
        EndpointInfo {
            method: "GET",
            path: "/api/v1/profile",
            description: "获取个人资料",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/profile",
            description: "更新个人资料",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/profile/password",
            description: "修改密码",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/profile/api-key",
            description: "获取当前用户的 API Key",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/profile/api-key",
            description: "生成/重置 API Key",
            auth_required: true,
        },
        // Analytics
        EndpointInfo {
            method: "GET",
            path: "/api/v1/analytics/trend",
            description: "访问趋势统计",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/analytics/total-views",
            description: "总浏览量",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/analytics/total-likes",
            description: "总点赞数",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/analytics/total-comments",
            description: "总评论数",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/analytics/post-views",
            description: "文章浏览量排行",
            auth_required: true,
        },
        // Import/Export
        EndpointInfo {
            method: "POST",
            path: "/api/v1/export/posts",
            description: "导出文章",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/import/posts",
            description: "导入文章",
            auth_required: true,
        },
        // Network Resources
        EndpointInfo {
            method: "GET",
            path: "/api/v1/network-resources",
            description: "网络资源列表",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/network-resources",
            description: "添加网络资源",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/network-resources/:id",
            description: "更新网络资源",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/network-resources/:id",
            description: "删除网络资源",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/network-resources/:id/references",
            description: "查询网络资源引用清单",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/network-resources/:id/resolve",
            description: "解析资源 ID → 302 重定向",
            auth_required: false,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/network-resources/ensure",
            description: "确保 URL 已入库（自动查重）",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/network-resources/batch-resolve",
            description: "批量解析 ID → URL",
            auth_required: true,
        },
        // Admin
        EndpointInfo {
            method: "GET",
            path: "/api/v1/admin/posts",
            description: "管理端文章列表",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/admin/users",
            description: "用户列表",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/admin/users",
            description: "创建用户",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/admin/users/:id",
            description: "更新用户",
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE",
            path: "/api/v1/admin/users/:id",
            description: "删除用户",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/admin/users/:id/status",
            description: "更新用户状态",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/admin/users/:id/role",
            description: "更新用户角色",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/admin/comments",
            description: "管理端评论列表",
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT",
            path: "/api/v1/admin/comments/:id",
            description: "更新评论状态",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/admin/likes",
            description: "管理端点赞记录",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/admin/applications/pending-count",
            description: "待审批数量",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/admin/applications/:id/approve",
            description: "审批通过",
            auth_required: true,
        },
        EndpointInfo {
            method: "POST",
            path: "/api/v1/admin/applications/:id/reject",
            description: "审批拒绝",
            auth_required: true,
        },
        EndpointInfo {
            method: "GET",
            path: "/api/v1/site/admin-info",
            description: "获取管理员信息",
            auth_required: false,
        },
        // CSP violation report
        EndpointInfo {
            method: "POST",
            path: "/api/v1/csp-report",
            description: "CSP 违规报告",
            auth_required: false,
        },
        // AI
        EndpointInfo {
            method: "GET",
            path: "/api/v1/ai/default-agent",
            description: "获取默认 AI 代理配置",
            auth_required: false,
        },
    ];
    Json(ApiResponse::new(endpoints))
}

pub fn api_routes(state: AppState) -> Router<AppState> {
    // 构建 OpenAPI spec（编译时静态生成，运行时只序列化）
    let openapi = ApiDoc::openapi();

    Router::new()
        // OpenAPI 文档 — AI agent 可消费的标准接口描述
        .route(
            "/api/v1/openapi.json",
            get({
                let openapi = openapi.clone();
                move || async move { axum::Json(openapi.clone()) }
            }),
        )
        // Scalar UI — 交互式 API 文档（仅管理员可访问 /scalar）
        .merge({
            let scalar = Router::<AppState>::new().merge(Scalar::with_url("/scalar", openapi));
            scalar.route_layer(middleware::from_fn_with_state(
                state.clone(),
                require_admin_middleware,
            ))
        })
        // API discovery
        .route("/api/v1/", get(list_endpoints))
        // Health check
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/version", get(version))
        // CSP violation report
        .route("/api/v1/csp-report", post(csp::csp_report_handler))
        // Auth
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        // Posts - slug route must come BEFORE {id} route to avoid path capture conflict
        .route(
            "/api/v1/posts",
            get(posts::list_posts).post(posts::create_post),
        )
        .route("/api/v1/posts/slug/:slug", get(posts::get_post_by_slug))
        .route(
            "/api/v1/posts/:id",
            get(posts::get_post)
                .put(posts::update_post)
                .delete(posts::delete_post),
        )
        .route("/api/v1/posts/:id/adjacent", get(posts::get_adjacent_posts))
        .route("/api/v1/posts/:id/like", post(posts::toggle_like))
        .route("/api/v1/posts/:id/like-status", get(posts::get_like_status))
        // Comments (public)
        .route(
            "/api/v1/posts/:id/comments",
            get(comments::list_post_comments).post(comments::create_comment),
        )
        // Categories
        .route(
            "/api/v1/categories",
            get(categories::list_categories).post(categories::create_category),
        )
        .route(
            "/api/v1/categories/:id",
            put(categories::update_category).delete(categories::delete_category),
        )
        .route(
            "/api/v1/admin/categories/reorder",
            put(categories::reorder_categories),
        )
        .route(
            "/api/v1/admin/categories",
            get(categories::list_admin_categories),
        )
        // Tags
        .route("/api/v1/tags", get(tags::list_tags).post(tags::create_tag))
        .route(
            "/api/v1/tags/:id",
            put(tags::update_tag).delete(tags::delete_tag),
        )
        // Authors
        .route("/api/v1/authors", get(posts::list_authors))
        // Search
        .route("/api/v1/search", get(posts::unified_search))
        // Files
        .route("/api/v1/files/upload", post(files::upload_file))
        .route("/api/v1/files/batch", post(files::batch_upload))
        .route("/api/v1/files/check-md5", post(files::check_md5_exists))
        .route("/api/v1/files", get(files::list_files))
        .route(
            "/api/v1/files/unreferenced",
            get(files::list_unreferenced_files),
        )
        .route("/api/v1/files/:id", delete(files::delete_file))
        .route("/api/v1/files/batch", delete(files::batch_delete_files))
        // Settings
        .route(
            "/api/v1/settings",
            get(settings::get_settings).put(settings::update_settings),
        )
        // Article types & statuses (public)
        .route("/api/v1/article-types", get(article_kb::list_article_types))
        .route(
            "/api/v1/article-statuses",
            get(article_kb::list_article_statuses),
        )
        // Article types & statuses (admin)
        .route(
            "/api/v1/admin/article-types",
            get(article_kb::list_admin_article_types).post(article_kb::create_article_type),
        )
        .route(
            "/api/v1/admin/article-types/:id",
            put(article_kb::update_article_type).delete(article_kb::delete_article_type),
        )
        .route(
            "/api/v1/admin/article-types/reorder",
            post(article_kb::reorder_article_types),
        )
        .route(
            "/api/v1/admin/article-statuses",
            get(article_kb::list_admin_article_statuses).post(article_kb::create_article_status),
        )
        .route(
            "/api/v1/admin/article-statuses/:id",
            put(article_kb::update_article_status).delete(article_kb::delete_article_status),
        )
        .route(
            "/api/v1/admin/article-statuses/reorder",
            post(article_kb::reorder_article_statuses),
        )
        // Guestbook (public)
        .route(
            "/api/v1/guestbook",
            get(guestbook::list_entries).post(guestbook::create_entry),
        )
        .route(
            "/api/v1/admin/guestbook/:id/reply",
            put(guestbook::reply_entry),
        )
        .route(
            "/api/v1/admin/guestbook/:id",
            delete(guestbook::delete_entry),
        )
        // Site public info
        .route(
            "/api/v1/site/admin-info",
            get(profile::get_site_manager_info),
        )
        // Setup
        .route("/api/v1/setup/status", get(setup::setup_status))
        .route("/api/v1/setup", post(setup::setup))
        // Author applications (user)
        .route(
            "/api/v1/apply",
            post(author_applications::submit_application),
        )
        .route(
            "/api/v1/apply/status",
            get(author_applications::get_application_status),
        )
        // Author applications (admin)
        .route(
            "/api/v1/admin/applications/pending-count",
            get(author_applications::get_pending_count),
        )
        .route(
            "/api/v1/admin/applications/:id/approve",
            post(author_applications::approve_application),
        )
        .route(
            "/api/v1/admin/applications/:id/reject",
            post(author_applications::reject_application),
        )
        // Admin posts
        .route("/api/v1/admin/posts", get(posts::list_admin_posts))
        .route(
            "/api/v1/admin/posts/batch-delete",
            post(posts::batch_delete_posts),
        )
        .route(
            "/api/v1/admin/posts/batch-publish",
            post(posts::batch_publish_posts),
        )
        .route(
            "/api/v1/admin/posts/batch-unpublish",
            post(posts::batch_unpublish_posts),
        )
        .route(
            "/api/v1/admin/posts/pin-order",
            put(posts::update_pin_order),
        )
        .route("/api/v1/admin/posts/:id/pin", post(posts::pin_post))
        .route("/api/v1/admin/posts/:id/unpin", post(posts::unpin_post))
        // Pinned posts (public)
        .route("/api/v1/posts/pinned", get(posts::list_pinned_posts))
        .route(
            "/api/v1/admin/users",
            get(admin::list_users).post(admin::create_user),
        )
        .route(
            "/api/v1/admin/users/:id/status",
            put(admin::update_user_status),
        )
        .route("/api/v1/admin/users/:id/role", put(admin::update_user_role))
        .route("/api/v1/admin/users/:id/reset-password", put(admin::reset_user_password))
        .route(
            "/api/v1/admin/users/:id",
            put(admin::update_user).delete(admin::delete_user),
        )
        // Profile
        .route(
            "/api/v1/profile",
            get(profile::get_profile).put(profile::update_profile),
        )
        .route("/api/v1/profile/password", put(profile::change_password))
        .route(
            "/api/v1/profile/api-key",
            get(profile::get_api_key).put(profile::regenerate_api_key),
        )
        // Comments (admin)
        .route("/api/v1/admin/comments", get(comments::list_all_comments))
        .route(
            "/api/v1/admin/comments/pending-count",
            get(comments::pending_count),
        )
        .route(
            "/api/v1/admin/comments/:id",
            put(comments::update_comment_status),
        )
        // Analytics
        .route("/api/v1/analytics/trend", get(analytics::get_trend))
        .route(
            "/api/v1/analytics/total-views",
            get(analytics::get_total_views),
        )
        .route(
            "/api/v1/analytics/total-likes",
            get(analytics::get_total_likes),
        )
        .route(
            "/api/v1/analytics/total-comments",
            get(analytics::get_total_comments),
        )
        .route(
            "/api/v1/analytics/today-likes",
            get(analytics::get_today_likes),
        )
        .route(
            "/api/v1/analytics/today-posts",
            get(analytics::get_today_posts),
        )
        .route(
            "/api/v1/analytics/post-views",
            get(analytics::get_post_views),
        )
        .route("/api/v1/admin/likes", get(analytics::get_like_records))
        // Login & Read logs
        .route("/api/v1/admin/login-logs", get(admin::list_login_logs))
        .route("/api/v1/admin/read-logs", get(admin::list_read_logs))
        .route("/api/v1/read-logs", post(posts::record_read_log))
        // Ops (admin only)
        .route("/api/v1/admin/logs", get(ops::get_logs))
        .route("/api/v1/admin/health", get(ops::get_health))
        .route("/api/v1/admin/stats", get(ops::get_stats))
        // Import/Export
        .route("/api/v1/export/posts", post(import_export::export_posts))
        .route("/api/v1/import/posts", post(import_export::import_markdown))
        // Network Resources
        .route(
            "/api/v1/network-resources",
            get(network_resources::list_resources).post(network_resources::create_resource),
        )
        .route(
            "/api/v1/network-resources/ensure",
            post(network_resources::ensure_resource),
        )
        .route(
            "/api/v1/network-resources/batch-resolve",
            post(network_resources::batch_resolve),
        )
        .route(
            "/api/v1/network-resources/:id",
            put(network_resources::update_resource).delete(network_resources::delete_resource),
        )
        .route(
            "/api/v1/network-resources/:id/references",
            get(network_resources::get_references),
        )
        .route(
            "/api/v1/network-resources/:id/resolve",
            get(network_resources::resolve_resource),
        )
        // AI (public)
        .route("/api/v1/ai/default-agent", get(ai::get_default_agent))
        // Changelog (public — MUST be before :id!)
        .route(
            "/api/v1/changelogs/latest",
            get(changelog::get_latest_version),
        )
        .route(
            "/api/v1/changelogs/public",
            get(changelog::list_public_changelogs),
        )
        // Changelog (admin)
        .route(
            "/api/v1/changelogs",
            get(changelog::list_changelogs).post(changelog::create_changelog),
        )
        .route(
            "/api/v1/changelogs/:id",
            put(changelog::update_changelog).delete(changelog::delete_changelog),
        )
        // News (public)
        .route("/api/v1/news/topic-types", get(news::list_topic_types))
        .route("/api/v1/news", get(news::list_news))
        .route("/api/v1/news/:id", get(news::get_news))
        // News (admin)
        .route(
            "/api/v1/admin/news",
            get(news::list_admin_news).post(news::create_news),
        )
        .route(
            "/api/v1/admin/news/batch-delete",
            post(news::batch_delete_news),
        )
        .route(
            "/api/v1/admin/news/:id",
            get(news::get_news)
                .put(news::update_news)
                .delete(news::delete_news),
        )
        // AI (admin)
        .route(
            "/api/v1/admin/ai/providers",
            get(ai::list_providers).post(ai::create_provider),
        )
        .route(
            "/api/v1/admin/ai/providers/:id",
            put(ai::update_provider).delete(ai::delete_provider),
        )
        .route(
            "/api/v1/admin/ai/providers/:id/test",
            post(ai::test_provider),
        )
        .route(
            "/api/v1/admin/ai/skills",
            get(ai::list_skills).post(ai::create_skill),
        )
        .route(
            "/api/v1/admin/ai/skills/:id",
            put(ai::update_skill).delete(ai::delete_skill),
        )
        .route(
            "/api/v1/admin/ai/tasks",
            get(ai::list_tasks).post(ai::create_task),
        )
        .route(
            "/api/v1/admin/ai/tasks/:id",
            put(ai::update_task).delete(ai::delete_task),
        )
        .route("/api/v1/admin/ai/tasks/:id/run", post(ai::run_task))
        .route("/api/v1/admin/ai/tasks/:id/trace", get(ai::get_task_trace))
        .route("/api/v1/admin/ai/tasks/:id/logs", get(ai::list_task_logs))
        .route(
            "/api/v1/admin/ai/tasks/:id/logs/:log_id",
            get(ai::get_task_log).delete(ai::delete_task_log),
        )
        .route(
            "/api/v1/admin/ai/agent-configs",
            get(ai::list_agent_configs).post(ai::create_agent_config),
        )
        .route(
            "/api/v1/admin/ai/agent-configs/:id",
            put(ai::update_agent_config).delete(ai::delete_agent_config),
        )
        .route(
            "/api/v1/admin/ai/models",
            get(ai::list_models).post(ai::create_model),
        )
        .route(
            "/api/v1/admin/ai/models/:id",
            put(ai::update_model).delete(ai::delete_model),
        )
        .route(
            "/api/v1/admin/ai/sessions",
            get(ai::list_sessions).post(ai::create_session),
        )
        .route(
            "/api/v1/admin/ai/sessions/:id",
            get(ai::get_session).delete(ai::delete_session),
        )
        .route("/api/v1/admin/ai/chat", post(ai::chat))
        .route(
            "/api/v1/admin/ai/tools",
            get(ai::list_tools).post(ai::create_tool),
        )
        .route(
            "/api/v1/admin/ai/tools/:id",
            put(ai::update_tool).delete(ai::delete_tool),
        )
        // Static files
        .route("/uploads/:filename", get(files::serve_upload))
        .with_state(state)
}

pub fn page_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(pages::aggregate_page))
        .route("/knowledge-base", get(pages::aggregate_page))
        .route("/categories", get(pages::aggregate_page))
        .route("/category/:slug", get(pages::aggregate_page))
        .route("/tags", get(pages::aggregate_page))
        .route("/tag/:slug", get(pages::aggregate_page))
        .route("/authors", get(pages::aggregate_page))
        .route("/author/:id", get(pages::aggregate_page))
        .route("/types", get(pages::aggregate_page))
        .route("/type/:code", get(pages::aggregate_page))
        .route("/statuses", get(pages::aggregate_page))
        .route("/status/:code", get(pages::aggregate_page))
        .route("/pinned", get(pages::aggregate_page))
        .route("/changelog", get(pages::aggregate_page))
        .route("/post/:slug", get(pages::post_detail))
        .route("/robots.txt", get(pages::robots_txt))
        .route("/sitemap.xml", get(pages::sitemap_xml))
        .route("/favicon.svg", get(pages::favicon_svg))
        .route("/favicon.png", get(pages::favicon_png))
        .route("/default-og.png", get(pages::default_og_image))
        .nest_service("/assets", ServeDir::new("static/frontend/assets"))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(serde::Serialize)]
struct VersionInfo {
    version: &'static str,
    name: &'static str,
}

async fn version() -> axum::Json<VersionInfo> {
    axum::Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION"),
        name: env!("CARGO_PKG_NAME"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn unknown_page_returns_http_404() {
        let app = Router::new().fallback(pages::spa_fallback);
        let server = TestServer::new(app).expect("test server should start");

        let response = server.get("/nonexistent-seo-test-404").await;

        assert_eq!(response.status_code(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn known_spa_page_returns_http_200() {
        let app = Router::new().fallback(pages::spa_fallback);
        let server = TestServer::new(app).expect("test server should start");

        let response = server.get("/knowledge-base").await;

        assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    }
}
