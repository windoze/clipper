import { useState } from "react";
import { useI18n } from "../i18n";

interface DateTagProps {
  dateStr: string;
  onSetStartDate?: (isoDate: string) => void;
  onSetEndDate?: (isoDate: string) => void;
}

export function DateTag({ dateStr, onSetStartDate, onSetEndDate }: DateTagProps) {
  const { t } = useI18n();
  const [isHovered, setIsHovered] = useState(false);

  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  const handleSetStartDate = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onSetStartDate) {
      // Set start of day for the start date
      const date = new Date(dateStr);
      date.setHours(0, 0, 0, 0);
      onSetStartDate(date.toISOString());
    }
  };

  const handleSetEndDate = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onSetEndDate) {
      // Set end of day for the end date
      const date = new Date(dateStr);
      date.setHours(23, 59, 59, 999);
      onSetEndDate(date.toISOString());
    }
  };

  return (
    <span
      className="date-tag"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <button
        className={`date-tag-arrow date-tag-arrow-left ${isHovered ? "visible" : ""}`}
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
      <span className="date-tag-text">{formatDate(dateStr)}</span>
      <button
        className={`date-tag-arrow date-tag-arrow-right ${isHovered ? "visible" : ""}`}
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
