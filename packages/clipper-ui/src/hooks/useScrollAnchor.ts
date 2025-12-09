import { useRef, useCallback } from "react";

/**
 * Represents a scroll anchor point that can be used to restore scroll position
 * after a list modification.
 */
export interface ScrollAnchor {
  /** The clip ID of the anchor element */
  clipId: string;
  /** The index of the anchor element in the clip list */
  index: number;
  /** The offset from the top of the viewport to the anchor element */
  offsetFromViewport: number;
}

/**
 * Gets the scroll container element for the clip list.
 */
function getScrollContainer(): Element | null {
  // The scrollable container is .app-main which has overflow-y: auto
  return document.querySelector(".app-main") || document.scrollingElement || document.documentElement;
}

/**
 * Gets the clip element by its ID.
 */
function getClipElement(clipId: string): HTMLElement | null {
  return document.querySelector(`[data-clip-id="${clipId}"]`);
}

/**
 * Hook for managing scroll anchors during list modifications.
 *
 * This hook provides functions to:
 * 1. Capture an anchor point before a list modification (delete/edit)
 * 2. Restore the scroll position after the modification using the anchor
 *
 * The anchor is typically the clip above the one being modified, so the user
 * sees the same clips in the same positions after the modification.
 */
export function useScrollAnchor() {
  // Store the captured anchor that's pending restoration
  const pendingAnchorRef = useRef<ScrollAnchor | null>(null);

  /**
   * Captures an anchor point before a list modification.
   *
   * @param clipsList - Current array of clips
   * @param targetClipId - The ID of the clip being modified (deleted/edited)
   * @returns The captured anchor, or null if no valid anchor exists
   */
  const captureAnchor = useCallback((clipsList: { id: string }[], targetClipId: string): ScrollAnchor | null => {
    // Find the index of the target clip
    const targetIndex = clipsList.findIndex(c => c.id === targetClipId);
    if (targetIndex < 0) {
      return null;
    }

    // The anchor is the clip above the target, or the target itself if it's the first
    // For deletion: if deleting clip[2], anchor is clip[1]
    // For edit: anchor is the same clip since it stays in place
    const anchorIndex = targetIndex > 0 ? targetIndex - 1 : 0;
    const anchorClipId = clipsList[anchorIndex].id;

    // Get the anchor element and its position
    const anchorElement = getClipElement(anchorClipId);
    const scrollContainer = getScrollContainer();

    if (!anchorElement || !scrollContainer) {
      return null;
    }

    // Calculate the offset from the top of the viewport
    const anchorRect = anchorElement.getBoundingClientRect();
    const containerRect = scrollContainer.getBoundingClientRect();
    const offsetFromViewport = anchorRect.top - containerRect.top;

    const anchor: ScrollAnchor = {
      clipId: anchorClipId,
      index: anchorIndex,
      offsetFromViewport,
    };

    return anchor;
  }, []);

  /**
   * Restores the scroll position using the provided anchor.
   * Uses requestAnimationFrame to ensure DOM has been updated.
   */
  const restoreScroll = useCallback((anchor: ScrollAnchor) => {
    const scrollContainer = getScrollContainer();
    const anchorElement = getClipElement(anchor.clipId);

    console.log("[ScrollAnchor] restoreScroll called, anchor:", anchor);
    console.log("[ScrollAnchor] scrollContainer:", scrollContainer?.className);
    console.log("[ScrollAnchor] anchorElement:", anchorElement?.getAttribute("data-clip-id"));

    if (!scrollContainer || !anchorElement) {
      console.log("[ScrollAnchor] Missing container or element, aborting");
      return;
    }

    // Calculate where the anchor element currently is
    const anchorRect = anchorElement.getBoundingClientRect();
    const containerRect = scrollContainer.getBoundingClientRect();
    const currentOffset = anchorRect.top - containerRect.top;

    // Calculate how much we need to scroll to restore the original position
    const scrollDelta = currentOffset - anchor.offsetFromViewport;

    console.log("[ScrollAnchor] currentOffset:", currentOffset, "originalOffset:", anchor.offsetFromViewport, "delta:", scrollDelta);
    console.log("[ScrollAnchor] scrollTop before:", scrollContainer.scrollTop);

    // Apply the scroll adjustment
    scrollContainer.scrollTop += scrollDelta;

    console.log("[ScrollAnchor] scrollTop after:", scrollContainer.scrollTop);
  }, []);

  /**
   * Schedules scroll restoration after the DOM has been updated.
   * Uses multiple requestAnimationFrame calls to ensure the DOM is fully updated.
   */
  const scheduleRestore = useCallback((anchor: ScrollAnchor) => {
    // Use nested RAF to ensure we run after React's commit phase and browser layout
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        restoreScroll(anchor);
      });
    });
  }, [restoreScroll]);

  return {
    captureAnchor,
    restoreScroll,
    scheduleRestore,
    pendingAnchorRef,
  };
}
