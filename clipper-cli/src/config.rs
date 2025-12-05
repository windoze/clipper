//! Configuration loading for clipper-cli
//!
//! This module attempts to load configuration from the Clipper desktop app's
//! settings file. If the settings file is not available, it falls back to
//! default values which can be overridden by environment variables or CLI args.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const APP_IDENTIFIER: &str = "codes.unwritten.clipper";
const SETTINGS_FILE_NAME: &str = "settings.json";

/// Minimal settings structure that mirrors the desktop app's settings.json
/// We only deserialize the fields we need for the CLI.
#[derive(Debug, Deserialize, Serialize)]
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
    /// Trusted certificate fingerprints (host -> SHA-256 fingerprint)
    #[serde(default)]
    trusted_certificates: HashMap<String, String>,
}

/// Configuration resolved from the desktop app's settings
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub server_url: String,
    pub token: Option<String>,
    pub trusted_certificates: HashMap<String, String>,
    /// Path to the config file (for saving trusted certificates)
    pub config_path: Option<PathBuf>,
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

    Some(ResolvedConfig {
        server_url,
        token,
        trusted_certificates: settings.trusted_certificates,
        config_path: Some(path.to_path_buf()),
    })
}

/// Try to load configuration from the Clipper desktop app's settings file.
/// Returns None if the settings file doesn't exist or can't be parsed.
pub fn load_desktop_config() -> Option<ResolvedConfig> {
    let config_dir = get_app_config_dir()?;
    let settings_path = config_dir.join(SETTINGS_FILE_NAME);
    load_config_from_path(&settings_path)
}

/// Get the default config path (desktop app's settings file)
pub fn get_default_config_path() -> Option<PathBuf> {
    get_app_config_dir().map(|dir| dir.join(SETTINGS_FILE_NAME))
}

/// Save a trusted certificate to the config file
/// This updates the trustedCertificates field in the settings.json
pub fn save_trusted_certificate(
    config_path: &Path,
    host: &str,
    fingerprint: &str,
) -> std::io::Result<()> {
    // Read existing config or create new one
    let contents = std::fs::read_to_string(config_path).unwrap_or_else(|_| "{}".to_string());

    // Parse as generic JSON to preserve all fields
    let mut json: serde_json::Value =
        serde_json::from_str(&contents).unwrap_or_else(|_| serde_json::json!({}));

    // Ensure trustedCertificates exists
    if json.get("trustedCertificates").is_none() {
        json["trustedCertificates"] = serde_json::json!({});
    }

    // Add the new certificate
    json["trustedCertificates"][host] = serde_json::Value::String(fingerprint.to_string());

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write back
    let output = serde_json::to_string_pretty(&json).map_err(std::io::Error::other)?;
    std::fs::write(config_path, output)
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
