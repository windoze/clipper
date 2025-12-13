# Repository Guidelines

Scope: Clipper desktop app (React + Vite + Tauri 2).

## Project Structure & Modules
- `src/`: React views, hooks, and shared UI glue; `public/` holds static assets.
- `src-tauri/`: Rust backend for Tauri commands; keep API surface minimal and typed.
- `scripts/`: helper utilities (e.g., `build-server.js`) used by Tauri builds.
- `packages/clipper-ui` is consumed locally for shared components and styles.

## Build, Test, and Development Commands
- `npm install` (run in this folder) sets up the desktop app.
- `npm run dev`: Vite dev server for the React shell (no Tauri).
- `npm run tauri:dev`: builds the Tauri sidecar and launches the desktop app for live testing.
- `npm run tauri:build`: production bundle; runs `build:server` before packaging.
- `npm run build`: type-checks via `tsc` then builds static assets.

## Coding Style & Naming Conventions
- Type-first: keep `tsc` clean, prefer explicit types on component props and Tauri payloads.
- Components: PascalCase files in `src`, hooks in `src/hooks`, shared utilities in `src/lib` when added; camelCase variables/functions.
- Keep side effects inside hooks; avoid direct clipboard access outside dedicated services. Keep Rust commands small and typed with `serde`.

## Testing Guidelines
- No default UI harness is included; when adding logic-heavy pieces, add focused tests (e.g., Vitest + DOM helpers) and keep them deterministic.
- For Tauri code, prefer unit tests in `src-tauri` that mock the filesystem/network; avoid hitting the real clipboard or OS services.
- Smoke-test UI flows via `npm run tauri:dev` before opening PRs; document manual steps.

## Commit & Pull Request Guidelines
- Use short, imperative commits (e.g., “Fix tray sync badge”) and group UI + Tauri changes logically.
- PRs should list scope, manual test matrix (OS/arch), and screenshots or short recordings for UI tweaks. Flag breaking changes or new permissions.
- Update docs or in-app help if UX or settings behavior shifts; align text with `README.md` phrasing where possible.

## Security & Configuration Tips
- Do not log clipboard content or bearer tokens; scrub debug output before submitting.
- `build:server` bundles the backend—ensure config defaults (listen addr, auth) match `clipper-server` expectations.
- Avoid adding new permissions or Tauri plugins without documenting why and how they’re gated by settings.
