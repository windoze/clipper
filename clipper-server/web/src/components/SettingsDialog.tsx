import { useState, useEffect, useCallback, useRef } from "react";
import {
  useI18n,
  languageNames,
  supportedLanguages,
  SYNTAX_THEMES,
} from "@unwritten-codes/clipper-ui";
import type { Language, Theme, SyntaxTheme } from "@unwritten-codes/clipper-ui";

// Storage key for the auth token (same as in main.tsx)
const AUTH_TOKEN_KEY = "clipper-web-token";

type SettingsTab = "appearance" | "about";

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
    short_url_enabled: boolean;
    short_url_base?: string;
    short_url_expiration_hours?: number;
  };
}

interface GitHubRelease {
  tag_name: string;
}

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  syntaxTheme: SyntaxTheme;
  onSyntaxThemeChange: (theme: SyntaxTheme) => void;
}

// Format uptime in human-friendly format
function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);

  return parts.join(" ");
}

// Compare versions (returns true if latest > current)
function isNewerVersion(latest: string, current: string): boolean {
  // Remove 'v' prefix if present
  const latestClean = latest.replace(/^v/, "");
  const currentClean = current.replace(/^v/, "");

  const latestParts = latestClean.split(".").map((n) => parseInt(n, 10) || 0);
  const currentParts = currentClean.split(".").map((n) => parseInt(n, 10) || 0);

  for (let i = 0; i < Math.max(latestParts.length, currentParts.length); i++) {
    const l = latestParts[i] || 0;
    const c = currentParts[i] || 0;
    if (l > c) return true;
    if (l < c) return false;
  }
  return false;
}

