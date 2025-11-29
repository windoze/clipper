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
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::state::AppState;

/// Heartbeat interval - server sends ping every 30 seconds
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

pub fn routes() -> Router<AppState> {
    Router::new().route("/ws", get(websocket_handler))
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

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

    info!("WebSocket connection closed");
}
