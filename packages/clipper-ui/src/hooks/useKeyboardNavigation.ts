import { useState, useCallback, useEffect, useRef } from "react";
import { Clip } from "../types";

export interface KeyboardNavigationState {
  /** Currently focused clip ID (null if no clip is focused) */
  focusedClipId: string | null;
  /** Currently focused button index within the focused clip (-1 if no button is focused) */
  focusedButtonIndex: number;
  /** Whether the search input should be focused */
  searchFocused: boolean;
  /** Whether keyboard navigation is currently active (suppresses hover styles) */
  keyboardNavigating: boolean;
}

export interface KeyboardNavigationActions {
  /** Set the focused clip ID */
  setFocusedClipId: (id: string | null) => void;
  /** Set the focused button index */
  setFocusedButtonIndex: (index: number) => void;
  /** Set search focus state */
  setSearchFocused: (focused: boolean) => void;
  /** Focus the next clip in the list */
  focusNextClip: () => void;
  /** Focus the previous clip in the list */
  focusPreviousClip: () => void;
  /** Focus the next button in the current clip */
  focusNextButton: () => void;
  /** Focus the previous button in the current clip */
  focusPreviousButton: () => void;
  /** Clear all focus (blur everything) */
  clearFocus: () => void;
  /** Get the currently focused clip */
  getFocusedClip: () => Clip | null;
}

export interface UseKeyboardNavigationOptions {
  /** List of clips for navigation */
  clips: Clip[];
  /** Number of action buttons per clip (copy, share, favorite, delete, expand, etc.) */
  buttonCount: number;
  /** Callback when a clip should be expanded/collapsed */
  onExpandToggle?: (clipId: string) => void;
  /** Callback when a clip should be deleted */
  onDelete?: (clipId: string) => void;
  /** Callback when a button should be activated */
  onButtonActivate?: (clipId: string, buttonIndex: number) => void;
  /** Reference to the search input element */
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  /** Whether the keyboard navigation is enabled */
  enabled?: boolean;
  /** Whether there are more clips to load */
  hasMore?: boolean;
  /** Callback to load more clips when navigating past the end */
  onLoadMore?: () => void;
  /** Reference to the clip list container for Page Up/Down scrolling */
  containerRef?: React.RefObject<HTMLElement | null>;
}

// Buttons in order: copy, share (if enabled), favorite, delete
// We'll define standard button actions
export type ClipButtonAction = "copy" | "share" | "favorite" | "notes" | "expand" | "delete";

export interface UseKeyboardNavigationReturn extends KeyboardNavigationState, KeyboardNavigationActions {
  /** Handle keydown events - attach to the container */
  handleKeyDown: (e: React.KeyboardEvent | KeyboardEvent) => void;
}

