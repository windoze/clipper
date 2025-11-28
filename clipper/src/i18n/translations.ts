import type { Language } from "@anthropic/clipper-ui";

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
    "settings.networkAccess": "Allow network access",
    "settings.networkAccess.restarting": "Restarting server...",
    "settings.networkAccess.hint": "When enabled, the server will listen on all network interfaces, allowing other devices on your network to access clips.",
    "settings.serverUrls": "Server URLs",
    "settings.serverUrls.empty": "No network interfaces found.",
    "settings.serverUrls.hint": "Use any of these URLs to access the server from other devices on your network.",
    "settings.serverUrl": "Server URL",
    "settings.serverUrl.placeholder": "http://localhost:3000",
    "settings.serverUrl.hint": "Enter the URL of your external clipper-server.",

    // Storage
    "settings.storage": "Storage",
    "settings.defaultSaveLocation": "Default Save Location",
    "settings.defaultSaveLocation.placeholder": "System default",
    "settings.defaultSaveLocation.hint": "Default folder for saving downloaded attachments.",
    "settings.browse": "Browse...",

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

    // Extra Toast Messages
    "toast.clipReceived": "New clip received",
    "toast.dataCleared": "All data cleared",
    "toast.serverStarted": "Server started",
    "toast.serverStopped": "Server stopped",
    "toast.serverConnected": "Connected to server",
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
    "settings.networkAccess": "允许网络访问",
    "settings.networkAccess.restarting": "重启服务器中...",
    "settings.networkAccess.hint": "启用后，服务器将监听所有网络接口，允许网络上的其他设备访问剪贴。",
    "settings.serverUrls": "服务器地址",
    "settings.serverUrls.empty": "未找到网络接口。",
    "settings.serverUrls.hint": "使用这些地址从网络上的其他设备访问服务器。",
    "settings.serverUrl": "服务器地址",
    "settings.serverUrl.placeholder": "http://localhost:3000",
    "settings.serverUrl.hint": "输入外部 clipper-server 的地址。",

    // Storage
    "settings.storage": "存储",
    "settings.defaultSaveLocation": "默认保存位置",
    "settings.defaultSaveLocation.placeholder": "系统默认",
    "settings.defaultSaveLocation.hint": "保存下载附件的默认文件夹。",
    "settings.browse": "浏览...",

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

    // Extra Toast Messages
    "toast.clipReceived": "收到新剪贴",
    "toast.dataCleared": "所有数据已清除",
    "toast.serverStarted": "服务器已启动",
    "toast.serverStopped": "服务器已停止",
    "toast.serverConnected": "已连接到服务器",
  },
};
