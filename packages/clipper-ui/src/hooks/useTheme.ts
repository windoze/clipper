import { useState, useEffect, useCallback } from "react";

export type Theme = "light" | "dark" | "auto";
export type ResolvedTheme = "light" | "dark";

interface UseThemeOptions {
  /** Initial theme preference */
  initialTheme?: Theme;
  /** Storage key for persisting theme preference */
  storageKey?: string;
  /** Custom theme loader (e.g., from Tauri settings) */
  loadTheme?: () => Promise<Theme>;
  /** Custom theme saver (e.g., to Tauri settings) */
  saveTheme?: (theme: Theme) => Promise<void>;
}

function getSystemTheme(): ResolvedTheme {
  if (
    typeof window !== "undefined" &&
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  ) {
    return "dark";
  }
  return "light";
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "auto") {
    // For auto, set the resolved theme
    root.setAttribute("data-theme", getSystemTheme());
  } else {
    root.setAttribute("data-theme", theme);
  }
}

function resolveTheme(theme: Theme): ResolvedTheme {
  if (theme === "auto") {
    return getSystemTheme();
  }
  return theme;
}

export function useTheme(options: UseThemeOptions = {}) {
  const {
    initialTheme,
    storageKey = "clipper-theme",
    loadTheme,
    saveTheme,
  } = options;

  const [themePreference, setThemePreference] = useState<Theme>(() => {
    if (initialTheme) return initialTheme;
    // Try localStorage first
    if (typeof localStorage !== "undefined") {
      const stored = localStorage.getItem(storageKey);
      if (stored === "light" || stored === "dark" || stored === "auto") {
        return stored;
      }
    }
    return "auto";
  });

  const [resolvedTheme, setResolvedTheme] = useState<ResolvedTheme>(() =>
    resolveTheme(themePreference)
  );

  const [isLoaded, setIsLoaded] = useState(!loadTheme);

  // Load theme from custom loader if provided
  useEffect(() => {
    if (loadTheme) {
      loadTheme().then((theme) => {
        setThemePreference(theme);
        applyTheme(theme);
        setResolvedTheme(resolveTheme(theme));
        setIsLoaded(true);
      });
    }
  }, [loadTheme]);

  // Apply theme on mount and when theme changes
  useEffect(() => {
    if (!isLoaded) return;
    applyTheme(themePreference);
    setResolvedTheme(resolveTheme(themePreference));
    // Save to localStorage
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(storageKey, themePreference);
    }
  }, [themePreference, storageKey, isLoaded]);

  // Listen for system theme changes when using auto
  useEffect(() => {
    if (themePreference !== "auto") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = (e: MediaQueryListEvent) => {
      applyTheme("auto");
      setResolvedTheme(e.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [themePreference]);

  const updateTheme = useCallback(
    async (newTheme: Theme) => {
      setThemePreference(newTheme);
      if (saveTheme) {
        await saveTheme(newTheme);
      }
    },
    [saveTheme]
  );

  return {
    /** The user's theme preference (light, dark, or auto) */
    themePreference,
    /** The resolved theme (light or dark) based on preference and system settings */
    resolvedTheme,
    /** Update the theme preference */
    updateTheme,
    /** Alias for updateTheme for backwards compatibility */
    setTheme: updateTheme,
    /** The current system theme */
    systemTheme: getSystemTheme(),
    /** Alias for themePreference for backwards compatibility */
    theme: themePreference,
  };
}
