# clipper-indexer

Core library for indexing and searching clipboard entries using SurrealDB (RocksDB backend) and object_store.

## Build & Test

```bash
# Build (default - local storage only)
cargo build -p clipper-indexer

# Build with AWS S3 support
cargo build -p clipper-indexer --features aws

# Build with Azure Blob Storage support
cargo build -p clipper-indexer --features azure

# Build with all cloud storage backends
cargo build -p clipper-indexer --features cloud

# Test
cargo test -p clipper-indexer
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `aws` | Enable AWS S3 storage backend support |
| `azure` | Enable Azure Blob Storage backend support (includes Managed Identity / Workload Identity) |
| `cloud` | Enable all cloud storage backends (aws + azure) |

## Architecture

- `ClipperIndexer` is the main entry point
- Uses SurrealDB for metadata and full-text search (BM25)
- Uses object_store for file attachments (supports local, S3, Azure Blob)
- All operations are async (Tokio runtime)
- **Pagination support**: `search_entries()` and `list_entries()` return `PagedResult<ClipboardEntry>`

## Storage Backends

### Local Storage (default)
Files stored on local filesystem. Always available.

### AWS S3 (requires `aws` feature)
Supports:
- Access key and secret key authentication
- IAM role (when running on AWS)
- Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
- AWS credentials file (~/.aws/credentials)
- S3-compatible storage (MinIO, LocalStack, etc.)

### Azure Blob Storage (requires `azure` feature)
Supports multiple authentication methods:
- **Managed Identity / Workload Identity** (recommended for Azure-hosted apps)
- Service Principal (client credentials)
- Account Key
- SAS Token
- Azure CLI credentials
- Default credential chain

## Database Schema (SurrealDB)

- Table: `clipboard` with fields: id, content, created_at, tags, additional_notes, file_attachment, search_content
- Indexes: created_at, tags, full-text search on search_content
- Schema auto-initialized in `ClipperIndexer::new()`

## Key Design Decisions

- **File Storage**: Files stored separately via object_store, not in database. Entry contains file_key reference.
- **Search Content**: Concatenation of content + additional_notes for full-text indexing
- **Pagination**: Implemented with `PagingParams` and `PagedResult<T>`

## Working with ClipperIndexer

```rust
// Local storage (backward compatible)
let indexer = ClipperIndexer::new(db_path, storage_path).await?;

// Using storage configuration
use clipper_indexer::StorageBackendConfig;

// Local storage via config
let config = StorageBackendConfig::local("./data/storage");
let indexer = ClipperIndexer::new_with_config(db_path, config).await?;

// S3 storage (requires aws feature)
#[cfg(feature = "aws")]
{
    let config = StorageBackendConfig::s3("my-bucket", "us-east-1");
    let indexer = ClipperIndexer::new_with_config(db_path, config).await?;
}

// Azure storage (requires azure feature)
#[cfg(feature = "azure")]
{
    let config = StorageBackendConfig::azure("myaccount", "mycontainer");
    let indexer = ClipperIndexer::new_with_config(db_path, config).await?;
}

// For file uploads, use add_entry_from_file_content (bytes + filename)
// For local files, use add_entry_from_file (path)

// Pagination
let paging = PagingParams::default(); // page: 1, page_size: 20
let result = indexer.search_entries(query, filters, paging).await?;
```

## Azure Authentication Examples

```rust
#[cfg(feature = "azure")]
{
    use clipper_indexer::{AzureAuthConfig, AzureStorageConfig, StorageBackendConfig};

    // System-assigned Managed Identity (recommended for Azure VMs, App Service, etc.)
    let auth = AzureAuthConfig::managed_identity();

    // User-assigned Managed Identity
    let auth = AzureAuthConfig::managed_identity_with_client_id("client-id-here");

    // Workload Identity (AKS)
    let auth = AzureAuthConfig::workload_identity("client-id", "/var/run/secrets/azure/tokens/azure-identity-token");

    // Service Principal
    let auth = AzureAuthConfig::service_principal("tenant-id", "client-id", "client-secret");

    // Account Key
    let auth = AzureAuthConfig::account_key("your-storage-key");

    // SAS Token
    let auth = AzureAuthConfig::sas_token("sv=2021-06-08&ss=b&...");

    // Azure CLI (local development)
    let auth = AzureAuthConfig::azure_cli();

    // Apply auth to storage config
    let config = StorageBackendConfig::Azure(AzureStorageConfig {
        account: "myaccount".to_string(),
        container: "mycontainer".to_string(),
        prefix: Some("clips/".to_string()),
        auth,
        endpoint: None, // Use default, or set for sovereign clouds
    });
}
```

## Pagination Pattern

```rust
let paging = PagingParams { page: 1, page_size: 20 };
let result: PagedResult<ClipboardEntry> = indexer.search_entries(query, filters, paging).await?;
```

## Error Handling

- `clipper_indexer::IndexerError` - core library errors
