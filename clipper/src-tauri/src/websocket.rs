use crate::clipboard::set_clipboard_content;
use crate::state::AppState;
use clipper_client::ClipNotification;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

pub async fn start_websocket_listener(app: AppHandle) {
    let state = app.state::<AppState>();
    let client = state.client().clone();

    loop {
        let (tx, mut rx) = mpsc::unbounded_channel::<ClipNotification>();

        match client.subscribe_notifications(tx).await {
            Ok(handle) => {
                while let Some(notification) = rx.recv().await {
                    match &notification {
                        ClipNotification::NewClip { id, content, tags } => {
                            // Update system clipboard with new content
                            if let Err(e) = set_clipboard_content(content) {
                                eprintln!("Failed to set clipboard: {}", e);
                            } else {
                                // Update last synced content to prevent loop
                                state.set_last_synced_content(content.clone());
                            }

                            // Emit event to frontend
                            let _ = app.emit("new-clip", serde_json::json!({
                                "id": id,
                                "content": content,
                                "tags": tags
                            }));
                        }
                        ClipNotification::UpdatedClip { id } => {
                            let _ = app.emit("clip-updated", serde_json::json!({ "id": id }));
                        }
                        ClipNotification::DeletedClip { id } => {
                            let _ = app.emit("clip-deleted", serde_json::json!({ "id": id }));
                        }
                    }
                }

                // Wait for the handle to complete
                let _ = handle.await;
            }
            Err(e) => {
                eprintln!("Failed to connect to WebSocket: {}", e);
            }
        }

        // Wait before reconnecting
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        eprintln!("Reconnecting to WebSocket...");
    }
}
