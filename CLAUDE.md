# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Documentation Index

Each subproject has its own CLAUDE.md with detailed information:

| Component | Path | Description |
|-----------|------|-------------|
| **clipper-indexer** | [clipper-indexer/CLAUDE.md](clipper-indexer/CLAUDE.md) | Core library for indexing and searching (SurrealDB, object_store) |
| **clipper-server** | [clipper-server/CLAUDE.md](clipper-server/CLAUDE.md) | REST API server with WebSocket support |
| **clipper-server/web** | [clipper-server/web/CLAUDE.md](clipper-server/web/CLAUDE.md) | Web UI frontend (React + Vite) |
| **clipper-client** | [clipper-client/CLAUDE.md](clipper-client/CLAUDE.md) | Rust client library for server API |
| **clipper-cli** | [clipper-cli/CLAUDE.md](clipper-cli/CLAUDE.md) | Command-line interface application |
| **clipper** (Tauri) | [clipper/CLAUDE.md](clipper/CLAUDE.md) | Desktop GUI (Tauri 2 + React + TypeScript) |
| **clipper-slint** | [clipper-slint/CLAUDE.md](clipper-slint/CLAUDE.md) | Alternative GUI (Slint UI framework) |
| **@unwritten-codes/clipper-ui** | [packages/clipper-ui/CLAUDE.md](packages/clipper-ui/CLAUDE.md) | Shared React UI component library |

## Project Overview

Clipper is a clipboard management system with seven main components:
- **clipper-indexer**: Core library for indexing and searching clipboard entries using SurrealDB (RocksDB backend) and object_store
- **clipper-server**: REST API server with WebSocket support for real-time clip updates, includes built-in web UI
- **clipper-server/web**: Pure frontend Web UI (React + Vite) for browser-based access
- **clipper-client**: Rust client library for interacting with the server REST API and WebSocket
- **clipper-cli**: Command-line interface application for managing clips
- **clipper** (Tauri): Desktop GUI application built with Tauri 2 + React + TypeScript
- **clipper-slint**: Alternative GUI application built with Slint UI framework

## Upgrading Version

The version info is in following files:

- `Cargo.toml` (workspace and each package)
- `clipper-server/web/package.json` (Server web UI)
- `clipper/package.json`
- `clipper/src-tauri/package.json`
- `clipper/src-tauri/tauri.conf.json`
- `packages/clipper-ui/package.json`
- `Dockerfile`
- `Dockerfile.backup`
- `README.md` and `README.zh-CN.md`

All above files should be updated to keep consistent version.

Then we should run `npm install` in following directories to update lock files:
1. `packages/clipper-ui/`
2. `clipper/`
3. `clipper-server/web/`

And run `cargo check` in the root to update Cargo.lock.

Then we need to update the changelog in `CHANGELOG.md` base on git history since the last release.

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
cargo build -p clipper          # Tauri backend (requires frontend build first)
cargo build -p clipper-slint

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

# Run tests sequentially (important for server/client tests)
cargo test --test api_tests -p clipper-server -- --test-threads=1
cargo test --test integration_tests -p clipper-client -- --test-threads=1
```

## Architecture Overview

### Data Flow

1. **clipper-indexer** - Core library with `ClipperIndexer` as main entry point
2. **clipper-server** - Axum-based REST API + WebSocket server
3. **clipper-client** - reqwest + tokio-tungstenite client library
4. **clipper-cli** - clap-based CLI using clipper-client
5. **clipper** (Tauri) - Desktop app with bundled server
6. **clipper-slint** - Alternative Slint-based GUI
7. **clipper-server/web** - React frontend served by clipper-server

### Key Design Decisions

- **File Storage**: Files stored separately via object_store, not in database
- **Search Content**: Concatenation of content + additional_notes for full-text indexing
- **WebSocket Updates**: Broadcast channel pattern - all clients receive clip events
- **Pagination**: Implemented at indexer level with `PagingParams` and `PagedResult<T>`
- **Configuration**: Multi-source with priority: CLI args > env vars > config file > defaults
- **Testing**: Server and client tests use temporary databases (TempDir) for isolation

## Important Patterns

### Pagination

All search and list operations support pagination:

```rust
// Indexer level
let paging = PagingParams { page: 1, page_size: 20 };
let result: PagedResult<ClipboardEntry> = indexer.search_entries(query, filters, paging).await?;

