# clipper-slint

Alternative GUI application built with Slint UI framework.

## Build

```bash
cargo build -p clipper-slint
```

## Architecture

- Built with Slint 1.14 UI framework
- Uses Skia renderer with Winit backend
- Simpler architecture than Tauri version
- Connects to clipper-server via clipper-client

## Status

Basic implementation - simpler alternative to the full-featured Tauri desktop app.
