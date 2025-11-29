use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "clipper-server")]
#[command(about = "Clipper server with REST API and WebSocket support", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, env = "CLIPPER_CONFIG")]
    pub config: Option<PathBuf>,

    /// Database path
    #[arg(long, env = "CLIPPER_DB_PATH")]
    pub db_path: Option<String>,

    /// Storage path for file attachments
    #[arg(long, env = "CLIPPER_STORAGE_PATH")]
    pub storage_path: Option<String>,

    /// Server listen address
    #[arg(long, env = "CLIPPER_LISTEN_ADDR")]
    pub listen_addr: Option<String>,

    /// Server listen port (HTTP)
    #[arg(short, long, env = "PORT")]
    pub port: Option<u16>,

    // TLS options
    /// Enable HTTPS/TLS
    #[arg(long, env = "CLIPPER_TLS_ENABLED")]
    pub tls_enabled: Option<bool>,

    /// HTTPS port (default: 443)
    #[arg(long, env = "CLIPPER_TLS_PORT")]
    pub tls_port: Option<u16>,

    /// Path to TLS certificate file (PEM format)
    #[arg(long, env = "CLIPPER_TLS_CERT")]
    pub tls_cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format)
    #[arg(long, env = "CLIPPER_TLS_KEY")]
    pub tls_key: Option<PathBuf>,

    /// Redirect HTTP to HTTPS
    #[arg(long, env = "CLIPPER_TLS_REDIRECT")]
    pub tls_redirect: Option<bool>,

    /// Interval in seconds to reload certificates from disk (0 = disabled)
    /// Useful when certificates are managed by external tools
    #[arg(long, env = "CLIPPER_TLS_RELOAD_INTERVAL")]
    pub tls_reload_interval: Option<u64>,

    // ACME options
    /// Enable ACME automatic certificate management
    #[arg(long, env = "CLIPPER_ACME_ENABLED")]
    pub acme_enabled: Option<bool>,

    /// Domain name for ACME certificate
    #[arg(long, env = "CLIPPER_ACME_DOMAIN")]
    pub acme_domain: Option<String>,

    /// Contact email for ACME (Let's Encrypt notifications)
    #[arg(long, env = "CLIPPER_ACME_EMAIL")]
    pub acme_email: Option<String>,

    /// Use ACME staging environment (for testing)
    #[arg(long, env = "CLIPPER_ACME_STAGING")]
    pub acme_staging: Option<bool>,

    /// Directory for certificate cache
    #[arg(long, env = "CLIPPER_CERTS_DIR")]
    pub certs_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub server: NetworkConfig,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub acme: AcmeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub port: u16,
}

/// TLS/HTTPS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable HTTPS
    pub enabled: bool,
    /// HTTPS port (default: 443)
    pub port: u16,
    /// Path to certificate file (PEM format)
    pub cert_path: Option<PathBuf>,
    /// Path to private key file (PEM format)
    pub key_path: Option<PathBuf>,
    /// Redirect HTTP to HTTPS when both are enabled
    pub redirect_http: bool,
    /// Interval in seconds to reload certificates from disk (0 = disabled)
    /// Useful when certificates are managed by external tools like certbot
    #[serde(default)]
    pub reload_interval_secs: u64,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 443,
            cert_path: None,
            key_path: None,
            redirect_http: true,
            reload_interval_secs: 0, // Disabled by default
        }
    }
}

impl TlsConfig {
    /// Check if certificate reload is enabled
    pub fn reload_enabled(&self) -> bool {
        self.reload_interval_secs > 0
    }

    /// Get reload interval as Duration, returns None if disabled
    pub fn reload_interval(&self) -> Option<std::time::Duration> {
        if self.reload_interval_secs > 0 {
            Some(std::time::Duration::from_secs(self.reload_interval_secs))
        } else {
            None
        }
    }
}

/// ACME (Let's Encrypt) automatic certificate configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AcmeConfig {
    /// Enable ACME automatic certificate management
    pub enabled: bool,
    /// Domain name for the certificate
    pub domain: Option<String>,
    /// Contact email for Let's Encrypt notifications
    pub contact_email: Option<String>,
    /// Use staging environment (for testing, avoids rate limits)
    pub staging: bool,
    /// Directory for certificate cache
    pub certs_dir: Option<PathBuf>,
}

