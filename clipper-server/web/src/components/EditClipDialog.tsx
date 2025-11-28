import { useState, useEffect, useRef } from "react";
import { Clip } from "../types";
import { useI18n } from "../i18n";
import { useToast } from "./Toast";
import * as api from "../api/client";

interface EditClipDialogProps {
  clip: Clip;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedClip: Clip) => void;
}

export function EditClipDialog({
  clip,
  isOpen,
  onClose,
  onSave,
}: EditClipDialogProps) {
  const { t } = useI18n();
  const { showToast } = useToast();
  // Filter out system tags (starting with $) for editing
  const userTags = clip.tags.filter((tag) => !tag.startsWith("$"));
  const systemTags = clip.tags.filter((tag) => tag.startsWith("$"));

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
      const updatedClip = await api.updateClip(
        clip.id,
        allTags,
        notes || null
      );
      onSave(updatedClip);
      onClose();
      showToast(t("toast.clipSaved"));
    } catch (e) {
      setError(t("editClip.saveError", { error: String(e) }));
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
          <h2>{t("editClip.title")}</h2>
          <button className="edit-clip-close" onClick={handleCancel}>
            &times;
          </button>
        </div>

        <div className="edit-clip-content">
          {error && <div className="edit-clip-error">{error}</div>}

          <div className="edit-clip-field">
            <label>{t("editClip.tags")}</label>
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
                  placeholder={
                    tags.length === 0 ? t("editClip.tags.placeholder") : ""
                  }
                />
              </div>
            </div>
            <p className="edit-clip-hint">{t("editClip.tags.hint")}</p>
          </div>

          <div className="edit-clip-field">
            <label htmlFor="notes">{t("editClip.notes")}</label>
            <textarea
              id="notes"
              className="notes-input"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder={t("editClip.notes.placeholder")}
              rows={4}
            />
          </div>
        </div>

        <div className="edit-clip-footer">
          <button className="edit-clip-btn secondary" onClick={handleCancel}>
            {t("common.cancel")}
          </button>
          <button
            className="edit-clip-btn primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? t("common.saving") : t("common.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
