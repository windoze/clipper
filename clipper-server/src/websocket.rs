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
use tracing::{error, info};

use crate::state::AppState;

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

    // Spawn a task to forward updates to the WebSocket client
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            let json = match serde_json::to_string(&update) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize update: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json.into())).await.is_err() {
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

    info!("WebSocket connection closed");
}
