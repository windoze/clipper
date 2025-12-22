# clipper-client

Rust client library for interacting with the clipper-server REST API and WebSocket.

## Build & Test

```bash
# Build
cargo build -p clipper-client

# Test (must run sequentially - tests start temporary server instances)
cargo test -p clipper-client -- --test-threads=1
cargo test --test integration_tests -p clipper-client -- --test-threads=1
```

## Architecture

- Built with reqwest for HTTP client
- Uses tokio-tungstenite for WebSocket connections
- Type-safe API wrapping all server endpoints
- `subscribe_notifications()` for real-time updates via WebSocket
- Full support for pagination in search and list operations

## Usage

```rust
// Basic client
let client = ClipperClient::new("http://localhost:3000");

// With authentication token
let client = ClipperClient::new("http://localhost:3000")
    .with_token("your-bearer-token");

// Create clip with language
let clip = client.create_clip(
    content,
    tags,
    additional_notes,   // Option<String>
    Some("rust".to_string())  // language: Option<String>
).await?;

// Update clip language
let updated = client.update_clip(
    &clip.id,
    None,  // tags unchanged
    None,  // notes unchanged
    Some("python".to_string())  // change language
).await?;

// Pagination
let result = client.search_clips(query, filters, page, page_size).await?;
println!("Page {} of {}, Total: {}", result.page, result.total_pages, result.total);
```

## Language Field

The `Clip` struct includes an optional `language` field for syntax highlighting:
- `create_clip(content, tags, notes, language)` - 4th parameter sets the language
- `update_clip(id, tags, notes, language)` - 4th parameter updates the language
- Pass `None` to leave language unchanged, `Some("")` to clear it

## WebSocket Subscription

```rust
let (tx, mut rx) = mpsc::unbounded_channel();
let handle = client.subscribe_notifications(tx).await?;
while let Some(notification) = rx.recv().await {
    match notification {
        ClipNotification::NewClip { id, content, tags } => { /* handle */ }
        ClipNotification::UpdatedClip { id } => { /* handle */ }
        ClipNotification::DeletedClip { id } => { /* handle */ }
        ClipNotification::ClipsCleanedUp { ids, count } => { /* handle */ }
    }
}
```

## Error Handling

- `clipper_client::ClientError` - client-specific errors

## Testing Notes

- Client tests must run sequentially: `-- --test-threads=1`
- Tests require running server (tests start temporary server instances)
- **Test coverage**: 26 tests (2 unit tests + 24 integration tests, including 6 language persistence tests)
