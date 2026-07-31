//! Authentication middleware for Bearer token authentication.

use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::state::AppState;

/// Middleware that validates Bearer token authentication.
///
/// If authentication is not configured (no bearer token set), all requests are allowed.
/// If authentication is configured, requests must include a valid
/// `Authorization: Bearer <token>` header.
///
/// Certain endpoints are always allowed without authentication:
/// - GET /health - Health check endpoint
/// - GET /auth/check - Authentication status check
/// - GET /ws - WebSocket endpoint (handles its own message-based authentication)
/// - GET /s/{code} - Public short URL resolver
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
    // WebSocket endpoint handles its own message-based authentication
    // /s/{code} is the public short URL resolver (no auth required)
    // /shared-assets/* serves static files for shared clip pages (no auth required)
    let path = request.uri().path();
    if path == "/health"
        || path == "/auth/check"
        || path == "/ws"
        || path.starts_with("/s/")
        || path.starts_with("/shared-assets/")
    {
        return next.run(request).await;
    }

    // Try to extract token from Authorization header first
    let auth_header = request.headers().get(header::AUTHORIZATION);

    if let Some(header_value) = auth_header {
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
            return next.run(request).await;
        } else {
            return unauthorized_response("Invalid bearer token");
        }
    }

    unauthorized_response("Missing Authorization header")
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
