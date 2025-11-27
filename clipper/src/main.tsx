import React from "react";
import ReactDOM from "react-dom/client";
import { I18nProvider } from "./i18n/I18nProvider";
import { ToastProvider } from "./components/Toast";
import App from "./App";

// Detect platform and set data attribute on document element for platform-specific CSS
function detectPlatform(): "macos" | "windows" | "linux" {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  return "linux";
}
document.documentElement.dataset.platform = detectPlatform();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <I18nProvider>
      <ToastProvider>
        <App />
      </ToastProvider>
    </I18nProvider>
  </React.StrictMode>,
);
