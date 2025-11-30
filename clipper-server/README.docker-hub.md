# Clipper Server

A clipboard management server with REST API, WebSocket support, and built-in Web UI.

## Features

- **REST API** for CRUD operations on clipboard entries
- **Full-text search** with filters and pagination
- **WebSocket** for real-time updates
- **File attachment** support (text, images, binary files)
- **Built-in Web UI** with drag-and-drop upload
- **TLS/HTTPS** with manual or automatic (Let's Encrypt) certificates
- **Auto-cleanup** for old clips
- **Multi-architecture** support (amd64, arm64)

## Quick Start

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -v clipper-data:/data \
  windoze/clipper-server
```

Access the Web UI at `http://localhost:3000`

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_DB_PATH` | `/data/db` | Database directory |
| `CLIPPER_STORAGE_PATH` | `/data/storage` | File storage directory |
| `CLIPPER_LISTEN_ADDR` | `0.0.0.0` | Listen address |
| `PORT` | `3000` | HTTP port |
| `RUST_LOG` | `clipper_server=info` | Log level |

### Auto-Cleanup

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_CLEANUP_ENABLED` | `false` | Enable automatic cleanup |
| `CLIPPER_CLEANUP_RETENTION_DAYS` | `30` | Delete clips older than N days |
| `CLIPPER_CLEANUP_INTERVAL_HOURS` | `24` | Cleanup interval in hours |

### TLS (Manual Certificates)

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_TLS_ENABLED` | `false` | Enable HTTPS |
| `CLIPPER_TLS_PORT` | `443` | HTTPS port |
| `CLIPPER_TLS_CERT` | `/certs/cert.pem` | TLS certificate path |
| `CLIPPER_TLS_KEY` | `/certs/key.pem` | TLS private key path |
| `CLIPPER_TLS_REDIRECT` | `true` | Redirect HTTP to HTTPS |

### TLS (Let's Encrypt)

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_ACME_ENABLED` | `false` | Enable automatic certificates |
| `CLIPPER_ACME_DOMAIN` | - | Domain name for certificate |
| `CLIPPER_ACME_EMAIL` | - | Contact email for Let's Encrypt |
| `CLIPPER_ACME_STAGING` | `false` | Use staging environment |
| `CLIPPER_CERTS_DIR` | `/data/certs` | Certificate cache directory |

### Authentication

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_BEARER_TOKEN` | - | Bearer token for authentication (if set, all requests require auth) |

## Usage Examples

### Basic HTTP

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -v clipper-data:/data \
  windoze/clipper-server
```

### HTTPS with Manual Certificates

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -p 443:443 \
  -v clipper-data:/data \
  -v /path/to/certs:/certs:ro \
  -e CLIPPER_TLS_ENABLED=true \
  windoze/clipper-server
```

### HTTPS with Let's Encrypt

```bash
docker run -d \
  --name clipper \
  -p 80:3000 \
  -p 443:443 \
  -v clipper-data:/data \
  -e CLIPPER_ACME_ENABLED=true \
  -e CLIPPER_ACME_DOMAIN=clips.example.com \
  -e CLIPPER_ACME_EMAIL=admin@example.com \
  windoze/clipper-server
```

### With Auto-Cleanup

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -v clipper-data:/data \
  -e CLIPPER_CLEANUP_ENABLED=true \
  -e CLIPPER_CLEANUP_RETENTION_DAYS=7 \
  windoze/clipper-server
```

### With Authentication

```bash
docker run -d \
  --name clipper \
  -p 3000:3000 \
  -v clipper-data:/data \
  -e CLIPPER_BEARER_TOKEN=your-secret-token \
  windoze/clipper-server
```

When authentication is enabled, all API requests must include:
- `Authorization: Bearer your-secret-token` header, OR
- `?token=your-secret-token` query parameter

## Docker Compose

```yaml
services:
  clipper:
    image: windoze/clipper-server
    ports:
      - "3000:3000"
      - "443:443"
    volumes:
      - clipper-data:/data
      - ./certs:/certs:ro  # Optional: for manual TLS
    environment:
      - RUST_LOG=clipper_server=info
      # Enable authentication:
      # - CLIPPER_BEARER_TOKEN=your-secret-token
      # Enable HTTPS with Let's Encrypt:
      # - CLIPPER_ACME_ENABLED=true
      # - CLIPPER_ACME_DOMAIN=clips.example.com
      # - CLIPPER_ACME_EMAIL=admin@example.com
    restart: unless-stopped

volumes:
  clipper-data:
```

## Volumes

| Path | Description |
|------|-------------|
| `/data` | Database and file storage (persistent) |
| `/certs` | TLS certificates (for manual HTTPS) |

## Ports

| Port | Description |
|------|-------------|
| `3000` | HTTP (Web UI + REST API + WebSocket) |
| `443` | HTTPS (when TLS enabled) |

## REST API

- `GET /health` - Health check
- `GET /version` - Server version and status information
- `POST /clips` - Create a clip
- `POST /clips/upload` - Upload a file
- `GET /clips` - List clips (with pagination)
- `GET /clips/search?q=query` - Search clips
- `GET /clips/:id` - Get a clip
- `PUT /clips/:id` - Update a clip
- `DELETE /clips/:id` - Delete a clip
- `GET /clips/:id/file` - Download file attachment

## WebSocket

Connect to `ws://localhost:3000/ws` for real-time updates:

```json
{ "type": "new_clip", "id": "abc123", "content": "...", "tags": [] }
{ "type": "updated_clip", "id": "abc123" }
{ "type": "deleted_clip", "id": "abc123" }
{ "type": "clips_cleaned_up", "ids": ["..."], "count": 5 }
```

## Security

- Runs as non-root user (UID 65532)
- Uses distroless base image for minimal attack surface
- Proper signal handling with tini

## Links

- [GitHub Repository](https://github.com/user/clipper)
- [Full Documentation](https://github.com/user/clipper/blob/main/clipper-server/README.md)
