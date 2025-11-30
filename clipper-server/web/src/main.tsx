import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import {
  I18nProvider,
  ToastProvider,
  ApiProvider,
  createRestApiClient,
} from "@unwritten-codes/clipper-ui";
import { CleanupConfigWrapper } from "./components/CleanupConfigWrapper";
import App from "./App";
import "./App.css";

// Create the REST API client for the web UI
const api = createRestApiClient("");

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <I18nProvider storageKey="clipper-web-language">
      <ApiProvider value={api}>
        <ToastProvider>
          <CleanupConfigWrapper>
            <App />
          </CleanupConfigWrapper>
        </ToastProvider>
      </ApiProvider>
    </I18nProvider>
  </StrictMode>
);
