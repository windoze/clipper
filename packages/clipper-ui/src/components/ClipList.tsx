import { useEffect, useRef, useCallback, useState } from "react";
import { Clip, Tag } from "../types";
import { ClipEntry } from "./ClipEntry";
import { ConnectionError } from "./ConnectionError";
import { useI18n } from "../i18n";
import { useScrollAnchor } from "../hooks/useScrollAnchor";
import { useKeyboardNavigation, ClipButtonAction } from "../hooks/useKeyboardNavigation";

// Maximum number of skeleton placeholders to render at once
// Higher count helps prevent blank pages during fast scrolling
const MAX_SKELETON_COUNT = 50;

interface ClipListProps {
  clips: Clip[];
  loading: boolean;
  loadingMore: boolean;
  error: string | null;
  hasMore: boolean;
  /** Total number of clips (for scrollbar sizing) */
  total: number;
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
  /** Reference to the search input for keyboard navigation focus */
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  /** Whether keyboard navigation is enabled (default: true) */
  keyboardNavigationEnabled?: boolean;
  /** Reference to the scroll container (for external scroll control) */
  scrollContainerRef?: React.RefObject<HTMLDivElement | null>;
}

// Helper to detect connection errors vs other errors
function isConnectionError(error: string): boolean {
  const connectionErrorPatterns = [
    /fetch|network|connection|connect|refused|unreachable|timeout|econnrefused|enetunreach|ehostunreach|socket/i,
    /failed to fetch/i,
    /network error/i,
    /connection refused/i,
    /server is not running/i,
    /could not connect/i,
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

// Button actions in order: copy, share, favorite, notes, expand, delete
const BUTTON_ACTIONS: ClipButtonAction[] = ["copy", "share", "favorite", "notes", "expand", "delete"];
const BUTTON_COUNT = BUTTON_ACTIONS.length;


// Skeleton placeholder component for loading state
function SkeletonPlaceholder() {
  return (
    <div className="clip-entry-placeholder">
      <div className="skeleton-header">
        <div className="skeleton-avatar"></div>
        <div className="skeleton-meta">
          <div className="skeleton-line skeleton-line-shorter"></div>
          <div className="skeleton-line skeleton-line-short"></div>
        </div>
      </div>
      <div className="skeleton-content">
        <div className="skeleton-line"></div>
        <div className="skeleton-line skeleton-line-medium"></div>
        <div className="skeleton-line"></div>
        <div className="skeleton-line skeleton-line-short"></div>
        <div className="skeleton-line skeleton-line-medium"></div>
        <div className="skeleton-line skeleton-line-shorter"></div>
      </div>
      <div className="placeholder-shimmer"></div>
    </div>
  );
}

export function ClipList({
  clips,
  loading,
  loadingMore,
  error,
  hasMore,
  total,
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
  searchInputRef,
  keyboardNavigationEnabled = true,
  scrollContainerRef,
}: ClipListProps) {
  const { t } = useI18n();
  const internalScrollRef = useRef<HTMLDivElement>(null);
  const scrollRef = scrollContainerRef || internalScrollRef;
  const loadMoreTriggerRef = useRef<HTMLDivElement>(null);

  // Track which clip is expanded (for keyboard expand toggle)
  const [expandedClipIds, setExpandedClipIds] = useState<Set<string>>(new Set());

  // Use scroll anchor hook for maintaining scroll position during delete/edit
  const { captureAnchor, restoreScroll, pendingAnchorRef } = useScrollAnchor();

  // Callbacks for keyboard navigation
  const handleExpandToggle = useCallback((clipId: string) => {
    setExpandedClipIds(prev => {
      const next = new Set(prev);
      if (next.has(clipId)) {
        next.delete(clipId);
      } else {
        next.add(clipId);
      }
      return next;
    });
  }, []);

  // Track which clip to focus after deletion
  const nextFocusAfterDeleteRef = useRef<string | null>(null);

  const handleDeleteRequest = useCallback((clipId: string) => {
    // Determine which clip to focus after deletion: next clip, or previous if at end
    const currentIndex = clips.findIndex(c => c.id === clipId);
    if (currentIndex !== -1) {
      if (currentIndex < clips.length - 1) {
        // Focus the next clip (which will move up to current position after deletion)
        nextFocusAfterDeleteRef.current = clips[currentIndex + 1].id;
      } else if (currentIndex > 0) {
        // At the end, focus the previous clip
        nextFocusAfterDeleteRef.current = clips[currentIndex - 1].id;
      } else {
        // Only one clip, nothing to focus
        nextFocusAfterDeleteRef.current = null;
      }
    }

    const clipEntry = document.querySelector(`[data-clip-id="${clipId}"]`);
    if (clipEntry) {
      clipEntry.dispatchEvent(new CustomEvent("keyboard-delete-request", { bubbles: false }));
    }
  }, [clips]);

  const handleButtonActivate = useCallback((clipId: string, buttonIndex: number) => {
    const action = BUTTON_ACTIONS[buttonIndex];
    const clipEntry = document.querySelector(`[data-clip-id="${clipId}"]`);
    if (clipEntry) {
      clipEntry.dispatchEvent(new CustomEvent("keyboard-button-activate", {
        bubbles: false,
        detail: { action, buttonIndex },
      }));
    }
  }, []);

  // Keyboard navigation hook
  const {
    focusedClipId,
    focusedButtonIndex,
    keyboardNavigating,
    setFocusedClipId,
    setFocusedButtonIndex,
    handleKeyDown,
  } = useKeyboardNavigation({
    clips,
    buttonCount: BUTTON_COUNT,
    onExpandToggle: handleExpandToggle,
    onDelete: handleDeleteRequest,
    onButtonActivate: handleButtonActivate,
    searchInputRef,
    enabled: keyboardNavigationEnabled,
    hasMore,
    onLoadMore,
    containerRef: scrollRef,
  });

  // Handle click activation of a clip (without scrolling)
  const handleClipActivate = useCallback((clipId: string) => {
    setFocusedClipId(clipId);
    setFocusedButtonIndex(-1);
  }, [setFocusedClipId, setFocusedButtonIndex]);

  // Global keyboard listener for navigation
  useEffect(() => {
    if (!keyboardNavigationEnabled) return;
    const handler = (e: KeyboardEvent) => handleKeyDown(e);
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [keyboardNavigationEnabled, handleKeyDown]);

  // Handle onBeforeClipModified
  const handleBeforeClipModified = useCallback((clipId: string) => {
    const anchor = captureAnchor(clips, clipId);
    pendingAnchorRef.current = anchor;
    onBeforeClipModified?.(clipId);
  }, [clips, captureAnchor, pendingAnchorRef, onBeforeClipModified]);

  // Wrap onClipDeleted
  const handleClipDeleted = useCallback((clipId: string) => {
    const anchor = pendingAnchorRef.current;
    pendingAnchorRef.current = null;

    // Focus the next clip after deletion
    const nextFocusId = nextFocusAfterDeleteRef.current;
    nextFocusAfterDeleteRef.current = null;
    if (nextFocusId) {
      setFocusedClipId(nextFocusId);
      setFocusedButtonIndex(-1);
    }

    onClipDeleted?.(clipId, anchor ? () => restoreScroll(anchor) : undefined);
  }, [onClipDeleted, pendingAnchorRef, restoreScroll, setFocusedClipId, setFocusedButtonIndex]);

  // Wrap onClipUpdated
  const handleClipUpdated = useCallback((updatedClip: Clip) => {
    const anchor = pendingAnchorRef.current;
    pendingAnchorRef.current = null;
    onClipUpdated?.(updatedClip, anchor ? () => restoreScroll(anchor) : undefined);
  }, [onClipUpdated, pendingAnchorRef, restoreScroll]);

  // Refs to track values for load-more logic
  const hasMoreRef = useRef(hasMore);
  const loadingMoreRef = useRef(loadingMore);
  const onLoadMoreRef = useRef(onLoadMore);

  useEffect(() => {
    hasMoreRef.current = hasMore;
    loadingMoreRef.current = loadingMore;
    onLoadMoreRef.current = onLoadMore;
  });

  // Callback ref to capture scroll element
  const scrollCallbackRef = useCallback((node: HTMLDivElement | null) => {
    if (scrollContainerRef) {
      (scrollContainerRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
    }
    (internalScrollRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
  }, [scrollContainerRef]);

  // IntersectionObserver for infinite scroll - triggers when load-more element is visible
  useEffect(() => {
    const trigger = loadMoreTriggerRef.current;
    const container = scrollRef.current;
    if (!trigger || !container) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const entry = entries[0];
        if (entry.isIntersecting && hasMoreRef.current && !loadingMoreRef.current) {
          onLoadMoreRef.current();
        }
      },
      {
        root: container,
        rootMargin: "800px", // Trigger early to stay ahead of fast scrolling
        threshold: 0,
      }
    );

    observer.observe(trigger);
    return () => observer.disconnect();
  }, [scrollRef, clips.length]); // Re-observe when clips change

  // Background prefetch: keep loading items until all are loaded
  // This ensures data is ready before user scrolls to it
  useEffect(() => {
    // Only prefetch when not currently loading and there's more to load
    if (loadingMore || !hasMore || loading) return;

    // Start prefetch after a short delay to avoid blocking initial render
    const prefetchTimer = setTimeout(() => {
      if (hasMoreRef.current && !loadingMoreRef.current) {
        onLoadMoreRef.current();
      }
    }, 100); // Small delay between fetches

    return () => clearTimeout(prefetchTimer);
  }, [loadingMore, hasMore, loading, clips.length]);

  // Calculate number of skeleton placeholders to show for unloaded items
  const loadedCount = clips.length;
  const unloadedCount = Math.max(0, total - loadedCount);
  // Limit skeleton count for performance, but show at least a few to indicate more items
  const skeletonCount = Math.min(unloadedCount, MAX_SKELETON_COUNT);

  if (loading) {
    return (
      <div className="clip-list-status">
        <div className="loading-spinner"></div>
        <span>{t("clipList.loading")}</span>
      </div>
    );
  }

  if (error) {
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
    <div
      ref={scrollCallbackRef}
      className={`clip-list-container${keyboardNavigating ? " keyboard-navigating" : ""}`}
    >
      {/* Render all loaded clips - they stay in DOM and don't re-render during scroll */}
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
            isFocused={focusedClipId === clip.id}
            focusedButtonIndex={focusedClipId === clip.id ? focusedButtonIndex : -1}
            isExpandedByKeyboard={expandedClipIds.has(clip.id)}
            onKeyboardExpandChange={(expanded) => {
              setExpandedClipIds(prev => {
                const next = new Set(prev);
                if (expanded) {
                  next.add(clip.id);
                } else {
                  next.delete(clip.id);
                }
                return next;
              });
            }}
            onActivate={handleClipActivate}
          />
        ))}
      </div>

      {/* Load more trigger - IntersectionObserver watches this */}
      <div ref={loadMoreTriggerRef} className="clip-list-load-trigger" />

      {/* Skeleton placeholders for unloaded items */}
      {skeletonCount > 0 && (
        <div className="clip-list-skeletons">
          {Array.from({ length: skeletonCount }, (_, i) => (
            <SkeletonPlaceholder key={`skeleton-${i}`} />
          ))}
        </div>
      )}

      {/* End of list */}
      {!hasMore && !loadingMore && (
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
  );
}
