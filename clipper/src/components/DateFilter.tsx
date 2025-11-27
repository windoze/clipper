import { SearchFilters } from "../types";
import { useI18n } from "../i18n";

interface DateFilterProps {
  filters: SearchFilters;
  onChange: (filters: SearchFilters) => void;
}

export function DateFilter({ filters, onChange }: DateFilterProps) {
  const { t } = useI18n();
  const handleStartDateChange = (value: string) => {
    const newFilters = { ...filters };
    if (value) {
      // Convert local date to ISO string (start of day in UTC)
      const date = new Date(value);
      date.setHours(0, 0, 0, 0);
      newFilters.start_date = date.toISOString();
    } else {
      delete newFilters.start_date;
    }
    onChange(newFilters);
  };

  const handleEndDateChange = (value: string) => {
    const newFilters = { ...filters };
    if (value) {
      // Convert local date to ISO string (end of day in UTC)
      const date = new Date(value);
      date.setHours(23, 59, 59, 999);
      newFilters.end_date = date.toISOString();
    } else {
      delete newFilters.end_date;
    }
    onChange(newFilters);
  };

  // Convert ISO string back to date input format (YYYY-MM-DD)
  const formatDateForInput = (isoString?: string): string => {
    if (!isoString) return "";
    const date = new Date(isoString);
    return date.toISOString().split("T")[0];
  };

  return (
    <div className="date-filter">
      <div className="date-input-group">
        <label htmlFor="start-date">{t("filter.from")}</label>
        <input
          type="date"
          id="start-date"
          value={formatDateForInput(filters.start_date)}
          onChange={(e) => handleStartDateChange(e.target.value)}
          className="date-input"
        />
        {filters.start_date && (
          <button
            className="clear-button small"
            onClick={() => handleStartDateChange("")}
          >
            ×
          </button>
        )}
      </div>
      <div className="date-input-group">
        <label htmlFor="end-date">{t("filter.to")}</label>
        <input
          type="date"
          id="end-date"
          value={formatDateForInput(filters.end_date)}
          onChange={(e) => handleEndDateChange(e.target.value)}
          className="date-input"
        />
        {filters.end_date && (
          <button
            className="clear-button small"
            onClick={() => handleEndDateChange("")}
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
}
