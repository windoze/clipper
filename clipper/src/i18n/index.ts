import { createContext, useContext } from "react";
import { en, TranslationKey } from "./translations/en";
import { zh } from "./translations/zh";

export type Language = "en" | "zh";

export const translations: Record<Language, Record<string, string>> = {
  en,
  zh,
};

export const languageNames: Record<Language, string> = {
  en: "English",
  zh: "简体中文",
};

export const supportedLanguages: Language[] = ["en", "zh"];

// Detect system language
export function detectSystemLanguage(): Language {
  const browserLang = navigator.language.toLowerCase();

  // Check for Chinese variants
  if (browserLang.startsWith("zh")) {
    return "zh";
  }

  // Check for exact match
  for (const lang of supportedLanguages) {
    if (browserLang.startsWith(lang)) {
      return lang;
    }
  }

  // Default to English
  return "en";
}

// Translation function type
export type TranslateFunction = (key: TranslationKey, params?: Record<string, string | number>) => string;

// Create translation function for a specific language
export function createTranslator(language: Language): TranslateFunction {
  return (key: TranslationKey, params?: Record<string, string | number>): string => {
    let text = translations[language][key] || translations.en[key] || key;

    // Replace parameters like {count}, {error}, etc.
    if (params) {
      Object.entries(params).forEach(([paramKey, value]) => {
        text = text.replace(new RegExp(`\\{${paramKey}\\}`, "g"), String(value));
      });
    }

    return text;
  };
}

// Context
interface I18nContextValue {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: TranslateFunction;
}

export const I18nContext = createContext<I18nContextValue | null>(null);

// Hook
export function useI18n(): I18nContextValue {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider");
  }
  return context;
}

// Re-export TranslationKey type
export type { TranslationKey };
