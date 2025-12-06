# Clipper

A modern, cross-platform clipboard manager with full-text search, real-time sync, and a beautiful desktop interface.

[![Homepage](https://img.shields.io/badge/homepage-clipper.unwritten.codes-blue)](https://clipper.unwritten.codes)
![Version](https://img.shields.io/badge/version-0.17.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

English | [简体中文](README.zh-CN.md)

## Features

- **Clipboard Monitoring** - Automatically captures text and images from your clipboard
- **Full-Text Search** - Find any clip instantly with powerful BM25-ranked search
- **Tags & Favorites** - Organize clips with tags and mark your favorites
- **File Attachments** - Store files alongside text clips
- **Real-time Sync** - WebSocket-based synchronization across devices
- **Bundled Server** - Zero-configuration setup with embedded server
- **Network Sharing** - Share clips across your local network
- **HTTPS/TLS Support** - Secure connections with manual certificates or automatic Let's Encrypt
- **Self-Signed Certificates** - Trust self-signed certificates with SSH-like fingerprint verification
- **Authentication** - Optional Bearer token authentication for API security
- **Auto-cleanup** - Automatic deletion of old clips based on retention policy
- **Clip Sharing** - Share clips publicly via short URLs with optional expiration
- **Web UI** - Browser-based access with drag-and-drop file upload
- **Multi-language** - English and Chinese interface
- **Theme Support** - Light, dark, and auto themes
- **Cross-platform** - Works on macOS, Windows, and Linux

## Quick Start

### Download

Download the latest release for your platform from the [Releases](https://github.com/windoze/clipper/releases) page.

> **Note:** macOS binaries are signed and notarized. Windows and Linux binaries are not code-signed. See [Platform Notes](#platform-notes) for platform-specific instructions.

### Build from Source

```bash
# Clone the repository
git clone https://github.com/windoze/clipper.git
cd clipper

# Build the desktop app
cd clipper
npm install
npm run tauri:build
```

## Architecture

Clipper is built as a modular Rust workspace with six main components:

```
clipper/
├── clipper-indexer/     # Core library - SurrealDB storage & full-text search
├── clipper-server/      # REST API + WebSocket server (Axum) with built-in Web UI
├── clipper-client/      # Rust client library
├── clipper-cli/         # Command-line interface
├── clipper/             # Desktop app (Tauri 2 + React + TypeScript)
├── clipper-slint/       # Alternative GUI (Slint UI, WIP)
└── packages/clipper-ui/ # Shared React UI components
```

### Technology Stack

| Component | Technology |
|-----------|------------|
| Core Storage | SurrealDB with RocksDB backend |
| Full-Text Search | SurrealDB FTS with BM25 ranking |
| File Storage | object_store (LocalFileSystem) |
| Server | Axum with Tower middleware |
| Desktop Frontend | React 19 + TypeScript + Vite |
| Desktop Backend | Tauri 2 |
| CLI | clap |

## Desktop Application

The desktop app provides a full-featured clipboard manager with a modern interface.

### Features

- **System Tray** - Runs in background with quick access
- **Clipboard Monitor** - Captures text and images automatically
- **Infinite Scroll** - Smooth browsing through large collections
- **Image Preview** - Click to preview image clips
- **Drag & Drop** - Drop files directly into the app
- **Auto-launch** - Start on system login

### Settings

| Setting | Description |
|---------|-------------|
| Server Mode | Bundled (automatic) or External server |
| Network Access | Allow LAN access for multi-device sync |
| Bundled Server Token | Authentication token for bundled server (shown when network access enabled) |
| External Server Token | Authentication token for connecting to external server |
| Theme | Light, Dark, or Auto (follows system) |
| Language | English or Chinese |
| Notifications | Toast notifications on/off |
| Auto-launch | Start on system login |

## Server

The server can run standalone or bundled with the desktop app.

### Standalone Server

```bash
# Run with defaults
cargo run --bin clipper-server

# With custom configuration
cargo run --bin clipper-server -- \
  --db-path ./data/db \
  --storage-path ./data/storage \
  --port 3000
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_DB_PATH` | `./data/db` | Database directory |
| `CLIPPER_STORAGE_PATH` | `./data/storage` | File storage directory |
| `CLIPPER_LISTEN_ADDR` | `0.0.0.0` | Server bind address |
| `PORT` | `3000` | Server port |
| `CLIPPER_CLEANUP_ENABLED` | `false` | Enable automatic cleanup |
| `CLIPPER_CLEANUP_RETENTION_DAYS` | `30` | Days to retain clips |
| `CLIPPER_CLEANUP_INTERVAL_HOURS` | `24` | Hours between cleanups |
| `CLIPPER_BEARER_TOKEN` | - | Bearer token for authentication |
| `CLIPPER_SHORT_URL_BASE` | - | Base URL for sharing (enables sharing) |
| `CLIPPER_SHORT_URL_EXPIRATION_HOURS` | `24` | Default short URL expiration |

### Authentication

Enable authentication by setting a bearer token:

```bash
# Set a bearer token to require authentication
cargo run --bin clipper-server -- --bearer-token your-secret-token

# Or via environment variable
CLIPPER_BEARER_TOKEN=your-secret-token cargo run --bin clipper-server
```

When authentication is enabled, all API requests must include the token:

```bash
curl -H "Authorization: Bearer your-secret-token" http://localhost:3000/clips
```

### TLS/HTTPS Configuration

For secure connections, build with TLS features:

```bash
# Manual certificates
cargo build -p clipper-server --features tls

# Automatic Let's Encrypt certificates
cargo build -p clipper-server --features acme

# Full TLS support with secure storage
cargo build -p clipper-server --features full-tls
```

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_TLS_ENABLED` | `false` | Enable HTTPS |
| `CLIPPER_TLS_PORT` | `443` | HTTPS port |
| `CLIPPER_TLS_CERT` | - | Path to certificate (PEM) |
| `CLIPPER_TLS_KEY` | - | Path to private key (PEM) |
| `CLIPPER_ACME_ENABLED` | `false` | Enable Let's Encrypt |
| `CLIPPER_ACME_DOMAIN` | - | Domain for certificate |
| `CLIPPER_ACME_EMAIL` | - | Contact email |

### Docker Deployment

```bash
# Build the image
docker build -t clipper-server .

# Run container
docker run -d -p 3000:3000 -v clipper-data:/data clipper-server

# Access at http://localhost:3000
```

### REST API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/clips` | GET | List clips (paginated) |
| `/clips` | POST | Create text clip |
| `/clips/upload` | POST | Upload file clip |
| `/clips/search` | GET | Search clips (paginated) |
| `/clips/:id` | GET | Get clip by ID |
| `/clips/:id` | PUT | Update clip metadata |
| `/clips/:id` | DELETE | Delete clip |
| `/clips/:id/file` | GET | Download file attachment |
| `/clips/:id/short-url` | POST | Create short URL for sharing |
| `/s/:code` | GET | Resolve short URL (public) |
| `/export` | GET | Export all clips as tar.gz archive |
| `/import` | POST | Import clips from tar.gz archive |
| `/ws` | WS | Real-time notifications |

## CLI

The command-line interface provides full access to all features.

```bash
# Create a clip
clipper-cli create "Hello, World!" --tags greeting,example

# Search clips
clipper-cli search "hello" --page 1 --page-size 20

# Watch for real-time updates
clipper-cli watch

# Get a specific clip
clipper-cli get <clip-id>

# Update clip metadata
clipper-cli update <clip-id> --tags updated,important

# Delete a clip
clipper-cli delete <clip-id>

# Share a clip (requires CLIPPER_SHORT_URL_BASE on server)
clipper-cli share <clip-id> --expires 48

# Export all clips to archive
clipper-cli export -o backup.tar.gz

# Import clips from archive
clipper-cli import backup.tar.gz
```

### Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_URL` | `http://localhost:3000` | Server URL |
| `CLIPPER_TOKEN` | - | Bearer token for authentication |

### Authentication

When connecting to a server with authentication enabled:

```bash
# Using command-line option
clipper-cli --token your-secret-token search "hello"

# Using environment variable
CLIPPER_TOKEN=your-secret-token clipper-cli search "hello"
```

## Client Library

Use the Rust client library to integrate Clipper into your applications.

```rust
use clipper_client::{ClipperClient, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client (optionally with authentication token)
    let client = ClipperClient::new("http://localhost:3000")
        .with_token("your-secret-token".to_string()); // Optional

    // Create a clip
    let clip = client
        .create_clip(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            None,
        )
        .await?;

    // Search with pagination
    let result = client
        .search_clips("Hello", SearchFilters::new(), 1, 20)
        .await?;

    println!("Found {} clips", result.total);

    // Subscribe to real-time updates
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    client.subscribe_notifications(tx).await?;

    while let Some(notification) = rx.recv().await {
        println!("Update: {:?}", notification);
    }

    Ok(())
}
```

## Development

### Prerequisites

- Rust 1.91+
- Node.js 18+
- Platform-specific dependencies for Tauri ([see docs](https://tauri.app/start/prerequisites/))

### Building

```bash
# Build entire workspace
cargo build --workspace

# Build specific package
cargo build -p clipper-indexer
cargo build -p clipper-server
cargo build -p clipper-client
cargo build -p clipper-cli

# Build desktop app
cd clipper && npm install && npm run tauri:build

# Release build
cargo build --workspace --release
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run server tests (sequential)
cargo test -p clipper-server -- --test-threads=1

# Run client tests (sequential)
cargo test -p clipper-client -- --test-threads=1
```

## Project Structure

```
clipper/
├── CLAUDE.md              # Development guide for AI assistants
├── Cargo.toml             # Workspace configuration
├── LICENSE                # MIT License
├── README.md              # This file
├── clipper/               # Tauri desktop application
│   ├── src/               # React frontend
│   ├── src-tauri/         # Tauri backend (Rust)
│   └── package.json
├── clipper-indexer/       # Core indexing library
│   ├── src/
│   └── README.md
├── clipper-server/        # REST API server
│   ├── src/
│   └── README.md
├── clipper-client/        # Rust client library
│   ├── src/
│   └── README.md
├── clipper-cli/           # Command-line interface
│   ├── src/
│   └── README.md
└── clipper-slint/         # Alternative Slint GUI
    └── src/
```

## Platform Notes

### macOS

macOS binaries are **signed and notarized** by Apple. On first launch, macOS may show a dialog saying the app is from an identified developer - click **"Open"** to proceed.

Available formats:
- **DMG** - Disk image with drag-to-install interface
- **app.zip** - Compressed app bundle for manual installation

### Windows

Windows SmartScreen may show a warning that the app is from an "unknown publisher."

**Workaround:**

1. When the SmartScreen popup appears, click **"More info"**
2. Click **"Run anyway"**

Alternatively, you can right-click the executable, select **Properties**, and check **"Unblock"** at the bottom of the General tab.

### Linux

Linux generally doesn't have the same signing restrictions, but you may need to make the AppImage executable:

```bash
chmod +x Clipper.AppImage
./Clipper.AppImage
```

If you encounter permission issues, you can also run:

```bash
# For AppImage
chmod +x Clipper*.AppImage

# For .deb package
sudo dpkg -i clipper_*.deb

# For .rpm package
sudo rpm -i clipper-*.rpm
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [SurrealDB](https://surrealdb.com/) - Multi-model database
- [Tauri](https://tauri.app/) - Desktop app framework
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [React](https://react.dev/) - UI library
- [Nerd Fonts](https://www.nerdfonts.com/) - Symbols Nerd Font Mono for icon/powerline glyph support (MIT License)
