import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  useI18n,
  useToast,
  supportedLanguages,
  languageNames,
  SYNTAX_THEMES,
} from "@unwritten-codes/clipper-ui";
import type { Language, SyntaxTheme } from "@unwritten-codes/clipper-ui";

export type ThemePreference = "light" | "dark" | "auto";

export interface SettingsWindowGeometry {
  width: number | null;
  height: number | null;
  x: number | null;
  y: number | null;
}

export interface Settings {
  serverAddress: string;
  defaultSaveLocation: string | null;
  openOnStartup: boolean;
  startOnLogin: boolean;
  theme: ThemePreference;
  syntaxTheme: SyntaxTheme;
  useBundledServer: boolean;
  listenOnAllInterfaces: boolean;
  language: string | null;
  notificationsEnabled: boolean;
  globalShortcut: string;
  cleanupEnabled: boolean;
  cleanupRetentionDays: number;
  externalServerToken: string | null;
  bundledServerToken: string | null;
  maxUploadSizeMb: number;
  settingsWindowGeometry: SettingsWindowGeometry;
}

interface ServerInfo {
  version: string;
  uptime_secs: number;
  active_ws_connections: number;
  config: {
    port: number;
    tls_enabled: boolean;
    tls_port?: number;
    acme_enabled: boolean;
    acme_domain?: string;
    cleanup_enabled: boolean;
    cleanup_interval_mins?: number;
    cleanup_retention_days?: number;
    auth_required: boolean;
    max_upload_size_bytes: number;
  };
}

interface UpdateInfo {
  version: string;
  current_version: string;
  body: string | null;
  date: string | null;
}

type SettingsTab = "appearance" | "startup" | "server" | "about";

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onThemeChange?: (theme: ThemePreference) => void;
  onSyntaxThemeChange?: (theme: SyntaxTheme) => void;
}

