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
  });
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Store the original theme when dialog opens to revert on cancel
  const originalThemeRef = useRef<ThemePreference>("auto");

  // Load settings when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadSettings();
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
    onClose();
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
                  <label htmlFor="serverAddress">Server Address</label>
                  <input
                    id="serverAddress"
                    type="text"
                    value={settings.serverAddress}
                    onChange={(e) =>
                      handleChange("serverAddress", e.target.value)
                    }
                    placeholder="http://localhost:3000"
                  />
                  <p className="settings-hint">
                    URL of the Clipper server for syncing clips. Changes require
                    app restart.
                  </p>
                </div>
              </div>

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
