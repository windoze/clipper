# Clipper Server

A REST API server with WebSocket support for managing clipboard entries using the `clipper_indexer` library.

## Features

- **REST API** for CRUD operations on clipboard entries
- **Full-text search** with filters (tags, date ranges) and pagination
- **WebSocket support** for real-time updates
- **File attachment support** for clipboard entries
- **Metadata management** (tags and additional notes)
- **Multi-source configuration** (CLI args, environment variables, config files)
- **Graceful shutdown** handling
- **Built-in Web UI** with drag-and-drop file upload
- **TLS/HTTPS support** with manual or automatic (Let's Encrypt) certificates
- **Certificate hot-reload** for zero-downtime certificate updates
- **Automatic cleanup** with configurable retention policy

## Getting Started

### Configuration

The server can be configured through multiple sources (in order of priority):

1. **Command line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file** (TOML)
4. **Default values** (lowest priority)

#### Command Line Arguments

```bash
clipper-server [OPTIONS]

Options:
  -c, --config <FILE>              Path to configuration file
      --db-path <PATH>             Database path
      --storage-path <PATH>        Storage path for file attachments
      --listen-addr <ADDR>         Server listen address (default: 0.0.0.0)
  -p, --port <PORT>                Server listen port (default: 3000)
      --bearer-token <TOKEN>       Bearer token for authentication
      --cleanup-enabled            Enable automatic cleanup of old clips
      --cleanup-retention-days <DAYS>   Retention period in days (default: 30)
      --cleanup-interval-hours <HOURS>  Cleanup interval in hours (default: 24)
  -h, --help                       Print help
```

#### Environment Variables

- `CLIPPER_CONFIG` - Path to configuration file
- `CLIPPER_DB_PATH` - Path to the database directory (default: `./data/db`)
- `CLIPPER_STORAGE_PATH` - Path to the file storage directory (default: `./data/storage`)
- `CLIPPER_LISTEN_ADDR` - Server listen address (default: `0.0.0.0`)
- `PORT` - Server port (default: `3000`)
- `RUST_LOG` - Logging level (default: `clipper_server=debug,tower_http=debug`)
- `CLIPPER_CLEANUP_ENABLED` - Enable automatic cleanup (default: `false`)
- `CLIPPER_CLEANUP_RETENTION_DAYS` - Retention period in days (default: `30`)
- `CLIPPER_CLEANUP_INTERVAL_HOURS` - Cleanup interval in hours (default: `24`)
- `CLIPPER_BEARER_TOKEN` - Bearer token for authentication (if set, all requests require auth)

#### Configuration File

Create a `config.toml` or `clipper-server.toml` file:

```toml
[database]
path = "./data/db"

[storage]
path = "./data/storage"

[server]
listen_addr = "0.0.0.0"
port = 3000

[cleanup]
enabled = false
retention_days = 30
interval_hours = 24

[auth]
# bearer_token = "your-secret-token"
```

Or specify a custom config file location:

```bash
clipper-server --config /path/to/config.toml
```

See `config.toml.example` for a complete example.

### Authentication

Enable Bearer token authentication to protect the API:

```bash
# Via command line
clipper-server --bearer-token your-secret-token

# Via environment variable
CLIPPER_BEARER_TOKEN=your-secret-token clipper-server

# Via config file (see config.toml.example)
```

When authentication is enabled:
- All REST API endpoints (except `/health`) require the `Authorization: Bearer <token>` header
- File downloads also support `?token=<token>` query parameter
- WebSocket connections use message-based authentication (client sends auth message after connecting)
- The Web UI will show a login screen when authentication is required

Example authenticated request:
```bash
curl -H "Authorization: Bearer your-secret-token" http://localhost:3000/clips
```

### TLS/HTTPS Configuration

Build with TLS features for HTTPS support:

```bash
# Manual certificates
cargo build -p clipper-server --features tls

# Automatic Let's Encrypt certificates
cargo build -p clipper-server --features acme

# Full TLS with secure credential storage
cargo build -p clipper-server --features full-tls
```

#### TLS Environment Variables (requires `tls` feature)

- `CLIPPER_TLS_ENABLED` - Enable HTTPS (default: `false`)
- `CLIPPER_TLS_PORT` - HTTPS port (default: `443`)
- `CLIPPER_TLS_CERT` - Path to TLS certificate file (PEM format)
- `CLIPPER_TLS_KEY` - Path to TLS private key file (PEM format)
- `CLIPPER_TLS_REDIRECT` - Redirect HTTP to HTTPS (default: `true`)
- `CLIPPER_TLS_RELOAD_INTERVAL` - Seconds between certificate reload checks (default: `0` = disabled)

#### ACME Environment Variables (requires `acme` feature)

- `CLIPPER_ACME_ENABLED` - Enable automatic certificate management (default: `false`)
- `CLIPPER_ACME_DOMAIN` - Domain name for the certificate
- `CLIPPER_ACME_EMAIL` - Contact email for Let's Encrypt notifications
- `CLIPPER_ACME_STAGING` - Use staging environment for testing (default: `false`)
- `CLIPPER_CERTS_DIR` - Directory for certificate cache (default: `~/.config/com.0d0a.clipper/certs/`)

#### Example: HTTPS with Let's Encrypt

```bash
CLIPPER_ACME_ENABLED=true \
CLIPPER_ACME_DOMAIN=clips.example.com \
CLIPPER_ACME_EMAIL=admin@example.com \
cargo run --bin clipper-server --features acme
```

#### Using Self-Signed Certificates

For development or internal deployments, you can use self-signed certificates:

1. **Generate a self-signed certificate**:

```bash
# Generate a private key and self-signed certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

2. **Start the server with the self-signed certificate**:

```bash
CLIPPER_TLS_ENABLED=true \
CLIPPER_TLS_CERT=./cert.pem \
CLIPPER_TLS_KEY=./key.pem \
cargo run --bin clipper-server --features tls
```

3. **Connect from CLI or desktop app**:

When connecting to a server with a self-signed certificate, both the CLI and desktop app use SSH-like fingerprint verification:

- **First connection**: The certificate's SHA-256 fingerprint is displayed for verification
- **Trust decision**: You can choose to trust the certificate permanently or not
- **Fingerprint storage**: Trusted fingerprints are stored in `~/.config/com.0d0a.clipper/settings.json`
- **Security warning**: If the fingerprint changes (potential MITM attack), you'll see a warning similar to SSH's "REMOTE HOST IDENTIFICATION HAS CHANGED"

Example CLI interaction:
```
$ clipper-cli --url https://clips.example.com:3000 list
The authenticity of host 'clips.example.com' can't be established.
The server's certificate is not signed by a trusted Certificate Authority (CA).
This could mean:
  - The server is using a self-signed certificate
  - The server's CA is not in your system's trust store
  - Someone may be intercepting your connection (man-in-the-middle attack)

Certificate SHA256 fingerprint:
  a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2

Full fingerprint (verify with server administrator):
  A1:B2:C3:D4:E5:F6:A7:B8
  C9:D0:E1:F2:A3:B4:C5:D6
  E7:F8:A9:B0:C1:D2:E3:F4
  A5:B6:C7:D8:E9:F0:A1:B2

Are you sure you want to trust this certificate and continue connecting (yes/no)?
```

The CLI and desktop app share the same trusted certificates store, so a certificate trusted in one will be automatically trusted in the other.

### Running the Server

Basic usage:
```bash
cargo run --bin clipper-server
```

With custom port:
```bash
cargo run --bin clipper-server -- --port 8080
```

With custom configuration:
```bash
cargo run --bin clipper-server -- --config config.toml
```

With environment variables:
```bash
CLIPPER_DB_PATH=/var/lib/clipper/db PORT=8080 cargo run --bin clipper-server
```

The server will start on `http://0.0.0.0:3000` by default (configurable).

## Web UI

The server includes a built-in web UI accessible at the root URL (e.g., `http://localhost:3000/`).

### Web UI Features

- View, search, edit, and delete clips
- Drag-and-drop file upload
- Send clipboard content button (for manual clipboard sync)
- Real-time updates via WebSocket (HTTPS only)
- WebSocket connection status indicator (connected/disconnected/HTTPS required)
- Auto-refresh clip list on WebSocket notifications
- Theme support (light/dark/auto)
- Internationalization (English/Chinese)
- Favorites and date filtering
- Infinite scroll with pagination
- Visual fade-out for clips approaching auto-cleanup date (when cleanup is enabled)

### Building with Embedded Web UI

For Docker deployments, build with the embedded web UI:

```bash
cd clipper-server/web && npm install && npm run build
cargo build -p clipper-server --release --features embed-web
```

## REST API Endpoints

### Health Check

```
GET /health
```

Returns `OK` if the server is running.

### Version and Status

```
GET /version
```

Returns server version and status information.

**Response**: `200 OK`
```json
{
  "version": "0.10.0",
  "uptime_secs": 3600,
  "active_ws_connections": 5,
  "config": {
    "port": 3000,
    "tls_enabled": false,
    "acme_enabled": false,
    "cleanup_enabled": true,
    "cleanup_retention_days": 30
  }
}
```

### Create a Clip

```
POST /clips
Content-Type: application/json

{
  "content": "Text content to store",
  "tags": ["tag1", "tag2"],
  "additional_notes": "Optional notes"
}
```

**Response**: `201 Created`
```json
{
  "id": "abc123",
  "content": "Text content to store",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["tag1", "tag2"],
  "additional_notes": "Optional notes"
}
```

### Upload a File

```
POST /clips/upload
Content-Type: multipart/form-data
```

Form fields:
- `file` - The file to upload (required)
- `tags` - Comma-separated list of tags (optional)
- `additional_notes` - Additional notes about the file (optional)

**Response**: `201 Created`
```json
{
  "id": "abc123",
  "content": "File content (text) or 'Binary file: filename'",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["tag1", "tag2"],
  "additional_notes": "Optional notes",
  "file_attachment": "stored_file_key"
}
```

### List Clips

```
GET /clips?start_date=<RFC3339>&end_date=<RFC3339>&tags=<comma-separated>&page=<number>&page_size=<number>
```

Query parameters (all optional):
- `start_date` - Filter clips created after this date (RFC3339 format)
- `end_date` - Filter clips created before this date (RFC3339 format)
- `tags` - Comma-separated list of tags to filter by
- `page` - Page number (default: 1)
- `page_size` - Number of items per page (default: 20)

**Response**: `200 OK`
```json
{
  "items": [
    {
      "id": "abc123",
      "content": "Text content",
      "created_at": "2025-11-26T10:00:00Z",
      "tags": ["tag1", "tag2"],
      "additional_notes": "Optional notes"
    }
  ],
  "total": 100,
  "page": 1,
  "page_size": 20,
  "total_pages": 5
}
```

### Search Clips

```
GET /clips/search?q=<query>&start_date=<RFC3339>&end_date=<RFC3339>&tags=<comma-separated>&page=<number>&page_size=<number>
```

Query parameters:
- `q` - Search query (required)
- `start_date` - Filter clips created after this date (RFC3339 format, optional)
- `end_date` - Filter clips created before this date (RFC3339 format, optional)
- `tags` - Comma-separated list of tags to filter by (optional)
- `page` - Page number (default: 1, optional)
- `page_size` - Number of items per page (default: 20, optional)

**Response**: `200 OK` (same paginated format as list clips)

### Get a Clip

```
GET /clips/:id
```

**Response**: `200 OK`
```json
{
  "id": "abc123",
  "content": "Text content",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["tag1", "tag2"],
  "additional_notes": "Optional notes",
  "file_attachment": "optional_file_key"
}
```

### Update a Clip

```
PUT /clips/:id
Content-Type: application/json

{
  "tags": ["new_tag1", "new_tag2"],
  "additional_notes": "Updated notes"
}
```

Both fields are optional. Omit a field to leave it unchanged.

**Response**: `200 OK` (same format as get clip)

### Delete a Clip

```
DELETE /clips/:id
```

**Response**: `204 No Content`

### Get Clip File Attachment

```
GET /clips/:id/file
```

Returns the file content if the clip has a file attachment.

**Response**: `200 OK` with file content as binary data

## WebSocket API

Connect to the WebSocket endpoint to receive real-time updates:

```
ws://localhost:3000/ws
```

### Message Format

The server sends JSON messages for clip updates:

#### New Clip
```json
{
  "type": "new_clip",
  "id": "abc123",
  "content": "Text content",
  "tags": ["tag1", "tag2"]
}
```

#### Updated Clip
```json
{
  "type": "updated_clip",
  "id": "abc123"
}
```

#### Deleted Clip
```json
{
  "type": "deleted_clip",
  "id": "abc123"
}
```

#### Clips Cleaned Up
```json
{
  "type": "clips_cleaned_up",
  "ids": ["abc123", "def456"],
  "count": 2
}
```

### Client Messages

Clients can send:
- **Ping messages** - Server responds with pong to keep connection alive
- **Authentication message** - Required when server has authentication enabled:
  ```json
  {"type": "auth", "token": "your-secret-token"}
  ```
  Server responds with:
  ```json
  {"type": "auth_response", "success": true}
  ```
  or
  ```json
  {"type": "auth_response", "success": false, "error": "Invalid token"}
  ```
- **Text messages** - Logged by the server (reserved for future features)

## Example Usage

### Using curl

Create a clip:
```bash
curl -X POST http://localhost:3000/clips \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello, world!", "tags": ["greeting"]}'
```

List clips with pagination:
```bash
curl "http://localhost:3000/clips?page=1&page_size=10"
```

Search clips with pagination:
```bash
curl "http://localhost:3000/clips/search?q=hello&tags=greeting&page=1&page_size=20"
```

Upload a file:
```bash
curl -X POST http://localhost:3000/clips/upload \
  -F "file=@/path/to/your/file.txt" \
  -F "tags=document,important" \
  -F "additional_notes=This is a test file"
```

With authentication:
```bash
# All requests with authentication header
curl -H "Authorization: Bearer your-secret-token" \
  http://localhost:3000/clips

# File download with query parameter
curl "http://localhost:3000/clips/abc123/file?token=your-secret-token" -o file.txt
```

### Using WebSocket (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Received update:', update);
  
  if (update.type === 'new_clip') {
    console.log('New clip created:', update.id);
  } else if (update.type === 'updated_clip') {
    console.log('Clip updated:', update.id);
  } else if (update.type === 'deleted_clip') {
    console.log('Clip deleted:', update.id);
  }
};

