# Clipper CLI

A command-line interface for managing clipboard entries with the Clipper server.

## Features

- **Create clips** from command line with tags and notes
- **Search clips** with full-text search and filters
- **List and filter** clips by tags and date ranges
- **Update clip metadata** (tags and notes)
- **Delete clips** by ID
- **Watch mode** for real-time clip notifications
- **Pagination support** for search and list operations
- **Authentication support** for secured servers
- **Multiple output formats**: JSON (default) or plain text

## Installation

Build from source:

```bash
cargo build --release -p clipper-cli
```

The binary will be available at `target/release/clipper-cli`.

## Configuration

The CLI can be configured using environment variables:

- `CLIPPER_URL` - Server URL (default: `http://localhost:3000`)
- `CLIPPER_TOKEN` - Bearer token for authentication (optional)

Example:
```bash
export CLIPPER_URL=http://clipper-server.local:8080
export CLIPPER_TOKEN=your-secret-token
```

## Usage

### Prerequisites

The Clipper server must be running before using the CLI. Start the server:

```bash
cargo run --bin clipper-server
```

Or with custom configuration:
```bash
CLIPPER_DB_PATH=./data/db cargo run --bin clipper-server
```

### Basic Commands

```bash
clipper-cli [OPTIONS] <COMMAND>

Options:
  -u, --url <URL>      Server URL [env: CLIPPER_URL] [default: http://localhost:3000]
  -t, --token <TOKEN>  Bearer token for authentication [env: CLIPPER_TOKEN]
  -h, --help           Print help
```

## Commands

### create - Create a new clip

```bash
clipper-cli create <CONTENT> [OPTIONS]

Arguments:
  <CONTENT>  Clip content (text)

Options:
  -t, --tags <TAGS>              Tags (comma-separated)
  -n, --notes <NOTES>            Additional notes
  -h, --help                     Print help

Examples:
  # Simple clip
  clipper-cli create "Hello, World!"

  # With tags
  clipper-cli create "Important meeting notes" --tags work,meeting

  # With tags and notes
  clipper-cli create "TODO: Review PR" --tags todo,urgent --notes "Due by Friday"

  # Pipe content from stdin
  echo "Clipboard content" | xargs clipper-cli create
```

**Output**: JSON with created clip details

### get - Get a clip by ID

```bash
clipper-cli get <ID> [OPTIONS]

Arguments:
  <ID>  Clip ID

Options:
  -f, --format <FORMAT>  Output format: json or text [default: json]
  -h, --help             Print help

Examples:
  # Get as JSON
  clipper-cli get abc123

  # Get content only (text format)
  clipper-cli get abc123 --format text

  # Save content to file
  clipper-cli get abc123 --format text > output.txt
```

### search - Search clips

```bash
clipper-cli search <QUERY> [OPTIONS]

Arguments:
  <QUERY>  Search query

Options:
  -t, --tags <TAGS>                  Filter by tags (comma-separated)
      --start-date <START_DATE>      Filter by start date (ISO 8601 format)
      --end-date <END_DATE>          Filter by end date (ISO 8601 format)
  -p, --page <PAGE>                  Page number [default: 1]
      --page-size <PAGE_SIZE>        Items per page [default: 20]
  -f, --format <FORMAT>              Output format: json or text [default: json]
  -h, --help                         Print help

Examples:
  # Basic search
  clipper-cli search hello

  # Search with tag filter
  clipper-cli search meeting --tags work

  # Search with multiple filters
  clipper-cli search report --tags work,important --start-date 2025-11-01T00:00:00Z

  # Paginated search
  clipper-cli search todo --page 2 --page-size 10

  # Text output (easier to parse)
  clipper-cli search notes --format text
```

**Output**: 
- JSON format: Complete paginated result with metadata
- Text format: One clip per entry (ID + content), pagination info to stderr

### update - Update a clip's metadata

```bash
clipper-cli update <ID> [OPTIONS]

Arguments:
  <ID>  Clip ID

Options:
  -t, --tags <TAGS>      New tags (comma-separated)
  -n, --notes <NOTES>    New additional notes
  -h, --help             Print help

Examples:
  # Update tags
  clipper-cli update abc123 --tags done,archived

  # Update notes
  clipper-cli update abc123 --notes "Completed on 2025-11-26"

  # Update both
  clipper-cli update abc123 --tags work,completed --notes "Finished"
```

**Note**: At least one of `--tags` or `--notes` must be provided.

### delete - Delete a clip

```bash
clipper-cli delete <ID>

Arguments:
  <ID>  Clip ID

Examples:
  clipper-cli delete abc123
```

### watch - Watch for real-time notifications

```bash
clipper-cli watch

Examples:
  # Watch and display all clip events
  clipper-cli watch

  # Filter events with jq (requires jq to be installed)
  clipper-cli watch | jq 'select(.type == "new_clip")'

  # Save events to file
  clipper-cli watch > clips.ndjson
```

**Output**: NDJSON (newline-delimited JSON) - one JSON object per line

