# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Clipper is a clipboard management system with seven main components:
- **clipper-indexer**: Core library for indexing and searching clipboard entries using SurrealDB (RocksDB backend) and object_store
- **clipper-server**: REST API server with WebSocket support for real-time clip updates, includes built-in web UI
- **clipper-server/web**: Pure frontend Web UI (React + Vite) for browser-based access
- **clipper-client**: Rust client library for interacting with the server REST API and WebSocket
- **clipper-cli**: Command-line interface application for managing clips
- **clipper** (Tauri): Desktop GUI application built with Tauri 2 + React + TypeScript
- **clipper-slint**: Alternative GUI application built with Slint UI framework

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

# Build with embedded web UI (for Docker/deployment)
cargo build -p clipper-server --release --features embed-web

# Build with TLS support (manual certificates)
cargo build -p clipper-server --features tls

# Build with ACME (Let's Encrypt automatic certificates)
cargo build -p clipper-server --features acme

# Build with full TLS + ACME + secure storage (OS keychain)
cargo build -p clipper-server --features full-tls
```

### Web UI (clipper-server/web)

```bash
# Install dependencies
cd clipper-server/web && npm install

# Development mode (requires server running on localhost:3000)
cd clipper-server/web && npm run dev

# Build production (output in dist/)
cd clipper-server/web && npm run build
```

### Tauri Application

```bash
# Install frontend dependencies
cd clipper && npm install

# Development mode (runs both frontend and backend)
cd clipper && npm run tauri dev

# Build production app
cd clipper && npm run tauri build
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

### Docker

```bash
# Build Docker image (includes embedded web UI)
docker build -t clipper-server .

# Run container
docker run -d -p 3000:3000 -v clipper-data:/data clipper-server

# Access at http://localhost:3000
```

## Architecture

### Data Flow

1. **clipper-indexer (Core Library)**
   - `ClipperIndexer` is the main entry point
   - Uses SurrealDB for metadata and full-text search (BM25)
   - Uses object_store (LocalFileSystem) for file attachments
   - All operations are async (Tokio runtime)
   - **Pagination support**: `search_entries()` and `list_entries()` return `PagedResult<ClipboardEntry>`

2. **clipper-server (REST API + WebSocket + Web UI)**
   - Built with Axum framework
   - `AppState` wraps `Arc<ClipperIndexer>` and broadcast channel for WebSocket updates
   - REST endpoints in `api.rs`: CRUD operations, search with pagination, file upload
   - WebSocket in `websocket.rs`: real-time clip updates
   - All state mutations trigger WebSocket notifications
   - **Configuration**: Multi-source configuration (CLI args, env vars, TOML files)
   - **Built-in Web UI**: Serves static files from `web/dist/` directory
   - **Web UI features**: View, search, edit, delete clips with i18n support (English/Chinese)

3. **clipper-server/web (Web UI Frontend)**
   - **Technology**: React 19 + TypeScript + Vite
   - **Features**:
     - View and search clips with infinite scroll
     - Edit clip tags and notes
     - Delete clips with confirmation
     - Image preview popup
     - Favorites filtering
     - Date range filtering
     - Theme support (light/dark/auto)
     - **Internationalization**: English and Chinese languages
   - **Architecture**: Pure frontend, communicates with server via REST API
   - **Components** (in `clipper-server/web/src/`):
     - `api/client.ts`: REST API client
     - `hooks/useClips.ts`: Clip data management with pagination
     - `hooks/useTheme.ts`: Theme persistence
     - `i18n/`: Internationalization (same pattern as Tauri app)
     - `components/`: Reusable UI components

4. **clipper-client (Client Library)**
   - Built with reqwest for HTTP client
   - Uses tokio-tungstenite for WebSocket connections
   - Type-safe API wrapping all server endpoints
   - `subscribe_notifications()` for real-time updates via WebSocket
   - Full support for pagination in search and list operations

5. **clipper-cli (Command-Line Interface)**
   - Built with clap for argument parsing
   - Commands: create, get, update, search, delete, watch
   - Search with pagination support (--page, --page-size flags)
   - Output formats: JSON (default) or text
   - Watch command outputs NDJSON (newline-delimited JSON) for real-time updates

