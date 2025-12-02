import { useCallback, useEffect, useRef, useState, DragEvent } from "react";
import {
  useClips,
  useTheme,
  useSyntaxTheme,
  useI18n,
  useToast,
  useApi,
  SearchBox,
  DateFilter,
  FavoriteToggle,
  ClipList,
} from "@unwritten-codes/clipper-ui";
import { SettingsDialog, useSettingsDialog } from "./components/SettingsDialog";
import { useWebSocket, isSecureContext } from "./hooks/useWebSocket";

interface AppProps {
  /** Auth token for WebSocket authentication (when server requires auth) */
  authToken?: string;
}

function App({ authToken }: AppProps) {
  const { t } = useI18n();
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
  const { theme, updateTheme } = useTheme();
  const { syntaxTheme, setSyntaxTheme } = useSyntaxTheme();
  const { showToast } = useToast();
  const api = useApi();

  // Track if we've shown the connected toast (only show once per session)
  const hasShownConnectedToast = useRef(false);

  // File drop state
  const [isDragging, setIsDragging] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const dragCounter = useRef(0);

  // WebSocket for real-time updates (only enabled on HTTPS)
  const { isConnected, isSecure } = useWebSocket({
    onNewClip: useCallback((_id: string, _content: string, _tags: string[]) => {
      showToast(t("toast.newClip"), "info");
      refetch();
    }, [showToast, t, refetch]),
    onUpdatedClip: useCallback((_id: string) => {
      showToast(t("toast.clipUpdated"), "info");
      refetch();
    }, [showToast, t, refetch]),
    onDeletedClip: useCallback((_id: string) => {
      refetch();
    }, [refetch]),
    onClipsCleanedUp: useCallback((_ids: string[], count: number) => {
      showToast(t("toast.clipsCleanedUp").replace("{count}", String(count)), "info");
      refetch();
    }, [showToast, t, refetch]),
    onError: useCallback((_error: string) => {
      showToast(t("toast.serverError"), "error");
    }, [showToast, t]),
    onAuthError: useCallback((_error: string) => {
      showToast(t("toast.wsAuthFailed"), "error");
    }, [showToast, t]),
    enabled: isSecureContext(),
    token: authToken,
  });

  // Show connection status toast (only on first connect)
  useEffect(() => {
    if (isSecure && isConnected && !hasShownConnectedToast.current) {
      hasShownConnectedToast.current = true;
      showToast(t("toast.wsConnected"), "success");
    }
  }, [isSecure, isConnected, showToast, t]);

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

  // File drop handlers
  const handleDragEnter = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current++;
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragging(true);
    }
  }, []);

  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current--;
    if (dragCounter.current === 0) {
      setIsDragging(false);
    }
  }, []);

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(async (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    dragCounter.current = 0;

    const files = e.dataTransfer.files;
    if (files.length === 0) return;

    setIsUploading(true);
    try {
      for (const file of Array.from(files)) {
        await api.uploadFile(file);
      }
      showToast(t("toast.fileUploaded"), "success");
      refetch();
    } catch (err) {
      console.error("Upload failed:", err);
      showToast(t("toast.uploadFailed"), "error");
    } finally {
      setIsUploading(false);
    }
  }, [api, showToast, t, refetch]);

  // Send clipboard content to server
  const handleSendClipboard = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text || text.trim() === "") {
        showToast(t("toast.clipboardEmpty"), "info");
        return;
      }
      await api.createClip(text);
      showToast(t("toast.clipboardSent"), "success");
      refetch();
    } catch (err) {
      console.error("Failed to read clipboard:", err);
      showToast(t("toast.clipboardReadFailed"), "error");
    }
  }, [api, showToast, t, refetch]);

  return (
    <div
      className={`app ${isDragging ? "dragging" : ""}`}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Drop overlay */}
      {isDragging && (
        <div className="drop-overlay">
          <div className="drop-message">
            {t("fileDrop.hint")}
          </div>
        </div>
      )}

      {/* Upload indicator */}
      {isUploading && (
        <div className="upload-indicator">
          {t("fileDrop.uploading")}
        </div>
      )}

      <header className="app-header">
        <div className="app-title-group">
          <svg
            className="app-icon"
            viewBox="0 0 512 512"
            xmlns="http://www.w3.org/2000/svg"
          >
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
            </defs>
            <g>
              <rect
                x="96"
                y="80"
                width="320"
                height="400"
                rx="32"
                ry="32"
                fill="url(#boardGrad)"
              />
              <rect
                x="128"
                y="140"
                width="256"
                height="310"
                rx="16"
                ry="16"
                fill="url(#paperGrad)"
              />
              <g fill="#C7D2FE">
                <rect x="160" y="180" width="180" height="14" rx="7" />
                <rect x="160" y="215" width="140" height="14" rx="7" />
                <rect x="160" y="250" width="192" height="14" rx="7" />
                <rect x="160" y="285" width="120" height="14" rx="7" />
                <rect x="160" y="320" width="160" height="14" rx="7" />
              </g>
              <g>
                <rect
                  x="186"
                  y="48"
                  width="140"
                  height="72"
                  rx="12"
                  ry="12"
                  fill="url(#clipGrad)"
                />
                <g stroke="#64748B" strokeWidth="3" strokeLinecap="round">
                  <line x1="206" y1="60" x2="206" y2="108" />
                  <line x1="222" y1="60" x2="222" y2="108" />
                  <line x1="290" y1="60" x2="290" y2="108" />
                  <line x1="306" y1="60" x2="306" y2="108" />
                </g>
                <path
                  d="M194 120 C194 136, 210 148, 230 148"
                  stroke="#94A3B8"
                  strokeWidth="10"
                  strokeLinecap="round"
                  fill="none"
                />
                <path
                  d="M318 120 C318 136, 302 148, 282 148"
                  stroke="#94A3B8"
                  strokeWidth="10"
                  strokeLinecap="round"
                  fill="none"
                />
              </g>
            </g>
            <g>
              <circle cx="368" cy="400" r="44" fill="#10B981" />
              <path
                d="M346 400 L360 414 L390 384"
                stroke="#FFFFFF"
                strokeWidth="10"
                strokeLinecap="round"
                strokeLinejoin="round"
                fill="none"
              />
            </g>
          </svg>
          <h1 className="app-title">{t("app.title")}</h1>
        </div>
        <div className="header-right">
          <div className="header-links">
            <a
              href="https://clipper.unwritten.codes"
              target="_blank"
              rel="noopener noreferrer"
              className="header-link"
              title="Clipper Homepage"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8.707 1.5a1 1 0 0 0-1.414 0L.646 8.146a.5.5 0 0 0 .708.708L2 8.207V13.5A1.5 1.5 0 0 0 3.5 15h9a1.5 1.5 0 0 0 1.5-1.5V8.207l.646.647a.5.5 0 0 0 .708-.708L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293L8.707 1.5ZM13 7.207V13.5a.5.5 0 0 1-.5.5h-9a.5.5 0 0 1-.5-.5V7.207l5-5 5 5Z"/>
              </svg>
            </a>
            <a
              href="https://github.com/windoze/clipper"
              target="_blank"
              rel="noopener noreferrer"
              className="header-link"
              title="GitHub Repository"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
              </svg>
            </a>
          </div>
          <div className="header-button-group">
            <span className="header-clip-count">
              <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
                <path d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1v-1z" />
                <path d="M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5h3zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0h-3z" />
              </svg>
              {total}
            </span>
            <span
              className={`header-ws-dot ${isConnected ? "ws-connected" : isSecure ? "ws-disconnected" : "ws-unavailable"}`}
              title={isConnected ? t("status.wsConnected") : isSecure ? t("status.wsDisconnected") : t("status.wsUnavailable")}
            />
            <button
              className="header-button-group-item"
              onClick={handleSendClipboard}
              title={t("tooltip.sendClipboard")}
            >
              <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
                <path d="M.5 9.9a.5.5 0 0 1 .5.5v2.5a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-2.5a.5.5 0 0 1 1 0v2.5a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2v-2.5a.5.5 0 0 1 .5-.5z"/>
                <path d="M7.646 1.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1-.708.708L8.5 2.707V11.5a.5.5 0 0 1-1 0V2.707L5.354 4.854a.5.5 0 1 1-.708-.708l3-3z"/>
              </svg>
            </button>
            <button
              className="header-button-group-item"
              onClick={openSettings}
              title={t("tooltip.settings")}
            >
              <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z" />
                <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115l.094-.319z" />
              </svg>
            </button>
          </div>
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

      <SettingsDialog
        isOpen={isSettingsOpen}
        onClose={closeSettings}
        theme={theme}
        onThemeChange={updateTheme}
        syntaxTheme={syntaxTheme}
        onSyntaxThemeChange={setSyntaxTheme}
      />
    </div>
  );
}

export default App;
