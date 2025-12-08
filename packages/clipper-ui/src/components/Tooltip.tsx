import { useState, useRef, useCallback, useEffect, ReactNode } from "react";
import { createPortal } from "react-dom";

interface TooltipProps {
  content: ReactNode;
  children: ReactNode;
  position?: "top" | "bottom" | "left" | "right";
  maxWidth?: number;
  delay?: number;
}

interface TooltipPosition {
  top: number;
  left: number;
}

export function Tooltip({
  content,
  children,
  position = "top",
  maxWidth = 400,
  delay = 200,
}: TooltipProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [tooltipPosition, setTooltipPosition] = useState<TooltipPosition | null>(null);
  const timeoutRef = useRef<number | null>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);

  const calculatePosition = useCallback(() => {
    if (!wrapperRef.current) return null;

    // getBoundingClientRect returns viewport coordinates, which is what we need for position: fixed
    const rect = wrapperRef.current.getBoundingClientRect();

    let top = 0;
    let left = 0;

    switch (position) {
      case "top":
        top = rect.top;
        left = rect.left + rect.width / 2;
        break;
      case "bottom":
        top = rect.bottom;
        left = rect.left + rect.width / 2;
        break;
      case "left":
        top = rect.top + rect.height / 2;
        left = rect.left;
        break;
      case "right":
        top = rect.top + rect.height / 2;
        left = rect.right;
        break;
    }

    return { top, left };
  }, [position]);

  const showTooltip = useCallback(() => {
    timeoutRef.current = window.setTimeout(() => {
      setTooltipPosition(calculatePosition());
      setIsVisible(true);
    }, delay);
  }, [delay, calculatePosition]);

  const hideTooltip = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setIsVisible(false);
    setTooltipPosition(null);
  }, []);

  // Hide tooltip on scroll to avoid stale positioning
  useEffect(() => {
    if (!isVisible) return;

    const handleScroll = () => {
      hideTooltip();
    };

    // Listen on capture phase to catch scroll events from any scrollable container
    window.addEventListener("scroll", handleScroll, true);

    return () => {
      window.removeEventListener("scroll", handleScroll, true);
    };
  }, [isVisible, hideTooltip]);

  const tooltipStyle: React.CSSProperties = tooltipPosition
    ? {
        position: "fixed",
        top: tooltipPosition.top,
        left: tooltipPosition.left,
        maxWidth,
        transform:
          position === "top"
            ? "translate(-50%, -100%) translateY(-8px)"
            : position === "bottom"
              ? "translate(-50%, 0) translateY(8px)"
              : position === "left"
                ? "translate(-100%, -50%) translateX(-8px)"
                : "translate(0, -50%) translateX(8px)",
      }
    : {};

  return (
    <div
      ref={wrapperRef}
      className="tooltip-wrapper"
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
      onFocus={showTooltip}
      onBlur={hideTooltip}
    >
      {children}
      {isVisible &&
        content &&
        tooltipPosition &&
        createPortal(
          <div
            className={`tooltip tooltip-${position} tooltip-portal`}
            style={tooltipStyle}
            role="tooltip"
          >
            <div className="tooltip-content">{content}</div>
            <div className="tooltip-arrow" />
          </div>,
          document.body
        )}
    </div>
  );
}
