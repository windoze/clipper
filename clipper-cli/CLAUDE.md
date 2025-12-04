# clipper-cli

Command-line interface application for managing clips via clipper-server.

## Build & Run

```bash
# Build
cargo build -p clipper-cli

# Run (requires server running)
cargo run --bin clipper-cli -- create "Hello, World!" --tags greeting
cargo run --bin clipper-cli -- search hello --page 1 --page-size 20
cargo run --bin clipper-cli -- watch
```

## Architecture

- Built with clap for argument parsing
- Uses clipper-client for server communication
- Output formats: JSON (default) or text
- Watch command outputs NDJSON (newline-delimited JSON) for real-time updates

## Commands

```bash
clipper-cli create <content> [--tags tag1,tag2] [--notes "notes"]
clipper-cli get <id> [--format json|text]
clipper-cli update <id> [--tags tag1,tag2] [--notes "notes"]
clipper-cli search <query> [--tags tag1,tag2] [--start-date ISO8601] [--end-date ISO8601] [--page 1] [--page-size 20] [--format json|text]
clipper-cli delete <id>
clipper-cli watch  # Real-time notifications as NDJSON
clipper-cli share <id> [--expires <hours>]  # Create a short URL for sharing (requires server with CLIPPER_SHORT_URL_BASE)
```

## Environment Configuration

- `CLIPPER_URL` - Server URL (default: `http://localhost:3000`)
- `CLIPPER_TOKEN` - Bearer token for authentication (optional)

## Error Handling

- CLI uses anyhow for error context