// Client level
let result = client.search_clips(query, filters, page, page_size).await?;

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

### WebSocket Notifications

Server broadcasts four types of notifications:
- `NewClip`: { type: "new_clip", id, content, tags }
- `UpdatedClip`: { type: "updated_clip", id }
- `DeletedClip`: { type: "deleted_clip", id }
- `ClipsCleanedUp`: { type: "clips_cleaned_up", ids, count }

### Clip Sharing (Short URLs)

Clips can be shared publicly via short URLs:
- **Enable**: Set `CLIPPER_SHORT_URL_BASE` env var (e.g., `https://clip.example.com`)
- **Create**: `POST /clips/:id/short-url` returns a short URL
- **Access**: `GET /s/:code` resolves to clip content (HTML page, JSON, or file download)
- **Expiration**: Default 24 hours, configurable via `CLIPPER_SHORT_URL_EXPIRATION_HOURS`
- **UI**: Share button appears in Tauri app and Web UI when enabled

### Export/Import

The server supports exporting and importing clips via tar.gz archives:
- **Export**: `GET /export` returns a tar.gz archive with all clips and attachments
- **Import**: `POST /import` (multipart form) imports from a tar.gz archive with deduplication
- **Archive format**: Contains `manifest.json` with clip metadata and `files/` directory for attachments
- **Deduplication**: Clips are skipped if same ID or same content hash already exists
- **Short URLs**: Not included in export (they are ephemeral/local to each server)

### Self-Signed Certificate Trust

Both the CLI and desktop app support connecting to HTTPS servers with self-signed certificates:
- **Storage**: Trusted certificate fingerprints stored in `trustedCertificates` field of settings.json
- **Verification**: SHA-256 fingerprint displayed for user verification (similar to SSH)
- **Security**: Fingerprint change detection warns users (like SSH's "REMOTE HOST IDENTIFICATION HAS CHANGED")
- **CLI**: Interactive prompt on first connection to untrusted server
- **Desktop**: UI dialog showing certificate details and trust options
- **Shared storage**: Both CLI and desktop app share the same trusted certificates store

### Error Handling

- `clipper_indexer::IndexerError` - core library errors
- `clipper_server::ServerError` - server-specific errors (implements IntoResponse)
- `clipper_client::ClientError` - client-specific errors
- CLI uses anyhow for error context
- Tauri commands return `Result<T, String>` for frontend error handling

## Testing Notes

- Server tests must run sequentially: `-- --test-threads=1`
- Client tests must run sequentially: `-- --test-threads=1`
- Each test creates isolated temporary database
- **Total test coverage**: clipper-indexer (all core operations), clipper-server (81 tests), clipper-client (20 tests)

## Project Status

### Completed
- Core indexer with full-text search and pagination
- REST API server with all CRUD operations
- WebSocket real-time notifications
- File attachment support (including images)
- Rust client library with full API coverage
- CLI application with all major operations
- Multi-source configuration system
- Comprehensive test coverage (54+ tests across packages)
- Tauri Desktop Application with bundled server
- Slint GUI alternative (basic implementation)
- Web UI with full feature parity
- TLS/HTTPS Support with ACME
- Auto-cleanup with configurable retention
- Authentication (Bearer token) across all components
- Clip sharing via short URLs (optional, requires `CLIPPER_SHORT_URL_BASE`)
- File content preview/rendering improvements
- Keyboard shortcuts
- Global hotkey support
- Advanced search operators
- Export/import functionality (tar.gz archive with clips and attachments, deduplication on import)
- Self-signed certificate trust (SSH-like fingerprint verification)
- Streaming file upload/download for reduced memory consumption
- Tags management with dedicated tags table
- Search result highlighting
- PII detection before sharing

### Future Work
- Clipboard monitoring daemon (standalone)
- Complete Slint GUI alternative
