# clipper (Tauri Desktop App)

Desktop GUI application built with Tauri 2 + React + TypeScript.

## Build & Development

```bash
# Install frontend dependencies
npm install

# Development mode (runs both frontend and backend)
npm run tauri dev

# Build production app
npm run tauri build

# Build Rust backend only (requires frontend build first)
cargo build -p clipper
```

## Technology Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri 2 with Rust

## Features

- **Bundled server**: Includes clipper-server as a sidecar that starts automatically
- **Server mode selection**: Choose between bundled server or external server
- **Network access**: Option to listen on all interfaces for LAN access
- System tray with show/hide and quit menu
- Clipboard monitoring (text and images) with polling
- WebSocket connection for real-time sync
- Drag-and-drop file upload
- Settings dialog with theme support (light/dark/auto)
- Auto-launch on login (macOS, Linux, Windows)
- Favorites tagging system
- Infinite scroll clip list
- Image preview popup
- **Internationalization**: English and Chinese language support
- **Toast notifications**: Configurable notification system
- **Clear all data**: Option to wipe all clips and restart server
- **Auto-reconnect**: Reconnects to server when URL changes in settings
- **Visual fade-out**: Clips approaching auto-cleanup date gradually fade
- **Auto-refresh clip list** on WebSocket notifications (new/update/delete)
- **Clip sharing**: Generate short URLs to share clips publicly (when server has sharing enabled)

## Key Modules

Located in `src-tauri/src/`:

- `lib.rs`: Tauri app setup, plugin initialization, event handlers
- `state.rs`: AppState with ClipperClient
- `commands.rs`: Tauri commands (list_clips, search_clips, create_clip, etc.)
- `clipboard.rs`: Clipboard monitoring with text/image support
- `websocket.rs`: WebSocket listener for real-time notifications
- `settings.rs`: Settings persistence (JSON file in app config dir)
- `tray.rs`: System tray setup
- `autolaunch.rs`: Platform-specific auto-start configuration
- `server.rs`: ServerManager for bundled server lifecycle

## Configuration

Settings stored in platform-specific config directory:
- macOS: `~/Library/Application Support/com.0d0a.clipper/settings.json`
- Linux: `~/.config/com.0d0a.clipper/settings.json`
- Windows: `%APPDATA%\com.0d0a.clipper\settings.json`

### Settings Fields

- `serverAddress`: Server URL (default: `http://localhost:3000`)
- `defaultSaveLocation`: Optional default save path
- `openOnStartup`: Show window on app start
- `startOnLogin`: Auto-launch on system login
- `theme`: "light" | "dark" | "auto"
- `useBundledServer`: Use bundled server (true) or external server (false)
- `listenOnAllInterfaces`: Allow LAN access to bundled server
- `serverPort`: Port for bundled server (persisted across restarts)
- `language`: UI language ("en", "zh", or null for auto)
- `notificationsEnabled`: Show toast notifications
- `bundledServerToken`: Bearer token for bundled server authentication (auto-generated when network access is enabled)
- `externalServerToken`: Bearer token for external server authentication

## Tauri Commands

Available via `invoke()` from `@tauri-apps/api/core`:

```typescript
list_clips(filters: SearchFiltersInput, page: number, page_size: number): Promise<PagedResult>
search_clips(query: string, filters: SearchFiltersInput, page: number, page_size: number): Promise<PagedResult>
create_clip(content: string, tags: string[], additional_notes?: string): Promise<Clip>
update_clip(id: string, tags?: string[], additional_notes?: string): Promise<Clip>
delete_clip(id: string): Promise<void>
get_clip(id: string): Promise<Clip>
copy_to_clipboard(content: string): Promise<void>
upload_file(path: string, tags: string[], additional_notes?: string): Promise<Clip>
get_file_url(clip_id: string): string
download_file(clip_id: string, filename: string): Promise<string>
get_settings(): Settings
save_settings(settings: Settings): Promise<void>
browse_directory(): Promise<string | null>
check_auto_launch_status(): Promise<boolean>
get_server_url(): Promise<string>
is_bundled_server(): Promise<boolean>
switch_to_bundled_server(): Promise<string>
switch_to_external_server(server_url: string): Promise<void>
clear_all_data(): Promise<void>
toggle_listen_on_all_interfaces(listen_on_all: boolean): Promise<string>
get_local_ip_addresses(): Promise<string[]>
update_tray_language(language: string): Promise<void>
```

## Adding New Tauri Commands

1. Add function with `#[tauri::command]` attribute in `src-tauri/src/commands.rs`
2. Register in `invoke_handler` in `src-tauri/src/lib.rs`
3. Call from frontend using `invoke()` from `@tauri-apps/api/core`

## Tauri Events

The app emits events to the frontend:

```typescript
import { listen } from "@tauri-apps/api/event";

await listen("new-clip", (event) => {
  console.log("New clip:", event.payload);
});

await listen("clip-updated", (event) => { /* ... */ });
await listen("clip-deleted", (event) => { /* ... */ });
await listen("clips-cleaned-up", (event) => { /* ... */ }); // From auto-cleanup
await listen("clip-created", (event) => { /* ... */ }); // From clipboard monitor
await listen("open-settings", () => { /* ... */ }); // From tray menu
```

## Sharing Feature

The Tauri app implements clip sharing via the `tauriClient.ts` API adapter:
- `shareClip(clipId)` calls the server's `POST /clips/:id/short-url` endpoint directly using fetch
- Returns the full share URL which is displayed in the `ShareDialog` component
- Share button only appears when server has `shortUrlEnabled: true` in `/version` response

## Key Design Decisions

- **Tauri State**: Uses Tauri's managed state for AppState and SettingsManager
- **Clipboard Loop Prevention**: Last synced content tracked to prevent infinite clipboard-to-server loop

## Error Handling

- Tauri commands return `Result<T, String>` for frontend error handling
