use crate::autolaunch;
use crate::server::ServerManager;
use crate::settings::{Settings, SettingsManager};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use clipper_client::models::PagedResult;
use clipper_client::{Clip, SearchFilters, ServerInfo};
use gethostname::gethostname;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tokio::fs;

/// Get the hostname tag in the format `$host:<hostname>`
fn get_hostname_tag() -> String {
    let hostname = gethostname().to_string_lossy().to_string();
    format!("$host:{}", hostname)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFiltersInput {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl SearchFiltersInput {
    pub fn into_search_filters(self) -> SearchFilters {
        let mut filters = SearchFilters::new();

        if let Some(start) = self.start_date
            && let Ok(dt) = start.parse::<DateTime<Utc>>()
        {
            filters.start_date = Some(dt);
        }

        if let Some(end) = self.end_date
            && let Ok(dt) = end.parse::<DateTime<Utc>>()
        {
            filters.end_date = Some(dt);
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
    let mut tags_with_host = tags;
    tags_with_host.push(get_hostname_tag());
    client
        .create_clip(content, tags_with_host, additional_notes)
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

    // Check file size limit
    let max_size = state.get_max_upload_size_bytes();
    if bytes.len() as u64 > max_size {
        let max_size_mb = max_size as f64 / (1024.0 * 1024.0);
        let file_size_mb = bytes.len() as f64 / (1024.0 * 1024.0);
        return Err(format!(
            "File size ({:.2} MB) exceeds maximum allowed size ({:.2} MB)",
            file_size_mb, max_size_mb
        ));
    }

    // Get filename from path
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let client = state.client();
    let mut tags_with_host = tags;
    tags_with_host.push(get_hostname_tag());
    client
        .upload_file_bytes(bytes, filename, tags_with_host, additional_notes)
        .await
        .map_err(|e| e.to_string())
}

/// Get the URL for a clip's file attachment
/// If authentication is configured, the token is included as a query parameter
#[tauri::command]
pub fn get_file_url(state: State<'_, AppState>, clip_id: String) -> String {
    let base_url = format!("{}/clips/{}/file", state.base_url(), clip_id);
    match state.token() {
        Some(token) => format!("{}?token={}", base_url, token),
        None => base_url,
    }
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
/// Note: This only saves settings to disk. Server restart (when token/cleanup changes)
/// is handled by the frontend when the settings dialog is closed, via switch_to_bundled_server.
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

    // Save settings to disk
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

    eprintln!(
        "[clipper] All data cleared and server restarted at {}",
        new_url
    );
    Ok(())
}

/// Switch to using the bundled server
/// This will restart the server if it's already running to pick up any configuration changes
/// (token, cleanup settings, etc.)
#[tauri::command]
pub async fn switch_to_bundled_server(
    app: tauri::AppHandle,
    server_manager: State<'_, ServerManager>,
    settings_manager: State<'_, SettingsManager>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tauri::Emitter;

    eprintln!("[clipper] Switching to bundled server...");

    // Always restart the server to pick up any configuration changes (token, cleanup, etc.)
    // If already running, restart; otherwise just start
    let server_url = if server_manager.is_running().await {
        server_manager.restart(&app).await?
    } else {
        server_manager.start(&app).await?
    };

    // Get token for bundled server - always use if set (server requires it when configured)
    let token = settings_manager.get_bundled_server_token();

    // Update the client to use the bundled server URL with token
    state.set_server_url_with_token(&server_url, token);

    // Update settings to remember this choice
    let mut settings = settings_manager.get();
    settings.use_bundled_server = true;
    settings_manager.update(settings).await?;

    // Emit event to refresh the clip list
    let _ = app.emit("server-switched", ());

    eprintln!("[clipper] Switched to bundled server at {}", server_url);
    Ok(server_url)
}

/// Switch to using an external server
#[tauri::command]
pub async fn switch_to_external_server(
    app: tauri::AppHandle,
    server_manager: State<'_, ServerManager>,
    settings_manager: State<'_, SettingsManager>,
    state: State<'_, AppState>,
    server_url: String,
) -> Result<(), String> {
    use tauri::Emitter;

    eprintln!(
        "[clipper] Switching to external server at {}...",
        server_url
    );

    // Stop the bundled server if running
    if server_manager.is_running().await {
        server_manager.stop().await?;
    }

    // Get token from settings
    let token = settings_manager.get_external_server_token();

    // Update the client to use the external server URL with token
    state.set_server_url_with_token(&server_url, token);

    // Update settings to remember this choice and the external URL
    let mut settings = settings_manager.get();
    settings.use_bundled_server = false;
    settings.server_address = server_url.clone();
    settings_manager.update(settings).await?;

    // Emit event to refresh the clip list
    let _ = app.emit("server-switched", ());

    eprintln!("[clipper] Switched to external server at {}", server_url);
    Ok(())
}

/// Get all local IP addresses for the machine
#[tauri::command]
pub fn get_local_ip_addresses() -> Result<Vec<String>, String> {
    use local_ip_address::list_afinet_netifas;

    let network_interfaces =
        list_afinet_netifas().map_err(|e| format!("Failed to get network interfaces: {}", e))?;

    let ips: Vec<String> = network_interfaces
        .into_iter()
        .filter_map(|(_name, ip)| {
            // Filter out loopback addresses
            if ip.is_loopback() {
                return None;
            }
            // Only include IPv4 addresses for simplicity
            if let std::net::IpAddr::V4(ipv4) = ip {
                // Filter out link-local addresses (169.254.x.x)
                if !ipv4.is_link_local() {
                    return Some(ipv4.to_string());
                }
            }
            None
        })
        .collect();

    Ok(ips)
}

/// Toggle the listen on all interfaces setting and restart the server
#[tauri::command]
pub async fn toggle_listen_on_all_interfaces(
    app: tauri::AppHandle,
    server_manager: State<'_, ServerManager>,
    settings_manager: State<'_, SettingsManager>,
    state: State<'_, AppState>,
    listen_on_all: bool,
) -> Result<String, String> {
    use tauri::Emitter;

    eprintln!(
        "[clipper] Toggling listen_on_all_interfaces to {}...",
        listen_on_all
    );

    // Update the setting
    let mut settings = settings_manager.get();
    settings.listen_on_all_interfaces = listen_on_all;
    settings_manager.update(settings).await?;

    // Restart the server if it's running
    if server_manager.is_running().await {
        server_manager.stop().await?;
        let new_url = server_manager.start(&app).await?;

        // Get token - always use if set (server requires it when configured)
        let token = settings_manager.get_bundled_server_token();

        state.set_server_url_with_token(&new_url, token);

        // Emit event to refresh the clip list
        let _ = app.emit("server-switched", ());

        eprintln!(
            "[clipper] Server restarted with listen_on_all_interfaces={}",
            listen_on_all
        );
        return Ok(new_url);
    }

    Ok(state.base_url())
}

/// Update the tray menu language
#[tauri::command]
pub fn update_tray_language(app: tauri::AppHandle, language: String) -> Result<(), String> {
    crate::tray::update_tray_language(&app, &language).map_err(|e| e.to_string())
}

/// Get the current WebSocket connection status
#[tauri::command]
pub fn get_websocket_status(state: State<'_, AppState>) -> bool {
    state.is_websocket_connected()
}

/// Update the global shortcut
#[tauri::command]
pub fn update_global_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Parse the new shortcut
    let new_shortcut = crate::parse_shortcut(&shortcut)
        .ok_or_else(|| format!("Invalid shortcut format: {}", shortcut))?;

    // Unregister all existing shortcuts
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;

    // Register the new shortcut
    app.global_shortcut()
        .register(new_shortcut)
        .map_err(|e| format!("Failed to register shortcut '{}': {}", shortcut, e))?;

    eprintln!("[clipper] Global shortcut updated to: {}", shortcut);
    Ok(())
}

/// Get server info (including max upload size) from the connected server
#[tauri::command]
pub async fn get_server_info(state: State<'_, AppState>) -> Result<ServerInfo, String> {
    let client = state.client();
    let info = client
        .get_server_info()
        .await
        .map_err(|e| e.to_string())?;

    // Update the max upload size in app state
    state.set_max_upload_size_bytes(info.config.max_upload_size_bytes);

    Ok(info)
}

/// Get the current effective max upload size in bytes
#[tauri::command]
pub fn get_max_upload_size_bytes(state: State<'_, AppState>) -> u64 {
    state.get_max_upload_size_bytes()
}
