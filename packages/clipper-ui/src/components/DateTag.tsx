import { useI18n } from "../i18n";

interface DateTagProps {
  dateStr: string;
  onSetStartDate?: (isoDate: string) => void;
  onSetEndDate?: (isoDate: string) => void;
}

export function DateTag({ dateStr, onSetStartDate, onSetEndDate }: DateTagProps) {
  const { t } = useI18n();

  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  /**
   * Set the start date filter to the beginning of the day (local time) for this clip's date.
   *
   * The dateStr is an ISO string (UTC) from the clip's created_at field.
   * We convert it to the local date and set the filter to local midnight of that day.
   * This ensures the filter uses local timezone semantics consistent with DateFilter.
   */
  const handleSetStartDate = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onSetStartDate) {
      // Parse the clip's timestamp
      const date = new Date(dateStr);
      // Set to start of day in local timezone (local midnight)
      date.setHours(0, 0, 0, 0);
      // Convert to UTC ISO string for the server
      onSetStartDate(date.toISOString());
    }
  };

  /**
   * Set the end date filter to include the entire day (local time) for this clip's date.
   *
   * The dateStr is an ISO string (UTC) from the clip's created_at field.
   * We convert it to the local date and set the filter to local midnight of the NEXT day.
   * This uses exclusive end boundary semantics consistent with DateFilter.
   */
  const handleSetEndDate = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onSetEndDate) {
      // Parse the clip's timestamp
      const date = new Date(dateStr);
      // Set to start of day in local timezone
      date.setHours(0, 0, 0, 0);
      // Add one day to get the exclusive end boundary (start of next day)
      // This ensures all clips from the selected date are included
      date.setDate(date.getDate() + 1);
      // Convert to UTC ISO string for the server
      onSetEndDate(date.toISOString());
    }
  };

  return (
    <span className="date-tag">
      <button
        className="date-tag-arrow date-tag-arrow-left"
        onClick={handleSetStartDate}
        title={t("dateTag.setStartDate")}
      >
        <svg
          width="10"
          height="10"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      <svg
        className="date-tag-icon"
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="16" y1="2" x2="16" y2="6"></line>
        <line x1="8" y1="2" x2="8" y2="6"></line>
        <line x1="3" y1="10" x2="21" y2="10"></line>
      </svg>
      <span className="date-tag-text">{formatDate(dateStr)}</span>
      <button
        className="date-tag-arrow date-tag-arrow-right"
        onClick={handleSetEndDate}
        title={t("dateTag.setEndDate")}
      >
        <svg
          width="10"
          height="10"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
      </button>
    </span>
  );
}
