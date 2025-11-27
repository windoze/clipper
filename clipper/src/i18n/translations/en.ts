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

  // Clip List
  "clipList.loading": "Loading clips...",
  "clipList.error": "Error: {error}",
  "clipList.empty": "No clips found",
  "clipList.loadingMore": "Loading more...",
  "clipList.noMore": "No more clips",

  // Search & Filters
  "search.placeholder": "Search clips...",
  "filter.from": "From:",
  "filter.to": "To:",
  "filter.startDate": "Start date",
  "filter.endDate": "End date",
  "filter.favorites": "Favorites only",

  // Settings Dialog
  "settings.title": "Settings",
  "settings.appearance": "Appearance",
  "settings.theme": "Theme",
  "settings.theme.light": "Light",
  "settings.theme.dark": "Dark",
  "settings.theme.auto": "Auto",
  "settings.theme.hint": "Choose your preferred color theme. Auto follows your system settings.",

  "settings.language": "Language",
  "settings.language.hint": "Choose your preferred language.",
  "settings.language.en": "English",
  "settings.language.zh": "Chinese (Simplified)",

  "settings.startup": "Startup",
  "settings.openOnStartup": "Open main window on startup",
  "settings.openOnStartup.hint": "Show the main window when the app starts. If disabled, the app will start minimized to the system tray.",
  "settings.startOnLogin": "Start application on login",
  "settings.startOnLogin.hint": "Automatically start Clipper when you log in to your computer.",

  "settings.server": "Server",
  "settings.serverMode": "Server Mode",
  "settings.serverMode.bundled": "Bundled",
  "settings.serverMode.external": "External",
  "settings.serverMode.hint.bundled": "Use the bundled server (automatically managed). Data is stored locally.",
  "settings.serverMode.hint.external": "Connect to an external clipper-server instance.",
  "settings.networkAccess": "Allow network access",
  "settings.networkAccess.restarting": "Restarting server...",
  "settings.networkAccess.hint": "When enabled, the server will listen on all network interfaces, allowing other devices on your network to access clips.",
  "settings.serverUrls": "Server URLs",
  "settings.serverUrls.empty": "No network interfaces found.",
  "settings.serverUrls.hint": "Use any of these URLs to access the server from other devices on your network.",
  "settings.serverUrl": "Server URL",
  "settings.serverUrl.placeholder": "http://localhost:3000",
  "settings.serverUrl.hint": "Enter the URL of your external clipper-server.",

  "settings.storage": "Storage",
  "settings.defaultSaveLocation": "Default Save Location",
  "settings.defaultSaveLocation.placeholder": "System default",
  "settings.defaultSaveLocation.hint": "Default folder for saving downloaded attachments.",
  "settings.browse": "Browse...",

  "settings.dataManagement": "Data Management",
  "settings.clearAllData": "Clear All Data",
  "settings.clearAllData.button": "Clear All Data",
  "settings.clearAllData.hint": "This will permanently delete all clips and attachments. This action cannot be undone.",
  "settings.clearAllData.confirm": "Are you sure? This will permanently delete all {count} clips and their attachments. This action cannot be undone.",
  "settings.clearAllData.clearing": "Clearing...",
  "settings.clearAllData.confirmButton": "Yes, Delete Everything",

  // Edit Clip Dialog
  "editClip.title": "Edit Clip",
  "editClip.tags": "Tags",
  "editClip.tags.placeholder": "Add tags...",
  "editClip.tags.hint": "Press Enter to add a tag, Backspace to remove the last one.",
  "editClip.notes": "Notes",
  "editClip.notes.placeholder": "Add notes about this clip...",
  "editClip.saveError": "Failed to save: {error}",

  // Drop Zone
  "dropZone.hint": "Drop files here to upload",

  // Image Popup
  "imagePopup.download": "Download",

  // Tooltips
  "tooltip.settings": "Settings",
  "tooltip.refresh": "Refresh",
  "tooltip.copy": "Copy to clipboard",

  // Tray Menu
  "tray.showHide": "Show/Hide Main Window",
  "tray.settings": "Settings...",
  "tray.quit": "Quit Application",

  // Notifications Settings
  "settings.notifications": "Show notifications",
  "settings.notifications.hint": "Show toast notifications for clipboard actions and sync events.",

  // Toast Messages
  "toast.clipCopied": "Copied to clipboard",
  "toast.clipSaved": "Clip saved",
  "toast.clipDeleted": "Clip deleted",
  "toast.clipReceived": "New clip received",
  "toast.dataCleared": "All data cleared",
  "toast.serverStarted": "Server started",
  "toast.serverStopped": "Server stopped",
  "toast.serverConnected": "Connected to server",
};

export type TranslationKey = keyof typeof en;
