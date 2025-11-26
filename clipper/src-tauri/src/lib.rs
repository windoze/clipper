mod clipboard;
mod commands;
mod state;
mod tray;
mod websocket;

use state::AppState;
use std::env;
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{DragDropEvent, Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let base_url = env::var("CLIPPER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(move |app| {
            let app_state = AppState::new(&base_url);
            app.manage(app_state);

            // Setup system tray
            if let Err(e) = tray::setup_tray(app.handle()) {
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
                    let client = state.client().clone();

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
