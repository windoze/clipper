import { useClips } from "./hooks/useClips";
import { useTheme } from "./hooks/useTheme";
import { SearchBox } from "./components/SearchBox";
import { DateFilter } from "./components/DateFilter";
import { FavoriteToggle } from "./components/FavoriteToggle";
import { ClipList } from "./components/ClipList";
import { DropZone } from "./components/DropZone";
import { SettingsDialog, useSettingsDialog } from "./components/SettingsDialog";
import "./App.css";

function App() {
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
  } = useClips();

  const { isOpen: isSettingsOpen, open: openSettings, close: closeSettings } = useSettingsDialog();
  const { updateTheme } = useTheme();

  return (
    <DropZone>
      <div className="app">
        <header className="app-header">
          <h1 className="app-title">Clipper</h1>
          <div className="header-buttons">
            <button className="settings-button" onClick={openSettings} title="Settings">
              &#9881;
            </button>
            <button className="refresh-button" onClick={refetch} title="Refresh">
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
          <span className="clip-count">{total} clip(s)</span>
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
          />
        </main>

        <SettingsDialog isOpen={isSettingsOpen} onClose={closeSettings} onThemeChange={updateTheme} />
      </div>
    </DropZone>
  );
}

export default App;
