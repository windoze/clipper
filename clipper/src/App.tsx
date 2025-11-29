import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  useClips,
  useTheme,
  useI18n,
  useToast,
  SearchBox,
  DateFilter,
  FavoriteToggle,
  ClipList,
} from "@anthropic/clipper-ui";
import { TitleBar } from "./components/TitleBar";
import { DropZone } from "./components/DropZone";
import { SettingsDialog, useSettingsDialog } from "./components/SettingsDialog";
import "./App.css";

// Detect platform from user agent
function detectPlatform(): "macos" | "windows" | "linux" {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  return "linux";
}

function App() {
  const { t } = useI18n();
  const [os] = useState(() => detectPlatform());
  const [isMaximized, setIsMaximized] = useState(false);
  const [wsConnected, setWsConnected] = useState(false);
  const {
    clips,
    loading,
    loadingMore,
    error,
    total,
    hasMore,
    searchQuery,
    setSearchQuery,
    filters,
    setFilters,
    favoritesOnly,
    setFavoritesOnly,
    refetch,
    loadMore,
    toggleFavorite,
    updateClipInList,
    deleteClipFromList,
  } = useClips();

  const { isOpen: isSettingsOpen, open: openSettings, close: closeSettings } = useSettingsDialog();
  const { updateTheme } = useTheme();
  const { showToast } = useToast();

  // Track window maximized state for Windows and Linux
  useEffect(() => {
    if (os !== "windows" && os !== "linux") return;

    const checkMaximized = async () => {
      const maximized = await getCurrentWindow().isMaximized();
      setIsMaximized(maximized);
    };
    checkMaximized();

    const unlisten = getCurrentWindow().onResized(async () => {
      const maximized = await getCurrentWindow().isMaximized();
      setIsMaximized(maximized);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [os]);

  // Get initial WebSocket status and listen for changes
  useEffect(() => {
    // Get initial status
    invoke<boolean>("get_websocket_status").then(setWsConnected).catch(() => { });

    // Listen for status changes
    const unlistenWsStatus = listen<{ connected: boolean }>("websocket-status", (event) => {
      setWsConnected(event.payload.connected);
    });

    return () => {
      unlistenWsStatus.then((fn) => fn());
    };
  }, []);

  // Listen for data-cleared and server-switched events to refresh clips
  useEffect(() => {
    const unlistenDataCleared = listen("data-cleared", () => {
      refetch();
    });

    const unlistenServerSwitched = listen("server-switched", () => {
      refetch();
    });

    // Listen for WebSocket notifications from server
    const unlistenNewClip = listen("new-clip", () => {
      showToast(t("toast.clipReceived"));
      refetch();
    });

    const unlistenClipUpdated = listen("clip-updated", () => {
      refetch();
    });

    const unlistenClipDeleted = listen("clip-deleted", () => {
      refetch();
    });

    return () => {
      unlistenDataCleared.then((fn) => fn());
      unlistenServerSwitched.then((fn) => fn());
      unlistenNewClip.then((fn) => fn());
      unlistenClipUpdated.then((fn) => fn());
      unlistenClipDeleted.then((fn) => fn());
    };
  }, [refetch, showToast, t]);

  // Window control handlers for Windows
  const handleMinimize = () => {
    getCurrentWindow().minimize();
  };

  const handleMaximize = () => {
    getCurrentWindow().toggleMaximize();
  };

  const handleClose = () => {
    getCurrentWindow().close();
  };

  // Tag filter handlers
  const filterTags = filters.tags || [];

  const handleAddTagFilter = useCallback((tag: string) => {
    setFilters((prev) => {
      const currentTags = prev.tags || [];
      // Don't add if already in filter
      if (currentTags.includes(tag)) return prev;
      return {
        ...prev,
        tags: [...currentTags, tag],
      };
    });
  }, [setFilters]);

  const handleRemoveTagFilter = useCallback((tag: string) => {
    setFilters((prev) => ({
      ...prev,
      tags: (prev.tags || []).filter((t) => t !== tag),
    }));
  }, [setFilters]);

  const handleClearAllTags = useCallback(() => {
    setFilters((prev) => ({
      ...prev,
      tags: [],
    }));
  }, [setFilters]);

  return (
    <DropZone>
      <div className={`app ${os}`}>
        {/* Only render TitleBar on macOS, for Windows and Linux we integrate controls into header */}
        {os === "macos" && <TitleBar />}
        {/* Composited title bar for macOS with all controls in one row */}
        {os === "macos" ? (
          <header className="app-header-unified" data-tauri-drag-region>
            {/* Left section: Icon and title */}
            <div className="header-left" data-tauri-drag-region>
              <svg className="app-icon" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg" data-tauri-drag-region>
                <defs>
                  <linearGradient id="boardGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#6366F1" />
                    <stop offset="100%" stopColor="#8B5CF6" />
                  </linearGradient>
                  <linearGradient id="clipGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stopColor="#F1F5F9" />
                    <stop offset="30%" stopColor="#CBD5E1" />
                    <stop offset="70%" stopColor="#94A3B8" />
                    <stop offset="100%" stopColor="#64748B" />
                  </linearGradient>
                  <linearGradient id="paperGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" stopColor="#FFFFFF" />
                    <stop offset="100%" stopColor="#F8FAFC" />
                  </linearGradient>
                  <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
                    <feDropShadow dx="0" dy="8" stdDeviation="16" floodColor="#1E1B4B" floodOpacity="0.35" />
                  </filter>
                  <filter id="innerDepth">
                    <feDropShadow dx="0" dy="2" stdDeviation="2" floodColor="#4338CA" floodOpacity="0.3" />
                  </filter>
                </defs>
                <g filter="url(#shadow)">
                  <rect x="96" y="80" width="320" height="400" rx="32" ry="32" fill="url(#boardGrad)" />
                  <rect x="128" y="140" width="256" height="310" rx="16" ry="16" fill="url(#paperGrad)" />
                  <g fill="#C7D2FE">
                    <rect x="160" y="180" width="180" height="14" rx="7" />
                    <rect x="160" y="215" width="140" height="14" rx="7" />
                    <rect x="160" y="250" width="192" height="14" rx="7" />
                    <rect x="160" y="285" width="120" height="14" rx="7" />
                    <rect x="160" y="320" width="160" height="14" rx="7" />
                  </g>
                  <g>
                    <rect x="186" y="48" width="140" height="72" rx="12" ry="12" fill="url(#clipGrad)" />
                    <g stroke="#64748B" strokeWidth="3" strokeLinecap="round">
                      <line x1="206" y1="60" x2="206" y2="108" />
                      <line x1="222" y1="60" x2="222" y2="108" />
                      <line x1="290" y1="60" x2="290" y2="108" />
                      <line x1="306" y1="60" x2="306" y2="108" />
                    </g>
                    <path d="M194 120 C194 136, 210 148, 230 148" stroke="#94A3B8" strokeWidth="10" strokeLinecap="round" fill="none" />
                    <path d="M318 120 C318 136, 302 148, 282 148" stroke="#94A3B8" strokeWidth="10" strokeLinecap="round" fill="none" />
                  </g>
                </g>
                <g filter="url(#innerDepth)">
                  <circle cx="368" cy="400" r="44" fill="#10B981" />
                  <path d="M346 400 L360 414 L390 384" stroke="#FFFFFF" strokeWidth="10" strokeLinecap="round" strokeLinejoin="round" fill="none" />
                </g>
              </svg>
              <h1 className="app-title" data-tauri-drag-region>{t("app.title")}</h1>
            </div>

            {/* Center section: Search and filters */}
            <div className="header-center">
              <SearchBox
                value={searchQuery}
                onChange={setSearchQuery}
                filterTags={filterTags}
                onRemoveTag={handleRemoveTagFilter}
                onClearAllTags={handleClearAllTags}
              />
              <DateFilter filters={filters} onChange={setFilters} />
              <FavoriteToggle value={favoritesOnly} onChange={setFavoritesOnly} />
            </div>

            {/* Right section: Action buttons */}
            <div className="header-right">
              <div className="header-button-group">
                <span className="header-clip-count">{total}</span>
                <span
                  className={`header-ws-dot ${wsConnected ? "ws-connected" : "ws-disconnected"}`}
                  title={wsConnected ? t("status.wsConnected") : t("status.wsDisconnected")}
                />
                <button className="header-button-group-item" onClick={openSettings} title={t("tooltip.settings")}>
                  <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z" />
                    <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115l.094-.319z" />
                  </svg>
                </button>
                <button className="header-button-group-item" onClick={refetch} title={t("tooltip.refresh")}>
                  <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
                    <path fillRule="evenodd" d="M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2v1z" />
                    <path d="M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466z" />
                  </svg>
                </button>
              </div>
            </div>
          </header>
        ) : (
          <>
            <header className="app-header" data-tauri-drag-region>
              <div className="app-title-group" data-tauri-drag-region>
                <svg className="app-icon" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg" data-tauri-drag-region>
                  <defs>
                    <linearGradient id="boardGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stopColor="#6366F1" />
                      <stop offset="100%" stopColor="#8B5CF6" />
                    </linearGradient>
                    <linearGradient id="clipGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                      <stop offset="0%" stopColor="#F1F5F9" />
                      <stop offset="30%" stopColor="#CBD5E1" />
                      <stop offset="70%" stopColor="#94A3B8" />
                      <stop offset="100%" stopColor="#64748B" />
                    </linearGradient>
                    <linearGradient id="paperGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                      <stop offset="0%" stopColor="#FFFFFF" />
                      <stop offset="100%" stopColor="#F8FAFC" />
                    </linearGradient>
                    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
                      <feDropShadow dx="0" dy="8" stdDeviation="16" floodColor="#1E1B4B" floodOpacity="0.35" />
                    </filter>
                    <filter id="innerDepth">
                      <feDropShadow dx="0" dy="2" stdDeviation="2" floodColor="#4338CA" floodOpacity="0.3" />
                    </filter>
                  </defs>
                  <g filter="url(#shadow)">
                    <rect x="96" y="80" width="320" height="400" rx="32" ry="32" fill="url(#boardGrad)" />
                    <rect x="128" y="140" width="256" height="310" rx="16" ry="16" fill="url(#paperGrad)" />
                    <g fill="#C7D2FE">
                      <rect x="160" y="180" width="180" height="14" rx="7" />
                      <rect x="160" y="215" width="140" height="14" rx="7" />
                      <rect x="160" y="250" width="192" height="14" rx="7" />
                      <rect x="160" y="285" width="120" height="14" rx="7" />
                      <rect x="160" y="320" width="160" height="14" rx="7" />
                    </g>
                    <g>
                      <rect x="186" y="48" width="140" height="72" rx="12" ry="12" fill="url(#clipGrad)" />
                      <g stroke="#64748B" strokeWidth="3" strokeLinecap="round">
                        <line x1="206" y1="60" x2="206" y2="108" />
                        <line x1="222" y1="60" x2="222" y2="108" />
                        <line x1="290" y1="60" x2="290" y2="108" />
                        <line x1="306" y1="60" x2="306" y2="108" />
                      </g>
                      <path d="M194 120 C194 136, 210 148, 230 148" stroke="#94A3B8" strokeWidth="10" strokeLinecap="round" fill="none" />
                      <path d="M318 120 C318 136, 302 148, 282 148" stroke="#94A3B8" strokeWidth="10" strokeLinecap="round" fill="none" />
                    </g>
                  </g>
                  <g filter="url(#innerDepth)">
                    <circle cx="368" cy="400" r="44" fill="#10B981" />
                    <path d="M346 400 L360 414 L390 384" stroke="#FFFFFF" strokeWidth="10" strokeLinecap="round" strokeLinejoin="round" fill="none" />
                  </g>
                </svg>
                <h1 className="app-title" data-tauri-drag-region>{t("app.title")}</h1>
              </div>
              {/* Window controls for Windows and Linux */}
              <div className="window-controls">
                <button className="window-control-button" onClick={openSettings} title={t("tooltip.settings")}>
                  &#9881;
                </button>
                <button className="window-control-button" onClick={refetch} title={t("tooltip.refresh")}>
                  ↻
                </button>
                <button
                  className="window-control-button window-minimize"
                  onClick={handleMinimize}
                  aria-label="Minimize"
                >
                  <svg width="10" height="1" viewBox="0 0 10 1">
                    <rect width="10" height="1" fill="currentColor" />
                  </svg>
                </button>
                <button
                  className="window-control-button window-maximize"
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
                  className="window-control-button window-close"
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
            </header>

            <div className="filters-bar">
              <SearchBox
                value={searchQuery}
                onChange={setSearchQuery}
                filterTags={filterTags}
                onRemoveTag={handleRemoveTagFilter}
                onClearAllTags={handleClearAllTags}
              />
              <DateFilter filters={filters} onChange={setFilters} />
              <FavoriteToggle value={favoritesOnly} onChange={setFavoritesOnly} />
            </div>

            <div className="status-bar">
              <span className="clip-count">{t("app.clips_count", { count: total })}</span>
              <span
                className={`ws-status ${wsConnected ? "ws-connected" : "ws-disconnected"}`}
                title={wsConnected ? t("status.wsConnected") : t("status.wsDisconnected")}
              >
                <span className="ws-status-dot"></span>
                <span className="ws-status-text">
                  {wsConnected ? t("status.wsConnected") : t("status.wsDisconnected")}
                </span>
              </span>
            </div>
          </>
        )}

        <main className="app-main">
          <ClipList
            clips={clips}
            loading={loading}
            loadingMore={loadingMore}
            error={error}
            hasMore={hasMore}
            onToggleFavorite={toggleFavorite}
            onLoadMore={loadMore}
            onClipUpdated={updateClipInList}
            onClipDeleted={deleteClipFromList}
            onTagClick={handleAddTagFilter}
          />
        </main>

        <SettingsDialog isOpen={isSettingsOpen} onClose={closeSettings} onThemeChange={updateTheme} />
      </div>
    </DropZone>
  );
}

export default App;
