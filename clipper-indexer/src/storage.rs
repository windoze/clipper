//! Storage backend implementations for clipper-indexer.
//!
//! This module provides a unified `FileStorage` abstraction over different storage backends:
//! - Local filesystem (always available)
//! - AWS S3 (requires `aws` feature)
//! - Azure Blob Storage (requires `azure` feature)

use crate::error::{IndexerError, Result};
use crate::storage_config::{LocalStorageConfig, StorageBackendConfig};
use bytes::Bytes;
use object_store::{local::LocalFileSystem, path::Path as ObjectPath, ObjectStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature = "aws")]
use crate::storage_config::S3StorageConfig;

#[cfg(feature = "azure")]
use crate::storage_config::{AzureAuthConfig, AzureStorageConfig};

/// Unified file storage abstraction supporting multiple backends.
///
/// This struct provides a consistent interface for file operations regardless
/// of the underlying storage backend (local filesystem, S3, Azure Blob, etc.).
pub struct FileStorage {
    store: Arc<dyn ObjectStore>,
    /// Base path for local storage (None for cloud storage)
    base_path: Option<PathBuf>,
    /// Storage backend type description
    backend_type: &'static str,
    /// Optional prefix for cloud storage
    prefix: Option<String>,
}

impl FileStorage {
    /// Create a new FileStorage instance from configuration.
    ///
    /// # Arguments
    /// * `config` - Storage backend configuration
    ///
    /// # Returns
    /// A configured FileStorage instance
    #[allow(clippy::result_large_err)]
    pub fn from_config(config: &StorageBackendConfig) -> Result<Self> {
        match config {
            StorageBackendConfig::Local(local_config) => Self::new_local(local_config),
            #[cfg(feature = "aws")]
            StorageBackendConfig::S3(s3_config) => Self::new_s3(s3_config),
            #[cfg(feature = "azure")]
            StorageBackendConfig::Azure(azure_config) => Self::new_azure(azure_config),
        }
    }

