use crate::autolaunch;
use crate::server::ServerManager;
use crate::settings::{Settings, SettingsManager};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use clipper_client::models::PagedResult;
use clipper_client::{fetch_server_certificate, Clip, SearchFilters, ServerInfo};
use gethostname::gethostname;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    // Add a grace period to ensure the server is fully ready to accept connections
    // The server's health check may pass before it's ready for WebSocket connections
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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
/// Returns Ok(None) if the server is reachable, or Ok(Some(error_message)) if unreachable
/// The switch always happens regardless of connectivity
#[tauri::command]
pub async fn switch_to_external_server(
    app: tauri::AppHandle,
    server_manager: State<'_, ServerManager>,
    settings_manager: State<'_, SettingsManager>,
    state: State<'_, AppState>,
    server_url: String,
) -> Result<Option<String>, String> {
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
    state.set_server_url_with_token(&server_url, token.clone());

    // Update settings to remember this choice and the external URL
    let mut settings = settings_manager.get();
    settings.use_bundled_server = false;
    settings.server_address = server_url.clone();
    settings_manager.update(settings).await?;

    // Emit event to refresh the clip list
    let _ = app.emit("server-switched", ());

    eprintln!("[clipper] Switched to external server at {}", server_url);

    // Check if the external server is reachable (but don't block the switch)
    // Use current trusted fingerprints for certificate verification
    let trusted_fingerprints = state.get_trusted_fingerprints();
    let connection_error = check_server_reachable(&server_url, token.as_deref(), trusted_fingerprints).await;

    if let Some(ref err) = connection_error {
        eprintln!(
            "[clipper] Warning: External server is not reachable: {}",
            err
        );
    }

    Ok(connection_error)
}

/// Check if a server is reachable by calling its health endpoint
/// Returns None if reachable, Some(error_message) if not
async fn check_server_reachable(
    server_url: &str,
    token: Option<&str>,
    trusted_fingerprints: HashMap<String, String>,
) -> Option<String> {
    // Use trusted certificates for HTTPS connections
    let client = match clipper_client::create_http_client_with_trusted_certs(trusted_fingerprints) {
        Ok(c) => c,
        Err(e) => return Some(format!("Failed to create HTTP client: {}", e)),
    };

    let health_url = format!("{}/health", server_url.trim_end_matches('/'));

    let mut request = client.get(&health_url);
    if let Some(t) = token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    match request.send().await {
        Ok(response) if response.status().is_success() => None,
        Ok(response) => Some(format!(
            "Server returned error status: {}",
            response.status()
        )),
        Err(e) => Some(format!("Cannot connect to server: {}", e)),
    }
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

// ============ App Info Commands ============

/// Get the current app version
#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

// ============ Updater Commands ============

/// Information about an available update
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub body: Option<String>,
    pub date: Option<String>,
}

