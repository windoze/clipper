import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useClips } from "./hooks/useClips";
import { useTheme } from "./hooks/useTheme";
import { useI18n } from "./i18n";
import { useToast } from "./components/Toast";
import { TitleBar } from "./components/TitleBar";
import { SearchBox } from "./components/SearchBox";
import { DateFilter } from "./components/DateFilter";
import { FavoriteToggle } from "./components/FavoriteToggle";
import { ClipList } from "./components/ClipList";
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

  // Listen for data-cleared and server-switched events to refresh clips
  useEffect(() => {
    const unlistenDataCleared = listen("data-cleared", () => {
      refetch();
    });

    const unlistenServerSwitched = listen("server-switched", () => {
      refetch();
    });

    // Listen for new clips from WebSocket (from server)
    const unlistenNewClip = listen("new-clip", () => {
      showToast(t("toast.clipReceived"));
    });

    return () => {
      unlistenDataCleared.then((fn) => fn());
      unlistenServerSwitched.then((fn) => fn());
      unlistenNewClip.then((fn) => fn());
    };
  }, [refetch, showToast, t]);

  return (
    <DropZone>
      <div className={`app ${os}`}>
        <TitleBar />
        <header className="app-header" data-tauri-drag-region={os === "macos" ? true : undefined}>
          <div className="app-title-group">
            <svg className="app-icon" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
              <defs>
                <linearGradient id="boardGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#6366F1"/>
                  <stop offset="100%" stopColor="#8B5CF6"/>
                </linearGradient>
                <linearGradient id="clipGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" stopColor="#F1F5F9"/>
                  <stop offset="30%" stopColor="#CBD5E1"/>
                  <stop offset="70%" stopColor="#94A3B8"/>
                  <stop offset="100%" stopColor="#64748B"/>
                </linearGradient>
                <linearGradient id="paperGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" stopColor="#FFFFFF"/>
                  <stop offset="100%" stopColor="#F8FAFC"/>
                </linearGradient>
                <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
                  <feDropShadow dx="0" dy="8" stdDeviation="16" floodColor="#1E1B4B" floodOpacity="0.35"/>
                </filter>
                <filter id="innerDepth">
                  <feDropShadow dx="0" dy="2" stdDeviation="2" floodColor="#4338CA" floodOpacity="0.3"/>
                </filter>
              </defs>
              <g filter="url(#shadow)">
                <rect x="96" y="80" width="320" height="400" rx="32" ry="32" fill="url(#boardGrad)"/>
                <rect x="128" y="140" width="256" height="310" rx="16" ry="16" fill="url(#paperGrad)"/>
                <g fill="#C7D2FE">
                  <rect x="160" y="180" width="180" height="14" rx="7"/>
                  <rect x="160" y="215" width="140" height="14" rx="7"/>
                  <rect x="160" y="250" width="192" height="14" rx="7"/>
                  <rect x="160" y="285" width="120" height="14" rx="7"/>
                  <rect x="160" y="320" width="160" height="14" rx="7"/>
                </g>
                <g>
                  <rect x="186" y="48" width="140" height="72" rx="12" ry="12" fill="url(#clipGrad)"/>
                  <g stroke="#64748B" strokeWidth="3" strokeLinecap="round">
                    <line x1="206" y1="60" x2="206" y2="108"/>
                    <line x1="222" y1="60" x2="222" y2="108"/>
                    <line x1="290" y1="60" x2="290" y2="108"/>
                    <line x1="306" y1="60" x2="306" y2="108"/>
                  </g>
                  <path d="M194 120 C194 136, 210 148, 230 148" stroke="#94A3B8" strokeWidth="10" strokeLinecap="round" fill="none"/>
                  <path d="M318 120 C318 136, 302 148, 282 148" stroke="#94A3B8" strokeWidth="10" strokeLinecap="round" fill="none"/>
                </g>
              </g>
              <g filter="url(#innerDepth)">
                <circle cx="368" cy="400" r="44" fill="#10B981"/>
                <path d="M346 400 L360 414 L390 384" stroke="#FFFFFF" strokeWidth="10" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
              </g>
            </svg>
            <h1 className="app-title">{t("app.title")}</h1>
          </div>
          <div className="header-buttons">
            <button className="settings-button" onClick={openSettings} title={t("tooltip.settings")}>
              &#9881;
            </button>
            <button className="refresh-button" onClick={refetch} title={t("tooltip.refresh")}>
              ↻
            </button>
          </div>
        </header>

        <div className="filters-bar">
          <SearchBox value={searchQuery} onChange={setSearchQuery} />
          <DateFilter filters={filters} onChange={setFilters} />
          <FavoriteToggle value={favoritesOnly} onChange={setFavoritesOnly} />
        </div>

        <div className="status-bar">
          <span className="clip-count">{t("app.clips_count", { count: total })}</span>
        </div>

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
          />
        </main>

        <SettingsDialog isOpen={isSettingsOpen} onClose={closeSettings} onThemeChange={updateTheme} />
      </div>
    </DropZone>
  );
}

export default App;
