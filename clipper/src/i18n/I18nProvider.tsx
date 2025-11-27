import { useState, useEffect, useMemo, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  I18nContext,
  Language,
  createTranslator,
  detectSystemLanguage,
  supportedLanguages,
} from "./index";

interface I18nProviderProps {
  children: ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  const [language, setLanguageState] = useState<Language>("en");
  const [initialized, setInitialized] = useState(false);

  // Load language from settings on mount
  useEffect(() => {
    const loadLanguage = async () => {
      try {
        const settings = await invoke<{ language?: string }>("get_settings");
        if (settings.language && supportedLanguages.includes(settings.language as Language)) {
          setLanguageState(settings.language as Language);
        } else {
          // Use system language as default
          setLanguageState(detectSystemLanguage());
        }
      } catch {
        // Fallback to system language detection
        setLanguageState(detectSystemLanguage());
      }
      setInitialized(true);
    };

    loadLanguage();
  }, []);

  const setLanguage = async (lang: Language) => {
    setLanguageState(lang);
    // Language will be saved when settings are saved
  };

  const t = useMemo(() => createTranslator(language), [language]);

  const value = useMemo(
    () => ({
      language,
      setLanguage,
      t,
    }),
    [language, t]
  );

  // Don't render until we've loaded the language preference
  if (!initialized) {
    return null;
  }

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