/// Check for available updates
/// Returns Some(UpdateInfo) if an update is available, None if already up to date
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            let current_version = app.package_info().version.to_string();
            Ok(Some(UpdateInfo {
                version: update.version.clone(),
                current_version,
                body: update.body.clone(),
                date: update.date.map(|d| d.to_string()),
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Download and install the available update
/// This will download the update and prompt the user to restart
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            // Emit progress events during download
            let app_handle = app.clone();
            update
                .download_and_install(
                    |chunk_length, content_length| {
                        let progress = content_length.map(|total| {
                            (chunk_length as f64 / total as f64 * 100.0) as u32
                        });
                        let _ = app_handle.emit("update-download-progress", progress);
                    },
                    || {
                        let _ = app_handle.emit("update-download-finished", ());
                    },
                )
                .await
                .map_err(|e| e.to_string())?;

            // Emit event that update is ready and app needs restart
            let _ = app.emit("update-ready", ());

            Ok(())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

// ============ Certificate Commands ============

/// Certificate information returned to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateInfoResponse {
    pub host: String,
    pub fingerprint: String,
    pub is_trusted: bool,
}

/// Result of checking server certificate
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCertificateCheckResult {
    /// Whether the server uses HTTPS
    pub is_https: bool,
    /// Certificate info if HTTPS and certificate was retrieved
    pub certificate: Option<CertificateInfoResponse>,
    /// Whether the certificate is already trusted
    pub is_trusted: bool,
    /// Whether the certificate is self-signed or has verification issues
    pub needs_trust_confirmation: bool,
    /// Error message if certificate couldn't be retrieved
    pub error: Option<String>,
}

/// Fetch and check the certificate for a server URL
/// Returns certificate info and whether it needs user confirmation
#[tauri::command]
pub async fn check_server_certificate(
    settings_manager: State<'_, SettingsManager>,
    server_url: String,
) -> Result<ServerCertificateCheckResult, String> {
    use tauri::Url;

    // Parse the URL to get host and port
    let url = Url::parse(&server_url)
        .map_err(|e| format!("Invalid URL: {}", e))?;

    // Check if it's HTTPS
    let is_https = url.scheme() == "https";
    if !is_https {
        return Ok(ServerCertificateCheckResult {
            is_https: false,
            certificate: None,
            is_trusted: true, // HTTP doesn't need certificate trust
            needs_trust_confirmation: false,
            error: None,
        });
    }

    let host = url.host_str()
        .ok_or_else(|| "URL has no host".to_string())?
        .to_string();
    let port = url.port().unwrap_or(443);

    // Try to fetch the certificate
    match fetch_server_certificate(&host, port).await {
        Ok(cert_info) => {
            let fingerprint = cert_info.fingerprint.clone();
            let is_system_trusted = cert_info.is_system_trusted;

            // Check if this certificate is already trusted by us (in settings)
            let is_user_trusted = settings_manager.is_certificate_trusted(&host, &fingerprint);

            // Certificate is trusted if it passes system verification OR if user has trusted it
            let is_trusted = is_system_trusted || is_user_trusted;

            // Only need confirmation if:
            // 1. Certificate is NOT system trusted (self-signed or invalid chain)
            // 2. AND user has NOT already trusted it
            let needs_trust_confirmation = !is_system_trusted && !is_user_trusted;

            Ok(ServerCertificateCheckResult {
                is_https: true,
                certificate: Some(CertificateInfoResponse {
                    host: host.clone(),
                    fingerprint,
                    is_trusted,
                }),
                is_trusted,
                needs_trust_confirmation,
                error: None,
            })
        }
        Err(e) => {
            // Certificate fetch failed - this could be various TLS errors
            Ok(ServerCertificateCheckResult {
                is_https: true,
                certificate: None,
                is_trusted: false,
                needs_trust_confirmation: false,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Trust a certificate fingerprint for a specific host
#[tauri::command]
pub async fn trust_certificate(
    settings_manager: State<'_, SettingsManager>,
    state: State<'_, AppState>,
    host: String,
    fingerprint: String,
) -> Result<(), String> {
    // Save to settings
    settings_manager.trust_certificate(host.clone(), fingerprint.clone()).await?;

    // Update AppState with new trusted fingerprints
    let trusted = settings_manager.get_trusted_certificates();
    state.set_trusted_fingerprints(trusted);

    // Signal WebSocket to reconnect with the new trusted certificate
    state.signal_ws_reconnect();

    eprintln!("[clipper] Trusted certificate for {}: {}", host, fingerprint);
    Ok(())
}

/// Remove trust for a certificate
#[tauri::command]
pub async fn untrust_certificate(
    settings_manager: State<'_, SettingsManager>,
    state: State<'_, AppState>,
    host: String,
) -> Result<(), String> {
    // Remove from settings
    settings_manager.untrust_certificate(&host).await?;

    // Update AppState
    let trusted = settings_manager.get_trusted_certificates();
    state.set_trusted_fingerprints(trusted);

    eprintln!("[clipper] Removed certificate trust for {}", host);
    Ok(())
}

/// Get all trusted certificates
#[tauri::command]
pub fn get_trusted_certificates(
    settings_manager: State<'_, SettingsManager>,
) -> HashMap<String, String> {
    settings_manager.get_trusted_certificates()
}