Notification types:
```json
{"type":"new_clip","id":"abc123","content":"Hello","tags":["greeting"]}
{"type":"updated_clip","id":"abc123"}
{"type":"deleted_clip","id":"abc123"}
{"type":"clips_cleaned_up","ids":["abc123","def456"],"count":2}
```

## Output Formats

### JSON Format (default)

Pretty-printed JSON with full clip details:

```json
{
  "id": "abc123",
  "content": "Hello, World!",
  "created_at": "2025-11-26T10:00:00Z",
  "tags": ["greeting"],
  "additional_notes": "A friendly message"
}
```

For search/list commands, includes pagination metadata:

```json
{
  "items": [...],
  "total": 100,
  "page": 1,
  "page_size": 20,
  "total_pages": 5
}
```

### Text Format

Plain text output for easier processing:

```bash
# get command - outputs content only
Hello, World!

# search command - outputs ID and content
abc123
Hello, World!

def456
Another clip
```

Pagination info is printed to stderr, so it doesn't interfere with piping content.

## Pagination

Search and list operations support pagination:

```bash
# Get first page (default: 20 items)
clipper-cli search "query"

# Get second page with custom page size
clipper-cli search "query" --page 2 --page-size 50

# Large page size for bulk operations
clipper-cli search "query" --page-size 100
```

## Advanced Usage

### Scripting Examples

Create multiple clips from a file:
```bash
while IFS= read -r line; do
  clipper-cli create "$line" --tags imported
done < input.txt
```

Search and delete old clips:
```bash
clipper-cli search "" --tags temporary --end-date 2025-11-01T00:00:00Z --format json | \
  jq -r '.items[].id' | \
  while read id; do
    clipper-cli delete "$id"
  done
```

Monitor new clips in real-time:
```bash
clipper-cli watch | jq 'select(.type == "new_clip") | .content'
```

Export all clips to JSON:
```bash
# Get total pages first
total_pages=$(clipper-cli search "" --page 1 --page-size 100 --format json | jq '.total_pages')

# Fetch all pages
for page in $(seq 1 $total_pages); do
  clipper-cli search "" --page $page --page-size 100 --format json | jq '.items[]'
done > all_clips.json
```

### Integration with Other Tools

**fzf integration** (fuzzy search):
```bash
clipper-cli search "" --format text | fzf
```

**rofi integration** (GUI menu):
```bash
clip_id=$(clipper-cli search "" --format text | rofi -dmenu -i -p "Clip:" | head -1)
clipper-cli get "$clip_id" --format text | xclip -selection clipboard
```

**Copy to system clipboard**:
```bash
# Linux (X11)
clipper-cli get abc123 --format text | xclip -selection clipboard

# macOS
clipper-cli get abc123 --format text | pbcopy

# WSL
clipper-cli get abc123 --format text | clip.exe
```

## Error Handling

The CLI returns appropriate exit codes:
- `0` - Success
- `1` - Error (connection failed, clip not found, invalid input, etc.)

Errors are printed to stderr with context:
```
Error: Failed to get clip

Caused by:
    404 Not Found: Clip not found: abc123
```

## Environment Variables

- `CLIPPER_URL` - Server URL (can be overridden with `-u` flag)
- `CLIPPER_TOKEN` - Bearer token for authentication (can be overridden with `-t` flag)
- `RUST_LOG` - Log level for debugging (e.g., `RUST_LOG=debug clipper-cli search test`)

## Authentication

If the server requires authentication, provide the bearer token:

```bash
# Using command-line option
clipper-cli --token your-secret-token search "hello"

# Using environment variable
export CLIPPER_TOKEN=your-secret-token
clipper-cli search "hello"

# One-time with environment variable
CLIPPER_TOKEN=your-secret-token clipper-cli search "hello"
```

The token is sent as an `Authorization: Bearer <token>` header with all requests.

## Requirements

- Rust 1.70 or later (for building)
- Running clipper-server instance
- Network connectivity to the server

## Common Issues

**Connection refused**:
```
Error: Failed to create clip
Caused by: error sending request for url (http://localhost:3000/clips): connection error: Connection refused
```
→ Make sure the server is running: `cargo run --bin clipper-server`

**Invalid date format**:
```
Error: Invalid start_date format, use ISO 8601
```
→ Use RFC3339/ISO 8601 format: `2025-11-26T10:00:00Z`

**Server URL not found**:
→ Set the correct server URL: `clipper-cli -u http://your-server:3000 search test`

**401 Unauthorized**:
```
Error: Failed to search clips
Caused by: 401 Unauthorized: Invalid or missing authentication token
```
→ The server requires authentication. Provide the token: `clipper-cli --token your-secret-token search test`

## Development

Run from source:
```bash
cargo run --bin clipper-cli -- create "Test clip"
```

Run tests:
```bash
cargo test -p clipper-cli
```

Build release binary:
```bash
cargo build --release -p clipper-cli
./target/release/clipper-cli --help
```

## Related Projects

- **clipper-server** - REST API server backend
- **clipper-client** - Rust client library
- **clipper-indexer** - Core indexing and search library

## License

See the main project license.
