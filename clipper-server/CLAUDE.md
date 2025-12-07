# clipper-server

REST API server with WebSocket support for real-time clip updates, includes built-in web UI.

## Build & Test

```bash
# Build
cargo build -p clipper-server

# Build with embedded web UI (for Docker/deployment)
cargo build -p clipper-server --release --features embed-web

# Build with TLS support (manual certificates)
cargo build -p clipper-server --features tls

# Build with ACME (Let's Encrypt automatic certificates)
cargo build -p clipper-server --features acme

# Build with full TLS + ACME + secure storage (OS keychain)
cargo build -p clipper-server --features full-tls

# Test (must run sequentially to avoid database conflicts)
cargo test -p clipper-server -- --test-threads=1
cargo test --test api_tests -p clipper-server -- --test-threads=1

# Run single test
cargo test test_create_clip -p clipper-server
```

## Running

```bash
# Run (requires CLIPPER_DB_PATH and CLIPPER_STORAGE_PATH env vars or uses defaults)
cargo run --bin clipper-server

# With custom paths
CLIPPER_DB_PATH=./data/db CLIPPER_STORAGE_PATH=./data/storage cargo run --bin clipper-server

# With configuration file
cargo run --bin clipper-server -- --config config.toml
```

## Docker

```bash
# Build Docker image (includes embedded web UI)
docker build -t clipper-server .

# Run container
docker run -d -p 3000:3000 -v clipper-data:/data clipper-server

# Access at http://localhost:3000
```

## Architecture

- Built with Axum framework
- `AppState` wraps `Arc<ClipperIndexer>` and broadcast channel for WebSocket updates
- REST endpoints in `api.rs`: CRUD operations, search with pagination, file upload
- WebSocket in `websocket.rs`: real-time clip updates
- All state mutations trigger WebSocket notifications
- **Configuration**: Multi-source configuration (CLI args, env vars, TOML files)
- **Built-in Web UI**: Serves static files from `web/dist/` directory
- **Web UI features**: View, search, edit, delete clips with i18n support (English/Chinese)

## Environment Configuration

Multiple configuration sources (in priority order):
1. Command line arguments
2. Environment variables
3. Configuration file (TOML)
4. Default values

### Basic Environment Variables

- `CLIPPER_CONFIG` - Path to configuration file
- `CLIPPER_DB_PATH` (default: `./data/db`)
- `CLIPPER_STORAGE_PATH` (default: `./data/storage`)
- `CLIPPER_LISTEN_ADDR` (default: `0.0.0.0`)
- `CLIPPER_WEB_DIR` - Path to web UI dist directory (default: auto-detected `./web/dist`)
- `PORT` (default: `3000`)
- `RUST_LOG` for tracing (default: `clipper_server=debug,tower_http=debug`)

### TLS Environment Variables (requires `tls` feature)

- `CLIPPER_TLS_ENABLED` - Enable HTTPS (default: `false`)
- `CLIPPER_TLS_PORT` - HTTPS port (default: `443`)
- `CLIPPER_TLS_CERT` - Path to TLS certificate file (PEM format)
- `CLIPPER_TLS_KEY` - Path to TLS private key file (PEM format)
- `CLIPPER_TLS_REDIRECT` - Redirect HTTP to HTTPS (default: `true`)
- `CLIPPER_TLS_RELOAD_INTERVAL` - Seconds between certificate reload checks (default: `0` = disabled)

#### Setting Up Self-Signed Certificates

For development or private networks where you don't have a domain name, you can use self-signed certificates:

1. **Generate a self-signed certificate using OpenSSL:**

```bash
# Create a directory for certificates
mkdir -p certs

# Generate a private key and self-signed certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem -days 365 -nodes \
  -subj "/CN=clipper-server" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:192.168.1.100"
```

Replace `192.168.1.100` with your server's actual IP address. You can add multiple IPs or DNS names.

2. **Run the server with TLS:**

```bash
CLIPPER_TLS_ENABLED=true \
CLIPPER_TLS_PORT=3443 \
CLIPPER_TLS_CERT=./certs/cert.pem \
CLIPPER_TLS_KEY=./certs/key.pem \
cargo run --bin clipper-server --features tls
```

3. **Connect from the Clipper desktop app:**

