# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Clipper is a clipboard management system with five main components:
- **clipper-indexer**: Core library for indexing and searching clipboard entries using SurrealDB (RocksDB backend) and object_store
- **clipper-server**: REST API server with WebSocket support for real-time clip updates
- **clipper-client**: Rust client library for interacting with the server REST API and WebSocket
- **clipper-cli**: Command-line interface application for managing clips
- **clipper**: Future GUI application (currently placeholder)

## Build & Test Commands

### Building

```bash
# Build entire workspace
cargo build --workspace

# Build specific package
cargo build -p clipper-indexer
cargo build -p clipper-server
cargo build -p clipper-client
cargo build -p clipper-cli

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
cargo test -p clipper-client

# Run specific test file
cargo test --test api_tests -p clipper-server
cargo test --test integration_tests -p clipper-client

# Run single test
cargo test test_create_clip -p clipper-server

# Run tests sequentially (important for server tests to avoid database conflicts)
cargo test --test api_tests -p clipper-server -- --test-threads=1
cargo test --test integration_tests -p clipper-client -- --test-threads=1
```

### Running

```bash
# Run the server (requires CLIPPER_DB_PATH and CLIPPER_STORAGE_PATH env vars or uses defaults)
cargo run --bin clipper-server

# With custom paths
CLIPPER_DB_PATH=./data/db CLIPPER_STORAGE_PATH=./data/storage cargo run --bin clipper-server

# With configuration file
cargo run --bin clipper-server -- --config config.toml

# Run CLI (requires server running)
cargo run --bin clipper-cli -- create "Hello, World!" --tags greeting
cargo run --bin clipper-cli -- search hello --page 1 --page-size 20
cargo run --bin clipper-cli -- watch
```

## Architecture

### Data Flow

1. **clipper-indexer (Core Library)**
   - `ClipperIndexer` is the main entry point
   - Uses SurrealDB for metadata and full-text search (BM25)
   - Uses object_store (LocalFileSystem) for file attachments
   - All operations are async (Tokio runtime)
   - **Pagination support**: `search_entries()` and `list_entries()` return `PagedResult<ClipboardEntry>`

2. **clipper-server (REST API + WebSocket)**
   - Built with Axum framework
   - `AppState` wraps `Arc<ClipperIndexer>` and broadcast channel for WebSocket updates
   - REST endpoints in `api.rs`: CRUD operations, search with pagination, file upload
   - WebSocket in `websocket.rs`: real-time clip updates
   - All state mutations trigger WebSocket notifications
   - **Configuration**: Multi-source configuration (CLI args, env vars, TOML files)

3. **clipper-client (Client Library)**
   - Built with reqwest for HTTP client
   - Uses tokio-tungstenite for WebSocket connections
   - Type-safe API wrapping all server endpoints
   - `subscribe_notifications()` for real-time updates via WebSocket
   - Full support for pagination in search and list operations

4. **clipper-cli (Command-Line Interface)**
   - Built with clap for argument parsing
   - Commands: create, get, update, search, delete, watch
   - Search with pagination support (--page, --page-size flags)
   - Output formats: JSON (default) or text
   - Watch command outputs NDJSON (newline-delimited JSON) for real-time updates

5. **Database Schema (SurrealDB)**
   - Table: `clipboard` with fields: id, content, created_at, tags, additional_notes, file_attachment, search_content
   - Indexes: created_at, tags, full-text search on search_content
   - Schema auto-initialized in `ClipperIndexer::new()`

### Key Design Decisions

- **File Storage**: Files stored separately via object_store, not in database. Entry contains file_key reference.
- **Search Content**: Concatenation of content + additional_notes for full-text indexing
- **WebSocket Updates**: Broadcast channel pattern - all connected clients receive clip events (NewClip, UpdatedClip, DeletedClip)
- **Testing**: Server and client tests use temporary databases (TempDir) for isolation
- **Pagination**: Implemented at indexer level with `PagingParams` and `PagedResult<T>`, exposed through API and CLI
- **Configuration Management**: Multi-source configuration with priority: CLI args > env vars > config file > defaults

## Important Patterns

### Pagination

All search and list operations support pagination:

```rust
// Indexer level
let paging = PagingParams { page: 1, page_size: 20 };
let result: PagedResult<ClipboardEntry> = indexer.search_entries(query, filters, paging).await?;

// Client level
let result = client.search_clips(query, filters, page, page_size).await?;
println!("Page {} of {}, Total: {}", result.page, result.total_pages, result.total);

// CLI level
clipper-cli search "hello" --page 1 --page-size 20
```

