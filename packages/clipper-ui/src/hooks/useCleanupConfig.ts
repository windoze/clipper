import { createContext, useContext } from "react";
import { CleanupConfig } from "../types";

/**
 * Context for providing cleanup configuration to components.
 * When null, cleanup aging visual effects are disabled.
 */
const CleanupConfigContext = createContext<CleanupConfig | null>(null);

export const CleanupConfigProvider = CleanupConfigContext.Provider;

/**
 * Hook to access the cleanup configuration.
 * Returns null if no cleanup config is provided (aging effects disabled).
 */
export function useCleanupConfig(): CleanupConfig | null {
  return useContext(CleanupConfigContext);
}
