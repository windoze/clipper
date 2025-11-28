import { useState, useEffect, ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "@anthropic/clipper-ui";

interface DropZoneProps {
  children: ReactNode;
}

interface DragDropPayload {
  paths: string[];
  position: { x: number; y: number };
}

export function DropZone({ children }: DropZoneProps) {
  const { t } = useI18n();
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    // Listen for Tauri drag events
    const unlistenEnter = listen<DragDropPayload>("tauri://drag-enter", () => {
      setIsDragging(true);
    });

    const unlistenLeave = listen("tauri://drag-leave", () => {
      setIsDragging(false);
    });

    const unlistenDrop = listen<DragDropPayload>("tauri://drag-drop", () => {
      setIsDragging(false);
      // The actual file upload is handled by the backend in lib.rs
    });

    return () => {
      unlistenEnter.then((fn) => fn());
      unlistenLeave.then((fn) => fn());
      unlistenDrop.then((fn) => fn());
    };
  }, []);

  return (
    <div className={`drop-zone ${isDragging ? "dragging" : ""}`}>
      {children}
      {isDragging && (
        <div className="drop-overlay">
          <div className="drop-indicator">
            <span className="drop-icon">📁</span>
            <span className="drop-text">{t("dropZone.hint")}</span>
          </div>
        </div>
      )}
    </div>
  );
}
