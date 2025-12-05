pub mod error;
pub mod indexer;
pub mod models;
pub mod storage;
pub mod storage_config;

pub use error::{IndexerError, Result};
pub use indexer::ClipperIndexer;
pub use models::{ClipboardEntry, PagedResult, PagingParams, SearchFilters, ShortUrl};
pub use storage::FileStorage;
pub use storage_config::{LocalStorageConfig, StorageBackendConfig};

#[cfg(feature = "aws")]
pub use storage_config::S3StorageConfig;

#[cfg(feature = "azure")]
pub use storage_config::{AzureAuthConfig, AzureStorageConfig};
