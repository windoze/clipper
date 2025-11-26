import { Clip, isFavorite, FAVORITE_TAG } from "../types";
import { invoke } from "@tauri-apps/api/core";

interface ClipEntryProps {
  clip: Clip;
  onToggleFavorite: (clip: Clip) => void;
}

export function ClipEntry({ clip, onToggleFavorite }: ClipEntryProps) {
  const favorite = isFavorite(clip);
  const displayTags = clip.tags.filter((t) => t !== FAVORITE_TAG);

  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  const handleCopy = async () => {
    try {
      // Use custom command that copies without creating a new server clip
      await invoke("copy_to_clipboard", { content: clip.content });
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  };

  const truncateContent = (content: string, maxLength: number = 200): string => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + "...";
  };

  return (
    <div className="clip-entry" onClick={handleCopy}>
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
      <div className="clip-content">{truncateContent(clip.content)}</div>
      {displayTags.length > 0 && (
        <div className="clip-tags">
          {displayTags.map((tag) => (
            <span key={tag} className="tag">
              {tag}
            </span>
          ))}
        </div>
      )}
      {clip.file_attachment && (
        <div className="clip-attachment">
          <span className="attachment-icon">📎</span>
          <span className="attachment-name">{clip.file_attachment}</span>
        </div>
      )}
    </div>
  );
}
