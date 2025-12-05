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

    /// Storage path for file attachments (used with local storage backend)
    #[arg(long, env = "CLIPPER_STORAGE_PATH")]
    pub storage_path: Option<String>,

    /// Storage backend type: local, s3, or azure
    #[arg(long, env = "CLIPPER_STORAGE_BACKEND")]
    pub storage_backend: Option<String>,

    // S3 storage options (require --features aws)
    /// S3 bucket name
    #[arg(long, env = "CLIPPER_S3_BUCKET")]
    pub s3_bucket: Option<String>,

    /// S3 region
    #[arg(long, env = "CLIPPER_S3_REGION")]
    pub s3_region: Option<String>,

    /// S3 prefix/folder
    #[arg(long, env = "CLIPPER_S3_PREFIX")]
    pub s3_prefix: Option<String>,

    /// S3 endpoint URL (for S3-compatible storage)
    #[arg(long, env = "CLIPPER_S3_ENDPOINT")]
    pub s3_endpoint: Option<String>,

    /// S3 access key ID
    #[arg(long, env = "AWS_ACCESS_KEY_ID")]
    pub s3_access_key_id: Option<String>,

    /// S3 secret access key
    #[arg(long, env = "AWS_SECRET_ACCESS_KEY")]
    pub s3_secret_access_key: Option<String>,

    // Azure storage options (require --features azure)
    /// Azure storage account name
    #[arg(long, env = "CLIPPER_AZURE_ACCOUNT")]
    pub azure_account: Option<String>,

    /// Azure blob container name
    #[arg(long, env = "CLIPPER_AZURE_CONTAINER")]
    pub azure_container: Option<String>,

    /// Azure storage prefix/folder
    #[arg(long, env = "CLIPPER_AZURE_PREFIX")]
    pub azure_prefix: Option<String>,

    /// Azure auth method: managed_identity, service_principal, account_key, sas, azure_cli, default
    #[arg(long, env = "CLIPPER_AZURE_AUTH_METHOD")]
    pub azure_auth_method: Option<String>,

    /// Azure client ID (for managed identity or service principal)
    #[arg(long, env = "AZURE_CLIENT_ID")]
    pub azure_client_id: Option<String>,

    /// Azure tenant ID (for service principal)
    #[arg(long, env = "AZURE_TENANT_ID")]
    pub azure_tenant_id: Option<String>,

    /// Azure client secret (for service principal)
    #[arg(long, env = "AZURE_CLIENT_SECRET")]
    pub azure_client_secret: Option<String>,

    /// Azure storage account key
    #[arg(long, env = "AZURE_STORAGE_KEY")]
    pub azure_account_key: Option<String>,

    /// Azure SAS token
    #[arg(long, env = "AZURE_STORAGE_SAS_TOKEN")]
    pub azure_sas_token: Option<String>,

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

    // Auth options
    /// Bearer token for authentication (if set, all requests must include this token)
    #[arg(long, env = "CLIPPER_BEARER_TOKEN")]
    pub bearer_token: Option<String>,

    // Cleanup options
    /// Enable automatic cleanup of old clips
    #[arg(long, env = "CLIPPER_CLEANUP_ENABLED")]
    pub cleanup_enabled: Option<bool>,

    /// Delete clips older than this many days (only clips without meaningful tags)
    #[arg(long, env = "CLIPPER_CLEANUP_RETENTION_DAYS")]
    pub cleanup_retention_days: Option<u32>,

    /// Interval in hours between cleanup runs
    #[arg(long, env = "CLIPPER_CLEANUP_INTERVAL_HOURS")]
    pub cleanup_interval_hours: Option<u32>,

    // Upload options
    /// Maximum upload size in megabytes (default: 10)
    #[arg(long, env = "CLIPPER_MAX_UPLOAD_SIZE_MB")]
    pub max_upload_size_mb: Option<u64>,

    // Short URL options
    /// Base URL for short URLs (e.g., "https://clip.example.com/s/")
    /// If not set, short URL functionality is disabled
    #[arg(long, env = "CLIPPER_SHORT_URL_BASE")]
    pub short_url_base: Option<String>,

    /// Default expiration time for short URLs in hours (default: 24, 0 = no expiration)
    #[arg(long, env = "CLIPPER_SHORT_URL_EXPIRATION_HOURS")]
    pub short_url_expiration_hours: Option<u32>,

    // Hidden option for parent process monitoring (used by bundled server in Tauri app)
    /// Pipe handle from parent process for lifecycle monitoring (internal use only)
    #[arg(long, hide = true)]
    pub parent_pipe_handle: Option<u64>,
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
    #[serde(default)]
    pub cleanup: CleanupConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub upload: UploadConfig,
    #[serde(default)]
    pub short_url: ShortUrlConfig,
}

