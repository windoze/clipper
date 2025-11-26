use crate::state::AppState;
use arboard::Clipboard;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const POLL_INTERVAL_MS: u64 = 500;

pub fn start_clipboard_monitor(app: AppHandle) {
    let state = app.state::<AppState>();
    let client = state.client().clone();
    let last_synced = Arc::clone(&state.last_synced_content);
    let last_content = Arc::new(std::sync::Mutex::new(String::new()));

    // Spawn clipboard monitoring task
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                eprintln!("Failed to initialize clipboard: {}", e);
                return;
            }
        };

        // Initialize with current clipboard content
        if let Ok(current) = clipboard.get_text() {
            *last_content.lock().unwrap() = current;
        }

        loop {
            std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));

            let current_content = match clipboard.get_text() {
                Ok(text) => text,
                Err(_) => continue,
            };

            let last = last_content.lock().unwrap().clone();
            let synced = last_synced.lock().unwrap().clone();

            // Skip if content hasn't changed
            if current_content == last {
                continue;
            }

            // Skip if this content was just synced from server (avoid loop)
            if current_content == synced {
                *last_content.lock().unwrap() = current_content;
                continue;
            }

            // Content changed, create a new clip
            *last_content.lock().unwrap() = current_content.clone();

            let client_clone = client.clone();
            let content = current_content.clone();
            let app_handle = app.clone();

            rt.spawn(async move {
                match client_clone.create_clip(content, vec![], None).await {
                    Ok(clip) => {
                        // Emit event to frontend
                        let _ = app_handle.emit("clip-created", &clip);
                    }
                    Err(e) => {
                        eprintln!("Failed to create clip from clipboard: {}", e);
                    }
                }
            });
        }
    });
}

pub fn set_clipboard_content(content: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(content).map_err(|e| e.to_string())
}
