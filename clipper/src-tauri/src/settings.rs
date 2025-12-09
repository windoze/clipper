use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::Manager;
use tokio::fs;

pub const SETTINGS_FILE_NAME: &str = "settings.json";

/// Theme preference: "light", "dark", or "auto" (follows system)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
    #[default]
    Auto,
}

/// Syntax highlighting theme preference
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SyntaxTheme {
    #[default]
    Github,
    Monokai,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    OneDark,
    VsCode,
    Gruvbox,
}

/// Settings dialog window geometry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettingsWindowGeometry {
    /// Window width
    pub width: Option<u32>,
    /// Window height
    pub height: Option<u32>,
    /// Window X position (logical)
    pub x: Option<i32>,
    /// Window Y position (logical)
    pub y: Option<i32>,
}

/// Main window geometry (size and position)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MainWindowGeometry {
    /// Window width
    pub width: Option<u32>,
    /// Window height
    pub height: Option<u32>,
    /// Window X position (logical)
    pub x: Option<i32>,
    /// Window Y position (logical)
    pub y: Option<i32>,
    /// Whether the window is maximized
    pub maximized: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Server address for syncing clips
    pub server_address: String,
    /// Default save location for clipped content
    pub default_save_location: Option<String>,
    /// Whether to show the main window on startup
    pub open_on_startup: bool,
    /// Whether to start the application on login
    pub start_on_login: bool,
    /// Theme preference: light, dark, or auto
    #[serde(default)]
    pub theme: ThemePreference,
    /// Syntax highlighting theme for code snippets
    #[serde(default)]
    pub syntax_theme: SyntaxTheme,
    /// Server port for the bundled server (persisted across restarts)
    #[serde(default)]
    pub server_port: Option<u16>,
    /// Whether to use the bundled server (true) or external server (false)
    #[serde(default = "default_use_bundled_server")]
    pub use_bundled_server: bool,
    /// Whether to listen on all network interfaces (bundled server only)
    #[serde(default)]
    pub listen_on_all_interfaces: bool,
    /// Language preference (e.g., "en", "zh")
    #[serde(default)]
    pub language: Option<String>,
    /// Whether to show toast notifications
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,
    /// Global shortcut to toggle window visibility (e.g., "CmdOrCtrl+Shift+V")
    #[serde(default = "default_global_shortcut")]
    pub global_shortcut: String,
    /// Whether to enable automatic cleanup of old clips (bundled server only)
    #[serde(default)]
    pub cleanup_enabled: bool,
    /// Retention period in days for automatic cleanup (bundled server only)
    #[serde(default = "default_cleanup_retention_days")]
    pub cleanup_retention_days: u32,
    /// Bearer token for external server authentication
    #[serde(default)]
    pub external_server_token: Option<String>,
    /// Bearer token for bundled server when external access is enabled
    #[serde(default)]
    pub bundled_server_token: Option<String>,
    /// Maximum upload size in MB for bundled server (default: 10)
    #[serde(default = "default_max_upload_size_mb")]
    pub max_upload_size_mb: u64,
    /// Settings dialog window geometry (size and position)
    #[serde(default)]
    pub settings_window_geometry: SettingsWindowGeometry,
    /// Main window geometry (size and position)
    #[serde(default)]
    pub main_window_geometry: MainWindowGeometry,
    /// Trusted certificate fingerprints for self-signed HTTPS servers
    /// Maps server hostname to SHA-256 fingerprint (hex encoded)
    #[serde(default)]
    pub trusted_certificates: std::collections::HashMap<String, String>,
    /// Enable debug logging to log file (manually configurable only)
    /// When false (default), only INFO and above are written to the log file
    /// When true, DEBUG logs are also written to the log file
    #[serde(default)]
    pub debug_logging: bool,
}

fn default_cleanup_retention_days() -> u32 {
    30
}

fn default_max_upload_size_mb() -> u64 {
    10
}

fn default_global_shortcut() -> String {
    #[cfg(target_os = "macos")]
    {
        "Command+Shift+V".to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl+Shift+V".to_string()
    }
}

fn default_use_bundled_server() -> bool {
    true
}

fn default_notifications_enabled() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_address: "http://localhost:3000".to_string(),
            default_save_location: None,
            open_on_startup: true,
            start_on_login: false,
            theme: ThemePreference::Auto,
            syntax_theme: SyntaxTheme::Github,
            server_port: None,
            use_bundled_server: true,
            listen_on_all_interfaces: false,
            language: None,
            notifications_enabled: true,
            global_shortcut: default_global_shortcut(),
            cleanup_enabled: false,
            cleanup_retention_days: default_cleanup_retention_days(),
            external_server_token: None,
            bundled_server_token: None,
            max_upload_size_mb: default_max_upload_size_mb(),
            settings_window_geometry: SettingsWindowGeometry::default(),
            main_window_geometry: MainWindowGeometry::default(),
            trusted_certificates: std::collections::HashMap::new(),
            debug_logging: false,
        }
    }
}

#[derive(Clone)]
pub struct SettingsManager {
    settings: Arc<RwLock<Settings>>,
    config_path: PathBuf,
}

