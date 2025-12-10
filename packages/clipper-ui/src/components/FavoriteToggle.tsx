import { forwardRef, useImperativeHandle, useRef } from "react";
import { useI18n } from "../i18n";

/** Handle to access FavoriteToggle's internal checkbox input */
export interface FavoriteToggleHandle {
  /** Focus the checkbox input */
  focus: () => void;
}

interface FavoriteToggleProps {
  value: boolean;
  onChange: (value: boolean) => void;
  /** Reference to the first element in the tab cycle (search input) for forward Tab cycling */
  tabCycleRef?: React.RefObject<HTMLInputElement | null>;
  /** Reference to the last date input (end-date) for backward Shift+Tab cycling */
  shiftTabCycleRef?: React.RefObject<HTMLInputElement | null>;
}

export const FavoriteToggle = forwardRef<FavoriteToggleHandle, FavoriteToggleProps>(
  function FavoriteToggle({ value, onChange, tabCycleRef, shiftTabCycleRef }, ref) {
    const { t } = useI18n();
    const inputRef = useRef<HTMLInputElement>(null);

    // Expose focus method to parent via ref
    useImperativeHandle(ref, () => ({
      focus: () => inputRef.current?.focus(),
    }));

    // Handle keyboard navigation
    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.currentTarget.blur();
      } else if (e.key === "Tab") {
        if (!e.shiftKey && tabCycleRef?.current) {
          // Tab (without Shift) cycles back to search input
          e.preventDefault();
          tabCycleRef.current.focus();
        } else if (e.shiftKey && shiftTabCycleRef?.current) {
          // Shift+Tab cycles back to end-date input
          e.preventDefault();
          shiftTabCycleRef.current.focus();
        }
      }
    };

    return (
      <div className="favorite-toggle">
        <label className="toggle-label">
          <input
            ref={inputRef}
            type="checkbox"
            checked={value}
            onChange={(e) => onChange(e.target.checked)}
            onKeyDown={handleKeyDown}
            className="toggle-input"
          />
          <span className="toggle-switch"></span>
          <span className="toggle-text">{t("filter.favorites")}</span>
        </label>
      </div>
    );
  }
);
