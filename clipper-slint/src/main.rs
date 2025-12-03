use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use clipper_client::{Clip, ClipperClient, SearchFilters};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock, Weak as ArcWeak};

mod server;
mod settings;

use server::{ServerManager, get_local_ip_addresses};
use settings::{SettingsManager, Theme};

slint::include_modules!();

/// Manages the client connection state that can be updated at runtime
struct ClientState {
    client: RwLock<ClipperClient>,
}

impl ClientState {
    fn new(url: String, token: Option<String>) -> Self {
        let client = if let Some(t) = token {
            ClipperClient::new_with_token(url, t)
        } else {
            ClipperClient::new(url)
        };
        Self {
            client: RwLock::new(client),
        }
    }

    fn update(&self, url: String, token: Option<String>) {
        let client = if let Some(t) = token {
            ClipperClient::new_with_token(url, t)
        } else {
            ClipperClient::new(url)
        };
        *self.client.write().unwrap() = client;
    }

    fn get(&self) -> ClipperClient {
        self.client.read().unwrap().clone()
    }
}

const PAGE_SIZE: usize = 200;
const FAVORITE_TAG: &str = "$favorite";

fn main() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("Failed to start Tokio runtime")?;

    // Initialize settings
    let settings = Arc::new(SettingsManager::new().context("Failed to initialize settings")?);

    // Determine server URL based on settings
    let (base_url, token, server_manager) = if settings.is_bundled_server() {
        // Start the bundled server
        eprintln!("[clipper-slint] Starting bundled server...");
        let server_manager = Arc::new(
            ServerManager::new(Arc::clone(&settings))
                .map_err(|e| anyhow!("Failed to create server manager: {}", e))?,
        );

        let result = runtime
            .block_on(server_manager.start())
            .map_err(|e| anyhow!("Failed to start bundled server: {}", e))?;

        eprintln!("[clipper-slint] Bundled server started at {}", result.url);
        (result.url, result.token, Some(server_manager))
    } else {
        // Use external server URL from environment or settings
        let url = env::var("CLIPPER_URL").unwrap_or_else(|_| settings.get_external_server_url());
        let token = settings.get_external_server_token();
        eprintln!("[clipper-slint] Using external server at {}", url);
        (url, token, None)
    };

    let client_state = Arc::new(ClientState::new(base_url, token));

    let app = App::new().map_err(|e| anyhow!("Failed to initialize UI: {e}"))?;

    // Initialize theme and settings state
    {
        let current_theme = settings.get_theme();
        let theme_index = match current_theme {
            Theme::Light => 0,
            Theme::Dark => 1,
            Theme::Auto => 2,
        };
        app.set_theme_mode(theme_index);

        // Determine if dark mode based on theme setting
        let is_dark = match current_theme {
            Theme::Light => false,
            Theme::Dark => true,
            Theme::Auto => is_system_dark_mode(),
        };
        app.set_is_dark_mode(is_dark);

        // Set server settings
        app.set_use_bundled_server(settings.is_bundled_server());
        app.set_external_server_url(settings.get_external_server_url().into());
        app.set_listen_on_all_interfaces(settings.get_listen_on_all_interfaces());

        // Set local IP addresses for LAN access display
        let ips = get_local_ip_addresses();
        app.set_local_ip_addresses(ips.join(", ").into());
    }

    let controller = AppController::new(
        Arc::clone(&client_state),
        runtime.handle().clone(),
        app.as_weak(),
    );

    // Settings callbacks
    {
        let settings_clone = Arc::clone(&settings);
        let app_weak = app.as_weak();
        app.on_open_settings(move || {
            // Refresh settings state when opening dialog
            if let Some(app) = app_weak.upgrade() {
                app.set_use_bundled_server(settings_clone.is_bundled_server());
                app.set_external_server_url(settings_clone.get_external_server_url().into());
                app.set_listen_on_all_interfaces(settings_clone.get_listen_on_all_interfaces());
            }
        });
    }

    {
        let settings_clone = Arc::clone(&settings);
        let app_weak = app.as_weak();
        app.on_theme_changed(move |index| {
            let theme = match index {
                0 => Theme::Light,
                1 => Theme::Dark,
                _ => Theme::Auto,
            };

            if let Err(e) = settings_clone.set_theme(theme) {
                eprintln!("[clipper-slint] Failed to save theme: {}", e);
            }

            // Update dark mode based on new theme
            let is_dark = match theme {
                Theme::Light => false,
                Theme::Dark => true,
                Theme::Auto => is_system_dark_mode(),
            };

            if let Some(app) = app_weak.upgrade() {
                app.set_is_dark_mode(is_dark);
            }
        });
    }

    {
        let settings_clone = Arc::clone(&settings);
        app.on_server_mode_changed(move |bundled| {
            if let Err(e) = settings_clone.set_use_bundled_server(bundled) {
                eprintln!("[clipper-slint] Failed to save server mode: {}", e);
            }
        });
    }

    {
        let settings_clone = Arc::clone(&settings);
        app.on_external_url_changed(move |url| {
            if let Err(e) = settings_clone.set_external_server_url(url.to_string()) {
                eprintln!("[clipper-slint] Failed to save external URL: {}", e);
            }
        });
    }

    {
        app.on_save_settings(move || {
            eprintln!("[clipper-slint] Settings saved");
        });
    }

    // Toggle listen on all interfaces (LAN access)
    {
        let settings_clone = Arc::clone(&settings);
        let server_manager_clone = server_manager.clone();
        let client_state_clone = Arc::clone(&client_state);
        let app_weak = app.as_weak();
        let runtime_handle = runtime.handle().clone();
        app.on_listen_on_all_changed(move |value| {
            if let Err(e) = settings_clone.set_listen_on_all_interfaces(value) {
                eprintln!("[clipper-slint] Failed to save listen setting: {}", e);
                return;
            }

            // Restart bundled server if running to apply the change
            if let Some(ref server_manager) = server_manager_clone {
                let sm = Arc::clone(server_manager);
                let client_state = Arc::clone(&client_state_clone);
                let app_weak = app_weak.clone();
                runtime_handle.spawn(async move {
                    match sm.restart().await {
                        Ok(result) => {
                            client_state.update(result.url.clone(), result.token);
                            eprintln!(
                                "[clipper-slint] Server restarted with new network settings at {}",
                                result.url
                            );
                            // Update UI
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(app) = app_weak.upgrade() {
                                    app.set_status_text("Server restarted".into());
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("[clipper-slint] Failed to restart server: {}", e);
                        }
                    }
                });
            }
        });
    }

    // Clear all data
    {
        let server_manager_clone = server_manager.clone();
        let client_state_clone = Arc::clone(&client_state);
        let app_weak = app.as_weak();
        let runtime_handle = runtime.handle().clone();
        let controller_clone = controller.clone();
        app.on_clear_all_data(move || {
            if let Some(ref server_manager) = server_manager_clone {
                let sm = Arc::clone(server_manager);
                let client_state = Arc::clone(&client_state_clone);
                let app_weak = app_weak.clone();
                let controller = controller_clone.clone();
                runtime_handle.spawn(async move {
                    // Stop server first
                    if let Err(e) = sm.stop().await {
                        eprintln!("[clipper-slint] Failed to stop server: {}", e);
                        return;
                    }

                    // Clear data
                    if let Err(e) = sm.clear_data().await {
                        eprintln!("[clipper-slint] Failed to clear data: {}", e);
                        // Try to restart anyway
                    }

                    // Restart server
                    match sm.start().await {
                        Ok(result) => {
                            client_state.update(result.url.clone(), result.token);
                            eprintln!("[clipper-slint] Server restarted after clearing data");
                            // Update UI and reload clips
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(app) = app_weak.upgrade() {
                                    app.set_status_text("All data cleared".into());
                                }
                            });
                            // Reload clips on a separate task
                            controller.load_clips();
                        }
                        Err(e) => {
                            eprintln!("[clipper-slint] Failed to restart server: {}", e);
                        }
                    }
                });
            } else {
                eprintln!("[clipper-slint] Cannot clear data: not using bundled server");
            }
        });
    }

    {
        let controller = controller.clone();
        app.on_search_text_changed(move |text| {
            controller.update_search(text.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_start_date_changed(move |text| {
            controller.update_start_date(text.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_end_date_changed(move |text| {
            controller.update_end_date(text.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_favorites_only_changed(move |value| {
            controller.update_favorites_only(value);
        });
    }

    {
        let controller = controller.clone();
        app.on_toggle_favorite(move |id| {
            controller.toggle_favorite(id.to_string());
        });
    }

    {
        let controller = controller.clone();
        app.on_refresh_request(move || {
            controller.load_clips();
        });
    }

    controller.load_clips();

    app.run().map_err(|e| anyhow!("UI error: {e}"))?;

    // Stop the bundled server if running
    if let Some(server_manager) = server_manager {
        eprintln!("[clipper-slint] Stopping bundled server...");
        runtime.block_on(async {
            if let Err(e) = server_manager.stop().await {
                eprintln!(
                    "[clipper-slint] Warning: Failed to stop server cleanly: {}",
                    e
                );
            }
        });
        eprintln!("[clipper-slint] Server stopped");
    }

    drop(runtime);

    Ok(())
}

#[derive(Clone, Default)]
struct FilterState {
    search_text: String,
    start_date_input: String,
    end_date_input: String,
    favorites_only: bool,
}

impl FilterState {
    fn prepare(&self) -> Result<PreparedFilters, String> {
        let mut filters = SearchFilters::new();

        if let Some(start) = parse_date(&self.start_date_input, "start")? {
            filters.start_date = Some(start);
        }

        if let Some(end) = parse_date(&self.end_date_input, "end")? {
            filters.end_date = Some(end);
        }

        if self.favorites_only {
            filters.tags = Some(vec![FAVORITE_TAG.to_string()]);
        }

        Ok(PreparedFilters {
            query: self.search_text.trim().to_string(),
            search_filters: filters,
        })
    }
}

struct PreparedFilters {
    query: String,
    search_filters: SearchFilters,
}

struct AppController {
    client_state: Arc<ClientState>,
    runtime: tokio::runtime::Handle,
    ui: slint::Weak<App>,
    filters: Mutex<FilterState>,
    cache: Arc<Mutex<HashMap<String, Clip>>>,
}

impl AppController {
    fn new(
        client_state: Arc<ClientState>,
        runtime: tokio::runtime::Handle,
        ui: slint::Weak<App>,
    ) -> Arc<Self> {
        Arc::new(Self {
            client_state,
            runtime,
            ui,
            filters: Mutex::new(FilterState::default()),
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn load_clips(self: &Arc<Self>) {
        let prepared = {
            let filters = self.filters.lock().unwrap().clone();
            filters.prepare()
        };

        let prepared = match prepared {
            Ok(p) => p,
            Err(err) => {
                update_status(&self.ui, SharedString::from(err));
                return;
            }
        };

        update_status(&self.ui, SharedString::from("Loading clips..."));

        let client = self.client_state.get();
        let ui = self.ui.clone();
        let cache = self.cache.clone();
        let query = prepared.query;
        let search_filters = prepared.search_filters;

        self.runtime.spawn(async move {
            let response = if query.is_empty() {
                client.list_clips(search_filters, 1, PAGE_SIZE).await
            } else {
                client
                    .search_clips(&query, search_filters, 1, PAGE_SIZE)
                    .await
            };

            match response {
                Ok(result) => {
                    {
                        let mut cache_guard = cache.lock().unwrap();
                        cache_guard.clear();
                        for clip in &result.items {
                            cache_guard.insert(clip.id.clone(), clip.clone());
                        }
                    }
                    let entries: Vec<ClipEntryData> =
                        result.items.iter().map(clip_to_ui_entry).collect();
                    update_clip_list(&ui, entries);
                    update_status(
                        &ui,
                        SharedString::from(format!("Showing {} clip(s)", result.items.len())),
                    );
                }
                Err(err) => {
                    update_status(
                        &ui,
                        SharedString::from(format!("Failed to load clips: {err}")),
                    );
                }
            }
        });
    }

    fn update_search(self: &Arc<Self>, text: String) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.search_text = text;
        }
        self.load_clips();
    }

    fn update_start_date(self: &Arc<Self>, text: String) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.start_date_input = text;
        }
        self.load_clips();
    }

    fn update_end_date(self: &Arc<Self>, text: String) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.end_date_input = text;
        }
        self.load_clips();
    }

    fn update_favorites_only(self: &Arc<Self>, value: bool) {
        {
            let mut filters = self.filters.lock().unwrap();
            filters.favorites_only = value;
        }
        self.load_clips();
    }

    fn toggle_favorite(self: &Arc<Self>, id: String) {
        let clip = {
            let cache = self.cache.lock().unwrap();
            cache.get(&id).cloned()
        };

        let Some(clip) = clip else {
            update_status(
                &self.ui,
                SharedString::from("Clip not found in current list"),
            );
            return;
        };

        let mut tags = clip.tags.clone();
        let is_favorite = tags.iter().any(|tag| tag == FAVORITE_TAG);
        if is_favorite {
            tags.retain(|tag| tag != FAVORITE_TAG);
        } else {
            tags.push(FAVORITE_TAG.to_string());
        }

        update_status(&self.ui, SharedString::from("Updating favorite..."));

        let client = self.client_state.get();
        let ui = self.ui.clone();
        let cache = self.cache.clone();
        let weak_self: ArcWeak<Self> = Arc::downgrade(self);

        self.runtime.spawn(async move {
            match client.update_clip(&clip.id, Some(tags), None).await {
                Ok(updated) => {
                    {
                        let mut cache_guard = cache.lock().unwrap();
                        cache_guard.insert(updated.id.clone(), updated.clone());
                    }

                    let status = if updated.tags.iter().any(|tag| tag == FAVORITE_TAG) {
                        "Marked as favorite"
                    } else {
                        "Removed favorite"
                    };
                    update_status(&ui, SharedString::from(status));

                    if let Some(controller) = weak_self.upgrade() {
                        controller.load_clips();
                    }
                }
                Err(err) => {
                    update_status(
                        &ui,
                        SharedString::from(format!("Failed to update favorite: {err}")),
                    );
                }
            }
        });
    }
}

fn parse_date(input: &str, label: &str) -> Result<Option<DateTime<Utc>>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|_| format!("Invalid {label} date. Use YYYY-MM-DD"))?;

    let datetime = if label == "end" {
        date.and_hms_opt(23, 59, 59)
    } else {
        date.and_hms_opt(0, 0, 0)
    }
    .ok_or_else(|| format!("Invalid {label} date"))?;

    Ok(Some(Utc.from_utc_datetime(&datetime)))
}

fn clip_to_ui_entry(clip: &Clip) -> ClipEntryData {
    ClipEntryData {
        id: clip.id.clone().into(),
        content: clip.content.clone().into(),
        created_at: format_timestamp(&clip.created_at).into(),
        tags: clip.tags.join(", ").into(),
        favorite: clip.tags.iter().any(|tag| tag == FAVORITE_TAG),
    }
}

fn format_timestamp(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn update_status(ui: &slint::Weak<App>, text: SharedString) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = ui.upgrade() {
            app.set_status_text(text.clone());
        }
    });
}

fn update_clip_list(ui: &slint::Weak<App>, clips: Vec<ClipEntryData>) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = ui.upgrade() {
            let model: ModelRc<ClipEntryData> = Rc::new(VecModel::from(clips)).into();
            app.set_clips(model.clone());
        }
    });
}

/// Detect if system is in dark mode
fn is_system_dark_mode() -> bool {
    // Try to detect system dark mode
    // On macOS, we can check the AppleInterfaceStyle default
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.trim().eq_ignore_ascii_case("dark");
        }
    }

    // Default to light mode if detection fails
    false
}
