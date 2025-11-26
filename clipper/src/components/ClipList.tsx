import { Clip } from "../types";
import { ClipEntry } from "./ClipEntry";

interface ClipListProps {
  clips: Clip[];
  loading: boolean;
  error: string | null;
  onToggleFavorite: (clip: Clip) => void;
}

export function ClipList({
  clips,
  loading,
  error,
  onToggleFavorite,
}: ClipListProps) {
  if (loading) {
    return (
      <div className="clip-list-status">
        <div className="loading-spinner"></div>
        <span>Loading clips...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="clip-list-status error">
        <span>Error: {error}</span>
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="clip-list-status empty">
        <span>No clips found</span>
      </div>
    );
  }

  return (
    <div className="clip-list">
      {clips.map((clip) => (
        <ClipEntry
          key={clip.id}
          clip={clip}
          onToggleFavorite={onToggleFavorite}
        />
      ))}
    </div>
  );
}
