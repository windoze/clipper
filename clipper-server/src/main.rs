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
use {axum::http::Request, std::convert::Infallible, tower_http::services::ServeDir};

#[cfg(feature = "tls")]
use clipper_server::TlsManager;

#[cfg(feature = "acme")]
use {
    clipper_server::acme::{challenge_handler::AcmeChallengeState, AcmeManager},
    clipper_server::cert_storage::create_storage,
    std::sync::Arc,
};

// Embedded web UI files (only when embed-web feature is enabled)
#[cfg(feature = "embed-web")]
#[derive(rust_embed::RustEmbed)]
#[folder = "web/dist/"]
struct WebAssets;

#[tokio::main]
async fn main() {
    // Install the ring crypto provider for rustls
    // This must be done before any TLS operations
    #[cfg(feature = "tls")]
    {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
    }

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

    // Validate configuration
    if let Err(err) = config.validate() {
        eprintln!("Configuration error: {}", err);
        std::process::exit(1);
    }

    tracing::info!("Configuration loaded:");
    tracing::info!("  Database path: {}", config.database.path);
    tracing::info!("  Storage path: {}", config.storage.path);
    tracing::info!("  Listen address: {}", config.server.listen_addr);
    tracing::info!("  HTTP Port: {}", config.server.port);
    #[cfg(feature = "tls")]
    if config.tls.enabled {
        tracing::info!("  HTTPS Port: {}", config.tls.port);
        tracing::info!("  TLS enabled: true");
        #[cfg(feature = "acme")]
        if config.acme.enabled {
            tracing::info!("  ACME enabled: true");
            tracing::info!(
                "  ACME domain: {}",
                config.acme.domain.as_deref().unwrap_or("not set")
            );
            tracing::info!(
                "  ACME staging: {}",
                if config.acme.staging { "yes" } else { "no" }
            );
        }
    }

    // Initialize the indexer
    let indexer = ClipperIndexer::new(&config.database.path, &config.storage.path)
        .await
        .expect("Failed to initialize indexer");

    // Create application state
    let state = AppState::new(indexer);

    // Build the application with routes
    #[allow(unused_mut)]
    let mut api_routes = Router::new()
        .route("/health", get(health_check))
        .merge(api::routes())
        .merge(websocket::routes())
        .with_state(state);

    // Add ACME challenge route if enabled
    #[cfg(feature = "acme")]
    let acme_manager: Option<Arc<AcmeManager>> = if config.acme.enabled {
        let storage = create_storage(config.acme.get_certs_dir());
        let manager = Arc::new(AcmeManager::new(config.acme.clone(), storage));

        // Add challenge handler route
        let challenge_state = AcmeChallengeState {
            challenges: manager.pending_challenges(),
        };
        api_routes = api_routes.route(
            "/.well-known/acme-challenge/{token}",
            get(clipper_server::acme::challenge_handler::handle_challenge)
                .with_state(challenge_state),
        );

        Some(manager)
    } else {
        None
    };

    // Build the app with web UI serving
    let app = build_app_with_web_ui(api_routes);

    // Start the server(s)
    #[cfg(feature = "tls")]
    if config.tls.enabled {
        start_with_tls(config, app, {
            #[cfg(feature = "acme")]
            {
                acme_manager
            }
            #[cfg(not(feature = "acme"))]
            {
                None::<()>
            }
        })
        .await;
    } else {
        start_http_only(config, app).await;
    }

    #[cfg(not(feature = "tls"))]
    start_http_only(config, app).await;
}

/// Start HTTP-only server (no TLS).
async fn start_http_only(config: ServerConfig, app: Router) {
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

    tracing::info!("HTTP server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server failed");
}

