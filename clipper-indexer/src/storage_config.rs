//! Storage backend configuration types for clipper-indexer.
//!
//! This module defines configuration structures for various storage backends:
//! - Local filesystem (default)
//! - AWS S3 (requires `aws` feature)
//! - Azure Blob Storage (requires `azure` feature)

use serde::{Deserialize, Serialize};

/// Configuration for the storage backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageBackendConfig {
    /// Local filesystem storage (default)
    Local(LocalStorageConfig),
    /// AWS S3 storage (requires `aws` feature)
    #[cfg(feature = "aws")]
    S3(S3StorageConfig),
    /// Azure Blob Storage (requires `azure` feature)
    #[cfg(feature = "azure")]
    Azure(AzureStorageConfig),
}

impl Default for StorageBackendConfig {
    fn default() -> Self {
        StorageBackendConfig::Local(LocalStorageConfig::default())
    }
}

/// Local filesystem storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    /// Path to the storage directory
    pub path: String,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            path: "./data/storage".to_string(),
        }
    }
}

/// AWS S3 storage configuration.
///
/// Supports multiple authentication methods:
/// - Access key and secret key
/// - IAM role (when running on AWS)
/// - Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
/// - AWS credentials file (~/.aws/credentials)
#[cfg(feature = "aws")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3StorageConfig {
    /// S3 bucket name
    pub bucket: String,
    /// AWS region (e.g., "us-east-1")
    pub region: String,
    /// Optional prefix/folder within the bucket
    #[serde(default)]
    pub prefix: Option<String>,
    /// Optional custom endpoint URL (for S3-compatible storage like MinIO)
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
    #[serde(default = "default_virtual_hosted_style")]
    pub virtual_hosted_style_request: bool,
    /// Whether to allow HTTP (non-HTTPS) connections (default: false)
    #[serde(default)]
    pub allow_http: bool,
}

#[cfg(feature = "aws")]
fn default_virtual_hosted_style() -> bool {
    true
}

#[cfg(feature = "aws")]
impl Default for S3StorageConfig {
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
///
/// Supports multiple authentication methods:
/// - Account key (connection string or explicit key)
/// - Shared Access Signature (SAS) token
/// - Managed Identity / Workload Identity (Azure AD)
/// - Service Principal (client credentials)
/// - Azure CLI credentials
#[cfg(feature = "azure")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureStorageConfig {
    /// Azure storage account name
    pub account: String,
    /// Azure blob container name
    pub container: String,
    /// Optional prefix/folder within the container
    #[serde(default)]
    pub prefix: Option<String>,
    /// Authentication method to use
    #[serde(default)]
    pub auth: AzureAuthConfig,
    /// Custom endpoint URL (optional, for Azure Government, China, etc.)
    #[serde(default)]
    pub endpoint: Option<String>,
}

#[cfg(feature = "azure")]
impl Default for AzureStorageConfig {
    fn default() -> Self {
        Self {
            account: String::new(),
            container: String::new(),
            prefix: None,
            auth: AzureAuthConfig::default(),
            endpoint: None,
        }
    }
}

/// Azure authentication configuration.
///
/// Supports the following methods:
/// - `ManagedIdentity`: Azure Managed Identity / Workload Identity (recommended for Azure-hosted apps)
/// - `ServicePrincipal`: Azure AD service principal with client ID and secret
/// - `AccountKey`: Storage account access key
/// - `Sas`: Shared Access Signature token
/// - `AzureCli`: Use credentials from Azure CLI login
/// - `Default`: Use Azure default credential chain (tries multiple methods)
#[cfg(feature = "azure")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum AzureAuthConfig {
    /// Use Azure Managed Identity / Workload Identity Federation
    /// This is the recommended method for applications running in Azure
    /// (App Service, AKS, Azure Functions, VMs with managed identity)
    ManagedIdentity {
        /// Optional client ID for user-assigned managed identity
        /// If not specified, system-assigned managed identity is used
        #[serde(default)]
        client_id: Option<String>,
        /// Optional object/principal ID (alternative to client_id)
        #[serde(default)]
        object_id: Option<String>,
        /// Optional MSI resource ID (for user-assigned managed identity)
        #[serde(default)]
        msi_resource_id: Option<String>,
        /// Federated token file path (for Workload Identity on AKS)
        /// Usually set via AZURE_FEDERATED_TOKEN_FILE environment variable
        #[serde(default)]
        federated_token_file: Option<String>,
        /// Authority host URL (for sovereign clouds)
        /// Defaults to https://login.microsoftonline.com
        #[serde(default)]
        authority_host: Option<String>,
    },
    /// Use Azure AD Service Principal (application)
    ServicePrincipal {
        /// Azure AD tenant ID
        tenant_id: String,
        /// Service principal (application) client ID
        client_id: String,
        /// Client secret
        client_secret: String,
        /// Authority host URL (for sovereign clouds)
        #[serde(default)]
        authority_host: Option<String>,
    },
    /// Use storage account access key
    AccountKey {
        /// The storage account access key
        key: String,
    },
    /// Use Shared Access Signature (SAS) token
    Sas {
        /// The SAS token (without leading '?')
        token: String,
    },
    /// Use Azure CLI credentials (useful for local development)
    AzureCli,
    /// Use default Azure credential chain
    /// Tries: Environment -> Managed Identity -> Azure CLI -> etc.
    Default,
}

