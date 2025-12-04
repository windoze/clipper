import { createContext, useContext } from "react";
import { ServerConfig } from "../types";

/**
 * Context for providing server configuration to components.
 * When null, server-specific features like sharing are disabled.
 */
const ServerConfigContext = createContext<ServerConfig | null>(null);

export const ServerConfigProvider = ServerConfigContext.Provider;

/**
 * Hook to access the server configuration.
 * Returns null if no server config is provided.
 */
export function useServerConfig(): ServerConfig | null {
  return useContext(ServerConfigContext);
}
