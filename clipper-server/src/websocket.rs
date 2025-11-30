use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::state::AppState;

/// Heartbeat interval - server sends ping every 30 seconds
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Timeout for receiving auth message after connection
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// Authentication request message from client
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },
}

/// Authentication response message to client
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerAuthResponse {
    #[serde(rename = "auth_success")]
    AuthSuccess,
    #[serde(rename = "auth_error")]
    AuthError { message: String },
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/ws", get(websocket_handler))
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Check if authentication is required
    let auth_required = state.config.auth.is_enabled();

    if auth_required {
        // Wait for auth message from client
        info!("WebSocket: waiting for auth message");

        let auth_result = tokio::time::timeout(AUTH_TIMEOUT, async {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        // Try to parse as auth message
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Auth { token }) => {
                                // Validate the token
                                if state.config.auth.validate_token(&token) {
                                    return Ok(());
                                } else {
                                    return Err("Invalid bearer token".to_string());
                                }
                            }
                            Err(e) => {
                                warn!("WebSocket: failed to parse auth message: {}", e);
                                return Err("Invalid auth message format".to_string());
                            }
                        }
                    }
                    Message::Close(_) => {
                        return Err("Client closed connection before auth".to_string());
                    }
                    Message::Ping(_) | Message::Pong(_) => {
                        // Ignore ping/pong during auth phase
                        continue;
                    }
                    _ => {
                        continue;
                    }
                }
            }
            Err("Connection closed before auth".to_string())
        })
        .await;

        match auth_result {
            Ok(Ok(())) => {
                // Auth successful, send success response
                let response = serde_json::to_string(&ServerAuthResponse::AuthSuccess).unwrap();
                if sender.send(Message::Text(response.into())).await.is_err() {
                    error!("WebSocket: failed to send auth success response");
                    return;
                }
                info!("WebSocket: authentication successful");
            }
            Ok(Err(msg)) => {
                // Auth failed, send error response and close
                warn!("WebSocket: authentication failed: {}", msg);
                let response =
                    serde_json::to_string(&ServerAuthResponse::AuthError { message: msg }).unwrap();
                let _ = sender.send(Message::Text(response.into())).await;
                let _ = sender.send(Message::Close(None)).await;
                return;
            }
            Err(_) => {
                // Timeout waiting for auth
                warn!("WebSocket: auth timeout");
                let response = serde_json::to_string(&ServerAuthResponse::AuthError {
                    message: "Auth timeout".to_string(),
                })
                .unwrap();
                let _ = sender.send(Message::Text(response.into())).await;
                let _ = sender.send(Message::Close(None)).await;
                return;
            }
        }
    }

    // Track this connection (only after successful auth)
    state.ws_connect();

    // Subscribe to clip updates
    let mut rx = state.clip_updates.subscribe();

    // Create a channel for sending messages (updates + heartbeat pings)
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<Message>();

    // Clone sender for heartbeat task
    let heartbeat_tx = msg_tx.clone();

    // Spawn heartbeat task - sends ping every HEARTBEAT_INTERVAL
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
        loop {
            interval.tick().await;
            if heartbeat_tx.send(Message::Ping(vec![].into())).is_err() {
                break;
            }
        }
    });

    // Clone sender for updates task
    let updates_tx = msg_tx;

    // Spawn a task to forward updates to the message channel
    let updates_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            let json = match serde_json::to_string(&update) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize update: {}", e);
                    continue;
                }
            };

            if updates_tx.send(Message::Text(json.into())).is_err() {
                break;
            }
        }
    });

    // Spawn a task to send messages from the channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (e.g., ping/pong, client commands)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    info!("Client disconnected");
                    break;
                }
                Message::Ping(data) => {
                    // Respond to pings to keep connection alive
                    // Note: axum automatically handles pong responses
                    info!("Received ping: {:?}", data);
                }
                Message::Pong(_) => {
                    // Client responded to our ping - connection is alive
                    // No action needed, just prevents timeout
                }
                Message::Text(text) => {
                    info!("Received text message: {}", text);
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        },
    }

    // Clean up
    heartbeat_task.abort();
    updates_task.abort();

    // Track disconnection
    state.ws_disconnect();

    info!("WebSocket connection closed");
}
