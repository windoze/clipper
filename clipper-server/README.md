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
  -h, --help                       Print help
```

#### Environment Variables

- `CLIPPER_CONFIG` - Path to configuration file
- `CLIPPER_DB_PATH` - Path to the database directory (default: `./data/db`)
- `CLIPPER_STORAGE_PATH` - Path to the file storage directory (default: `./data/storage`)
- `CLIPPER_LISTEN_ADDR` - Server listen address (default: `0.0.0.0`)
- `PORT` - Server port (default: `3000`)
- `RUST_LOG` - Logging level (default: `clipper_server=debug,tower_http=debug`)

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
```

Or specify a custom config file location:

```bash
clipper-server --config /path/to/config.toml
```

See `config.toml.example` for a complete example.

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

## REST API Endpoints

### Health Check

```
GET /health
```

Returns `OK` if the server is running.

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

### Client Messages

Clients can send:
- **Ping messages** - Server responds with pong to keep connection alive
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
    let client = ClipperClient::new("http://localhost:3000");
    
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

### Docker Deployment (Example)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin clipper-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/clipper-server /usr/local/bin/
ENV CLIPPER_DB_PATH=/data/db
ENV CLIPPER_STORAGE_PATH=/data/storage
VOLUME ["/data"]
EXPOSE 3000
CMD ["clipper-server"]
```

## License

See the main project license.
