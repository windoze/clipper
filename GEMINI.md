# Project: Clipper

## Overview
**Clipper** is a Rust-based clipboard management and indexing system designed to persist, tag, and search clipboard history. It is organized as a Cargo workspace containing a core library (`clipper_indexer`) and a CLI application (`clipper`).

The project aims to provide a local, privacy-focused way to recall past clipboard content using full-text search and rich metadata.

## Architecture

### Workspace Structure
*   **`clipper_indexer/` (Library):** The core engine of the application.
    *   **Database:** Uses embedded **SurrealDB** (with RocksDB backend) for storing entry metadata and full-text search indices.
    *   **File Storage:** Uses the `object_store` crate to manage file attachments (blobs) separately from the metadata.
    *   **Search:** Leverages SurrealDB's BM25 ranking for full-text search capabilities.
*   **`clipper/` (Binary):** The user-facing Command Line Interface.
    *   *Current Status:* Placeholder (Work In Progress). It currently only prints "Hello, world!".

### Key Technologies
*   **Rust:** Primary programming language (Edition 2024).
*   **SurrealDB:** Embedded database for structural data and search.
*   **Object Store:** Abstraction for file storage.
*   **Tokio:** Async runtime.

## Development

### Prerequisites
*   Rust 1.70+
*   `cmake` (required for building RocksDB/SurrealDB dependencies)

### Build & Run

**Build the entire workspace:**
```bash
cargo build
```

**Run the Indexer Example:**
Since the main CLI is not yet ready, the best way to see the project in action is via the library examples.
```bash
cargo run -p clipper_indexer --example basic_usage
```

**Run Tests:**
Run tests for the indexer library (comprehensive suite covering DB, storage, and search).
```bash
cargo test -p clipper_indexer
```

## Usage (Planned)
The `clipper` CLI will eventually provide commands similar to:
*   `clipper add "text content"`
*   `clipper add -f ./image.png`
*   `clipper search "query"`
*   `clipper list --tags "work"`

## Current Status & Roadmap
*   [x] **Core Library (`clipper_indexer`):** fully implemented with database schema, storage logic, and search API.
*   [ ] **CLI Application (`clipper`):** Currently a scaffold. Needs implementation to interface with `clipper_indexer`.
*   [ ] **Daemon/Watcher:** (Potential future feature) To automatically capture clipboard changes.

## Conventions
*   **Async/Await:** The codebase is heavily async, relying on the Tokio runtime.
*   **Error Handling:** Uses a custom `IndexerError` enum in `clipper_indexer::error`.
*   **Dependency Management:** Workspace-based dependency management.