ws.onopen = () => {
  console.log('Connected to clipper server');
};
```

### Using the Rust Client

```rust
use clipper_client::{ClipperClient, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with optional authentication
    let client = ClipperClient::new("http://localhost:3000")
        .with_token("your-secret-token".to_string()); // Optional

    // Create a clip
    let clip = client.create_clip(
        "Hello, World!".to_string(),
        vec!["greeting".to_string()],
        None,
    ).await?;
    
    // Search with pagination
    let result = client.search_clips(
        "Hello",
        SearchFilters::new(),
        1,  // page
        20, // page_size
    ).await?;
    
    println!("Found {} clips on page {} of {}", 
             result.items.len(), result.page, result.total_pages);
    
    Ok(())
}
```

## Architecture

- **axum** - Web framework for REST API and WebSocket
- **tokio** - Async runtime for non-blocking I/O
- **tower-http** - CORS and tracing middleware
- **clipper_indexer** - Backend storage and search engine
- **broadcast channel** - WebSocket pub/sub for real-time updates
- **clap + config** - Multi-source configuration management

## Error Handling

All errors are returned as JSON:

```json
{
  "error": "Error message description"
}
```

HTTP status codes:
- `400 Bad Request` - Invalid input (malformed JSON, missing required fields)
- `404 Not Found` - Resource not found (clip ID doesn't exist)
- `500 Internal Server Error` - Server error (database issues, storage errors)

## Testing

Run the comprehensive test suite:

```bash
# Run all server tests
cargo test -p clipper-server

