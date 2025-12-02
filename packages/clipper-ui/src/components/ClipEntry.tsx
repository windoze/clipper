import { useState, useEffect, useMemo } from "react";
import hljs from "highlight.js";
import { Clip, isFavorite, calculateAgeRatio } from "../types";
import { ImagePopup } from "./ImagePopup";
import { EditClipDialog } from "./EditClipDialog";
import { LanguageSelector, LanguageId, LANGUAGES } from "./LanguageSelector";
import { useI18n } from "../i18n";
import { useToast } from "./Toast";
import { useApi } from "../api";
import { useCleanupConfig } from "../hooks/useCleanupConfig";

interface ClipEntryProps {
  clip: Clip;
  onToggleFavorite: (clip: Clip) => void;
  onClipUpdated?: (updatedClip: Clip) => void;
  onClipDeleted?: (clipId: string) => void;
  onTagClick?: (tag: string) => void;
}

// Image file extensions
const IMAGE_EXTENSIONS = [
  ".png",
  ".jpg",
  ".jpeg",
  ".gif",
  ".webp",
  ".bmp",
  ".svg",
];

function isImageFile(filename: string): boolean {
  const lower = filename.toLowerCase();
  return IMAGE_EXTENSIONS.some((ext) => lower.endsWith(ext));
}

const MAX_CONTENT_LENGTH = 200;

// Minimum opacity to ensure readability (0.5 = 50%)
const MIN_OPACITY = 0.5;

// Try to detect language from content
function detectLanguage(content: string): LanguageId {
  // Try auto-detection with highlight.js
  const result = hljs.highlightAuto(content, LANGUAGES.map(l => l.id).filter(id => id !== "plaintext"));
  if (result.language && result.relevance > 5) {
    return result.language as LanguageId;
  }
  return "plaintext";
}

// Escape HTML for safe rendering
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// Truncate content
function truncateContent(content: string): string {
  if (content.length <= MAX_CONTENT_LENGTH) return content;
  return content.substring(0, MAX_CONTENT_LENGTH) + "...";
}

