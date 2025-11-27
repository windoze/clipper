mod autolaunch;
mod clipboard;
mod commands;
mod server;
mod settings;
mod state;
mod tray;
mod tray_i18n;
mod websocket;

use server::{get_server_data_dir, ServerManager};
use settings::{get_app_config_dir, SettingsManager};
use state::AppState;
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{DragDropEvent, Emitter, Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .setup(move |app| {
            // Initialize settings manager
            let config_dir = get_app_config_dir(app.handle())?;
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
            let server_url = if use_bundled {
                tauri::async_runtime::block_on(async {
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
                })
            } else {
                let external_url = settings_manager.get().server_address.clone();
                eprintln!("Using external server at: {}", external_url);
                external_url
            };

            // Register server manager
            app.manage(server_manager);

            // Create app state with the server URL
            let app_state = AppState::new(&server_url);
            app.manage(app_state);

            // Handle window visibility based on settings
            let settings = settings_manager.get();
            if !settings.open_on_startup {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.hide();
                    #[cfg(target_os = "macos")]
                    let _ = app_handle.set_activation_policy(ActivationPolicy::Accessory);
                }
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
                    let app = window.app_handle();
                    let state = app.state::<AppState>();
                    let client = state.client();

                    for path in paths {
                        let path = path.clone();
                        let client = client.clone();
                        let app_handle = app.clone();

                        // Read file and upload
                        tauri::async_runtime::spawn(async move {
                            match tokio::fs::read(&path).await {
                                Ok(bytes) => {
                                    let filename = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string();

                                    match client
                                        .upload_file_bytes(
                                            bytes,
                                            filename,
                                            vec!["$file".to_string()],
                                            None,
                                        )
                                        .await
                                    {
                                        Ok(clip) => {
                                            let _ = app_handle.emit("clip-created", &clip);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to upload dropped file: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to read dropped file: {}", e);
                                }
                            }
                        });
                    }
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
