import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n, Language, supportedLanguages, languageNames } from "../i18n";
import { useToast } from "./Toast";

export type ThemePreference = "light" | "dark" | "auto";

export interface Settings {
  serverAddress: string;
  defaultSaveLocation: string | null;
  openOnStartup: boolean;
  startOnLogin: boolean;
  theme: ThemePreference;
  useBundledServer: boolean;
  listenOnAllInterfaces: boolean;
  language: string | null;
  notificationsEnabled: boolean;
}

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onThemeChange?: (theme: ThemePreference) => void;
}

export function SettingsDialog({ isOpen, onClose, onThemeChange }: SettingsDialogProps) {
  const { t, language: currentLanguage, setLanguage } = useI18n();
  const { showToast } = useToast();
  const [settings, setSettings] = useState<Settings>({
    serverAddress: "http://localhost:3000",
    defaultSaveLocation: null,
    openOnStartup: true,
    startOnLogin: false,
    theme: "auto",
    useBundledServer: true,
    listenOnAllInterfaces: false,
    language: null,
    notificationsEnabled: true,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [serverUrl, setServerUrl] = useState<string>("");
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [localIpAddresses, setLocalIpAddresses] = useState<string[]>([]);
  const [togglingNetworkAccess, setTogglingNetworkAccess] = useState(false);
  // Track the original server address to detect changes on close
  const [originalServerAddress, setOriginalServerAddress] = useState<string>("");

  // Load settings when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadSettings();
      loadServerInfo();
      loadLocalIpAddresses();
    }
  }, [isOpen]);

  const loadSettings = async () => {
    setLoading(true);
    setError(null);
    try {
      const loadedSettings = await invoke<Settings>("get_settings");
      setSettings(loadedSettings);
      // Store the original server address to detect changes on close
      setOriginalServerAddress(loadedSettings.serverAddress);
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
    } catch (e) {
      console.error("Failed to load server info:", e);
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
    value: string | boolean | null
  ) => {
    const newSettings = { ...settings, [field]: value };
    setSettings(newSettings);

    // Apply theme change immediately
    if (field === "theme" && typeof value === "string") {
      onThemeChange?.(value as ThemePreference);
    }

    // Save settings immediately
    await saveSettings(newSettings);
  };

  // Handle close - reconnect if server URL changed while using external server
  const handleClose = async () => {
    setShowClearConfirm(false);

    // If using external server and the server address changed, reconnect
    if (!settings.useBundledServer && settings.serverAddress !== originalServerAddress) {
      try {
        await invoke("switch_to_external_server", { serverUrl: settings.serverAddress });
        setServerUrl(settings.serverAddress);
        showToast(t("toast.serverConnected"));
      } catch (e) {
        console.error("Failed to reconnect to server:", e);
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
    try {
      if (useBundled) {
        // Switch to bundled server
        const newUrl = await invoke<string>("switch_to_bundled_server");
        setServerUrl(newUrl);
        setSettings((prev) => ({ ...prev, useBundledServer: true }));
        showToast(t("toast.serverStarted"));
      } else {
        // Switch to external server
        await invoke("switch_to_external_server", { serverUrl: settings.serverAddress });
        setServerUrl(settings.serverAddress);
        setSettings((prev) => ({ ...prev, useBundledServer: false }));
        showToast(t("toast.serverConnected"));
      }
    } catch (e) {
      setError(`Failed to switch server mode: ${e}`);
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
      setSettings((prev) => ({ ...prev, listenOnAllInterfaces: listenOnAll }));
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

  if (!isOpen) return null;

  return (
    <div className="settings-backdrop" onClick={handleClose}>
      <div className="settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>{t("settings.title")}</h2>
          <button className="settings-close" onClick={handleClose}>
            &times;
          </button>
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

              <div className="settings-section">
                <h3>{t("settings.appearance")}</h3>
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
              </div>

              <div className="settings-section">
                <h3>{t("settings.startup")}</h3>
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

              <div className="settings-section">
                <h3>{t("settings.server")}</h3>
                <div className="settings-field">
                  <label>{t("settings.serverMode")}</label>
                  <div className="server-mode-selector">
                    <button
                      type="button"
                      className={`server-mode-option ${settings.useBundledServer ? "active" : ""}`}
                      onClick={() => handleServerModeChange(true)}
                    >
                      <span className="server-mode-icon">&#9881;</span>
                      <span>{t("settings.serverMode.bundled")}</span>
                    </button>
                    <button
                      type="button"
                      className={`server-mode-option ${!settings.useBundledServer ? "active" : ""}`}
                      onClick={() => handleServerModeChange(false)}
                    >
                      <span className="server-mode-icon">&#8599;</span>
                      <span>{t("settings.serverMode.external")}</span>
                    </button>
                  </div>
                  <p className="settings-hint">
                    {settings.useBundledServer
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
                )}

                {!settings.useBundledServer && (
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
