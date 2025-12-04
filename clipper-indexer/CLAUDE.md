# clipper-indexer

Core library for indexing and searching clipboard entries using SurrealDB (RocksDB backend) and object_store.

## Build & Test

```bash
# Build
cargo build -p clipper-indexer

# Test
cargo test -p clipper-indexer
```

## Architecture

- `ClipperIndexer` is the main entry point
- Uses SurrealDB for metadata and full-text search (BM25)
- Uses object_store (LocalFileSystem) for file attachments
- All operations are async (Tokio runtime)
- **Pagination support**: `search_entries()` and `list_entries()` return `PagedResult<ClipboardEntry>`

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
// Always use ClipperIndexer through Arc in server context
let indexer = ClipperIndexer::new(db_path, storage_path).await?;
let state = AppState::new(indexer); // Wraps in Arc

// For file uploads, use add_entry_from_file_content (bytes + filename)
// For local files, use add_entry_from_file (path)

// Pagination
let paging = PagingParams::default(); // page: 1, page_size: 20
let result = indexer.search_entries(query, filters, paging).await?;
```

## Pagination Pattern

```rust
let paging = PagingParams { page: 1, page_size: 20 };
let result: PagedResult<ClipboardEntry> = indexer.search_entries(query, filters, paging).await?;
```

## Error Handling

- `clipper_indexer::IndexerError` - core library errors
