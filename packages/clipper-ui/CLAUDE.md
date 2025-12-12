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
  - `ClipList.tsx` - Virtualized clip list with infinite scroll
  - `ConnectionError.tsx` - Connection error display with retry
  - `DateFilter.tsx` - Date range filter picker
  - `DateTag.tsx` - Date display tag component
  - `FavoriteToggle.tsx` - Favorite star toggle button
  - `ImagePopup.tsx` - Image preview popup
  - `LanguageSelector.tsx` - Language selection dropdown
  - `SearchBox.tsx` - Search input with tag suggestions and autocomplete
  - `ShareDialog.tsx` - Dialog for generating and copying share URLs
  - `Toast.tsx` - Toast notification system
  - `Tooltip.tsx` - Tooltip component
- `src/hooks/` - React hooks
  - `useCleanupConfig.ts` - Fetches cleanup config for fade-out effect
  - `useClips.ts` - Clip data fetching and caching
  - `useKeyboardNavigation.ts` - Keyboard navigation for clip list (arrow keys, Enter, etc.)
  - `useScrollAnchor.ts` - Scroll position preservation
  - `useServerConfig.ts` - Fetches server config (checks if sharing is enabled via `shortUrlEnabled`)
  - `useSyntaxTheme.ts` - Syntax highlighting theme management
  - `useTheme.ts` - Theme (light/dark/auto) management
- `src/i18n/` - Internationalization (English, Chinese)
- `src/styles/` - CSS styles (base, dark theme)
- `src/types.ts` - TypeScript type definitions (includes `ServerConfig` with `shortUrlEnabled`)
- `src/utils/` - Utility functions
- `src/index.ts` - Main export file

## Usage

This package is consumed by:
- `clipper/` (Tauri desktop app)
- `clipper-server/web/` (Web UI)

Both apps share the same components for consistent look and feel.

## Dependencies

- `highlight.js` - Syntax highlighting for code clips
- `validator` - Input validation utilities

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
