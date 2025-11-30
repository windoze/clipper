import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CleanupConfig } from "@unwritten-codes/clipper-ui";

interface VersionResponse {
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
  };
}

/**
 * Hook to fetch cleanup configuration from the server.
 * Automatically refetches when the server is switched.
 */
export function useFetchCleanupConfig(): CleanupConfig | null {
  const [cleanupConfig, setCleanupConfig] = useState<CleanupConfig | null>(null);

  const fetchConfig = async () => {
    try {
      // Get the current server URL
      const serverUrl = await invoke<string>("get_server_url");

      // Fetch version info from the server
      const response = await fetch(`${serverUrl}/version`);
      if (!response.ok) {
        console.warn("Failed to fetch server version:", response.status);
        setCleanupConfig(null);
        return;
      }

      const data: VersionResponse = await response.json();

      setCleanupConfig({
        enabled: data.config.cleanup_enabled,
        retentionDays: data.config.cleanup_retention_days,
      });
    } catch (err) {
      console.warn("Failed to fetch cleanup config:", err);
      setCleanupConfig(null);
    }
  };

  useEffect(() => {
    // Fetch on mount
    fetchConfig();

    // Re-fetch when server is switched
    const unlistenServerSwitched = listen("server-switched", () => {
      fetchConfig();
    });

    // Re-fetch when data is cleared (server restart)
    const unlistenDataCleared = listen("data-cleared", () => {
      fetchConfig();
    });

    return () => {
      unlistenServerSwitched.then((fn) => fn());
      unlistenDataCleared.then((fn) => fn());
    };
  }, []);

  return cleanupConfig;
}