When connecting to a server with a self-signed certificate, the Clipper app will show a confirmation dialog displaying the certificate's SHA-256 fingerprint. If you trust the server:
- Verify the fingerprint matches (you can check it with: `openssl x509 -in certs/cert.pem -noout -fingerprint -sha256`)
- Click "Trust Certificate" to save the fingerprint
- Future connections to this server will automatically trust this certificate

**Note:** The certificate fingerprint is stored per-host in the app's settings. If you regenerate the certificate, you'll need to trust the new fingerprint.

### ACME Environment Variables (requires `acme` feature)

- `CLIPPER_ACME_ENABLED` - Enable automatic certificate management (default: `false`)
- `CLIPPER_ACME_DOMAIN` - Domain name for the certificate
- `CLIPPER_ACME_EMAIL` - Contact email for Let's Encrypt notifications
- `CLIPPER_ACME_STAGING` - Use staging environment for testing (default: `false`)
- `CLIPPER_CERTS_DIR` - Directory for certificate cache (default: `~/.config/com.0d0a.clipper/certs/`)

**Note:** When ACME is enabled, `CLIPPER_BEARER_TOKEN` is required for security. Since ACME exposes the server to the internet with a public domain, authentication is mandatory to prevent unauthorized access.

### Auto-cleanup Environment Variables

- `CLIPPER_CLEANUP_ENABLED` - Enable automatic cleanup of old clips (default: `false`)
- `CLIPPER_CLEANUP_RETENTION_DAYS` - Delete clips older than this many days (default: `30`)
- `CLIPPER_CLEANUP_INTERVAL_HOURS` - Interval in hours between cleanup runs (default: `24`)

### Authentication Environment Variables

- `CLIPPER_BEARER_TOKEN` - Bearer token for authentication (if set, all requests require `Authorization: Bearer <token>` header)

### Short URL / Sharing Environment Variables

- `CLIPPER_SHORT_URL_BASE` - Base URL for short URLs (e.g., `https://clip.example.com`). If not set, sharing is disabled.
- `CLIPPER_SHORT_URL_EXPIRATION_HOURS` - Default expiration time for short URLs in hours (default: `24`, `0` = no expiration)

## REST API Endpoints

- `GET /health` - Health check
- `GET /version` - Server version and status (version, uptime, active connections, config)
- `POST /clips` - Create clip from text
- `POST /clips/upload` - Upload file as clip
- `GET /clips` - List clips with pagination (query params: start_date, end_date, tags, page, page_size)
- `GET /clips/search` - Search clips with pagination (query params: q, start_date, end_date, tags, page, page_size)
- `GET /clips/:id` - Get clip by ID
- `PUT /clips/:id` - Update clip metadata
- `DELETE /clips/:id` - Delete clip
- `GET /clips/:id/file` - Download file attachment
- `POST /clips/:id/short-url` - Create a short URL for sharing a clip (requires `CLIPPER_SHORT_URL_BASE`)
- `GET /s/:code` - Public endpoint to resolve short URL (returns HTML page, JSON, or file based on Accept header)
- `GET /shared-assets/:filename` - Static assets for shared clip page (CSS/JS)

## WebSocket Endpoint

- `WS /ws` - Real-time clip notifications

### WebSocket Notifications

Server broadcasts four types of notifications:

```rust
// NewClip: { type: "new_clip", id, content, tags }
// UpdatedClip: { type: "updated_clip", id }
// DeletedClip: { type: "deleted_clip", id }
// ClipsCleanedUp: { type: "clips_cleaned_up", ids, count }
```

## Adding New API Endpoints

1. Add handler function in `src/api.rs`
2. Register route in `pub fn routes()`
3. If it modifies clips, call `state.notify_*()` for WebSocket updates
4. Add test in `tests/api_tests.rs`
5. Add client method in `clipper-client/src/lib.rs`
6. Add test in `clipper-client/tests/integration_tests.rs`
7. Add CLI command in `clipper-cli/src/main.rs` if user-facing

## Error Handling

- `clipper_server::ServerError` - server-specific errors (implements IntoResponse)
- Server errors automatically converted to JSON responses with appropriate HTTP status codes

## Testing Notes

- Server tests must run sequentially: `-- --test-threads=1`
- Each test creates isolated temporary database
- Server tests use raw HTTP requests via tower::ServiceExt
- Multipart file upload tests construct raw HTTP multipart bodies
- **Test coverage**: 18 tests
