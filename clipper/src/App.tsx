import { useClips } from "./hooks/useClips";
import { SearchBox } from "./components/SearchBox";
import { DateFilter } from "./components/DateFilter";
import { FavoriteToggle } from "./components/FavoriteToggle";
import { ClipList } from "./components/ClipList";
import { DropZone } from "./components/DropZone";
import "./App.css";

function App() {
  const {
    clips,
    loading,
    error,
    total,
    searchQuery,
    setSearchQuery,
    filters,
    setFilters,
    favoritesOnly,
    setFavoritesOnly,
    refetch,
    toggleFavorite,
  } = useClips();

  return (
    <DropZone>
      <div className="app">
        <header className="app-header">
          <h1 className="app-title">Clipper</h1>
          <button className="refresh-button" onClick={refetch} title="Refresh">
            ↻
          </button>
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
            error={error}
            onToggleFavorite={toggleFavorite}
          />
        </main>
      </div>
    </DropZone>
  );
}

export default App;
