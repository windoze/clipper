import { useRef, useImperativeHandle, forwardRef } from "react";
import { SearchFilters } from "../types";
import { useI18n } from "../i18n";

/** Handle to access DateFilter's internal refs */
export interface DateFilterHandle {
  /** Focus the end-date input */
  focusEndDate: () => void;
}

interface DateFilterProps {
  filters: SearchFilters;
  onChange: (filters: SearchFilters) => void;
  /** Reference to the search input for Shift+Tab cycling from start-date */
  shiftTabCycleRef?: React.RefObject<HTMLInputElement | null>;
}

/**
 * Helper to parse a date input string (YYYY-MM-DD) as local midnight.
 * The HTML date input returns a string like "2025-12-03" without time info.
 * We need to interpret this as local midnight (start of day in local timezone),
 * then convert to UTC for the server.
 */
function parseLocalDateString(dateString: string): Date {
  // Split the date string to avoid timezone issues with Date.parse
  const [year, month, day] = dateString.split("-").map(Number);
  // Create date at local midnight (months are 0-indexed in JavaScript)
  return new Date(year, month - 1, day, 0, 0, 0, 0);
}

/**
 * Format a Date object to YYYY-MM-DD string in local timezone.
 * Used for displaying dates in the date input which expects local dates.
 */
function formatLocalDate(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export const DateFilter = forwardRef<DateFilterHandle, DateFilterProps>(function DateFilter(
  { filters, onChange, shiftTabCycleRef },
  ref
) {
  const { t } = useI18n();
  const startDateRef = useRef<HTMLInputElement>(null);
  const endDateRef = useRef<HTMLInputElement>(null);

  // Expose methods to parent via ref
  useImperativeHandle(ref, () => ({
    focusEndDate: () => endDateRef.current?.focus(),
  }));

  /**
   * Handle start date change from the date picker.
   * The date picker value is in YYYY-MM-DD format (local date, no time).
   *
   * Behavior:
   * - When user selects YYYY-MM-DD, we filter from YYYY-MM-DD 00:00:00 local time
   * - The displayed date in the picker shows YYYY-MM-DD (the selected local date)
   * - The server receives the UTC equivalent of local midnight
   */
  const handleStartDateChange = (value: string) => {
    const newFilters = { ...filters };
    if (value) {
      // Parse the date string as local midnight
      const date = parseLocalDateString(value);
      // Convert to UTC ISO string for the server
      newFilters.start_date = date.toISOString();
    } else {
      delete newFilters.start_date;
    }
    onChange(newFilters);
  };

  /**
   * Handle end date change from the date picker.
   * The date picker value is in YYYY-MM-DD format (local date, no time).
   *
   * Behavior:
   * - When user selects YYYY-MM-DD, we filter up to (but not including) YYYY-MM-DD+1 00:00:00 local time
   * - This means all clips from the selected date are included
   * - The displayed date in the picker shows YYYY-MM-DD (the selected local date, not +1)
   * - The server receives the UTC equivalent of the next day's local midnight
   */
  const handleEndDateChange = (value: string) => {
    const newFilters = { ...filters };
    if (value) {
      // Parse the date string as local midnight
      const date = parseLocalDateString(value);
      // Add one day to get the exclusive end boundary (start of next day)
      // This ensures all clips from the selected end date are included
      date.setDate(date.getDate() + 1);
      // Convert to UTC ISO string for the server
      newFilters.end_date = date.toISOString();
    } else {
      delete newFilters.end_date;
    }
    onChange(newFilters);
  };

  /**
   * Convert an ISO string (UTC) back to date input format (YYYY-MM-DD) in local timezone.
   *
   * For start_date: The stored ISO string represents local midnight in UTC,
   * so we convert it back to local timezone and format as YYYY-MM-DD.
   *
   * For end_date: The stored ISO string represents local midnight of the NEXT day,
   * so we need to subtract one day before displaying.
   */
  const formatDateForInput = (isoString: string | undefined, isEndDate: boolean): string => {
    if (!isoString) return "";
    const date = new Date(isoString);
    if (isEndDate) {
      // Subtract one day since we stored the start of the next day
      date.setDate(date.getDate() - 1);
    }
    // Format in local timezone
    return formatLocalDate(date);
  };

  // Handle keyboard navigation for start-date input
  const handleStartDateKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.currentTarget.blur();
    } else if (e.key === "Tab" && e.shiftKey && shiftTabCycleRef?.current) {
      // Shift+Tab cycles back to search input
      e.preventDefault();
      shiftTabCycleRef.current.focus();
    } else if ((e.key === "ArrowUp" || e.key === "ArrowDown") && !e.currentTarget.value) {
      // When empty and pressing up/down, set to today's date first
      e.preventDefault();
      const today = formatLocalDate(new Date());
      e.currentTarget.value = today;
      // Trigger the change handler
      handleStartDateChange(today);
    }
  };

  // Handle keyboard navigation for end-date input
  const handleEndDateKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.currentTarget.blur();
    } else if (e.key === "Tab" && e.shiftKey) {
      // Shift+Tab goes to start-date input
      e.preventDefault();
      startDateRef.current?.focus();
    } else if ((e.key === "ArrowUp" || e.key === "ArrowDown") && !e.currentTarget.value) {
      // When empty and pressing up/down, set to today's date first
      e.preventDefault();
      const today = formatLocalDate(new Date());
      e.currentTarget.value = today;
      // Trigger the change handler
      handleEndDateChange(today);
    }
  };

  return (
    <div className="date-filter">
      <div className={`date-input-group${filters.start_date ? " date-input-group-active" : ""}`}>
        <label htmlFor="start-date">{t("filter.from")}</label>
        <input
          ref={startDateRef}
          type="date"
          id="start-date"
          value={formatDateForInput(filters.start_date, false)}
          onChange={(e) => handleStartDateChange(e.target.value)}
          onKeyDown={handleStartDateKeyDown}
          className="date-input"
        />
        {filters.start_date && (
          <button
            className="clear-button small"
            onClick={() => handleStartDateChange("")}
            tabIndex={-1}
          >
            ×
          </button>
        )}
      </div>
      <div className={`date-input-group${filters.end_date ? " date-input-group-active" : ""}`}>
        <label htmlFor="end-date">{t("filter.to")}</label>
        <input
          ref={endDateRef}
          type="date"
          id="end-date"
          value={formatDateForInput(filters.end_date, true)}
          onChange={(e) => handleEndDateChange(e.target.value)}
          onKeyDown={handleEndDateKeyDown}
          className="date-input"
        />
        {filters.end_date && (
          <button
            className="clear-button small"
            onClick={() => handleEndDateChange("")}
            tabIndex={-1}
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
});
