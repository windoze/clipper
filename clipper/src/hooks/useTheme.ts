import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ThemePreference, Settings } from "../components/SettingsDialog";

export type ResolvedTheme = "light" | "dark";

export function useTheme() {
  const [themePreference, setThemePreference] = useState<ThemePreference>("auto");
  const [resolvedTheme, setResolvedTheme] = useState<ResolvedTheme>("light");

  // Get the system theme preference
  const getSystemTheme = useCallback((): ResolvedTheme => {
    if (typeof window !== "undefined" && window.matchMedia) {
      return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    return "light";
  }, []);

  // Resolve the actual theme based on preference
  const resolveTheme = useCallback((preference: ThemePreference): ResolvedTheme => {
    if (preference === "auto") {
      return getSystemTheme();
    }
    return preference;
  }, [getSystemTheme]);

  // Apply theme to document
  const applyTheme = useCallback((theme: ResolvedTheme) => {
    document.documentElement.setAttribute("data-theme", theme);
    setResolvedTheme(theme);
  }, []);

  // Update theme when preference changes
  const updateTheme = useCallback((preference: ThemePreference) => {
    setThemePreference(preference);
    applyTheme(resolveTheme(preference));
  }, [applyTheme, resolveTheme]);

  // Load initial theme from settings
  useEffect(() => {
    const loadTheme = async () => {
      try {
        const settings = await invoke<Settings>("get_settings");
        setThemePreference(settings.theme);
        applyTheme(resolveTheme(settings.theme));
      } catch (e) {
        // Use auto theme if settings can't be loaded
        applyTheme(resolveTheme("auto"));
      }
    };

    loadTheme();
  }, [applyTheme, resolveTheme]);

  // Listen for system theme changes when in auto mode
  useEffect(() => {
    if (themePreference !== "auto") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (e: MediaQueryListEvent) => {
      applyTheme(e.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [themePreference, applyTheme]);

  return {
    themePreference,
    resolvedTheme,
    updateTheme,
  };
}
