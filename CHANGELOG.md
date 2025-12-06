# Changelog

All notable changes to this project will be documented in this file.

## [0.17.0] - 2025-12-07

### Added
- Export/Import functionality for clips via tar.gz archives
  - `GET /export` - Export all clips and attachments as a tar.gz archive
  - `POST /import` - Import clips from a tar.gz archive with automatic deduplication
  - CLI commands: `clipper-cli export` and `clipper-cli import`
  - Client library methods: `export_to_file()`, `export_to_writer()`, `import_from_file()`, `import_from_reader()`
- Deduplication on import (skips clips with same ID or content hash)
- Streaming support for efficient handling of large archives

### Documentation
- Updated all README files with export/import documentation
- Added export/import to REST API endpoint tables
- Added CLI command documentation for export and import
- Added client library API documentation for export/import functions

## [0.16.4] - 2025-12-06

### Security
- Upgraded React to 19.2.1 to address CVE-2025-55182 (note: this project does not use React Server Components and is not affected by this vulnerability)

### Changed
- Upgraded React to 19.2.1
- Upgraded TypeScript to 5.9.3
- Upgraded @vitejs/plugin-react to 5.1.1
- Upgraded Vite to 7.2.6
- Upgraded @types/react to 19.2.7
- Upgraded @types/react-dom to 19.2.3

### Fixed
- Fixed duplicate React instances issue when using linked packages
- Optimized build chunks for better caching (split react and highlight.js into separate chunks)

## [0.16.3] - 2025-12-06

### Added
- Confirmation dialog before sharing clips
- Favicon for shared clip pages
- Nerd Font for better glyph/icon displaying
- Tooltip for additional notes field

## [0.16.2] - 2025-12-06

### Added
- Download progress bar with speed indicator in update settings dialog
- Restart functionality after update download (spawns new instance after exit)

### Fixed
- "Download and Install" button remaining visible after update download on macOS
- "Quit Now" button not working after update download
- Graceful server shutdown before app restart to avoid port conflicts

## [0.16.0] - 2025-12-05

### Added
- Self-signed certificate support with SSH-like fingerprint verification in clipper-cli
- Self-signed certificate trust in desktop app with UI dialog for certificate verification
- Shared trusted certificates storage between CLI and desktop app

### Changed
- File too big toast notification for oversized uploads

### Fixed
- Various compiler warnings
- Copy/paste file handling improvements
- Auto cleanup for expired short URLs

## [0.15.0] - Previous Release

Initial tracked release with:
- Core clipboard indexer with full-text search
- REST API server with WebSocket support
- Desktop application (Tauri 2 + React)
- CLI application
- Web UI
- TLS/HTTPS support with ACME
- Authentication (Bearer token)
- Clip sharing via short URLs
- Auto-cleanup with configurable retention
