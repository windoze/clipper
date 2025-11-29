import { useState, useEffect, useCallback, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { ToastProvider, ToastType, useI18n } from "@unwritten-codes/clipper-ui";

interface TauriToastWrapperProps {
  children: ReactNode;
}

export function TauriToastWrapper({ children }: TauriToastWrapperProps) {
  const { t } = useI18n();
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [systemNotificationPermission, setSystemNotificationPermission] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);

  // Load notification setting on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const settings = await invoke<{ notificationsEnabled?: boolean }>("get_settings");
        if (settings.notificationsEnabled !== undefined) {
          setNotificationsEnabled(settings.notificationsEnabled);
        }
      } catch {
        // Use default
      }
      setIsLoaded(true);
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

  // Handle notifications enabled change
  const handleNotificationsEnabledChange = useCallback(async (enabled: boolean) => {
    setNotificationsEnabled(enabled);
    try {
      const settings = await invoke<Record<string, unknown>>("get_settings");
      await invoke("save_settings", {
        settings: { ...settings, notificationsEnabled: enabled },
      });
    } catch (e) {
      console.error("Failed to save notification settings:", e);
    }
  }, []);

  // Handle showing toast with system notification for success messages
  const handleShowToast = useCallback(
    (message: string, type: ToastType): boolean => {
      if (systemNotificationPermission && type === "success") {
        try {
          sendNotification({
            title: t("app.title"),
            body: message,
          });
          return true;
        } catch {
          // Fall through to in-app toast
        }
      }
      return false;
    },
    [systemNotificationPermission, t]
  );

  // Don't render until settings are loaded
  if (!isLoaded) {
    return null;
  }

  return (
    <ToastProvider
      initialNotificationsEnabled={notificationsEnabled}
      onNotificationsEnabledChange={handleNotificationsEnabledChange}
      onShowToast={handleShowToast}
    >
      {children}
    </ToastProvider>
  );
}
