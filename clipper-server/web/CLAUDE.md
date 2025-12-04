# clipper-server/web

Pure frontend Web UI (React + Vite) for browser-based access to Clipper.

## Build & Development

```bash
# Install dependencies
npm install

# Development mode (requires server running on localhost:3000)
npm run dev

# Build production (output in dist/)
npm run build
```

## Technology Stack

- React 19 + TypeScript + Vite
- Communicates with server via REST API and WebSocket

## Features

- View and search clips with infinite scroll
- Edit clip tags and notes
- Delete clips with confirmation
- Image preview popup
- Favorites filtering
- Date range filtering
- Theme support (light/dark/auto)
- **Internationalization**: English and Chinese languages
- **Drag-and-drop file upload**: Drop files anywhere to upload
- **Send clipboard button**: Manually send clipboard content (browsers can't auto-monitor)
- **WebSocket real-time sync**: Toast notifications on new/updated clips (HTTPS only)
- **WebSocket connection status indicator**: connected/disconnected/HTTPS required
- **Visual fade-out**: Clips approaching auto-cleanup date gradually fade
- **Auto-refresh clip list** on WebSocket notifications

## Key Components

Located in `src/`:

- `hooks/useWebSocket.ts`: WebSocket connection for real-time updates
- `hooks/useCleanupConfig.ts`: Fetches cleanup config from `/version` API for fade-out effect
- `components/`: Reusable UI components (shared via @unwritten-codes/clipper-ui package)

## Architecture

- Pure frontend, communicates with server via REST API and WebSocket
- Aligned look and feel with Tauri desktop app
- Served directly from clipper-server when embedded