export function ClipEntry({
  clip,
  onToggleFavorite,
  onClipUpdated,
  onClipDeleted,
  onTagClick,
}: ClipEntryProps) {
  const { t } = useI18n();
  const { showToast } = useToast();
  const api = useApi();
  const cleanupConfig = useCleanupConfig();
  const favorite = isFavorite(clip);

  // Calculate age-based opacity for visual aging effect
  const ageStyle = useMemo(() => {
    const ageRatio = calculateAgeRatio(clip, cleanupConfig);
    if (ageRatio === null) {
      return undefined;
    }
    // Map ageRatio (0-1) to opacity (1.0 - MIN_OPACITY)
    // ageRatio 0 = full opacity (1.0), ageRatio 1 = minimum opacity (MIN_OPACITY)
    const opacity = 1 - (ageRatio * (1 - MIN_OPACITY));
    return {
      "--clip-age-opacity": opacity,
    } as React.CSSProperties;
  }, [clip, cleanupConfig]);
  // Show regular tags and $host: tags (with special styling)
  const displayTags = clip.tags.filter((tag) => !tag.startsWith("$") || tag.startsWith("$host:"));

  // Helper to check if a tag is a host tag
  const isHostTag = (tag: string) => tag.startsWith("$host:");

  // Get the display name for a host tag (remove the $host: prefix)
  const getHostTagDisplay = (tag: string) => tag.replace("$host:", "");
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [showPopup, setShowPopup] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [selectedLanguage, setSelectedLanguage] = useState<LanguageId>(() => detectLanguage(clip.content));

  const isImage = clip.file_attachment && isImageFile(clip.file_attachment);
  const isLongContent = clip.content.length > MAX_CONTENT_LENGTH;

  // Generate highlighted HTML
  const highlightedContent = useMemo(() => {
    if (selectedLanguage === "plaintext") {
      return null;
    }
    try {
      const result = hljs.highlight(clip.content, { language: selectedLanguage });
      return result.value;
    } catch {
      return null;
    }
  }, [clip.content, selectedLanguage]);

  // Truncate highlighted content (preserving HTML structure is complex, so we use plain truncation for preview)
  const displayContent = useMemo(() => {
    const content = isExpanded ? clip.content : truncateContent(clip.content);
    if (selectedLanguage === "plaintext" || !highlightedContent) {
      return { __html: escapeHtml(content) };
    }
    // For highlighted content, we need to re-highlight the truncated version
    if (!isExpanded && isLongContent) {
      try {
        const result = hljs.highlight(content.replace(/\.\.\.$/, ""), { language: selectedLanguage });
        return { __html: result.value + (isLongContent ? "..." : "") };
      } catch {
        return { __html: escapeHtml(content) };
      }
    }
    return { __html: highlightedContent };
  }, [clip.content, selectedLanguage, highlightedContent, isExpanded, isLongContent]);

  // Get file URL for image clips
  useEffect(() => {
    if (isImage) {
      // Use async version if available (for Tauri), otherwise use sync version
      if (api.getFileUrlAsync) {
        api.getFileUrlAsync(clip.id).then(setImageUrl);
      } else {
        setImageUrl(api.getFileUrl(clip.id));
      }
    }
  }, [clip.id, isImage, api]);

  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  const handleCopy = async () => {
    // Don't copy if this is an image clip
    if (isImage) return;

    try {
      await api.copyToClipboard(clip.content);
      showToast(t("toast.clipCopied"));
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
      showToast(t("toast.copyFailed"), "error");
    }
  };

  const handleImageClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowPopup(true);
  };

  const handleEditClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowEditDialog(true);
  };

  const handleClipSaved = (updatedClip: Clip) => {
    onClipUpdated?.(updatedClip);
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowDeleteConfirm(true);
  };

  const handleDeleteConfirm = async () => {
    setDeleting(true);
    try {
      await api.deleteClip(clip.id);
      setShowDeleteConfirm(false);
      onClipDeleted?.(clip.id);
      showToast(t("toast.clipDeleted"));
    } catch (err) {
      console.error("Failed to delete clip:", err);
    } finally {
      setDeleting(false);
    }
  };

  const handleDeleteCancel = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowDeleteConfirm(false);
  };

  const handleDownload = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!clip.file_attachment) return;

    const downloadFilename = clip.original_filename || clip.file_attachment;

    try {
      await api.downloadFile(clip.id, downloadFilename);
    } catch (err) {
      // User cancelled or error occurred
      if (err !== "Save cancelled") {
        console.error("Failed to download file:", err);
      }
    }
  };

  const handleExpandToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(!isExpanded);
  };

  // Build class names including aging class if applicable
  const clipEntryClassName = [
    "clip-entry",
    isImage ? "clip-entry-image" : "",
    ageStyle ? "clip-entry-aging" : "",
  ].filter(Boolean).join(" ");

  return (
    <>
      <div
        className={clipEntryClassName}
        style={ageStyle}
        onClick={handleCopy}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <div className="clip-header">
          <span className="clip-date">{formatDate(clip.created_at)}</span>
          <div className="clip-actions">
            <button
              className="edit-button"
              onClick={handleEditClick}
              title={t("clip.edit")}
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
              </svg>
            </button>
            <button
              className={`favorite-button ${favorite ? "active" : ""}`}
              onClick={(e) => {
                e.stopPropagation();
                onToggleFavorite(clip);
              }}
              title={
                favorite ? t("clip.favorite.remove") : t("clip.favorite.add")
              }
            >
              {favorite ? "★" : "☆"}
            </button>
          </div>
        </div>

        {isImage && imageUrl ? (
          <div className="clip-image-container" onClick={handleImageClick}>
            <img
              src={imageUrl}
              alt={clip.file_attachment || "Image"}
              className="clip-image-thumbnail"
            />
            <div className="clip-image-overlay">
              <span className="clip-image-zoom-icon">🔍</span>
            </div>
          </div>
        ) : (
          <div className="clip-content-wrapper">
            <LanguageSelector
              value={selectedLanguage}
              onChange={setSelectedLanguage}
              visible={isHovered}
            />
            <div
              className={`clip-content ${isExpanded ? "expanded" : ""} ${selectedLanguage !== "plaintext" ? "hljs" : ""}`}
              dangerouslySetInnerHTML={displayContent}
            />
            {isLongContent && (
              <button
                className="expand-button"
                onClick={handleExpandToggle}
                title={isExpanded ? t("clip.collapse") : t("clip.expand")}
              >
                {isExpanded ? t("clip.collapse") : t("clip.expand")}
              </button>
            )}
          </div>
        )}

        {displayTags.length > 0 && (
          <div className="clip-tags">
            {displayTags.map((tag) => (
              <button
                key={tag}
                className={`tag tag-clickable ${isHostTag(tag) ? "tag-host" : ""}`}
                onClick={(e) => {
                  e.stopPropagation();
                  onTagClick?.(tag);
                }}
                title={t("filter.clickToFilter")}
              >
                {isHostTag(tag) && (
                  <svg
                    className="tag-host-icon"
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
                    <line x1="8" y1="21" x2="16" y2="21"></line>
                    <line x1="12" y1="17" x2="12" y2="21"></line>
                  </svg>
                )}
                {isHostTag(tag) ? getHostTagDisplay(tag) : tag}
              </button>
            ))}
          </div>
        )}

        {clip.file_attachment && !isImage && (
          <div className="clip-attachment">
            <span className="attachment-icon">📎</span>
            <button
              className="attachment-name-button"
              onClick={handleDownload}
              title={t("clip.download")}
            >
              {clip.original_filename || clip.file_attachment}
            </button>
          </div>
        )}

        {isImage && clip.file_attachment && (
          <div className="clip-attachment">
            <span className="attachment-icon">🖼️</span>
            <button
              className="attachment-name-button"
              onClick={handleDownload}
              title={t("clip.download")}
            >
              {clip.original_filename || clip.file_attachment}
            </button>
          </div>
        )}

        <button
          className="delete-button"
          onClick={handleDeleteClick}
          title={t("clip.delete")}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            <line x1="10" y1="11" x2="10" y2="17"></line>
            <line x1="14" y1="11" x2="14" y2="17"></line>
          </svg>
        </button>
      </div>

      {showPopup && imageUrl && clip.file_attachment && (
        <ImagePopup
          imageUrl={imageUrl}
          filename={clip.original_filename || clip.file_attachment}
          onClose={() => setShowPopup(false)}
        />
      )}

      <EditClipDialog
        clip={clip}
        isOpen={showEditDialog}
        onClose={() => setShowEditDialog(false)}
        onSave={handleClipSaved}
      />

      {showDeleteConfirm && (
        <div className="delete-confirm-backdrop" onClick={handleDeleteCancel}>
          <div
            className="delete-confirm-dialog"
            onClick={(e) => e.stopPropagation()}
          >
            <p>{t("clip.delete_confirm")}</p>
            <div className="delete-confirm-actions">
              <button
                className="delete-confirm-btn cancel"
                onClick={handleDeleteCancel}
                disabled={deleting}
              >
                {t("common.cancel")}
              </button>
              <button
                className="delete-confirm-btn confirm"
                onClick={handleDeleteConfirm}
                disabled={deleting}
              >
                {deleting ? t("common.deleting") : t("common.delete")}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
