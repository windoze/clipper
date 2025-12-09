import { useState, useRef, useEffect } from "react";
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
  const dropdownRef = useRef<HTMLDivElement>(null);

  const selectedLanguage = LANGUAGES.find((l) => l.id === value) || LANGUAGES[0];

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

  const handleSelect = (languageId: LanguageId) => {
    onChange(languageId);
    setIsOpen(false);
  };

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(!isOpen);
  };

  return (
    <div
      ref={dropdownRef}
      className={`language-selector ${visible || isOpen ? "visible" : ""} ${isOpen ? "open" : ""}`}
      onClick={(e) => e.stopPropagation()}
    >
      <button
        className="language-selector-button"
        onClick={handleToggle}
        title={t("clip.selectLanguage")}
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
        <div className="language-selector-dropdown">
          {LANGUAGES.map((lang) => (
            <button
              key={lang.id}
              className={`language-selector-option ${lang.id === value ? "selected" : ""}`}
              onClick={() => handleSelect(lang.id)}
            >
              {lang.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
