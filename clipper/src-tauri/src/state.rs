use clipper_client::ClipperClient;
use std::sync::{Arc, Mutex, RwLock};

pub struct AppState {
    client: RwLock<ClipperClient>,
    pub last_synced_content: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: RwLock::new(ClipperClient::new(base_url)),
            last_synced_content: Arc::new(Mutex::new(String::new())),
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
}
