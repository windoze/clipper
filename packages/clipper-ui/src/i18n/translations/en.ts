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
  "clip.share": "Share clip",
  "clip.edit": "Edit clip",
  "clip.delete": "Delete clip",
  "clip.favorite.add": "Add to favorites",
  "clip.favorite.remove": "Remove from favorites",
  "clip.download": "Click to download",
  "clip.delete_confirm": "Are you sure you want to delete this clip?",
  "clip.expand": "Show more",
  "clip.collapse": "Show less",
  "clip.selectLanguage": "Select syntax highlighting language",

  // Date Tag
  "dateTag.setStartDate": "Filter from this date",
  "dateTag.setEndDate": "Filter until this date",

  // Clip List
  "clipList.loading": "Loading clips...",
  "clipList.error": "Error: {error}",
  "clipList.empty": "No clips found",
  "clipList.loadingMore": "Loading more...",
  "clipList.noMore": "No more clips",

  // Search & Filters
  "search.label": "Search",
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
  "settings.syntaxTheme": "Code Syntax Theme",
  "settings.syntaxTheme.github": "GitHub",
  "settings.syntaxTheme.monokai": "Monokai",
  "settings.syntaxTheme.dracula": "Dracula",
  "settings.syntaxTheme.nord": "Nord",
  "settings.syntaxTheme.hint": "Choose the color theme for syntax highlighting in code snippets.",
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

  // Share Dialog
  "share.title": "Share Clip",
  "share.generating": "Generating share link...",
  "share.copy": "Copy",
  "share.copied": "Copied!",
  "share.hint": "Anyone with this link can view this clip.",
  "share.error": "Failed to generate share link",
  "share.notAvailable": "Sharing is not available",

  // Image Popup
  "imagePopup.download": "Download",

  // Tooltips
  "tooltip.settings": "Settings",
  "tooltip.refresh": "Refresh",
  "tooltip.copy": "Copy to clipboard",
  "tooltip.sendClipboard": "Send clipboard content",
  "tooltip.viewNotes": "View notes",

  // File Drop
  "fileDrop.hint": "Drop files here to upload",
  "fileDrop.uploading": "Uploading...",

  // Status Indicator
  "status.wsConnected": "Real-time sync",
  "status.wsDisconnected": "Disconnected",
  "status.wsUnavailable": "HTTPS required",

  // Toast Messages
  "toast.clipCopied": "Copied to clipboard",
  "toast.clipSaved": "Clip saved",
  "toast.clipDeleted": "Clip deleted",
  "toast.copyFailed": "Failed to copy to clipboard",
  "toast.serverError": "Server connection error",
  "toast.newClip": "New clip added",
  "toast.clipUpdated": "Clip updated",
  "toast.clipsCleanedUp": "{count} old clips cleaned up",
  "toast.wsConnected": "Real-time sync connected",
  "toast.wsDisconnected": "Real-time sync disconnected",
  "toast.fileUploaded": "File uploaded",
  "toast.uploadFailed": "Failed to upload file",
  "toast.clipboardSent": "Clipboard content sent",
  "toast.clipboardEmpty": "Clipboard is empty",
  "toast.clipboardReadFailed": "Failed to read clipboard",
  "toast.wsAuthFailed": "WebSocket authentication failed",

  // Connection Error
  "connectionError.title": "Unable to Connect",
  "connectionError.description": "We couldn't connect to the Clipper server. This could be due to:",
  "connectionError.reason.serverDown": "The server is not running or is unreachable",
  "connectionError.reason.networkIssue": "Network connectivity issues",
  "connectionError.reason.wrongUrl": "Incorrect server URL in settings",
  "connectionError.retry": "Try Again",
  "connectionError.openSettings": "Open Settings",
  "connectionError.hint": "If the problem persists, check if the server is running and verify your network connection.",

  // Authentication
  "auth.title": "Authentication Required",
  "auth.description": "This server requires authentication. Please enter your access token.",
  "auth.tokenLabel": "Access Token",
  "auth.tokenPlaceholder": "Enter your bearer token...",
  "auth.login": "Login",
  "auth.logout": "Logout",
  "auth.loggingIn": "Logging in...",
  "auth.error": "Authentication failed. Please check your token and try again.",
  "auth.invalidToken": "Invalid token",
  "auth.sessionExpired": "Session expired. Please login again.",

  // Certificate Trust Dialog
  "certificate.title": "Untrusted Certificate",
  "certificate.warning":
    "The server's certificate is not signed by a trusted Certificate Authority (CA).",
  "certificate.explanation": "This could mean:",
  "certificate.reason1": "The server is using a self-signed certificate",
  "certificate.reason2": "The server's CA is not in your system's trust store",
  "certificate.reason3": "Someone may be intercepting your connection (man-in-the-middle attack)",
  "certificate.host": "Server",
  "certificate.fingerprint": "SHA-256 Fingerprint",
  "certificate.fingerprintHint": "Verify this fingerprint with your server administrator",
  "certificate.hint":
    "If you trust this certificate, it will be saved and you won't be prompted again for this server.",
  "certificate.trust": "Trust Certificate",
  "certificate.trusting": "Trusting...",
  "toast.certificateTrusted": "Certificate trusted for {host}",
  "toast.certificateError": "Failed to verify certificate",

  // Certificate Mismatch Dialog (Critical MITM Warning)
  "certificateMismatch.title": "Security Warning: Certificate Changed",
  "certificateMismatch.criticalWarning": "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!",
  "certificateMismatch.warning":
    "The certificate fingerprint for this server has changed since you last connected. This is a serious security concern.",
  "certificateMismatch.explanation": "This could indicate:",
  "certificateMismatch.reason1": "Someone may be intercepting your connection (man-in-the-middle attack)",
  "certificateMismatch.reason2": "The server's certificate was legitimately renewed or replaced",
  "certificateMismatch.reason3": "You are connecting to a different server than before",
  "certificateMismatch.host": "Server",
  "certificateMismatch.storedFingerprint": "Previously Trusted Fingerprint",
  "certificateMismatch.newFingerprint": "New Fingerprint (Current)",
  "certificateMismatch.recommendation":
    "If you did not expect the certificate to change, do NOT proceed. Contact your server administrator to verify the new fingerprint before continuing.",
  "certificateMismatch.reject": "Disconnect (Recommended)",
  "certificateMismatch.acceptRisk": "Accept Risk & Continue",
  "certificateMismatch.accepting": "Accepting...",
  "toast.certificateMismatchDetected": "Certificate fingerprint mismatch detected for {host}",
};

export type TranslationKey = keyof typeof en;