/// Authentication configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Bearer token for authentication (if set, all requests must include this token)
    pub bearer_token: Option<String>,
}

impl AuthConfig {
    /// Check if authentication is required
    pub fn is_enabled(&self) -> bool {
        !self.bearer_token.as_deref().unwrap_or("").is_empty()
    }

    /// Validate a token against the configured bearer token
    pub fn validate_token(&self, token: &str) -> bool {
        match &self.bearer_token {
            Some(expected) if !expected.is_empty() => expected == token,
            _ => true, // No auth required
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

/// Storage configuration that supports multiple backends.
///
/// For local storage (default), only the `path` field is needed.
/// For cloud storage (S3, Azure), additional configuration is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type: "local", "s3", or "azure"
    #[serde(default = "default_storage_backend")]
    pub backend: String,

    /// Path for local storage (used when backend = "local")
    #[serde(default = "default_storage_path")]
    pub path: String,

    /// S3 configuration (used when backend = "s3")
    #[serde(default)]
    #[cfg(feature = "aws")]
    pub s3: Option<S3Config>,

    /// Azure configuration (used when backend = "azure")
    #[serde(default)]
    #[cfg(feature = "azure")]
    pub azure: Option<AzureConfig>,
}

fn default_storage_backend() -> String {
    "local".to_string()
}

fn default_storage_path() -> String {
    "./data/storage".to_string()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_storage_backend(),
            path: default_storage_path(),
            #[cfg(feature = "aws")]
            s3: None,
            #[cfg(feature = "azure")]
            azure: None,
        }
    }
}

/// AWS S3 storage configuration.
#[cfg(feature = "aws")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// AWS region (e.g., "us-east-1")
    pub region: String,
    /// Optional prefix/folder within the bucket
    #[serde(default)]
    pub prefix: Option<String>,
    /// Custom endpoint URL (for S3-compatible storage like MinIO)
    #[serde(default)]
    pub endpoint: Option<String>,
    /// AWS access key ID (optional, uses default credential chain if not set)
    #[serde(default)]
    pub access_key_id: Option<String>,
    /// AWS secret access key (optional, uses default credential chain if not set)
    #[serde(default)]
    pub secret_access_key: Option<String>,
    /// Optional session token for temporary credentials
    #[serde(default)]
    pub session_token: Option<String>,
    /// Whether to use virtual hosted-style addressing (default: true)
    #[serde(default = "default_true")]
    pub virtual_hosted_style_request: bool,
    /// Whether to allow HTTP (non-HTTPS) connections (default: false)
    #[serde(default)]
    pub allow_http: bool,
}

#[cfg(feature = "aws")]
fn default_true() -> bool {
    true
}

#[cfg(feature = "aws")]
impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: "us-east-1".to_string(),
            prefix: None,
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            virtual_hosted_style_request: true,
            allow_http: false,
        }
    }
}

/// Azure Blob Storage configuration.
#[cfg(feature = "azure")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    /// Azure storage account name
    pub account: String,
    /// Azure blob container name
    pub container: String,
    /// Optional prefix/folder within the container
    #[serde(default)]
    pub prefix: Option<String>,
    /// Custom endpoint URL (for sovereign clouds)
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Authentication method: "managed_identity", "service_principal", "account_key", "sas", "azure_cli", or "default"
    #[serde(default = "default_azure_auth_method")]
    pub auth_method: String,
    /// Client ID for managed identity (user-assigned) or service principal
    #[serde(default)]
    pub client_id: Option<String>,
    /// Tenant ID for service principal authentication
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Client secret for service principal authentication
    #[serde(default)]
    pub client_secret: Option<String>,
    /// Storage account access key
    #[serde(default)]
    pub account_key: Option<String>,
    /// SAS token
    #[serde(default)]
    pub sas_token: Option<String>,
    /// Federated token file path (for Workload Identity on AKS)
    #[serde(default)]
    pub federated_token_file: Option<String>,
    /// Authority host URL (for sovereign clouds)
    #[serde(default)]
    pub authority_host: Option<String>,
}

