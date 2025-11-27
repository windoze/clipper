use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::Manager;
use tokio::fs;

const SETTINGS_FILE_NAME: &str = "settings.json";

/// Theme preference: "light", "dark", or "auto" (follows system)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
    #[default]
    Auto,
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
    /// Server port for the bundled server (persisted across restarts)
    #[serde(default)]
    pub server_port: Option<u16>,
    /// Whether to use the bundled server (true) or external server (false)
    #[serde(default = "default_use_bundled_server")]
    pub use_bundled_server: bool,
}

fn default_use_bundled_server() -> bool {
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
            server_port: None,
            use_bundled_server: true,
        }
    }
}

#[derive(Clone)]
pub struct SettingsManager {
    settings: Arc<RwLock<Settings>>,
    config_path: PathBuf,
}

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
}

/// Get the platform-specific config directory for the app
pub fn get_app_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))
}
