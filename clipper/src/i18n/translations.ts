import type { Language } from "@unwritten-codes/clipper-ui";

// Extra translations specific to the Tauri desktop app
export const tauriExtraTranslations: Record<Language, Record<string, string>> = {
  en: {
    // Startup
    "settings.startup": "Startup",
    "settings.openOnStartup": "Open main window on startup",
    "settings.openOnStartup.hint": "Show the main window when the app starts. If disabled, the app will start minimized to the system tray.",
    "settings.startOnLogin": "Start application on login",
    "settings.startOnLogin.hint": "Automatically start Clipper when you log in to your computer.",

    // Server
    "settings.server": "Server",
    "settings.serverMode": "Server Mode",
    "settings.serverMode.bundled": "Bundled",
    "settings.serverMode.external": "External",
    "settings.serverMode.hint.bundled": "Use the bundled server (automatically managed). Data is stored locally.",
    "settings.serverMode.hint.external": "Connect to an external clipper-server instance.",
    "settings.serverMode.switching": "Switching server mode...",
    "settings.networkAccess": "Allow network access",
    "settings.networkAccess.restarting": "Restarting server...",
    "settings.networkAccess.hint": "When enabled, the server will listen on all network interfaces, allowing other devices on your network to access clips.",
    "settings.serverUrls": "Server URLs",
    "settings.serverUrls.empty": "No network interfaces found.",
    "settings.serverUrls.hint": "Use any of these URLs to access the server from other devices on your network.",
    "settings.serverUrl": "Server URL",
    "settings.serverUrl.placeholder": "http://localhost:3000",
    "settings.serverUrl.hint": "Enter the URL of your external clipper-server.",
    "settings.serverToken": "Access Token",
    "settings.serverToken.placeholder": "Enter token (optional)",
    "settings.serverToken.hint": "Bearer token for authenticating with the external server. Leave empty if the server doesn't require authentication.",
    "settings.bundledServerToken": "Access Token",
    "settings.bundledServerToken.placeholder": "Enter token (optional)",
    "settings.bundledServerToken.hint": "Set a token to require authentication when accessing the server from other devices. Leave empty to allow unauthenticated access.",
    "settings.token.show": "Show token",
    "settings.token.hide": "Hide token",

    // Storage
    "settings.storage": "Storage",
    "settings.defaultSaveLocation": "Default Save Location",
    "settings.defaultSaveLocation.placeholder": "System default",
    "settings.defaultSaveLocation.hint": "Default folder for saving downloaded attachments.",
    "settings.browse": "Browse...",

    // Auto-cleanup
    "settings.cleanup": "Auto-cleanup old clips",
    "settings.cleanup.hint": "Automatically delete old clips based on retention period. Clips marked as favorites or with user-defined tags are never deleted.",
    "settings.cleanup.retentionDays": "Retention Period (days)",
    "settings.cleanup.retentionDays.hint": "Clips older than this will be automatically deleted. Range: 1-365 days.",
    "settings.cleanup.restartNotice": "Server will restart when you close settings to apply cleanup changes.",

    // Data Management
    "settings.dataManagement": "Data Management",
    "settings.clearAllData": "Clear All Data",
    "settings.clearAllData.button": "Clear All Data",
    "settings.clearAllData.hint": "This will permanently delete all clips and attachments. This action cannot be undone.",
    "settings.clearAllData.confirm": "Are you sure? This will permanently delete all {count} clips and their attachments. This action cannot be undone.",
    "settings.clearAllData.clearing": "Clearing...",
    "settings.clearAllData.confirmButton": "Yes, Delete Everything",

    // Drop Zone
    "dropZone.hint": "Drop files here to upload",

    // Tray Menu
    "tray.showHide": "Show/Hide Main Window",
    "tray.settings": "Settings...",
    "tray.quit": "Quit Application",

    // Notifications Settings
    "settings.notifications": "Show notifications",
    "settings.notifications.hint": "Show toast notifications for clipboard actions and sync events.",

    // Global Shortcut Settings
    "settings.globalShortcut": "Global Shortcut",
    "settings.globalShortcut.hint": "Keyboard shortcut to show/hide the main window from anywhere.",
    "settings.globalShortcut.recording": "Press keys...",
    "settings.shortcut.updated": "Shortcut updated",
    "settings.shortcut.error": "Failed to set shortcut",

    // Connection Error - Tauri specific additions
    "connectionError.reason.bundledServer": "The bundled server failed to start",
    "connectionError.checkingServer": "Checking server...",

    // Extra Toast Messages
    "toast.clipReceived": "New clip received",
    "toast.clipsCleanedUp": "{count} old clips cleaned up",
    "toast.dataCleared": "All data cleared",
    "toast.serverStarted": "Server started",
    "toast.serverStopped": "Server stopped",
    "toast.serverConnected": "Connected to server",
    "toast.serverRestarted": "Server restarted with new settings",

    // File Upload Errors
    "toast.fileTooLarge": "File too large: {filename} ({size} MB). Maximum size is {maxSize} MB.",
    "toast.fileUploadFailed": "Failed to upload file: {filename}",

    // Max Upload Size
    "settings.maxUploadSize": "Maximum Upload Size (MB)",
    "settings.maxUploadSize.hint": "Maximum file size allowed for uploads. Larger files will be rejected.",
    "settings.maxUploadSize.externalHint": "This value is configured on the external server and cannot be changed here.",
  },
  zh: {
    // Startup
    "settings.startup": "启动",
    "settings.openOnStartup": "启动时打开主窗口",
    "settings.openOnStartup.hint": "应用启动时显示主窗口。如果禁用，应用将最小化到系统托盘。",
    "settings.startOnLogin": "登录时启动应用",
    "settings.startOnLogin.hint": "登录计算机时自动启动 Clipper。",

    // Server
    "settings.server": "服务器",
    "settings.serverMode": "服务器模式",
    "settings.serverMode.bundled": "内置",
    "settings.serverMode.external": "外部",
    "settings.serverMode.hint.bundled": "使用内置服务器（自动管理）。数据存储在本地。",
    "settings.serverMode.hint.external": "连接到外部 clipper-server 实例。",
    "settings.serverMode.switching": "正在切换服务器模式...",
    "settings.networkAccess": "允许网络访问",
    "settings.networkAccess.restarting": "重启服务器中...",
    "settings.networkAccess.hint": "启用后，服务器将监听所有网络接口，允许网络上的其他设备访问剪贴。",
    "settings.serverUrls": "服务器地址",
    "settings.serverUrls.empty": "未找到网络接口。",
    "settings.serverUrls.hint": "使用这些地址从网络上的其他设备访问服务器。",
    "settings.serverUrl": "服务器地址",
    "settings.serverUrl.placeholder": "http://localhost:3000",
    "settings.serverUrl.hint": "输入外部 clipper-server 的地址。",
    "settings.serverToken": "访问令牌",
    "settings.serverToken.placeholder": "输入令牌（可选）",
    "settings.serverToken.hint": "用于外部服务器身份验证的令牌。如果服务器不需要身份验证，请留空。",
    "settings.bundledServerToken": "访问令牌",
    "settings.bundledServerToken.placeholder": "输入令牌（可选）",
    "settings.bundledServerToken.hint": "设置令牌以在其他设备访问服务器时要求身份验证。留空则允许无需身份验证即可访问。",
    "settings.token.show": "显示令牌",
    "settings.token.hide": "隐藏令牌",

    // Storage
    "settings.storage": "存储",
    "settings.defaultSaveLocation": "默认保存位置",
    "settings.defaultSaveLocation.placeholder": "系统默认",
    "settings.defaultSaveLocation.hint": "保存下载附件的默认文件夹。",
    "settings.browse": "浏览...",

    // Auto-cleanup
    "settings.cleanup": "自动清理旧剪贴",
    "settings.cleanup.hint": "根据保留期限自动删除旧剪贴。标记为收藏或带有用户定义标签的剪贴不会被自动删除。",
    "settings.cleanup.retentionDays": "保留期限（天）",
    "settings.cleanup.retentionDays.hint": "超过此天数的剪贴将被自动删除。范围：1-365 天。",
    "settings.cleanup.restartNotice": "关闭设置后将重启服务器以应用清理设置更改。",

    // Data Management
    "settings.dataManagement": "数据管理",
    "settings.clearAllData": "清除所有数据",
    "settings.clearAllData.button": "清除所有数据",
    "settings.clearAllData.hint": "这将永久删除所有剪贴和附件。此操作无法撤销。",
    "settings.clearAllData.confirm": "确定吗？这将永久删除所有 {count} 条剪贴及其附件。此操作无法撤销。",
    "settings.clearAllData.clearing": "清除中...",
    "settings.clearAllData.confirmButton": "是的，删除全部",

    // Drop Zone
    "dropZone.hint": "拖放文件到此处上传",

    // Tray Menu
    "tray.showHide": "显示/隐藏主窗口",
    "tray.settings": "设置...",
    "tray.quit": "退出应用",

    // Notifications Settings
    "settings.notifications": "显示通知",
    "settings.notifications.hint": "显示剪贴板操作和同步事件的通知。",

    // Global Shortcut Settings
    "settings.globalShortcut": "全局快捷键",
    "settings.globalShortcut.hint": "从任何位置显示/隐藏主窗口的快捷键。",
    "settings.globalShortcut.recording": "请按下快捷键...",
    "settings.shortcut.updated": "快捷键已更新",
    "settings.shortcut.error": "设置快捷键失败",

    // Connection Error - Tauri specific additions
    "connectionError.reason.bundledServer": "内置服务器启动失败",
    "connectionError.checkingServer": "检查服务器中...",

    // Extra Toast Messages
    "toast.clipReceived": "收到新剪贴",
    "toast.clipsCleanedUp": "已清理 {count} 条旧剪贴",
    "toast.dataCleared": "所有数据已清除",
    "toast.serverStarted": "服务器已启动",
    "toast.serverStopped": "服务器已停止",
    "toast.serverConnected": "已连接到服务器",
    "toast.serverRestarted": "服务器已重启以应用新设置",

    // File Upload Errors
    "toast.fileTooLarge": "文件过大：{filename}（{size} MB）。最大允许 {maxSize} MB。",
    "toast.fileUploadFailed": "上传文件失败：{filename}",

    // Max Upload Size
    "settings.maxUploadSize": "最大上传大小 (MB)",
    "settings.maxUploadSize.hint": "允许上传的最大文件大小。超过此大小的文件将被拒绝。",
    "settings.maxUploadSize.externalHint": "此值在外部服务器上配置，无法在此处更改。",
  },
};