#[cfg(feature = "azure")]
fn default_azure_auth_method() -> String {
    "default".to_string()
}

#[cfg(feature = "azure")]
impl Default for AzureConfig {
    fn default() -> Self {
        Self {
            account: String::new(),
            container: String::new(),
            prefix: None,
            endpoint: None,
            auth_method: default_azure_auth_method(),
            client_id: None,
            tenant_id: None,
            client_secret: None,
            account_key: None,
            sas_token: None,
            federated_token_file: None,
            authority_host: None,
        }
    }
}

impl StorageConfig {
    /// Convert to clipper_indexer's StorageBackendConfig.
    pub fn to_backend_config(&self) -> Result<clipper_indexer::StorageBackendConfig, String> {
        match self.backend.as_str() {
            "local" => Ok(clipper_indexer::StorageBackendConfig::local(&self.path)),

            #[cfg(feature = "aws")]
            "s3" => {
                let s3 = self
                    .s3
                    .as_ref()
                    .ok_or_else(|| "S3 backend selected but s3 config is missing".to_string())?;

                if s3.bucket.is_empty() {
                    return Err("S3 bucket name is required".to_string());
                }

                let config = clipper_indexer::S3StorageConfig {
                    bucket: s3.bucket.clone(),
                    region: s3.region.clone(),
                    prefix: s3.prefix.clone(),
                    endpoint: s3.endpoint.clone(),
                    access_key_id: s3.access_key_id.clone(),
                    secret_access_key: s3.secret_access_key.clone(),
                    session_token: s3.session_token.clone(),
                    virtual_hosted_style_request: s3.virtual_hosted_style_request,
                    allow_http: s3.allow_http,
                };

                Ok(clipper_indexer::StorageBackendConfig::S3(config))
            }

            #[cfg(not(feature = "aws"))]
            "s3" => Err(
                "S3 storage backend requires the 'aws' feature. \
                 Rebuild with --features aws or use a different backend."
                    .to_string(),
            ),

            #[cfg(feature = "azure")]
            "azure" => {
                let azure = self
                    .azure
                    .as_ref()
                    .ok_or_else(|| "Azure backend selected but azure config is missing".to_string())?;

                if azure.account.is_empty() {
                    return Err("Azure storage account name is required".to_string());
                }
                if azure.container.is_empty() {
                    return Err("Azure container name is required".to_string());
                }

                // Convert auth method to AzureAuthConfig
                let auth = match azure.auth_method.as_str() {
                    "managed_identity" => clipper_indexer::AzureAuthConfig::ManagedIdentity {
                        client_id: azure.client_id.clone(),
                        object_id: None,
                        msi_resource_id: None,
                        federated_token_file: azure.federated_token_file.clone(),
                        authority_host: azure.authority_host.clone(),
                    },
                    "service_principal" => {
                        let tenant_id = azure
                            .tenant_id
                            .clone()
                            .ok_or_else(|| "tenant_id is required for service_principal auth".to_string())?;
                        let client_id = azure
                            .client_id
                            .clone()
                            .ok_or_else(|| "client_id is required for service_principal auth".to_string())?;
                        let client_secret = azure
                            .client_secret
                            .clone()
                            .ok_or_else(|| "client_secret is required for service_principal auth".to_string())?;

                        clipper_indexer::AzureAuthConfig::ServicePrincipal {
                            tenant_id,
                            client_id,
                            client_secret,
                            authority_host: azure.authority_host.clone(),
                        }
                    }
                    "account_key" => {
                        let key = azure
                            .account_key
                            .clone()
                            .ok_or_else(|| "account_key is required for account_key auth".to_string())?;
                        clipper_indexer::AzureAuthConfig::AccountKey { key }
                    }
                    "sas" => {
                        let token = azure
                            .sas_token
                            .clone()
                            .ok_or_else(|| "sas_token is required for sas auth".to_string())?;
                        clipper_indexer::AzureAuthConfig::Sas { token }
                    }
                    "azure_cli" => clipper_indexer::AzureAuthConfig::AzureCli,
                    "default" | "" => clipper_indexer::AzureAuthConfig::Default,
                    other => {
                        return Err(format!(
                            "Unknown Azure auth method: '{}'. Valid options: managed_identity, \
                             service_principal, account_key, sas, azure_cli, default",
                            other
                        ))
                    }
                };

                let config = clipper_indexer::AzureStorageConfig {
                    account: azure.account.clone(),
                    container: azure.container.clone(),
                    prefix: azure.prefix.clone(),
                    auth,
                    endpoint: azure.endpoint.clone(),
                };

                Ok(clipper_indexer::StorageBackendConfig::Azure(config))
            }

            #[cfg(not(feature = "azure"))]
            "azure" => Err(
                "Azure storage backend requires the 'azure' feature. \
                 Rebuild with --features azure or use a different backend."
                    .to_string(),
            ),

            other => Err(format!(
                "Unknown storage backend: '{}'. Valid options: local{}{}",
                other,
                if cfg!(feature = "aws") { ", s3" } else { "" },
                if cfg!(feature = "azure") { ", azure" } else { "" },
            )),
        }
    }

