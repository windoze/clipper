# Repository Guidelines

Scope: Clipper server (Axum + SurrealDB) with bundled web UI.

## Project Structure & Modules
- `src/`: HTTP handlers, models, storage, and sync layers; keep modules cohesive and narrow.
- `tests/`: integration-style Rust tests; run sequentially when touching the database.
- `web/`: Vite React web UI bundled into server assets; depends on `packages/clipper-ui`.
- `config.toml.example`: reference for runtime configuration; mirror updates here when adding settings.

## Build, Test, and Development Commands
- `cargo build -p clipper-server` (add `--release` for production) builds the server binary.
- `cargo test -p clipper-server -- --test-threads=1` runs server tests sequentially to avoid DB contention.
- `cargo run -p clipper-server -- --db-path ./data/db --storage-path ./data/storage` for local manual runs.
- `cd web && npm install` to set up UI deps, `npm run dev` for UI development, `npm run build` to refresh bundled assets.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy --workspace --tests` before pushing. Prefer explicit error types over `unwrap`.
- Name handlers with HTTP verbs (`get_clip`, `create_clip`), config structs with `Config*`, and feature flags with clear prefixes.
- Keep web UI components in PascalCase; co-locate page-level styles and avoid global CSS churn.

## Testing Guidelines
- Favor integration tests around API endpoints and storage boundaries; seed deterministic fixtures and clean temp directories.
- Avoid real network dependencies; gate slow/db-heavy tests with feature flags if needed.
- When touching TLS/auth paths, cover both authenticated and unauthenticated cases and note required env vars in test docs.

## Commit & Pull Request Guidelines
- Commit messages: short imperative (e.g., “Add cleanup retention flag”); keep config + docs changes in the same PR.
- PRs should call out migrations, new env vars, and manual test steps; include screenshots for web UI updates.
- Update `config.toml.example`, README, and Docker docs when behavior or defaults change.

## Security & Configuration Tips
- Default bind is `0.0.0.0`; set `CLIPPER_LISTEN_ADDR=127.0.0.1` for local-only use.
- Require auth when exposed: set `CLIPPER_BEARER_TOKEN`, and prefer TLS builds (`--features tls` or `acme`) on untrusted networks.
- Treat storage paths and exports as sensitive; ensure permissions are user-only and avoid logging clip contents.
