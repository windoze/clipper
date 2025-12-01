use clipper_client::ClipperClient;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub struct AppState {
    client: RwLock<ClipperClient>,
    pub last_synced_content: Arc<Mutex<String>>,
    /// Last synced image content (PNG bytes) to prevent duplicate uploads
    pub last_synced_image: Arc<Mutex<Vec<u8>>>,
    pub websocket_connected: Arc<AtomicBool>,
    /// Counter that increments when WebSocket should reconnect (e.g., token changed)
    pub ws_reconnect_counter: Arc<AtomicU64>,
}

impl AppState {
    /// Create a new AppState with optional Bearer token
    pub fn new_with_token(base_url: &str, token: Option<String>) -> Self {
        let client = match token {
            Some(t) => ClipperClient::new_with_token(base_url, t),
            None => ClipperClient::new(base_url),
        };
        Self {
            client: RwLock::new(client),
            last_synced_content: Arc::new(Mutex::new(String::new())),
            last_synced_image: Arc::new(Mutex::new(Vec::new())),
            websocket_connected: Arc::new(AtomicBool::new(false)),
            ws_reconnect_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn client(&self) -> ClipperClient {
        self.client.read().unwrap().clone()
    }

    pub fn base_url(&self) -> String {
        self.client.read().unwrap().base_url().to_string()
    }

    /// Get the current token (if any)
    pub fn token(&self) -> Option<String> {
        self.client.read().unwrap().token().map(|s| s.to_string())
    }

    /// Update the server URL (called when bundled server starts)
    #[allow(dead_code)]
    pub fn set_server_url(&self, url: &str) {
        *self.client.write().unwrap() = ClipperClient::new(url);
    }

    /// Update the server URL with optional token
    /// This also signals the WebSocket to reconnect
    pub fn set_server_url_with_token(&self, url: &str, token: Option<String>) {
        let client = match token {
            Some(t) => ClipperClient::new_with_token(url, t),
            None => ClipperClient::new(url),
        };
        *self.client.write().unwrap() = client;
        // Signal WebSocket to reconnect with new credentials
        self.signal_ws_reconnect();
    }

    /// Signal the WebSocket listener to reconnect (e.g., after token change)
    pub fn signal_ws_reconnect(&self) {
        self.ws_reconnect_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Get the current reconnect counter value
    pub fn ws_reconnect_counter(&self) -> u64 {
        self.ws_reconnect_counter.load(Ordering::SeqCst)
    }

    pub fn set_last_synced_content(&self, content: String) {
        *self.last_synced_content.lock().unwrap() = content;
    }

    pub fn set_last_synced_image(&self, image_bytes: Vec<u8>) {
        *self.last_synced_image.lock().unwrap() = image_bytes;
    }

    pub fn set_websocket_connected(&self, connected: bool) {
        self.websocket_connected.store(connected, Ordering::SeqCst);
    }

    pub fn is_websocket_connected(&self) -> bool {
        self.websocket_connected.load(Ordering::SeqCst)
    }
}