6. **clipper (Tauri Desktop App)**
   - **Frontend**: React 19 + TypeScript + Vite
   - **Backend**: Tauri 2 with Rust
   - **Features**:
     - **Bundled server**: Includes clipper-server as a sidecar that starts automatically
     - **Server mode selection**: Choose between bundled server or external server
     - **Network access**: Option to listen on all interfaces for LAN access
     - System tray with show/hide and quit menu
     - Clipboard monitoring (text and images) with polling
     - WebSocket connection for real-time sync
     - Drag-and-drop file upload
     - Settings dialog with theme support (light/dark/auto)
     - Auto-launch on login (macOS, Linux, Windows)
     - Favorites tagging system
     - Infinite scroll clip list
     - Image preview popup
     - **Internationalization**: English and Chinese language support
     - **Toast notifications**: Configurable notification system
     - **Clear all data**: Option to wipe all clips and restart server
     - **Auto-reconnect**: Reconnects to server when URL changes in settings
   - **Key Modules** (in `clipper/src-tauri/src/`):
     - `lib.rs`: Tauri app setup, plugin initialization, event handlers
     - `state.rs`: AppState with ClipperClient
     - `commands.rs`: Tauri commands (list_clips, search_clips, create_clip, etc.)
     - `clipboard.rs`: Clipboard monitoring with text/image support
     - `websocket.rs`: WebSocket listener for real-time notifications
     - `settings.rs`: Settings persistence (JSON file in app config dir)
     - `tray.rs`: System tray setup
     - `autolaunch.rs`: Platform-specific auto-start configuration
     - `server.rs`: ServerManager for bundled server lifecycle

7. **clipper-slint (Slint GUI - Alternative)**
   - Built with Slint 1.14 UI framework
   - Uses Skia renderer with Winit backend
   - Simpler architecture than Tauri version
   - Connects to clipper-server via clipper-client

8. **Database Schema (SurrealDB)**
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
- **Tauri State**: Uses Tauri's managed state for AppState and SettingsManager
- **Clipboard Loop Prevention**: Last synced content tracked to prevent infinite clipboard-to-server loop

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

### Adding New Tauri Commands

1. Add function with `#[tauri::command]` attribute in `clipper/src-tauri/src/commands.rs`
2. Register in `invoke_handler` in `clipper/src-tauri/src/lib.rs`
3. Call from frontend using `invoke()` from `@tauri-apps/api/core`

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

### Tauri Events

The Tauri app emits events to the frontend:

```typescript
// Listen for new clips
import { listen } from "@tauri-apps/api/event";

await listen("new-clip", (event) => {
  console.log("New clip:", event.payload);
});

await listen("clip-updated", (event) => { /* ... */ });
await listen("clip-deleted", (event) => { /* ... */ });
await listen("clip-created", (event) => { /* ... */ }); // From clipboard monitor
await listen("open-settings", () => { /* ... */ }); // From tray menu
```

### Error Handling

- `clipper_indexer::IndexerError` - core library errors
- `clipper_server::ServerError` - server-specific errors (implements IntoResponse)
- `clipper_client::ClientError` - client-specific errors
- Server errors automatically converted to JSON responses with appropriate HTTP status codes
- CLI uses anyhow for error context
- Tauri commands return `Result<T, String>` for frontend error handling

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
- `CLIPPER_WEB_DIR` - Path to web UI dist directory (default: auto-detected `./web/dist`)
- `PORT` (default: `3000`)
- `RUST_LOG` for tracing (default: `clipper_server=debug,tower_http=debug`)

TLS environment variables (requires `tls` feature):
- `CLIPPER_TLS_ENABLED` - Enable HTTPS (default: `false`)
- `CLIPPER_TLS_PORT` - HTTPS port (default: `443`)
- `CLIPPER_TLS_CERT` - Path to TLS certificate file (PEM format)
- `CLIPPER_TLS_KEY` - Path to TLS private key file (PEM format)
- `CLIPPER_TLS_REDIRECT` - Redirect HTTP to HTTPS (default: `true`)
- `CLIPPER_TLS_RELOAD_INTERVAL` - Seconds between certificate reload checks (default: `0` = disabled)

