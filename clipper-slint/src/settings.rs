use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

// Use the same app identifier as the Tauri app to share settings
const APP_IDENTIFIER: &str = "codes.unwritten.clipper";
const SETTINGS_FILE: &str = "settings.json";

/// Theme preference: "light", "dark", or "auto" (follows system)
/// Uses camelCase to match the Tauri app's settings format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    Auto,
}

/// Settings structure matching the Tauri app's settings.json format.
/// All fields use camelCase to match the Tauri app's format.
/// Fields not supported by clipper-slint are still deserialized to maintain
/// compatibility when reading/writing the shared settings file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Server address for syncing clips (maps to external_server_url in the old format)
    #[serde(default = "default_server_address")]
    pub server_address: String,

    /// Default save location for clipped content (not used by clipper-slint)
    #[serde(default)]
    pub default_save_location: Option<String>,

    /// Whether to show the main window on startup (not used by clipper-slint)
    #[serde(default = "default_open_on_startup")]
    pub open_on_startup: bool,

    /// Whether to start the application on login (not used by clipper-slint)
    #[serde(default)]
    pub start_on_login: bool,

    /// Theme preference: light, dark, or auto
    #[serde(default)]
    pub theme: Theme,

    /// Syntax highlighting theme for code snippets (not used by clipper-slint)
    #[serde(default)]
    pub syntax_theme: String,

    /// Server port for the bundled server (persisted across restarts)
    #[serde(default)]
    pub server_port: Option<u16>,

    /// Whether to use the bundled server (true) or external server (false)
    #[serde(default = "default_use_bundled_server")]
    pub use_bundled_server: bool,

    /// Whether to listen on all network interfaces (bundled server only)
    #[serde(default)]
    pub listen_on_all_interfaces: bool,

    /// Language preference (not used by clipper-slint)
    #[serde(default)]
    pub language: Option<String>,

    /// Whether to show toast notifications (not used by clipper-slint)
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,

    /// Global shortcut to toggle window visibility (not used by clipper-slint)
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

    /// Maximum upload size in MB for bundled server
    #[serde(default = "default_max_upload_size_mb")]
    pub max_upload_size_mb: u64,

    /// Settings dialog window geometry (not used by clipper-slint)
    #[serde(default)]
    pub settings_window_geometry: serde_json::Value,

    /// Main window geometry (not used by clipper-slint)
    #[serde(default)]
    pub main_window_geometry: serde_json::Value,

    /// Trusted certificate fingerprints for self-signed HTTPS servers
    #[serde(default)]
    pub trusted_certificates: HashMap<String, String>,

    /// Enable debug logging to log file (not used by clipper-slint)
    #[serde(default)]
    pub debug_logging: bool,
}

fn default_server_address() -> String {
    "http://localhost:3000".to_string()
}

fn default_open_on_startup() -> bool {
    true
}

fn default_use_bundled_server() -> bool {
    true
}

fn default_notifications_enabled() -> bool {
    true
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

fn default_cleanup_retention_days() -> u32 {
    30
}

fn default_max_upload_size_mb() -> u64 {
    10
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_address: default_server_address(),
            default_save_location: None,
            open_on_startup: default_open_on_startup(),
            start_on_login: false,
            theme: Theme::default(),
            syntax_theme: String::new(),
            server_port: None,
            use_bundled_server: default_use_bundled_server(),
            listen_on_all_interfaces: false,
            language: None,
            notifications_enabled: default_notifications_enabled(),
            global_shortcut: default_global_shortcut(),
            cleanup_enabled: false,
            cleanup_retention_days: default_cleanup_retention_days(),
            external_server_token: None,
            bundled_server_token: None,
            max_upload_size_mb: default_max_upload_size_mb(),
            settings_window_geometry: serde_json::Value::Object(Default::default()),
            main_window_geometry: serde_json::Value::Object(Default::default()),
            trusted_certificates: HashMap::new(),
            debug_logging: false,
        }
    }
}

pub struct SettingsManager {
    settings_path: PathBuf,
    settings: RwLock<Settings>,
}

