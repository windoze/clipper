# @unwritten-codes/clipper-ui

Shared React UI component library used by both the Tauri desktop app and the web UI.

## Package Info

- **Name**: `@unwritten-codes/clipper-ui`
- **Type**: ES Module
- **Peer Dependencies**: React 19

## Exports

```typescript
import { ... } from "@unwritten-codes/clipper-ui";           // Main entry
import { ... } from "@unwritten-codes/clipper-ui/types";     // Type definitions
import { ... } from "@unwritten-codes/clipper-ui/api";       // API utilities
import { ... } from "@unwritten-codes/clipper-ui/i18n";      // Internationalization
import { ... } from "@unwritten-codes/clipper-ui/components"; // React components
import { ... } from "@unwritten-codes/clipper-ui/hooks";     // React hooks

// Styles
import "@unwritten-codes/clipper-ui/styles/index.css";
import "@unwritten-codes/clipper-ui/styles/base.css";
import "@unwritten-codes/clipper-ui/styles/dark.css";
```

## Directory Structure

- `src/api/` - API client utilities
- `src/components/` - Reusable React components
  - `ClipEntry.tsx` - Individual clip display with copy, edit, delete, share buttons
  - `ShareDialog.tsx` - Dialog for generating and copying share URLs
  - `Toast.tsx` - Toast notification system
  - `ImagePopup.tsx` - Image preview popup
  - `EditClipDialog.tsx` - Edit clip tags and notes
- `src/hooks/` - React hooks
  - `useServerConfig.ts` - Fetches server config (checks if sharing is enabled via `shortUrlEnabled`)
  - `useCleanupConfig.ts` - Fetches cleanup config for fade-out effect
- `src/i18n/` - Internationalization (English, Chinese)
- `src/styles/` - CSS styles (base, dark theme)
- `src/types.ts` - TypeScript type definitions (includes `ServerConfig` with `shortUrlEnabled`)
- `src/index.ts` - Main export file

## Usage

This package is consumed by:
- `clipper/` (Tauri desktop app)
- `clipper-server/web/` (Web UI)

Both apps share the same components for consistent look and feel.

## Dependencies

- `highlight.js` - Syntax highlighting for code clips

## Sharing Feature

The `ShareDialog` component handles clip sharing:
- Opens when user clicks share button on a clip
- Calls `api.shareClip(clipId)` to generate a short URL
- Displays the URL with a copy button
- Only shown when server has sharing enabled (`ServerConfig.shortUrlEnabled`)

The API client interface requires a `shareClip` method:
```typescript
interface ApiClient {
  shareClip?(clipId: string): Promise<string>;  // Returns the share URL
  // ... other methods
}
```

## Copy Image Feature

The `ClipEntry` component supports copying images to clipboard:
- Image clips show a "Copy" button as well
- Calls `api.copyImageToClipboard(clipId)` to copy the image
- Works in both desktop app (via Tauri command) and web UI (via Clipboard API)

The API client interface requires a `copyImageToClipboard` method:
```typescript
interface ApiClient {
  copyImageToClipboard?(clipId: string): Promise<void>;
  // ... other methods
}
```