    /// Create a new local filesystem storage.
    ///
    /// This is the default and always available storage backend.
    #[allow(clippy::result_large_err)]
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let config = LocalStorageConfig {
            path: base_path.as_ref().to_string_lossy().to_string(),
        };
        Self::new_local(&config)
    }

    /// Create a new local filesystem storage from config.
    #[allow(clippy::result_large_err)]
    fn new_local(config: &LocalStorageConfig) -> Result<Self> {
        let base_path = PathBuf::from(&config.path);

        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&base_path)?;

        let store =
            LocalFileSystem::new_with_prefix(&base_path).map_err(IndexerError::ObjectStore)?;

        tracing::info!("Initialized local file storage at: {}", base_path.display());

        Ok(Self {
            store: Arc::new(store),
            base_path: Some(base_path),
            backend_type: "local",
            prefix: None,
        })
    }

    /// Create a new AWS S3 storage backend.
    ///
    /// # Arguments
    /// * `config` - S3 storage configuration
    ///
    /// # Returns
    /// A configured FileStorage instance for S3
    #[cfg(feature = "aws")]
    #[allow(clippy::result_large_err)]
    fn new_s3(config: &S3StorageConfig) -> Result<Self> {
        use object_store::aws::AmazonS3Builder;

        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&config.bucket)
            .with_region(&config.region);

        // Set custom endpoint if provided (for MinIO, LocalStack, etc.)
        if let Some(endpoint) = &config.endpoint {
            builder = builder.with_endpoint(endpoint);
        }

        // Set credentials if provided explicitly
        if let Some(access_key_id) = &config.access_key_id {
            builder = builder.with_access_key_id(access_key_id);
        }

        if let Some(secret_access_key) = &config.secret_access_key {
            builder = builder.with_secret_access_key(secret_access_key);
        }

        if let Some(session_token) = &config.session_token {
            builder = builder.with_token(session_token);
        }

        // Configure addressing style
        if !config.virtual_hosted_style_request {
            builder = builder.with_virtual_hosted_style_request(false);
        }

        // Allow HTTP if configured (useful for local testing)
        if config.allow_http {
            builder = builder.with_allow_http(true);
        }

        let store = builder.build().map_err(IndexerError::ObjectStore)?;

        tracing::info!(
            "Initialized S3 storage: bucket={}, region={}",
            config.bucket,
            config.region
        );

        Ok(Self {
            store: Arc::new(store),
            base_path: None,
            backend_type: "s3",
            prefix: config.prefix.clone(),
        })
    }

    /// Create a new Azure Blob Storage backend.
    ///
    /// Supports multiple authentication methods including Managed Identity,
    /// Service Principal, Account Key, and SAS tokens.
    ///
    /// # Arguments
    /// * `config` - Azure storage configuration
    ///
    /// # Returns
    /// A configured FileStorage instance for Azure Blob Storage
    #[cfg(feature = "azure")]
    #[allow(clippy::result_large_err)]
    fn new_azure(config: &AzureStorageConfig) -> Result<Self> {
        use object_store::azure::MicrosoftAzureBuilder;

        let mut builder = MicrosoftAzureBuilder::new()
            .with_account(&config.account)
            .with_container_name(&config.container);

        // Set custom endpoint if provided (for sovereign clouds)
        if let Some(endpoint) = &config.endpoint {
            builder = builder.with_url(endpoint);
        }

        // Configure authentication based on the auth method
        builder = match &config.auth {
            AzureAuthConfig::ManagedIdentity {
                client_id,
                federated_token_file,
                authority_host,
                ..
            } => {
                let mut b = builder.with_use_azure_cli(false);

                // Enable managed identity / workload identity
                if let Some(client_id) = client_id {
                    b = b.with_client_id(client_id);
                }

                // For Workload Identity on AKS, set the federated token file
                if let Some(token_file) = federated_token_file {
                    b = b.with_federated_token_file(token_file);
                }

                // Set authority host for sovereign clouds
                if let Some(authority) = authority_host {
                    b = b.with_authority_host(authority);
                }

                // If we have tenant_id from environment (for workload identity)
                if let Ok(tenant_id) = std::env::var("AZURE_TENANT_ID") {
                    b = b.with_tenant_id(&tenant_id);
                }

                tracing::info!(
                    "Using Azure Managed Identity authentication (client_id: {:?})",
                    client_id
                );

                b
            }
            AzureAuthConfig::ServicePrincipal {
                tenant_id,
                client_id,
                client_secret,
                authority_host,
            } => {
                let mut b = builder
                    .with_tenant_id(tenant_id)
                    .with_client_id(client_id)
                    .with_client_secret(client_secret);

                if let Some(authority) = authority_host {
                    b = b.with_authority_host(authority);
                }

                tracing::info!(
                    "Using Azure Service Principal authentication (tenant_id: {}, client_id: {})",
                    tenant_id,
                    client_id
                );

                b
            }
            AzureAuthConfig::AccountKey { key } => {
                tracing::info!("Using Azure Account Key authentication");
                builder.with_access_key(key)
            }
            AzureAuthConfig::Sas { token } => {
                tracing::info!("Using Azure SAS token authentication");
                // Parse SAS token into query pairs
                // SAS tokens look like: sv=2021-06-08&ss=b&srt=sco&sp=rwdlacup&...
                let query_pairs: Vec<(String, String)> = token
                    .trim_start_matches('?')
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        match (parts.next(), parts.next()) {
                            (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                            _ => None,
                        }
                    })
                    .collect();
                builder.with_sas_authorization(query_pairs)
            }
            AzureAuthConfig::AzureCli => {
                tracing::info!("Using Azure CLI authentication");
                builder.with_use_azure_cli(true)
            }
            AzureAuthConfig::Default => {
                // Use default credential chain - object_store will try:
                // 1. Environment variables
                // 2. Managed Identity
                // 3. Azure CLI
                tracing::info!("Using Azure default credential chain");

                // Check for environment variables
                if let Ok(key) = std::env::var("AZURE_STORAGE_KEY") {
                    builder = builder.with_access_key(&key);
                } else if let Ok(sas) = std::env::var("AZURE_STORAGE_SAS_TOKEN") {
                    // Parse SAS token into query pairs
                    let query_pairs: Vec<(String, String)> = sas
                        .trim_start_matches('?')
                        .split('&')
                        .filter_map(|pair| {
                            let mut parts = pair.splitn(2, '=');
                            match (parts.next(), parts.next()) {
                                (Some(key), Some(value)) => {
                                    Some((key.to_string(), value.to_string()))
                                }
                                _ => None,
                            }
                        })
                        .collect();
                    builder = builder.with_sas_authorization(query_pairs);
                } else if let Ok(tenant) = std::env::var("AZURE_TENANT_ID") {
                    // Service Principal via environment
                    if let (Ok(client_id), Ok(client_secret)) = (
                        std::env::var("AZURE_CLIENT_ID"),
                        std::env::var("AZURE_CLIENT_SECRET"),
                    ) {
                        builder = builder
                            .with_tenant_id(&tenant)
                            .with_client_id(&client_id)
                            .with_client_secret(&client_secret);
                    } else if let Ok(federated_token_file) =
                        std::env::var("AZURE_FEDERATED_TOKEN_FILE")
                    {
                        // Workload Identity
                        if let Ok(client_id) = std::env::var("AZURE_CLIENT_ID") {
                            builder = builder
                                .with_tenant_id(&tenant)
                                .with_client_id(&client_id)
                                .with_federated_token_file(&federated_token_file);
                        }
                    }
                }

                builder
            }
        };

        let store = builder.build().map_err(IndexerError::ObjectStore)?;

        tracing::info!(
            "Initialized Azure Blob storage: account={}, container={}",
            config.account,
            config.container
        );

        Ok(Self {
            store: Arc::new(store),
            base_path: None,
            backend_type: "azure",
            prefix: config.prefix.clone(),
        })
    }

    /// Get the full object path, including any configured prefix.
    fn get_object_path(&self, file_key: &str) -> ObjectPath {
        match &self.prefix {
            Some(prefix) => {
                let full_path = format!("{}/{}", prefix.trim_end_matches('/'), file_key);
                ObjectPath::from(full_path)
            }
            None => ObjectPath::from(file_key),
        }
    }

    /// Store a file from the local filesystem.
    ///
    /// # Arguments
    /// * `source_path` - Path to the source file on the local filesystem
    ///
    /// # Returns
    /// The file key (unique identifier) for the stored file
    pub async fn put_file(&self, source_path: impl AsRef<Path>) -> Result<String> {
        let source_path = source_path.as_ref();

        // Read the file content
        let content = tokio::fs::read(source_path).await?;

        // Generate a unique filename using UUID
        let file_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| IndexerError::InvalidInput("Invalid file name".to_string()))?;

        let unique_id = uuid::Uuid::new_v4();
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let stored_file_name = if extension.is_empty() {
            format!("{}_{}", unique_id, file_name)
        } else {
            format!(
                "{}_{}.{}",
                unique_id,
                file_name.trim_end_matches(&format!(".{}", extension)),
                extension
            )
        };

        let object_path = self.get_object_path(&stored_file_name);

        // Store the file
        self.store
            .put(&object_path, Bytes::from(content).into())
            .await
            .map_err(IndexerError::ObjectStore)?;

        Ok(stored_file_name)
    }

    /// Store file content directly from bytes.
    ///
    /// # Arguments
    /// * `content` - The file content as bytes
    /// * `original_filename` - The original filename (used for extension extraction)
    ///
    /// # Returns
    /// The file key (unique identifier) for the stored file
    pub async fn put_file_bytes(&self, content: Bytes, original_filename: &str) -> Result<String> {
        // Generate a unique filename using UUID
        let unique_id = uuid::Uuid::new_v4();

        // Extract extension from original filename
        let extension = Path::new(original_filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let base_name = Path::new(original_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let stored_file_name = if extension.is_empty() {
            format!("{}_{}", unique_id, base_name)
        } else {
            format!("{}_{}.{}", unique_id, base_name, extension)
        };

        let object_path = self.get_object_path(&stored_file_name);

        // Store the file
        self.store
            .put(&object_path, content.into())
            .await
            .map_err(IndexerError::ObjectStore)?;

        Ok(stored_file_name)
    }

    /// Retrieve a file by its key.
    ///
    /// # Arguments
    /// * `file_key` - The unique identifier for the file
    ///
    /// # Returns
    /// The file content as bytes
    pub async fn get_file(&self, file_key: &str) -> Result<Bytes> {
        let object_path = self.get_object_path(file_key);

        let result = self
            .store
            .get(&object_path)
            .await
            .map_err(IndexerError::ObjectStore)?;

        let bytes = result.bytes().await.map_err(IndexerError::ObjectStore)?;

        Ok(bytes)
    }

    /// Delete a file by its key.
    ///
    /// # Arguments
    /// * `file_key` - The unique identifier for the file to delete
    pub async fn delete_file(&self, file_key: &str) -> Result<()> {
        let object_path = self.get_object_path(file_key);

        self.store
            .delete(&object_path)
            .await
            .map_err(IndexerError::ObjectStore)?;

        Ok(())
    }

    /// Get the base path for local storage.
    ///
    /// Returns `None` for cloud storage backends.
    pub fn get_base_path(&self) -> Option<&Path> {
        self.base_path.as_deref()
    }

    /// Get the storage backend type.
    pub fn backend_type(&self) -> &'static str {
        self.backend_type
    }

    /// Check if this is local storage.
    pub fn is_local(&self) -> bool {
        self.backend_type == "local"
    }

    /// Check if this is cloud storage.
    pub fn is_cloud(&self) -> bool {
        !self.is_local()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage_put_get_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        assert!(storage.is_local());
        assert_eq!(storage.backend_type(), "local");

        // Create a test file
        let test_content = b"Hello, World!";
        let file_key = storage
            .put_file_bytes(Bytes::from_static(test_content), "test.txt")
            .await
            .unwrap();

        assert!(file_key.ends_with(".txt"));

        // Get the file
        let content = storage.get_file(&file_key).await.unwrap();
        assert_eq!(content.as_ref(), test_content);

        // Delete the file
        storage.delete_file(&file_key).await.unwrap();

        // Verify it's deleted
        let result = storage.get_file(&file_key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_storage_from_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageBackendConfig::local(temp_dir.path().to_string_lossy().to_string());

        let storage = FileStorage::from_config(&config).unwrap();
        assert!(storage.is_local());
        assert_eq!(
            storage.get_base_path().unwrap(),
            temp_dir.path()
        );
    }

    #[test]
    fn test_storage_backend_type() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        assert_eq!(storage.backend_type(), "local");
        assert!(storage.is_local());
        assert!(!storage.is_cloud());
    }
}
