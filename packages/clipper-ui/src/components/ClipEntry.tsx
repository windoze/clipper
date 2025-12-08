import { useState, useEffect, useMemo } from "react";
import hljs from "highlight.js";
import { Clip, Tag, isFavorite, calculateAgeRatio } from "../types";
import { ImagePopup } from "./ImagePopup";
import { EditClipDialog } from "./EditClipDialog";
import { ShareDialog } from "./ShareDialog";
import { LanguageSelector, LanguageId, LANGUAGES } from "./LanguageSelector";
import { DateTag } from "./DateTag";
import { Tooltip } from "./Tooltip";
import { useI18n } from "../i18n";
import { useToast } from "./Toast";
import { useApi } from "../api";
import { useCleanupConfig } from "../hooks/useCleanupConfig";
import { useServerConfig } from "../hooks/useServerConfig";

interface ClipEntryProps {
  clip: Clip;
  onToggleFavorite: (clip: Clip) => void;
  onClipUpdated?: (updatedClip: Clip) => void;
  onClipDeleted?: (clipId: string) => void;
  onTagClick?: (tag: string) => void;
  onSetStartDate?: (isoDate: string) => void;
  onSetEndDate?: (isoDate: string) => void;
  /** Function to search tags for autocomplete in edit dialog */
  onSearchTags?: (query: string) => Promise<Tag[]>;
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

const MAX_CONTENT_LINES = 6;

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

// Count number of lines in content
function countLines(content: string): number {
  return content.split("\n").length;
}

// Get content up to N lines
function getFirstNLines(content: string, n: number): string {
  const lines = content.split("\n");
  if (lines.length <= n) return content;
  return lines.slice(0, n).join("\n");
}

// Check if content exceeds max lines
function isLongContentByLines(content: string): boolean {
  return countLines(content) > MAX_CONTENT_LINES;
}

// Truncate content by lines
function truncateContent(content: string): string {
  if (!isLongContentByLines(content)) return content;
  return getFirstNLines(content, MAX_CONTENT_LINES) + "\n...";
}

// Truncate highlighted content by lines while ensuring at least one highlight is visible
// Returns truncated content with "..." prefix/suffix as needed
function truncateHighlightedContent(highlightedContent: string, plainContent: string): string {
  // If content is short enough (by lines), return as-is
  if (!isLongContentByLines(plainContent)) {
    return highlightedContent;
  }

  // Split into lines (preserving HTML tags within lines)
  const lines = highlightedContent.split("\n");

  // Find the line index containing the first <mark> tag
  let firstMarkLineIndex = -1;
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].includes("<mark>")) {
      firstMarkLineIndex = i;
      break;
    }
  }

  // If no highlights found, just truncate from the beginning
  if (firstMarkLineIndex === -1) {
    if (lines.length <= MAX_CONTENT_LINES) {
      return highlightedContent;
    }
    return lines.slice(0, MAX_CONTENT_LINES).join("\n") + "\n...";
  }

  // If first highlight is within the first MAX_CONTENT_LINES lines, show from start
  if (firstMarkLineIndex < MAX_CONTENT_LINES) {
    // First highlight is early enough, truncate from the end
    if (lines.length <= MAX_CONTENT_LINES) {
      return highlightedContent;
    }
    return lines.slice(0, MAX_CONTENT_LINES).join("\n") + "\n...";
  }

  // First highlight is beyond initial lines, need to start from before the highlight
  // Show 1 line of context before the highlight
  const CONTEXT_LINES_BEFORE = 1;
  const startLineIndex = Math.max(0, firstMarkLineIndex - CONTEXT_LINES_BEFORE);
  const endLineIndex = Math.min(lines.length, startLineIndex + MAX_CONTENT_LINES);

  const prefix = startLineIndex > 0 ? "...\n" : "";
  const suffix = endLineIndex < lines.length ? "\n..." : "";

  return prefix + lines.slice(startLineIndex, endLineIndex).join("\n") + suffix;
}

// Escape HTML but preserve <mark> tags for search highlighting
function escapeHtmlPreserveMark(text: string): string {
  // Split by <mark> and </mark> tags, escape the parts, then rejoin
  const parts = text.split(/(<\/?mark>)/g);
  return parts.map(part => {
    if (part === "<mark>" || part === "</mark>") {
      return part;
    }
    return escapeHtml(part);
  }).join("");
}

