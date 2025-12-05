import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Hook that ensures the main window is at least a minimum size when a dialog is shown.
 * If the window is smaller than the required size, it will be expanded to fit the dialog.
 * When the dialog is closed, the window is NOT restored to its previous size
 * (since the user might have intentionally resized it while the dialog was open).
 *
 * This uses a Tauri command to handle the resize on the Rust side, which works
 * even when the window is hidden (e.g., when opening settings from tray menu).
 *
 * @param isOpen - Whether the dialog is currently open
 * @param minWidth - Minimum width required for the dialog (default: 550)
 * @param minHeight - Minimum height required for the dialog (default: 500)
 */
export function useEnsureWindowSize(
  isOpen: boolean,
  minWidth: number = 550,
  minHeight: number = 500
) {
  useEffect(() => {
    if (!isOpen) {
      return;
    }

    let cancelled = false;

    const ensureMinimumSize = async () => {
      if (cancelled) return;

      try {
        // Use Tauri command which handles showing the window if hidden
        // and resizing it properly
        await invoke("ensure_window_size", {
          minWidth,
          minHeight,
        });
      } catch (error) {
        console.error("Failed to ensure window size:", error);
      }
    };

    // Small delay to ensure React has finished rendering
    const timeoutId = setTimeout(() => ensureMinimumSize(), 10);

    return () => {
      cancelled = true;
      clearTimeout(timeoutId);
    };
  }, [isOpen, minWidth, minHeight]);
}
