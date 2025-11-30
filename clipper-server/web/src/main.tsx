import { StrictMode, useState, useCallback, useEffect } from "react";
import { createRoot } from "react-dom/client";
import {
  I18nProvider,
  ToastProvider,
  ApiProvider,
  createRestApiClient,
  useI18n,
} from "@unwritten-codes/clipper-ui";
import { CleanupConfigWrapper } from "./components/CleanupConfigWrapper";
import { LoginScreen } from "./components/LoginScreen";
import App from "./App";
import "./App.css";

// Storage key for the auth token
const AUTH_TOKEN_KEY = "clipper-web-token";

// Create the REST API client for the web UI
const api = createRestApiClient({
  baseUrl: "",
  token: localStorage.getItem(AUTH_TOKEN_KEY) || undefined,
});

// Check if server requires authentication
async function checkAuthRequired(): Promise<boolean> {
  try {
    const response = await fetch("/auth/check");
    if (response.ok) {
      const data = await response.json();
      return data.auth_required === true;
    }
  } catch (e) {
    console.error("Failed to check auth status:", e);
  }
  return false;
}

// Validate the current token by making a test request
async function validateToken(token: string): Promise<boolean> {
  try {
    const response = await fetch("/clips?page=1&page_size=1", {
      headers: { Authorization: `Bearer ${token}` },
    });
    return response.ok;
  } catch (e) {
    return false;
  }
}

function AuthWrapper() {
  const { t } = useI18n();
  const [isLoading, setIsLoading] = useState(true);
  const [authRequired, setAuthRequired] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [authError, setAuthError] = useState<string>();

  // Check auth status on mount
  useEffect(() => {
    async function checkAuth() {
      const required = await checkAuthRequired();
      setAuthRequired(required);

      if (required) {
        // Check if we have a saved token that's still valid
        const savedToken = localStorage.getItem(AUTH_TOKEN_KEY);
        if (savedToken) {
          const isValid = await validateToken(savedToken);
          if (isValid) {
            api.setToken(savedToken);
            setIsAuthenticated(true);
          } else {
            // Token is invalid, clear it
            localStorage.removeItem(AUTH_TOKEN_KEY);
            api.setToken(undefined);
          }
        }
      } else {
        // No auth required, proceed
        setIsAuthenticated(true);
      }

      setIsLoading(false);
    }

    checkAuth();
  }, []);

  // Handle login
  const handleLogin = useCallback(async (token: string) => {
    setAuthError(undefined);

    const isValid = await validateToken(token);
    if (isValid) {
      localStorage.setItem(AUTH_TOKEN_KEY, token);
      api.setToken(token);
      setIsAuthenticated(true);
    } else {
      setAuthError(t("auth.error"));
    }
  }, [t]);

  // Check token validity on window focus
  useEffect(() => {
    const handleVisibilityChange = async () => {
      if (document.visibilityState === "visible" && authRequired && isAuthenticated) {
        const savedToken = localStorage.getItem(AUTH_TOKEN_KEY);
        if (savedToken) {
          const isValid = await validateToken(savedToken);
          if (!isValid) {
            localStorage.removeItem(AUTH_TOKEN_KEY);
            api.setToken(undefined);
            setIsAuthenticated(false);
            setAuthError(t("auth.sessionExpired"));
          }
        }
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [authRequired, isAuthenticated, t]);

  if (isLoading) {
    return (
      <div className="app-loading">
        <div className="loading-spinner" />
      </div>
    );
  }

  if (authRequired && !isAuthenticated) {
    return <LoginScreen onLogin={handleLogin} error={authError} />;
  }

  // Get the current token for WebSocket authentication
  const currentToken = authRequired ? localStorage.getItem(AUTH_TOKEN_KEY) || undefined : undefined;

  return (
    <CleanupConfigWrapper>
      <App authToken={currentToken} />
    </CleanupConfigWrapper>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <I18nProvider storageKey="clipper-web-language">
      <ApiProvider value={api}>
        <ToastProvider>
          <AuthWrapper />
        </ToastProvider>
      </ApiProvider>
    </I18nProvider>
  </StrictMode>
);