impl SettingsManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join(APP_IDENTIFIER);

        std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        // Secure the config directory and fix any incorrect permissions
        match clipper_security::secure_directory_recursive(&config_dir, |msg| {
            eprintln!("[clipper-slint] {}", msg)
        }) {
            Ok(count) if count > 0 => {
                eprintln!(
                    "[clipper-slint] Fixed permissions on {} items in config directory",
                    count
                );
            }
            Err(e) => eprintln!("[clipper-slint] Failed to secure config directory: {}", e),
            _ => {}
        }

        let settings_path = config_dir.join(SETTINGS_FILE);
        let settings = Self::load_settings(&settings_path);

        Ok(Self {
            settings_path,
            settings: RwLock::new(settings),
        })
    }

    fn load_settings(path: &PathBuf) -> Settings {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => {
                        eprintln!("[settings] Failed to parse settings: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("[settings] Failed to read settings file: {}", e);
                }
            }
        }
        Settings::default()
    }

    #[allow(dead_code)]
    pub fn get(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn save(&self) -> Result<()> {
        let settings = self.settings.read().unwrap().clone();
        let content =
            serde_json::to_string_pretty(&settings).context("Failed to serialize settings")?;
        std::fs::write(&self.settings_path, content).context("Failed to write settings file")?;
        Ok(())
    }

    pub fn get_server_port(&self) -> Option<u16> {
        self.settings.read().unwrap().server_port
    }

    pub fn set_server_port(&self, port: u16) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.server_port = Some(port);
        }
        self.save()
    }

    pub fn get_listen_on_all_interfaces(&self) -> bool {
        self.settings.read().unwrap().listen_on_all_interfaces
    }

    #[allow(dead_code)]
    pub fn set_listen_on_all_interfaces(&self, value: bool) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.listen_on_all_interfaces = value;
        }
        self.save()
    }

    pub fn is_bundled_server(&self) -> bool {
        self.settings.read().unwrap().use_bundled_server
    }

    pub fn get_external_server_url(&self) -> String {
        self.settings.read().unwrap().server_address.clone()
    }

    pub fn set_external_server_url(&self, url: String) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.server_address = url;
        }
        self.save()
    }

    pub fn set_use_bundled_server(&self, value: bool) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.use_bundled_server = value;
        }
        self.save()
    }

    pub fn get_theme(&self) -> Theme {
        self.settings.read().unwrap().theme
    }

    pub fn set_theme(&self, theme: Theme) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.theme = theme;
        }
        self.save()
    }

    pub fn get_cleanup_enabled(&self) -> bool {
        self.settings.read().unwrap().cleanup_enabled
    }

    #[allow(dead_code)]
    pub fn set_cleanup_enabled(&self, value: bool) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.cleanup_enabled = value;
        }
        self.save()
    }

    pub fn get_cleanup_retention_days(&self) -> u32 {
        self.settings.read().unwrap().cleanup_retention_days
    }

    #[allow(dead_code)]
    pub fn set_cleanup_retention_days(&self, days: u32) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.cleanup_retention_days = days;
        }
        self.save()
    }

    pub fn get_bundled_server_token(&self) -> Option<String> {
        self.settings.read().unwrap().bundled_server_token.clone()
    }

    #[allow(dead_code)]
    pub fn set_bundled_server_token(&self, token: Option<String>) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.bundled_server_token = token;
        }
        self.save()
    }

    pub fn get_external_server_token(&self) -> Option<String> {
        self.settings.read().unwrap().external_server_token.clone()
    }

    #[allow(dead_code)]
    pub fn set_external_server_token(&self, token: Option<String>) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.external_server_token = token;
        }
        self.save()
    }

    pub fn get_max_upload_size_mb(&self) -> u64 {
        self.settings.read().unwrap().max_upload_size_mb
    }

    #[allow(dead_code)]
    pub fn set_max_upload_size_mb(&self, mb: u64) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.max_upload_size_mb = mb;
        }
        self.save()
    }
}
