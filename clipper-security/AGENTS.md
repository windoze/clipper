# Repository Guidelines

Scope: Security utilities crate for cross-platform file permission management.

## Project Structure & Modules
- `src/`: platform-specific implementations behind shared interfaces; keep OS conditionals localized.
- Workspace consumers include the server, client, and CLI—maintain stable APIs and minimal dependencies.

## Build, Test, and Development Commands
- `cargo build -p clipper-security` (add `--release` for production consumers) builds the library.
- `cargo test -p clipper-security` runs tests; add platform guards to OS-specific cases.
- When adding new platform features, test on the target OS or mock the relevant APIs in CI-friendly ways.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy` before committing; keep `unsafe` blocks minimized and well-commented.
- Use snake_case modules, PascalCase types, and clear enums for permission states; avoid leaking platform details into public APIs.
- Centralize error handling and include context (path, operation) without exposing secrets.

## Testing Guidelines
- Write unit tests for permission changes and path handling; avoid modifying user data—use temp directories.
- Gate platform-specific tests with `cfg` attributes; ensure they fail fast when run on unsupported OSes.
- Document any required capabilities (e.g., Windows ACL tweaks) in test descriptions.

## Commit & Pull Request Guidelines
- Commit subjects should be imperative (e.g., “Add macOS ACL enforcement”); group platform changes logically.
- PRs should describe platform coverage, manual test steps, and any new public APIs; highlight breaking changes for downstream crates.
- Update README or inline docs when altering behavior or adding configuration knobs.

## Security & Configuration Tips
- Default to least privilege; never relax permissions silently. Warn clearly before destructive operations.
- Scrub sensitive paths from logs; prefer hashes or placeholders when troubleshooting permission issues.
