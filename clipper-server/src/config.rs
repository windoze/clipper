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

    /// Server listen port
    #[arg(short, long, env = "PORT")]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub server: NetworkConfig,
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

        Ok(cfg)
    }

    /// Get the socket address to bind to
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.server.listen_addr, self.server.port).parse()
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
    }

    #[test]
    fn test_socket_addr() {
        let config = ServerConfig::default();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "0.0.0.0:3000");
    }
}
