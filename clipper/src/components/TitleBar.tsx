import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

// Detect platform from user agent
function detectPlatform(): "macos" | "windows" | "linux" {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  return "linux";
}

interface TitleBarProps {
  onSettingsClick?: () => void;
}

export function TitleBar({ onSettingsClick }: TitleBarProps) {
  const [os] = useState(() => detectPlatform());
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    // Check initial maximized state
    const checkMaximized = async () => {
      const maximized = await getCurrentWindow().isMaximized();
      setIsMaximized(maximized);
    };
    checkMaximized();

    // Listen for window resize to update maximized state
    const unlisten = getCurrentWindow().onResized(async () => {
      const maximized = await getCurrentWindow().isMaximized();
      setIsMaximized(maximized);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for GTK header bar events on Linux
  useEffect(() => {
    if (os !== "linux") return;

    const unlisteners: Promise<() => void>[] = [];

    // GTK Settings button clicked
    unlisteners.push(
      listen("gtk-settings-clicked", () => {
        onSettingsClick?.();
      })
    );

    // GTK Minimize button clicked
    unlisteners.push(
      listen("gtk-minimize-clicked", () => {
        getCurrentWindow().minimize();
      })
    );

    // GTK Maximize button clicked
    unlisteners.push(
      listen("gtk-maximize-clicked", () => {
        getCurrentWindow().toggleMaximize();
      })
    );

    // GTK Close button clicked
    unlisteners.push(
      listen("gtk-close-clicked", () => {
        getCurrentWindow().close();
      })
    );

    return () => {
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, [os, onSettingsClick]);

  const handleMinimize = () => {
    getCurrentWindow().minimize();
  };

  const handleMaximize = () => {
    getCurrentWindow().toggleMaximize();
  };

  const handleClose = () => {
    getCurrentWindow().close();
  };

  // On macOS, we use the native traffic lights with transparent title bar
  // Just render a drag region, no custom buttons needed
  if (os === "macos") {
    return <div className="titlebar titlebar-macos" data-tauri-drag-region />;
  }

  // On Linux, we use native GTK header bar - no custom title bar needed
  // The GTK header bar is set up in Rust code
  if (os === "linux") {
    return null;
  }

  // On Windows, render custom title bar with window controls
  return (
    <div className="titlebar titlebar-windows" data-tauri-drag-region>
      <div className="titlebar-content" data-tauri-drag-region>
        <div className="titlebar-icon">
          <svg viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <linearGradient id="tbBoardGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor="#6366F1"/>
                <stop offset="100%" stopColor="#8B5CF6"/>
              </linearGradient>
              <linearGradient id="tbClipGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="#F1F5F9"/>
                <stop offset="30%" stopColor="#CBD5E1"/>
                <stop offset="70%" stopColor="#94A3B8"/>
                <stop offset="100%" stopColor="#64748B"/>
              </linearGradient>
              <linearGradient id="tbPaperGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="#FFFFFF"/>
                <stop offset="100%" stopColor="#F8FAFC"/>
              </linearGradient>
            </defs>
            <g>
              <rect x="96" y="80" width="320" height="400" rx="32" ry="32" fill="url(#tbBoardGrad)"/>
              <rect x="128" y="140" width="256" height="310" rx="16" ry="16" fill="url(#tbPaperGrad)"/>
              <g fill="#C7D2FE">
                <rect x="160" y="180" width="180" height="14" rx="7"/>
                <rect x="160" y="215" width="140" height="14" rx="7"/>
                <rect x="160" y="250" width="192" height="14" rx="7"/>
              </g>
              <g>
                <rect x="186" y="48" width="140" height="72" rx="12" ry="12" fill="url(#tbClipGrad)"/>
                <g stroke="#64748B" strokeWidth="3" strokeLinecap="round">
                  <line x1="206" y1="60" x2="206" y2="108"/>
                  <line x1="222" y1="60" x2="222" y2="108"/>
                  <line x1="290" y1="60" x2="290" y2="108"/>
                  <line x1="306" y1="60" x2="306" y2="108"/>
                </g>
              </g>
            </g>
            <circle cx="368" cy="400" r="44" fill="#10B981"/>
            <path d="M346 400 L360 414 L390 384" stroke="#FFFFFF" strokeWidth="10" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
          </svg>
        </div>
        <span className="titlebar-title">Clipper</span>
      </div>
      <div className="titlebar-controls">
        <button
          className="titlebar-button titlebar-minimize"
          onClick={handleMinimize}
          aria-label="Minimize"
        >
          <svg width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button
          className="titlebar-button titlebar-maximize"
          onClick={handleMaximize}
          aria-label={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
                d="M2.5,0.5 h7 v7 M0.5,2.5 v7 h7 v-7 h-7"
              />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10">
              <rect
                width="9"
                height="9"
                x="0.5"
                y="0.5"
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
              />
            </svg>
          )}
        </button>
        <button
          className="titlebar-button titlebar-close"
          onClick={handleClose}
          aria-label="Close"
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path
              fill="currentColor"
              d="M1.41 0L0 1.41 3.59 5 0 8.59 1.41 10 5 6.41 8.59 10 10 8.59 6.41 5 10 1.41 8.59 0 5 3.59z"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
