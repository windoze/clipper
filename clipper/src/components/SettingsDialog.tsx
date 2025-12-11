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
import { CertificateConfirmDialog, CertificateInfo } from "./CertificateConfirmDialog";
import { CertificateMismatchDialog, CertificateMismatchInfo } from "./CertificateMismatchDialog";
import { useEnsureWindowSize } from "../hooks/useEnsureWindowSize";

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
    export_import_enabled?: boolean;
  };
}

interface UpdateInfo {
  version: string;
  current_version: string;
  body: string | null;
  date: string | null;
}

interface ServerCertificateCheckResult {
  isHttps: boolean;
  certificate: CertificateInfo | null;
  isTrusted: boolean;
  needsTrustConfirmation: boolean;
  fingerprintMismatch: boolean;
  storedFingerprint: string | null;
  error: string | null;
}

type SettingsTab = "appearance" | "startup" | "server" | "about";

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onThemeChange?: (theme: ThemePreference) => void;
  onSyntaxThemeChange?: (theme: SyntaxTheme) => void;
  initialTab?: SettingsTab;
  autoCheckUpdates?: boolean;
}

export function SettingsDialog({ isOpen, onClose, onThemeChange, onSyntaxThemeChange, initialTab, autoCheckUpdates }: SettingsDialogProps) {
  const { t, language: currentLanguage, setLanguage } = useI18n();
  const { showToast } = useToast();
  // Detect platform for default shortcut
  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const defaultShortcut = isMac ? "Command+Shift+V" : "Ctrl+Shift+V";

  const [activeTab, setActiveTab] = useState<SettingsTab>("appearance");
  const [focusedTabIndex, setFocusedTabIndex] = useState(0);
  const tabsRef = useRef<HTMLDivElement>(null);
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
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
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
  // Download progress state
  const [downloadProgress, setDownloadProgress] = useState<{
    percentage: number | null;
    downloadedBytes: number;
    totalBytes: number | null;
    speedBytesPerSec: number | null;
  } | null>(null);
  // App version for About tab
  const [appVersion, setAppVersion] = useState<string>("");
  // Dialog resizing state
  const dialogRef = useRef<HTMLDivElement>(null);
  const [isResizing, setIsResizing] = useState(false);
  const [resizeEdge, setResizeEdge] = useState<string | null>(null);
  const justFinishedResizing = useRef(false);
  // Certificate confirmation dialog state
  const [showCertDialog, setShowCertDialog] = useState(false);
  const [pendingCertificate, setPendingCertificate] = useState<CertificateInfo | null>(null);
  const [pendingServerUrl, setPendingServerUrl] = useState<string>("");
  const [trustingCertificate, setTrustingCertificate] = useState(false);
  // Certificate mismatch dialog state (critical MITM warning)
  const [showMismatchDialog, setShowMismatchDialog] = useState(false);
  const [pendingMismatch, setPendingMismatch] = useState<CertificateMismatchInfo | null>(null);
  const [acceptingMismatch, setAcceptingMismatch] = useState(false);

  // Ensure window is large enough to show the settings dialog
  // Settings dialog requires min-width: 500px, min-height: 450px (from CSS)
  // Add some padding for window chrome
  useEnsureWindowSize(isOpen, 550, 500);

  // Load settings when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadSettings();
      loadServerInfo();
      loadLocalIpAddresses();
      loadAppVersion();
      // Set initial tab if specified
      if (initialTab) {
        setActiveTab(initialTab);
        const tabIds: SettingsTab[] = ["appearance", "startup", "server", "about"];
        setFocusedTabIndex(tabIds.indexOf(initialTab));
      } else {
        setFocusedTabIndex(0);
      }
      // Auto-check for updates if requested
      if (autoCheckUpdates) {
        // Small delay to ensure the dialog is fully rendered
        setTimeout(() => {
          handleCheckForUpdates();
        }, 100);
      }
    }
  }, [isOpen, initialTab, autoCheckUpdates]);

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

  // Handle ESC key to close dialog and Tab focus trapping
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isRecordingShortcut) {
        e.preventDefault();
        handleClose();
        return;
      }

      // Handle Tab/Shift+Tab for focus cycling
      if (e.key === "Tab" && dialogRef.current) {
        const focusableElements = dialogRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        const focusableArray = Array.from(focusableElements);

        if (focusableArray.length === 0) return;

        const firstElement = focusableArray[0];
        const lastElement = focusableArray[focusableArray.length - 1];
        const activeElement = document.activeElement;
        const currentIndex = focusableArray.indexOf(activeElement as HTMLElement);

        e.preventDefault();

        if (e.shiftKey) {
          // Shift+Tab: go backwards
          if (currentIndex <= 0 || !dialogRef.current.contains(activeElement)) {
            lastElement.focus();
          } else {
            focusableArray[currentIndex - 1].focus();
          }
        } else {
          // Tab: go forwards
          if (currentIndex === -1 || currentIndex >= focusableArray.length - 1 || !dialogRef.current.contains(activeElement)) {
            firstElement.focus();
          } else {
            focusableArray[currentIndex + 1].focus();
          }
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, isRecordingShortcut]);

  // Handle tab keyboard navigation
  const handleTabKeyDown = (e: React.KeyboardEvent) => {
    const tabIds: SettingsTab[] = ["appearance", "startup", "server", "about"];
    const tabCount = tabIds.length;
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      setFocusedTabIndex((prev) => (prev - 1 + tabCount) % tabCount);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      setFocusedTabIndex((prev) => (prev + 1) % tabCount);
    } else if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      setActiveTab(tabIds[focusedTabIndex]);
    }
  };

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

      // Generate a token for bundled server if one doesn't exist
      // This ensures the server always has authentication available
      if (!loadedSettings.bundledServerToken) {
        const token = generateToken();
        loadedSettings.bundledServerToken = token;
        await invoke("save_settings", { settings: loadedSettings });
      }

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
        // Check if the new server URL needs certificate confirmation
        const certResult = await invoke<ServerCertificateCheckResult>("check_server_certificate", { serverUrl: settings.serverAddress });

        if (certResult.needsTrustConfirmation && certResult.certificate) {
          // Show certificate confirmation dialog
          setPendingCertificate(certResult.certificate);
          setPendingServerUrl(settings.serverAddress);
          setShowCertDialog(true);
          return; // Don't close yet, wait for certificate confirmation
        }

        // No certificate confirmation needed, proceed with reconnect
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
        // For external server, first check if it's HTTPS with self-signed cert
        await switchToExternalServerWithCertCheck(settings.serverAddress);
        return; // switchToExternalServerWithCertCheck handles setSwitchingServerMode
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

  // Switch to external server with certificate checking
  const switchToExternalServerWithCertCheck = async (targetUrl: string) => {
    try {
      // Check if the server has a certificate that needs trust confirmation
      const certResult = await invoke<ServerCertificateCheckResult>("check_server_certificate", { serverUrl: targetUrl });

      // CRITICAL: Check for fingerprint mismatch first (potential MITM attack)
      if (certResult.fingerprintMismatch && certResult.certificate && certResult.storedFingerprint) {
        // Show critical mismatch warning dialog
        setPendingMismatch({
          host: certResult.certificate.host,
          fingerprint: certResult.certificate.fingerprint,
          storedFingerprint: certResult.storedFingerprint,
        });
        setPendingServerUrl(targetUrl);
        setShowMismatchDialog(true);
        setSwitchingServerMode(false);
        return;
      }

      if (certResult.needsTrustConfirmation && certResult.certificate) {
        // Show certificate confirmation dialog
        setPendingCertificate(certResult.certificate);
        setPendingServerUrl(targetUrl);
        setShowCertDialog(true);
        setSwitchingServerMode(false);
        return;
      }

      // No certificate confirmation needed, proceed with switch
      await completeSwitchToExternalServer(targetUrl);
    } catch (e) {
      setError(`Failed to check server certificate: ${e}`);
      setSwitchingServerMode(false);
    }
  };

  // Complete the switch to external server (after certificate is trusted or not needed)
  const completeSwitchToExternalServer = async (targetUrl: string) => {
    try {
      // Switch to external server
      // Returns null if connected successfully, or an error message if unreachable
      const connectionError = await invoke<string | null>("switch_to_external_server", { serverUrl: targetUrl });
      setServerUrl(targetUrl);
      if (connectionError) {
        showToast(connectionError, "error");
      } else {
        showToast(t("toast.serverConnected"));
      }
      // Reload settings from backend to ensure frontend is in sync
      const loadedSettings = await invoke<Settings>("get_settings");
      setSettings(loadedSettings);
    } catch (e) {
      setError(`Failed to switch server mode: ${e}`);
    } finally {
      setSwitchingServerMode(false);
    }
  };

  // Handle certificate trust confirmation
  const handleCertificateConfirm = async () => {
    if (!pendingCertificate) return;

    setTrustingCertificate(true);
    try {
      // Trust the certificate
      await invoke("trust_certificate", {
        host: pendingCertificate.host,
        fingerprint: pendingCertificate.fingerprint,
      });
      showToast(t("toast.certificateTrusted").replace("{host}", pendingCertificate.host));

      // Close dialog and proceed with connection
      setShowCertDialog(false);
      setPendingCertificate(null);
      setSwitchingServerMode(true);
      await completeSwitchToExternalServer(pendingServerUrl);
    } catch (e) {
      setError(`Failed to trust certificate: ${e}`);
    } finally {
      setTrustingCertificate(false);
    }
  };

  // Handle certificate trust cancellation
  const handleCertificateCancel = () => {
    setShowCertDialog(false);
    setPendingCertificate(null);
    setPendingServerUrl("");
  };

  // Handle certificate mismatch - accept risk (user decides to trust new cert)
  const handleMismatchAcceptRisk = async () => {
    if (!pendingMismatch) return;

    setAcceptingMismatch(true);
    try {
      // User is accepting the risk - trust the new certificate
      await invoke("trust_certificate", {
        host: pendingMismatch.host,
        fingerprint: pendingMismatch.fingerprint,
      });
      showToast(t("toast.certificateTrusted").replace("{host}", pendingMismatch.host));

      // Close dialog and proceed with connection
      setShowMismatchDialog(false);
      setPendingMismatch(null);
      setSwitchingServerMode(true);
      await completeSwitchToExternalServer(pendingServerUrl);
    } catch (e) {
      setError(`Failed to trust certificate: ${e}`);
    } finally {
      setAcceptingMismatch(false);
    }
  };

  // Handle certificate mismatch - reject (user decides not to proceed)
  const handleMismatchReject = () => {
    setShowMismatchDialog(false);
    setPendingMismatch(null);
    setPendingServerUrl("");
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

  // Handle export clips
  const handleExport = async () => {
    setExporting(true);
    setError(null);
    try {
      const result = await invoke<string>("export_clips");
      showToast(t("toast.exportSuccess", { path: result }));
    } catch (e) {
      const errorMsg = String(e);
      if (errorMsg !== "Save cancelled") {
        setError(`${t("settings.export.error")}: ${e}`);
      }
    } finally {
      setExporting(false);
    }
  };

  // Handle import clips
  const handleImport = async () => {
    setImporting(true);
    setError(null);
    try {
      const result = await invoke<{ imported_count: number; skipped_count: number }>("import_clips");
      showToast(t("toast.importSuccess", { imported: result.imported_count, skipped: result.skipped_count }));
    } catch (e) {
      const errorMsg = String(e);
      if (errorMsg !== "Open cancelled") {
        setError(`${t("settings.import.error")}: ${e}`);
      }
    } finally {
      setImporting(false);
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
    setDownloadProgress(null);
    try {
      await invoke("install_update");
      // The command completed successfully, update is ready
      // Set the state directly in case the event listener misses the event
      setInstallingUpdate(false);
      setUpdateReady(true);
      setDownloadProgress(null);
      setUpdateInfo(null);
    } catch (e) {
      setUpdateError(String(e));
      setInstallingUpdate(false);
      setDownloadProgress(null);
    }
  };

  // Listen for update events
  useEffect(() => {
    const unlistenUpdateReady = listen("update-ready", () => {
      setInstallingUpdate(false);
      setUpdateReady(true);
      setDownloadProgress(null);
      setUpdateInfo(null); // Clear update info so the download button disappears
      showToast(t("toast.updateDownloaded"));
    });

    // Listen for download progress events
    const unlistenProgress = listen<{
      downloadedBytes: number;
      totalBytes: number | null;
      percentage: number | null;
      speedBytesPerSec: number | null;
    }>("update-download-progress", (event) => {
      setDownloadProgress({
        percentage: event.payload.percentage ?? null,
        downloadedBytes: event.payload.downloadedBytes,
        totalBytes: event.payload.totalBytes ?? null,
        speedBytesPerSec: event.payload.speedBytesPerSec ?? null,
      });
    });

    return () => {
      unlistenUpdateReady.then((fn) => fn());
      unlistenProgress.then((fn) => fn());
    };
  }, [showToast, t, isMac]);

  // Restart to apply update
  const handleRestartToUpdate = async () => {
    try {
      // Use our custom restart_app command that spawns a delayed relaunch
      // before exiting, working around Tauri v2's broken relaunch() on macOS
      await invoke("restart_app");
    } catch (e) {
      console.error("Failed to restart:", e);
      // Fallback: try quit_app
      try {
        await invoke("quit_app");
      } catch (e2) {
        console.error("Fallback exit also failed:", e2);
      }
    }
  };

  // Format bytes to human readable string
  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  };

  // Format download speed to human readable string
  const formatSpeed = (bytesPerSec: number): string => {
    return `${formatBytes(bytesPerSec)}/s`;
  };

  // Generate a secure random 16-character token
  const generateToken = () => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*-_=+';
    const array = new Uint8Array(16);
    crypto.getRandomValues(array);
    let token = '';
    for (let i = 0; i < 16; i++) {
      token += chars[array[i] % chars.length];
    }
    return token;
  };

  // Handle generate token for bundled server
  const handleGenerateToken = async () => {
    const token = generateToken();
    const newSettings = { ...settings, bundledServerToken: token };
    setSettings(newSettings);
    await saveSettings(newSettings);
    // Show the token after generating
    setShowBundledToken(true);
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
                  onClick={handleGenerateToken}
                  title={t("settings.token.generate")}
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M23 4v6h-6"></path>
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
                  </svg>
                </button>
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
                spellCheck={false}
                autoCorrect="off"
                autoCapitalize="off"
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

      {(settings.useBundledServer || serverInfo?.config.export_import_enabled) && (
        <div className="settings-section">
          <h3>{t("settings.exportImport")}</h3>
          <div className="settings-field">
            <label>{t("settings.export")}</label>
            <button
              type="button"
              className="settings-btn"
              onClick={handleExport}
              disabled={exporting || importing}
            >
              {exporting ? t("settings.export.exporting") : t("settings.export.button")}
            </button>
            <p className="settings-hint">
              {t("settings.export.hint")}
            </p>
          </div>
          <div className="settings-field">
            <label>{t("settings.import")}</label>
            <button
              type="button"
              className="settings-btn"
              onClick={handleImport}
              disabled={exporting || importing}
            >
              {importing ? t("settings.import.importing") : t("settings.import.button")}
            </button>
            <p className="settings-hint">
              {t("settings.import.hint")}
            </p>
          </div>
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
                {t("settings.updates.restartRequired")}
              </p>
              <button
                type="button"
                className="settings-btn primary"
                onClick={handleRestartToUpdate}
              >
                {t("settings.updates.restartNow")}
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
              {installingUpdate && downloadProgress && (
                <div className="settings-download-progress">
                  <div className="settings-progress-bar">
                    <div
                      className="settings-progress-bar-fill"
                      style={{ width: `${downloadProgress.percentage ?? 0}%` }}
                    />
                  </div>
                  <div className="settings-progress-info">
                    <span className="settings-progress-size">
                      {formatBytes(downloadProgress.downloadedBytes)}
                      {downloadProgress.totalBytes && ` / ${formatBytes(downloadProgress.totalBytes)}`}
                    </span>
                    {downloadProgress.speedBytesPerSec && (
                      <span className="settings-progress-speed">
                        {formatSpeed(downloadProgress.speedBytesPerSec)}
                      </span>
                    )}
                  </div>
                </div>
              )}
              <button
                type="button"
                className="settings-btn primary"
                onClick={handleInstallUpdate}
                disabled={installingUpdate}
              >
                {installingUpdate
                  ? downloadProgress?.percentage != null
                    ? `${t("settings.updates.downloading")} ${downloadProgress.percentage}%`
                    : t("settings.updates.downloading")
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
          <div className="settings-close" onClick={handleClose} role="button" aria-label="Close">
            &times;
          </div>
        </div>

        <div
          ref={tabsRef}
          className="settings-tabs"
          role="tablist"
          tabIndex={0}
          onKeyDown={handleTabKeyDown}
        >
          {tabs.map((tab, index) => (
            <div
              key={tab.id}
              role="tab"
              aria-selected={activeTab === tab.id}
              className={`settings-tab ${activeTab === tab.id ? "active" : ""} ${focusedTabIndex === index ? "focused" : ""}`}
              onClick={() => {
                setActiveTab(tab.id);
                setFocusedTabIndex(index);
              }}
            >
              {tab.label}
            </div>
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

      {/* Certificate confirmation dialog */}
      <CertificateConfirmDialog
        isOpen={showCertDialog}
        certificate={pendingCertificate}
        onConfirm={handleCertificateConfirm}
        onCancel={handleCertificateCancel}
        loading={trustingCertificate}
      />

      {/* Certificate mismatch dialog - critical MITM warning */}
      <CertificateMismatchDialog
        isOpen={showMismatchDialog}
        mismatchInfo={pendingMismatch}
        onAcceptRisk={handleMismatchAcceptRisk}
        onReject={handleMismatchReject}
        loading={acceptingMismatch}
      />
    </div>
  );
}

// Hook to manage settings dialog state
export function useSettingsDialog() {
  const [isOpen, setIsOpen] = useState(false);
  const [initialTab, setInitialTab] = useState<SettingsTab | undefined>(undefined);
  const [autoCheckUpdates, setAutoCheckUpdates] = useState(false);

  useEffect(() => {
    // Listen for open-settings event from tray menu
    const unlistenSettings = listen("open-settings", () => {
      setInitialTab(undefined);
      setAutoCheckUpdates(false);
      setIsOpen(true);
    });

    // Listen for check-for-updates event from tray menu
    const unlistenUpdates = listen("check-for-updates", () => {
      setInitialTab("about");
      setAutoCheckUpdates(true);
      setIsOpen(true);
    });

    return () => {
      unlistenSettings.then((fn) => fn());
      unlistenUpdates.then((fn) => fn());
    };
  }, []);

  const open = useCallback(() => {
    setInitialTab(undefined);
    setAutoCheckUpdates(false);
    setIsOpen(true);
  }, []);

  const openWithTab = useCallback((tab: SettingsTab) => {
    setInitialTab(tab);
    setAutoCheckUpdates(false);
    setIsOpen(true);
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
    // Reset state after close
    setInitialTab(undefined);
    setAutoCheckUpdates(false);
  }, []);

  return { isOpen, open, openWithTab, close, initialTab, autoCheckUpdates };
}
