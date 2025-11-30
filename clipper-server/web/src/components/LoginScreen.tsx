import { useState, FormEvent, useCallback } from "react";
import { useI18n } from "@unwritten-codes/clipper-ui";
import "./LoginScreen.css";

interface LoginScreenProps {
  onLogin: (token: string) => Promise<void>;
  error?: string;
}

export function LoginScreen({ onLogin, error }: LoginScreenProps) {
  const { t } = useI18n();
  const [token, setToken] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      if (!token.trim() || isLoading) return;

      setIsLoading(true);
      try {
        await onLogin(token.trim());
      } finally {
        setIsLoading(false);
      }
    },
    [token, isLoading, onLogin]
  );

  return (
    <div className="login-screen">
      <div className="login-card">
        <div className="login-icon">
          <svg
            viewBox="0 0 512 512"
            xmlns="http://www.w3.org/2000/svg"
            width="64"
            height="64"
          >
            <defs>
              <linearGradient id="boardGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor="#6366F1" />
                <stop offset="100%" stopColor="#8B5CF6" />
              </linearGradient>
              <linearGradient id="clipGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="#F1F5F9" />
                <stop offset="30%" stopColor="#CBD5E1" />
                <stop offset="70%" stopColor="#94A3B8" />
                <stop offset="100%" stopColor="#64748B" />
              </linearGradient>
              <linearGradient id="paperGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="#FFFFFF" />
                <stop offset="100%" stopColor="#F8FAFC" />
              </linearGradient>
            </defs>
            <g>
              <rect
                x="96"
                y="80"
                width="320"
                height="400"
                rx="32"
                ry="32"
                fill="url(#boardGrad)"
              />
              <rect
                x="128"
                y="140"
                width="256"
                height="310"
                rx="16"
                ry="16"
                fill="url(#paperGrad)"
              />
              <g fill="#C7D2FE">
                <rect x="160" y="180" width="180" height="14" rx="7" />
                <rect x="160" y="215" width="140" height="14" rx="7" />
                <rect x="160" y="250" width="192" height="14" rx="7" />
                <rect x="160" y="285" width="120" height="14" rx="7" />
                <rect x="160" y="320" width="160" height="14" rx="7" />
              </g>
              <g>
                <rect
                  x="186"
                  y="48"
                  width="140"
                  height="72"
                  rx="12"
                  ry="12"
                  fill="url(#clipGrad)"
                />
                <g stroke="#64748B" strokeWidth="3" strokeLinecap="round">
                  <line x1="206" y1="60" x2="206" y2="108" />
                  <line x1="222" y1="60" x2="222" y2="108" />
                  <line x1="290" y1="60" x2="290" y2="108" />
                  <line x1="306" y1="60" x2="306" y2="108" />
                </g>
                <path
                  d="M194 120 C194 136, 210 148, 230 148"
                  stroke="#94A3B8"
                  strokeWidth="10"
                  strokeLinecap="round"
                  fill="none"
                />
                <path
                  d="M318 120 C318 136, 302 148, 282 148"
                  stroke="#94A3B8"
                  strokeWidth="10"
                  strokeLinecap="round"
                  fill="none"
                />
              </g>
            </g>
            {/* Lock icon overlay */}
            <g>
              <circle cx="368" cy="400" r="44" fill="#EF4444" />
              <rect x="348" y="390" width="40" height="32" rx="4" fill="#FFFFFF" />
              <path
                d="M354 390 L354 380 C354 370 362 362 368 362 C374 362 382 370 382 380 L382 390"
                stroke="#FFFFFF"
                strokeWidth="6"
                fill="none"
              />
            </g>
          </svg>
        </div>
        <h1 className="login-title">{t("auth.title")}</h1>
        <p className="login-description">{t("auth.description")}</p>

        <form onSubmit={handleSubmit} className="login-form">
          <div className="form-group">
            <label htmlFor="token" className="form-label">
              {t("auth.tokenLabel")}
            </label>
            <input
              type="password"
              id="token"
              className="form-input"
              placeholder={t("auth.tokenPlaceholder")}
              value={token}
              onChange={(e) => setToken(e.target.value)}
              disabled={isLoading}
              autoFocus
            />
          </div>

          {error && <div className="login-error">{error}</div>}

          <button
            type="submit"
            className="login-button"
            disabled={!token.trim() || isLoading}
          >
            {isLoading ? t("auth.loggingIn") : t("auth.login")}
          </button>
        </form>
      </div>
    </div>
  );
}