### Adding New API Endpoints

1. Add handler function in `clipper-server/src/api.rs`
2. Register route in `pub fn routes()` 
3. If it modifies clips, call `state.notify_*()` for WebSocket updates
4. Add test in `clipper-server/tests/api_tests.rs`
5. Add client method in `clipper-client/src/lib.rs`
6. Add test in `clipper-client/tests/integration_tests.rs`
7. Add CLI command in `clipper-cli/src/main.rs` if user-facing

### Working with ClipperIndexer

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

### WebSocket Notifications

Server broadcasts three types of notifications:

```rust
// NewClip: { type: "new_clip", id, content, tags }
// UpdatedClip: { type: "updated_clip", id }
// DeletedClip: { type: "deleted_clip", id }

// Client subscription
let (tx, mut rx) = mpsc::unbounded_channel();
let handle = client.subscribe_notifications(tx).await?;
while let Some(notification) = rx.recv().await {
    match notification {
        ClipNotification::NewClip { id, content, tags } => { /* handle */ }
        ClipNotification::UpdatedClip { id } => { /* handle */ }
        ClipNotification::DeletedClip { id } => { /* handle */ }
    }
}
```

### Error Handling

- `clipper_indexer::IndexerError` - core library errors
- `clipper_server::ServerError` - server-specific errors (implements IntoResponse)
- `clipper_client::ClientError` - client-specific errors
- Server errors automatically converted to JSON responses with appropriate HTTP status codes
- CLI uses anyhow for error context

## Environment Configuration

### Server Configuration

Multiple configuration sources (in priority order):
1. Command line arguments
2. Environment variables
3. Configuration file (TOML)
4. Default values

Environment variables:
- `CLIPPER_CONFIG` - Path to configuration file
- `CLIPPER_DB_PATH` (default: `./data/db`)
- `CLIPPER_STORAGE_PATH` (default: `./data/storage`)
- `CLIPPER_LISTEN_ADDR` (default: `0.0.0.0`)
- `PORT` (default: `3000`)
- `RUST_LOG` for tracing (default: `clipper_server=debug,tower_http=debug`)

### CLI Configuration

Environment variables:
- `CLIPPER_URL` - Server URL (default: `http://localhost:3000`)

## Testing Notes

- Server tests must run sequentially: `-- --test-threads=1`
- Client tests must run sequentially: `-- --test-threads=1`
- Each test creates isolated temporary database
- Server tests use raw HTTP requests via tower::ServiceExt
- Client tests require running server (tests start temporary server instances)
- Multipart file upload tests construct raw HTTP multipart bodies
- **Total test coverage**: clipper-indexer (all core operations), clipper-server (18 tests), clipper-client (18 tests)

## API Overview

### REST Endpoints

- `GET /health` - Health check
- `POST /clips` - Create clip from text
- `POST /clips/upload` - Upload file as clip
- `GET /clips` - List clips with pagination (query params: start_date, end_date, tags, page, page_size)
- `GET /clips/search` - Search clips with pagination (query params: q, start_date, end_date, tags, page, page_size)
- `GET /clips/:id` - Get clip by ID
- `PUT /clips/:id` - Update clip metadata
- `DELETE /clips/:id` - Delete clip
- `GET /clips/:id/file` - Download file attachment

### WebSocket Endpoint

- `WS /ws` - Real-time clip notifications

### CLI Commands

```bash
clipper-cli create <content> [--tags tag1,tag2] [--notes "notes"]
clipper-cli get <id> [--format json|text]
clipper-cli update <id> [--tags tag1,tag2] [--notes "notes"]
clipper-cli search <query> [--tags tag1,tag2] [--start-date ISO8601] [--end-date ISO8601] [--page 1] [--page-size 20] [--format json|text]
clipper-cli delete <id>
clipper-cli watch  # Real-time notifications as NDJSON
```

## Project Status

### Completed
- ✅ Core indexer with full-text search and pagination
- ✅ REST API server with all CRUD operations
- ✅ WebSocket real-time notifications
- ✅ File attachment support
- ✅ Rust client library with full API coverage
- ✅ CLI application with all major operations
- ✅ Multi-source configuration system
- ✅ Comprehensive test coverage (54+ tests across packages)

### Future Work
- 🔲 GUI application (clipper package)
- 🔲 File content preview/rendering
- 🔲 Advanced search operators
- 🔲 Export/import functionality
- 🔲 Clipboard monitoring daemon
