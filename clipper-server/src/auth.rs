//! Authentication middleware for Bearer token authentication.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::state::AppState;

/// Middleware that validates Bearer token authentication.
///
/// If authentication is not configured (no bearer token set), all requests are allowed.
/// If authentication is configured, requests must include a valid `Authorization: Bearer <token>` header.
///
/// Certain endpoints are always allowed without authentication:
/// - GET /health - Health check endpoint
/// - GET /auth/check - Authentication status check
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let auth_config = &state.config.auth;

    // If auth is not enabled, allow all requests
    if !auth_config.is_enabled() {
        return next.run(request).await;
    }

    // Allow certain endpoints without authentication
    let path = request.uri().path();
    if path == "/health" || path == "/auth/check" {
        return next.run(request).await;
    }

    // Extract and validate the Bearer token
    let auth_header = request.headers().get(header::AUTHORIZATION);

    match auth_header {
        Some(header_value) => {
            let header_str = match header_value.to_str() {
                Ok(s) => s,
                Err(_) => {
                    return unauthorized_response("Invalid Authorization header encoding");
                }
            };

            // Check for Bearer prefix
            if !header_str.starts_with("Bearer ") {
                return unauthorized_response("Authorization header must use Bearer scheme");
            }

            let token = &header_str[7..]; // Skip "Bearer "

            if auth_config.validate_token(token) {
                next.run(request).await
            } else {
                unauthorized_response("Invalid bearer token")
            }
        }
        None => unauthorized_response("Missing Authorization header"),
    }
}

/// Create an unauthorized response with a JSON body.
fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": message,
            "auth_required": true
        })),
    )
        .into_response()
}
