# Clipper Client

A Rust client library for interacting with the Clipper server REST API and WebSocket notifications.

## Features

- **Full REST API Support**: Create, read, update, delete clips
- **Search & Filter**: Full-text search with date range and tag filters
- **Pagination Support**: Built-in pagination for search and list operations
- **Real-time Notifications**: WebSocket support for live clip updates
- **File Operations**: Upload files and download attachments
- **Export/Import**: Export and import clips via tar.gz archives
- **Authentication Support**: Optional Bearer token authentication
- **Async/Await**: Built on Tokio for efficient async I/O
- **Type-Safe**: Strongly typed API with comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
clipper-client = { path = "../clipper-client" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use clipper_client::{ClipperClient, SearchFilters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client (with optional authentication)
    let client = ClipperClient::new("http://localhost:3000")
        .with_token("your-secret-token".to_string()); // Optional

    // Create a clip
    let clip = client
        .create_clip(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            Some("My first clip".to_string()),
        )
        .await?;

    println!("Created clip with ID: {}", clip.id);

    // Search for clips with pagination
    let result = client
        .search_clips("Hello", SearchFilters::new(), 1, 20)
        .await?;

    println!("Found {} clips (page {} of {})",
             result.total, result.page, result.total_pages);

    Ok(())
}
```

## API Reference

### Create a Clip

```rust
let clip = client
    .create_clip(
        content: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    )
    .await?;
```

### Upload a File

```rust
let file_content = std::fs::read("document.txt")?;

let clip = client
    .upload_file(
        file_content,
        "document.txt".to_string(),
        vec!["documents".to_string()],
        Some("Important document".to_string()),
    )
    .await?;
```

### Get a Clip by ID

```rust
let clip = client.get_clip("clip_id").await?;
```

### Update a Clip

```rust
let updated = client
    .update_clip(
        "clip_id",
        Some(vec!["new_tag".to_string()]),
        Some("Updated notes".to_string()),
    )
    .await?;
```

### Search Clips

```rust
use clipper_client::SearchFilters;
use chrono::{Duration, Utc};

// Search with filters and pagination
let filters = SearchFilters::new()
    .with_tags(vec!["important".to_string()])
    .with_start_date(Utc::now() - Duration::days(7))
    .with_end_date(Utc::now());

let result = client.search_clips("query", filters, 1, 20).await?;
println!("Page {} of {}, Total: {}", result.page, result.total_pages, result.total);

for clip in result.items {
    println!("- {}: {}", clip.id, clip.content);
}
```

### List Clips

```rust
// List all clips with pagination
let result = client.list_clips(SearchFilters::new(), 1, 20).await?;

// List with filters
let filters = SearchFilters::new()
    .with_tags(vec!["work".to_string()]);
let result = client.list_clips(filters, 1, 50).await?;
```

### Delete a Clip

```rust
client.delete_clip("clip_id").await?;
```

### Export Clips

```rust
// Export all clips to a file
let bytes_written = client.export_to_file("backup.tar.gz").await?;
println!("Exported {} bytes", bytes_written);

// Export to any AsyncWrite implementation
use tokio::io::AsyncWriteExt;
let mut buffer = Vec::new();
client.export_to_writer(&mut buffer).await?;
```

### Import Clips

```rust
use clipper_client::ImportResult;

// Import clips from a file
let result: ImportResult = client.import_from_file("backup.tar.gz").await?;
println!("Imported: {}, Skipped: {}", result.imported_count, result.skipped_count);

// Import from any AsyncRead implementation
use tokio::io::AsyncReadExt;
let file = tokio::fs::File::open("backup.tar.gz").await?;
let result = client.import_from_reader(file).await?;
```

## Authentication

If the server requires authentication, use the `with_token()` method:

```rust
use clipper_client::ClipperClient;

let client = ClipperClient::new("http://localhost:3000")
    .with_token("your-secret-token".to_string());

// All subsequent requests will include the Authorization header
let clips = client.list_clips(SearchFilters::new(), 1, 20).await?;
```

The token is automatically:
- Sent as `Authorization: Bearer <token>` header for REST API requests
- Sent as a message-based authentication after WebSocket connection
- Appended as `?token=<token>` query parameter for file downloads

## WebSocket Notifications

Receive real-time updates when clips are created, updated, or deleted:

```rust
use clipper_client::{ClipperClient, ClipNotification};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClipperClient::new("http://localhost:3000");

    // Create a channel to receive notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to notifications (returns a task handle)
    let handle = client.subscribe_notifications(tx).await?;

    // Process notifications
    tokio::spawn(async move {
        while let Some(notification) = rx.recv().await {
            match notification {
                ClipNotification::NewClip { id, content, tags } => {
                    println!("New clip created: {} - {}", id, content);
                }
                ClipNotification::UpdatedClip { id } => {
                    println!("Clip updated: {}", id);
                }
                ClipNotification::DeletedClip { id } => {
                    println!("Clip deleted: {}", id);
                }
                ClipNotification::ClipsCleanedUp { ids, count } => {
                    println!("{} old clips cleaned up", count);
                }
            }
        }
    });

    // Keep the connection alive
    handle.await??;

    Ok(())
}
```

## Error Handling

The client provides a comprehensive error type:

```rust
use clipper_client::ClientError;