impl AcmeConfig {
    /// Get the certificates directory, using a default if not specified
    pub fn get_certs_dir(&self) -> PathBuf {
        self.certs_dir.clone().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("com.0d0a.clipper")
                .join("certs")
        })
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: "./data/db".to_string(),
            },
            storage: StorageConfig {
                path: "./data/storage".to_string(),
            },
            server: NetworkConfig {
                listen_addr: "0.0.0.0".to_string(),
                port: 3000,
            },
            tls: TlsConfig::default(),
            acme: AcmeConfig::default(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from multiple sources with priority:
    /// 1. Command line arguments (highest priority)
    /// 2. Environment variables
    /// 3. Config file
    /// 4. Defaults (lowest priority)
    pub fn load(cli: Cli) -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder()
            .add_source(config::Config::try_from(&ServerConfig::default())?);

        // Load from config file if specified
        if let Some(config_path) = &cli.config {
            tracing::info!("Loading config from file: {}", config_path.display());
            builder = builder.add_source(config::File::from(config_path.as_ref()).required(false));
        } else {
            // Try to load from default locations
            builder = builder
                .add_source(config::File::with_name("clipper-server").required(false))
                .add_source(config::File::with_name("config").required(false));
        }

        // Build initial config
        let mut cfg: ServerConfig = builder.build()?.try_deserialize()?;

        // Override with CLI arguments (highest priority)
        if let Some(db_path) = cli.db_path {
            cfg.database.path = db_path;
        }

        if let Some(storage_path) = cli.storage_path {
            cfg.storage.path = storage_path;
        }

        if let Some(listen_addr) = cli.listen_addr {
            cfg.server.listen_addr = listen_addr;
        }

        if let Some(port) = cli.port {
            cfg.server.port = port;
        }

        // TLS configuration overrides
        if let Some(tls_enabled) = cli.tls_enabled {
            cfg.tls.enabled = tls_enabled;
        }

        if let Some(tls_port) = cli.tls_port {
            cfg.tls.port = tls_port;
        }

        if let Some(tls_cert) = cli.tls_cert {
            cfg.tls.cert_path = Some(tls_cert);
        }

        if let Some(tls_key) = cli.tls_key {
            cfg.tls.key_path = Some(tls_key);
        }

        if let Some(tls_redirect) = cli.tls_redirect {
            cfg.tls.redirect_http = tls_redirect;
        }

        if let Some(tls_reload_interval) = cli.tls_reload_interval {
            cfg.tls.reload_interval_secs = tls_reload_interval;
        }

        // ACME configuration overrides
        if let Some(acme_enabled) = cli.acme_enabled {
            cfg.acme.enabled = acme_enabled;
        }

        if let Some(acme_domain) = cli.acme_domain {
            cfg.acme.domain = Some(acme_domain);
        }

        if let Some(acme_email) = cli.acme_email {
            cfg.acme.contact_email = Some(acme_email);
        }

        if let Some(acme_staging) = cli.acme_staging {
            cfg.acme.staging = acme_staging;
        }

        if let Some(certs_dir) = cli.certs_dir {
            cfg.acme.certs_dir = Some(certs_dir);
        }

        // If ACME is enabled, TLS should also be enabled
        if cfg.acme.enabled {
            cfg.tls.enabled = true;
        }

        Ok(cfg)
    }

    /// Get the HTTP socket address to bind to
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.server.listen_addr, self.server.port).parse()
    }

    /// Get the HTTPS socket address to bind to
    pub fn tls_socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.server.listen_addr, self.tls.port).parse()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Only validate TLS settings if TLS is enabled in config
        if self.tls.enabled {
            // Check if TLS feature is compiled in
            #[cfg(not(feature = "tls"))]
            {
                return Err(
                    "TLS is enabled in config but the 'tls' feature is not compiled in. \
                     Rebuild with --features tls or set tls.enabled = false."
                        .to_string(),
                );
            }

            // If TLS is enabled but ACME is not, we need manual cert paths
            #[cfg(feature = "tls")]
            if !self.acme.enabled {
                if self.tls.cert_path.is_none() {
                    return Err("TLS enabled but no certificate path provided. \
                         Set tls.cert_path or enable ACME."
                        .to_string());
                }
                if self.tls.key_path.is_none() {
                    return Err("TLS enabled but no key path provided. \
                         Set tls.key_path or enable ACME."
                        .to_string());
                }
            }
        }

        // Only validate ACME settings if ACME is enabled in config
        if self.acme.enabled {
            // Check if ACME feature is compiled in
            #[cfg(not(feature = "acme"))]
            {
                return Err(
                    "ACME is enabled in config but the 'acme' feature is not compiled in. \
                     Rebuild with --features acme or set acme.enabled = false."
                        .to_string(),
                );
            }

            #[cfg(feature = "acme")]
            {
                if self.acme.domain.is_none() {
                    return Err("ACME enabled but no domain provided. Set acme.domain.".to_string());
                }
                if self.acme.contact_email.is_none() {
                    return Err(
                        "ACME enabled but no contact email provided. Set acme.contact_email."
                            .to_string(),
                    );
                }
            }
        }

        Ok(())
    }

    /// Check if TLS is available (feature compiled and enabled in config)
    pub fn tls_available(&self) -> bool {
        #[cfg(feature = "tls")]
        {
            self.tls.enabled
        }
        #[cfg(not(feature = "tls"))]
        {
            false
        }
    }

    /// Check if ACME is available (feature compiled and enabled in config)
    pub fn acme_available(&self) -> bool {
        #[cfg(feature = "acme")]
        {
            self.acme.enabled
        }
        #[cfg(not(feature = "acme"))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.database.path, "./data/db");
        assert_eq!(config.storage.path, "./data/storage");
        assert_eq!(config.server.listen_addr, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert!(!config.tls.enabled);
        assert_eq!(config.tls.port, 443);
        assert!(!config.acme.enabled);
    }

    #[test]
    fn test_socket_addr() {
        let config = ServerConfig::default();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "0.0.0.0:3000");
    }

    #[test]
    fn test_tls_socket_addr() {
        let config = ServerConfig::default();
        let addr = config.tls_socket_addr().unwrap();
        assert_eq!(addr.to_string(), "0.0.0.0:443");
    }

    #[test]
    fn test_validate_tls_disabled() {
        // TLS disabled should always validate OK
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_tls_without_certs() {
        let mut config = ServerConfig::default();
        config.tls.enabled = true;
        // Should fail - either feature not compiled or certs missing
        assert!(config.validate().is_err());
    }

    #[test]
    #[cfg(feature = "tls")]
    fn test_validate_tls_with_certs() {
        let mut config = ServerConfig::default();
        config.tls.enabled = true;
        config.tls.cert_path = Some(PathBuf::from("/path/to/cert.pem"));
        config.tls.key_path = Some(PathBuf::from("/path/to/key.pem"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_acme_without_domain() {
        let mut config = ServerConfig::default();
        config.acme.enabled = true;
        config.tls.enabled = true;
        assert!(config.validate().is_err());
    }

    #[test]
    #[cfg(feature = "acme")]
    fn test_validate_acme_with_domain_and_email() {
        let mut config = ServerConfig::default();
        config.acme.enabled = true;
        config.tls.enabled = true;
        config.acme.domain = Some("example.com".to_string());
        config.acme.contact_email = Some("admin@example.com".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_acme_certs_dir() {
        let config = AcmeConfig::default();
        let certs_dir = config.get_certs_dir();
        assert!(certs_dir.to_string_lossy().contains("com.0d0a.clipper"));
    }

    #[test]
    fn test_tls_available_when_disabled() {
        let config = ServerConfig::default();
        // TLS is disabled by default, so tls_available should be false
        assert!(!config.tls_available());
    }

    #[test]
    #[cfg(feature = "tls")]
    fn test_tls_available_when_enabled() {
        let mut config = ServerConfig::default();
        config.tls.enabled = true;
        assert!(config.tls_available());
    }

    #[test]
    fn test_acme_available_when_disabled() {
        let config = ServerConfig::default();
        assert!(!config.acme_available());
    }

    #[test]
    #[cfg(feature = "acme")]
    fn test_acme_available_when_enabled() {
        let mut config = ServerConfig::default();
        config.acme.enabled = true;
        assert!(config.acme_available());
    }
}
