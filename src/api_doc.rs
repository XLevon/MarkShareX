//! OpenAPI 文档 — utoipa + Scalar
//!
//! 生成 `/api/v1/openapi.json`（机器可读）和 `/scalar`（交互式 UI）。
//! AI agent 可直接消费 openapi.json；MCP Server 可用标准 OpenAPI→MCP 工具桥接。

use utoipa::{
    openapi::{
        path::{PathItemType, PathsBuilder},
        security::{
            ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme,
        },
    },
    OpenApi, Path,
};

use crate::api_endpoints::api_endpoint_catalog;

use crate::controllers::{
    admin::AdminUserResponse,
    ai::{AiProviderResponse, AiSkillResponse, AiTaskResponse},
    auth::{LoginResponse, UserResponse},
    author_applications::ApplicationResponse,
    categories::CategoryResponse,
    categories::ReorderRequest,
    changelog::ChangelogResponse,
    comments::CommentResponse,
    files::FileResponse,
    network_resources::NetworkResourceResponse,
    news::NewsResponse,
    ops::{HealthResponse, LogQueryResponse, StatsResponse},
    posts::{AdjacentPostsResponse, AuthorInfo, LikeStatusResponse, PostResponse, SearchResponse},
    profile::{ApiKeyResponse, ProfileResponse, SiteManagerResponse},
    settings::SettingsResponse,
    setup::SetupResponse,
    tags::TagResponse,
};

/// 聚合所有 API 路径和模型
#[derive(OpenApi)]
#[openapi(
    info(
        title = "MarkShareX API",
        description = "轻量自托管 Markdown 博客系统 — 完整 REST API",
        contact(
            name = "MarkShareX",
            url = "https://github.com/XLevon/MarkShareX",
        ),
    ),
    components(schemas(
        PostResponse, AdjacentPostsResponse, AuthorInfo, LikeStatusResponse, SearchResponse,
        CategoryResponse, TagResponse, FileResponse, CommentResponse,
        ReorderRequest,
        SettingsResponse, SetupResponse, LoginResponse, UserResponse,
        AdminUserResponse, ProfileResponse, SiteManagerResponse, ApiKeyResponse,
        NetworkResourceResponse, ApplicationResponse, ChangelogResponse,
        NewsResponse,
        AiProviderResponse, AiSkillResponse, AiTaskResponse,
        LogQueryResponse, HealthResponse, StatsResponse,
    )),
    tags(
        (name = "Admin", description = "管理后台"),
        (name = "Analytics", description = "数据统计"),
        (name = "Applications", description = "作者申请"),
        (name = "Auth", description = "认证与授权"),
        (name = "Categories", description = "分类管理"),
        (name = "Changelog", description = "版本更新说明"),
        (name = "Comments", description = "评论管理"),
        (name = "Files", description = "文件上传与管理"),
        (name = "Import/Export", description = "导入导出"),
        (name = "Network Resources", description = "网络资源库"),
        (name = "News", description = "咨询信息"),
        (name = "AI", description = "AI 模块"),
        (name = "Posts", description = "文章管理"),
        (name = "Profile", description = "个人资料"),
        (name = "Settings", description = "站点设置"),
        (name = "Tags", description = "标签管理"),
        (name = "Ops", description = "运维管理"),
        (name = "Setup", description = "系统初始化"),
    ),
)]
struct ApiComponents;

pub(crate) fn endpoint_description(method: &str, route_path: &str, _handler: &str) -> String {
    include_str!("endpoint_descriptions.tsv")
        .lines()
        .find_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let candidate_method = fields.next()?;
            let candidate_path = fields.next()?;
            let description = fields.next()?;
            (candidate_method == method && candidate_path == route_path)
                .then(|| description.to_string())
        })
        .unwrap_or_else(|| {
            panic!("missing deliberate endpoint description for {method} {route_path}")
        })
}

macro_rules! operation_type {
    (GET) => {
        PathItemType::Get
    };
    (POST) => {
        PathItemType::Post
    };
    (POST_CSP) => {
        PathItemType::Post
    };
    (PUT) => {
        PathItemType::Put
    };
    (DELETE) => {
        PathItemType::Delete
    };
}

macro_rules! build_openapi_paths {
    ($builder:expr; $(($method:ident, $route_path:literal, $handler:path, $doc:path, $auth:literal)),* $(,)?) => {{
        let mut builder = $builder;
        $(
            let mut item = <$doc as Path>::path_item(None);
            let operation = item
                .operations
                .get_mut(&operation_type!($method))
                .expect("catalog method must match the Utoipa operation method");
            if operation.summary.is_none() {
                operation.summary = Some(endpoint_description(
                    stringify!($method).trim_end_matches("_CSP"),
                    $route_path,
                    stringify!($handler),
                ));
            }
            if $auth {
                operation.security = Some(vec![
                    SecurityRequirement::new("bearerAuth", Vec::<String>::new()),
                    SecurityRequirement::new("apiKeyAuth", Vec::<String>::new()),
                ]);
            } else {
                operation.security = None;
            }
            builder = builder.path(<$doc as Path>::path(), item);
        )*
        builder
    }};
}

pub struct ApiDoc;

impl OpenApi for ApiDoc {
    fn openapi() -> utoipa::openapi::OpenApi {
        let mut document = ApiComponents::openapi();
        document.info.version = env!("CARGO_PKG_VERSION").to_string();
        document.paths = api_endpoint_catalog!(build_openapi_paths, PathsBuilder::new()).build();
        let components = document
            .components
            .as_mut()
            .expect("schema-derived OpenAPI document must contain components");
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "apiKeyAuth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "X-API-Key",
                "MarkShareX user API key",
            ))),
        );
        document
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_version_matches_the_cargo_package_version() {
        let document = ApiDoc::openapi();
        assert_eq!(document.info.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn openapi_contains_every_production_api_operation() {
        let document = ApiDoc::openapi();
        let operation_count = document
            .paths
            .paths
            .values()
            .map(|item| item.operations.len())
            .sum::<usize>();
        assert_eq!(operation_count, 162);
    }

    #[test]
    fn openapi_registers_auth_schemes_and_human_readable_summaries() {
        let document = ApiDoc::openapi();
        let components = document.components.as_ref().expect("missing components");
        assert!(components.security_schemes.contains_key("bearerAuth"));
        assert!(components.security_schemes.contains_key("apiKeyAuth"));

        for (path, item) in &document.paths.paths {
            for operation in item.operations.values() {
                let summary = operation.summary.as_deref().unwrap_or_default();
                assert!(!summary.trim().is_empty(), "missing summary for {path}");
                if let Some(operation_id) = operation.operation_id.as_deref() {
                    assert_ne!(
                        summary, operation_id,
                        "operation identifier leaked as summary for {path}"
                    );
                }
            }
        }

        let serialized = serde_json::to_value(&document).expect("OpenAPI must serialize");
        assert_eq!(
            serialized["components"]["securitySchemes"]["bearerAuth"]["scheme"],
            "bearer"
        );
        assert_eq!(
            serialized["components"]["securitySchemes"]["apiKeyAuth"]["name"],
            "X-API-Key"
        );
        assert_eq!(
            serialized["components"]["securitySchemes"]["apiKeyAuth"]["in"],
            "header"
        );
    }
}
