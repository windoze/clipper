import { useState, useEffect, useRef, useCallback } from "react";
import { useI18n } from "../i18n";
import { Tag } from "../types";

interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
  filterTags?: string[];
  onRemoveTag?: (tag: string) => void;
  onClearAllTags?: () => void;
  onAddTag?: (tag: string) => void;
  label?: string;
  /** Function to search tags by query */
  onSearchTags?: (query: string) => Promise<Tag[]>;
  /** Function to list all tags (when "#" is typed with no query) */
  onListTags?: () => Promise<Tag[]>;
  /** Ref to the search input element for keyboard navigation */
  inputRef?: React.RefObject<HTMLInputElement | null>;
  /** Reference to the favorite toggle for Shift+Tab cycling */
  shiftTabCycleRef?: React.RefObject<HTMLInputElement | null>;
}

// Input mode type: regular tags (#) or host tags (@)
type TagInputMode = "tag" | "host";

// Helper to check if a tag is a host tag
const isHostTag = (tag: string) => tag.startsWith("$host:");

// Helper to check if a tag is a system tag (starts with $)
const isSystemTag = (tag: string) => tag.startsWith("$");

// Get the display name for a host tag (remove the $host: prefix)
const getHostTagDisplay = (tag: string) => tag.replace("$host:", "");

