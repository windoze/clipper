mod autolaunch;
mod clipboard;
mod commands;
mod migration;
mod server;
mod settings;
mod state;
mod tray;
mod tray_i18n;
mod websocket;

use log::{error, info, warn};
use server::{ServerManager, get_server_data_dir};
use settings::{MainWindowGeometry, SETTINGS_FILE_NAME, SettingsManager, get_app_config_dir};
use state::AppState;
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{DragDropEvent, Emitter, Manager, RunEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// Read the debug_logging setting from the settings file before the app is fully initialized.
/// This is needed because the log plugin must be configured before the settings manager is available.
fn read_debug_logging_setting() -> bool {
    // Try to find the settings file in the standard config location
    if let Some(config_dir) = dirs::config_dir() {
        let settings_path = config_dir
            .join("codes.unwritten.clipper")
            .join(SETTINGS_FILE_NAME);
        if settings_path.exists()
            && let Ok(contents) = std::fs::read_to_string(&settings_path)
            && let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents)
            && let Some(debug_logging) = json.get("debug_logging").and_then(|v| v.as_bool())
        {
            return debug_logging;
        }
    }
    false // Default to false if setting not found
}

/// Payload for single-instance events
#[derive(Clone, serde::Serialize)]
struct SingleInstancePayload {
    args: Vec<String>,
    cwd: String,
}

/// Parse a shortcut string like "Command+Shift+V" or "Ctrl+Alt+C" into a Shortcut
pub fn parse_shortcut(shortcut_str: &str) -> Option<Shortcut> {
    let parts: Vec<&str> = shortcut_str.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "command" | "cmd" | "super" | "meta" => modifiers |= Modifiers::SUPER,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            // Single character keys
            key if key.len() == 1 => {
                let c = key.chars().next().unwrap().to_ascii_uppercase();
                key_code = match c {
                    'A' => Some(Code::KeyA),
                    'B' => Some(Code::KeyB),
                    'C' => Some(Code::KeyC),
                    'D' => Some(Code::KeyD),
                    'E' => Some(Code::KeyE),
                    'F' => Some(Code::KeyF),
                    'G' => Some(Code::KeyG),
                    'H' => Some(Code::KeyH),
                    'I' => Some(Code::KeyI),
                    'J' => Some(Code::KeyJ),
                    'K' => Some(Code::KeyK),
                    'L' => Some(Code::KeyL),
                    'M' => Some(Code::KeyM),
                    'N' => Some(Code::KeyN),
                    'O' => Some(Code::KeyO),
                    'P' => Some(Code::KeyP),
                    'Q' => Some(Code::KeyQ),
                    'R' => Some(Code::KeyR),
                    'S' => Some(Code::KeyS),
                    'T' => Some(Code::KeyT),
                    'U' => Some(Code::KeyU),
                    'V' => Some(Code::KeyV),
                    'W' => Some(Code::KeyW),
                    'X' => Some(Code::KeyX),
                    'Y' => Some(Code::KeyY),
                    'Z' => Some(Code::KeyZ),
                    '0' => Some(Code::Digit0),
                    '1' => Some(Code::Digit1),
                    '2' => Some(Code::Digit2),
                    '3' => Some(Code::Digit3),
                    '4' => Some(Code::Digit4),
                    '5' => Some(Code::Digit5),
                    '6' => Some(Code::Digit6),
                    '7' => Some(Code::Digit7),
                    '8' => Some(Code::Digit8),
                    '9' => Some(Code::Digit9),
                    _ => None,
                };
            }
            // Named keys
            "space" => key_code = Some(Code::Space),
            "enter" | "return" => key_code = Some(Code::Enter),
            "tab" => key_code = Some(Code::Tab),
            "escape" | "esc" => key_code = Some(Code::Escape),
            "backspace" => key_code = Some(Code::Backspace),
            "delete" => key_code = Some(Code::Delete),
            "up" => key_code = Some(Code::ArrowUp),
            "down" => key_code = Some(Code::ArrowDown),
            "left" => key_code = Some(Code::ArrowLeft),
            "right" => key_code = Some(Code::ArrowRight),
            "home" => key_code = Some(Code::Home),
            "end" => key_code = Some(Code::End),
            "pageup" => key_code = Some(Code::PageUp),
            "pagedown" => key_code = Some(Code::PageDown),
            "f1" => key_code = Some(Code::F1),
            "f2" => key_code = Some(Code::F2),
            "f3" => key_code = Some(Code::F3),
            "f4" => key_code = Some(Code::F4),
            "f5" => key_code = Some(Code::F5),
            "f6" => key_code = Some(Code::F6),
            "f7" => key_code = Some(Code::F7),
            "f8" => key_code = Some(Code::F8),
            "f9" => key_code = Some(Code::F9),
            "f10" => key_code = Some(Code::F10),
            "f11" => key_code = Some(Code::F11),
            "f12" => key_code = Some(Code::F12),
            _ => {}
        }
    }

    key_code.map(|code| {
        if modifiers.is_empty() {
            Shortcut::new(None, code)
        } else {
            Shortcut::new(Some(modifiers), code)
        }
    })
}

