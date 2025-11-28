import { useState, useEffect } from "react";
import { useI18n } from "../i18n";

interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
  filterTags?: string[];
  onRemoveTag?: (tag: string) => void;
  onClearAllTags?: () => void;
}

export function SearchBox({ value, onChange, filterTags = [], onRemoveTag, onClearAllTags }: SearchBoxProps) {
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

  const hasFilters = localValue || filterTags.length > 0;

  const handleClearAll = () => {
    setLocalValue("");
    onChange("");
    onClearAllTags?.();
  };

  return (
    <div className="search-box">
      <div className="search-box-inner">
        {filterTags.length > 0 && (
          <div className="search-filter-tags">
            {filterTags.map((tag) => (
              <span key={tag} className="search-filter-tag">
                {tag}
                <button
                  className="search-filter-tag-remove"
                  onClick={() => onRemoveTag?.(tag)}
                  title={t("filter.removeTag")}
                >
                  ×
                </button>
              </span>
            ))}
          </div>
        )}
        <input
          type="text"
          placeholder={filterTags.length > 0 ? t("search.placeholderWithTags") : t("search.placeholder")}
          value={localValue}
          onChange={(e) => setLocalValue(e.target.value)}
          className="search-input"
        />
      </div>
      {hasFilters && (
        <button
          className="clear-button"
          onClick={handleClearAll}
          title={t("filter.clearAll")}
        >
          ×
        </button>
      )}
    </div>
  );
}