# Run integration tests (must be sequential)
cargo test --test api_tests -p clipper-server -- --test-threads=1
```

Tests cover:
- Creating clips with and without optional fields
- Uploading files (text and binary)
- Listing clips with filters and pagination
- Searching clips with full-text queries and pagination
- Getting clips by ID
- Updating clip metadata
- Deleting clips
- File attachment retrieval
- WebSocket notifications for all operations

**18 tests total - all passing ✓**

## Deployment

### Production Considerations

1. **Database Path**: Use persistent storage for production:
   ```bash
   CLIPPER_DB_PATH=/var/lib/clipper/db \
   CLIPPER_STORAGE_PATH=/var/lib/clipper/storage \
   cargo run --release --bin clipper-server
   ```

2. **Logging**: Configure appropriate log levels:
   ```bash
   RUST_LOG=clipper_server=info,tower_http=info cargo run --release --bin clipper-server
   ```

3. **Port Binding**: For production, consider using a reverse proxy (nginx, caddy) in front of the server

4. **CORS**: The server uses permissive CORS for development. Configure appropriately for production.

5. **Graceful Shutdown**: The server handles SIGTERM and SIGINT signals for clean shutdowns.

### Docker Deployment

The project includes a production-ready multi-stage Dockerfile that builds clipper-server with the embedded Web UI and full TLS support.

#### Quick Start with Docker

```bash
# Build the image from the project root
docker build -t clipper-server .

