use crate::autolaunch;
use crate::server::ServerManager;
use crate::settings::{Settings, SettingsManager};
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

/// Get the URL for a clip's file attachment
#[tauri::command]
pub fn get_file_url(state: State<'_, AppState>, clip_id: String) -> String {
    format!("{}/clips/{}/file", state.base_url(), clip_id)
}

/// Download a clip's file attachment and save it to a user-selected location
#[tauri::command]
pub async fn download_file(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    clip_id: String,
    filename: String,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    // Show save dialog (blocking is safe in async command context)
    let file_path = app
        .dialog()
        .file()
        .set_file_name(&filename)
        .blocking_save_file();

    let save_path = match file_path {
        Some(path) => path,
        None => return Err("Save cancelled".to_string()),
    };

    // Download the file
    let client = state.client();
    let bytes = client
        .download_file(&clip_id)
        .await
        .map_err(|e| e.to_string())?;

    // Write to file
    let path_str = save_path.to_string();
    fs::write(&path_str, bytes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(path_str)
}

// ============ Settings Commands ============

/// Get the current settings
#[tauri::command]
pub fn get_settings(settings_manager: State<'_, SettingsManager>) -> Settings {
    settings_manager.get()
}

/// Save settings
#[tauri::command]
pub async fn save_settings(
    settings_manager: State<'_, SettingsManager>,
    settings: Settings,
) -> Result<(), String> {
    // Handle auto-launch setting change
    let current = settings_manager.get();
    if current.start_on_login != settings.start_on_login {
        autolaunch::set_auto_launch(settings.start_on_login).await?;
    }

    settings_manager.update(settings).await
}

/// Browse for a directory (for default save location)
#[tauri::command]
pub async fn browse_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let folder = app.dialog().file().blocking_pick_folder();

    Ok(folder.map(|p| p.to_string()))
}

/// Check if auto-launch is currently enabled (from system, not settings)
#[tauri::command]
pub fn check_auto_launch_status() -> Result<bool, String> {
    autolaunch::is_auto_launch_enabled()
}

/// Get the current server URL (from bundled server or settings)
#[tauri::command]
pub async fn get_server_url(
    server_manager: State<'_, ServerManager>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // If bundled server is running, return its URL
    if let Some(url) = server_manager.server_url().await {
        return Ok(url);
    }
    // Otherwise return the configured URL from AppState
    Ok(state.base_url())
}

/// Check if we're using the bundled server
#[tauri::command]
pub async fn is_bundled_server(server_manager: State<'_, ServerManager>) -> Result<bool, String> {
    Ok(server_manager.is_running().await)
}

/// Clear all stored clips by stopping server, deleting data, and restarting
#[tauri::command]
pub async fn clear_all_data(
    app: tauri::AppHandle,
    server_manager: State<'_, ServerManager>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Emitter;

    eprintln!("[clipper] Clearing all data...");

    // 1. Stop the server
    server_manager.stop().await?;

    // 2. Clear the data directories
    server_manager.clear_data().await?;

    // 3. Restart the server
    let new_url = server_manager.start(&app).await?;

    // 4. Update the client with the new URL
    state.set_server_url(&new_url);

    // 5. Emit event to refresh the clip list in the main window
    let _ = app.emit("data-cleared", ());

    eprintln!("[clipper] All data cleared and server restarted at {}", new_url);
    Ok(())
}