export function SearchBox({
  value,
  onChange,
  filterTags = [],
  onRemoveTag,
  onClearAllTags,
  onAddTag,
  label,
  onSearchTags,
  onListTags,
  inputRef,
  shiftTabCycleRef,
}: SearchBoxProps) {
  const { t } = useI18n();
  // Search text (without tag query)
  const [searchText, setSearchText] = useState(value);
  // Tag query (what user is typing after # or @)
  const [tagQuery, setTagQuery] = useState("");
  // Whether we're in tag input mode and what type
  const [tagInputMode, setTagInputMode] = useState<TagInputMode | null>(null);
  const [showTagDropdown, setShowTagDropdown] = useState(false);
  const [tagSuggestions, setTagSuggestions] = useState<Tag[]>([]);
  const [selectedTagIndex, setSelectedTagIndex] = useState(-1);
  const [isLoadingTags, setIsLoadingTags] = useState(false);
  const internalSearchInputRef = useRef<HTMLInputElement>(null);
  // Use external ref if provided, otherwise use internal ref
  const searchInputRef = inputRef || internalSearchInputRef;
  const tagInputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Helper to check if in any tag input mode
  const isTagInputModeActive = tagInputMode !== null;
  const isHostInputMode = tagInputMode === "host";

  // Tag search is only available if both callbacks are provided
  const tagSearchEnabled = Boolean(onSearchTags && onListTags);

  // Debounce the search
  useEffect(() => {
    const timer = setTimeout(() => {
      onChange(searchText);
    }, 300);

    return () => clearTimeout(timer);
  }, [searchText, onChange]);

  // Sync with external value
  useEffect(() => {
    setSearchText(value);
  }, [value]);

  // Handle tag suggestions based on tag query
  useEffect(() => {
    if (!isTagInputModeActive) {
      setShowTagDropdown(false);
      setTagSuggestions([]);
      return;
    }

    const fetchTags = async () => {
      setIsLoadingTags(true);
      try {
        let tags: Tag[] = [];
        // For host mode, prefix the query with $host:
        const searchPrefix = isHostInputMode ? "$host:" : "";
        const searchQuery = searchPrefix + tagQuery;

        if (tagQuery.length === 0 && onListTags) {
          // When no query, list tags with the prefix (or empty for regular tags)
          if (isHostInputMode && onSearchTags) {
            // For host mode with empty query, search with just the prefix
            tags = await onSearchTags(searchPrefix);
          } else if (!isHostInputMode) {
            tags = await onListTags();
          }
        } else if (tagQuery.length > 0 && onSearchTags) {
          tags = await onSearchTags(searchQuery);
        }

        // Filter based on mode
        let filteredTags: Tag[];
        if (isHostInputMode) {
          // For host mode, only show host tags (they should already have $host: prefix)
          filteredTags = tags.filter(
            (tag) => isHostTag(tag.text) && !filterTags.includes(tag.text)
          );
        } else {
          // For regular mode, filter out tags already in filterTags and system tags (starting with $)
          filteredTags = tags.filter(
            (tag) => !filterTags.includes(tag.text) && !isSystemTag(tag.text)
          );
        }
        setTagSuggestions(filteredTags);
        setShowTagDropdown(true);
        setSelectedTagIndex(-1);
      } catch (err) {
        console.error("Failed to fetch tags:", err);
        setTagSuggestions([]);
      } finally {
        setIsLoadingTags(false);
      }
    };

    const debounceTimer = setTimeout(fetchTags, 150);
    return () => clearTimeout(debounceTimer);
  }, [tagQuery, tagInputMode, isTagInputModeActive, isHostInputMode, onSearchTags, onListTags, filterTags]);

  // Focus tag input when entering tag input mode
  useEffect(() => {
    if (isTagInputModeActive && tagInputRef.current) {
      tagInputRef.current.focus();
    }
  }, [isTagInputModeActive]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node;
      const isOutsideDropdown = dropdownRef.current && !dropdownRef.current.contains(target);
      const isOutsideTagInput = tagInputRef.current && !tagInputRef.current.contains(target);
      const isOutsideSearchInput = searchInputRef.current && !searchInputRef.current.contains(target);

      if (isOutsideDropdown && isOutsideTagInput && isOutsideSearchInput) {
        setShowTagDropdown(false);
        // Exit tag input mode if clicking outside
        if (isTagInputModeActive) {
          setTagInputMode(null);
          setTagQuery("");
          // Focus search input after exiting tag mode
          setTimeout(() => searchInputRef.current?.focus(), 0);
        }
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isTagInputModeActive]);

  const handleSelectTag = useCallback(
    (tagText: string) => {
      onAddTag?.(tagText);
      setShowTagDropdown(false);
      setSelectedTagIndex(-1);
      setTagInputMode(null);
      setTagQuery("");
      // Focus back on search input after React re-renders
      setTimeout(() => searchInputRef.current?.focus(), 0);
    },
    [onAddTag]
  );

  const hasFilters = searchText || filterTags.length > 0 || isTagInputModeActive;

  const handleClearAll = () => {
    setSearchText("");
    setTagQuery("");
    setTagInputMode(null);
    onChange("");
    onClearAllTags?.();
  };

  // Handle search input changes - detect # or @ to enter tag mode (only if tag search is enabled)
  const handleSearchInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;

    // Only process tag triggers if tag search is enabled
    if (tagSearchEnabled) {
      const lastChar = newValue[newValue.length - 1];
      const secondLastChar = newValue[newValue.length - 2];
      const isAtEndOrAfterSpace = newValue.length === 1 || secondLastChar === " ";

      // Check if user just typed # at the end (or after a space) - enter regular tag mode
      if (lastChar === "#" && isAtEndOrAfterSpace) {
        setSearchText(newValue.slice(0, -1).trim());
        setTagInputMode("tag");
        setTagQuery("");
        return;
      }
      // Check if user just typed @ at the end (or after a space) - enter host tag mode
      if (lastChar === "@" && isAtEndOrAfterSpace) {
        setSearchText(newValue.slice(0, -1).trim());
        setTagInputMode("host");
        setTagQuery("");
        return;
      }
    }

    setSearchText(newValue);
  };

  const handleSearchInputKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      // If there's text or filter tags, clear them first
      if (searchText || filterTags.length > 0) {
        handleClearAll();
      }
      // Always blur the input on Escape
      searchInputRef.current?.blur();
    } else if (e.key === "Tab" && e.shiftKey && shiftTabCycleRef?.current) {
      // Shift+Tab cycles back to favorite toggle
      e.preventDefault();
      shiftTabCycleRef.current.focus();
    } else if (e.key === "Backspace") {
      // Remove last filter tag when backspace is pressed at the beginning of input
      const input = e.currentTarget;
      const cursorAtStart = input.selectionStart === 0 && input.selectionEnd === 0;
      if (cursorAtStart && filterTags.length > 0) {
        e.preventDefault();
        const lastTag = filterTags[filterTags.length - 1];
        onRemoveTag?.(lastTag);
      }
    }
  };

  const handleTagInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTagQuery(e.target.value);
  };

  const handleTagInputKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      setShowTagDropdown(false);
      setTagInputMode(null);
      setTagQuery("");
      // Blur the tag input (consistent with Escape blurring search bar controls)
      tagInputRef.current?.blur();
      return;
    }

    if (e.key === "Backspace" && tagQuery === "") {
      // Exit tag input mode when backspacing on empty tag query
      e.preventDefault();
      setTagInputMode(null);
      // Focus search input after React re-renders
      setTimeout(() => searchInputRef.current?.focus(), 0);
      return;
    }

    if (!showTagDropdown || tagSuggestions.length === 0) return;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedTagIndex((prev) =>
          prev < tagSuggestions.length - 1 ? prev + 1 : 0
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedTagIndex((prev) =>
          prev > 0 ? prev - 1 : tagSuggestions.length - 1
        );
        break;
      case "Enter":
        if (selectedTagIndex >= 0 && selectedTagIndex < tagSuggestions.length) {
          e.preventDefault();
          handleSelectTag(tagSuggestions[selectedTagIndex].text);
        }
        break;
      case "Tab":
        if (selectedTagIndex >= 0 && selectedTagIndex < tagSuggestions.length) {
          e.preventDefault();
          handleSelectTag(tagSuggestions[selectedTagIndex].text);
        }
        break;
    }
  };

  return (
    <div className="search-box">
      {label && <label className="search-box-label">{label}</label>}
      <div className="search-box-inner">
        {/* Filter tags - shown first */}
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
        {/* Tag input - shown when in tag input mode, after filter tags */}
        {isTagInputModeActive && (
          <div className={`search-tag-input-wrapper ${isHostInputMode ? "search-tag-input-wrapper-host" : ""}`}>
            {isHostInputMode ? (
              <svg
                className="search-tag-input-host-icon"
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
            ) : (
              <span className="search-tag-input-hash">#</span>
            )}
            <input
              ref={tagInputRef}
              type="text"
              value={tagQuery}
              onChange={handleTagInputChange}
              onKeyDown={handleTagInputKeyDown}
              className="search-tag-input"
              placeholder={isHostInputMode ? (t("search.hostPlaceholder") || "host name") : (t("search.tagPlaceholder") || "tag name")}
              spellCheck={false}
              autoCorrect="off"
              autoCapitalize="off"
            />
          </div>
        )}
        {/* Search input - shown after tags */}
        {(!isTagInputModeActive || searchText) && (
          <input
            ref={searchInputRef}
            type="text"
            placeholder={
              filterTags.length > 0
                ? t("search.placeholderWithTags")
                : tagSearchEnabled
                  ? t("search.placeholder")
                  : t("search.placeholderNoTagSearch")
            }
            value={searchText}
            onChange={handleSearchInputChange}
            onKeyDown={handleSearchInputKeyDown}
            className="search-input"
            spellCheck={false}
            autoCorrect="off"
            autoCapitalize="off"
          />
        )}
        {/* Tag dropdown */}
        {showTagDropdown && (onSearchTags || onListTags) && (
          <div className="search-tag-dropdown" ref={dropdownRef}>
            {isLoadingTags ? (
              <div className="search-tag-dropdown-loading">
                {t("search.loadingTags")}
              </div>
            ) : tagSuggestions.length === 0 ? (
              <div className="search-tag-dropdown-empty">
                {t("search.noTagsFound")}
              </div>
            ) : (
              tagSuggestions.map((tag, index) => (
                <div
                  key={tag.id}
                  className={`search-tag-dropdown-item ${index === selectedTagIndex ? "selected" : ""}`}
                  onClick={() => handleSelectTag(tag.text)}
                  onMouseEnter={() => setSelectedTagIndex(index)}
                >
                  {isHostInputMode ? (
                    <svg
                      className="search-tag-dropdown-item-host-icon"
                      width="14"
                      height="14"
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
                  ) : (
                    <span className="search-tag-dropdown-item-hash">#</span>
                  )}
                  {isHostInputMode ? getHostTagDisplay(tag.text) : tag.text}
                </div>
              ))
            )}
          </div>
        )}
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
