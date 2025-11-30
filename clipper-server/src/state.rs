use clipper_indexer::ClipperIndexer;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;

use crate::config::ServerConfig;

#[derive(Clone)]
pub struct AppState {
    pub indexer: Arc<ClipperIndexer>,
    pub clip_updates: broadcast::Sender<ClipUpdate>,
    /// Server start time for uptime calculation
    pub start_time: Instant,
    /// Number of active WebSocket connections
    pub ws_connection_count: Arc<AtomicUsize>,
    /// Server configuration
    pub config: Arc<ServerConfig>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClipUpdate {
    NewClip {
        id: String,
        content: String,
        tags: Vec<String>,
    },
    UpdatedClip {
        id: String,
    },
    DeletedClip {
        id: String,
    },
    ClipsCleanedUp {
        ids: Vec<String>,
        count: usize,
    },
}

impl AppState {
    pub fn new(indexer: ClipperIndexer, config: ServerConfig) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            indexer: Arc::new(indexer),
            clip_updates: tx,
            start_time: Instant::now(),
            ws_connection_count: Arc::new(AtomicUsize::new(0)),
            config: Arc::new(config),
        }
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get the current number of active WebSocket connections
    pub fn active_ws_connections(&self) -> usize {
        self.ws_connection_count.load(Ordering::Relaxed)
    }

    /// Increment WebSocket connection count
    pub fn ws_connect(&self) {
        self.ws_connection_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement WebSocket connection count
    pub fn ws_disconnect(&self) {
        self.ws_connection_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn notify_new_clip(&self, id: String, content: String, tags: Vec<String>) {
        let _ = self
            .clip_updates
            .send(ClipUpdate::NewClip { id, content, tags });
    }

    pub fn notify_updated_clip(&self, id: String) {
        let _ = self.clip_updates.send(ClipUpdate::UpdatedClip { id });
    }

    pub fn notify_deleted_clip(&self, id: String) {
        let _ = self.clip_updates.send(ClipUpdate::DeletedClip { id });
    }

    pub fn notify_clips_cleaned_up(&self, ids: Vec<String>) {
        let count = ids.len();
        let _ = self
            .clip_updates
            .send(ClipUpdate::ClipsCleanedUp { ids, count });
    }
}
