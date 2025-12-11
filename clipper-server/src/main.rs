use std::io::IsTerminal;

use axum::{
    Router,
    body::Body,
    http::{StatusCode, Uri, header},
    middleware,
    response::Response,
    routing::get,
};
use clap::Parser;
use clipper_indexer::ClipperIndexer;
use clipper_server::{
    AppState, Cli, ServerConfig, api, auth_middleware, run_clip_cleanup_task,
    run_short_url_cleanup_task, websocket,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(not(feature = "embed-web"))]
use {axum::http::Request, std::convert::Infallible, tower_http::services::ServeDir};

#[cfg(feature = "tls")]
use clipper_server::TlsManager;

#[cfg(feature = "acme")]
use {
    clipper_server::acme::{AcmeManager, challenge_handler::AcmeChallengeState},
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
    let use_color = std::io::stdout().is_terminal();
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clipper_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(use_color))
        .init();

    // Set restrictive permissions for newly created files and directories.
    // On Unix: Sets umask to 0o077 (files 0600, directories 0700)
    // On Windows: This is a no-op; directories are secured after creation with ACLs
    clipper_security::set_restrictive_umask();
    tracing::debug!("Set restrictive file permissions");

    // Parse command line arguments
    let cli = Cli::parse();

    // Start parent process monitor if running in bundled mode
    // This must be done early before the cli is consumed
    let parent_shutdown_rx = if let Some(handle) = cli.parent_pipe_handle {
        let rx = clipper_server::parent_monitor::init_shutdown_channel();
        clipper_server::parent_monitor::start_parent_monitor(handle);
        Some(rx)
    } else {
        None
    };

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

    // Secure the data directories and fix any incorrect permissions
    // On Unix: checks and fixes permissions to 0700/0600
    // On Windows: sets DACL to grant access only to current user
    let db_path = std::path::Path::new(&config.database.path);
    let storage_path = std::path::Path::new(&config.storage.path);

    match clipper_security::secure_directory_recursive(db_path, |msg| tracing::warn!("{}", msg)) {
        Ok(count) if count > 0 => {
            tracing::info!("Fixed permissions on {} items in database directory", count);
        }
        Err(e) => tracing::warn!("Failed to secure database directory: {}", e),
        _ => {}
    }

    match clipper_security::secure_directory_recursive(storage_path, |msg| {
        tracing::warn!("{}", msg)
    }) {
        Ok(count) if count > 0 => {
            tracing::info!("Fixed permissions on {} items in storage directory", count);
        }
        Err(e) => tracing::warn!("Failed to secure storage directory: {}", e),
        _ => {}
    }

    // Create application state
    let state = AppState::new(indexer, config.clone());

    // Start clip cleanup task if enabled
    if config.cleanup.is_active() {
        tracing::info!(
            "Auto-cleanup enabled: retention={} days, interval={} hours",
            config.cleanup.retention_days,
            config.cleanup.interval_hours
        );
        let cleanup_state = state.clone();
        let cleanup_config = config.cleanup.clone();
        tokio::spawn(async move {
            run_clip_cleanup_task(cleanup_state, cleanup_config).await;
        });
    }

    // Start short URL cleanup task (always runs to clean expired short URLs)
    {
        let short_url_cleanup_state = state.clone();
        tokio::spawn(async move {
            run_short_url_cleanup_task(short_url_cleanup_state).await;
        });
    }

    // Log auth status
    if config.auth.is_enabled() {
        tracing::info!("Authentication enabled (Bearer token required)");
    } else {
        tracing::info!("Authentication disabled (open access)");
    }

    // Build the application with routes
    #[allow(unused_mut)]
    let mut api_routes = Router::new()
        .route("/health", get(health_check))
        .merge(api::routes(config.upload.max_size_bytes))
        .merge(websocket::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
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
        start_with_tls(
            config,
            app,
            {
                #[cfg(feature = "acme")]
                {
                    acme_manager
                }
                #[cfg(not(feature = "acme"))]
                {
                    None::<()>
                }
            },
            parent_shutdown_rx,
        )
        .await;
    } else {
        start_http_only(config, app, parent_shutdown_rx).await;
    }

    #[cfg(not(feature = "tls"))]
    start_http_only(config, app, parent_shutdown_rx).await;
}

/// Start HTTP-only server (no TLS).
async fn start_http_only(
    config: ServerConfig,
    app: Router,
    parent_shutdown_rx: Option<tokio::sync::broadcast::Receiver<()>>,
) {
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
        .with_graceful_shutdown(shutdown_signal(parent_shutdown_rx))
        .await
        .expect("Server failed");
}

