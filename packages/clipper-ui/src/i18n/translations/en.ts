export const en = {
  // App
  "app.title": "Clipper",
  "app.clips_count": "{count} clip(s)",

  // Common
  "common.save": "Save",
  "common.cancel": "Cancel",
  "common.delete": "Delete",
  "common.close": "Close",
  "common.loading": "Loading...",
  "common.error": "Error",
  "common.saving": "Saving...",
  "common.deleting": "Deleting...",

  // Clip Entry
  "clip.copy": "Click to copy",
  "clip.edit": "Edit clip",
  "clip.delete": "Delete clip",
  "clip.favorite.add": "Add to favorites",
  "clip.favorite.remove": "Remove from favorites",
  "clip.download": "Click to download",
  "clip.delete_confirm": "Are you sure you want to delete this clip?",
  "clip.expand": "Show more",
  "clip.collapse": "Show less",

  // Clip List
  "clipList.loading": "Loading clips...",
  "clipList.error": "Error: {error}",
  "clipList.empty": "No clips found",
  "clipList.loadingMore": "Loading more...",
  "clipList.noMore": "No more clips",

  // Search & Filters
  "search.placeholder": "Search clips...",
  "search.placeholderWithTags": "Search within filtered tags...",
  "filter.from": "From:",
  "filter.to": "To:",
  "filter.startDate": "Start date",
  "filter.endDate": "End date",
  "filter.favorites": "Favorites only",
  "filter.removeTag": "Remove tag filter",
  "filter.clearAll": "Clear all filters",
  "filter.clickToFilter": "Click to filter by this tag",

  // Settings (base settings shared by all apps)
  "settings.title": "Settings",
  "settings.appearance": "Appearance",
  "settings.theme": "Theme",
  "settings.theme.light": "Light",
  "settings.theme.dark": "Dark",
  "settings.theme.auto": "Auto",
  "settings.theme.hint":
    "Choose your preferred color theme. Auto follows your system settings.",
  "settings.language": "Language",
  "settings.language.hint": "Choose your preferred language.",
  "settings.language.en": "English",
  "settings.language.zh": "Chinese (Simplified)",

  // Edit Clip Dialog
  "editClip.title": "Edit Clip",
  "editClip.tags": "Tags",
  "editClip.tags.placeholder": "Add tags...",
  "editClip.tags.hint": "Press Enter to add a tag, Backspace to remove the last one.",
  "editClip.notes": "Notes",
  "editClip.notes.placeholder": "Add notes about this clip...",
  "editClip.saveError": "Failed to save: {error}",

  // Image Popup
  "imagePopup.download": "Download",

  // Tooltips
  "tooltip.settings": "Settings",
  "tooltip.refresh": "Refresh",
  "tooltip.copy": "Copy to clipboard",

  // Toast Messages
  "toast.clipCopied": "Copied to clipboard",
  "toast.clipSaved": "Clip saved",
  "toast.clipDeleted": "Clip deleted",
  "toast.copyFailed": "Failed to copy to clipboard",
  "toast.serverError": "Server connection error",
};

export type TranslationKey = keyof typeof en;
