use axum::{routing::get, Router};
use clap::Parser;
use clipper_indexer::ClipperIndexer;
use clipper_server::{api, websocket, AppState, Cli, ServerConfig};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(api::routes())
        .merge(websocket::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

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
