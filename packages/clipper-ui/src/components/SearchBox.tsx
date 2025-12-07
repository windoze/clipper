import { useState, useEffect } from "react";
import { useI18n } from "../i18n";

interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
  filterTags?: string[];
  onRemoveTag?: (tag: string) => void;
  onClearAllTags?: () => void;
  label?: string;
}

// Helper to check if a tag is a host tag
const isHostTag = (tag: string) => tag.startsWith("$host:");

// Get the display name for a host tag (remove the $host: prefix)
const getHostTagDisplay = (tag: string) => tag.replace("$host:", "");

export function SearchBox({
  value,
  onChange,
  filterTags = [],
  onRemoveTag,
  onClearAllTags,
  label,
}: SearchBoxProps) {
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
      {label && <label className="search-box-label">{label}</label>}
      <div className="search-box-inner">
        {filterTags.length > 0 && (
          <div className="search-filter-tags">
            {filterTags.map((tag) => (
              <span key={tag} className={`search-filter-tag ${isHostTag(tag) ? "search-filter-tag-host" : ""}`}>
                {isHostTag(tag) && (
                  <svg
                    className="search-filter-tag-host-icon"
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
                    <line x1="8" y1="21" x2="16" y2="21"></line>
                    <line x1="12" y1="17" x2="12" y2="21"></line>
                  </svg>
                )}
                {isHostTag(tag) ? getHostTagDisplay(tag) : tag}
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
          placeholder={
            filterTags.length > 0
              ? t("search.placeholderWithTags")
              : t("search.placeholder")
          }
          value={localValue}
          onChange={(e) => setLocalValue(e.target.value)}
          className="search-input"
          spellCheck={false}
          autoCorrect="off"
          autoCapitalize="off"
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
