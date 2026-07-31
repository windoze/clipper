use axum::http::{HeaderValue, Method, header};
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::ServerConfig;

pub fn build_cors_layer(config: &ServerConfig) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .expose_headers([header::CONTENT_DISPOSITION, header::CONTENT_TYPE])
        .max_age(Duration::from_secs(600));

    if config.cors.allowed_origins.is_empty() {
        return layer;
    }

    let origins = config
        .cors
        .allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).expect("CORS origins were validated"));

    layer.allow_origin(AllowOrigin::list(origins))
}
