import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type ThemePreference = "light" | "dark" | "auto";

export interface Settings {
  serverAddress: string;
  defaultSaveLocation: string | null;
  openOnStartup: boolean;
  startOnLogin: boolean;
  theme: ThemePreference;
  useBundledServer: boolean;
  listenOnAllInterfaces: boolean;
}

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onThemeChange?: (theme: ThemePreference) => void;
}

export function SettingsDialog({ isOpen, onClose, onThemeChange }: SettingsDialogProps) {
  const [settings, setSettings] = useState<Settings>({
    serverAddress: "http://localhost:3000",
    defaultSaveLocation: null,
    openOnStartup: true,
    startOnLogin: false,
    theme: "auto",
    useBundledServer: true,
    listenOnAllInterfaces: false,
  });
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [serverUrl, setServerUrl] = useState<string>("");
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [localIpAddresses, setLocalIpAddresses] = useState<string[]>([]);
  const [togglingNetworkAccess, setTogglingNetworkAccess] = useState(false);

  // Store the original theme when dialog opens to revert on cancel
  const originalThemeRef = useRef<ThemePreference>("auto");

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
      // Store the original theme to revert on cancel
      originalThemeRef.current = loadedSettings.theme;
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

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await invoke("save_settings", { settings });
      // Theme is already applied via preview, just close the dialog
      onClose();
    } catch (e) {
      setError(`Failed to save settings: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleBrowseDirectory = async () => {
    try {
      const path = await invoke<string | null>("browse_directory");
      if (path) {
        setSettings((prev) => ({ ...prev, defaultSaveLocation: path }));
      }
    } catch (e) {
      setError(`Failed to browse directory: ${e}`);
    }
  };

  const handleChange = (
    field: keyof Settings,
    value: string | boolean | null
  ) => {
    setSettings((prev) => ({ ...prev, [field]: value }));

    // Preview theme change immediately
    if (field === "theme" && typeof value === "string") {
      onThemeChange?.(value as ThemePreference);
    }
  };

  // Handle cancel - revert theme to original
  const handleCancel = () => {
    // Revert theme to original if it was changed
    if (settings.theme !== originalThemeRef.current) {
      onThemeChange?.(originalThemeRef.current);
    }
    setShowClearConfirm(false);
    onClose();
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
      } else {
        // Switch to external server
        await invoke("switch_to_external_server", { serverUrl: settings.serverAddress });
        setServerUrl(settings.serverAddress);
        setSettings((prev) => ({ ...prev, useBundledServer: false }));
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
    <div className="settings-backdrop" onClick={handleCancel}>
      <div className="settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close" onClick={handleCancel}>
            &times;
          </button>
        </div>

        <div className="settings-content">
          {loading ? (
            <div className="settings-loading">
              <div className="loading-spinner"></div>
              <span>Loading settings...</span>
            </div>
          ) : (
            <>
              {error && <div className="settings-error">{error}</div>}

              <div className="settings-section">
                <h3>Appearance</h3>
                <div className="settings-field">
                  <label htmlFor="theme">Theme</label>
                  <div className="theme-selector">
                    <button
                      type="button"
                      className={`theme-option ${settings.theme === "light" ? "active" : ""}`}
                      onClick={() => handleChange("theme", "light")}
                    >
                      <span className="theme-icon">&#9788;</span>
                      <span>Light</span>
                    </button>
                    <button
                      type="button"
                      className={`theme-option ${settings.theme === "dark" ? "active" : ""}`}
                      onClick={() => handleChange("theme", "dark")}
                    >
                      <span className="theme-icon">&#9790;</span>
                      <span>Dark</span>
                    </button>
                    <button
                      type="button"
                      className={`theme-option ${settings.theme === "auto" ? "active" : ""}`}
                      onClick={() => handleChange("theme", "auto")}
                    >
                      <span className="theme-icon">&#9881;</span>
                      <span>Auto</span>
                    </button>
                  </div>
                  <p className="settings-hint">
                    Choose your preferred color theme. Auto follows your system settings.
                  </p>
                </div>
              </div>

              <div className="settings-section">
                <h3>Server</h3>
                <div className="settings-field">
                  <label>Server Mode</label>
                  <div className="server-mode-selector">
                    <button
                      type="button"
                      className={`server-mode-option ${settings.useBundledServer ? "active" : ""}`}
                      onClick={() => handleServerModeChange(true)}
                    >
                      <span className="server-mode-icon">&#9881;</span>
                      <span>Bundled</span>
                    </button>
                    <button
                      type="button"
                      className={`server-mode-option ${!settings.useBundledServer ? "active" : ""}`}
                      onClick={() => handleServerModeChange(false)}
                    >
                      <span className="server-mode-icon">&#8599;</span>
                      <span>External</span>
                    </button>
                  </div>
                  <p className="settings-hint">
                    {settings.useBundledServer
                      ? "Use the bundled server (automatically managed). Data is stored locally."
                      : "Connect to an external clipper-server instance."}
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
                        {togglingNetworkAccess ? "Restarting server..." : "Allow network access"}
                      </span>
                    </label>
                    <p className="settings-hint">
                      When enabled, the server will listen on all network interfaces, allowing other devices on your network to access clips.
                    </p>
                  </div>
                )}

                {settings.useBundledServer && settings.listenOnAllInterfaces && (
                  <div className="settings-field">
                    <label>Server URLs</label>
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
                                title="Copy to clipboard"
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
                        <p className="settings-hint">No network interfaces found.</p>
                      )}
                    </div>
                    <p className="settings-hint">
                      Use any of these URLs to access the server from other devices on your network.
                    </p>
                  </div>
                )}

                {!settings.useBundledServer && (
                  <div className="settings-field">
                    <label htmlFor="serverUrl">Server URL</label>
                    <input
                      id="serverUrl"
                      type="text"
                      value={settings.serverAddress}
                      onChange={(e) => handleChange("serverAddress", e.target.value)}
                      placeholder="http://localhost:3000"
                    />
                    <p className="settings-hint">
                      Enter the URL of your external clipper-server.
                    </p>
                  </div>
                )}
              </div>

              {settings.useBundledServer && (
                <div className="settings-section">
                  <h3>Storage</h3>
                  <div className="settings-field">
                    <label htmlFor="defaultSaveLocation">
                      Default Save Location
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
                        placeholder="System default"
                      />
                      <button
                        className="browse-button"
                        onClick={handleBrowseDirectory}
                      >
                        Browse...
                      </button>
                    </div>
                    <p className="settings-hint">
                      Default folder for saving downloaded attachments.
                    </p>
                  </div>
                </div>
              )}

              <div className="settings-section">
                <h3>Startup</h3>
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
                      Open main window on startup
                    </span>
                  </label>
                  <p className="settings-hint">
                    Show the main window when the app starts. If disabled, the
                    app will start minimized to the system tray.
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
                      Start application on login
                    </span>
                  </label>
                  <p className="settings-hint">
                    Automatically start Clipper when you log in to your
                    computer.
                  </p>
                </div>
              </div>

              {settings.useBundledServer && (
                <div className="settings-section">
                  <h3>Data Management</h3>
                  <div className="settings-field">
                    <label>Clear All Data</label>
                    {!showClearConfirm ? (
                      <>
                        <button
                          type="button"
                          className="settings-btn danger"
                          onClick={() => setShowClearConfirm(true)}
                          disabled={clearing}
                        >
                          Clear All Clips
                        </button>
                        <p className="settings-hint">
                          Permanently delete all stored clips and attachments. This action cannot be undone.
                        </p>
                      </>
                    ) : (
                      <div className="clear-confirm">
                        <p className="clear-confirm-message">
                          Are you sure you want to delete all clips? This will stop the server, delete all data, and restart.
                        </p>
                        <div className="clear-confirm-buttons">
                          <button
                            type="button"
                            className="settings-btn secondary"
                            onClick={() => setShowClearConfirm(false)}
                            disabled={clearing}
                          >
                            Cancel
                          </button>
                          <button
                            type="button"
                            className="settings-btn danger"
                            onClick={handleClearData}
                            disabled={clearing}
                          >
                            {clearing ? "Clearing..." : "Yes, Delete All"}
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
          <button className="settings-btn secondary" onClick={handleCancel}>
            Cancel
          </button>
          <button
            className="settings-btn primary"
            onClick={handleSave}
            disabled={loading || saving}
          >
            {saving ? "Saving..." : "Save"}
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
