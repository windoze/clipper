# Repository Guidelines

Scope: Shared React component library consumed by the desktop app and server web UI.

## Project Structure & Modules
- `src/components/`: reusable UI pieces; `src/hooks/`, `src/utils/`, and `src/api/` house supporting logic.
- Styles live in `src/styles/` (CSS exports referenced by consumers); keep tokens and variants consistent.
- Entry points and exports are defined in `package.json`; maintain backwards compatibility when moving files.

## Build, Test, and Development Commands
- `npm install` (in this folder) sets up deps.
- Type-check with `npx tsc --noEmit` before publishing or consuming changes.
- To validate usage, run dependents: `npm run dev` in `clipper-server/web` or `clipper` after linking/bootstrapping.

## Coding Style & Naming Conventions
- Components in PascalCase, hooks in camelCase prefixed with `use*`, utilities in camelCase; file names kebab-case unless exporting a component.
- Keep components pure and prop-driven; avoid app-specific state—leave wiring to consumers.
- Maintain strict typing for props and return values; avoid default exports to preserve tree-shaking clarity.

## Testing Guidelines
- No bundled test runner; when adding logic-heavy pieces, add light unit tests (e.g., Vitest) and keep them deterministic.
- Prefer story-like usage examples in code comments or README updates to show expected props and variations.
- Verify style exports across both light/dark themes when adding CSS tokens.

## Commit & Pull Request Guidelines
- Commit subjects should be imperative (e.g., “Add ClipboardItemCard skeleton”); document breaking or visual changes in PRs.
- Include screenshots for notable visual updates and list consumer projects tested (desktop, server web).
- Update README and changelog entries in dependent apps when APIs or styles shift.

## Security & Configuration Tips
- Avoid embedding secrets or network calls; keep the package UI-only.
- Be careful with clipboard-related helpers—leave permission and sanitization concerns to host apps.