    /// Get a description of the storage backend for logging.
    pub fn backend_description(&self) -> String {
        match self.backend.as_str() {
            "local" => format!("local ({})", self.path),
            #[cfg(feature = "aws")]
            "s3" => {
                if let Some(s3) = &self.s3 {
                    format!("S3 (bucket: {}, region: {})", s3.bucket, s3.region)
                } else {
                    "S3 (not configured)".to_string()
                }
            }
            #[cfg(feature = "azure")]
            "azure" => {
                if let Some(azure) = &self.azure {
                    format!(
                        "Azure Blob (account: {}, container: {}, auth: {})",
                        azure.account, azure.container, azure.auth_method
                    )
                } else {
                    "Azure Blob (not configured)".to_string()
                }
            }
            other => format!("unknown ({})", other),
        }
    }
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

/// Auto-cleanup configuration for old clips
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Enable automatic cleanup of old clips
    pub enabled: bool,
    /// Delete clips older than this many days (only clips without meaningful tags)
    pub retention_days: u32,
    /// Interval in hours between cleanup runs (default: 24)
    pub interval_hours: u32,
}

/// Upload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// Maximum upload size in bytes (default: 10MB)
    pub max_size_bytes: u64,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl UploadConfig {
    /// Get the maximum upload size in megabytes
    pub fn max_size_mb(&self) -> f64 {
        self.max_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Short URL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrlConfig {
    /// Base URL for short URLs (e.g., "https://clip.example.com/s/")
    /// If not set (None or empty), short URL functionality is disabled
    pub base_url: Option<String>,
    /// Default expiration time for short URLs in hours (0 = no expiration)
    pub default_expiration_hours: u32,
}

impl Default for ShortUrlConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            default_expiration_hours: 24,
        }
    }
}

impl ShortUrlConfig {
    /// Check if short URL functionality is enabled
    pub fn is_enabled(&self) -> bool {
        self.base_url
            .as_ref()
            .is_some_and(|url| !url.trim().is_empty())
    }

    /// Get the full short URL for a given short code
    pub fn get_full_url(&self, short_code: &str) -> Option<String> {
        self.base_url.as_ref().map(|base| {
            let base = base.trim_end_matches('/');
            format!("{}/{}", base, short_code)
        })
    }

    /// Get the default expiration as Duration, None if no expiration
    pub fn default_expiration(&self) -> Option<std::time::Duration> {
        if self.default_expiration_hours > 0 {
            Some(std::time::Duration::from_secs(
                self.default_expiration_hours as u64 * 3600,
            ))
        } else {
            None
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            retention_days: 30,
            interval_hours: 24,
        }
    }
}

impl CleanupConfig {
    /// Check if cleanup is enabled and properly configured
    pub fn is_active(&self) -> bool {
        self.enabled && self.retention_days > 0 && self.interval_hours > 0
    }

