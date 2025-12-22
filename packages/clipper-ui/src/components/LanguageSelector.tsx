import { useState, useRef, useEffect, useCallback } from "react";
import { useI18n } from "../i18n";

// Common programming languages for syntax highlighting
export const LANGUAGES = [
  { id: "plaintext", name: "Plain Text" },
  { id: "javascript", name: "JavaScript" },
  { id: "typescript", name: "TypeScript" },
  { id: "python", name: "Python" },
  { id: "java", name: "Java" },
  { id: "c", name: "C" },
  { id: "cpp", name: "C++" },
  { id: "csharp", name: "C#" },
  { id: "go", name: "Go" },
  { id: "rust", name: "Rust" },
  { id: "swift", name: "Swift" },
  { id: "kotlin", name: "Kotlin" },
  { id: "ruby", name: "Ruby" },
  { id: "php", name: "PHP" },
  { id: "html", name: "HTML" },
  { id: "css", name: "CSS" },
  { id: "scss", name: "SCSS" },
  { id: "json", name: "JSON" },
  { id: "xml", name: "XML" },
  { id: "yaml", name: "YAML" },
  { id: "markdown", name: "Markdown" },
  { id: "sql", name: "SQL" },
  { id: "shell", name: "Shell/Bash" },
  { id: "powershell", name: "PowerShell" },
  { id: "dockerfile", name: "Dockerfile" },
] as const;

export type LanguageId = (typeof LANGUAGES)[number]["id"];

interface LanguageSelectorProps {
  value: LanguageId;
  onChange: (language: LanguageId) => void;
  visible: boolean;
}

export function LanguageSelector({ value, onChange, visible }: LanguageSelectorProps) {
  const { t } = useI18n();
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const optionsRef = useRef<HTMLDivElement>(null);

  const selectedLanguage = LANGUAGES.find((l) => l.id === value) || LANGUAGES[0];
  const selectedIndex = LANGUAGES.findIndex((l) => l.id === value);

  // Reset highlighted index when dropdown opens
  useEffect(() => {
    if (isOpen) {
      // Start with the currently selected language highlighted
      setHighlightedIndex(selectedIndex >= 0 ? selectedIndex : 0);
    } else {
      setHighlightedIndex(-1);
    }
  }, [isOpen, selectedIndex]);

  // Scroll highlighted option into view
  useEffect(() => {
    if (isOpen && highlightedIndex >= 0 && optionsRef.current) {
      const options = optionsRef.current.querySelectorAll(".language-selector-option");
      const highlightedOption = options[highlightedIndex] as HTMLElement;
      if (highlightedOption) {
        highlightedOption.scrollIntoView({ block: "nearest" });
      }
    }
  }, [isOpen, highlightedIndex]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  // Close dropdown when parent clip entry is deactivated
  useEffect(() => {
    if (!visible && isOpen) {
      setIsOpen(false);
    }
  }, [visible, isOpen]);

  const handleSelect = useCallback((languageId: LanguageId) => {
    onChange(languageId);
    setIsOpen(false);
  }, [onChange]);

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(!isOpen);
  };

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!isOpen) {
      // When closed, Enter or Space or ArrowDown opens the dropdown
      if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
        e.preventDefault();
        e.stopPropagation();
        setIsOpen(true);
      }
      return;
    }

    // When open, handle navigation
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        e.stopPropagation();
        setHighlightedIndex((prev) =>
          prev < LANGUAGES.length - 1 ? prev + 1 : prev
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        e.stopPropagation();
        setHighlightedIndex((prev) => (prev > 0 ? prev - 1 : prev));
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        e.stopPropagation();
        if (highlightedIndex >= 0 && highlightedIndex < LANGUAGES.length) {
          handleSelect(LANGUAGES[highlightedIndex].id);
        }
        break;
      case "Escape":
        e.preventDefault();
        e.stopPropagation();
        setIsOpen(false);
        break;
      case "Tab":
        // Close dropdown and let focus move naturally to next/previous control
        setIsOpen(false);
        break;
      case "Home":
        e.preventDefault();
        e.stopPropagation();
        setHighlightedIndex(0);
        break;
      case "End":
        e.preventDefault();
        e.stopPropagation();
        setHighlightedIndex(LANGUAGES.length - 1);
        break;
    }
  }, [isOpen, highlightedIndex, handleSelect]);

  // Close dropdown when focus leaves the component entirely
  // React's onBlur on a container bubbles up from children (unlike native blur)
  const handleContainerBlur = useCallback((e: React.FocusEvent) => {
    // Check if the new focus target is outside this component
    // relatedTarget is the element receiving focus
    if (!dropdownRef.current?.contains(e.relatedTarget as Node)) {
      setIsOpen(false);
    }
  }, []);

  return (
    <div
      ref={dropdownRef}
      className={`language-selector ${visible || isOpen ? "visible" : ""} ${isOpen ? "open" : ""}`}
      onClick={(e) => e.stopPropagation()}
      onKeyDown={handleKeyDown}
      onBlur={handleContainerBlur}
    >
      <button
        className="language-selector-button"
        onClick={handleToggle}
        title={t("clip.selectLanguage")}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="16 18 22 12 16 6"></polyline>
          <polyline points="8 6 2 12 8 18"></polyline>
        </svg>
        <span className="language-selector-label">{selectedLanguage.name}</span>
        <svg
          className={`language-selector-arrow ${isOpen ? "open" : ""}`}
          width="10"
          height="10"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      {isOpen && (
        <div
          ref={optionsRef}
          className="language-selector-dropdown"
          role="listbox"
        >
          {LANGUAGES.map((lang, index) => (
            <button
              key={lang.id}
              className={`language-selector-option ${lang.id === value ? "selected" : ""} ${index === highlightedIndex ? "highlighted" : ""}`}
              onClick={() => handleSelect(lang.id)}
              onMouseEnter={() => setHighlightedIndex(index)}
              role="option"
              aria-selected={lang.id === value}
            >
              {lang.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
