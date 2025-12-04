import { useState, useEffect } from "react";
import type { ServerConfig } from "@unwritten-codes/clipper-ui";

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
    short_url_enabled: boolean;
    short_url_base?: string;
  };
}

/**
 * Hook to fetch server configuration from the server.
 * Uses the /version endpoint to get the server config.
 */
export function useFetchServerConfig(): ServerConfig | null {
  const [serverConfig, setServerConfig] = useState<ServerConfig | null>(null);

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        // Fetch version info from the server (same origin)
        const response = await fetch("/version");
        if (!response.ok) {
          console.warn("Failed to fetch server version:", response.status);
          setServerConfig(null);
          return;
        }

        const data: VersionResponse = await response.json();

        setServerConfig({
          shortUrlEnabled: data.config.short_url_enabled,
          shortUrlBase: data.config.short_url_base,
        });
      } catch (err) {
        console.warn("Failed to fetch server config:", err);
        setServerConfig(null);
      }
    };

    fetchConfig();
  }, []);

  return serverConfig;
}
