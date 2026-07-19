use axum::{extract::DefaultBodyLimit, Router};
use tower_http::trace::TraceLayer;

pub mod api_doc;
pub mod config;
pub mod controllers;
pub mod crypto;
pub mod middleware;
pub mod migrations;
pub mod models;
pub mod services;
pub mod utils;

use utils::AppState;

/// Build the production HTTP router from a fully initialized application state.
///
/// Keeping router construction in the library lets the binary and integration
/// tests exercise the same routes and middleware stack.
pub fn build_router(state: AppState) -> Router {
    let max_body_size = state.config.storage.max_file_size as usize;

    Router::new()
        .merge(controllers::api_routes(state.clone()))
        .merge(controllers::page_routes(state.clone()))
        .fallback(controllers::pages::spa_fallback)
        .layer(middleware::compression())
        .layer(middleware::SecurityHeadersLayer)
        .layer(middleware::AssetCacheLayer)
        .layer(DefaultBodyLimit::max(max_body_size))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::ip_guard::ip_guard_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::stack())
        .with_state(state)
}
