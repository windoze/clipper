import { useEffect, useRef, useCallback } from "react";
import { Clip, Tag } from "../types";
import { ClipEntry } from "./ClipEntry";
import { ConnectionError } from "./ConnectionError";
import { useI18n } from "../i18n";
import { useScrollAnchor } from "../hooks/useScrollAnchor";

interface ClipListProps {
  clips: Clip[];
  loading: boolean;
  loadingMore: boolean;
  error: string | null;
  hasMore: boolean;
  onToggleFavorite: (clip: Clip) => void;
  onLoadMore: () => void;
  onClipUpdated?: (updatedClip: Clip, onUpdated?: () => void) => void;
  onClipDeleted?: (clipId: string, onDeleted?: () => void) => void;
  /** Called before any clip modification to register the clip ID (for skipping WebSocket refetch) */
  onBeforeClipModified?: (clipId: string) => void;
  onTagClick?: (tag: string) => void;
  onSetStartDate?: (isoDate: string) => void;
  onSetEndDate?: (isoDate: string) => void;
  onRetry?: () => void;
  onOpenSettings?: () => void;
  showBundledServerReason?: boolean;
  onOpenUrl?: (url: string) => void;
  /** Function to search tags for autocomplete in edit dialog */
  onSearchTags?: (query: string) => Promise<Tag[]>;
}

// Helper to detect connection errors vs other errors
function isConnectionError(error: string): boolean {
  const connectionErrorPatterns = [
    // Generic network/connection patterns
    /fetch|network|connection|connect|refused|unreachable|timeout|econnrefused|enetunreach|ehostunreach|socket/i,
    /failed to fetch/i,
    /network error/i,
    /connection refused/i,
    /server is not running/i,
    /could not connect/i,
    // Rust/reqwest specific patterns
    /http request failed/i,
    /error sending request/i,
    /dns error/i,
    /no route to host/i,
    /connection reset/i,
    /broken pipe/i,
    /connection closed/i,
    /tcp connect error/i,
    /hyper::/i,
    /reqwest/i,
  ];
  return connectionErrorPatterns.some((pattern) => pattern.test(error));
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
  onBeforeClipModified,
  onTagClick,
  onSetStartDate,
  onSetEndDate,
  onRetry,
  onOpenSettings,
  showBundledServerReason = false,
  onOpenUrl,
  onSearchTags,
}: ClipListProps) {
  const { t } = useI18n();
  const observerRef = useRef<IntersectionObserver | null>(null);

  // Use scroll anchor hook for maintaining scroll position during delete/edit
  const { captureAnchor, restoreScroll, pendingAnchorRef } = useScrollAnchor();

  // Handle onBeforeClipModified: capture anchor AND call parent's onBeforeClipModified (to register clip ID)
  const handleBeforeClipModified = useCallback((clipId: string) => {
    console.log("[ClipList] handleBeforeClipModified called with clipId:", clipId);
    // Capture anchor BEFORE any API call or state change
    const anchor = captureAnchor(clips, clipId);
    console.log("[ClipList] Captured anchor:", anchor);
    // Store anchor for later use in delete/update handlers
    pendingAnchorRef.current = anchor;
    // Call parent's onBeforeClipModified to register the clip ID for WebSocket skip
    onBeforeClipModified?.(clipId);
  }, [clips, captureAnchor, pendingAnchorRef, onBeforeClipModified]);

  // Wrap onClipDeleted to handle scroll anchoring
  const handleClipDeleted = useCallback((clipId: string) => {
    console.log("[ClipList] handleClipDeleted called with clipId:", clipId);
    // Use the anchor captured in handleBeforeClipModified
    const anchor = pendingAnchorRef.current;
    pendingAnchorRef.current = null;
    console.log("[ClipList] Using pending anchor:", anchor);

    // Call the original delete callback with a restore callback
    // The restore callback will be called after state update completes
    console.log("[ClipList] Calling onClipDeleted, exists:", !!onClipDeleted);
    onClipDeleted?.(clipId, anchor ? () => {
      console.log("[ClipList] Restore callback invoked");
      restoreScroll(anchor);
    } : undefined);
  }, [onClipDeleted, pendingAnchorRef, restoreScroll]);

  // Wrap onClipUpdated to handle scroll anchoring
  const handleClipUpdated = useCallback((updatedClip: Clip) => {
    console.log("[ClipList] handleClipUpdated called with clipId:", updatedClip.id);
    // Use the anchor captured in handleBeforeClipModified
    const anchor = pendingAnchorRef.current;
    pendingAnchorRef.current = null;
    console.log("[ClipList] Using pending anchor:", anchor);

    // Call the original update callback with a restore callback
    // The restore callback will be called after state update completes
    console.log("[ClipList] Calling onClipUpdated, exists:", !!onClipUpdated);
    onClipUpdated?.(updatedClip, anchor ? () => {
      console.log("[ClipList] Restore callback invoked");
      restoreScroll(anchor);
    } : undefined);
  }, [onClipUpdated, pendingAnchorRef, restoreScroll]);

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
    // Show friendly connection error page for connection-related errors
    if (isConnectionError(error) && onRetry) {
      return (
        <ConnectionError
          error={error}
          onRetry={onRetry}
          onOpenSettings={onOpenSettings}
          showBundledServerReason={showBundledServerReason}
        />
      );
    }
    // Fall back to simple error message for other errors
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
          onClipUpdated={handleClipUpdated}
          onClipDeleted={handleClipDeleted}
          onBeforeClipModified={handleBeforeClipModified}
          onTagClick={onTagClick}
          onSetStartDate={onSetStartDate}
          onSetEndDate={onSetEndDate}
          onSearchTags={onSearchTags}
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
            <button
              className="end-of-list-link"
              onClick={() => onOpenUrl?.("https://clipper.unwritten.codes")}
              title="Home"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8.354 1.146a.5.5 0 0 0-.708 0l-6 6A.5.5 0 0 0 1.5 7.5v7a.5.5 0 0 0 .5.5h4.5a.5.5 0 0 0 .5-.5v-4h2v4a.5.5 0 0 0 .5.5H14a.5.5 0 0 0 .5-.5v-7a.5.5 0 0 0-.146-.354L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293L8.354 1.146zM2.5 14V7.707l5.5-5.5 5.5 5.5V14H10v-4a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5v4H2.5z"/>
              </svg>
            </button>
            <span>{t("clipList.noMore")}</span>
            <button
              className="end-of-list-link"
              onClick={() => onOpenUrl?.("https://github.com/windoze/clipper")}
              title="GitHub"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
              </svg>
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
