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

/// Extract token from query string (e.g., ?token=xxx)
fn extract_query_token(query: Option<&str>) -> Option<String> {
    query.and_then(|q| {
        q.split('&')
            .filter_map(|pair| {
                let (key, value) = pair.split_once('=')?;

                if key == "token" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
            .next()
    })
}

/// Middleware that validates Bearer token authentication.
///
/// If authentication is not configured (no bearer token set), all requests are allowed.
/// If authentication is configured, requests must include either:
/// - A valid `Authorization: Bearer <token>` header, OR
/// - A valid `?token=<token>` query parameter (useful for file downloads, WebSocket, etc.)
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
    let path = request.uri().path();
    if path == "/health" || path == "/auth/check" || path == "/ws" || path.starts_with("/s/") {
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

    // Fall back to query parameter token (useful for file downloads, images, etc.)
    if let Some(token) = extract_query_token(request.uri().query()) {
        if auth_config.validate_token(&token) {
            return next.run(request).await;
        } else {
            return unauthorized_response("Invalid token");
        }
    }

    unauthorized_response("Missing Authorization header or token parameter")
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
