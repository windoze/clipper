use clipper_client::ClipperClient;
use std::sync::{Arc, Mutex};

pub struct AppState {
    client: ClipperClient,
    pub last_synced_content: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: ClipperClient::new(base_url),
            last_synced_content: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn client(&self) -> &ClipperClient {
        &self.client
    }

    pub fn set_last_synced_content(&self, content: String) {
        *self.last_synced_content.lock().unwrap() = content;
    }
}
