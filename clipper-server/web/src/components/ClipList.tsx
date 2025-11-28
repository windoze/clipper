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
}: ClipListProps) {
  const { t } = useI18n();
  const observerRef = useRef<IntersectionObserver | null>(null);
  const loadMoreTriggerRef = useRef<HTMLDivElement | null>(null);

  // Set up intersection observer for infinite scroll
  const setupObserver = useCallback(() => {
    if (observerRef.current) {
      observerRef.current.disconnect();
    }

    observerRef.current = new IntersectionObserver(
      (entries) => {
        const [entry] = entries;
        if (entry.isIntersecting && hasMore && !loadingMore) {
          onLoadMore();
        }
      },
      {
        root: null,
        rootMargin: "100px",
        threshold: 0,
      }
    );

    if (loadMoreTriggerRef.current) {
      observerRef.current.observe(loadMoreTriggerRef.current);
    }
  }, [hasMore, loadingMore, onLoadMore]);

  useEffect(() => {
    setupObserver();
    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect();
      }
    };
  }, [setupObserver]);

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