#[cfg(feature = "azure")]
impl Default for AzureAuthConfig {
    fn default() -> Self {
        // Default to using the default credential chain
        AzureAuthConfig::Default
    }
}

impl StorageBackendConfig {
    /// Get a description of the storage backend type
    pub fn backend_type(&self) -> &'static str {
        match self {
            StorageBackendConfig::Local(_) => "local",
            #[cfg(feature = "aws")]
            StorageBackendConfig::S3(_) => "s3",
            #[cfg(feature = "azure")]
            StorageBackendConfig::Azure(_) => "azure",
        }
    }

    /// Check if this is a local storage backend
    pub fn is_local(&self) -> bool {
        matches!(self, StorageBackendConfig::Local(_))
    }

    /// Create a local storage config from a path
    pub fn local(path: impl Into<String>) -> Self {
        StorageBackendConfig::Local(LocalStorageConfig { path: path.into() })
    }

    /// Create an S3 storage config (requires `aws` feature)
    #[cfg(feature = "aws")]
    pub fn s3(bucket: impl Into<String>, region: impl Into<String>) -> Self {
        StorageBackendConfig::S3(S3StorageConfig {
            bucket: bucket.into(),
            region: region.into(),
            ..Default::default()
        })
    }

    /// Create an Azure storage config (requires `azure` feature)
    #[cfg(feature = "azure")]
    pub fn azure(account: impl Into<String>, container: impl Into<String>) -> Self {
        StorageBackendConfig::Azure(AzureStorageConfig {
            account: account.into(),
            container: container.into(),
            ..Default::default()
        })
    }
}

#[cfg(feature = "azure")]
impl AzureAuthConfig {
    /// Create a Managed Identity auth config (system-assigned)
    pub fn managed_identity() -> Self {
        AzureAuthConfig::ManagedIdentity {
            client_id: None,
            object_id: None,
            msi_resource_id: None,
            federated_token_file: None,
            authority_host: None,
        }
    }

    /// Create a Managed Identity auth config with a specific client ID (user-assigned)
    pub fn managed_identity_with_client_id(client_id: impl Into<String>) -> Self {
        AzureAuthConfig::ManagedIdentity {
            client_id: Some(client_id.into()),
            object_id: None,
            msi_resource_id: None,
            federated_token_file: None,
            authority_host: None,
        }
    }

    /// Create a Workload Identity auth config (for AKS)
    pub fn workload_identity(
        client_id: impl Into<String>,
        federated_token_file: impl Into<String>,
    ) -> Self {
        AzureAuthConfig::ManagedIdentity {
            client_id: Some(client_id.into()),
            object_id: None,
            msi_resource_id: None,
            federated_token_file: Some(federated_token_file.into()),
            authority_host: None,
        }
    }

    /// Create a Service Principal auth config
    pub fn service_principal(
        tenant_id: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        AzureAuthConfig::ServicePrincipal {
            tenant_id: tenant_id.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            authority_host: None,
        }
    }

    /// Create an Account Key auth config
    pub fn account_key(key: impl Into<String>) -> Self {
        AzureAuthConfig::AccountKey { key: key.into() }
    }

    /// Create a SAS token auth config
    pub fn sas_token(token: impl Into<String>) -> Self {
        AzureAuthConfig::Sas {
            token: token.into(),
        }
    }

