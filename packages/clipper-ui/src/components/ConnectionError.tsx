import { useState } from "react";
import { useI18n } from "../i18n";

interface ConnectionErrorProps {
  error: string;
  onRetry: () => void;
  onOpenSettings?: () => void;
  showBundledServerReason?: boolean;
}

export function ConnectionError({
  error,
  onRetry,
  onOpenSettings,
  showBundledServerReason = false,
}: ConnectionErrorProps) {
  const { t } = useI18n();
  const [isRetrying, setIsRetrying] = useState(false);

  const handleRetry = async () => {
    setIsRetrying(true);
    try {
      await onRetry();
    } finally {
      // Add a small delay to show the loading state
      setTimeout(() => setIsRetrying(false), 500);
    }
  };

  return (
    <div className="connection-error">
      <div className="connection-error-content">
        {/* Icon */}
        <div className="connection-error-icon">
          <svg
            width="64"
            height="64"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            {/* Cloud with X */}
            <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z" />
            <line x1="14" y1="11" x2="10" y2="15" />
            <line x1="10" y1="11" x2="14" y2="15" />
          </svg>
        </div>

        {/* Title */}
        <h2 className="connection-error-title">{t("connectionError.title")}</h2>

        {/* Description */}
        <p className="connection-error-description">
          {t("connectionError.description")}
        </p>

        {/* Reasons list */}
        <ul className="connection-error-reasons">
          <li>
            <span className="reason-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
              </svg>
            </span>
            {t("connectionError.reason.serverDown")}
          </li>
          <li>
            <span className="reason-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
              </svg>
            </span>
            {t("connectionError.reason.networkIssue")}
          </li>
          <li>
            <span className="reason-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
              </svg>
            </span>
            {t("connectionError.reason.wrongUrl")}
          </li>
          {showBundledServerReason && (
            <li>
              <span className="reason-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
                </svg>
              </span>
              {t("connectionError.reason.bundledServer")}
            </li>
          )}
        </ul>

        {/* Error details (collapsed by default) */}
        <details className="connection-error-details">
          <summary>{t("common.error")}</summary>
          <code>{error}</code>
        </details>

        {/* Action buttons */}
        <div className="connection-error-actions">
          <button
            className="connection-error-btn primary"
            onClick={handleRetry}
            disabled={isRetrying}
          >
            {isRetrying ? (
              <>
                <span className="loading-spinner small"></span>
                {t("connectionError.checkingServer")}
              </>
            ) : (
              <>
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="23 4 23 10 17 10" />
                  <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                </svg>
                {t("connectionError.retry")}
              </>
            )}
          </button>
          {onOpenSettings && (
            <button
              className="connection-error-btn secondary"
              onClick={onOpenSettings}
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
              {t("connectionError.openSettings")}
            </button>
          )}
        </div>

        {/* Hint */}
        <p className="connection-error-hint">{t("connectionError.hint")}</p>
      </div>
    </div>
  );
}