export function SettingsDialog({ isOpen, onClose, onThemeChange, onSyntaxThemeChange }: SettingsDialogProps) {
  const { t, language: currentLanguage, setLanguage } = useI18n();
  const { showToast } = useToast();
  // Detect platform for default shortcut
  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const defaultShortcut = isMac ? "Command+Shift+V" : "Ctrl+Shift+V";

  const [activeTab, setActiveTab] = useState<SettingsTab>("appearance");
  const [settings, setSettings] = useState<Settings>({
    serverAddress: "http://localhost:3000",
    defaultSaveLocation: null,
    openOnStartup: true,
    startOnLogin: false,
    theme: "auto",
    syntaxTheme: "github",
    useBundledServer: true,
    listenOnAllInterfaces: false,
    language: null,
    notificationsEnabled: true,
    globalShortcut: defaultShortcut,
    cleanupEnabled: false,
    cleanupRetentionDays: 30,
    externalServerToken: null,
    bundledServerToken: null,
    maxUploadSizeMb: 10,
    settingsWindowGeometry: { width: null, height: null, x: null, y: null },
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [serverUrl, setServerUrl] = useState<string>("");
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [localIpAddresses, setLocalIpAddresses] = useState<string[]>([]);
  const [togglingNetworkAccess, setTogglingNetworkAccess] = useState(false);
  const [switchingServerMode, setSwitchingServerMode] = useState(false);
  // Password visibility toggles
  const [showBundledToken, setShowBundledToken] = useState(false);
  const [showExternalToken, setShowExternalToken] = useState(false);
  // Track the original server address to detect changes on close
  const [originalServerAddress, setOriginalServerAddress] = useState<string>("");
  // Track original cleanup settings to detect changes (requires server restart)
  const [originalCleanupEnabled, setOriginalCleanupEnabled] = useState(false);
  const [originalCleanupRetentionDays, setOriginalCleanupRetentionDays] = useState(30);
  // Track original token values to detect changes on close
  const [originalExternalServerToken, setOriginalExternalServerToken] = useState<string | null>(null);
  const [originalBundledServerToken, setOriginalBundledServerToken] = useState<string | null>(null);
  // Track original max upload size
  const [originalMaxUploadSizeMb, setOriginalMaxUploadSizeMb] = useState(10);
  // External server info (read-only, fetched from server)
  const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null);
  // Shortcut recording state
  const [isRecordingShortcut, setIsRecordingShortcut] = useState(false);
  const [recordedKeys, setRecordedKeys] = useState<string[]>([]);
  const shortcutInputRef = useRef<HTMLDivElement>(null);
  // Update state
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checkingForUpdates, setCheckingForUpdates] = useState(false);
  const [installingUpdate, setInstallingUpdate] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [updateReady, setUpdateReady] = useState(false);
  const [updateChecked, setUpdateChecked] = useState(false);
  // App version for About tab
  const [appVersion, setAppVersion] = useState<string>("");
  // Dialog resizing state
  const dialogRef = useRef<HTMLDivElement>(null);
  const [isResizing, setIsResizing] = useState(false);
  const [resizeEdge, setResizeEdge] = useState<string | null>(null);
  const justFinishedResizing = useRef(false);

  // Load settings when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadSettings();
      loadServerInfo();
      loadLocalIpAddresses();
      loadAppVersion();
    }
  }, [isOpen]);

  // Apply saved window geometry when dialog opens
  useEffect(() => {
    if (isOpen && dialogRef.current && settings.settingsWindowGeometry) {
      const { width, height } = settings.settingsWindowGeometry;
      if (width && height) {
        dialogRef.current.style.width = `${width}px`;
        dialogRef.current.style.height = `${height}px`;
      }
    }
  }, [isOpen, settings.settingsWindowGeometry]);

  // Handle ESC key to close dialog
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isRecordingShortcut) {
        e.preventDefault();
        handleClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, isRecordingShortcut]);

  // Handle mouse up to end resizing
  useEffect(() => {
    if (!isResizing) return;

    const handleMouseUp = () => {
      setIsResizing(false);
      setResizeEdge(null);
      // Set flag to prevent backdrop click from closing the dialog
      justFinishedResizing.current = true;
      setTimeout(() => {
        justFinishedResizing.current = false;
      }, 100);
      // Save the new geometry
      if (dialogRef.current) {
        const rect = dialogRef.current.getBoundingClientRect();
        const newGeometry: SettingsWindowGeometry = {
          width: Math.round(rect.width),
          height: Math.round(rect.height),
          x: null,
          y: null,
        };
        const newSettings = { ...settings, settingsWindowGeometry: newGeometry };
        setSettings(newSettings);
        saveSettings(newSettings);
      }
    };

    const handleMouseMove = (e: MouseEvent) => {
      if (!dialogRef.current || !resizeEdge) return;

      const rect = dialogRef.current.getBoundingClientRect();
      const minWidth = 500;
      const minHeight = 450;
      const maxWidth = window.innerWidth * 0.95;
      const maxHeight = window.innerHeight * 0.95;

      if (resizeEdge.includes('e')) {
        const newWidth = Math.min(maxWidth, Math.max(minWidth, e.clientX - rect.left));
        dialogRef.current.style.width = `${newWidth}px`;
      }
      if (resizeEdge.includes('w')) {
        const newWidth = Math.min(maxWidth, Math.max(minWidth, rect.right - e.clientX));
        dialogRef.current.style.width = `${newWidth}px`;
      }
      if (resizeEdge.includes('s')) {
        const newHeight = Math.min(maxHeight, Math.max(minHeight, e.clientY - rect.top));
        dialogRef.current.style.height = `${newHeight}px`;
      }
      if (resizeEdge.includes('n')) {
        const newHeight = Math.min(maxHeight, Math.max(minHeight, rect.bottom - e.clientY));
        dialogRef.current.style.height = `${newHeight}px`;
      }
    };

    window.addEventListener("mouseup", handleMouseUp);
    window.addEventListener("mousemove", handleMouseMove);
    return () => {
      window.removeEventListener("mouseup", handleMouseUp);
      window.removeEventListener("mousemove", handleMouseMove);
    };
  }, [isResizing, resizeEdge, settings]);

  const loadAppVersion = async () => {
    try {
      const version = await invoke<string>("get_app_version");
      setAppVersion(version);
    } catch (e) {
      console.error("Failed to load app version:", e);
    }
  };

  const loadSettings = async () => {
    setLoading(true);
    setError(null);
    try {
      const loadedSettings = await invoke<Settings>("get_settings");
      setSettings(loadedSettings);
      // Store the original server address to detect changes on close
      setOriginalServerAddress(loadedSettings.serverAddress);
      // Store original cleanup settings to detect changes
      setOriginalCleanupEnabled(loadedSettings.cleanupEnabled);
      setOriginalCleanupRetentionDays(loadedSettings.cleanupRetentionDays);
      // Store original token values to detect changes
      setOriginalExternalServerToken(loadedSettings.externalServerToken);
      setOriginalBundledServerToken(loadedSettings.bundledServerToken);
      // Store original max upload size
      setOriginalMaxUploadSizeMb(loadedSettings.maxUploadSizeMb);
    } catch (e) {
      setError(`Failed to load settings: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const loadServerInfo = async () => {
    try {
      const url = await invoke<string>("get_server_url");
      setServerUrl(url);
      // Fetch server info
      const info = await invoke<ServerInfo>("get_server_info");
      setServerInfo(info);
    } catch (e) {
      console.error("Failed to load server info:", e);
      setServerInfo(null);
    }
  };

  const loadLocalIpAddresses = async () => {
    try {
      const ips = await invoke<string[]>("get_local_ip_addresses");
      setLocalIpAddresses(ips);
    } catch (e) {
      console.error("Failed to load local IP addresses:", e);
    }
  };

  // Save settings immediately
  const saveSettings = async (newSettings: Settings) => {
    try {
      await invoke("save_settings", { settings: newSettings });
    } catch (e) {
      setError(`Failed to save settings: ${e}`);
    }
  };

  const handleBrowseDirectory = async () => {
    try {
      const path = await invoke<string | null>("browse_directory");
      if (path) {
        const newSettings = { ...settings, defaultSaveLocation: path };
        setSettings(newSettings);
        await saveSettings(newSettings);
      }
    } catch (e) {
      setError(`Failed to browse directory: ${e}`);
    }
  };

  const handleChange = async (
    field: keyof Settings,
    value: string | boolean | number | null | SettingsWindowGeometry
  ) => {
    const newSettings = { ...settings, [field]: value };
    setSettings(newSettings);

    // Apply theme change immediately
    if (field === "theme" && typeof value === "string") {
      onThemeChange?.(value as ThemePreference);
    }

    // Apply syntax theme change immediately
    if (field === "syntaxTheme" && typeof value === "string") {
      onSyntaxThemeChange?.(value as SyntaxTheme);
    }

    // Save settings immediately
    await saveSettings(newSettings);
  };

  // Handle close - reconnect if server URL or token changed while using external server
  // or restart bundled server if cleanup settings or token changed
  const handleClose = async () => {
    setShowClearConfirm(false);

    // If using external server and the server address or token changed, reconnect
    const externalServerChanged = !settings.useBundledServer && (
      settings.serverAddress !== originalServerAddress ||
      settings.externalServerToken !== originalExternalServerToken
    );
    if (externalServerChanged) {
      try {
        // switch_to_external_server returns null if connected, or an error message if not
        const connectionError = await invoke<string | null>("switch_to_external_server", { serverUrl: settings.serverAddress });
        setServerUrl(settings.serverAddress);
        if (connectionError) {
          showToast(connectionError, "error");
        } else {
          showToast(t("toast.serverConnected"));
        }
      } catch (e) {
        console.error("Failed to reconnect to server:", e);
      }
    }

    // If using bundled server and cleanup settings, token, or max upload size changed, restart the server
    const bundledServerNeedsRestart = settings.useBundledServer && (
      settings.cleanupEnabled !== originalCleanupEnabled ||
      settings.cleanupRetentionDays !== originalCleanupRetentionDays ||
      settings.maxUploadSizeMb !== originalMaxUploadSizeMb ||
      (settings.listenOnAllInterfaces && settings.bundledServerToken !== originalBundledServerToken)
    );
    if (bundledServerNeedsRestart) {
      try {
        // Restart by switching to bundled server again
        const newUrl = await invoke<string>("switch_to_bundled_server");
        setServerUrl(newUrl);
        showToast(t("toast.serverRestarted"));
      } catch (e) {
        console.error("Failed to restart server with new settings:", e);
      }
    }

    onClose();
  };

  // Handle language change - save immediately
  const handleLanguageChange = async (lang: Language) => {
    setLanguage(lang);
    // Save settings with new language
    const newSettings = { ...settings, language: lang };
    setSettings(newSettings);
    await saveSettings(newSettings);
  };

  // Handle server mode change
  const handleServerModeChange = async (useBundled: boolean) => {
    if (useBundled === settings.useBundledServer) return;

    setError(null);
    setSwitchingServerMode(true);
    try {
      if (useBundled) {
        // Switch to bundled server
        const newUrl = await invoke<string>("switch_to_bundled_server");
        setServerUrl(newUrl);
        showToast(t("toast.serverStarted"));
      } else {
        // Switch to external server
        // Returns null if connected successfully, or an error message if unreachable
        const connectionError = await invoke<string | null>("switch_to_external_server", { serverUrl: settings.serverAddress });
        setServerUrl(settings.serverAddress);
        if (connectionError) {
          showToast(connectionError, "error");
        } else {
          showToast(t("toast.serverConnected"));
        }
      }
      // Reload settings from backend to ensure frontend is in sync
      // This is important because the switch commands update settings on the backend
      const loadedSettings = await invoke<Settings>("get_settings");
      setSettings(loadedSettings);
    } catch (e) {
      setError(`Failed to switch server mode: ${e}`);
    } finally {
      setSwitchingServerMode(false);
    }
  };

  // Handle clear all data
  const handleClearData = async () => {
    setClearing(true);
    setError(null);
    try {
      await invoke("clear_all_data");
      setShowClearConfirm(false);
      showToast(t("toast.dataCleared"));
      // Close the dialog after successful clear
      onClose();
    } catch (e) {
      setError(`Failed to clear data: ${e}`);
    } finally {
      setClearing(false);
    }
  };

  // Handle toggling network access
  const handleToggleNetworkAccess = async (listenOnAll: boolean) => {
    if (listenOnAll === settings.listenOnAllInterfaces) return;

    setTogglingNetworkAccess(true);
    setError(null);
    try {
      const newUrl = await invoke<string>("toggle_listen_on_all_interfaces", {
        listenOnAll,
      });
      setServerUrl(newUrl);
      // Reload settings from backend to ensure frontend is in sync
      const loadedSettings = await invoke<Settings>("get_settings");
      setSettings(loadedSettings);
    } catch (e) {
      setError(`Failed to toggle network access: ${e}`);
    } finally {
      setTogglingNetworkAccess(false);
    }
  };

  // Get the port from the server URL
  const getServerPort = () => {
    try {
      const url = new URL(serverUrl);
      return url.port || "3000";
    } catch {
      return "3000";
    }
  };

  // Handle shortcut recording
  const handleShortcutKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!isRecordingShortcut) return;

    e.preventDefault();
    e.stopPropagation();

    const keys: string[] = [];

    // Add modifiers in a consistent order
    if (e.metaKey) keys.push(isMac ? "Command" : "Super");
    if (e.ctrlKey) keys.push("Ctrl");
    if (e.altKey) keys.push(isMac ? "Option" : "Alt");
    if (e.shiftKey) keys.push("Shift");

    // Get the main key
    const key = e.key;
    if (!["Control", "Shift", "Alt", "Meta", "OS"].includes(key)) {
      // Handle special keys
      let keyName = key;
      if (key.length === 1) {
        keyName = key.toUpperCase();
      } else if (key === " ") {
        keyName = "Space";
      } else if (key === "ArrowUp") {
        keyName = "Up";
      } else if (key === "ArrowDown") {
        keyName = "Down";
      } else if (key === "ArrowLeft") {
        keyName = "Left";
      } else if (key === "ArrowRight") {
        keyName = "Right";
      }
      keys.push(keyName);
    }

    setRecordedKeys(keys);
  }, [isRecordingShortcut, isMac]);

  const handleShortcutKeyUp = useCallback(async (_e: React.KeyboardEvent) => {
    if (!isRecordingShortcut) return;

    // If we have a complete shortcut (at least one modifier + one key)
    if (recordedKeys.length >= 2) {
      const shortcutStr = recordedKeys.join("+");

      // Try to update the shortcut
      try {
        await invoke("update_global_shortcut", { shortcut: shortcutStr });
        const newSettings = { ...settings, globalShortcut: shortcutStr };
        setSettings(newSettings);
        await saveSettings(newSettings);
        setIsRecordingShortcut(false);
        setRecordedKeys([]);
        showToast(t("settings.shortcut.updated"));
      } catch (err) {
        setError(`${t("settings.shortcut.error")}: ${err}`);
        setRecordedKeys([]);
      }
    }
  }, [isRecordingShortcut, recordedKeys, settings, showToast, t]);

  const startRecordingShortcut = () => {
    setIsRecordingShortcut(true);
    setRecordedKeys([]);
    setError(null);
    // Focus the input after state update
    setTimeout(() => {
      shortcutInputRef.current?.focus();
    }, 0);
  };

  const cancelRecordingShortcut = () => {
    setIsRecordingShortcut(false);
    setRecordedKeys([]);
  };

  // Check for updates
  const handleCheckForUpdates = async () => {
    setCheckingForUpdates(true);
    setUpdateError(null);
    setUpdateInfo(null);
    setUpdateChecked(false);
    try {
      const info = await invoke<UpdateInfo | null>("check_for_updates");
      setUpdateInfo(info);
      setUpdateChecked(true);
      if (info) {
        showToast(t("toast.updateAvailable").replace("{version}", info.version));
      }
    } catch (e) {
      setUpdateError(String(e));
    } finally {
      setCheckingForUpdates(false);
    }
  };

  // Install update
  const handleInstallUpdate = async () => {
    setInstallingUpdate(true);
    setUpdateError(null);
    try {
      await invoke("install_update");
      // The update-ready event will be emitted by the backend
    } catch (e) {
      setUpdateError(String(e));
      setInstallingUpdate(false);
    }
  };

  // Listen for update events
  useEffect(() => {
    const unlistenUpdateReady = listen("update-ready", () => {
      setInstallingUpdate(false);
      setUpdateReady(true);
      // Show different toast message for macOS (relaunch doesn't work on macOS in Tauri v2)
      showToast(isMac ? t("toast.updateDownloadedMac") : t("toast.updateDownloaded"));
    });

    return () => {
      unlistenUpdateReady.then((fn) => fn());
    };
  }, [showToast, t, isMac]);

  // Restart to apply update
  const handleRestartToUpdate = async () => {
    try {
      // On macOS, relaunch() doesn't work in Tauri v2 (known bug: https://github.com/tauri-apps/tauri/issues/13923)
      // So we just exit and let the user reopen the app manually
      if (isMac) {
        const { exit } = await import("@tauri-apps/plugin-process");
        await exit(0);
      } else {
        // On Windows/Linux, relaunch should work
        const { relaunch } = await import("@tauri-apps/plugin-process");
        await relaunch();
      }
    } catch (e) {
      console.error("Failed to restart:", e);
      // Fallback: just close the app
      const { exit } = await import("@tauri-apps/plugin-process");
      await exit(0);
    }
  };

  // Format shortcut for display (replace Ctrl/Command based on platform)
  const formatShortcutForDisplay = (shortcut: string) => {
    if (isMac) {
      return shortcut
        .replace(/Ctrl/gi, "⌃")
        .replace(/Command/gi, "⌘")
        .replace(/Option/gi, "⌥")
        .replace(/Alt/gi, "⌥")
        .replace(/Shift/gi, "⇧")
        .replace(/\+/g, "");
    }
    return shortcut;
  };

  // Handle resize edge mouse down
  const handleResizeMouseDown = (edge: string) => (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsResizing(true);
    setResizeEdge(edge);
  };

  if (!isOpen) return null;

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "appearance", label: t("settings.tab.appearance") },
    { id: "startup", label: t("settings.tab.startup") },
    { id: "server", label: t("settings.tab.server") },
    { id: "about", label: t("settings.tab.about") },
  ];

  const renderAppearanceTab = () => (
    <>
      <div className="settings-section">
        <div className="settings-field">
          <label htmlFor="theme">{t("settings.theme")}</label>
          <div className="theme-selector">
            <button
              type="button"
              className={`theme-option ${settings.theme === "light" ? "active" : ""}`}
              onClick={() => handleChange("theme", "light")}
            >
              <span className="theme-icon">&#9788;</span>
              <span>{t("settings.theme.light")}</span>
            </button>
            <button
              type="button"
              className={`theme-option ${settings.theme === "dark" ? "active" : ""}`}
              onClick={() => handleChange("theme", "dark")}
            >
              <span className="theme-icon">&#9790;</span>
              <span>{t("settings.theme.dark")}</span>
            </button>
            <button
              type="button"
              className={`theme-option ${settings.theme === "auto" ? "active" : ""}`}
              onClick={() => handleChange("theme", "auto")}
            >
              <span className="theme-icon">&#9881;</span>
              <span>{t("settings.theme.auto")}</span>
            </button>
          </div>
          <p className="settings-hint">
            {t("settings.theme.hint")}
          </p>
        </div>

        <div className="settings-field">
          <label htmlFor="language">{t("settings.language")}</label>
          <select
            id="language"
            value={currentLanguage}
            onChange={(e) => handleLanguageChange(e.target.value as Language)}
            className="settings-select"
          >
            {supportedLanguages.map((lang) => (
              <option key={lang} value={lang}>
                {languageNames[lang]}
              </option>
            ))}
          </select>
          <p className="settings-hint">
            {t("settings.language.hint")}
          </p>
        </div>

        <div className="settings-field">
          <label htmlFor="syntaxTheme">{t("settings.syntaxTheme")}</label>
          <select
            id="syntaxTheme"
            value={settings.syntaxTheme}
            onChange={(e) => handleChange("syntaxTheme", e.target.value)}
            className="settings-select"
          >
            {SYNTAX_THEMES.map((theme) => (
              <option key={theme} value={theme}>
                {t(`settings.syntaxTheme.${theme}` as const)}
              </option>
            ))}
          </select>
          <p className="settings-hint">
            {t("settings.syntaxTheme.hint")}
          </p>
        </div>

        <div className="settings-field settings-checkbox">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.notificationsEnabled}
              onChange={(e) =>
                handleChange("notificationsEnabled", e.target.checked)
              }
            />
            <span className="checkbox-text">
              {t("settings.notifications")}
            </span>
          </label>
          <p className="settings-hint">
            {t("settings.notifications.hint")}
          </p>
        </div>

        <div className="settings-field">
          <label>{t("settings.globalShortcut")}</label>
          <div className="shortcut-editor">
            {isRecordingShortcut ? (
              <div
                ref={shortcutInputRef}
                className="shortcut-input recording"
                tabIndex={0}
                onKeyDown={handleShortcutKeyDown}
                onKeyUp={handleShortcutKeyUp}
                onBlur={cancelRecordingShortcut}
              >
                {recordedKeys.length > 0
                  ? formatShortcutForDisplay(recordedKeys.join("+"))
                  : t("settings.globalShortcut.recording")}
              </div>
            ) : (
              <button
                type="button"
                className="shortcut-input"
                onClick={startRecordingShortcut}
              >
                {formatShortcutForDisplay(settings.globalShortcut)}
              </button>
            )}
            {isRecordingShortcut && (
              <button
                type="button"
                className="shortcut-cancel"
                onMouseDown={(e) => e.preventDefault()}
                onClick={cancelRecordingShortcut}
              >
                {t("common.cancel")}
              </button>
            )}
          </div>
          <p className="settings-hint">
            {t("settings.globalShortcut.hint")}
          </p>
        </div>
      </div>
    </>
  );

  const renderStartupTab = () => (
    <>
      <div className="settings-section">
        <div className="settings-field settings-checkbox">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.openOnStartup}
              onChange={(e) =>
                handleChange("openOnStartup", e.target.checked)
              }
            />
            <span className="checkbox-text">
              {t("settings.openOnStartup")}
            </span>
          </label>
          <p className="settings-hint">
            {t("settings.openOnStartup.hint")}
          </p>
        </div>

        <div className="settings-field settings-checkbox">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.startOnLogin}
              onChange={(e) =>
                handleChange("startOnLogin", e.target.checked)
              }
            />
            <span className="checkbox-text">
              {t("settings.startOnLogin")}
            </span>
          </label>
          <p className="settings-hint">
            {t("settings.startOnLogin.hint")}
          </p>
        </div>
      </div>
    </>
  );

  const renderServerTab = () => (
    <>
      <div className="settings-section">
        <div className="settings-field">
          <label>{t("settings.serverMode")}</label>
          <div className={`server-mode-selector ${switchingServerMode ? "switching" : ""}`}>
            <button
              type="button"
              className={`server-mode-option ${settings.useBundledServer ? "active" : ""}`}
              onClick={() => handleServerModeChange(true)}
              disabled={switchingServerMode}
            >
              {switchingServerMode && !settings.useBundledServer ? (
                <span className="server-mode-spinner"></span>
              ) : (
                <span className="server-mode-icon">&#9881;</span>
              )}
              <span>{t("settings.serverMode.bundled")}</span>
            </button>
            <button
              type="button"
              className={`server-mode-option ${!settings.useBundledServer ? "active" : ""}`}
              onClick={() => handleServerModeChange(false)}
              disabled={switchingServerMode}
            >
              {switchingServerMode && settings.useBundledServer ? (
                <span className="server-mode-spinner"></span>
              ) : (
                <span className="server-mode-icon">&#8599;</span>
              )}
              <span>{t("settings.serverMode.external")}</span>
            </button>
          </div>
          <p className="settings-hint">
            {switchingServerMode
              ? t("settings.serverMode.switching")
              : settings.useBundledServer
                ? t("settings.serverMode.hint.bundled")
                : t("settings.serverMode.hint.external")}
          </p>
        </div>

        {settings.useBundledServer && (
          <div className="settings-field settings-checkbox">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.listenOnAllInterfaces}
                onChange={(e) => handleToggleNetworkAccess(e.target.checked)}
                disabled={togglingNetworkAccess}
              />
              <span className="checkbox-text">
                {togglingNetworkAccess ? t("settings.networkAccess.restarting") : t("settings.networkAccess")}
              </span>
            </label>
            <p className="settings-hint">
              {t("settings.networkAccess.hint")}
            </p>
          </div>
        )}

        {settings.useBundledServer && settings.listenOnAllInterfaces && (
          <>
            <div className="settings-field">
              <label>{t("settings.serverUrls")}</label>
              <div className="server-url-list">
                {localIpAddresses.length > 0 ? (
                  localIpAddresses.map((ip) => {
                    const url = `http://${ip}:${getServerPort()}`;
                    return (
                      <div key={ip} className="settings-url-input">
                        <input
                          type="text"
                          value={url}
                          readOnly
                          className="settings-readonly with-copy"
                        />
                        <button
                          type="button"
                          className="copy-icon-button"
                          onClick={() => {
                            navigator.clipboard.writeText(url);
                          }}
                          title={t("tooltip.copy")}
                        >
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                          </svg>
                        </button>
                      </div>
                    );
                  })
                ) : (
                  <p className="settings-hint">{t("settings.serverUrls.empty")}</p>
                )}
              </div>
              <p className="settings-hint">
                {t("settings.serverUrls.hint")}
              </p>
            </div>

            <div className="settings-field">
              <label htmlFor="bundledServerToken">{t("settings.bundledServerToken")}</label>
              <div className="settings-password-input">
                <input
                  id="bundledServerToken"
                  type={showBundledToken ? "text" : "password"}
                  value={settings.bundledServerToken || ""}
                  onChange={(e) => handleChange("bundledServerToken", e.target.value || null)}
                  placeholder={t("settings.bundledServerToken.placeholder")}
                  autoComplete="off"
                />
                <button
                  type="button"
                  className="password-toggle-button"
                  onClick={() => setShowBundledToken(!showBundledToken)}
                  title={showBundledToken ? t("settings.token.hide") : t("settings.token.show")}
                >
                  {showBundledToken ? (
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                      <line x1="1" y1="1" x2="23" y2="23"></line>
                    </svg>
                  ) : (
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  )}
                </button>
              </div>
              <p className="settings-hint">
                {t("settings.bundledServerToken.hint")}
              </p>
            </div>
          </>
        )}

        {!settings.useBundledServer && (
          <>
            <div className="settings-field">
              <label htmlFor="serverUrl">{t("settings.serverUrl")}</label>
              <input
                id="serverUrl"
                type="text"
                value={settings.serverAddress}
                onChange={(e) => handleChange("serverAddress", e.target.value)}
                placeholder={t("settings.serverUrl.placeholder")}
              />
              <p className="settings-hint">
                {t("settings.serverUrl.hint")}
              </p>
            </div>

            <div className="settings-field">
              <label htmlFor="externalServerToken">{t("settings.serverToken")}</label>
              <div className="settings-password-input">
                <input
                  id="externalServerToken"
                  type={showExternalToken ? "text" : "password"}
                  value={settings.externalServerToken || ""}
                  onChange={(e) => handleChange("externalServerToken", e.target.value || null)}
                  placeholder={t("settings.serverToken.placeholder")}
                  autoComplete="off"
                />
                <button
                  type="button"
                  className="password-toggle-button"
                  onClick={() => setShowExternalToken(!showExternalToken)}
                  title={showExternalToken ? t("settings.token.hide") : t("settings.token.show")}
                >
                  {showExternalToken ? (
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                      <line x1="1" y1="1" x2="23" y2="23"></line>
                    </svg>
                  ) : (
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  )}
                </button>
              </div>
              <p className="settings-hint">
                {t("settings.serverToken.hint")}
              </p>
            </div>

            {serverInfo && (
              <div className="settings-server-info">
                <h4>{t("settings.serverInfo")}</h4>
                <div className="settings-server-info-grid">
                  <div className="settings-server-info-item">
                    <span className="settings-server-info-label">{t("settings.serverInfo.version")}</span>
                    <span className="settings-server-info-value">{serverInfo.version}</span>
                  </div>
                  <div className="settings-server-info-item">
                    <span className="settings-server-info-label">{t("settings.serverInfo.maxUploadSize")}</span>
                    <span className="settings-server-info-value">
                      {Math.round(serverInfo.config.max_upload_size_bytes / (1024 * 1024))} MB
                    </span>
                  </div>
                  {serverInfo.config.cleanup_enabled && serverInfo.config.cleanup_retention_days && (
                    <div className="settings-server-info-item">
                      <span className="settings-server-info-label">{t("settings.serverInfo.cleanupRetention")}</span>
                      <span className="settings-server-info-value">
                        {serverInfo.config.cleanup_retention_days} {t("settings.serverInfo.days")}
                      </span>
                    </div>
                  )}
                  {serverInfo.config.acme_enabled && serverInfo.config.acme_domain && (
                    <div className="settings-server-info-item">
                      <span className="settings-server-info-label">{t("settings.serverInfo.acmeDomain")}</span>
                      <span className="settings-server-info-value">{serverInfo.config.acme_domain}</span>
                    </div>
                  )}
                </div>
                <p className="settings-hint">{t("settings.serverInfo.hint")}</p>
              </div>
            )}
          </>
        )}
      </div>

      {settings.useBundledServer && (
        <div className="settings-section">
          <h3>{t("settings.storage")}</h3>
          <div className="settings-field">
            <label htmlFor="defaultSaveLocation">
              {t("settings.defaultSaveLocation")}
            </label>
            <div className="settings-path-input">
              <input
                id="defaultSaveLocation"
                type="text"
                value={settings.defaultSaveLocation || ""}
                onChange={(e) =>
                  handleChange(
                    "defaultSaveLocation",
                    e.target.value || null
                  )
                }
                placeholder={t("settings.defaultSaveLocation.placeholder")}
              />
              <button
                className="browse-button"
                onClick={handleBrowseDirectory}
              >
                {t("settings.browse")}
              </button>
            </div>
            <p className="settings-hint">
              {t("settings.defaultSaveLocation.hint")}
            </p>
          </div>

          <div className="settings-field settings-checkbox">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.cleanupEnabled}
                onChange={(e) =>
                  handleChange("cleanupEnabled", e.target.checked)
                }
              />
              <span className="checkbox-text">
                {t("settings.cleanup")}
              </span>
            </label>
            <p className="settings-hint">
              {t("settings.cleanup.hint")}
            </p>
          </div>

          {settings.cleanupEnabled && (
            <div className="settings-field">
              <label htmlFor="cleanupRetentionDays">
                {t("settings.cleanup.retentionDays")}
              </label>
              <input
                id="cleanupRetentionDays"
                type="number"
                min="1"
                max="365"
                value={settings.cleanupRetentionDays}
                onChange={(e) =>
                  handleChange(
                    "cleanupRetentionDays",
                    Math.max(1, Math.min(365, parseInt(e.target.value) || 30))
                  )
                }
                className="settings-number-input"
              />
              <p className="settings-hint">
                {t("settings.cleanup.retentionDays.hint")}
              </p>
            </div>
          )}

          <div className="settings-field">
            <label htmlFor="maxUploadSizeMb">
              {t("settings.maxUploadSize")}
            </label>
            <input
              id="maxUploadSizeMb"
              type="number"
              min="1"
              max="1024"
              value={settings.maxUploadSizeMb}
              onChange={(e) =>
                handleChange(
                  "maxUploadSizeMb",
                  Math.max(1, Math.min(1024, parseInt(e.target.value) || 10))
                )
              }
              className="settings-number-input"
            />
            <p className="settings-hint">
              {t("settings.maxUploadSize.hint")}
            </p>
          </div>

          {(settings.cleanupEnabled !== originalCleanupEnabled ||
            settings.cleanupRetentionDays !== originalCleanupRetentionDays ||
            settings.maxUploadSizeMb !== originalMaxUploadSizeMb) && (
            <div className="settings-notice">
              {t("settings.cleanup.restartNotice")}
            </div>
          )}
        </div>
      )}

      {settings.useBundledServer && (
        <div className="settings-section">
          <h3>{t("settings.dataManagement")}</h3>
          <div className="settings-field">
            <label>{t("settings.clearAllData")}</label>
            {!showClearConfirm ? (
              <>
                <button
                  type="button"
                  className="settings-btn danger"
                  onClick={() => setShowClearConfirm(true)}
                  disabled={clearing}
                >
                  {t("settings.clearAllData.button")}
                </button>
                <p className="settings-hint">
                  {t("settings.clearAllData.hint")}
                </p>
              </>
            ) : (
              <div className="clear-confirm">
                <p className="clear-confirm-message">
                  {t("settings.clearAllData.confirm", { count: "all" })}
                </p>
                <div className="clear-confirm-buttons">
                  <button
                    type="button"
                    className="settings-btn secondary"
                    onClick={() => setShowClearConfirm(false)}
                    disabled={clearing}
                  >
                    {t("common.cancel")}
                  </button>
                  <button
                    type="button"
                    className="settings-btn danger"
                    onClick={handleClearData}
                    disabled={clearing}
                  >
                    {clearing ? t("settings.clearAllData.clearing") : t("settings.clearAllData.confirmButton")}
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );

  const renderAboutTab = () => (
    <>
      <div className="settings-section">
        <h3>{t("settings.about")}</h3>
        <div className="settings-about-info">
          <div className="settings-about-logo">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M16 4H18C18.5304 4 19.0391 4.21071 19.4142 4.58579C19.7893 4.96086 20 5.46957 20 6V20C20 20.5304 19.7893 21.0391 19.4142 21.4142C19.0391 21.7893 18.5304 22 18 22H6C5.46957 22 4.96086 21.7893 4.58579 21.4142C4.21071 21.0391 4 20.5304 4 20V6C4 5.46957 4.21071 4.96086 4.58579 4.58579C4.96086 4.21071 5.46957 4 6 4H8" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M15 2H9C8.44772 2 8 2.44772 8 3V5C8 5.55228 8.44772 6 9 6H15C15.5523 6 16 5.55228 16 5V3C16 2.44772 15.5523 2 15 2Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </div>
          <div className="settings-about-text">
            <h4>Clipper</h4>
            <p className="settings-about-version">{t("settings.about.version")}: {appVersion}</p>
            <p className="settings-about-copyright">{t("settings.about.copyright")}</p>
          </div>
        </div>
      </div>

      <div className="settings-section">
        <h3>{t("settings.updates")}</h3>
        <div className="settings-field">
          {updateReady ? (
            <>
              <p className="settings-update-ready">
                {isMac ? t("settings.updates.restartRequiredMac") : t("settings.updates.restartRequired")}
              </p>
              <button
                type="button"
                className="settings-btn primary"
                onClick={handleRestartToUpdate}
              >
                {isMac ? t("settings.updates.quitNow") : t("settings.updates.restartNow")}
              </button>
            </>
          ) : updateInfo ? (
            <>
              <p className="settings-update-available">
                {t("settings.updates.available")}
              </p>
              <p className="settings-hint">
                {t("settings.updates.available.hint")
                  .replace("{version}", updateInfo.version)
                  .replace("{currentVersion}", updateInfo.current_version)}
              </p>
              {updateInfo.body && (
                <div className="settings-update-notes">
                  <pre>{updateInfo.body}</pre>
                </div>
              )}
              <button
                type="button"
                className="settings-btn primary"
                onClick={handleInstallUpdate}
                disabled={installingUpdate}
              >
                {installingUpdate
                  ? t("settings.updates.downloading")
                  : t("settings.updates.downloadAndInstall")}
              </button>
            </>
          ) : checkingForUpdates ? (
            <p className="settings-hint">{t("settings.updates.checking")}</p>
          ) : updateChecked && !updateInfo ? (
            <>
              <p className="settings-update-uptodate">{t("settings.updates.upToDate")}</p>
              <p className="settings-hint">
                {t("settings.updates.upToDate.hint").replace("{version}", appVersion)}
              </p>
              <button
                type="button"
                className="settings-btn secondary"
                onClick={handleCheckForUpdates}
                disabled={checkingForUpdates}
              >
                {t("settings.updates.checkForUpdates")}
              </button>
            </>
          ) : (
            <>
              <button
                type="button"
                className="settings-btn secondary"
                onClick={handleCheckForUpdates}
                disabled={checkingForUpdates}
              >
                {t("settings.updates.checkForUpdates")}
              </button>
              {updateError && (
                <p className="settings-error">{t("settings.updates.error")}: {updateError}</p>
              )}
            </>
          )}
        </div>
      </div>
    </>
  );

  // Handle backdrop click - don't close if we just finished resizing
  const handleBackdropClick = () => {
    if (!justFinishedResizing.current) {
      handleClose();
    }
  };

  return (
    <div className="settings-backdrop" onClick={handleBackdropClick}>
      <div
        ref={dialogRef}
        className="settings-dialog settings-dialog-tabbed"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Resize handles */}
        <div className="settings-resize-handle settings-resize-n" onMouseDown={handleResizeMouseDown('n')} />
        <div className="settings-resize-handle settings-resize-s" onMouseDown={handleResizeMouseDown('s')} />
        <div className="settings-resize-handle settings-resize-e" onMouseDown={handleResizeMouseDown('e')} />
        <div className="settings-resize-handle settings-resize-w" onMouseDown={handleResizeMouseDown('w')} />
        <div className="settings-resize-handle settings-resize-ne" onMouseDown={handleResizeMouseDown('ne')} />
        <div className="settings-resize-handle settings-resize-nw" onMouseDown={handleResizeMouseDown('nw')} />
        <div className="settings-resize-handle settings-resize-se" onMouseDown={handleResizeMouseDown('se')} />
        <div className="settings-resize-handle settings-resize-sw" onMouseDown={handleResizeMouseDown('sw')} />

        <div className="settings-header">
          <h2>{t("settings.title")}</h2>
          <button className="settings-close" onClick={handleClose}>
            &times;
          </button>
        </div>

        <div className="settings-tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`settings-tab ${activeTab === tab.id ? "active" : ""}`}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="settings-content">
          {loading ? (
            <div className="settings-loading">
              <div className="loading-spinner"></div>
              <span>{t("common.loading")}</span>
            </div>
          ) : (
            <>
              {error && <div className="settings-error">{error}</div>}

              {activeTab === "appearance" && renderAppearanceTab()}
              {activeTab === "startup" && renderStartupTab()}
              {activeTab === "server" && renderServerTab()}
              {activeTab === "about" && renderAboutTab()}
            </>
          )}
        </div>

        <div className="settings-footer">
          <button
            className="settings-btn primary"
            onClick={handleClose}
            disabled={loading}
          >
            {t("common.close")}
          </button>
        </div>
      </div>
    </div>
  );
}

// Hook to manage settings dialog state
export function useSettingsDialog() {
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    // Listen for open-settings event from tray menu
    const unlisten = listen("open-settings", () => {
      setIsOpen(true);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);

  return { isOpen, open, close };
}
