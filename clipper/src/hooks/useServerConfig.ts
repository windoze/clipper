import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ServerConfig } from "@unwritten-codes/clipper-ui";

interface VersionResponse {
  version: string;
  uptime_secs: number;
  active_ws_connections: number;
  index_version: number;
  config: {
    port: number;
    tls_enabled: boolean;
    tls_port?: number;
    acme_enabled: boolean;
    acme_domain?: string;
    cleanup_enabled: boolean;
    cleanup_interval_mins?: number;
    cleanup_retention_days?: number;
    short_url_enabled: boolean;
    short_url_base?: string;
    short_url_expiration_hours?: number;
  };
}

/**
 * Hook to fetch server configuration from the server.
 * Automatically refetches when the server is switched.
 */
export function useFetchServerConfig(): ServerConfig | null {
  const [serverConfig, setServerConfig] = useState<ServerConfig | null>(null);

  const fetchConfig = async () => {
    try {
      // Get the current server URL
      const serverUrl = await invoke<string>("get_server_url");

      // Fetch version info from the server
      const response = await fetch(`${serverUrl}/version`);
      if (!response.ok) {
        console.warn("Failed to fetch server version:", response.status);
        setServerConfig(null);
        return;
      }

      const data: VersionResponse = await response.json();

      // If index_version is 0 or absent (older servers), assume version 1
      const indexVersion = data.index_version || 1;

      setServerConfig({
        shortUrlEnabled: data.config.short_url_enabled,
        shortUrlBase: data.config.short_url_base,
        shortUrlExpirationHours: data.config.short_url_expiration_hours,
        indexVersion,
      });
    } catch (err) {
      console.warn("Failed to fetch server config:", err);
      setServerConfig(null);
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

  return serverConfig;
}
