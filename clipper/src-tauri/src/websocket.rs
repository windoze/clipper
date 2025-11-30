use crate::clipboard::set_clipboard_content;
use crate::state::AppState;
use clipper_client::ClipNotification;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

/// Emit WebSocket connection status to frontend
fn emit_ws_status(app: &AppHandle, connected: bool) {
    let state = app.state::<AppState>();
    state.set_websocket_connected(connected);
    let _ = app.emit(
        "websocket-status",
        serde_json::json!({ "connected": connected }),
    );
}

pub async fn start_websocket_listener(app: AppHandle) {
    let state = app.state::<AppState>();
    let mut reconnect_delay = 1u64; // Start with 1 second delay

    loop {
        let client = state.client().clone();
        let (tx, mut rx) = mpsc::unbounded_channel::<ClipNotification>();

        // Remember the current reconnect counter to detect changes
        let reconnect_counter_at_connect = state.ws_reconnect_counter();

        match client.subscribe_notifications(tx).await {
            Ok(handle) => {
                // Connected successfully
                emit_ws_status(&app, true);
                reconnect_delay = 1; // Reset delay on successful connection
                eprintln!("WebSocket connected");

                loop {
                    // Check if we should reconnect (e.g., token changed)
                    if state.ws_reconnect_counter() != reconnect_counter_at_connect {
                        eprintln!("WebSocket: reconnect signal received, disconnecting...");
                        handle.abort();
                        break;
                    }

                    // Use a short timeout to periodically check for reconnect signals
                    let recv_result = tokio::time::timeout(
                        tokio::time::Duration::from_millis(500),
                        rx.recv(),
                    )
                    .await;

                    match recv_result {
                        Ok(Some(notification)) => {
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
                                    let _ = app.emit(
                                        "new-clip",
                                        serde_json::json!({
                                            "id": id,
                                            "content": content,
                                            "tags": tags
                                        }),
                                    );
                                }
                                ClipNotification::UpdatedClip { id } => {
                                    let _ = app.emit("clip-updated", serde_json::json!({ "id": id }));
                                }
                                ClipNotification::DeletedClip { id } => {
                                    let _ = app.emit("clip-deleted", serde_json::json!({ "id": id }));
                                }
                                ClipNotification::ClipsCleanedUp { ids, count } => {
                                    let _ = app.emit(
                                        "clips-cleaned-up",
                                        serde_json::json!({
                                            "ids": ids,
                                            "count": count
                                        }),
                                    );
                                }
                            }
                        }
                        Ok(None) => {
                            // Channel closed, connection ended
                            break;
                        }
                        Err(_) => {
                            // Timeout, just continue to check reconnect signal
                            continue;
                        }
                    }
                }

                // Connection closed, mark as disconnected
                emit_ws_status(&app, false);
                eprintln!("WebSocket disconnected");

                // Wait for the handle to complete (if not already aborted)
                let _ = handle.await;
            }
            Err(e) => {
                emit_ws_status(&app, false);
                eprintln!("Failed to connect to WebSocket: {}", e);
            }
        }

        // If reconnect was signaled, reconnect immediately without delay
        if state.ws_reconnect_counter() != reconnect_counter_at_connect {
            eprintln!("Reconnecting to WebSocket immediately (credentials changed)...");
            reconnect_delay = 1;
            continue;
        }

        // Exponential backoff with max delay of 30 seconds
        eprintln!(
            "Reconnecting to WebSocket in {} seconds...",
            reconnect_delay
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(reconnect_delay)).await;
        reconnect_delay = (reconnect_delay * 2).min(30);
    }
}
