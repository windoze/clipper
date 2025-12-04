import { ReactNode } from "react";
import { ServerConfigProvider } from "@unwritten-codes/clipper-ui";
import { useFetchServerConfig } from "../hooks/useServerConfig";

interface ServerConfigWrapperProps {
  children: ReactNode;
}

/**
 * Wrapper component that fetches the server config from the server
 * and provides it to children via context.
 */
export function ServerConfigWrapper({ children }: ServerConfigWrapperProps) {
  const serverConfig = useFetchServerConfig();

  return (
    <ServerConfigProvider value={serverConfig}>
      {children}
    </ServerConfigProvider>
  );
}