    /// Create an Azure CLI auth config
    pub fn azure_cli() -> Self {
        AzureAuthConfig::AzureCli
    }

    /// Check if this auth method supports automatic token refresh
    pub fn supports_auto_refresh(&self) -> bool {
        matches!(
            self,
            AzureAuthConfig::ManagedIdentity { .. }
                | AzureAuthConfig::ServicePrincipal { .. }
                | AzureAuthConfig::AzureCli
                | AzureAuthConfig::Default
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_storage_config() {
        let config = StorageBackendConfig::default();
        assert!(config.is_local());
        assert_eq!(config.backend_type(), "local");
    }

    #[test]
    fn test_local_storage_config() {
        let config = StorageBackendConfig::local("/custom/path");
        match config {
            StorageBackendConfig::Local(local) => {
                assert_eq!(local.path, "/custom/path");
            }
            #[allow(unreachable_patterns)]
            _ => panic!("Expected local storage config"),
        }
    }

    #[cfg(feature = "aws")]
    #[test]
    fn test_s3_storage_config() {
        let config = StorageBackendConfig::s3("my-bucket", "us-west-2");
        match config {
            StorageBackendConfig::S3(s3) => {
                assert_eq!(s3.bucket, "my-bucket");
                assert_eq!(s3.region, "us-west-2");
                assert!(s3.virtual_hosted_style_request);
            }
            _ => panic!("Expected S3 storage config"),
        }
    }

    #[cfg(feature = "azure")]
    #[test]
    fn test_azure_storage_config() {
        let config = StorageBackendConfig::azure("myaccount", "mycontainer");
        match config {
            StorageBackendConfig::Azure(azure) => {
                assert_eq!(azure.account, "myaccount");
                assert_eq!(azure.container, "mycontainer");
            }
            _ => panic!("Expected Azure storage config"),
        }
    }

    #[cfg(feature = "azure")]
    #[test]
    fn test_azure_auth_methods() {
        // Test managed identity
        let auth = AzureAuthConfig::managed_identity();
        assert!(auth.supports_auto_refresh());

        // Test service principal
        let auth =
            AzureAuthConfig::service_principal("tenant-id", "client-id", "client-secret");
        assert!(auth.supports_auto_refresh());

        // Test account key
        let auth = AzureAuthConfig::account_key("my-key");
        assert!(!auth.supports_auto_refresh());

        // Test SAS token
        let auth = AzureAuthConfig::sas_token("sv=2021-06-08&ss=b&srt=sco...");
        assert!(!auth.supports_auto_refresh());
    }

    #[test]
    fn test_local_config_serialization() {
        let config = StorageBackendConfig::local("/data/storage");
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"local\""));
        assert!(json.contains("/data/storage"));

        let deserialized: StorageBackendConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_local());
    }

    #[cfg(feature = "aws")]
    #[test]
    fn test_s3_config_serialization() {
        let config = StorageBackendConfig::S3(S3StorageConfig {
            bucket: "test-bucket".to_string(),
            region: "eu-west-1".to_string(),
            prefix: Some("clips/".to_string()),
            endpoint: Some("https://s3.custom.endpoint".to_string()),
            access_key_id: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            secret_access_key: Some("secret".to_string()),
            session_token: None,
            virtual_hosted_style_request: false,
            allow_http: false,
        });

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"type\": \"s3\""));
        assert!(json.contains("test-bucket"));
        assert!(json.contains("eu-west-1"));

        let deserialized: StorageBackendConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.backend_type(), "s3");
    }

    #[cfg(feature = "azure")]
    #[test]
    fn test_azure_config_serialization() {
        let config = StorageBackendConfig::Azure(AzureStorageConfig {
            account: "mystorageaccount".to_string(),
            container: "clipper-files".to_string(),
            prefix: Some("v1/".to_string()),
            auth: AzureAuthConfig::ManagedIdentity {
                client_id: Some("12345678-1234-1234-1234-123456789012".to_string()),
                object_id: None,
                msi_resource_id: None,
                federated_token_file: None,
                authority_host: None,
            },
            endpoint: None,
        });

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"type\": \"azure\""));
        assert!(json.contains("mystorageaccount"));
        assert!(json.contains("managed_identity"));

        let deserialized: StorageBackendConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.backend_type(), "azure");
    }
}
