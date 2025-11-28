import { useState, useEffect, useCallback } from "react";
import {
  useI18n,
  languageNames,
  supportedLanguages,
} from "@anthropic/clipper-ui";
import type { Language, Theme } from "@anthropic/clipper-ui";

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function SettingsDialog({
  isOpen,
  onClose,
  theme,
  onThemeChange,
}: SettingsDialogProps) {
  const { t, language, setLanguage } = useI18n();
  const [localTheme, setLocalTheme] = useState<Theme>(theme);
  const [localLanguage, setLocalLanguage] = useState<Language>(language);

  // Reset local state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setLocalTheme(theme);
      setLocalLanguage(language);
    }
  }, [isOpen, theme, language]);

  // Handle ESC key
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  const handleSave = useCallback(() => {
    onThemeChange(localTheme);
    setLanguage(localLanguage);
    onClose();
  }, [localTheme, localLanguage, onThemeChange, setLanguage, onClose]);

  if (!isOpen) return null;

  return (
    <div className="settings-backdrop" onClick={onClose}>
      <div className="settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>{t("settings.title")}</h2>
          <button className="settings-close" onClick={onClose}>
            &times;
          </button>
        </div>

        <div className="settings-content">
          {/* Appearance Section */}
          <div className="settings-section">
            <h3>{t("settings.appearance")}</h3>

            {/* Theme */}
            <div className="settings-field">
              <label>{t("settings.theme")}</label>
              <div className="theme-selector">
                <button
                  className={`theme-option ${localTheme === "light" ? "active" : ""}`}
                  onClick={() => setLocalTheme("light")}
                >
                  <span className="theme-icon">☀️</span>
                  <span>{t("settings.theme.light")}</span>
                </button>
                <button
                  className={`theme-option ${localTheme === "dark" ? "active" : ""}`}
                  onClick={() => setLocalTheme("dark")}
                >
                  <span className="theme-icon">🌙</span>
                  <span>{t("settings.theme.dark")}</span>
                </button>
                <button
                  className={`theme-option ${localTheme === "auto" ? "active" : ""}`}
                  onClick={() => setLocalTheme("auto")}
                >
                  <span className="theme-icon">💻</span>
                  <span>{t("settings.theme.auto")}</span>
                </button>
              </div>
              <p className="settings-hint">{t("settings.theme.hint")}</p>
            </div>

            {/* Language */}
            <div className="settings-field">
              <label>{t("settings.language")}</label>
              <select
                className="settings-select"
                value={localLanguage}
                onChange={(e) => setLocalLanguage(e.target.value as Language)}
              >
                {supportedLanguages.map((lang) => (
                  <option key={lang} value={lang}>
                    {languageNames[lang]}
                  </option>
                ))}
              </select>
              <p className="settings-hint">{t("settings.language.hint")}</p>
            </div>
          </div>
        </div>

        <div className="settings-footer">
          <button className="settings-btn secondary" onClick={onClose}>
            {t("common.cancel")}
          </button>
          <button className="settings-btn primary" onClick={handleSave}>
            {t("common.save")}
          </button>
        </div>
      </div>
    </div>
  );
}

// Hook to manage settings dialog state
export function useSettingsDialog() {
  const [isOpen, setIsOpen] = useState(false);

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);

  return { isOpen, open, close };
}
