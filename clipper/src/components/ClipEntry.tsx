import { useState, useEffect } from "react";
import { Clip, isFavorite, FAVORITE_TAG } from "../types";
import { invoke } from "@tauri-apps/api/core";
import { ImagePopup } from "./ImagePopup";

interface ClipEntryProps {
  clip: Clip;
  onToggleFavorite: (clip: Clip) => void;
}

// Image file extensions
const IMAGE_EXTENSIONS = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".svg"];

function isImageFile(filename: string): boolean {
  const lower = filename.toLowerCase();
  return IMAGE_EXTENSIONS.some((ext) => lower.endsWith(ext));
}

export function ClipEntry({ clip, onToggleFavorite }: ClipEntryProps) {
  const favorite = isFavorite(clip);
  const displayTags = clip.tags.filter((t) => !t.startsWith("$"));
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [showPopup, setShowPopup] = useState(false);

  const isImage = clip.file_attachment && isImageFile(clip.file_attachment);

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
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  };

  const handleImageClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowPopup(true);
  };

  const handleDownload = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!clip.file_attachment) return;

    try {
      await invoke("download_file", {
        clipId: clip.id,
        filename: clip.file_attachment,
      });
    } catch (err) {
      // User cancelled or error occurred
      if (err !== "Save cancelled") {
        console.error("Failed to download file:", err);
      }
    }
  };

  const truncateContent = (content: string, maxLength: number = 200): string => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + "...";
  };

  return (
    <>
      <div
        className={`clip-entry ${isImage ? "clip-entry-image" : ""}`}
        onClick={handleCopy}
      >
        <div className="clip-header">
          <span className="clip-date">{formatDate(clip.created_at)}</span>
          <button
            className={`favorite-button ${favorite ? "active" : ""}`}
            onClick={(e) => {
              e.stopPropagation();
              onToggleFavorite(clip);
            }}
            title={favorite ? "Remove from favorites" : "Add to favorites"}
          >
            {favorite ? "★" : "☆"}
          </button>
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
          <div className="clip-content">{truncateContent(clip.content)}</div>
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
              title="Click to download"
            >
              {clip.file_attachment}
            </button>
          </div>
        )}

        {isImage && clip.file_attachment && (
          <div className="clip-attachment">
            <span className="attachment-icon">🖼️</span>
            <button
              className="attachment-name-button"
              onClick={handleDownload}
              title="Click to download"
            >
              {clip.file_attachment}
            </button>
          </div>
        )}
      </div>

      {showPopup && imageUrl && clip.file_attachment && (
        <ImagePopup
          imageUrl={imageUrl}
          filename={clip.file_attachment}
          onClose={() => setShowPopup(false)}
        />
      )}
    </>
  );
}