export function SettingsDialog({
  isOpen,
  onClose,
  theme,
  onThemeChange,
  syntaxTheme,
  onSyntaxThemeChange,
}: SettingsDialogProps) {
  const { t, language, setLanguage } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>("appearance");
  const [focusedTabIndex, setFocusedTabIndex] = useState(0);
  const tabsRef = useRef<HTMLDivElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);

  // About tab state
  const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null);
  const [totalClips, setTotalClips] = useState<number | null>(null);
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [updateCheckFailed, setUpdateCheckFailed] = useState(false);
  const [loadingServerInfo, setLoadingServerInfo] = useState(false);

  // Reset to appearance tab when dialog opens
  useEffect(() => {
    if (isOpen) {
      setActiveTab("appearance");
      setFocusedTabIndex(0);
    }
  }, [isOpen]);

  // Handle tab keyboard navigation
  const handleTabKeyDown = (e: React.KeyboardEvent) => {
    const tabCount = 2; // appearance, about
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      setFocusedTabIndex((prev) => (prev - 1 + tabCount) % tabCount);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      setFocusedTabIndex((prev) => (prev + 1) % tabCount);
    } else if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      const tabs: SettingsTab[] = ["appearance", "about"];
      setActiveTab(tabs[focusedTabIndex]);
    }
  };

  // Load server info when About tab is selected
  useEffect(() => {
    if (isOpen && activeTab === "about") {
      loadServerInfo();
      loadTotalClips();
      checkForUpdates();
    }
  }, [isOpen, activeTab]);

  const loadServerInfo = async () => {
    setLoadingServerInfo(true);
    try {
      const headers: Record<string, string> = {};
      const token = localStorage.getItem(AUTH_TOKEN_KEY);
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }
      const response = await fetch("/version", { headers });
      if (response.ok) {
        const data = await response.json();
        setServerInfo(data);
      }
    } catch (e) {
      console.error("Failed to load server info:", e);
    } finally {
      setLoadingServerInfo(false);
    }
  };

  const loadTotalClips = async () => {
    try {
      const headers: Record<string, string> = {};
      const token = localStorage.getItem(AUTH_TOKEN_KEY);
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }
      // Fetch with page_size=1 just to get the total count
      const response = await fetch("/clips?page=1&page_size=1", { headers });
      if (response.ok) {
        const data = await response.json();
        setTotalClips(data.total);
      }
    } catch (e) {
      console.error("Failed to load total clips:", e);
    }
  };

  const checkForUpdates = async () => {
    setCheckingUpdates(true);
    setUpdateCheckFailed(false);
    setLatestVersion(null);
    try {
      const response = await fetch(
        "https://api.github.com/repos/windoze/clipper/releases/latest"
      );
      if (response.ok) {
        const data: GitHubRelease = await response.json();
        setLatestVersion(data.tag_name);
      } else {
        setUpdateCheckFailed(true);
      }
    } catch (e) {
      console.error("Failed to check for updates:", e);
      setUpdateCheckFailed(true);
    } finally {
      setCheckingUpdates(false);
    }
  };

  // Handle ESC key and Tab focus trapping
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
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

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "appearance", label: t("settings.tab.appearance") },
    { id: "about", label: t("settings.tab.about") },
  ];

  const renderAppearanceTab = () => (
    <div className="settings-section">
      <h3>{t("settings.appearance")}</h3>

      {/* Theme */}
      <div className="settings-field">
        <label>{t("settings.theme")}</label>
        <div className="theme-selector">
          <button
            className={`theme-option ${theme === "light" ? "active" : ""}`}
            onClick={() => onThemeChange("light")}
          >
            <span className="theme-icon">☀️</span>
            <span>{t("settings.theme.light")}</span>
          </button>
          <button
            className={`theme-option ${theme === "dark" ? "active" : ""}`}
            onClick={() => onThemeChange("dark")}
          >
            <span className="theme-icon">🌙</span>
            <span>{t("settings.theme.dark")}</span>
          </button>
          <button
            className={`theme-option ${theme === "auto" ? "active" : ""}`}
            onClick={() => onThemeChange("auto")}
          >
            <span className="theme-icon">💻</span>
            <span>{t("settings.theme.auto")}</span>
          </button>
        </div>
        <p className="settings-hint">{t("settings.theme.hint")}</p>
      </div>

      {/* Language */}
      <div className="settings-field">
        <label>{t("settings.language")}</label>
        <select
          className="settings-select"
          value={language}
          onChange={(e) => setLanguage(e.target.value as Language)}
        >
          {supportedLanguages.map((lang) => (
            <option key={lang} value={lang}>
              {languageNames[lang]}
            </option>
          ))}
        </select>
        <p className="settings-hint">{t("settings.language.hint")}</p>
      </div>

      {/* Syntax Theme */}
      <div className="settings-field">
        <label>{t("settings.syntaxTheme")}</label>
        <select
          className="settings-select"
          value={syntaxTheme}
          onChange={(e) => onSyntaxThemeChange(e.target.value as SyntaxTheme)}
        >
          {SYNTAX_THEMES.map((themeOption) => (
            <option key={themeOption} value={themeOption}>
              {t(`settings.syntaxTheme.${themeOption}` as const)}
            </option>
          ))}
        </select>
        <p className="settings-hint">{t("settings.syntaxTheme.hint")}</p>
      </div>
    </div>
  );

  const renderAboutTab = () => (
    <>
      <div className="settings-section">
        <h3>{t("settings.about.serverInfo")}</h3>

        {loadingServerInfo ? (
          <p className="settings-hint">{t("common.loading")}</p>
        ) : serverInfo ? (
          <div className="settings-about-grid">
            <div className="settings-about-item">
              <span className="settings-about-label">{t("settings.about.version")}</span>
              <span className="settings-about-value">{serverInfo.version}</span>
            </div>

            <div className="settings-about-item">
              <span className="settings-about-label">{t("settings.about.uptime")}</span>
              <span className="settings-about-value">{formatUptime(serverInfo.uptime_secs)}</span>
            </div>

            <div className="settings-about-item">
              <span className="settings-about-label">{t("settings.about.totalClips")}</span>
              <span className="settings-about-value">
                {totalClips !== null ? totalClips.toLocaleString() : "-"}
              </span>
            </div>

            <div className="settings-about-item">
              <span className="settings-about-label">{t("settings.about.tlsEnabled")}</span>
              <span className="settings-about-value">
                {serverInfo.config.tls_enabled
                  ? t("settings.about.tlsEnabled.yes")
                  : t("settings.about.tlsEnabled.no")}
              </span>
            </div>

            {serverInfo.config.cleanup_enabled && serverInfo.config.cleanup_retention_days && (
              <div className="settings-about-item">
                <span className="settings-about-label">{t("settings.about.retentionPeriod")}</span>
                <span className="settings-about-value">
                  {t("settings.about.retentionPeriod.days").replace(
                    "{days}",
                    String(serverInfo.config.cleanup_retention_days)
                  )}
                </span>
              </div>
            )}

            {serverInfo.config.short_url_enabled && serverInfo.config.short_url_base && (
              <div className="settings-about-item">
                <span className="settings-about-label">{t("settings.about.shortUrlBase")}</span>
                <span className="settings-about-value">{serverInfo.config.short_url_base}</span>
              </div>
            )}
          </div>
        ) : (
          <p className="settings-hint">{t("common.error")}</p>
        )}
      </div>

      <div className="settings-section">
        <h3>{t("settings.about.updates")}</h3>

        <div className="settings-field">
          {checkingUpdates ? (
            <p className="settings-hint">{t("settings.about.checkingUpdates")}</p>
          ) : updateCheckFailed ? (
            <p className="settings-error">{t("settings.about.checkFailed")}</p>
          ) : latestVersion && serverInfo ? (
            isNewerVersion(latestVersion, serverInfo.version) ? (
              <p className="settings-update-available">
                {t("settings.about.updateAvailable").replace("{version}", latestVersion)}
              </p>
            ) : (
              <p className="settings-update-uptodate">{t("settings.about.upToDate")}</p>
            )
          ) : null}
        </div>
      </div>
    </>
  );

  return (
    <div className="settings-backdrop" onClick={onClose}>
      <div ref={dialogRef} className="settings-dialog settings-dialog-tabbed" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>{t("settings.title")}</h2>
          <div className="settings-close" onClick={onClose} role="button" aria-label="Close">
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
          {activeTab === "appearance" && renderAppearanceTab()}
          {activeTab === "about" && renderAboutTab()}
        </div>

        <div className="settings-footer">
          <button className="settings-btn primary" onClick={onClose}>
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

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);

  return { isOpen, open, close };
}
