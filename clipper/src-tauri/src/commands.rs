use crate::state::AppState;
use chrono::{DateTime, Utc};
use clipper_client::models::PagedResult;
use clipper_client::{Clip, SearchFilters};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFiltersInput {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl SearchFiltersInput {
    pub fn into_search_filters(self) -> SearchFilters {
        let mut filters = SearchFilters::new();

        if let Some(start) = self.start_date {
            if let Ok(dt) = start.parse::<DateTime<Utc>>() {
                filters.start_date = Some(dt);
            }
        }

        if let Some(end) = self.end_date {
            if let Ok(dt) = end.parse::<DateTime<Utc>>() {
                filters.end_date = Some(dt);
            }
        }

        if let Some(tags) = self.tags {
            filters.tags = Some(tags);
        }

        filters
    }
}

#[tauri::command]
pub async fn list_clips(
    state: State<'_, AppState>,
    filters: SearchFiltersInput,
    page: usize,
    page_size: usize,
) -> Result<PagedResult, String> {
    let client = state.client();
    client
        .list_clips(filters.into_search_filters(), page, page_size)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_clips(
    state: State<'_, AppState>,
    query: String,
    filters: SearchFiltersInput,
    page: usize,
    page_size: usize,
) -> Result<PagedResult, String> {
    let client = state.client();
    client
        .search_clips(&query, filters.into_search_filters(), page, page_size)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_clip(
    state: State<'_, AppState>,
    content: String,
    tags: Vec<String>,
    additional_notes: Option<String>,
) -> Result<Clip, String> {
    let client = state.client();
    client
        .create_clip(content, tags, additional_notes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_clip(
    state: State<'_, AppState>,
    id: String,
    tags: Option<Vec<String>>,
    additional_notes: Option<String>,
) -> Result<Clip, String> {
    let client = state.client();
    client
        .update_clip(&id, tags, additional_notes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clip(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let client = state.client();
    client.delete_clip(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_clip(state: State<'_, AppState>, id: String) -> Result<Clip, String> {
    let client = state.client();
    client.get_clip(&id).await.map_err(|e| e.to_string())
}

/// Copy content to clipboard without creating a new clip on the server.
/// This marks the content as "synced" so the clipboard monitor won't create a duplicate.
#[tauri::command]
pub fn copy_to_clipboard(state: State<'_, AppState>, content: String) -> Result<(), String> {
    use arboard::Clipboard;

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&content).map_err(|e| e.to_string())?;

    // Mark this content as synced to prevent clipboard monitor from creating a duplicate
    state.set_last_synced_content(content);

    Ok(())
}

/// Upload a file to create a clip entry
#[tauri::command]
pub async fn upload_file(
    state: State<'_, AppState>,
    path: PathBuf,
    tags: Vec<String>,
    additional_notes: Option<String>,
) -> Result<Clip, String> {
    // Read file bytes
    let bytes = fs::read(&path).await.map_err(|e| e.to_string())?;

    // Get filename from path
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let client = state.client();
    client
        .upload_file_bytes(bytes, filename, tags, additional_notes)
        .await
        .map_err(|e| e.to_string())
}