/// Start server with TLS support.
#[cfg(feature = "tls")]
async fn start_with_tls<T>(config: ServerConfig, app: Router, acme_manager: Option<T>)
where
    T: std::any::Any + Send + Sync + 'static,
{
    #[cfg(feature = "acme")]
    use std::any::Any;

    let tls_addr = config.tls_socket_addr().unwrap_or_else(|err| {
        eprintln!("Invalid TLS listen address: {}", err);
        std::process::exit(1);
    });

    // Get certificate and key
    let (cert_pem, key_pem) = get_certificate(&config, &acme_manager).await;

    // Create TLS manager
    let tls_manager = TlsManager::from_pem(&cert_pem, &key_pem)
        .await
        .unwrap_or_else(|err| {
            eprintln!("Failed to configure TLS: {}", err);
            std::process::exit(1);
        });

    let rustls_config = tls_manager.config();

    // Spawn HTTP redirect server if enabled
    if config.tls.redirect_http {
        let http_addr = config.socket_addr().unwrap_or_else(|err| {
            eprintln!("Invalid HTTP listen address: {}", err);
            std::process::exit(1);
        });
        let https_port = config.tls.port;

        tokio::spawn(async move {
            run_http_redirect_server(http_addr, https_port).await;
        });
    }

    // Start certificate renewal task if ACME is enabled
    #[cfg(feature = "acme")]
    if let Some(ref manager) = acme_manager {
        if let Some(acme) = (manager as &dyn Any).downcast_ref::<Arc<AcmeManager>>() {
            let acme_clone = acme.clone();
            let tls_config_clone = rustls_config.clone();
            tokio::spawn(async move {
                clipper_server::acme::certificate_renewal_task(acme_clone, move |cert, key| {
                    let config = tls_config_clone.clone();
                    tokio::spawn(async move {
                        if let Err(e) = config
                            .reload_from_pem(cert.as_bytes().to_vec(), key.as_bytes().to_vec())
                            .await
                        {
                            tracing::error!("Failed to reload certificate: {}", e);
                        }
                    });
                })
                .await;
            });
        }
    }

    // Start periodic certificate reload task for manually managed certificates
    if let Some(interval) = config.tls.reload_interval() {
        if let (Some(cert_path), Some(key_path)) =
            (config.tls.cert_path.clone(), config.tls.key_path.clone())
        {
            let tls_config_clone = rustls_config.clone();
            tracing::info!(
                "Certificate reload enabled: checking every {} seconds",
                interval.as_secs()
            );
            tokio::spawn(async move {
                run_certificate_reload_task(tls_config_clone, cert_path, key_path, interval).await;
            });
        }
    }

    tracing::info!("HTTPS server listening on {}", tls_addr);

    axum_server::bind_rustls(tls_addr, rustls_config)
        .serve(app.into_make_service())
        .await
        .expect("HTTPS server failed");
}

/// Get certificate from ACME or manual configuration.
#[cfg(feature = "tls")]
async fn get_certificate<T>(
    config: &ServerConfig,
    #[allow(unused)] acme_manager: &Option<T>,
) -> (String, String)
where
    T: std::any::Any + Send + Sync + 'static,
{
    #[cfg(feature = "acme")]
    use std::any::Any;

    // Try ACME first if enabled
    #[cfg(feature = "acme")]
    if config.acme.enabled {
        if let Some(ref manager) = acme_manager {
            if let Some(acme) = (manager as &dyn Any).downcast_ref::<Arc<AcmeManager>>() {
                match acme.provision_certificate().await {
                    Ok((cert, key)) => return (cert, key),
                    Err(e) => {
                        tracing::error!("ACME certificate provisioning failed: {}", e);
                        // Fall through to manual cert or self-signed
                    }
                }
            }
        }
    }

    // Try manual certificate paths
    if let (Some(cert_path), Some(key_path)) = (&config.tls.cert_path, &config.tls.key_path) {
        let cert_pem = std::fs::read_to_string(cert_path).unwrap_or_else(|err| {
            eprintln!("Failed to read certificate file: {}", err);
            std::process::exit(1);
        });
        let key_pem = std::fs::read_to_string(key_path).unwrap_or_else(|err| {
            eprintln!("Failed to read key file: {}", err);
            std::process::exit(1);
        });
        return (cert_pem, key_pem);
    }

    // Generate self-signed certificate for development
    #[cfg(feature = "acme")]
    {
        let domain = config.acme.domain.as_deref().unwrap_or("localhost");
        tracing::warn!(
            "No certificate available, generating self-signed certificate for {}",
            domain
        );
        clipper_server::tls::generate_self_signed_cert(domain).unwrap_or_else(|err| {
            eprintln!("Failed to generate self-signed certificate: {}", err);
            std::process::exit(1);
        })
    }

    #[cfg(not(feature = "acme"))]
    {
        eprintln!("TLS enabled but no certificate configured");
        std::process::exit(1);
    }
}

