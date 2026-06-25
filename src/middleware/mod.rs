pub mod auth;
pub mod ip_guard;


use tower_http::cors::CorsLayer;

pub fn stack() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}
