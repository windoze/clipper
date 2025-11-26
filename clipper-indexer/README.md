# Clipper Indexer

A Rust library for indexing and searching clipboard entries using SurrealDB and object_store.

## Features

- **Persistent Storage**: Uses SurrealDB with RocksDB backend for reliable data persistence
- **Full-Text Search**: Powered by SurrealDB's full-text search with BM25 ranking
- **Pagination Support**: Built-in pagination for search and list operations
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
use clipper_indexer::{ClipperIndexer, SearchFilters, PagingParams};

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

    // Search for entries with pagination
    let paging = PagingParams { page: 1, page_size: 20 };
    let result = indexer
        .search_entries("Hello", SearchFilters::new(), paging)
        .await?;

    println!("Found {} entries (page {} of {})", 
             result.total, result.page, result.total_pages);

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

### Add Entry from File Content

For uploaded files or in-memory content:

```rust
let entry = indexer
    .add_entry_from_file_content(
        file_bytes,
        filename,
        tags,
        optional_notes,
    )
    .await?;
```

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

### Search Entries with Pagination

Full-text search with optional filters and pagination:

```rust
use chrono::{Duration, Utc};

let filters = SearchFilters::new()
    .with_tags(vec!["rust".to_string()])
    .with_date_range(
        Utc::now() - Duration::days(7),
        Utc::now(),
    );

let paging = PagingParams {
    page: 1,
    page_size: 20,
};

let result = indexer
    .search_entries("search query", filters, paging)
    .await?;

println!("Page {} of {}, Total entries: {}", 
         result.page, result.total_pages, result.total);

for entry in result.items {
    println!("- {}: {}", entry.id, entry.content);
}
```

### List Entries with Pagination

List entries with filters (without full-text search):

```rust
let filters = SearchFilters::new()
    .with_tags(vec!["important".to_string()]);

let paging = PagingParams::default(); // page: 1, page_size: 20

let result = indexer.list_entries(filters, paging).await?;

println!("Showing {} of {} total entries", 
         result.items.len(), result.total);
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

## Pagination

The library provides built-in pagination support for search and list operations:

### PagingParams

```rust
pub struct PagingParams {
    pub page: usize,        // Page number (starting from 1)
    pub page_size: usize,   // Number of items per page
}

// Default: page 1, page_size 20
let paging = PagingParams::default();

// Custom pagination
let paging = PagingParams { page: 2, page_size: 50 };
```

### PagedResult

```rust
pub struct PagedResult<T> {
    pub items: Vec<T>,       // Items for current page
    pub total: usize,        // Total number of items
    pub page: usize,         // Current page number
    pub page_size: usize,    // Items per page
    pub total_pages: usize,  // Total number of pages
}
```

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
- Adding entries from file content (bytes)
- Retrieving and updating entries
- Full-text search functionality with pagination
- Date range and tag filtering
- File storage and retrieval
- Entry deletion
- Pagination edge cases

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
- **Pagination**: Efficient LIMIT/OFFSET queries with metadata calculation
- **Models**: Type-safe data models with serde serialization

## Performance Considerations

- **Pagination**: Uses SQL LIMIT and OFFSET for efficient page retrieval
- **Indexes**: Automatically indexes created_at, tags, and search_content for fast queries
- **File Storage**: Large files stored separately from database to maintain query performance
- **Search**: BM25 ranking provides relevant results even with large datasets

## Requirements

- Rust 1.70 or later
- Tokio runtime (async)
- Sufficient disk space for database and file storage

## License

See the main project license.
