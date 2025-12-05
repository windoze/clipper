use crate::clipboard::{set_clipboard_content, set_clipboard_image};
use crate::settings::SettingsManager;
use crate::state::AppState;
use clipper_client::{fetch_server_certificate, ClipNotification};
use gethostname::gethostname;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

/// Get the hostname tag in the format `$host:<hostname>`
fn get_hostname_tag() -> String {
    let hostname = gethostname().to_string_lossy().to_string();
    format!("$host:{}", hostname)
}

/// Emit WebSocket connection status to frontend
fn emit_ws_status(app: &AppHandle, connected: bool) {
    let state = app.state::<AppState>();
    state.set_websocket_connected(connected);
    let _ = app.emit(
        "websocket-status",
        serde_json::json!({ "connected": connected }),
    );
}

/// Check if the error message indicates a certificate verification failure
fn is_certificate_error(error_msg: &str) -> bool {
    let lower = error_msg.to_lowercase();
    lower.contains("certificate")
        || lower.contains("ca used as end entity")
        || lower.contains("webpki")
        || lower.contains("invalid peer certificate")
        || lower.contains("self signed")
        || lower.contains("selfcertverify")
        || lower.contains("unknownissuer")
}

/// Extract host and port from URL
fn parse_url(url: &str) -> Option<(String, u16)> {
    // Handle URLs that might not have a scheme
    let url_with_scheme = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    tauri::Url::parse(&url_with_scheme).ok().map(|u| {
        let host = u.host_str().unwrap_or("").to_string();
        let port = u.port().unwrap_or(if u.scheme() == "https" { 443 } else { 80 });
        (host, port)
    })
}

/// Check certificate and emit event if trust is required
async fn check_and_emit_certificate_trust(app: &AppHandle, url: &str) -> bool {
    // Only check HTTPS URLs
    if !url.starts_with("https://") {
        return false;
    }

    let settings_manager = app.state::<SettingsManager>();

    if let Some((host, port)) = parse_url(url) {
        match fetch_server_certificate(&host, port).await {
            Ok(cert_info) => {
                let fingerprint = cert_info.fingerprint.clone();
                let is_system_trusted = cert_info.is_system_trusted;
                let is_user_trusted = settings_manager.is_certificate_trusted(&host, &fingerprint);

                // Only emit if certificate is not trusted at all
                if !is_system_trusted && !is_user_trusted {
                    eprintln!("Certificate trust required for {}", host);
                    let _ = app.emit(
                        "certificate-trust-required",
                        serde_json::json!({
                            "host": host,
                            "fingerprint": fingerprint,
                            "isTrusted": false
                        }),
                    );
                    return true;
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch certificate for {}: {}", host, e);
            }
        }
    }

    false
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
                                    // Check if this clip originated from this machine
                                    let my_hostname_tag = get_hostname_tag();
                                    let is_from_this_machine =
                                        tags.iter().any(|t| t == &my_hostname_tag);

                                    // Check if this is an image clip
                                    let is_image_clip = tags.iter().any(|t| t == "$image");

                                    if is_image_clip {
                                        // For image clips from OTHER machines, download and set to clipboard
                                        if !is_from_this_machine {
                                            let client = state.client().clone();
                                            let clip_id = id.clone();
                                            let app_for_image = app.clone();
                                            // Download image in background and set to clipboard
                                            tokio::spawn(async move {
                                                match client.download_file(&clip_id).await {
                                                    Ok(image_bytes) => {
                                                        // Set last synced image BEFORE setting clipboard
                                                        // to prevent the clipboard monitor from uploading it again
                                                        let state =
                                                            app_for_image.state::<AppState>();
                                                        state.set_last_synced_image(
                                                            image_bytes.clone(),
                                                        );

                                                        if let Err(e) =
                                                            set_clipboard_image(&image_bytes)
                                                        {
                                                            eprintln!(
                                                                "Failed to set clipboard image: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to download image for clipboard: {}",
                                                            e
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                        // For image clips from THIS machine, don't touch clipboard
                                        // (the image is already there)
                                    } else {
                                        // For text clips, update system clipboard
                                        if let Err(e) = set_clipboard_content(content) {
                                            eprintln!("Failed to set clipboard: {}", e);
                                        } else {
                                            // Update last synced content to prevent loop
                                            state.set_last_synced_content(content.clone());
                                        }
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
                let error_msg = e.to_string();
                eprintln!("Failed to connect to WebSocket: {}", error_msg);

                // Check if this is a certificate error
                if is_certificate_error(&error_msg) {
                    let base_url = state.base_url();
                    // Check and emit certificate trust event
                    if check_and_emit_certificate_trust(&app, &base_url).await {
                        // Certificate trust is required, wait longer before retrying
                        // to give user time to trust the certificate
                        eprintln!("Waiting for certificate trust before retrying...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                }
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
