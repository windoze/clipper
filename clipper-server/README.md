# Clipper Server

A REST API server with WebSocket support for managing clipboard entries using the `clipper_indexer` library.

## Features

- **REST API** for CRUD operations on clipboard entries
- **Full-text search** with filters (tags, date ranges)
- **WebSocket support** for real-time updates
- **File attachment support** for clipboard entries
- **Metadata management** (tags and additional notes)

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
GET /clips?start_date=<RFC3339>&end_date=<RFC3339>&tags=<comma-separated>
```

Query parameters (all optional):
- `start_date` - Filter clips created after this date (RFC3339 format)
- `end_date` - Filter clips created before this date (RFC3339 format)
- `tags` - Comma-separated list of tags to filter by

**Response**: `200 OK`
```json
[
  {
    "id": "abc123",
    "content": "Text content",
    "created_at": "2025-11-26T10:00:00Z",
    "tags": ["tag1", "tag2"],
    "additional_notes": "Optional notes"
  }
]
```

### Search Clips

```
GET /clips/search?q=<query>&start_date=<RFC3339>&end_date=<RFC3339>&tags=<comma-separated>
```

Query parameters:
- `q` - Search query (required)
- `start_date` - Filter clips created after this date (RFC3339 format, optional)
- `end_date` - Filter clips created before this date (RFC3339 format, optional)
- `tags` - Comma-separated list of tags to filter by (optional)

**Response**: `200 OK` (same format as list clips)

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

Search clips:
```bash
curl "http://localhost:3000/clips/search?q=hello&tags=greeting"
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
  }
};

ws.onopen = () => {
  console.log('Connected to clipper server');
};
```

## Architecture

- **axum** - Web framework
- **tokio** - Async runtime
- **tower-http** - CORS and tracing middleware
- **clipper_indexer** - Backend storage and search
- **broadcast channel** - WebSocket pub/sub for real-time updates

## Error Handling

All errors are returned as JSON:

```json
{
  "error": "Error message description"
}
```

HTTP status codes:
- `400 Bad Request` - Invalid input
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error
