import { useState, useEffect, useMemo, useRef } from "react";
import hljs from "highlight.js";
import { Clip, Tag, isFavorite, calculateAgeRatio } from "../types";
import { ImagePopup } from "./ImagePopup";
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
  /** Called before any clip modification (update/delete) to allow caller to prepare (e.g., capture scroll anchor, register clip ID) */
  onBeforeClipModified?: (clipId: string) => void;
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

// Truncate notes for tooltip display (max 4 lines)
const MAX_TOOLTIP_LINES = 4;
function truncateNotesForTooltip(notes: string): string {
  const lines = notes.split("\n");
  if (lines.length <= MAX_TOOLTIP_LINES) return notes;
  return lines.slice(0, MAX_TOOLTIP_LINES).join("\n") + "\n...";
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
  onBeforeClipModified,
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

  // Check if clip has meaningful notes (not empty or blank)
  const hasNotes = clip.additional_notes?.trim();
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [showPopup, setShowPopup] = useState(false);
  const [showShareDialog, setShowShareDialog] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [selectedLanguage, setSelectedLanguage] = useState<LanguageId>(() => detectLanguage(clip.content));
  const [showNotesPopup, setShowNotesPopup] = useState(false);
  const [notesValue, setNotesValue] = useState(clip.additional_notes || "");
  const [savingNotes, setSavingNotes] = useState(false);
  const [notesPopupPosition, setNotesPopupPosition] = useState<{ top: number; left: number; showBelow: boolean } | null>(null);
  const notesButtonRef = useRef<HTMLButtonElement>(null);
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [newTagValue, setNewTagValue] = useState("");
  const [savingTag, setSavingTag] = useState(false);
  const addTagInputRef = useRef<HTMLInputElement>(null);
  const [tagSuggestions, setTagSuggestions] = useState<Tag[]>([]);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(-1);
  const [showTagSuggestions, setShowTagSuggestions] = useState(false);
  const tagSuggestionsRef = useRef<HTMLDivElement>(null);
  const [tagToRemove, setTagToRemove] = useState<string | null>(null);
  const [removingTag, setRemovingTag] = useState(false);

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
      if (isImage && api.copyImageToClipboard) {
        // Copy image to clipboard
        await api.copyImageToClipboard(clip.id);
        showToast(t("toast.imageCopied"));
      } else {
        // Copy text content
        await api.copyToClipboard(clip.content);
        showToast(t("toast.clipCopied"));
      }
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
      showToast(t("toast.copyFailed"), "error");
    }
  };

  // Check if image copy is supported (either via API method or browser Clipboard API)
  const canCopyImage = isImage && (api.copyImageToClipboard || typeof ClipboardItem !== "undefined");

  const handleImageClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowPopup(true);
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowDeleteConfirm(true);
  };

  const handleDeleteConfirm = async () => {
    setDeleting(true);
    try {
      // Call onBeforeClipModified BEFORE the API call so the handler can:
      // 1. Capture scroll anchor
      // 2. Register the clip ID to skip WebSocket refetch
      // The API call will trigger a WebSocket event, so we need to be ready
      onBeforeClipModified?.(clip.id);
      onClipDeleted?.(clip.id);
      await api.deleteClip(clip.id);
      setShowDeleteConfirm(false);
      showToast(t("toast.clipDeleted"));
    } catch (err) {
      console.error("Failed to delete clip:", err);
      // TODO: Could add rollback logic here if delete fails
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

  const handleNotesClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setNotesValue(clip.additional_notes || "");

    // Calculate position based on button location
    const button = notesButtonRef.current;
    if (button) {
      const rect = button.getBoundingClientRect();
      const popupHeight = 220; // Approximate height of popup
      const viewportHeight = window.innerHeight;
      const spaceAbove = rect.top;
      const spaceBelow = viewportHeight - rect.bottom;
      const gap = 8; // Gap between button and popup

      // Show below if not enough space above, or if there's significantly more space below
      const showBelow = spaceAbove < popupHeight + gap || spaceBelow > spaceAbove;

      setNotesPopupPosition({
        // When showing below: position top of popup at bottom of button + gap
        // When showing above: position bottom of popup at top of button - gap
        top: showBelow ? rect.bottom + gap : rect.top - gap,
        left: rect.left + rect.width / 2,
        showBelow,
      });
    }

    setShowNotesPopup(true);
  };

  const handleNotesSave = async () => {
    setSavingNotes(true);
    try {
      // Call onBeforeClipModified BEFORE the API call to prepare for WebSocket event
      onBeforeClipModified?.(clip.id);
      const trimmedNotes = notesValue.trim();
      // If existing note is cleared (trimmed to empty), send empty string to clear it
      // Otherwise send trimmed value, or undefined if no change needed
      const notesToSave = hasNotes && !trimmedNotes ? "" : (trimmedNotes || undefined);
      const updatedClip = await api.updateClip(clip.id, clip.tags, notesToSave);
      setShowNotesPopup(false);
      onClipUpdated?.(updatedClip);
      showToast(t("toast.clipUpdated"));
    } catch (err) {
      console.error("Failed to save notes:", err);
      showToast(t("toast.updateFailed"), "error");
    } finally {
      setSavingNotes(false);
    }
  };

  const handleNotesCancel = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowNotesPopup(false);
    setNotesValue(clip.additional_notes || "");
  };

  const handleNotesClear = async () => {
    setSavingNotes(true);
    try {
      // Call onBeforeClipModified BEFORE the API call to prepare for WebSocket event
      onBeforeClipModified?.(clip.id);
      // Send empty string to clear notes (null means "don't change" in the API)
      const updatedClip = await api.updateClip(clip.id, clip.tags, "");
      setShowNotesPopup(false);
      setNotesValue("");
      onClipUpdated?.(updatedClip);
      showToast(t("toast.clipUpdated"));
    } catch (err) {
      console.error("Failed to clear notes:", err);
      showToast(t("toast.updateFailed"), "error");
    } finally {
      setSavingNotes(false);
    }
  };

  // Handle keyboard events for notes popup
  useEffect(() => {
    if (!showNotesPopup) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setShowNotesPopup(false);
        setNotesValue(clip.additional_notes || "");
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [showNotesPopup, clip.additional_notes]);

  const handleAddTagClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsAddingTag(true);
    setNewTagValue("");
    setTagSuggestions([]);
    setSelectedSuggestionIndex(-1);
    setShowTagSuggestions(false);
  };

  // Focus the input when it becomes visible
  useEffect(() => {
    if (isAddingTag && addTagInputRef.current) {
      addTagInputRef.current.focus();
    }
  }, [isAddingTag]);

  // Fetch tag suggestions when input value changes
  useEffect(() => {
    if (!isAddingTag || !onSearchTags) {
      setTagSuggestions([]);
      setShowTagSuggestions(false);
      return;
    }

    const query = newTagValue.trim();
    if (query.length === 0) {
      setTagSuggestions([]);
      setShowTagSuggestions(false);
      return;
    }

    const timeoutId = setTimeout(async () => {
      try {
        const results = await onSearchTags(query);
        // Filter out system tags and tags already on this clip
        const filtered = results.filter(
          (tag) => !tag.text.startsWith("$") && !clip.tags.includes(tag.text)
        );
        setTagSuggestions(filtered);
        setShowTagSuggestions(filtered.length > 0);
        setSelectedSuggestionIndex(-1);
      } catch (err) {
        console.error("Failed to fetch tag suggestions:", err);
        setTagSuggestions([]);
        setShowTagSuggestions(false);
      }
    }, 150);

    return () => clearTimeout(timeoutId);
  }, [newTagValue, isAddingTag, onSearchTags, clip.tags]);

  const saveTag = async (tagText: string) => {
    const trimmedTag = tagText.trim();
    if (!trimmedTag || clip.tags.includes(trimmedTag)) {
      return false;
    }
    setSavingTag(true);
    try {
      // Call onBeforeClipModified BEFORE the API call to prepare for WebSocket event
      onBeforeClipModified?.(clip.id);
      const newTags = [...clip.tags, trimmedTag];
      const updatedClip = await api.updateClip(clip.id, newTags, clip.additional_notes || undefined);
      onClipUpdated?.(updatedClip);
      showToast(t("toast.clipUpdated"));
      setIsAddingTag(false);
      setNewTagValue("");
      setTagSuggestions([]);
      setShowTagSuggestions(false);
      return true;
    } catch (err) {
      console.error("Failed to add tag:", err);
      showToast(t("toast.updateFailed"), "error");
      return false;
    } finally {
      setSavingTag(false);
    }
  };

  const handleAddTagKeyDown = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      setIsAddingTag(false);
      setNewTagValue("");
      setTagSuggestions([]);
      setShowTagSuggestions(false);
    } else if (e.key === "ArrowDown") {
      if (showTagSuggestions && tagSuggestions.length > 0) {
        e.preventDefault();
        setSelectedSuggestionIndex((prev) =>
          prev < tagSuggestions.length - 1 ? prev + 1 : prev
        );
      }
    } else if (e.key === "ArrowUp") {
      if (showTagSuggestions && tagSuggestions.length > 0) {
        e.preventDefault();
        setSelectedSuggestionIndex((prev) => (prev > 0 ? prev - 1 : -1));
      }
    } else if (e.key === "Enter") {
      e.stopPropagation();
      e.preventDefault();
      // If a suggestion is selected, use it
      if (selectedSuggestionIndex >= 0 && tagSuggestions[selectedSuggestionIndex]) {
        await saveTag(tagSuggestions[selectedSuggestionIndex].text);
      } else {
        const trimmedTag = newTagValue.trim();
        if (trimmedTag) {
          await saveTag(trimmedTag);
        } else {
          // Empty input, just cancel
          setIsAddingTag(false);
          setNewTagValue("");
          setTagSuggestions([]);
          setShowTagSuggestions(false);
        }
      }
    }
  };

  const handleSuggestionClick = async (e: React.MouseEvent, tagText: string) => {
    e.stopPropagation();
    e.preventDefault();
    await saveTag(tagText);
  };

  const handleAddTagBlur = (e: React.FocusEvent) => {
    // Don't close if clicking on a suggestion
    if (tagSuggestionsRef.current?.contains(e.relatedTarget as Node)) {
      return;
    }
    // Cancel if we lose focus without saving
    if (!savingTag) {
      setIsAddingTag(false);
      setNewTagValue("");
      setTagSuggestions([]);
      setShowTagSuggestions(false);
    }
  };

  const handleRemoveTagClick = (e: React.MouseEvent, tag: string) => {
    e.stopPropagation();
    e.preventDefault();
    setTagToRemove(tag);
  };

  const handleRemoveTagConfirm = async () => {
    if (!tagToRemove) return;
    setRemovingTag(true);
    try {
      // Call onBeforeClipModified BEFORE the API call to prepare for WebSocket event
      onBeforeClipModified?.(clip.id);
      const newTags = clip.tags.filter((t) => t !== tagToRemove);
      const updatedClip = await api.updateClip(clip.id, newTags, clip.additional_notes || undefined);
      onClipUpdated?.(updatedClip);
      showToast(t("toast.clipUpdated"));
      setTagToRemove(null);
    } catch (err) {
      console.error("Failed to remove tag:", err);
      showToast(t("toast.updateFailed"), "error");
    } finally {
      setRemovingTag(false);
    }
  };

  const handleRemoveTagCancel = (e: React.MouseEvent) => {
    e.stopPropagation();
    setTagToRemove(null);
  };

  // Handle keyboard events for tag removal confirmation dialog
  useEffect(() => {
    if (!tagToRemove) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setTagToRemove(null);
      } else if (e.key === "Enter" && !removingTag) {
        e.preventDefault();
        handleRemoveTagConfirm();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [tagToRemove, removingTag]);

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
        data-clip-id={clip.id}
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
            {hasNotes ? (
              <Tooltip content={truncateNotesForTooltip(clip.additional_notes!)} position="top" maxWidth={450}>
                <button
                  ref={notesButtonRef}
                  className="notes-indicator"
                  onClick={handleNotesClick}
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
            ) : (
              <button
                ref={notesButtonRef}
                className="notes-indicator notes-indicator-empty"
                onClick={handleNotesClick}
                title={t("tooltip.addNotes")}
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
                  <line x1="12" y1="11" x2="12" y2="17"></line>
                  <line x1="9" y1="14" x2="15" y2="14"></line>
                </svg>
              </button>
            )}
          </div>
          <div className="clip-actions">
            {(!isImage || canCopyImage) && (
              <button
                className="copy-button"
                onClick={handleCopyClick}
                title={isImage ? t("tooltip.copyImage") : t("tooltip.copy")}
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

        <div className="clip-tags">
          {displayTags.map((tag) => (
            <div key={tag} className="tag-wrapper">
              <button
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
              {!isHostTag(tag) && (
                <button
                  className="tag-remove-btn"
                  onClick={(e) => handleRemoveTagClick(e, tag)}
                  title={t("clip.removeTag")}
                >
                  <svg
                    width="10"
                    height="10"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="3"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>
              )}
            </div>
          ))}
          {isAddingTag ? (
            <div className="tag-add-wrapper">
              <input
                ref={addTagInputRef}
                type="text"
                className="tag-add-input"
                value={newTagValue}
                onChange={(e) => setNewTagValue(e.target.value)}
                onKeyDown={handleAddTagKeyDown}
                onBlur={handleAddTagBlur}
                onClick={(e) => e.stopPropagation()}
                placeholder={t("clip.addTag.placeholder")}
                disabled={savingTag}
                autoCapitalize="off"
                autoCorrect="off"
                autoComplete="off"
                spellCheck={false}
              />
              {showTagSuggestions && tagSuggestions.length > 0 && (
                <div
                  ref={tagSuggestionsRef}
                  className="tag-suggestions"
                  onClick={(e) => e.stopPropagation()}
                >
                  {tagSuggestions.map((tag, index) => (
                    <button
                      key={tag.id}
                      className={`tag-suggestion-item ${index === selectedSuggestionIndex ? "selected" : ""}`}
                      onMouseDown={(e) => handleSuggestionClick(e, tag.text)}
                      onMouseEnter={() => setSelectedSuggestionIndex(index)}
                    >
                      {tag.text}
                    </button>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <button
              className="tag tag-add"
              onClick={handleAddTagClick}
              title={t("clip.addTag")}
            >
              +
            </button>
          )}
        </div>

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

      {showNotesPopup && notesPopupPosition && (
        <div className="notes-popup-backdrop" onClick={handleNotesCancel}>
          <div
            className={`notes-popup-dialog ${notesPopupPosition.showBelow ? "notes-popup-below" : "notes-popup-above"}`}
            style={{
              position: "fixed",
              top: notesPopupPosition.showBelow ? notesPopupPosition.top : "auto",
              bottom: notesPopupPosition.showBelow ? "auto" : `calc(100vh - ${notesPopupPosition.top}px)`,
              left: notesPopupPosition.left,
              transform: "translateX(-50%)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="notes-popup-header">
              <h3>{hasNotes ? t("editClip.notes") : t("tooltip.addNotes")}</h3>
            </div>
            <textarea
              className="notes-popup-input"
              value={notesValue}
              onChange={(e) => setNotesValue(e.target.value)}
              placeholder={t("editClip.notes.placeholder")}
              autoFocus
              rows={4}
            />
            <div className="notes-popup-actions">
              <button
                className="notes-popup-btn cancel"
                onClick={handleNotesCancel}
                disabled={savingNotes}
              >
                {t("common.cancel")}
              </button>
              {hasNotes && (
                <button
                  className="notes-popup-btn clear"
                  onClick={handleNotesClear}
                  disabled={savingNotes}
                >
                  {savingNotes ? t("common.clearing") : t("common.clear")}
                </button>
              )}
              <button
                className="notes-popup-btn save"
                onClick={handleNotesSave}
                disabled={savingNotes}
              >
                {savingNotes ? t("common.saving") : t("common.save")}
              </button>
            </div>
          </div>
        </div>
      )}

      {tagToRemove && (
        <div className="delete-confirm-backdrop" onClick={handleRemoveTagCancel}>
          <div
            className="delete-confirm-dialog"
            onClick={(e) => e.stopPropagation()}
          >
            <p>{t("clip.removeTag_confirm", { tag: tagToRemove })}</p>
            <div className="delete-confirm-actions">
              <button
                className="delete-confirm-btn cancel"
                onClick={handleRemoveTagCancel}
                disabled={removingTag}
              >
                {t("common.cancel")}
              </button>
              <button
                className="delete-confirm-btn confirm"
                onClick={handleRemoveTagConfirm}
                disabled={removingTag}
              >
                {removingTag ? t("common.removing") : t("common.remove")}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