export function ClipEntry({
  clip,
  onToggleFavorite,
  onClipUpdated,
  onClipDeleted,
  onTagClick,
  onSetStartDate,
  onSetEndDate,
  onSearchTags,
}: ClipEntryProps) {
  const { t } = useI18n();
  const { showToast } = useToast();
  const api = useApi();
  const cleanupConfig = useCleanupConfig();
  const serverConfig = useServerConfig();
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
  const [showShareDialog, setShowShareDialog] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [selectedLanguage, setSelectedLanguage] = useState<LanguageId>(() => detectLanguage(clip.content));

  const isImage = clip.file_attachment && isImageFile(clip.file_attachment);
  const isLongContent = isLongContentByLines(clip.content);

  // Check if we have search highlighting from the server
  const hasSearchHighlight = !!clip.highlighted_content;

  // Generate syntax highlighted HTML (only when no search highlight)
  const syntaxHighlightedContent = useMemo(() => {
    // Don't apply syntax highlighting if we have search highlights
    if (hasSearchHighlight || selectedLanguage === "plaintext") {
      return null;
    }
    try {
      const result = hljs.highlight(clip.content, { language: selectedLanguage });
      return result.value;
    } catch {
      return null;
    }
  }, [clip.content, selectedLanguage, hasSearchHighlight]);

  // Prepare display content with proper handling of search highlights and syntax highlighting
  const displayContent = useMemo(() => {
    // Case 1: We have search-highlighted content from the server
    if (hasSearchHighlight && clip.highlighted_content) {
      if (isExpanded) {
        // Show full highlighted content, escape HTML but preserve <mark> tags
        return { __html: escapeHtmlPreserveMark(clip.highlighted_content) };
      } else {
        // Truncate with smart positioning to show at least one highlight
        const truncated = truncateHighlightedContent(clip.highlighted_content, clip.content);
        return { __html: escapeHtmlPreserveMark(truncated) };
      }
    }

    // Case 2: No search highlight, use syntax highlighting or plain text
    const content = isExpanded ? clip.content : truncateContent(clip.content);

    if (selectedLanguage === "plaintext" || !syntaxHighlightedContent) {
      return { __html: escapeHtml(content) };
    }

    // For syntax highlighted content, we need to re-highlight the truncated version
    if (!isExpanded && isLongContent) {
      try {
        const result = hljs.highlight(content.replace(/\.\.\.$/, ""), { language: selectedLanguage });
        return { __html: result.value + (isLongContent ? "..." : "") };
      } catch {
        return { __html: escapeHtml(content) };
      }
    }
    return { __html: syntaxHighlightedContent };
  }, [clip.content, clip.highlighted_content, hasSearchHighlight, selectedLanguage, syntaxHighlightedContent, isExpanded, isLongContent]);

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

  const handleCopyClick = async (e: React.MouseEvent) => {
    e.stopPropagation();
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

  const handleShareClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!serverConfig?.shortUrlEnabled) return;
    setShowShareDialog(true);
  };

  // Handle click on the clip entry itself - toggle expand/collapse for long content
  const handleEntryClick = () => {
    // Only toggle if content is long (not for images)
    if (isLongContent && !isImage) {
      setIsExpanded(!isExpanded);
    }
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
        onClick={handleEntryClick}
      >
        <div className="clip-header">
          <div className="clip-header-left">
            <DateTag
              dateStr={clip.created_at}
              onSetStartDate={onSetStartDate}
              onSetEndDate={onSetEndDate}
            />
            {!isImage && (
              <LanguageSelector
                value={selectedLanguage}
                onChange={setSelectedLanguage}
                visible={true}
              />
            )}
            {clip.additional_notes && (
              <Tooltip content={clip.additional_notes} position="bottom" maxWidth={450}>
                <button
                  className="notes-indicator"
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowEditDialog(true);
                  }}
                  title={t("tooltip.viewNotes")}
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
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                    <polyline points="14 2 14 8 20 8"></polyline>
                    <line x1="16" y1="13" x2="8" y2="13"></line>
                    <line x1="16" y1="17" x2="8" y2="17"></line>
                    <polyline points="10 9 9 9 8 9"></polyline>
                  </svg>
                </button>
              </Tooltip>
            )}
          </div>
          <div className="clip-actions">
            {!isImage && (
              <button
                className="copy-button"
                onClick={handleCopyClick}
                title={t("tooltip.copy")}
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
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            )}
            {serverConfig?.shortUrlEnabled && (
              <>
                <span className="clip-action-separator">|</span>
                <button
                  className="share-button"
                  onClick={handleShareClick}
                  title={t("clip.share")}
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
                    <circle cx="18" cy="5" r="3"></circle>
                    <circle cx="6" cy="12" r="3"></circle>
                    <circle cx="18" cy="19" r="3"></circle>
                    <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line>
                    <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line>
                  </svg>
                </button>
              </>
            )}
            <span className="clip-action-separator">|</span>
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
        onSearchTags={onSearchTags}
      />

      <ShareDialog
        clipId={clip.id}
        clipContent={clip.content}
        isOpen={showShareDialog}
        onClose={() => setShowShareDialog(false)}
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
