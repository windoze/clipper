import { useState, useEffect } from "react";
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
 * Uses the /version endpoint to get the cleanup config.
 */
export function useFetchCleanupConfig(): CleanupConfig | null {
  const [cleanupConfig, setCleanupConfig] = useState<CleanupConfig | null>(null);

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        // Fetch version info from the server (same origin)
        const response = await fetch("/version");
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

    fetchConfig();
  }, []);

  return cleanupConfig;
}
