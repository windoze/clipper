// Types
export type {
  Clip,
  PagedResult,
  SearchFilters,
} from "./types";
export { FAVORITE_TAG, isFavorite } from "./types";

// API
export type { ClipperApi } from "./api";
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
  DateFilter,
  EditClipDialog,
  FavoriteToggle,
  ImagePopup,
  SearchBox,
  ToastProvider,
  useToast,
} from "./components";
export type { ToastType } from "./components";

// Hooks
export { useClips, useTheme } from "./hooks";
export type { Theme, ResolvedTheme } from "./hooks";