    /// Get the cleanup interval as Duration
    pub fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval_hours as u64 * 3600)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: "./data/db".to_string(),
            },
            storage: StorageConfig::default(),
            server: NetworkConfig {
                listen_addr: "0.0.0.0".to_string(),
                port: 3000,
            },
            tls: TlsConfig::default(),
            acme: AcmeConfig::default(),
            cleanup: CleanupConfig::default(),
            auth: AuthConfig::default(),
            upload: UploadConfig::default(),
            short_url: ShortUrlConfig::default(),
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

        // Storage backend configuration
        if let Some(storage_backend) = cli.storage_backend {
            cfg.storage.backend = storage_backend;
        }

        // S3 configuration overrides
        #[cfg(feature = "aws")]
        {
            let has_s3_config = cli.s3_bucket.is_some() || cli.s3_region.is_some();
            if has_s3_config {
                let s3 = cfg.storage.s3.get_or_insert_with(S3Config::default);
                if let Some(bucket) = cli.s3_bucket {
                    s3.bucket = bucket;
                }
                if let Some(region) = cli.s3_region {
                    s3.region = region;
                }
                if let Some(prefix) = cli.s3_prefix {
                    s3.prefix = Some(prefix);
                }
                if let Some(endpoint) = cli.s3_endpoint {
                    s3.endpoint = Some(endpoint);
                }
                if let Some(access_key_id) = cli.s3_access_key_id {
                    s3.access_key_id = Some(access_key_id);
                }
                if let Some(secret_access_key) = cli.s3_secret_access_key {
                    s3.secret_access_key = Some(secret_access_key);
                }
            }
        }

        // Azure configuration overrides
        #[cfg(feature = "azure")]
        {
            let has_azure_config = cli.azure_account.is_some() || cli.azure_container.is_some();
            if has_azure_config {
                let azure = cfg.storage.azure.get_or_insert_with(AzureConfig::default);
                if let Some(account) = cli.azure_account {
                    azure.account = account;
                }
                if let Some(container) = cli.azure_container {
                    azure.container = container;
                }
                if let Some(prefix) = cli.azure_prefix {
                    azure.prefix = Some(prefix);
                }
                if let Some(auth_method) = cli.azure_auth_method {
                    azure.auth_method = auth_method;
                }
                if let Some(client_id) = cli.azure_client_id {
                    azure.client_id = Some(client_id);
                }
                if let Some(tenant_id) = cli.azure_tenant_id {
                    azure.tenant_id = Some(tenant_id);
                }
                if let Some(client_secret) = cli.azure_client_secret {
                    azure.client_secret = Some(client_secret);
                }
                if let Some(account_key) = cli.azure_account_key {
                    azure.account_key = Some(account_key);
                }
                if let Some(sas_token) = cli.azure_sas_token {
                    azure.sas_token = Some(sas_token);
                }
            }
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

        // Cleanup configuration overrides
        if let Some(cleanup_enabled) = cli.cleanup_enabled {
            cfg.cleanup.enabled = cleanup_enabled;
        }

        if let Some(retention_days) = cli.cleanup_retention_days {
            cfg.cleanup.retention_days = retention_days;
        }

        if let Some(interval_hours) = cli.cleanup_interval_hours {
            cfg.cleanup.interval_hours = interval_hours;
        }

        // Auth configuration overrides
        if let Some(bearer_token) = cli.bearer_token
            && !bearer_token.is_empty()
        {
            cfg.auth.bearer_token = Some(bearer_token);
        }

        // Upload configuration overrides
        if let Some(max_upload_size_mb) = cli.max_upload_size_mb {
            cfg.upload.max_size_bytes = max_upload_size_mb * 1024 * 1024;
        }

        // Short URL configuration overrides
        if let Some(short_url_base) = cli.short_url_base {
            cfg.short_url.base_url = Some(short_url_base);
        }

        if let Some(short_url_expiration_hours) = cli.short_url_expiration_hours {
            cfg.short_url.default_expiration_hours = short_url_expiration_hours;
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

    #[test]
    fn test_cleanup_default() {
        let config = CleanupConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.interval_hours, 24);
        assert!(!config.is_active());
    }

    #[test]
    fn test_cleanup_is_active() {
        let mut config = CleanupConfig::default();
        assert!(!config.is_active());

        config.enabled = true;
        assert!(config.is_active());

        config.retention_days = 0;
        assert!(!config.is_active());

        config.retention_days = 30;
        config.interval_hours = 0;
        assert!(!config.is_active());
    }

    #[test]
    fn test_cleanup_interval() {
        let config = CleanupConfig {
            interval_hours: 12,
            ..Default::default()
        };
        assert_eq!(config.interval(), std::time::Duration::from_secs(12 * 3600));
    }

    #[test]
    fn test_default_config_includes_cleanup() {
        let config = ServerConfig::default();
        assert!(!config.cleanup.enabled);
        assert_eq!(config.cleanup.retention_days, 30);
        assert_eq!(config.cleanup.interval_hours, 24);
    }
}