# Run with HTTP only
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -v clipper-data:/data \
  clipper-server

# Access at http://localhost:3000
```

#### Docker with HTTPS (Manual Certificates)

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -p 443:443 \
  -v clipper-data:/data \
  -v /path/to/certs:/certs:ro \
  -e CLIPPER_TLS_ENABLED=true \
  -e CLIPPER_TLS_CERT=/certs/cert.pem \
  -e CLIPPER_TLS_KEY=/certs/key.pem \
  clipper-server
```

#### Docker with HTTPS (Let's Encrypt)

```bash
docker run -d \
  --name clipper \
  -p 80:3000 \
  -p 443:443 \
  -v clipper-data:/data \
  -e CLIPPER_ACME_ENABLED=true \
  -e CLIPPER_ACME_DOMAIN=clips.example.com \
  -e CLIPPER_ACME_EMAIL=admin@example.com \
  clipper-server
```

#### Docker Compose

```yaml
version: "3.8"
services:
  clipper:
    build: .
    ports:
      - "3000:3000"
      - "443:443"
    volumes:
      - clipper-data:/data
      - ./certs:/certs:ro  # Optional: for manual TLS
    environment:
      - RUST_LOG=clipper_server=info
      # Authentication (recommended for public deployments):
      # - CLIPPER_BEARER_TOKEN=your-secret-token
      # TLS with manual certificates:
      # - CLIPPER_TLS_ENABLED=true
      # - CLIPPER_TLS_CERT=/certs/cert.pem
      # - CLIPPER_TLS_KEY=/certs/key.pem
      # Or with Let's Encrypt:
      # - CLIPPER_ACME_ENABLED=true
      # - CLIPPER_ACME_DOMAIN=clips.example.com
      # - CLIPPER_ACME_EMAIL=admin@example.com
    restart: unless-stopped

volumes:
  clipper-data:
```

