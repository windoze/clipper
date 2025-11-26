import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Clip, PagedResult, SearchFilters, FAVORITE_TAG } from "../types";

interface UseClipsState {
  clips: Clip[];
  loading: boolean;
  error: string | null;
  total: number;
  page: number;
  totalPages: number;
}

interface UseClipsReturn extends UseClipsState {
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  filters: SearchFilters;
  setFilters: (filters: SearchFilters) => void;
  favoritesOnly: boolean;
  setFavoritesOnly: (value: boolean) => void;
  refetch: () => void;
  toggleFavorite: (clip: Clip) => Promise<void>;
}

const PAGE_SIZE = 50;

export function useClips(): UseClipsReturn {
  const [state, setState] = useState<UseClipsState>({
    clips: [],
    loading: true,
    error: null,
    total: 0,
    page: 1,
    totalPages: 0,
  });

  const [searchQuery, setSearchQuery] = useState("");
  const [filters, setFilters] = useState<SearchFilters>({});
  const [favoritesOnly, setFavoritesOnly] = useState(false);

  const fetchClips = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));

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
        result = await invoke<PagedResult>("search_clips", {
          query: searchQuery,
          filters: effectiveFilters,
          page: 1,
          pageSize: PAGE_SIZE,
        });
      } else {
        result = await invoke<PagedResult>("list_clips", {
          filters: effectiveFilters,
          page: 1,
          pageSize: PAGE_SIZE,
        });
      }

      setState({
        clips: result.items,
        loading: false,
        error: null,
        total: result.total,
        page: result.page,
        totalPages: result.total_pages,
      });
    } catch (err) {
      setState((prev) => ({
        ...prev,
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      }));
    }
  }, [searchQuery, filters, favoritesOnly]);

  const toggleFavorite = useCallback(async (clip: Clip) => {
    const isFav = clip.tags.includes(FAVORITE_TAG);
    const newTags = isFav
      ? clip.tags.filter((t) => t !== FAVORITE_TAG)
      : [...clip.tags, FAVORITE_TAG];

    try {
      await invoke("update_clip", {
        id: clip.id,
        tags: newTags,
        additionalNotes: null,
      });

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
  }, []);

  // Fetch clips when filters change
  useEffect(() => {
    fetchClips();
  }, [fetchClips]);

  // Listen for backend events
  useEffect(() => {
    const unlistenNewClip = listen("new-clip", () => {
      fetchClips();
    });

    const unlistenClipCreated = listen("clip-created", () => {
      fetchClips();
    });

    const unlistenClipUpdated = listen("clip-updated", () => {
      fetchClips();
    });

    const unlistenClipDeleted = listen("clip-deleted", () => {
      fetchClips();
    });

    return () => {
      unlistenNewClip.then((fn) => fn());
      unlistenClipCreated.then((fn) => fn());
      unlistenClipUpdated.then((fn) => fn());
      unlistenClipDeleted.then((fn) => fn());
    };
  }, [fetchClips]);

  return {
    ...state,
    searchQuery,
    setSearchQuery,
    filters,
    setFilters,
    favoritesOnly,
    setFavoritesOnly,
    refetch: fetchClips,
    toggleFavorite,
  };
}
