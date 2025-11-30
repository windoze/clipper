import { ReactNode } from "react";
import { CleanupConfigProvider } from "@unwritten-codes/clipper-ui";
import { useFetchCleanupConfig } from "../hooks/useCleanupConfig";

interface CleanupConfigWrapperProps {
  children: ReactNode;
}

/**
 * Wrapper component that fetches the cleanup config from the server
 * and provides it to children via context.
 */
export function CleanupConfigWrapper({ children }: CleanupConfigWrapperProps) {
  const cleanupConfig = useFetchCleanupConfig();

  return (
    <CleanupConfigProvider value={cleanupConfig}>
      {children}
    </CleanupConfigProvider>
  );
}