/// Start server with TLS support.
#[cfg(feature = "tls")]
async fn start_with_tls<T>(
    config: ServerConfig,
    app: Router,
    acme_manager: Option<T>,
    parent_shutdown_rx: Option<tokio::sync::broadcast::Receiver<()>>,
) where
    T: std::any::Any + Send + Sync + 'static,
{
    #[cfg(feature = "acme")]
    use std::any::Any;

    let tls_addr = config.tls_socket_addr().unwrap_or_else(|err| {
        eprintln!("Invalid TLS listen address: {}", err);
        std::process::exit(1);
    });

    // For ACME, we need to start the HTTP server BEFORE attempting certificate provisioning
    // because Let's Encrypt will validate the challenge on port 80
    #[cfg(feature = "acme")]
    let acme_challenges = if let Some(ref manager) = acme_manager
        && let Some(acme) = (manager as &dyn Any).downcast_ref::<Arc<AcmeManager>>()
    {
        acme.pending_challenges()
    } else {
        std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()))
    };

    // Start HTTP server for ACME challenges before certificate provisioning
    #[cfg(feature = "acme")]
    if config.tls.redirect_http {
        let http_addr = config.socket_addr().unwrap_or_else(|err| {
            eprintln!("Invalid HTTP listen address: {}", err);
            std::process::exit(1);
        });
        let https_port = config.tls.port;
        let challenges = acme_challenges.clone();

        tokio::spawn(async move {
            run_http_redirect_server(http_addr, https_port, challenges).await;
        });

        // Give the HTTP server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

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

    // For non-ACME builds, start HTTP redirect server after certificate is loaded
    #[cfg(not(feature = "acme"))]
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
    if let Some(ref manager) = acme_manager
        && let Some(acme) = (manager as &dyn Any).downcast_ref::<Arc<AcmeManager>>()
    {
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

    // Start periodic certificate reload task for manually managed certificates
    if let Some(interval) = config.tls.reload_interval()
        && let (Some(cert_path), Some(key_path)) =
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

    tracing::info!("HTTPS server listening on {}", tls_addr);

    // Create a handle for graceful shutdown
    let handle = axum_server::Handle::new();
    let shutdown_handle = handle.clone();

    // Spawn shutdown signal listener
    tokio::spawn(async move {
        shutdown_signal(parent_shutdown_rx).await;
        shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
    });

    axum_server::bind_rustls(tls_addr, rustls_config)
        .handle(handle)
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
    if config.acme.enabled
        && let Some(manager) = acme_manager
        && let Some(acme) = (manager as &dyn Any).downcast_ref::<Arc<AcmeManager>>()
    {
        match acme.provision_certificate().await {
            Ok((cert, key)) => return (cert, key),
            Err(e) => {
                tracing::error!("ACME certificate provisioning failed: {}", e);
                // Fall through to manual cert or self-signed
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
/// Note: This variant does NOT handle ACME challenges - use run_http_redirect_server_with_acme instead.
#[cfg(all(feature = "tls", not(feature = "acme")))]
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

/// Run HTTP to HTTPS redirect server with ACME challenge support.
/// ACME HTTP-01 challenges are served on port 80, all other requests are redirected to HTTPS.
#[cfg(all(feature = "tls", feature = "acme"))]
async fn run_http_redirect_server(
    http_addr: std::net::SocketAddr,
    https_port: u16,
    acme_challenges: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
) {
    use axum::extract::Path;
    use axum::response::{IntoResponse, Redirect};

    // Handler for ACME challenges
    let challenge_handler = {
        let challenges = acme_challenges.clone();
        move |Path(token): Path<String>| {
            let challenges = challenges.clone();
            async move {
                let challenges = challenges.read().await;
                if let Some(key_auth) = challenges.get(&token) {
                    tracing::debug!("Responding to ACME challenge for token: {}", token);
                    (StatusCode::OK, key_auth.clone()).into_response()
                } else {
                    tracing::warn!("Unknown ACME challenge token: {}", token);
                    (StatusCode::NOT_FOUND, "Challenge not found").into_response()
                }
            }
        }
    };

    // Redirect handler for all other requests
    let redirect_handler = move |uri: Uri| async move {
        let host = uri.host().unwrap_or("localhost");
        let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

        let https_uri = if https_port == 443 {
            format!("https://{}{}", host, path_and_query)
        } else {
            format!("https://{}:{}{}", host, https_port, path_and_query)
        };

        Redirect::permanent(&https_uri)
    };

    let redirect_app = Router::new()
        .route(
            "/.well-known/acme-challenge/{token}",
            get(challenge_handler),
        )
        .fallback(redirect_handler);

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
        "HTTP redirect server listening on {} -> HTTPS port {} (with ACME challenge support)",
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

async fn shutdown_signal(parent_shutdown_rx: Option<tokio::sync::broadcast::Receiver<()>>) {
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

    let parent_exit = async {
        if let Some(mut rx) = parent_shutdown_rx {
            let _ = rx.recv().await;
        } else {
            std::future::pending::<()>().await;
        }
    };

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, starting graceful shutdown");
        },
        _ = parent_exit => {
            tracing::info!("Shutdown signal received from parent, starting graceful shutdown");
        },
    }
}
