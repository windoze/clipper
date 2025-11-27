import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useClips } from "./hooks/useClips";
import { useTheme } from "./hooks/useTheme";
import { useI18n } from "./i18n";
import { SearchBox } from "./components/SearchBox";
import { DateFilter } from "./components/DateFilter";
import { FavoriteToggle } from "./components/FavoriteToggle";
import { ClipList } from "./components/ClipList";
import { DropZone } from "./components/DropZone";
import { SettingsDialog, useSettingsDialog } from "./components/SettingsDialog";
import "./App.css";

function App() {
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
  const { updateTheme } = useTheme();

  // Listen for data-cleared and server-switched events to refresh clips
  useEffect(() => {
    const unlistenDataCleared = listen("data-cleared", () => {
      refetch();
    });

    const unlistenServerSwitched = listen("server-switched", () => {
      refetch();
    });

    return () => {
      unlistenDataCleared.then((fn) => fn());
      unlistenServerSwitched.then((fn) => fn());
    };
  }, [refetch]);

  return (
    <DropZone>
      <div className="app">
        <header className="app-header">
          <div className="app-title-group">
            <img src="/clipper-icon.svg" alt={t("app.title")} className="app-icon" />
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
