// Types
export type {
  Clip,
  PagedResult,
  SearchFilters,
  CleanupConfig,
} from "./types";
export { FAVORITE_TAG, isFavorite, calculateAgeRatio } from "./types";

// API
export type { ClipperApi, RestApiClient, RestApiClientOptions } from "./api";
export { ApiProvider, useApi, createRestApiClient } from "./api";

// i18n
export type { Language, TranslateFunction, TranslationKey } from "./i18n";
export {
  I18nContext,
  useI18n,
  translations,
  languageNames,
  supportedLanguages,
  detectSystemLanguage,
  createTranslator,
} from "./i18n";
export { I18nProvider } from "./i18n/I18nProvider";

// Components
export {
  ClipEntry,
  ClipList,
  ConnectionError,
  DateFilter,
  DateTag,
  EditClipDialog,
  FavoriteToggle,
  ImagePopup,
  SearchBox,
  ToastProvider,
  useToast,
} from "./components";
export type { ToastType } from "./components";

// Hooks
export {
  useClips,
  useTheme,
  useCleanupConfig,
  CleanupConfigProvider,
  useSyntaxTheme,
  useSyntaxThemeContext,
  SyntaxThemeProvider,
  SYNTAX_THEMES,
} from "./hooks";
export type { Theme, ResolvedTheme, SyntaxTheme } from "./hooks";
