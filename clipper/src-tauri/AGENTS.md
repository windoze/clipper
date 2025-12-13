# Repository Guidelines

Scope: Tauri backend crate for the desktop app.

## Project Structure & Modules
- `src/`: commands, state management, and platform hooks; keep commands small and focused.
- `binaries/`: embedded resources; `icons/` for app assets; `capabilities/` for Tauri capability manifests.
- `tauri*.conf.json`: platform-specific configuration; update alongside capability changes.
- `build.rs`: Tauri build setup; adjust carefully when adding resources.

## Build, Test, and Development Commands
- `cargo build -p clipper` (from repo root) builds the Tauri backend; use `--release` for packaging.
- During app development, use `npm run tauri:dev` from `clipper/` to build and run both UI and backend.
- `cargo test -p clipper -- --test-threads=1` for backend-focused tests; keep them isolated from real clipboard/system services.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy` before committing.
- Commands: snake_case function names exposed via Tauri; keep request/response structs in PascalCase with `serde` derives.
- Avoid panics; prefer structured errors with user-friendly messages bubbled to the UI.
- Keep platform-specific code behind `cfg` guards; avoid leaking OS-only features into shared paths.

## Testing Guidelines
- Mock filesystem, network, and clipboard interactions; avoid touching real user data.
- When adding background tasks (sync, cleanup), ensure cancellation/cleanup is covered and does not block the UI thread.
- Document required env vars or feature flags for tests; keep temp directories per test.

## Commit & Pull Request Guidelines
- Imperative commit subjects (e.g., “Add bundled server memory cap”); group capability/conf changes with code updates.
- PRs should list manual test steps (OS, actions) and note any new permissions or plugins; include screenshots/log snippets if relevant.
- Update Tauri config files and desktop README when behavior or capabilities change.

## Security & Configuration Tips
- Never log clipboard contents or bearer tokens; scrub sensitive values.
- Keep capability manifests minimal; only add new permissions with clear rationale and UI gating.
- When spawning the bundled server, enforce localhost binding and token requirements by default.