/// Periodically reload certificates from disk.
/// Useful when certificates are managed by external tools like certbot.
#[cfg(feature = "tls")]
async fn run_certificate_reload_task(
    tls_config: axum_server::tls_rustls::RustlsConfig,
    cert_path: std::path::PathBuf,
    key_path: std::path::PathBuf,
    interval: std::time::Duration,
) {
    use std::time::SystemTime;

    // Track last modification times to avoid unnecessary reloads
    let mut last_cert_modified: Option<SystemTime> = None;
    let mut last_key_modified: Option<SystemTime> = None;

    loop {
        tokio::time::sleep(interval).await;

        // Check if files have been modified
        let cert_modified = tokio::fs::metadata(&cert_path)
            .await
            .ok()
            .and_then(|m| m.modified().ok());
        let key_modified = tokio::fs::metadata(&key_path)
            .await
            .ok()
            .and_then(|m| m.modified().ok());

        let cert_changed = match (&last_cert_modified, &cert_modified) {
            (Some(last), Some(current)) => current > last,
            (None, Some(_)) => true,
            _ => false,
        };

        let key_changed = match (&last_key_modified, &key_modified) {
            (Some(last), Some(current)) => current > last,
            (None, Some(_)) => true,
            _ => false,
        };

        if cert_changed || key_changed {
            tracing::info!("Certificate files changed, reloading...");

            match tls_config.reload_from_pem_file(&cert_path, &key_path).await {
                Ok(()) => {
                    tracing::info!("Certificate reloaded successfully");
                    last_cert_modified = cert_modified;
                    last_key_modified = key_modified;
                }
                Err(e) => {
                    tracing::error!("Failed to reload certificate: {}", e);
                    // Don't update last modified times so we retry next interval
                }
            }
        } else {
            tracing::debug!("Certificate files unchanged, skipping reload");
            // Update tracked times even if unchanged (first run)
            if last_cert_modified.is_none() {
                last_cert_modified = cert_modified;
            }
            if last_key_modified.is_none() {
                last_key_modified = key_modified;
            }
        }
    }
}

/// Run HTTP to HTTPS redirect server.
#[cfg(feature = "tls")]
async fn run_http_redirect_server(http_addr: std::net::SocketAddr, https_port: u16) {
    use axum::response::Redirect;

    let redirect_app = Router::new().fallback(move |uri: Uri| async move {
        let host = uri.host().unwrap_or("localhost");
        let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

        let https_uri = if https_port == 443 {
            format!("https://{}{}", host, path_and_query)
        } else {
            format!("https://{}:{}{}", host, https_port, path_and_query)
        };

        Redirect::permanent(&https_uri)
    });

    let listener = match tokio::net::TcpListener::bind(&http_addr).await {
        Ok(l) => l,
        Err(err) => {
            tracing::warn!(
                "Failed to bind HTTP redirect server to {}: {}",
                http_addr,
                err
            );
            return;
        }
    };

    tracing::info!(
        "HTTP redirect server listening on {} -> HTTPS port {}",
        http_addr,
        https_port
    );

    if let Err(e) = axum::serve(listener, redirect_app).await {
        tracing::error!("HTTP redirect server error: {}", e);
    }
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
