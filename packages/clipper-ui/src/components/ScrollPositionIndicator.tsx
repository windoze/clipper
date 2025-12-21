import { useState, useEffect, useRef, useCallback } from "react";

interface ScrollPositionIndicatorProps {
  /** Current index (1-based) of the focused/active item */
  currentIndex: number;
  /** Total number of items */
  total: number;
  /** How long to show the indicator after change (ms) */
  hideDelay?: number;
  /** Animation duration for fade out (ms) */
  fadeDuration?: number;
}

/**
 * A flyover indicator that shows the current position (x/y format)
 * when the activated entry changes. Appears on the right side near
 * the scrollbar and fades out after a delay.
 */
export function ScrollPositionIndicator({
  currentIndex,
  total,
  hideDelay = 1500,
  fadeDuration = 300,
}: ScrollPositionIndicatorProps) {
  const [visible, setVisible] = useState(false);
  const [fading, setFading] = useState(false);
  const hideTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const fadeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const prevIndexRef = useRef<number>(currentIndex);

  // Show the indicator
  const showIndicator = useCallback(() => {
    // Clear any pending hide/fade
    if (hideTimeoutRef.current) {
      clearTimeout(hideTimeoutRef.current);
      hideTimeoutRef.current = null;
    }
    if (fadeTimeoutRef.current) {
      clearTimeout(fadeTimeoutRef.current);
      fadeTimeoutRef.current = null;
    }

    setFading(false);
    setVisible(true);

    // Schedule hide
    hideTimeoutRef.current = setTimeout(() => {
      setFading(true);
      fadeTimeoutRef.current = setTimeout(() => {
        setVisible(false);
        setFading(false);
      }, fadeDuration);
    }, hideDelay);
  }, [hideDelay, fadeDuration]);

  // Show indicator when currentIndex changes
  useEffect(() => {
    if (currentIndex !== prevIndexRef.current) {
      prevIndexRef.current = currentIndex;
      showIndicator();
    }
  }, [currentIndex, showIndicator]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (hideTimeoutRef.current) {
        clearTimeout(hideTimeoutRef.current);
      }
      if (fadeTimeoutRef.current) {
        clearTimeout(fadeTimeoutRef.current);
      }
    };
  }, []);

  if (!visible || total === 0) {
    return null;
  }

  // Display index (ensure it's within bounds)
  const displayIndex = Math.max(1, Math.min(currentIndex, total));

  return (
    <div
      className={`scroll-position-indicator${fading ? " fading" : ""}`}
      style={{ "--fade-duration": `${fadeDuration}ms` } as React.CSSProperties}
    >
      <span className="scroll-position-current">{displayIndex}</span>
      <span className="scroll-position-separator">/</span>
      <span className="scroll-position-total">{total}</span>
    </div>
  );
}
