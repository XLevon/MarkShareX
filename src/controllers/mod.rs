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
use crate::api_endpoints::api_endpoint_catalog;
use crate::middleware::auth::require_admin_middleware;
use crate::utils::{ApiResponse, AppState};
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
#[cfg(test)]
use utoipa::{openapi::path::PathItemType, Path};
use utoipa_scalar::{Scalar, Servable};

// ── 权威端点目录派生 ──

#[derive(Serialize)]
struct EndpointInfo {
    method: &'static str,
    path: String,
    description: String,
    auth_required: bool,
}

struct EndpointDefinition {
    method: &'static str,
    route_path: &'static str,
    #[cfg(test)]
    openapi_path: String,
    #[cfg(test)]
    operation_type: PathItemType,
    #[cfg(test)]
    handler: &'static str,
    description: String,
    auth_required: bool,
}

macro_rules! endpoint_method_name {
    (GET) => {
        "GET"
    };
    (POST) => {
        "POST"
    };
    (POST_CSP) => {
        "POST"
    };
    (PUT) => {
        "PUT"
    };
    (DELETE) => {
        "DELETE"
    };
}

#[cfg(test)]
macro_rules! endpoint_operation_type {
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

macro_rules! build_endpoint_definitions {
    (; $(($method:ident, $route_path:literal, $handler:path, $doc:path, $auth:literal)),* $(,)?) => {{
        vec![$(
            EndpointDefinition {
                method: endpoint_method_name!($method),
                route_path: $route_path,
                #[cfg(test)]
                openapi_path: <$doc as Path>::path(),
                #[cfg(test)]
                operation_type: endpoint_operation_type!($method),
                #[cfg(test)]
                handler: stringify!($handler),
                description: crate::api_doc::endpoint_description(
                    endpoint_method_name!($method),
                    $route_path,
                    stringify!($handler),
                ),
                auth_required: $auth,
            }
        ),*]
    }};
}

/// GET /api/v1/ — OpenAPI、生产 Router 和认证标记共用同一端点目录。
#[utoipa::path(get, path = "/api/v1/", tag = "Core")]
async fn list_endpoints() -> Json<ApiResponse<Vec<EndpointInfo>>> {
    let endpoints = api_endpoint_catalog!(build_endpoint_definitions)
        .into_iter()
        .map(|definition| EndpointInfo {
            method: definition.method,
            path: definition.route_path.to_string(),
            description: definition.description,
            auth_required: definition.auth_required,
        })
        .collect();
    Json(ApiResponse::new(endpoints))
}

#[utoipa::path(get, path = "/api/v1/openapi.json", tag = "Core")]
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

macro_rules! endpoint_method_router {
    (GET, $handler:path) => {
        get($handler)
    };
    (POST, $handler:path) => {
        post($handler)
    };
    (POST_CSP, $handler:path) => {
        post($handler).layer(DefaultBodyLimit::max(16 * 1024))
    };
    (PUT, $handler:path) => {
        put($handler)
    };
    (DELETE, $handler:path) => {
        delete($handler)
    };
}

macro_rules! build_catalog_router {
    ($router:expr; $(($method:ident, $route_path:literal, $handler:path, $doc:path, $auth:literal)),* $(,)?) => {{
        let mut router = $router;
        $(router = router.route($route_path, endpoint_method_router!($method, $handler));)*
        router
    }};
}

pub fn api_routes(state: AppState) -> Router<AppState> {
    let openapi = ApiDoc::openapi();
    let router = api_endpoint_catalog!(build_catalog_router, Router::new());

    router
        .merge({
            let scalar = Router::<AppState>::new().merge(Scalar::with_url("/scalar", openapi));
            scalar.route_layer(middleware::from_fn_with_state(
                state.clone(),
                require_admin_middleware,
            ))
        })
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

#[utoipa::path(get, path = "/api/v1/health", tag = "Core")]
async fn health_check() -> &'static str {
    "OK"
}

#[derive(serde::Serialize)]
struct VersionInfo {
    version: &'static str,
    name: &'static str,
}

#[utoipa::path(get, path = "/api/v1/version", tag = "Core")]
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
    use std::collections::HashSet;

    fn operation_method(operation_type: &PathItemType) -> Option<&'static str> {
        match operation_type {
            PathItemType::Get => Some("GET"),
            PathItemType::Post => Some("POST"),
            PathItemType::Put => Some("PUT"),
            PathItemType::Delete => Some("DELETE"),
            _ => None,
        }
    }

    fn normalize_router_path(path: &str) -> String {
        path.split('/')
            .map(|segment| match segment.strip_prefix(':') {
                Some(parameter) => format!("{{{parameter}}}"),
                None => segment.to_string(),
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    fn handler_source(handler: &str) -> &'static str {
        let normalized = handler.replace(' ', "");
        let module = normalized.split("::").next().unwrap_or_default();
        match module {
            "admin" => include_str!("admin.rs"),
            "ai" => include_str!("ai.rs"),
            "analytics" => include_str!("analytics.rs"),
            "article_kb" => include_str!("article_kb.rs"),
            "auth" => include_str!("auth.rs"),
            "author_applications" => include_str!("author_applications.rs"),
            "categories" => include_str!("categories.rs"),
            "changelog" => include_str!("changelog.rs"),
            "comments" => include_str!("comments.rs"),
            "csp" => include_str!("csp.rs"),
            "files" => include_str!("files.rs"),
            "guestbook" => include_str!("guestbook.rs"),
            "import_export" => include_str!("import_export.rs"),
            "network_resources" => include_str!("network_resources.rs"),
            "news" => include_str!("news.rs"),
            "ops" => include_str!("ops.rs"),
            "posts" => include_str!("posts.rs"),
            "profile" => include_str!("profile.rs"),
            "settings" => include_str!("settings.rs"),
            "setup" => include_str!("setup.rs"),
            "tags" => include_str!("tags.rs"),
            _ if !normalized.contains("::") => include_str!("mod.rs"),
            _ => panic!("no source oracle for handler {handler}"),
        }
    }

    fn handler_parameters(handler: &str) -> String {
        let normalized = handler.replace(' ', "");
        let function = normalized.split("::").last().unwrap_or_default();
        let source = handler_source(handler);
        let public_needle = format!("pub async fn {function}");
        let private_needle = format!("async fn {function}");
        let start = source
            .find(&public_needle)
            .or_else(|| source.find(&private_needle))
            .unwrap_or_else(|| panic!("handler function not found: {handler}"));
        let signature = &source[start..];
        let open = signature.find('(').expect("handler must have parameters");
        let mut depth = 0usize;
        for (offset, ch) in signature[open..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return signature[open + 1..open + offset].to_string();
                    }
                }
                _ => {}
            }
        }
        panic!("unterminated handler parameter list: {handler}");
    }

    fn signature_requires_auth(handler: &str) -> bool {
        let parameters = handler_parameters(handler);
        [
            ": AuthUser",
            ": AdminUser",
            ": PrivilegedUser",
            ": AuthorOrPrivilegedUser",
        ]
        .iter()
        .any(|marker| parameters.contains(marker))
    }

    fn catalog_fingerprint(definitions: &[EndpointDefinition]) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        for definition in definitions {
            let record = format!(
                "{}\0{}\0{}\0{}\n",
                definition.method,
                definition.route_path,
                definition.handler.replace(' ', ""),
                definition.auth_required
            );
            for byte in record.bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash
    }

    #[test]
    fn endpoint_catalog_and_openapi_are_unique_and_one_to_one() {
        let definitions = api_endpoint_catalog!(build_endpoint_definitions);
        let router_operations = definitions
            .iter()
            .map(|definition| (definition.method, definition.route_path))
            .collect::<HashSet<_>>();
        assert_eq!(
            router_operations.len(),
            definitions.len(),
            "duplicate production Router operation"
        );

        for definition in &definitions {
            assert_eq!(
                operation_method(&definition.operation_type),
                Some(definition.method),
                "catalog method does not match Utoipa method for {}",
                definition.handler,
            );
            assert_eq!(
                normalize_router_path(definition.route_path),
                definition.openapi_path,
                "Router/OpenAPI path mismatch for {} {} ({})",
                definition.method,
                definition.route_path,
                definition.handler,
            );
        }

        let normalized_router_operations = definitions
            .iter()
            .map(|definition| {
                (
                    definition.method,
                    normalize_router_path(definition.route_path),
                )
            })
            .collect::<HashSet<_>>();
        let document = ApiDoc::openapi();
        let document_json = serde_json::to_value(&document).expect("OpenAPI must serialize");
        let registered_schemes = document_json["components"]["securitySchemes"]
            .as_object()
            .expect("OpenAPI securitySchemes must be an object");
        assert!(registered_schemes.contains_key("bearerAuth"));
        assert!(registered_schemes.contains_key("apiKeyAuth"));
        let openapi_operations = document
            .paths
            .paths
            .iter()
            .flat_map(|(path, item)| {
                item.operations.keys().filter_map(move |operation_type| {
                    operation_method(operation_type).map(|method| (method, path.clone()))
                })
            })
            .collect::<HashSet<_>>();

        assert_eq!(normalized_router_operations, openapi_operations);

        for definition in &definitions {
            let operation = document
                .paths
                .get_path_operation(&definition.openapi_path, definition.operation_type.clone())
                .unwrap_or_else(|| panic!("missing OpenAPI operation for {}", definition.handler));
            if definition.auth_required {
                let security = operation.security.as_ref().unwrap_or_else(|| {
                    panic!(
                        "missing OpenAPI security for {} {}",
                        definition.method, definition.route_path
                    )
                });
                assert_eq!(
                    serde_json::to_value(security).expect("security must serialize"),
                    serde_json::json!([
                        {"bearerAuth": []},
                        {"apiKeyAuth": []}
                    ]),
                    "protected operation must require bearerAuth OR apiKeyAuth with empty scopes for {} {}",
                    definition.method,
                    definition.route_path,
                );
                for requirement in security {
                    let requirement_json = serde_json::to_value(requirement)
                        .expect("security requirement must serialize");
                    let requirement_object = requirement_json
                        .as_object()
                        .expect("security requirement must be an object");
                    assert!(
                        !requirement_object.is_empty(),
                        "empty security requirement permits anonymous access"
                    );
                    for scheme in requirement_object.keys() {
                        assert!(
                            registered_schemes.contains_key(scheme),
                            "operation references unregistered security scheme {scheme}"
                        );
                    }
                }
            } else {
                assert!(
                    operation.security.is_none(),
                    "public operation must not define effective security for {} {}",
                    definition.method,
                    definition.route_path,
                );
            }
        }
    }

    #[test]
    fn endpoint_catalog_auth_matches_real_handler_extractors() {
        let definitions = api_endpoint_catalog!(build_endpoint_definitions);
        for definition in &definitions {
            assert_eq!(
                definition.auth_required,
                signature_requires_auth(definition.handler),
                "catalog auth flag disagrees with handler signature for {} {} ({})",
                definition.method,
                definition.route_path,
                definition.handler,
            );
        }
    }

    #[test]
    fn pre_refactor_endpoint_bindings_have_not_drifted() {
        let definitions = api_endpoint_catalog!(build_endpoint_definitions);
        assert_eq!(definitions.len(), 162);
        assert_eq!(
            catalog_fingerprint(&definitions),
            13_678_631_846_492_913_858
        );
    }

    #[tokio::test]
    async fn discovery_endpoint_is_derived_from_the_same_catalog() {
        let definitions = api_endpoint_catalog!(build_endpoint_definitions);
        let Json(response) = list_endpoints().await;

        assert_eq!(response.data.len(), definitions.len());
        for (endpoint, definition) in response.data.iter().zip(definitions.iter()) {
            assert_eq!(endpoint.method, definition.method);
            assert_eq!(endpoint.path, definition.route_path);
            assert_eq!(endpoint.auth_required, definition.auth_required);
            assert_eq!(endpoint.description, definition.description);
        }
        let all_descriptions = include_str!("../../tests/fixtures/endpoint_descriptions.tsv");
        let all_expected = all_descriptions
            .lines()
            .map(|line| {
                let mut fields = line.splitn(3, '\t');
                (
                    fields.next().expect("missing method"),
                    fields.next().expect("missing path"),
                    fields.next().expect("missing description"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(all_expected.len(), 162);
        assert_eq!(all_expected.len(), response.data.len());
        for (method, path, description) in all_expected {
            let endpoint = response
                .data
                .iter()
                .find(|endpoint| endpoint.method == method && endpoint.path == path)
                .unwrap_or_else(|| panic!("missing described endpoint {method} {path}"));
            assert_eq!(endpoint.description, description);
            assert!(!endpoint.description.trim().is_empty());
        }

        let historical_descriptions =
            include_str!("../../tests/fixtures/pre_refactor_endpoint_descriptions.tsv");
        let expected = historical_descriptions
            .lines()
            .map(|line| {
                let mut fields = line.splitn(3, '\t');
                (
                    fields.next().expect("missing baseline method"),
                    fields.next().expect("missing baseline path"),
                    fields.next().expect("missing baseline description"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 76);
        for (method, path, description) in expected {
            let endpoint = response
                .data
                .iter()
                .find(|endpoint| endpoint.method == method && endpoint.path == path)
                .unwrap_or_else(|| panic!("missing historical discovery endpoint {method} {path}"));
            assert_eq!(
                endpoint.description, description,
                "historical description drifted for {method} {path}"
            );
        }
    }

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