#### Docker Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_DB_PATH` | `/data/db` | Database directory |
| `CLIPPER_STORAGE_PATH` | `/data/storage` | File storage directory |
| `CLIPPER_LISTEN_ADDR` | `0.0.0.0` | Listen address |
| `PORT` | `3000` | HTTP port |
| `RUST_LOG` | `clipper_server=info` | Log level |
| `CLIPPER_TLS_ENABLED` | `false` | Enable HTTPS |
| `CLIPPER_TLS_PORT` | `443` | HTTPS port |
| `CLIPPER_TLS_CERT` | `/certs/cert.pem` | TLS certificate path |
| `CLIPPER_TLS_KEY` | `/certs/key.pem` | TLS private key path |
| `CLIPPER_TLS_REDIRECT` | `true` | Redirect HTTP to HTTPS |
| `CLIPPER_ACME_ENABLED` | `false` | Enable Let's Encrypt |
| `CLIPPER_ACME_DOMAIN` | - | Domain for certificate |
| `CLIPPER_ACME_EMAIL` | - | Contact email |
| `CLIPPER_ACME_STAGING` | `false` | Use staging environment |
| `CLIPPER_CERTS_DIR` | `/data/certs` | ACME certificate cache |
| `CLIPPER_BEARER_TOKEN` | - | Bearer token for authentication (if set, all requests require auth) |

#### Docker Volumes

- `/data` - Persistent storage for database and files
- `/certs` - Optional: Mount your TLS certificates here

#### Multi-Architecture Support

The Docker image supports multiple architectures via Docker buildx:

```bash
# Build for multiple platforms
docker buildx build --platform linux/amd64,linux/arm64 -t clipper-server .
```

## License

See the main project license.
