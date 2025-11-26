# Clipper Indexer

A Rust library for indexing and searching clipboard entries using SurrealDB and object_store.

## Features

- **Persistent Storage**: Uses SurrealDB with RocksDB backend for reliable data persistence
- **Full-Text Search**: Powered by SurrealDB's full-text search with BM25 ranking
- **File Attachments**: Store and retrieve files using the object_store crate
- **Flexible Filtering**: Search by date range, tags, and full-text queries
- **Type-Safe**: Fully typed API with comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
clipper_indexer = { path = "../clipper_indexer" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use clipper_indexer::{ClipperIndexer, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the indexer
    let indexer = ClipperIndexer::new("./db", "./storage").await?;

    // Add a text entry
    let entry = indexer
        .add_entry_from_text(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            Some("A friendly message".to_string()),
        )
        .await?;

    println!("Created entry with ID: {}", entry.id);

    // Search for entries
    let results = indexer
        .search_entries("Hello", SearchFilters::new())
        .await?;

    println!("Found {} entries", results.len());

    Ok(())
}
```

## API Overview

### Initialization

```rust
let indexer = ClipperIndexer::new(db_path, storage_path).await?;
```

### Add Entry from Text

```rust
let entry = indexer
    .add_entry_from_text(
        content,
        tags,
        optional_notes,
    )
    .await?;
```

### Add Entry from File

```rust
let entry = indexer
    .add_entry_from_file(
        file_path,
        tags,
        optional_notes,
    )
    .await?;
```

The file content is stored using `object_store` and the file path is saved in the entry.

### Retrieve Entry

```rust
let entry = indexer.get_entry(&entry_id).await?;
```

### Update Entry

```rust
let updated = indexer
    .update_entry(
        &entry_id,
        Some(new_tags),
        Some(new_notes),
    )
    .await?;
```

### Search Entries

Full-text search with optional filters:

```rust
use chrono::{Duration, Utc};

let filters = SearchFilters::new()
    .with_tags(vec!["rust".to_string()])
    .with_date_range(
        Utc::now() - Duration::days(7),
        Utc::now(),
    );

let results = indexer
    .search_entries("search query", filters)
    .await?;
```

### List Entries

List entries with filters (without full-text search):

```rust
let filters = SearchFilters::new()
    .with_tags(vec!["important".to_string()]);

let entries = indexer.list_entries(filters).await?;
```

### Get File Content

For entries with file attachments:

```rust
if let Some(file_key) = entry.file_attachment {
    let content = indexer.get_file_content(&file_key).await?;
    // Use the bytes content
}
```

### Delete Entry

```rust
indexer.delete_entry(&entry_id).await?;
```

This will also delete any associated file attachments.

## Database Schema

The library automatically creates the following schema:

### Table: clipboard

| Field | Type | Description |
|-------|------|-------------|
| id | string | Unique identifier (UUID) |
| content | string | Text content of the entry |
| created_at | datetime | Creation timestamp |
| tags | array\<string\> | List of tags |
| additional_notes | option\<string\> | Optional notes |
| file_attachment | option\<string\> | Optional file storage key |
| search_content | string | Combined content for full-text search |

### Indexes

- `idx_created_at`: Index on `created_at` for efficient date range queries
- `idx_tags`: Index on `tags` for tag filtering
- `idx_search_content`: Full-text search index with BM25 ranking and highlights

## Examples

See the [examples](./examples) directory for more detailed usage examples:

```bash
cargo run --example basic_usage
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Tests cover:
- Adding entries from text and files
- Retrieving and updating entries
- Full-text search functionality
- Date range and tag filtering
- File storage and retrieval
- Entry deletion

## Error Handling

The library provides a comprehensive error type:

```rust
pub enum IndexerError {
    Database(surrealdb::Error),
    ObjectStore(object_store::Error),
    Io(std::io::Error),
    NotFound(String),
    Serialization(String),
    InvalidInput(String),
}
```

All operations return `Result<T, IndexerError>`.

## Architecture

- **Database Layer**: SurrealDB with RocksDB backend provides ACID transactions and powerful querying
- **Storage Layer**: object_store handles file persistence with a clean abstraction
- **Search**: Full-text search powered by SurrealDB's built-in FTS with custom analyzer
- **Models**: Type-safe data models with serde serialization

## Requirements

- Rust 1.70 or later
- Tokio runtime (async)
- Sufficient disk space for database and file storage

## License

See the main project license.
