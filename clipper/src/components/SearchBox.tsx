import { useState, useEffect } from "react";
import { useI18n } from "../i18n";

interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBox({ value, onChange }: SearchBoxProps) {
  const { t } = useI18n();
  const [localValue, setLocalValue] = useState(value);

  // Debounce the search
  useEffect(() => {
    const timer = setTimeout(() => {
      onChange(localValue);
    }, 300);

    return () => clearTimeout(timer);
  }, [localValue, onChange]);

  // Sync with external value
  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  return (
    <div className="search-box">
      <input
        type="text"
        placeholder={t("search.placeholder")}
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        className="search-input"
      />
      {localValue && (
        <button
          className="clear-button"
          onClick={() => {
            setLocalValue("");
            onChange("");
          }}
        >
          ×
        </button>
      )}
    </div>
  );
}
