import { useState, useEffect, useRef, useCallback } from "react";
import { Clip, Tag } from "../types";
import { useI18n } from "../i18n";
import { useToast } from "./Toast";
import { useApi } from "../api";

interface EditClipDialogProps {
  clip: Clip;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedClip: Clip) => void;
  /** Function to search tags by query (optional, enables tag autocomplete) */
  onSearchTags?: (query: string) => Promise<Tag[]>;
}

export function EditClipDialog({
  clip,
  isOpen,
  onClose,
  onSave,
  onSearchTags,
}: EditClipDialogProps) {
  const { t } = useI18n();
  const { showToast } = useToast();
  const api = useApi();
  // Filter out system tags (starting with $) for editing
  const userTags = clip.tags.filter((tag) => !tag.startsWith("$"));
  const systemTags = clip.tags.filter((tag) => tag.startsWith("$"));

  const [tags, setTags] = useState<string[]>(userTags);
  const [tagInput, setTagInput] = useState("");
  const [notes, setNotes] = useState(clip.additional_notes || "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Tag autocomplete state
  const [tagSuggestions, setTagSuggestions] = useState<Tag[]>([]);
  const [showTagDropdown, setShowTagDropdown] = useState(false);
  const [selectedTagIndex, setSelectedTagIndex] = useState(-1);

  const tagInputRef = useRef<HTMLInputElement>(null);
  const tagDropdownRef = useRef<HTMLDivElement>(null);

  // Reset state when clip changes or dialog opens
  useEffect(() => {
    if (isOpen) {
      const userTags = clip.tags.filter((t) => !t.startsWith("$"));
      setTags(userTags);
      setNotes(clip.additional_notes || "");
      setTagInput("");
      setError(null);
      setTagSuggestions([]);
      setShowTagDropdown(false);
      setSelectedTagIndex(-1);
    }
  }, [isOpen, clip]);

  // Handle ESC key to close dialog
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !showTagDropdown) {
        onClose();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose, showTagDropdown]);

  // Fetch tag suggestions when input changes
  useEffect(() => {
    if (!onSearchTags || !tagInput.trim()) {
      setTagSuggestions([]);
      setShowTagDropdown(false);
      return;
    }

    const fetchTags = async () => {
      try {
        const results = await onSearchTags(tagInput.trim());
        // Filter out system tags and already-added tags
        const filtered = results.filter(
          (tag) => !tag.text.startsWith("$") && !tags.includes(tag.text)
        );
        setTagSuggestions(filtered);
        setShowTagDropdown(filtered.length > 0);
        setSelectedTagIndex(-1);
      } catch (err) {
        console.error("Failed to search tags:", err);
        setTagSuggestions([]);
        setShowTagDropdown(false);
      }
    };

    const debounceTimer = setTimeout(fetchTags, 150);
    return () => clearTimeout(debounceTimer);
  }, [tagInput, onSearchTags, tags]);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!showTagDropdown) return;

    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (
        tagDropdownRef.current &&
        !tagDropdownRef.current.contains(target) &&
        tagInputRef.current &&
        !tagInputRef.current.contains(target)
      ) {
        setShowTagDropdown(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [showTagDropdown]);

  const handleSelectTagFromDropdown = useCallback((tagText: string) => {
    if (!tags.includes(tagText) && !tagText.startsWith("$")) {
      setTags((prev) => [...prev, tagText]);
    }
    setTagInput("");
    setShowTagDropdown(false);
    setSelectedTagIndex(-1);
    tagInputRef.current?.focus();
  }, [tags]);

  const handleAddTag = () => {
    const trimmedTag = tagInput.trim();
    if (
      trimmedTag &&
      !tags.includes(trimmedTag) &&
      !trimmedTag.startsWith("$")
    ) {
      setTags([...tags, trimmedTag]);
      setTagInput("");
      setShowTagDropdown(false);
      setSelectedTagIndex(-1);
      tagInputRef.current?.focus();
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter((t) => t !== tagToRemove));
  };

  const handleTagInputKeyDown = (e: React.KeyboardEvent) => {
    // Handle dropdown navigation when open
    if (showTagDropdown && tagSuggestions.length > 0) {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedTagIndex((prev) =>
            prev < tagSuggestions.length - 1 ? prev + 1 : 0
          );
          return;
        case "ArrowUp":
          e.preventDefault();
          setSelectedTagIndex((prev) =>
            prev > 0 ? prev - 1 : tagSuggestions.length - 1
          );
          return;
        case "Enter":
          e.preventDefault();
          if (selectedTagIndex >= 0 && selectedTagIndex < tagSuggestions.length) {
            handleSelectTagFromDropdown(tagSuggestions[selectedTagIndex].text);
          } else {
            handleAddTag();
          }
          return;
        case "Tab":
          if (selectedTagIndex >= 0 && selectedTagIndex < tagSuggestions.length) {
            e.preventDefault();
            handleSelectTagFromDropdown(tagSuggestions[selectedTagIndex].text);
          }
          return;
        case "Escape":
          e.preventDefault();
          setShowTagDropdown(false);
          setSelectedTagIndex(-1);
          return;
      }
    }

    // Handle Enter and comma to add tag
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      handleAddTag();
    } else if (e.key === "Backspace" && tagInput === "" && tags.length > 0) {
      // Remove last tag when backspace is pressed on empty input
      setTags(tags.slice(0, -1));
    }
  };

  const handleTagInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    // If user types comma, add the tag before the comma
    if (value.endsWith(",")) {
      const tagToAdd = value.slice(0, -1).trim();
      if (tagToAdd && !tags.includes(tagToAdd) && !tagToAdd.startsWith("$")) {
        setTags([...tags, tagToAdd]);
        setTagInput("");
        setShowTagDropdown(false);
        setSelectedTagIndex(-1);
      }
    } else {
      setTagInput(value);
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
            <div className="tag-editor-wrapper">
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
                    onChange={handleTagInputChange}
                    onKeyDown={handleTagInputKeyDown}
                    placeholder={
                      tags.length === 0 ? t("editClip.tags.placeholder") : ""
                    }
                    autoComplete="off"
                  />
                </div>
              </div>
              {/* Tag suggestions dropdown */}
              {showTagDropdown && tagSuggestions.length > 0 && (
                <div className="tag-editor-dropdown" ref={tagDropdownRef}>
                  {tagSuggestions.map((tag, index) => (
                    <div
                      key={tag.id}
                      className={`tag-editor-dropdown-item ${index === selectedTagIndex ? "selected" : ""}`}
                      onClick={() => handleSelectTagFromDropdown(tag.text)}
                      onMouseEnter={() => setSelectedTagIndex(index)}
                    >
                      <span className="tag-editor-dropdown-item-hash">#</span>
                      {tag.text}
                    </div>
                  ))}
                </div>
              )}
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
