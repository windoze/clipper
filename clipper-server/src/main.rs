use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};
use clap::Parser;
use clipper_indexer::ClipperIndexer;
use clipper_server::{api, websocket, AppState, Cli, ServerConfig};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(not(feature = "embed-web"))]
use {
    axum::http::Request,
    std::convert::Infallible,
    tower_http::services::ServeDir,
};

// Embedded web UI files (only when embed-web feature is enabled)
#[cfg(feature = "embed-web")]
#[derive(rust_embed::RustEmbed)]
#[folder = "web/dist/"]
struct WebAssets;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clipper_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration from all sources
    let config = ServerConfig::load(cli).unwrap_or_else(|err| {
        eprintln!("Failed to load configuration: {}", err);
        std::process::exit(1);
    });

    tracing::info!("Configuration loaded:");
    tracing::info!("  Database path: {}", config.database.path);
    tracing::info!("  Storage path: {}", config.storage.path);
    tracing::info!("  Listen address: {}", config.server.listen_addr);
    tracing::info!("  Port: {}", config.server.port);

    // Initialize the indexer
    let indexer = ClipperIndexer::new(&config.database.path, &config.storage.path)
        .await
        .expect("Failed to initialize indexer");

    // Create application state
    let state = AppState::new(indexer);

    // Build the application with routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .merge(api::routes())
        .merge(websocket::routes())
        .with_state(state);

    // Build the app with web UI serving
    let app = build_app_with_web_ui(api_routes);

    // Get socket address
    let addr = config.socket_addr().unwrap_or_else(|err| {
        eprintln!("Invalid listen address: {}", err);
        std::process::exit(1);
    });

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| {
            eprintln!("Failed to bind to {}: {}", addr, err);
            std::process::exit(1);
        });

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server failed");
}

async fn health_check() -> &'static str {
    "OK"
}

// ============================================================================
// Embedded Web UI (when embed-web feature is enabled)
// ============================================================================

#[cfg(feature = "embed-web")]
fn build_app_with_web_ui(api_routes: Router) -> Router {
    tracing::info!("Serving embedded web UI");

    let app = Router::new()
        .merge(api_routes)
        .fallback(serve_embedded_file)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    app
}

#[cfg(feature = "embed-web")]
async fn serve_embedded_file(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file first
    if let Some(content) = WebAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // For SPA routing, serve index.html for non-file paths
    if let Some(content) = WebAssets::get("index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // No embedded files found
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Web UI not embedded in this build"))
        .unwrap()
}

// ============================================================================
// Filesystem-based Web UI (default, when embed-web feature is NOT enabled)
// ============================================================================

#[cfg(not(feature = "embed-web"))]
fn build_app_with_web_ui(api_routes: Router) -> Router {
    // Determine web UI directory
    let web_dir = std::env::var("CLIPPER_WEB_DIR").unwrap_or_else(|_| {
        // Check common locations for the web UI
        let possible_paths = [
            "./web/dist",                 // Development
            "../clipper-server/web/dist", // Running from repo root
            "./clipper-server/web/dist",  // Running from repo root
        ];
        for path in possible_paths {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }
        // Default to ./web/dist even if it doesn't exist (will serve 404s)
        "./web/dist".to_string()
    });

    tracing::info!("Web UI directory: {}", web_dir);

    // Serve static files and fall back to index.html for SPA routing
    let serve_dir =
        ServeDir::new(&web_dir).not_found_service(tower::service_fn(move |req: Request<Body>| {
            let web_dir = web_dir.clone();
            async move { serve_index_html_from_fs(&web_dir, req.uri().clone()).await }
        }));

    Router::new()
        .merge(api_routes)
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

#[cfg(not(feature = "embed-web"))]
async fn serve_index_html_from_fs(web_dir: &str, _uri: Uri) -> Result<Response<Body>, Infallible> {
    let index_path = format!("{}/index.html", web_dir);
    match tokio::fs::read(&index_path).await {
        Ok(contents) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(contents))
            .unwrap()),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Web UI not found. Build the web UI first with: cd web && npm install && npm run build"))
            .unwrap()),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
