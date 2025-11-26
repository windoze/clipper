# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Clipper is a clipboard management system with three main components:
- **clipper-indexer**: Core library for indexing and searching clipboard entries using SurrealDB (RocksDB backend) and object_store
- **clipper-server**: REST API server with WebSocket support for real-time clip updates
- **clipper**: CLI application (currently placeholder)

## Build & Test Commands

### Building

```bash
# Build entire workspace
cargo build --workspace

# Build specific package
cargo build -p clipper-indexer
cargo build -p clipper-server
cargo build -p clipper

# Release build
cargo build --workspace --release
```

### Testing

```bash
# Run all tests in workspace
cargo test --workspace

# Test specific package
cargo test -p clipper-indexer
cargo test -p clipper-server

# Run specific test file
cargo test --test api_tests -p clipper-server

# Run single test
cargo test test_create_clip -p clipper-server

# Run tests sequentially (important for server tests to avoid database conflicts)
cargo test --test api_tests -p clipper-server -- --test-threads=1
```

### Running

```bash
# Run the server (requires CLIPPER_DB_PATH and CLIPPER_STORAGE_PATH env vars or uses defaults)
cargo run --bin clipper-server

# With custom paths
CLIPPER_DB_PATH=./data/db CLIPPER_STORAGE_PATH=./data/storage cargo run --bin clipper-server
```

## Architecture

### Data Flow

1. **clipper-indexer (Core Library)**
   - `ClipperIndexer` is the main entry point
   - Uses SurrealDB for metadata and full-text search (BM25)
   - Uses object_store (LocalFileSystem) for file attachments
   - All operations are async (Tokio runtime)

2. **clipper-server (REST API + WebSocket)**
   - Built with Axum framework
   - `AppState` wraps `Arc<ClipperIndexer>` and broadcast channel for WebSocket updates
   - REST endpoints in `api.rs`: CRUD operations, search, file upload
   - WebSocket in `websocket.rs`: real-time clip updates
   - All state mutations trigger WebSocket notifications

3. **Database Schema (SurrealDB)**
   - Table: `clipboard` with fields: id, content, created_at, tags, additional_notes, file_attachment, search_content
   - Indexes: created_at, tags, full-text search on search_content
   - Schema auto-initialized in `ClipperIndexer::new()`

### Key Design Decisions

- **File Storage**: Files stored separately via object_store, not in database. Entry contains file_key reference.
- **Search Content**: Concatenation of content + additional_notes for full-text indexing
- **WebSocket Updates**: Broadcast channel pattern - all connected clients receive clip events (NewClip, UpdatedClip, DeletedClip)
- **Testing**: Server tests use temporary databases (TempDir) for isolation

## Important Patterns

### Adding New API Endpoints

1. Add handler function in `clipper-server/src/api.rs`
2. Register route in `pub fn routes()` 
3. If it modifies clips, call `state.notify_*()` for WebSocket updates
4. Add test in `clipper-server/tests/api_tests.rs`

### Working with ClipperIndexer

```rust
// Always use ClipperIndexer through Arc in server context
let indexer = ClipperIndexer::new(db_path, storage_path).await?;
let state = AppState::new(indexer); // Wraps in Arc

// For file uploads, use add_entry_from_file_content (bytes + filename)
// For local files, use add_entry_from_file (path)
```

### Error Handling

- `clipper_indexer::IndexerError` - core library errors
- `clipper_server::ServerError` - server-specific errors (implements IntoResponse)
- Server errors automatically converted to JSON responses with appropriate HTTP status codes

## Environment Configuration

Server reads these environment variables:
- `CLIPPER_DB_PATH` (default: `./data/db`)
- `CLIPPER_STORAGE_PATH` (default: `./data/storage`)
- `PORT` (default: `3000`)
- `RUST_LOG` for tracing (default: `clipper_server=debug,tower_http=debug`)

## Testing Notes

- Server tests must run sequentially: `-- --test-threads=1`
- Each test creates isolated temporary database
- Tests use raw HTTP requests via tower::ServiceExt (not axum-test due to version compatibility)
- Multipart file upload tests construct raw HTTP multipart bodies
