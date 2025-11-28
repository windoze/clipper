use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

const APP_NAME: &str = "clipper-slint";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Whether to use the bundled server (true) or external server (false)
    #[serde(default = "default_use_bundled_server")]
    pub use_bundled_server: bool,

    /// External server URL (used when use_bundled_server is false)
    #[serde(default = "default_external_server_url")]
    pub external_server_url: String,

    /// Port for the bundled server (persisted across restarts)
    #[serde(default)]
    pub server_port: Option<u16>,

    /// Whether to listen on all interfaces (0.0.0.0) for LAN access
    #[serde(default)]
    pub listen_on_all_interfaces: bool,

    /// UI theme (light, dark, auto)
    #[serde(default)]
    pub theme: Theme,
}

fn default_use_bundled_server() -> bool {
    true
}

fn default_external_server_url() -> String {
    "http://localhost:3000".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            use_bundled_server: default_use_bundled_server(),
            external_server_url: default_external_server_url(),
            server_port: None,
            listen_on_all_interfaces: false,
            theme: Theme::default(),
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
            .join(APP_NAME);

        std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

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
        self.settings.read().unwrap().external_server_url.clone()
    }

    pub fn set_external_server_url(&self, url: String) -> Result<()> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.external_server_url = url;
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
}