export function useKeyboardNavigation({
  clips,
  buttonCount,
  onExpandToggle,
  onDelete,
  onButtonActivate,
  searchInputRef,
  enabled = true,
  hasMore = false,
  onLoadMore,
  containerRef,
}: UseKeyboardNavigationOptions): UseKeyboardNavigationReturn {
  const [focusedClipId, setFocusedClipId] = useState<string | null>(null);
  const [focusedButtonIndex, setFocusedButtonIndex] = useState(-1);
  const [searchFocused, setSearchFocused] = useState(false);
  const [keyboardNavigating, setKeyboardNavigating] = useState(false);

  // Track last mouse position to detect actual mouse movement vs scroll-induced events
  const lastMousePosition = useRef<{ x: number; y: number } | null>(null);

  // Reset keyboard navigating state on actual mouse movement (not scroll-induced)
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      // Check if mouse actually moved (not just a scroll-induced event)
      const currentPos = { x: e.clientX, y: e.clientY };
      const lastPos = lastMousePosition.current;

      if (lastPos && (currentPos.x !== lastPos.x || currentPos.y !== lastPos.y)) {
        // Mouse actually moved
        if (keyboardNavigating) {
          setKeyboardNavigating(false);
        }
      }

      lastMousePosition.current = currentPos;
    };

    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, [keyboardNavigating]);

  // Check if any popup/dialog is currently open
  const isPopupOpen = useCallback(() => {
    // Check for any open dialog/popup in the DOM
    const popupSelectors = [
      ".delete-confirm-dialog",
      ".notes-popup-backdrop",
      ".notes-popup-dialog",
      ".share-dialog-backdrop",
      ".share-dialog",
      ".settings-dialog",
      ".image-popup-backdrop",
      ".image-popup-container",
      ".language-selector-dropdown",
    ];
    return popupSelectors.some(selector => document.querySelector(selector) !== null);
  }, []);

  // Track if any active element has focus (input, textarea, contenteditable)
  const isActiveElementInteractive = useCallback(() => {
    const activeElement = document.activeElement;
    if (!activeElement) return false;

    const tagName = activeElement.tagName.toLowerCase();
    if (tagName === "input" || tagName === "textarea" || tagName === "select") {
      return true;
    }
    if (activeElement.getAttribute("contenteditable") === "true") {
      return true;
    }
    // Check if inside a dialog or modal
    if (activeElement.closest(".delete-confirm-dialog, .notes-popup-dialog, .share-dialog, .settings-dialog")) {
      return true;
    }
    return false;
  }, []);

  // Check if there's an active button/control in the selected item that should receive the key event
  const hasActiveControlInSelectedItem = useCallback(() => {
    if (!focusedClipId) return false;
    const clipElement = document.querySelector(`[data-clip-id="${focusedClipId}"]`);
    if (!clipElement) return false;

    const activeElement = document.activeElement;
    if (!activeElement) return false;

    // Check if the active element is within the clip entry and is interactive
    if (clipElement.contains(activeElement)) {
      const tagName = activeElement.tagName.toLowerCase();
      if (tagName === "input" || tagName === "textarea" || tagName === "button") {
        return true;
      }
    }
    return false;
  }, [focusedClipId]);

  const getFocusedClip = useCallback((): Clip | null => {
    if (!focusedClipId) return null;
    return clips.find(clip => clip.id === focusedClipId) || null;
  }, [focusedClipId, clips]);

  // Blur any focused element that is NOT within the specified clip
  const blurElementsOutsideClip = useCallback((targetClipId: string) => {
    const activeElement = document.activeElement as HTMLElement;
    if (!activeElement || activeElement === document.body) return;

    const targetClipElement = document.querySelector(`[data-clip-id="${targetClipId}"]`);
    // If the active element is not within the target clip, blur it
    if (targetClipElement && !targetClipElement.contains(activeElement)) {
      activeElement.blur();
    }
  }, []);

  const focusNextClip = useCallback(() => {
    if (clips.length === 0) return;

    let targetClipId: string;

    if (!focusedClipId) {
      // No clip focused, focus the first one
      targetClipId = clips[0].id;
    } else {
      const currentIndex = clips.findIndex(clip => clip.id === focusedClipId);
      if (currentIndex === -1) {
        // Not found, focus the first one
        targetClipId = clips[0].id;
      } else if (currentIndex >= clips.length - 1) {
        // Already at the last clip - trigger load more if available, otherwise do nothing
        if (hasMore && onLoadMore) {
          onLoadMore();
        }
        return;
      } else {
        targetClipId = clips[currentIndex + 1].id;
      }
    }

    // Blur any focused element not in the target clip
    blurElementsOutsideClip(targetClipId);
    setFocusedClipId(targetClipId);
    setFocusedButtonIndex(-1);
    setKeyboardNavigating(true);
  }, [clips, focusedClipId, hasMore, onLoadMore, blurElementsOutsideClip]);

  const focusPreviousClip = useCallback(() => {
    if (clips.length === 0) return;

    let targetClipId: string;

    if (!focusedClipId) {
      // No clip focused, focus the first one
      targetClipId = clips[0].id;
    } else {
      const currentIndex = clips.findIndex(clip => clip.id === focusedClipId);
      if (currentIndex <= 0) {
        // Already at the first clip or not found - do nothing
        return;
      } else {
        targetClipId = clips[currentIndex - 1].id;
      }
    }

    // Blur any focused element not in the target clip
    blurElementsOutsideClip(targetClipId);
    setFocusedClipId(targetClipId);
    setFocusedButtonIndex(-1);
    setKeyboardNavigating(true);
  }, [clips, focusedClipId, blurElementsOutsideClip]);

  const focusNextButton = useCallback(() => {
    if (!focusedClipId || buttonCount === 0) return;

    const newIndex = focusedButtonIndex + 1;
    if (newIndex >= buttonCount) {
      // Wrap around to -1 (no button focused)
      setFocusedButtonIndex(-1);
    } else {
      setFocusedButtonIndex(newIndex);
    }
  }, [focusedClipId, focusedButtonIndex, buttonCount]);

  const focusPreviousButton = useCallback(() => {
    if (!focusedClipId || buttonCount === 0) return;

    const newIndex = focusedButtonIndex - 1;
    if (newIndex < -1) {
      // Wrap around to last button
      setFocusedButtonIndex(buttonCount - 1);
    } else {
      setFocusedButtonIndex(newIndex);
    }
  }, [focusedClipId, focusedButtonIndex, buttonCount]);

  const clearFocus = useCallback(() => {
    setFocusedClipId(null);
    setFocusedButtonIndex(-1);
    setSearchFocused(false);
  }, []);

  // Scroll focused clip into view
  useEffect(() => {
    if (!focusedClipId) return;

    const clipElement = document.querySelector(`[data-clip-id="${focusedClipId}"]`);
    if (clipElement) {
      clipElement.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }, [focusedClipId]);

  // Reset focus when clips change significantly
  useEffect(() => {
    if (focusedClipId && !clips.find(clip => clip.id === focusedClipId)) {
      // Focused clip was removed, clear focus
      setFocusedClipId(null);
      setFocusedButtonIndex(-1);
    }
  }, [clips, focusedClipId]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent | KeyboardEvent) => {
    if (!enabled) return;

    // Don't handle keyboard navigation when any popup/dialog is open
    // Let the popup handle its own keyboard events
    // But we need to trap Tab focus within the popup
    if (isPopupOpen()) {
      // For Tab key, prevent focus from leaving the popup
      if (e.key === "Tab") {
        const popupSelectors = [
          ".delete-confirm-dialog",
          ".notes-popup-dialog",
          ".share-dialog",
          ".settings-dialog",
          ".image-popup-container",
        ];

        for (const selector of popupSelectors) {
          const popup = document.querySelector(selector) as HTMLElement;
          if (popup) {
            // Get all focusable elements within the popup
            const focusableSelectors = [
              "button:not([disabled])",
              "input:not([disabled])",
              "textarea:not([disabled])",
              "select:not([disabled])",
              "[tabindex]:not([tabindex='-1'])",
              "a[href]",
            ].join(", ");
            const focusableElements = Array.from(
              popup.querySelectorAll(focusableSelectors)
            ) as HTMLElement[];

            if (focusableElements.length > 0) {
              const activeElement = document.activeElement as HTMLElement;
              const currentIndex = focusableElements.indexOf(activeElement);

              // If focus is outside the popup or at boundaries, trap it
              if (currentIndex === -1) {
                // Focus is outside popup, move it inside
                e.preventDefault();
                focusableElements[0].focus();
                return;
              }

              if (e.shiftKey && currentIndex === 0) {
                // Shift+Tab at first element, wrap to last
                e.preventDefault();
                focusableElements[focusableElements.length - 1].focus();
                return;
              }

              if (!e.shiftKey && currentIndex === focusableElements.length - 1) {
                // Tab at last element, wrap to first
                e.preventDefault();
                focusableElements[0].focus();
                return;
              }
            }
            break;
          }
        }
      }
      return;
    }

    const key = e.key;
    const isSearchFocused = searchInputRef?.current === document.activeElement;

    // Handle Escape - blur search input or clear clip focus
    if (key === "Escape") {
      if (isSearchFocused && searchInputRef?.current) {
        e.preventDefault();
        searchInputRef.current.blur();
        setSearchFocused(false);
        return;
      }
      if (focusedClipId) {
        e.preventDefault();
        clearFocus();
        return;
      }
      return;
    }

    // When search is focused, handle ArrowDown to exit search and focus first visible clip
    if (isSearchFocused) {
      if (key === "ArrowDown") {
        e.preventDefault();
        // Blur the search input
        searchInputRef?.current?.blur();
        setSearchFocused(false);

        // Find the first clip visible in the viewport
        const clipListElement = containerRef?.current;
        if (clipListElement && clips.length > 0) {
          // Find the scrollable parent to determine viewport
          let scrollableParent: HTMLElement | null = clipListElement.parentElement;
          while (scrollableParent) {
            const style = window.getComputedStyle(scrollableParent);
            const overflowY = style.overflowY;
            if (overflowY === "auto" || overflowY === "scroll") {
              break;
            }
            scrollableParent = scrollableParent.parentElement;
          }

          const scrollContainer = scrollableParent || document.documentElement;
          const containerRect = scrollContainer.getBoundingClientRect();

          // Find the first clip that's at least partially visible in the viewport
          let firstVisibleClipId: string | null = null;
          for (const clip of clips) {
            const clipElement = document.querySelector(`[data-clip-id="${clip.id}"]`);
            if (clipElement) {
              const clipRect = clipElement.getBoundingClientRect();
              // Check if the clip is at least partially visible
              if (clipRect.bottom > containerRect.top && clipRect.top < containerRect.bottom) {
                firstVisibleClipId = clip.id;
                break;
              }
            }
          }

          // If found a visible clip, focus it; otherwise focus the first clip
          setFocusedClipId(firstVisibleClipId || clips[0].id);
          setFocusedButtonIndex(-1);
        }
        return;
      }
      // Tab cycling through search bar controls is handled by the browser/SearchBox
      // Let other keys go through for typing
      return;
    }

    // Page Up/Down scrolls the list and changes active item
    // Handle this early, before isActiveElementInteractive check, because Page Up/Down
    // should work even when no element has focus (just visual activation)
    if (focusedClipId && (key === "PageUp" || key === "PageDown")) {
      // Find the scrollable parent (the element with overflow-y: auto/scroll)
      // The clip-list itself is not scrollable, but its parent (.app-main) is
      const clipListElement = containerRef?.current;
      if (clipListElement) {
        // Find the scrollable ancestor
        let scrollableParent: HTMLElement | null = clipListElement.parentElement;
        while (scrollableParent) {
          const style = window.getComputedStyle(scrollableParent);
          const overflowY = style.overflowY;
          if (overflowY === "auto" || overflowY === "scroll") {
            break;
          }
          scrollableParent = scrollableParent.parentElement;
        }

        const scrollContainer = scrollableParent || clipListElement;
        e.preventDefault();
        const scrollAmount = scrollContainer.clientHeight * 0.8; // Scroll 80% of visible height

        // Scroll first
        scrollContainer.scrollBy({
          top: key === "PageDown" ? scrollAmount : -scrollAmount,
          behavior: "smooth",
        });

        // Change active item: move by approximately a page worth of clips
        const currentIndex = clips.findIndex(clip => clip.id === focusedClipId);
        if (currentIndex !== -1) {
          // Estimate how many clips fit in one page by checking clip heights
          const currentClipElement = document.querySelector(`[data-clip-id="${focusedClipId}"]`);
          const avgClipHeight = currentClipElement?.getBoundingClientRect().height || 100;
          const clipsPerPage = Math.max(1, Math.floor(scrollAmount / avgClipHeight));

          let newIndex: number;
          if (key === "PageDown") {
            newIndex = Math.min(clips.length - 1, currentIndex + clipsPerPage);
            // If at the last clip and there's more to load, trigger load more
            if (newIndex === clips.length - 1 && hasMore && onLoadMore) {
              onLoadMore();
            }
          } else {
            newIndex = Math.max(0, currentIndex - clipsPerPage);
          }

          if (newIndex !== currentIndex) {
            blurElementsOutsideClip(clips[newIndex].id);
            setFocusedClipId(clips[newIndex].id);
            setFocusedButtonIndex(-1);
          }
        }
      }
      return;
    }

    // Check if we're in an interactive element (dialog, input, etc.)
    if (isActiveElementInteractive()) {
      return;
    }

    // Alphabetic keys, '#', and '@' focus the search input (unless there's an active control in selected item)
    // Note: We do NOT prevent default here so the character will be typed into the search input
    if (key.length === 1 && /[a-zA-Z#@]/.test(key)) {
      if (!hasActiveControlInSelectedItem()) {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
          setSearchFocused(true);
          // Let the default behavior happen so the character is typed
        }
        return;
      }
    }

    // Handle IME composition (for CJK input methods)
    // The "Process" key is sent during IME composition - focus the input
    if (e.key === "Process") {
      if (!hasActiveControlInSelectedItem() && searchInputRef?.current) {
        searchInputRef.current.focus();
        setSearchFocused(true);
      }
      return;
    }

    // Arrow key navigation between clips
    if (key === "ArrowDown") {
      e.preventDefault();
      focusNextClip();
      return;
    }

    if (key === "ArrowUp") {
      e.preventDefault();
      // Check if we're at the first clip - if so, focus search and scroll to top
      if (focusedClipId && clips.length > 0) {
        const currentIndex = clips.findIndex(clip => clip.id === focusedClipId);
        if (currentIndex === 0) {
          // At the first clip - focus search input and scroll to top
          clearFocus();
          if (searchInputRef?.current) {
            searchInputRef.current.focus();
            setSearchFocused(true);
          }
          // Scroll to top
          const clipListElement = containerRef?.current;
          if (clipListElement) {
            let scrollableParent: HTMLElement | null = clipListElement.parentElement;
            while (scrollableParent) {
              const style = window.getComputedStyle(scrollableParent);
              const overflowY = style.overflowY;
              if (overflowY === "auto" || overflowY === "scroll") {
                break;
              }
              scrollableParent = scrollableParent.parentElement;
            }
            const scrollContainer = scrollableParent || clipListElement;
            scrollContainer.scrollTo({ top: 0, behavior: "smooth" });
          }
          return;
        }
      }
      focusPreviousClip();
      return;
    }

    // If a clip is focused, handle clip-specific keys
    if (focusedClipId) {
      // Tab/Shift+Tab cycles through interactive elements within the clip
      if (key === "Tab") {
        const clipElement = document.querySelector(`[data-clip-id="${focusedClipId}"]`);
        if (clipElement) {
          // Get all focusable elements within the clip
          const focusableSelectors = [
            "button:not([disabled]):not([tabindex='-1'])",
            "input:not([disabled]):not([tabindex='-1'])",
            "select:not([disabled]):not([tabindex='-1'])",
            "[tabindex]:not([tabindex='-1'])",
          ].join(", ");
          const focusableElements = Array.from(
            clipElement.querySelectorAll(focusableSelectors)
          ) as HTMLElement[];

          if (focusableElements.length > 0) {
            e.preventDefault();
            const activeElement = document.activeElement as HTMLElement;
            const currentIndex = focusableElements.indexOf(activeElement);

            if (e.shiftKey) {
              // Shift+Tab - go backwards
              if (currentIndex <= 0) {
                // At the first element or not focused on any element, wrap to last
                focusableElements[focusableElements.length - 1].focus();
              } else {
                focusableElements[currentIndex - 1].focus();
              }
            } else {
              // Tab - go forwards
              if (currentIndex === -1) {
                // Not focused on any element, focus the first one
                focusableElements[0].focus();
              } else if (currentIndex >= focusableElements.length - 1) {
                // At the last element, wrap to first
                focusableElements[0].focus();
              } else {
                focusableElements[currentIndex + 1].focus();
              }
            }
            return;
          }
        }
      }

      // Left/Right arrow keys - behave like Tab/Shift+Tab for navigating focusable elements
      if (key === "ArrowRight" || key === "ArrowLeft") {
        const clipElement = document.querySelector(`[data-clip-id="${focusedClipId}"]`);
        if (clipElement) {
          // Get all focusable elements within the clip
          const focusableSelectors = [
            "button:not([disabled]):not([tabindex='-1'])",
            "input:not([disabled]):not([tabindex='-1'])",
            "select:not([disabled]):not([tabindex='-1'])",
            "[tabindex]:not([tabindex='-1'])",
          ].join(", ");
          const focusableElements = Array.from(
            clipElement.querySelectorAll(focusableSelectors)
          ) as HTMLElement[];

          if (focusableElements.length > 0) {
            e.preventDefault();
            const activeElement = document.activeElement as HTMLElement;
            const currentIndex = focusableElements.indexOf(activeElement);

            if (key === "ArrowRight") {
              if (currentIndex === -1) {
                // Not focused on any element, focus the first one
                focusableElements[0].focus();
              } else if (currentIndex >= focusableElements.length - 1) {
                // At the last element, wrap to first
                focusableElements[0].focus();
              } else {
                focusableElements[currentIndex + 1].focus();
              }
            } else {
              // ArrowLeft - go backwards
              if (currentIndex <= 0) {
                // At the first element or not focused on any element, wrap to last
                focusableElements[focusableElements.length - 1].focus();
              } else {
                focusableElements[currentIndex - 1].focus();
              }
            }
            return;
          }
        }
        return;
      }

      // Enter expands/collapses the clip
      if (key === "Enter") {
        e.preventDefault();
        if (focusedButtonIndex === -1) {
          // No button focused, toggle expand
          onExpandToggle?.(focusedClipId);
        } else {
          // A button is focused, activate it
          onButtonActivate?.(focusedClipId, focusedButtonIndex);
        }
        return;
      }

      // Space activates the focused button (if any)
      if (key === " ") {
        if (focusedButtonIndex >= 0) {
          e.preventDefault();
          onButtonActivate?.(focusedClipId, focusedButtonIndex);
        }
        return;
      }

      // Delete/Backspace deletes the clip (only if no button is focused)
      if (key === "Delete" || key === "Backspace") {
        if (!hasActiveControlInSelectedItem()) {
          e.preventDefault();
          onDelete?.(focusedClipId);
        }
        return;
      }
    }
  }, [
    enabled,
    clips,
    searchInputRef,
    containerRef,
    focusedClipId,
    focusedButtonIndex,
    hasMore,
    onLoadMore,
    blurElementsOutsideClip,
    isPopupOpen,
    isActiveElementInteractive,
    hasActiveControlInSelectedItem,
    focusNextClip,
    focusPreviousClip,
    focusNextButton,
    focusPreviousButton,
    clearFocus,
    onExpandToggle,
    onDelete,
    onButtonActivate,
  ]);

  return {
    focusedClipId,
    focusedButtonIndex,
    searchFocused,
    keyboardNavigating,
    setFocusedClipId,
    setFocusedButtonIndex,
    setSearchFocused,
    focusNextClip,
    focusPreviousClip,
    focusNextButton,
    focusPreviousButton,
    clearFocus,
    getFocusedClip,
    handleKeyDown,
  };
}
