import { useState, useEffect, useCallback, useRef } from "react";
import { Clip, PagedResult, SearchFilters, FAVORITE_TAG } from "../types";
import { useApi } from "../api";

interface UseClipsState {
  clips: Clip[];
  loading: boolean;
  loadingMore: boolean;
  error: string | null;
  total: number;
  page: number;
  totalPages: number;
  hasMore: boolean;
}

interface UseClipsReturn extends UseClipsState {
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  filters: SearchFilters;
  setFilters: (
    filters: SearchFilters | ((prev: SearchFilters) => SearchFilters)
  ) => void;
  favoritesOnly: boolean;
  setFavoritesOnly: (value: boolean) => void;
  refetch: () => void;
  loadMore: () => void;
  toggleFavorite: (clip: Clip) => Promise<void>;
  updateClipInList: (updatedClip: Clip) => void;
  deleteClipFromList: (clipId: string) => void;
}

const PAGE_SIZE = 20;

export function useClips(): UseClipsReturn {
  const api = useApi();
  const [state, setState] = useState<UseClipsState>({
    clips: [],
    loading: true,
    loadingMore: false,
    error: null,
    total: 0,
    page: 1,
    totalPages: 0,
    hasMore: false,
  });

  const [searchQuery, setSearchQuery] = useState("");
  const [filters, setFilters] = useState<SearchFilters>({});
  const [favoritesOnly, setFavoritesOnly] = useState(false);

  // Track current filters to prevent race conditions
  const currentFiltersRef = useRef({ searchQuery, filters, favoritesOnly });

  // Update ref when filters change
  useEffect(() => {
    currentFiltersRef.current = { searchQuery, filters, favoritesOnly };
  }, [searchQuery, filters, favoritesOnly]);

  const fetchClips = useCallback(
    async (page: number = 1, append: boolean = false) => {
      // Capture the current filter state at the start of this fetch
      const fetchFilters = { searchQuery, filters, favoritesOnly };

      if (append) {
        setState((prev) => ({ ...prev, loadingMore: true }));
      } else {
        setState((prev) => ({ ...prev, loading: true, error: null }));
      }

      try {
        const effectiveFilters: SearchFilters = { ...filters };
        if (favoritesOnly) {
          effectiveFilters.tags = [
            ...(effectiveFilters.tags || []),
            FAVORITE_TAG,
          ];
        }

        let result: PagedResult;

        if (searchQuery.trim()) {
          result = await api.searchClips(
            searchQuery,
            effectiveFilters,
            page,
            PAGE_SIZE
          );
        } else {
          result = await api.listClips(effectiveFilters, page, PAGE_SIZE);
        }

        // Check if filters changed during the fetch by comparing against the ref
        const current = currentFiltersRef.current;
        if (
          current.searchQuery !== fetchFilters.searchQuery ||
          current.favoritesOnly !== fetchFilters.favoritesOnly ||
          JSON.stringify(current.filters) !== JSON.stringify(fetchFilters.filters)
        ) {
          // Filters changed during the fetch, ignore this stale result
          return;
        }

        setState((prev) => ({
          clips: append ? [...prev.clips, ...result.items] : result.items,
          loading: false,
          loadingMore: false,
          error: null,
          total: result.total,
          page: result.page,
          totalPages: result.total_pages,
          hasMore: result.page < result.total_pages,
        }));
      } catch (err) {
        setState((prev) => ({
          ...prev,
          loading: false,
          loadingMore: false,
          error: err instanceof Error ? err.message : String(err),
        }));
      }
    },
    [searchQuery, filters, favoritesOnly, api]
  );

  const loadMore = useCallback(() => {
    if (state.loadingMore || !state.hasMore) return;
    fetchClips(state.page + 1, true);
  }, [fetchClips, state.loadingMore, state.hasMore, state.page]);

  const refetch = useCallback(() => {
    fetchClips(1, false);
  }, [fetchClips]);

  const toggleFavorite = useCallback(
    async (clip: Clip) => {
      const isFav = clip.tags.includes(FAVORITE_TAG);
      const newTags = isFav
        ? clip.tags.filter((t) => t !== FAVORITE_TAG)
        : [...clip.tags, FAVORITE_TAG];

      try {
        await api.updateClip(clip.id, newTags);

        // Update local state immediately
        setState((prev) => ({
          ...prev,
          clips: prev.clips.map((c) =>
            c.id === clip.id ? { ...c, tags: newTags } : c
          ),
        }));
      } catch (err) {
        console.error("Failed to toggle favorite:", err);
      }
    },
    [api]
  );

  const updateClipInList = useCallback((updatedClip: Clip) => {
    setState((prev) => ({
      ...prev,
      clips: prev.clips.map((c) =>
        c.id === updatedClip.id ? updatedClip : c
      ),
    }));
  }, []);

  const deleteClipFromList = useCallback((clipId: string) => {
    setState((prev) => ({
      ...prev,
      clips: prev.clips.filter((c) => c.id !== clipId),
      total: prev.total - 1,
    }));
  }, []);

  // Fetch clips when filters change (reset to page 1)
  useEffect(() => {
    fetchClips(1, false);
  }, [fetchClips]);

  return {
    ...state,
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
  };
}
