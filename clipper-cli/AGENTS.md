# Repository Guidelines

Scope: Clipper command-line interface for interacting with the server.

## Project Structure & Modules
- `src/`: CLI entrypoint and commands; keep subcommands isolated and reusable through helpers.
- Depends on `clipper-client` for API calls and `clipper-security` for filesystem handling.
- Config and auth are driven by flags or env (`CLIPPER_URL`, `CLIPPER_TOKEN`); document new options in README.

## Build, Test, and Development Commands
- `cargo build -p clipper-cli` (add `--release` for distribution) builds the binary.
- `cargo test -p clipper-cli -- --test-threads=1` runs tests sequentially (avoid parallel network/FS contention).
- For smoke tests, run against a local server: `cargo run -p clipper-server -- --db-path ./data/db` in another shell, then `cargo run -p clipper-cli -- search "hello"`.

## Coding Style & Naming Conventions
- Run `cargo fmt` and `cargo clippy --tests` before committing.
- Subcommand modules: snake_case files named after the command; argument structs in PascalCase, flag names kebab-case.
- Prefer explicit errors over panics; wrap server errors with context so they’re actionable for users.

## Testing Guidelines
- Unit-test parsing and validation for each command; stub network calls via `clipper-client` mocks when feasible.
- Keep integration tests deterministic with fixed fixtures; avoid hitting real clipboard data or external networks.
- When adding new flags or output formats, assert on stdout/stderr and exit codes.

## Commit & Pull Request Guidelines
- Use imperative commit subjects (e.g., “Add share subcommand expiration flag”).
- PR descriptions should include tested commands, expected output snippets, and any new env vars or defaults.
- Update CLI help/README when behavior changes; include examples showing both flags and env usage.

## Security & Configuration Tips
- Do not echo bearer tokens in logs; redact sensitive values in error messages.
- Respect OS-specific config paths; keep permissions restrictive when writing files (tokens, caches, exports).
