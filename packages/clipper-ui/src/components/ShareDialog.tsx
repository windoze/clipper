import { useState, useEffect, useRef } from "react";
import { useI18n } from "../i18n";
import { useToast } from "./Toast";
import { useApi } from "../api";

interface ShareDialogProps {
  clipId: string;
  isOpen: boolean;
  onClose: () => void;
}

export function ShareDialog({ clipId, isOpen, onClose }: ShareDialogProps) {
  const { t } = useI18n();
  const { showToast } = useToast();
  const api = useApi();
  const inputRef = useRef<HTMLInputElement>(null);

  const [shareUrl, setShareUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  // Generate short URL when dialog opens
  useEffect(() => {
    if (isOpen && clipId) {
      setShareUrl(null);
      setError(null);
      setCopied(false);
      generateShareUrl();
    }
  }, [isOpen, clipId]);

  // Handle ESC key to close dialog
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  // Select input text when URL is ready
  useEffect(() => {
    if (shareUrl && inputRef.current) {
      inputRef.current.select();
    }
  }, [shareUrl]);

  const generateShareUrl = async () => {
    if (!api.shareClip) {
      setError(t("share.notAvailable"));
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const url = await api.shareClip(clipId);
      setShareUrl(url);
    } catch (err) {
      console.error("Failed to generate share URL:", err);
      setError(t("share.error"));
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = async () => {
    if (!shareUrl) return;

    try {
      await api.copyToClipboard(shareUrl);
      setCopied(true);
      showToast(t("toast.clipCopied"));
      // Reset copied state after 2 seconds
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
      showToast(t("toast.copyFailed"), "error");
    }
  };

  const handleInputClick = () => {
    inputRef.current?.select();
  };

  if (!isOpen) return null;

  return (
    <div className="share-dialog-backdrop" onClick={onClose}>
      <div className="share-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="share-dialog-header">
          <h2>{t("share.title")}</h2>
          <button className="share-dialog-close" onClick={onClose}>
            &times;
          </button>
        </div>

        <div className="share-dialog-content">
          {error && <div className="share-dialog-error">{error}</div>}

          {loading && (
            <div className="share-dialog-loading">
              <div className="share-dialog-spinner" />
              <span>{t("share.generating")}</span>
            </div>
          )}

          {shareUrl && !loading && (
            <div className="share-dialog-url-container">
              <input
                ref={inputRef}
                type="text"
                className="share-dialog-url-input"
                value={shareUrl}
                readOnly
                onClick={handleInputClick}
              />
              <button
                className={`share-dialog-copy-btn ${copied ? "copied" : ""}`}
                onClick={handleCopy}
              >
                {copied ? (
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
                    <polyline points="20 6 9 17 4 12"></polyline>
                  </svg>
                ) : (
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
                    <rect
                      x="9"
                      y="9"
                      width="13"
                      height="13"
                      rx="2"
                      ry="2"
                    ></rect>
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                  </svg>
                )}
                {copied ? t("share.copied") : t("share.copy")}
              </button>
            </div>
          )}

          <p className="share-dialog-hint">{t("share.hint")}</p>
        </div>

        <div className="share-dialog-footer">
          <button className="share-dialog-btn secondary" onClick={onClose}>
            {t("common.close")}
          </button>
        </div>
      </div>
    </div>
  );
}
