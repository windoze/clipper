use clipper_client::ClipperClient;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub struct AppState {
    client: RwLock<ClipperClient>,
    pub last_synced_content: Arc<Mutex<String>>,
    pub websocket_connected: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: RwLock::new(ClipperClient::new(base_url)),
            last_synced_content: Arc::new(Mutex::new(String::new())),
            websocket_connected: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn client(&self) -> ClipperClient {
        self.client.read().unwrap().clone()
    }

    pub fn base_url(&self) -> String {
        self.client.read().unwrap().base_url().to_string()
    }

    /// Update the server URL (called when bundled server starts)
    #[allow(dead_code)]
    pub fn set_server_url(&self, url: &str) {
        *self.client.write().unwrap() = ClipperClient::new(url);
    }

    pub fn set_last_synced_content(&self, content: String) {
        *self.last_synced_content.lock().unwrap() = content;
    }

    pub fn set_websocket_connected(&self, connected: bool) {
        self.websocket_connected.store(connected, Ordering::SeqCst);
    }

    pub fn is_websocket_connected(&self) -> bool {
        self.websocket_connected.load(Ordering::SeqCst)
    }
}
