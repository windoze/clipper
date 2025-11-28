import { useState, useEffect, useMemo, ReactNode } from "react";
import {
  I18nContext,
  Language,
  createTranslator,
  detectSystemLanguage,
  supportedLanguages,
} from "./index";

const STORAGE_KEY = "clipper-web-language";

interface I18nProviderProps {
  children: ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  const [language, setLanguageState] = useState<Language>(() => {
    // Try to load from localStorage
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && supportedLanguages.includes(stored as Language)) {
      return stored as Language;
    }
    // Fall back to system language detection
    return detectSystemLanguage();
  });

  // Persist language changes to localStorage
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, language);
  }, [language]);

  const setLanguage = (lang: Language) => {
    setLanguageState(lang);
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

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
