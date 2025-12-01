//! Configuration loading for clipper-cli
//!
//! This module attempts to load configuration from the Clipper desktop app's
//! settings file. If the settings file is not available, it falls back to
//! default values which can be overridden by environment variables or CLI args.

use serde::Deserialize;
use std::path::{Path, PathBuf};

const APP_IDENTIFIER: &str = "codes.unwritten.clipper";
const SETTINGS_FILE_NAME: &str = "settings.json";

/// Minimal settings structure that mirrors the desktop app's settings.json
/// We only deserialize the fields we need for the CLI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopSettings {
    /// Server address for syncing clips (used when use_bundled_server is false)
    server_address: String,
    /// Whether to use the bundled server (true) or external server (false)
    #[serde(default)]
    use_bundled_server: bool,
    /// Server port for the bundled server
    #[serde(default)]
    server_port: Option<u16>,
    /// Bearer token for external server authentication
    #[serde(default)]
    external_server_token: Option<String>,
    /// Bearer token for bundled server when external access is enabled
    #[serde(default)]
    bundled_server_token: Option<String>,
}

/// Configuration resolved from the desktop app's settings
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub server_url: String,
    pub token: Option<String>,
}

/// Get the platform-specific config directory for the Clipper desktop app
fn get_app_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|p| p.join(APP_IDENTIFIER))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|p| p.join(APP_IDENTIFIER))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|p| p.join(APP_IDENTIFIER))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// Default port for bundled server
const DEFAULT_BUNDLED_SERVER_PORT: u16 = 3000;

/// Load configuration from a specific file path.
/// Returns None if the file doesn't exist or can't be parsed.
pub fn load_config_from_path(path: &Path) -> Option<ResolvedConfig> {
    let contents = std::fs::read_to_string(path).ok()?;
    let settings: DesktopSettings = serde_json::from_str(&contents).ok()?;

    // Determine server URL and token based on server mode
    let (server_url, token) = if settings.use_bundled_server {
        // When using bundled server, connect to localhost with the configured port
        let port = settings.server_port.unwrap_or(DEFAULT_BUNDLED_SERVER_PORT);
        let url = format!("http://localhost:{}", port);
        (url, settings.bundled_server_token)
    } else {
        // When using external server, use the configured server address
        (settings.server_address, settings.external_server_token)
    };

    Some(ResolvedConfig { server_url, token })
}

/// Try to load configuration from the Clipper desktop app's settings file.
/// Returns None if the settings file doesn't exist or can't be parsed.
pub fn load_desktop_config() -> Option<ResolvedConfig> {
    let config_dir = get_app_config_dir()?;
    let settings_path = config_dir.join(SETTINGS_FILE_NAME);
    load_config_from_path(&settings_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_config_dir() {
        // Just ensure it doesn't panic
        let _ = get_app_config_dir();
    }
}
