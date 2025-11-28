import {
  createContext,
  useContext,
  useState,
  useCallback,
  ReactNode,
} from "react";

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
  /** Initial notifications enabled state */
  initialNotificationsEnabled?: boolean;
  /** Callback when notifications enabled state changes */
  onNotificationsEnabledChange?: (enabled: boolean) => void;
  /** Custom toast handler (e.g., for system notifications) */
  onShowToast?: (message: string, type: ToastType) => boolean;
}

export function ToastProvider({
  children,
  initialNotificationsEnabled = true,
  onNotificationsEnabledChange,
  onShowToast,
}: ToastProviderProps) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [notificationsEnabled, setNotificationsEnabledState] = useState(
    initialNotificationsEnabled
  );

  const setNotificationsEnabled = useCallback(
    (enabled: boolean) => {
      setNotificationsEnabledState(enabled);
      onNotificationsEnabledChange?.(enabled);
    },
    [onNotificationsEnabledChange]
  );

  const showToast = useCallback(
    (message: string, type: ToastType = "success") => {
      if (!notificationsEnabled) return;

      const id = `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

      // Call custom handler if provided (e.g., for system notifications)
      // If it returns true, it handled the notification, but we still show in-app toast
      onShowToast?.(message, type);

      // Always show in-app toast
      setToasts((prev) => [...prev, { id, message, type }]);

      // Auto-remove after duration
      setTimeout(() => {
        setToasts((prev) => prev.filter((toast) => toast.id !== id));
      }, TOAST_DURATION);
    },
    [notificationsEnabled, onShowToast]
  );

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  return (
    <ToastContext.Provider
      value={{ showToast, notificationsEnabled, setNotificationsEnabled }}
    >
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
