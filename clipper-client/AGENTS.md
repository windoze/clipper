# Repository Guidelines

Scope: Rust client library used by CLI, desktop, and external integrations.

## Project Structure & Modules
- `src/`: API client, models, and notification helpers; keep HTTP concerns separated from data structs.
- `tests/`: integration-focused; run sequentially when touching the server or database.
- Exposes async interfaces; keep public API minimal and backwards compatible for downstream crates.

## Build, Test, and Development Commands
- `cargo build -p clipper-client` (add `--release` for consumers) builds the library.
- `cargo test -p clipper-client -- --test-threads=1` runs tests; prefer sequential to avoid race conditions with a shared server.
- For manual checks, run a local server (`cargo run -p clipper-server`) and exercise client calls inside a `tokio` main.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy` before pushing; keep `#![deny(missing_docs)]` ready for public surfaces when added.
- Use PascalCase for request/response structs, snake_case for modules, and clear enums for API variants.
- Prefer typed wrappers over raw strings for tokens/URLs; avoid leaking secrets in `Debug` impls.

## Testing Guidelines
- Tests should validate auth flows, paging, and notifications with deterministic fixtures; avoid real network dependencies beyond local server.
- Mock or record responses for edge cases; assert on status codes and payloads, not just success.
- Document any required env vars in test descriptions and ensure sensible defaults for CI.

## Commit & Pull Request Guidelines
- Commit messages: short imperative (e.g., “Handle TLS fingerprint pinning”); group API surface changes with docs.
- PRs should describe compatibility notes, new endpoints, and manual test steps; bump dependent crates only when necessary.
- Update README and inline docs when modifying public types or behaviors.

## Security & Configuration Tips
- Redact tokens/URLs in logs; avoid `Debug` on sensitive types.
- Keep TLS and fingerprint verification code well-tested; treat self-signed flows with explicit prompts.
