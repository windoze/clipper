import { useState, useEffect, useCallback, createContext, useContext } from "react";

export type SyntaxTheme = "github" | "monokai" | "dracula" | "nord";

export const SYNTAX_THEMES: SyntaxTheme[] = ["github", "monokai", "dracula", "nord"];

interface UseSyntaxThemeOptions {
  /** Initial syntax theme */
  initialTheme?: SyntaxTheme;
  /** Storage key for persisting theme preference */
  storageKey?: string;
  /** Custom theme loader (e.g., from Tauri settings) */
  loadTheme?: () => Promise<SyntaxTheme>;
  /** Custom theme saver (e.g., to Tauri settings) */
  saveTheme?: (theme: SyntaxTheme) => Promise<void>;
}

function applySyntaxTheme(theme: SyntaxTheme) {
  const root = document.documentElement;
  root.setAttribute("data-syntax-theme", theme);
}

export function useSyntaxTheme(options: UseSyntaxThemeOptions = {}) {
  const {
    initialTheme,
    storageKey = "clipper-syntax-theme",
    loadTheme,
    saveTheme,
  } = options;

  const [syntaxTheme, setSyntaxTheme] = useState<SyntaxTheme>(() => {
    if (initialTheme) return initialTheme;
    // Try localStorage first
    if (typeof localStorage !== "undefined") {
      const stored = localStorage.getItem(storageKey);
      if (stored && SYNTAX_THEMES.includes(stored as SyntaxTheme)) {
        return stored as SyntaxTheme;
      }
    }
    return "github";
  });

  const [isLoaded, setIsLoaded] = useState(!loadTheme);

  // Load theme from custom loader if provided
  useEffect(() => {
    if (loadTheme) {
      loadTheme().then((theme) => {
        setSyntaxTheme(theme);
        applySyntaxTheme(theme);
        setIsLoaded(true);
      });
    }
  }, [loadTheme]);

  // Apply theme on mount and when theme changes
  useEffect(() => {
    if (!isLoaded) return;
    applySyntaxTheme(syntaxTheme);
    // Save to localStorage
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(storageKey, syntaxTheme);
    }
  }, [syntaxTheme, storageKey, isLoaded]);

  const updateSyntaxTheme = useCallback(
    async (newTheme: SyntaxTheme) => {
      setSyntaxTheme(newTheme);
      if (saveTheme) {
        await saveTheme(newTheme);
      }
    },
    [saveTheme]
  );

  return {
    syntaxTheme,
    setSyntaxTheme: updateSyntaxTheme,
    isLoaded,
  };
}

// Context for syntax theme
interface SyntaxThemeContextValue {
  syntaxTheme: SyntaxTheme;
  setSyntaxTheme: (theme: SyntaxTheme) => void;
}

const SyntaxThemeContext = createContext<SyntaxThemeContextValue | null>(null);

export const SyntaxThemeProvider = SyntaxThemeContext.Provider;

export function useSyntaxThemeContext(): SyntaxThemeContextValue {
  const context = useContext(SyntaxThemeContext);
  if (!context) {
    // Return a default if not in context (for backwards compatibility)
    return {
      syntaxTheme: "github",
      setSyntaxTheme: () => {},
    };
  }
  return context;
}
