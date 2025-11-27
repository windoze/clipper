import { useState, useEffect } from "react";
import { Clip, isFavorite } from "../types";
import { invoke } from "@tauri-apps/api/core";
import { ImagePopup } from "./ImagePopup";
import { EditClipDialog } from "./EditClipDialog";
import { useI18n } from "../i18n";
import { useToast } from "./Toast";

interface ClipEntryProps {
  clip: Clip;
  onToggleFavorite: (clip: Clip) => void;
  onClipUpdated?: (updatedClip: Clip) => void;
  onClipDeleted?: (clipId: string) => void;
}

// Image file extensions
const IMAGE_EXTENSIONS = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".svg"];

function isImageFile(filename: string): boolean {
  const lower = filename.toLowerCase();
  return IMAGE_EXTENSIONS.some((ext) => lower.endsWith(ext));
}

const MAX_CONTENT_LENGTH = 200;

export function ClipEntry({ clip, onToggleFavorite, onClipUpdated, onClipDeleted }: ClipEntryProps) {
  const { t } = useI18n();
  const { showToast } = useToast();
  const favorite = isFavorite(clip);
  const displayTags = clip.tags.filter((t) => !t.startsWith("$"));
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [showPopup, setShowPopup] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);

  const isImage = clip.file_attachment && isImageFile(clip.file_attachment);
  const isLongContent = clip.content.length > MAX_CONTENT_LENGTH;

  // Get file URL for image clips
  useEffect(() => {
    if (isImage) {
      invoke<string>("get_file_url", { clipId: clip.id }).then(setImageUrl);
    }
  }, [clip.id, isImage]);

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
      // Use custom command that copies without creating a new server clip
      await invoke("copy_to_clipboard", { content: clip.content });
      showToast(t("toast.clipCopied"));
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
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
      await invoke("delete_clip", { id: clip.id });
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

    // Use original_filename if available, otherwise fall back to file_attachment
    const downloadFilename = clip.original_filename || clip.file_attachment;

    try {
      await invoke("download_file", {
        clipId: clip.id,
        filename: downloadFilename,
      });
    } catch (err) {
      // User cancelled or error occurred
      if (err !== "Save cancelled") {
        console.error("Failed to download file:", err);
      }
    }
  };

  const truncateContent = (content: string): string => {
    if (content.length <= MAX_CONTENT_LENGTH) return content;
    return content.substring(0, MAX_CONTENT_LENGTH) + "...";
  };

  const handleExpandToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(!isExpanded);
  };

  return (
    <>
      <div
        className={`clip-entry ${isImage ? "clip-entry-image" : ""}`}
        onClick={handleCopy}
      >
        <div className="clip-header">
          <span className="clip-date">{formatDate(clip.created_at)}</span>
          <div className="clip-actions">
            <button
              className="edit-button"
              onClick={handleEditClick}
              title={t("clip.edit")}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
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
              title={favorite ? t("clip.favorite.remove") : t("clip.favorite.add")}
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
          <div className={`clip-content ${isExpanded ? "expanded" : ""}`}>
            {isExpanded ? clip.content : truncateContent(clip.content)}
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
              <span key={tag} className="tag">
                {tag}
              </span>
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
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
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
          <div className="delete-confirm-dialog" onClick={(e) => e.stopPropagation()}>
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