#[allow(dead_code)]
impl SettingsManager {
    /// Create a new settings manager with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join(SETTINGS_FILE_NAME);
        Self {
            settings: Arc::new(RwLock::new(Settings::default())),
            config_path,
        }
    }

    /// Initialize the settings manager by loading settings from disk
    pub async fn init(&self) -> Result<(), String> {
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;

            // Secure the config directory and fix any incorrect permissions
            match clipper_security::secure_directory_recursive(parent, |msg| log::warn!("{}", msg))
            {
                Ok(count) if count > 0 => {
                    log::info!("Fixed permissions on {} items in config directory", count);
                }
                Err(e) => log::warn!("Failed to secure config directory: {}", e),
                _ => {}
            }
        }

        // Load settings if file exists
        if self.config_path.exists() {
            let contents = fs::read_to_string(&self.config_path)
                .await
                .map_err(|e| format!("Failed to read settings file: {}", e))?;

            let settings: Settings = serde_json::from_str(&contents)
                .map_err(|e| format!("Failed to parse settings: {}", e))?;

            *self.settings.write().unwrap() = settings;
        } else {
            // Save default settings
            self.save().await?;
        }

        Ok(())
    }

    /// Get a clone of the current settings
    pub fn get(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    /// Update settings and save to disk
    pub async fn update(&self, settings: Settings) -> Result<(), String> {
        *self.settings.write().unwrap() = settings;
        self.save().await
    }

    /// Save current settings to disk
    async fn save(&self) -> Result<(), String> {
        let settings = self.get();
        let contents = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.config_path, contents)
            .await
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    /// Get the saved server port
    pub fn get_server_port(&self) -> Option<u16> {
        self.settings.read().unwrap().server_port
    }

    /// Set and save the server port
    pub async fn set_server_port(&self, port: u16) -> Result<(), String> {
        {
            self.settings.write().unwrap().server_port = Some(port);
        }
        self.save().await
    }

    /// Get whether the server should listen on all interfaces
    pub fn get_listen_on_all_interfaces(&self) -> bool {
        self.settings.read().unwrap().listen_on_all_interfaces
    }

    /// Get whether cleanup is enabled
    pub fn get_cleanup_enabled(&self) -> bool {
        self.settings.read().unwrap().cleanup_enabled
    }

    /// Get the cleanup retention days
    pub fn get_cleanup_retention_days(&self) -> u32 {
        self.settings.read().unwrap().cleanup_retention_days
    }

    /// Get the bundled server token (for external access auth)
    pub fn get_bundled_server_token(&self) -> Option<String> {
        self.settings.read().unwrap().bundled_server_token.clone()
    }

    /// Set and save the bundled server token
    pub async fn set_bundled_server_token(&self, token: String) -> Result<(), String> {
        {
            self.settings.write().unwrap().bundled_server_token = Some(token);
        }
        self.save().await
    }

    /// Get the external server token
    pub fn get_external_server_token(&self) -> Option<String> {
        self.settings.read().unwrap().external_server_token.clone()
    }

    /// Get the maximum upload size in MB
    pub fn get_max_upload_size_mb(&self) -> u64 {
        self.settings.read().unwrap().max_upload_size_mb
    }

    /// Get whether debug logging to file is enabled
    pub fn get_debug_logging(&self) -> bool {
        self.settings.read().unwrap().debug_logging
    }

    /// Get all trusted certificate fingerprints
    pub fn get_trusted_certificates(&self) -> std::collections::HashMap<String, String> {
        self.settings.read().unwrap().trusted_certificates.clone()
    }

    /// Check if a certificate fingerprint is trusted for a given host
    pub fn is_certificate_trusted(&self, host: &str, fingerprint: &str) -> bool {
        self.settings
            .read()
            .unwrap()
            .trusted_certificates
            .get(host)
            .map(|fp| fp == fingerprint)
            .unwrap_or(false)
    }

    /// Get the stored fingerprint for a host, if any
    pub fn get_stored_fingerprint(&self, host: &str) -> Option<String> {
        self.settings
            .read()
            .unwrap()
            .trusted_certificates
            .get(host)
            .cloned()
    }

    /// Add a trusted certificate fingerprint for a host
    pub async fn trust_certificate(&self, host: String, fingerprint: String) -> Result<(), String> {
        {
            self.settings
                .write()
                .unwrap()
                .trusted_certificates
                .insert(host, fingerprint);
        }
        self.save().await
    }

    /// Remove a trusted certificate for a host
    pub async fn untrust_certificate(&self, host: &str) -> Result<(), String> {
        {
            self.settings
                .write()
                .unwrap()
                .trusted_certificates
                .remove(host);
        }
        self.save().await
    }

    /// Get the main window geometry
    pub fn get_main_window_geometry(&self) -> MainWindowGeometry {
        self.settings.read().unwrap().main_window_geometry.clone()
    }

    /// Save the main window geometry
    pub async fn save_main_window_geometry(
        &self,
        geometry: MainWindowGeometry,
    ) -> Result<(), String> {
        {
            self.settings.write().unwrap().main_window_geometry = geometry;
        }
        self.save().await
    }
}

/// Get the platform-specific config directory for the app
pub fn get_app_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))
}