match client.get_clip("id").await {
    Ok(clip) => println!("Got clip: {}", clip.content),
    Err(ClientError::NotFound(msg)) => println!("Not found: {}", msg),
    Err(ClientError::BadRequest(msg)) => println!("Bad request: {}", msg),
    Err(ClientError::ServerError { status, message }) => {
        println!("Server error {}: {}", status, message)
    }
    Err(e) => println!("Error: {}", e),
}
```

## Testing

The library includes comprehensive integration tests that require a running clipper-server:

```bash
# Start the server (in another terminal)
cargo run --bin clipper-server

# Run tests
cargo test -p clipper-client --test integration_tests -- --test-threads=1
```

Tests cover:
- Creating clips with and without optional fields
- Uploading files (text and binary)
- Getting clips by ID
- Updating clip metadata
- Searching and listing with filters
- Deleting clips
- WebSocket notifications for all operations

**18 tests total - all passing ✓**

## Examples

### Complete CRUD Example

```rust
use clipper_client::ClipperClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (with optional token for authenticated servers)
    let client = ClipperClient::new("http://localhost:3000");
    // .with_token("your-secret-token".to_string()); // Uncomment if server requires auth

    // Create
    let clip = client
        .create_clip(
            "My important note".to_string(),
            vec!["work".to_string(), "important".to_string()],
            Some("Don't forget!".to_string()),
        )
        .await?;
    
    println!("Created: {}", clip.id);

    // Read
    let retrieved = client.get_clip(&clip.id).await?;
    println!("Content: {}", retrieved.content);

    // Update
    let updated = client
        .update_clip(
            &clip.id,
            Some(vec!["work".to_string(), "done".to_string()]),
            Some("Completed".to_string()),
        )
        .await?;
    
    println!("Updated tags: {:?}", updated.tags);

    // Delete
    client.delete_clip(&clip.id).await?;
    println!("Deleted");

    Ok(())
}
```

### File Upload Example

```rust
use clipper_client::ClipperClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClipperClient::new("http://localhost:3000");

    // Read file from disk
    let file_content = std::fs::read("report.pdf")?;

    // Upload the file
    let clip = client
        .upload_file(
            file_content,
            "report.pdf".to_string(),
            vec!["reports".to_string(), "monthly".to_string()],
            Some("November 2025 report".to_string()),
        )
        .await?;

    println!("Uploaded file as clip: {}", clip.id);
    println!("File stored at: {:?}", clip.file_attachment);

    Ok(())
}
```

### Search with Multiple Filters and Pagination

```rust
use clipper_client::{ClipperClient, SearchFilters};
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClipperClient::new("http://localhost:3000");

    // Search for clips from the last week with specific tags
    let filters = SearchFilters::new()
        .with_start_date(Utc::now() - Duration::days(7))
        .with_end_date(Utc::now())
        .with_tags(vec!["important".to_string(), "work".to_string()]);

    // Search with pagination (page 1, 20 items per page)
    let result = client.search_clips("meeting", filters, 1, 20).await?;

    println!("Found {} total clips", result.total);
    for clip in result.items {
        println!("Found: {} - {:?}", clip.content, clip.tags);
    }

    Ok(())
}
```

## Architecture

- **HTTP Client**: Uses `reqwest` for REST API calls
- **WebSocket**: Uses `tokio-tungstenite` for real-time notifications
- **Async Runtime**: Built on Tokio for efficient async operations
- **Type Safety**: Strongly typed models with serde serialization

## Requirements

- Rust 1.91 or later
- Tokio runtime
- Running clipper-server instance

## License

See the main project license.
