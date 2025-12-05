# Changelog

All notable changes to this project will be documented in this file.

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
