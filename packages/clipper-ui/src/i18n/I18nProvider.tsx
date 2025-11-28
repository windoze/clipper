import { useState, useCallback, useMemo, useEffect, ReactNode } from "react";
import {
  I18nContext,
  Language,
  detectSystemLanguage,
  createTranslator,
} from "./index";

interface I18nProviderProps {
  children: ReactNode;
  /** Extra translations to merge with base translations (app-specific keys) */
  extraTranslations?: Record<Language, Record<string, string>>;
  /** Initial language override */
  initialLanguage?: Language | null;
  /** Storage key for persisting language preference */
  storageKey?: string;
  /** Custom language loader (e.g., from Tauri settings) */
  loadLanguage?: () => Promise<Language | null>;
  /** Custom language saver (e.g., to Tauri settings) */
  saveLanguage?: (lang: Language | null) => Promise<void>;
}

export function I18nProvider({
  children,
  extraTranslations,
  initialLanguage,
  storageKey = "clipper-language",
  loadLanguage,
  saveLanguage,
}: I18nProviderProps) {
  const [language, setLanguageState] = useState<Language>(() => {
    if (initialLanguage) return initialLanguage;
    // Try localStorage first
    if (typeof localStorage !== "undefined") {
      const saved = localStorage.getItem(storageKey);
      if (saved === "en" || saved === "zh") return saved;
    }
    return detectSystemLanguage();
  });

  const [isLoaded, setIsLoaded] = useState(!loadLanguage);

  // Load language from custom loader if provided
  useEffect(() => {
    if (loadLanguage) {
      loadLanguage().then((lang) => {
        if (lang) {
          setLanguageState(lang);
        }
        setIsLoaded(true);
      });
    }
  }, [loadLanguage]);

  const setLanguage = useCallback(
    (lang: Language) => {
      setLanguageState(lang);
      // Save to localStorage
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(storageKey, lang);
      }
      // Save via custom saver if provided
      if (saveLanguage) {
        saveLanguage(lang);
      }
    },
    [storageKey, saveLanguage]
  );

  const t = useMemo(
    () => createTranslator(language, extraTranslations),
    [language, extraTranslations]
  );

  const value = useMemo(
    () => ({ language, setLanguage, t }),
    [language, setLanguage, t]
  );

  // Wait for custom loader to complete
  if (!isLoaded) {
    return null;
  }

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
