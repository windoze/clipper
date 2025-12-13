# Repository Guidelines

Scope: Core indexing library (SurrealDB-backed) used by server and clients.

## Project Structure & Modules
- `src/`: indexing logic, search utilities, and storage abstractions; keep modules focused on a single layer (storage, search, models).
- `tests/`: integration and unit tests; prefer isolated temp dirs for DB/state.
- `examples/`: runnable samples; keep them small and illustrative of API usage.

## Build, Test, and Development Commands
- `cargo build -p clipper-indexer` (add `--release` for profiling) builds the library.
- `cargo test -p clipper-indexer -- --test-threads=1` runs tests sequentially to avoid DB file contention.
- `cargo run -p clipper-indexer --example <name>` executes examples; ensure they do not mutate real data.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy` before commit. Prefer explicit error types over `anyhow` in the public API.
- Keep data models and search parameters in PascalCase; modules snake_case; feature flags named for capabilities (`fts`, `attachments`).
- Avoid leaking sensitive content in logs; include context IDs instead of payloads.

## Testing Guidelines
- Use temporary directories for RocksDB/SurrealDB state; clean up after tests.
- Cover indexing, search ranking, and retention behaviors with deterministic fixtures; avoid timing-dependent assertions.
- When adding new feature flags or storage backends, gate tests appropriately and document required env vars.

## Commit & Pull Request Guidelines
- Commit subjects should be imperative (e.g., “Tune BM25 defaults”); keep schema or migration notes in PR descriptions.
- PRs must note compatibility impacts on server/client crates and include manual verification steps if APIs change.
- Update README and doc comments when altering public types or configuration knobs.

## Security & Configuration Tips
- Treat stored clips as sensitive: default to least-privilege filesystem permissions in examples and docs.
- Ensure new retention or cleanup options do not default to destructive behavior without explicit user opt-in.
