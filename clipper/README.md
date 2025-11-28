# Clipper Desktop Application

A cross-platform clipboard manager built with Tauri 2 + React + TypeScript.

## Features

### Core Features
- **Clipboard Monitoring**: Automatically captures text and images from your clipboard
- **Full-Text Search**: Quickly find any clip with powerful search
- **Tags & Favorites**: Organize clips with tags and mark favorites
- **File Attachments**: Store files alongside text clips
- **Real-time Sync**: WebSocket-based synchronization across devices

### Server Options
- **Bundled Server**: Includes clipper-server that starts automatically - no setup required
- **External Server**: Connect to a remote clipper-server for team/multi-device use
- **Network Access**: Enable LAN access to share clips across your local network

### User Interface
- **System Tray**: Runs in background with quick access from tray
- **Theme Support**: Light, dark, and auto (follows system) themes
- **Internationalization**: English and Chinese language support
- **Toast Notifications**: Configurable notification system
- **Infinite Scroll**: Smooth scrolling through large clip collections
- **Image Preview**: Click to preview image clips
- **Drag & Drop**: Drop files directly into the app

### Platform Support
- **macOS**: Full support with auto-launch
- **Windows**: Full support with auto-launch
- **Linux**: Full support with auto-launch

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Platform-specific dependencies (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

### Installation

```bash
# Clone the repository
git clone https://github.com/user/clipper.git
cd clipper/clipper

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Configuration

Settings are stored in platform-specific locations:
- **macOS**: `~/Library/Application Support/com.0d0a.clipper/settings.json`
- **Linux**: `~/.config/com.0d0a.clipper/settings.json`
- **Windows**: `%APPDATA%\com.0d0a.clipper\settings.json`

### Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `serverAddress` | string | `http://localhost:3000` | External server URL |
| `useBundledServer` | boolean | `true` | Use bundled or external server |
| `listenOnAllInterfaces` | boolean | `false` | Allow LAN access (bundled server) |
| `theme` | string | `auto` | Theme: "light", "dark", or "auto" |
| `language` | string | `null` | Language: "en", "zh", or null (auto) |
| `openOnStartup` | boolean | `true` | Show window when app starts |
| `startOnLogin` | boolean | `false` | Launch app on system login |
| `notificationsEnabled` | boolean | `true` | Show toast notifications |
| `defaultSaveLocation` | string | `null` | Default path for file downloads |

## Architecture

```
clipper/
├── src/                    # React frontend
│   ├── components/         # React components
│   │   ├── ClipList.tsx
│   │   ├── ClipEntry.tsx
│   │   ├── SearchBox.tsx
│   │   ├── SettingsDialog.tsx
│   │   └── ...
│   ├── i18n/              # Internationalization
│   └── App.tsx
├── src-tauri/             # Tauri backend (Rust)
│   └── src/
│       ├── lib.rs         # App setup and plugins
│       ├── commands.rs    # Tauri commands
│       ├── state.rs       # Application state
│       ├── server.rs      # Bundled server manager
│       ├── settings.rs    # Settings persistence
│       ├── clipboard.rs   # Clipboard monitoring
│       ├── websocket.rs   # WebSocket client
│       ├── tray.rs        # System tray
│       └── autolaunch.rs  # Auto-launch setup
└── package.json
```

## Development

### Frontend Development

The frontend is built with:
- **React 19** for UI components
- **TypeScript** for type safety
- **Vite** for fast development builds
- **CSS** for styling (no framework)

### Backend Development

The Tauri backend uses:
- **clipper-client** for server communication
- **arboard** for clipboard access
- **tokio** for async runtime
- **tauri-plugin-shell** for sidecar management

### Building the Server Binary

The bundled server binary is built automatically during the Tauri build process:

```bash
npm run build:server  # Builds clipper-server for current platform
npm run tauri:build   # Full production build
```

## Tauri Commands

The following commands are available via `invoke()`:

### Clip Management
- `list_clips(filters, page, page_size)` - List clips with pagination
- `search_clips(query, filters, page, page_size)` - Search clips
- `create_clip(content, tags, additional_notes)` - Create new clip
- `update_clip(id, tags, additional_notes)` - Update clip metadata
- `delete_clip(id)` - Delete a clip
- `get_clip(id)` - Get clip by ID
- `copy_to_clipboard(content)` - Copy content to system clipboard
- `upload_file(path, tags, additional_notes)` - Upload file as clip
- `download_file(clip_id, filename)` - Download file attachment

### Settings
- `get_settings()` - Get current settings
- `save_settings(settings)` - Save settings
- `browse_directory()` - Open folder picker dialog
- `check_auto_launch_status()` - Check if auto-launch is enabled

### Server Management
- `get_server_url()` - Get current server URL
- `is_bundled_server()` - Check if using bundled server
- `switch_to_bundled_server()` - Switch to bundled server
- `switch_to_external_server(server_url)` - Switch to external server
- `clear_all_data()` - Clear all clips and restart server
- `toggle_listen_on_all_interfaces(listen_on_all)` - Toggle LAN access
- `get_local_ip_addresses()` - Get machine's local IP addresses
- `update_tray_language(language)` - Update tray menu language

## Events

The app emits the following events to the frontend:

| Event | Payload | Description |
|-------|---------|-------------|
| `new-clip` | `{ id, content, tags }` | New clip from WebSocket (triggers list refresh) |
| `clip-updated` | `{ id }` | Clip updated from WebSocket (triggers list refresh) |
| `clip-deleted` | `{ id }` | Clip deleted from WebSocket (triggers list refresh) |
| `clip-created` | `{ id, ... }` | Clip created from clipboard monitor |
| `open-settings` | - | Settings requested from tray |
| `server-switched` | - | Server mode changed |
| `data-cleared` | - | All data cleared |

## License

See the main project license.
