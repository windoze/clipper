use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    middleware,
    routing::get,
};
use clipper_indexer::ClipperIndexer;
use clipper_server::{AppState, ServerConfig, auth_middleware, build_cors_layer};
use tempfile::TempDir;
use tower::ServiceExt;

async fn create_auth_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("db");
    let storage_path = temp_dir.path().join("storage");

    let indexer = ClipperIndexer::new(&db_path, &storage_path)
        .await
        .expect("Failed to create indexer");

    let mut config = ServerConfig::default();
    config.auth.bearer_token = Some("secret-token".to_string());

    let state = AppState::new(indexer, config);
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/private", get(|| async { "ok" }))
        .route("/version", get(|| async { "version" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    (app, temp_dir)
}

#[tokio::test]
async fn test_auth_accepts_bearer_header() {
    let (app, _temp_dir) = create_auth_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/private")
                .header(header::AUTHORIZATION, "Bearer secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_rejects_query_token() {
    let (app, _temp_dir) = create_auth_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/private?token=secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_allows_health_without_header() {
    let (app, _temp_dir) = create_auth_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_requires_bearer_header_for_version() {
    let (app, _temp_dir) = create_auth_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/version")
                .header(header::AUTHORIZATION, "Bearer secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cors_allows_only_configured_origins() {
    let mut config = ServerConfig::default();
    config.cors.allowed_origins = vec!["https://app.example.com".to_string()];

    let app = Router::new()
        .route("/ok", get(|| async { "ok" }))
        .layer(build_cors_layer(&config));

    let allowed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/ok")
                .header(header::ORIGIN, "https://app.example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        allowed_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some("https://app.example.com")
    );

    let rejected_response = app
        .oneshot(
            Request::builder()
                .uri("/ok")
                .header(header::ORIGIN, "https://evil.example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        rejected_response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}