/// Check certificate on startup and emit event if trust is required
async fn check_certificate_on_startup(
    app: &tauri::AppHandle,
    server_url: &str,
    settings_manager: &SettingsManager,
) {
    use clipper_client::fetch_server_certificate;

    // Parse URL to get host and port
    if let Ok(url) = tauri::Url::parse(server_url)
        && let Some(host) = url.host_str()
    {
        let port = url.port().unwrap_or(443);

        match fetch_server_certificate(host, port).await {
            Ok(cert_info) => {
                let fingerprint = cert_info.fingerprint.clone();
                let is_system_trusted = cert_info.is_system_trusted;

                // Check if we have a stored fingerprint for this host
                let stored_fingerprint = settings_manager.get_stored_fingerprint(host);
                let is_user_trusted = settings_manager.is_certificate_trusted(host, &fingerprint);

                // CRITICAL: Check for fingerprint mismatch - potential MITM attack
                let fingerprint_mismatch = stored_fingerprint
                    .as_ref()
                    .map(|stored| stored != &fingerprint)
                    .unwrap_or(false);

                if fingerprint_mismatch {
                    // This is a critical security warning - fingerprint changed!
                    warn!(
                        "CRITICAL: Certificate fingerprint mismatch on startup for {}!",
                        host
                    );
                    let _ = app.emit(
                        "certificate-fingerprint-mismatch",
                        serde_json::json!({
                            "host": host,
                            "fingerprint": fingerprint,
                            "storedFingerprint": stored_fingerprint,
                            "isTrusted": false
                        }),
                    );
                    return;
                }

                // Only emit trust required if certificate is not trusted at all
                if !is_system_trusted && !is_user_trusted {
                    info!("Certificate trust required on startup for {}", host);
                    let _ = app.emit(
                        "certificate-trust-required",
                        serde_json::json!({
                            "host": host,
                            "fingerprint": fingerprint,
                            "isTrusted": false
                        }),
                    );
                }
            }
            Err(e) => {
                warn!("Failed to fetch certificate for {} on startup: {}", host, e);
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set restrictive permissions for newly created files and directories.
    // On Unix: Sets umask to 0o077 (files 0600, directories 0700)
    // On Windows: This is a no-op; directories are secured after creation with ACLs
    clipper_security::set_restrictive_umask();

    // Install the ring crypto provider for rustls at startup
    // This is required for TLS certificate operations
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Read debug_logging setting early, before the app is fully initialized
    let debug_logging_enabled = read_debug_logging_setting();

    // Build log file target with appropriate filter based on debug_logging setting
    let log_file_target = if debug_logging_enabled {
        // Debug logging enabled: write all logs including DEBUG to file
        tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
            file_name: Some("clipper".into()),
        })
    } else {
        // Default: only INFO and above to file (no DEBUG logs)
        tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
            file_name: Some("clipper".into()),
        })
        .filter(|metadata| metadata.level() <= log::Level::Info)
    };

    // Use larger log file size when debug logging is enabled (10MB vs 1MB)
    let max_log_file_size = if debug_logging_enabled {
        10_000_000 // 10MB when debug logging enabled
    } else {
        1_000_000 // 1MB default
    };

    let app = tauri::Builder::default()
        // Log plugin should be registered early to capture all logs
        .plugin(
            tauri_plugin_log::Builder::new()
                // Clear default targets (Stdout + LogDir) to avoid duplicates
                .clear_targets()
                .targets([
                    // Stdout: show all logs including debug (for development)
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    // Log file: filtered based on debug_logging setting
                    log_file_target,
                    // Webview: show all logs for frontend debugging
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                // Allow debug logs globally (filtered per-target above)
                .level(log::LevelFilter::Debug)
                // Rotate logs: keep only one backup file
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .max_file_size(max_log_file_size)
                .build(),
        )
        // Single instance plugin must be registered FIRST (after log)
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // When a second instance is launched, show the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                #[cfg(target_os = "macos")]
                let _ = app.set_activation_policy(ActivationPolicy::Regular);
            }
            // Emit event to frontend with the args from the second instance
            let _ = app.emit("single-instance", SingleInstancePayload { args: argv, cwd });
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            // Get directories for migration and settings
            let config_dir = get_app_config_dir(app.handle())?;
            let data_dir = get_server_data_dir(app.handle())?;

            // Run migration from old app identifier if needed
            tauri::async_runtime::block_on(async {
                if let Err(e) = migration::migrate_from_old_location(&config_dir, &data_dir).await {
                    warn!("Migration warning: {}", e);
                }
            });

            // Initialize settings manager
            info!("Config directory: {}", config_dir.display());
            if debug_logging_enabled {
                info!("Debug logging to file enabled (set debug_logging: false in settings.json to disable)");
            }
            let settings_manager = SettingsManager::new(config_dir);

            // Load settings synchronously during setup
            let app_handle = app.handle().clone();
            let settings_manager_clone = settings_manager.clone();
            tauri::async_runtime::block_on(async move {
                if let Err(e) = settings_manager_clone.init().await {
                    error!("Failed to load settings: {}", e);
                }
            });

            // Register settings manager BEFORE starting server (server needs it for port persistence)
            app.manage(settings_manager.clone());

            // Get the server data directory for the bundled server
            let server_data_dir = get_server_data_dir(app.handle())?;
            let server_manager = ServerManager::new(server_data_dir);

            // Check if we should use bundled server based on settings
            let use_bundled = settings_manager.get().use_bundled_server;

            // Start the bundled server if enabled, or use external server URL
            let app_handle_for_server = app.handle().clone();
            let (server_url, token) = if use_bundled {
                // Get token for bundled server - always use if set (server requires it when configured)
                let bundled_token = settings_manager.get_bundled_server_token();

                let url = tauri::async_runtime::block_on(async {
                    match server_manager.start(&app_handle_for_server).await {
                        Ok(url) => {
                            info!("Bundled server started at: {}", url);
                            url
                        }
                        Err(e) => {
                            error!(
                                "Failed to start bundled server: {}. Falling back to settings.",
                                e
                            );
                            // Fall back to settings if bundled server fails
                            settings_manager.get().server_address.clone()
                        }
                    }
                });
                (url, bundled_token)
            } else {
                let external_url = settings_manager.get().server_address.clone();
                let external_token = settings_manager.get_external_server_token();
                info!("Using external server at: {}", external_url);
                (external_url, external_token)
            };

            // Register server manager
            app.manage(server_manager);

            // Create app state with the server URL, token, and trusted certificates
            let trusted_certs = settings_manager.get_trusted_certificates();
            let app_state = AppState::new_with_trusted_certs(&server_url, token, trusted_certs);
            app.manage(app_state);

            // Handle window visibility based on settings
            let settings = settings_manager.get();
            if !settings.open_on_startup
                && let Some(window) = app_handle.get_webview_window("main")
            {
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                let _ = app_handle.set_activation_policy(ActivationPolicy::Accessory);
            }

            // Restore main window geometry from settings
            if let Some(window) = app_handle.get_webview_window("main") {
                let geometry = settings_manager.get_main_window_geometry();

                // Restore size if saved
                if let (Some(width), Some(height)) = (geometry.width, geometry.height) {
                    let size = tauri::LogicalSize::new(width as f64, height as f64);
                    let _ = window.set_size(size);
                }

                // Restore position if saved
                if let (Some(x), Some(y)) = (geometry.x, geometry.y) {
                    let position = tauri::LogicalPosition::new(x as f64, y as f64);
                    let _ = window.set_position(position);
                }

                // Restore maximized state if saved
                if let Some(true) = geometry.maximized {
                    let _ = window.maximize();
                }
            }

            // Setup system tray with language from settings
            let settings_for_tray = settings_manager.get();
            let tray_language = settings_for_tray.language.as_deref().unwrap_or("en");
            if let Err(e) = tray::setup_tray(app.handle(), tray_language) {
                error!("Failed to setup tray: {}", e);
            }

            // Start clipboard monitoring
            clipboard::start_clipboard_monitor(app.handle().clone());

            // Check certificate on startup for external HTTPS servers
            // This runs in background and emits event to frontend if trust is required
            if !use_bundled && server_url.starts_with("https://") {
                let app_handle_cert = app.handle().clone();
                let server_url_cert = server_url.clone();
                let settings_manager_cert = settings_manager.clone();
                tauri::async_runtime::spawn(async move {
                    // Small delay to ensure frontend is ready to receive events
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    check_certificate_on_startup(
                        &app_handle_cert,
                        &server_url_cert,
                        &settings_manager_cert,
                    )
                    .await;
                });
            }

            // Start WebSocket listener
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                websocket::start_websocket_listener(app_handle).await;
            });

            // Handle window close - hide instead of quit
            let window = app.get_webview_window("main").unwrap();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    // Hide window instead of closing
                    // Note: window needs to be accessed differently
                }
            });

            // Register global shortcut to toggle window visibility
            // Use shortcut from settings, or default to Cmd+Shift+V on macOS, Ctrl+Shift+V on others
            let shortcut_str = settings_manager.get().global_shortcut;
            let shortcut = parse_shortcut(&shortcut_str).unwrap_or_else(|| {
                #[cfg(target_os = "macos")]
                {
                    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV)
                }
            });

            let app_handle = app.handle().clone();
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, _shortcut, event| {
                        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed
                            && let Some(window) = app_handle.get_webview_window("main")
                        {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                                #[cfg(target_os = "macos")]
                                let _ =
                                    app_handle.set_activation_policy(ActivationPolicy::Accessory);
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                                #[cfg(target_os = "macos")]
                                let _ = app_handle.set_activation_policy(ActivationPolicy::Regular);
                            }
                        }
                    })
                    .build(),
            )?;

            // Register the shortcut
            if let Err(e) = app.global_shortcut().register(shortcut) {
                error!(
                    "Failed to register global shortcut '{}': {}",
                    shortcut_str, e
                );
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                    // Hide dock icon on macOS when window is closed
                    #[cfg(target_os = "macos")]
                    let _ = window
                        .app_handle()
                        .set_activation_policy(ActivationPolicy::Accessory);
                }
                tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                    // Save window geometry when moved or resized
                    // Only save for the main window
                    if window.label() == "main" {
                        let app = window.app_handle().clone();
                        let window_clone = window.clone();
                        tauri::async_runtime::spawn(async move {
                            // Get current window state
                            let is_maximized = window_clone.is_maximized().unwrap_or(false);

                            // Only save geometry if not maximized (to preserve the normal window size)
                            let geometry = if is_maximized {
                                MainWindowGeometry {
                                    width: None,
                                    height: None,
                                    x: None,
                                    y: None,
                                    maximized: Some(true),
                                }
                            } else {
                                let size = window_clone.outer_size().ok();
                                let position = window_clone.outer_position().ok();
                                let scale_factor = window_clone.scale_factor().unwrap_or(1.0);

                                MainWindowGeometry {
                                    width: size.map(|s| (s.width as f64 / scale_factor) as u32),
                                    height: size.map(|s| (s.height as f64 / scale_factor) as u32),
                                    x: position.map(|p| (p.x as f64 / scale_factor) as i32),
                                    y: position.map(|p| (p.y as f64 / scale_factor) as i32),
                                    maximized: Some(false),
                                }
                            };

                            let settings_manager = app.state::<SettingsManager>();
                            if let Err(e) = settings_manager.save_main_window_geometry(geometry).await {
                                log::warn!("Failed to save window geometry: {}", e);
                            }
                        });
                    }
                }
                tauri::WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) => {
                    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

                    let app = window.app_handle().clone();
                    let state = app.state::<AppState>();
                    let client = state.client();
                    let paths = paths.clone();

                    // Process all files sequentially in a single async task to avoid race conditions
                    tauri::async_runtime::spawn(async move {
                        for path in paths {
                            // Check file size first
                            let metadata = match tokio::fs::metadata(&path).await {
                                Ok(m) => m,
                                Err(e) => {
                                    log::error!("Failed to read file metadata: {}", e);
                                    let _ = app.emit(
                                        "file-upload-error",
                                        serde_json::json!({
                                            "path": path.to_string_lossy(),
                                            "error": format!("Failed to read file: {}", e)
                                        }),
                                    );
                                    continue;
                                }
                            };

                            if metadata.len() > MAX_FILE_SIZE {
                                let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                                log::warn!(
                                    "File too large: {} ({:.1} MB, max {} MB)",
                                    path.display(),
                                    size_mb,
                                    MAX_FILE_SIZE / (1024 * 1024)
                                );
                                let _ = app.emit(
                                    "file-upload-error",
                                    serde_json::json!({
                                        "path": path.to_string_lossy(),
                                        "error": "file_too_large",
                                        "size_mb": size_mb,
                                        "max_size_mb": MAX_FILE_SIZE / (1024 * 1024)
                                    }),
                                );
                                continue;
                            }

                            match tokio::fs::read(&path).await {
                                Ok(bytes) => {
                                    let filename = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string();

                                    // Use full path as content
                                    let full_path = path.to_string_lossy().to_string();

                                    match client
                                        .upload_file_bytes_with_content(
                                            bytes,
                                            filename,
                                            vec!["$file".to_string()],
                                            None,
                                            Some(full_path),
                                        )
                                        .await
                                    {
                                        Ok(clip) => {
                                            let _ = app.emit("clip-created", &clip);
                                        }
                                        Err(e) => {
                                            log::error!("Failed to upload dropped file: {}", e);
                                            let _ = app.emit(
                                                "file-upload-error",
                                                serde_json::json!({
                                                    "path": path.to_string_lossy(),
                                                    "error": format!("Upload failed: {}", e)
                                                }),
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to read dropped file: {}", e);
                                    let _ = app.emit(
                                        "file-upload-error",
                                        serde_json::json!({
                                            "path": path.to_string_lossy(),
                                            "error": format!("Failed to read file: {}", e)
                                        }),
                                    );
                                }
                            }
                        }
                    });
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_clips,
            commands::search_clips,
            commands::create_clip,
            commands::update_clip,
            commands::delete_clip,
            commands::get_clip,
            commands::copy_to_clipboard,
            commands::copy_image_to_clipboard,
            commands::upload_file,
            commands::get_file_url,
            commands::download_file,
            commands::get_settings,
            commands::save_settings,
            commands::browse_directory,
            commands::check_auto_launch_status,
            commands::get_server_url,
            commands::is_bundled_server,
            commands::clear_all_data,
            commands::export_clips,
            commands::import_clips,
            commands::switch_to_bundled_server,
            commands::switch_to_external_server,
            commands::get_local_ip_addresses,
            commands::toggle_listen_on_all_interfaces,
            commands::update_tray_language,
            commands::update_global_shortcut,
            commands::get_websocket_status,
            commands::get_server_info,
            commands::get_max_upload_size_bytes,
            commands::get_app_version,
            commands::check_for_updates,
            commands::install_update,
            commands::check_server_certificate,
            commands::trust_certificate,
            commands::untrust_certificate,
            commands::get_trusted_certificates,
            commands::ensure_window_size,
            commands::quit_app,
            commands::restart_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Run the app with exit handler to stop the bundled server
    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            // Stop the bundled server when the app exits
            let server_manager = app_handle.state::<ServerManager>();
            tauri::async_runtime::block_on(async {
                if let Err(e) = server_manager.stop().await {
                    log::error!("Failed to stop bundled server: {}", e);
                }
            });
        }
    });
}
