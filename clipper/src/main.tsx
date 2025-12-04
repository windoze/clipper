import React from "react";
import ReactDOM from "react-dom/client";
import { I18nProvider, ApiProvider } from "@unwritten-codes/clipper-ui";
import { TauriToastWrapper } from "./components/TauriToastWrapper";
import { CleanupConfigWrapper } from "./components/CleanupConfigWrapper";
import { ServerConfigWrapper } from "./components/ServerConfigWrapper";
import { createTauriApiClient } from "./api/tauriClient";
import { tauriExtraTranslations } from "./i18n/translations";
import App from "./App";

// Detect platform and set data attribute on document element for platform-specific CSS
function detectPlatform(): "macos" | "windows" | "linux" {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  return "linux";
}
document.documentElement.dataset.platform = detectPlatform();

// Create the Tauri API client
const api = createTauriApiClient();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <I18nProvider
      storageKey="clipper-tauri-language"
      extraTranslations={tauriExtraTranslations}
    >
      <ApiProvider value={api}>
        <TauriToastWrapper>
          <CleanupConfigWrapper>
            <ServerConfigWrapper>
              <App />
            </ServerConfigWrapper>
          </CleanupConfigWrapper>
        </TauriToastWrapper>
      </ApiProvider>
    </I18nProvider>
  </React.StrictMode>
);
