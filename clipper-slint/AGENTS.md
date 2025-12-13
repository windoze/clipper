# Repository Guidelines

Scope: Alternative Slint-based GUI (WIP).

## Project Structure & Modules
- `src/`: Rust application logic and glue to the UI; keep state management centralized.
- `ui/`: Slint UI definitions; keep components small and reuse shared styles.
- `build.rs`: Slint build integration; adjust carefully when adding assets or new UI modules.

## Build, Test, and Development Commands
- `cargo build -p clipper-slint` builds the app (use `--release` when profiling rendering).
- `cargo run -p clipper-slint` launches the GUI for manual checks.
- Add new examples under `examples/` if you introduce them, and run via `cargo run -p clipper-slint --example <name>`.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy` before committing. Keep UI IDs and component names PascalCase; Rust modules snake_case.
- Centralize cross-component messages/events; avoid direct global state where possible.
- Keep platform-specific behavior behind feature flags or clear helper modules.

## Testing Guidelines
- There are no automated UI tests yet; add focused unit tests around non-UI logic and keep them deterministic.
- Manually verify core flows (clipboard read/write, navigation) on each target platform you touch and note findings in PRs.
- When adding async operations, guard against blocking the UI thread and cover cancellation/cleanup paths.

## Commit & Pull Request Guidelines
- Commit subjects should be imperative (e.g., “Add history list filtering”).
- PRs must include manual test notes (OS, steps) and screenshots/recordings for visible changes; call out new dependencies or permissions.
- Update README or inline docs if UX, shortcuts, or settings change.

## Security & Configuration Tips
- Avoid logging clipboard content; scrub debug output.
- Ensure new features respect existing auth and TLS flows from shared crates; do not bypass security checks for convenience.
