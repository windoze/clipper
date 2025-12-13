# Repository Guidelines

## Project Structure & Modules
- Workspace root holds Rust crates and JS tooling; see `Cargo.toml` for members.
- Desktop app (React + Tauri): `clipper/` with `src/` for UI and `src-tauri/` for backend bindings.
- Server (Axum + SurrealDB): `clipper-server/` with bundled web UI in `clipper-server/web/`.
- Core libraries and tooling: `clipper-indexer/`, `clipper-client/`, `clipper-cli/`, `packages/clipper-ui/`, and WIP `clipper-slint/`.
- Docs and supporting assets: `docs/`, `docker/`, `cloud-services/`, `data/`.

## Build, Test, and Development Commands
- Rust workspace build: `cargo build --workspace` (add `--release` for production).
- Rust tests: `cargo test --workspace` or package-specific (`cargo test -p clipper-server -- --test-threads=1` for sequential runs).
- Desktop app: `npm install` in `clipper/`, then `npm run tauri:dev` for live dev or `npm run tauri:build` for bundles.
- Shared UI package type check: `npm run build:ui` (invokes `tsc --noEmit` under `packages/clipper-ui`).
- Server web UI: `npm run dev:web` for Vite dev server; `npm run build:web` to emit production assets.

## Coding Style & Naming Conventions
- Rust: use `cargo fmt` before committing; keep modules small and prefer explicit `pub` surfaces. Run `cargo clippy` for linting.
- TypeScript/React: type-first, prefer `function` components, hooks over classes. PascalCase for components, camelCase for variables/functions, kebab-case for file names unless React component file.
- Tests and fixtures live beside sources when possible; name test files with `_test.rs` for Rust and `.test.ts(x)` for UI.

## Testing Guidelines
- Prioritize unit tests in crates and component tests in the web/desktop UI. Add integration tests for API/DB boundaries.
- Use deterministic data; avoid real network calls. For server/client crates, keep long-running tests gated and default to sequential when hitting the database.
- Document any new env vars or feature flags in test descriptions and ensure they have sensible defaults.

## Commit & Pull Request Guidelines
- Commit messages follow short, imperative summaries (e.g., “Fix clipboard sync retry”); group related changes per commit.
- PRs should describe scope, motivation, and risks; link issues when applicable and note manual test steps (commands, platforms). Include screenshots for UI changes and mention breaking changes explicitly.
- Keep changes minimal per PR; update docs (`README.md`, `docs/`) and configuration samples when behavior or flags change.

## Security & Configuration Tips
- Server defaults bind to `0.0.0.0`; set `CLIPPER_LISTEN_ADDR=127.0.0.1` for local-only use.
- Always set `CLIPPER_BEARER_TOKEN` when exposing the server and prefer TLS builds (`--features tls` or `acme`) on untrusted networks.
- Treat clipboard data and exports as sensitive; ensure storage paths are permissioned per-user and short URLs are used only for non-sensitive clips.
