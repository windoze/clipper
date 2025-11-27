import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Clip } from "../types";

interface EditClipDialogProps {
  clip: Clip;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedClip: Clip) => void;
}

export function EditClipDialog({ clip, isOpen, onClose, onSave }: EditClipDialogProps) {
  // Filter out system tags (starting with $) for editing
  const userTags = clip.tags.filter((t) => !t.startsWith("$"));
  const systemTags = clip.tags.filter((t) => t.startsWith("$"));

  const [tags, setTags] = useState<string[]>(userTags);
  const [tagInput, setTagInput] = useState("");
  const [notes, setNotes] = useState(clip.additional_notes || "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const tagInputRef = useRef<HTMLInputElement>(null);

  // Reset state when clip changes or dialog opens
  useEffect(() => {
    if (isOpen) {
      const userTags = clip.tags.filter((t) => !t.startsWith("$"));
      setTags(userTags);
      setNotes(clip.additional_notes || "");
      setTagInput("");
      setError(null);
    }
  }, [isOpen, clip]);

  const handleAddTag = () => {
    const trimmedTag = tagInput.trim();
    if (trimmedTag && !tags.includes(trimmedTag) && !trimmedTag.startsWith("$")) {
      setTags([...tags, trimmedTag]);
      setTagInput("");
      tagInputRef.current?.focus();
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter((t) => t !== tagToRemove));
  };

  const handleTagInputKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddTag();
    } else if (e.key === "Backspace" && tagInput === "" && tags.length > 0) {
      // Remove last tag when backspace is pressed on empty input
      setTags(tags.slice(0, -1));
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      // Combine user tags with system tags
      const allTags = [...tags, ...systemTags];
      const updatedClip = await invoke<Clip>("update_clip", {
        id: clip.id,
        tags: allTags,
        additionalNotes: notes || null,
      });
      onSave(updatedClip);
      onClose();
    } catch (e) {
      setError(`Failed to save: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="edit-clip-backdrop" onClick={handleCancel}>
      <div className="edit-clip-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="edit-clip-header">
          <h2>Edit Clip</h2>
          <button className="edit-clip-close" onClick={handleCancel}>
            &times;
          </button>
        </div>

        <div className="edit-clip-content">
          {error && <div className="edit-clip-error">{error}</div>}

          <div className="edit-clip-field">
            <label>Tags</label>
            <div className="tag-editor">
              <div className="tag-list">
                {tags.map((tag) => (
                  <span key={tag} className="tag editable">
                    {tag}
                    <button
                      type="button"
                      className="tag-remove"
                      onClick={() => handleRemoveTag(tag)}
                    >
                      &times;
                    </button>
                  </span>
                ))}
                <input
                  ref={tagInputRef}
                  type="text"
                  className="tag-input"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  onKeyDown={handleTagInputKeyDown}
                  placeholder={tags.length === 0 ? "Add tags..." : ""}
                />
              </div>
            </div>
            <p className="edit-clip-hint">
              Press Enter to add a tag, Backspace to remove the last one.
            </p>
          </div>

          <div className="edit-clip-field">
            <label htmlFor="notes">Notes</label>
            <textarea
              id="notes"
              className="notes-input"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="Add notes about this clip..."
              rows={4}
            />
          </div>
        </div>

        <div className="edit-clip-footer">
          <button className="edit-clip-btn secondary" onClick={handleCancel}>
            Cancel
          </button>
          <button
            className="edit-clip-btn primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