ACME environment variables (requires `acme` feature):
- `CLIPPER_ACME_ENABLED` - Enable automatic certificate management (default: `false`)
- `CLIPPER_ACME_DOMAIN` - Domain name for the certificate
- `CLIPPER_ACME_EMAIL` - Contact email for Let's Encrypt notifications
- `CLIPPER_ACME_STAGING` - Use staging environment for testing (default: `false`)
- `CLIPPER_CERTS_DIR` - Directory for certificate cache (default: `~/.config/com.0d0a.clipper/certs/`)

### CLI Configuration

Environment variables:
- `CLIPPER_URL` - Server URL (default: `http://localhost:3000`)

### Tauri App Configuration

Settings stored in platform-specific config directory:
- macOS: `~/Library/Application Support/com.0d0a.clipper/settings.json`
- Linux: `~/.config/com.0d0a.clipper/settings.json`
- Windows: `%APPDATA%\com.0d0a.clipper\settings.json`

Settings include:
- `serverAddress`: Server URL (default: `http://localhost:3000`)
- `defaultSaveLocation`: Optional default save path
- `openOnStartup`: Show window on app start
- `startOnLogin`: Auto-launch on system login
- `theme`: "light" | "dark" | "auto"
- `useBundledServer`: Use bundled server (true) or external server (false)
- `listenOnAllInterfaces`: Allow LAN access to bundled server
- `serverPort`: Port for bundled server (persisted across restarts)
- `language`: UI language ("en", "zh", or null for auto)
- `notificationsEnabled`: Show toast notifications

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

### Tauri Commands

```typescript
// Available via invoke() from @tauri-apps/api/core
list_clips(filters: SearchFiltersInput, page: number, page_size: number): Promise<PagedResult>
search_clips(query: string, filters: SearchFiltersInput, page: number, page_size: number): Promise<PagedResult>
create_clip(content: string, tags: string[], additional_notes?: string): Promise<Clip>
update_clip(id: string, tags?: string[], additional_notes?: string): Promise<Clip>
delete_clip(id: string): Promise<void>
get_clip(id: string): Promise<Clip>
copy_to_clipboard(content: string): Promise<void>
upload_file(path: string, tags: string[], additional_notes?: string): Promise<Clip>
get_file_url(clip_id: string): string
download_file(clip_id: string, filename: string): Promise<string>
get_settings(): Settings
save_settings(settings: Settings): Promise<void>
browse_directory(): Promise<string | null>
check_auto_launch_status(): Promise<boolean>
get_server_url(): Promise<string>
is_bundled_server(): Promise<boolean>
switch_to_bundled_server(): Promise<string>
switch_to_external_server(server_url: string): Promise<void>
clear_all_data(): Promise<void>
toggle_listen_on_all_interfaces(listen_on_all: boolean): Promise<string>
get_local_ip_addresses(): Promise<string[]>
update_tray_language(language: string): Promise<void>
```

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
- **Tauri Desktop Application**:
  - React frontend with TypeScript
  - System tray integration
  - Clipboard monitoring (text and images)
  - WebSocket real-time sync
  - Settings persistence
  - Theme support (light/dark/auto)
  - Auto-launch on login
  - Drag-and-drop file upload
  - Infinite scroll clip list
  - Image preview
  - Favorites system
  - **Bundled server** (clipper-server as sidecar)
  - **Server mode selection** (bundled vs external)
  - **Network access toggle** (LAN access for bundled server)
  - **Internationalization** (English, Chinese)
  - **Toast notifications**
  - **Clear all data** functionality
  - **Auto-reconnect** on server URL change
- Slint GUI alternative (basic implementation)
- **Web UI** (clipper-server/web):
  - React + TypeScript + Vite frontend
  - Aligned look and feel with desktop app
  - View, search, edit, delete clips
  - Theme support (light/dark/auto)
  - Internationalization (English, Chinese)
  - Infinite scroll with pagination
  - Favorites and date filtering
  - Served directly from clipper-server

### Future Work
- File content preview/rendering improvements
- Advanced search operators
- Export/import functionality
- Clipboard monitoring daemon (standalone)
- Keyboard shortcuts
- Global hotkey support
