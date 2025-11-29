import { useEffect, useRef, useCallback } from "react";
import { Clip } from "../types";
import { ClipEntry } from "./ClipEntry";
import { useI18n } from "../i18n";

interface ClipListProps {
  clips: Clip[];
  loading: boolean;
  loadingMore: boolean;
  error: string | null;
  hasMore: boolean;
  onToggleFavorite: (clip: Clip) => void;
  onLoadMore: () => void;
  onClipUpdated?: (updatedClip: Clip) => void;
  onClipDeleted?: (clipId: string) => void;
  onTagClick?: (tag: string) => void;
}

export function ClipList({
  clips,
  loading,
  loadingMore,
  error,
  hasMore,
  onToggleFavorite,
  onLoadMore,
  onClipUpdated,
  onClipDeleted,
  onTagClick,
}: ClipListProps) {
  const { t } = useI18n();
  const observerRef = useRef<IntersectionObserver | null>(null);

  // Use refs to track current values so the observer callback always has fresh values
  const hasMoreRef = useRef(hasMore);
  const loadingMoreRef = useRef(loadingMore);
  const onLoadMoreRef = useRef(onLoadMore);

  // Keep refs in sync with props
  useEffect(() => {
    hasMoreRef.current = hasMore;
  }, [hasMore]);

  useEffect(() => {
    loadingMoreRef.current = loadingMore;
  }, [loadingMore]);

  useEffect(() => {
    onLoadMoreRef.current = onLoadMore;
  }, [onLoadMore]);

  // Create the observer once
  useEffect(() => {
    observerRef.current = new IntersectionObserver(
      (entries) => {
        const [entry] = entries;
        if (entry.isIntersecting && hasMoreRef.current && !loadingMoreRef.current) {
          onLoadMoreRef.current();
        }
      },
      {
        root: null,
        rootMargin: "100px",
        threshold: 0,
      }
    );

    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect();
      }
    };
  }, []);

  // Callback ref to observe the trigger element when it mounts
  const loadMoreTriggerRef = useCallback((node: HTMLDivElement | null) => {
    if (observerRef.current) {
      observerRef.current.disconnect();
    }
    if (node && observerRef.current) {
      observerRef.current.observe(node);
    }
  }, []);

  if (loading) {
    return (
      <div className="clip-list-status">
        <div className="loading-spinner"></div>
        <span>{t("clipList.loading")}</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="clip-list-status error">
        <span>{t("clipList.error", { error })}</span>
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="clip-list-status empty">
        <span>{t("clipList.empty")}</span>
      </div>
    );
  }

  return (
    <div className="clip-list">
      {clips.map((clip) => (
        <ClipEntry
          key={clip.id}
          clip={clip}
          onToggleFavorite={onToggleFavorite}
          onClipUpdated={onClipUpdated}
          onClipDeleted={onClipDeleted}
          onTagClick={onTagClick}
        />
      ))}

      {/* Infinite scroll trigger */}
      <div ref={loadMoreTriggerRef} className="load-more-trigger">
        {loadingMore && (
          <div className="clip-list-status loading-more">
            <div className="loading-spinner small"></div>
            <span>{t("clipList.loadingMore")}</span>
          </div>
        )}
        {!hasMore && clips.length > 0 && (
          <div className="clip-list-status end-of-list">
            <span>{t("clipList.noMore")}</span>
          </div>
        )}
      </div>
    </div>
  );
}
