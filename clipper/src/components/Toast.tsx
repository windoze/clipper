import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { useI18n } from "../i18n";

export type ToastType = "success" | "error" | "info";

interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

interface ToastContextValue {
  showToast: (message: string, type?: ToastType) => void;
  notificationsEnabled: boolean;
  setNotificationsEnabled: (enabled: boolean) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

const TOAST_DURATION = 3000;

interface ToastProviderProps {
  children: ReactNode;
}

export function ToastProvider({ children }: ToastProviderProps) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [notificationsEnabled, setNotificationsEnabledState] = useState(true);
  const [systemNotificationPermission, setSystemNotificationPermission] = useState<boolean>(false);
  const { t } = useI18n();

  // Load notification setting on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const settings = await invoke<{ notificationsEnabled?: boolean }>("get_settings");
        if (settings.notificationsEnabled !== undefined) {
          setNotificationsEnabledState(settings.notificationsEnabled);
        }
      } catch {
        // Use default
      }
    };
    loadSettings();
  }, []);

  // Check and request system notification permission
  useEffect(() => {
    const checkPermission = async () => {
      try {
        let granted = await isPermissionGranted();
        if (!granted) {
          const permission = await requestPermission();
          granted = permission === "granted";
        }
        setSystemNotificationPermission(granted);
      } catch {
        setSystemNotificationPermission(false);
      }
    };
    checkPermission();
  }, []);

  const setNotificationsEnabled = useCallback(async (enabled: boolean) => {
    setNotificationsEnabledState(enabled);
    // Save to settings
    try {
      const settings = await invoke<Record<string, unknown>>("get_settings");
      await invoke("save_settings", {
        settings: { ...settings, notificationsEnabled: enabled },
      });
    } catch (e) {
      console.error("Failed to save notification settings:", e);
    }
  }, []);

  const showToast = useCallback(
    (message: string, type: ToastType = "success") => {
      if (!notificationsEnabled) return;

      const id = `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

      // Try system notification first for success messages
      if (systemNotificationPermission && type === "success") {
        try {
          sendNotification({
            title: t("app.title"),
            body: message,
          });
        } catch {
          // Fall back to in-app toast
        }
      }

      // Always show in-app toast as well
      setToasts((prev) => [...prev, { id, message, type }]);

      // Auto-remove after duration
      setTimeout(() => {
        setToasts((prev) => prev.filter((toast) => toast.id !== id));
      }, TOAST_DURATION);
    },
    [notificationsEnabled, systemNotificationPermission, t]
  );

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ showToast, notificationsEnabled, setNotificationsEnabled }}>
      {children}
      <ToastContainer toasts={toasts} onRemove={removeToast} />
    </ToastContext.Provider>
  );
}

// Toast container component
interface ToastContainerProps {
  toasts: Toast[];
  onRemove: (id: string) => void;
}

function ToastContainer({ toasts, onRemove }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div className="toast-container">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`toast toast-${toast.type}`}
          onClick={() => onRemove(toast.id)}
        >
          <span className="toast-icon">
            {toast.type === "success" && "✓"}
            {toast.type === "error" && "✕"}
            {toast.type === "info" && "ℹ"}
          </span>
          <span className="toast-message">{toast.message}</span>
        </div>
      ))}
    </div>
  );
}

// Hook to use toast
export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}
