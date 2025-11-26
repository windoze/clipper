use clipper_indexer::ClipperIndexer;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub indexer: Arc<ClipperIndexer>,
    pub clip_updates: broadcast::Sender<ClipUpdate>,
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
}

impl AppState {
    pub fn new(indexer: ClipperIndexer) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            indexer: Arc::new(indexer),
            clip_updates: tx,
        }
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
}
