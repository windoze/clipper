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

use server::{ServerManager, get_server_data_dir};
use settings::{SettingsManager, get_app_config_dir};
use state::AppState;
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{DragDropEvent, Emitter, Manager, RunEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Single instance plugin must be registered FIRST
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
                    eprintln!("[clipper] Migration warning: {}", e);
                }
            });

            // Initialize settings manager
            eprintln!("[clipper] Config directory: {}", config_dir.display());
            let settings_manager = SettingsManager::new(config_dir);

            // Load settings synchronously during setup
            let app_handle = app.handle().clone();
            let settings_manager_clone = settings_manager.clone();
            tauri::async_runtime::block_on(async move {
                if let Err(e) = settings_manager_clone.init().await {
                    eprintln!("Failed to load settings: {}", e);
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
                            eprintln!("Bundled server started at: {}", url);
                            url
                        }
                        Err(e) => {
                            eprintln!(
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
                eprintln!("Using external server at: {}", external_url);
                (external_url, external_token)
            };

            // Register server manager
            app.manage(server_manager);

            // Create app state with the server URL and token
            let app_state = AppState::new_with_token(&server_url, token);
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

            // Setup system tray with language from settings
            let settings_for_tray = settings_manager.get();
            let tray_language = settings_for_tray.language.as_deref().unwrap_or("en");
            if let Err(e) = tray::setup_tray(app.handle(), tray_language) {
                eprintln!("Failed to setup tray: {}", e);
            }

            // Start clipboard monitoring
            clipboard::start_clipboard_monitor(app.handle().clone());

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
                eprintln!(
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
                                    eprintln!("Failed to read file metadata: {}", e);
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
                                eprintln!(
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
                                            eprintln!("Failed to upload dropped file: {}", e);
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
                                    eprintln!("Failed to read dropped file: {}", e);
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
            commands::switch_to_bundled_server,
            commands::switch_to_external_server,
            commands::get_local_ip_addresses,
            commands::toggle_listen_on_all_interfaces,
            commands::update_tray_language,
            commands::update_global_shortcut,
            commands::get_websocket_status,
            commands::get_server_info,
            commands::get_max_upload_size_bytes,
            commands::check_for_updates,
            commands::install_update,
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
                    eprintln!("Failed to stop bundled server: {}", e);
                }
            });
        }
    });
}
