import { useEffect } from "react";

interface ImagePopupProps {
  imageUrl: string;
  filename: string;
  onClose: () => void;
}

export function ImagePopup({ imageUrl, filename, onClose }: ImagePopupProps) {
  // Close on escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  // Close on backdrop click
  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <div className="image-popup-backdrop" onClick={handleBackdropClick}>
      <div className="image-popup-container">
        <div className="image-popup-header">
          <span className="image-popup-filename">{filename}</span>
          <button className="image-popup-close" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="image-popup-content">
          <img src={imageUrl} alt={filename} className="image-popup-image" />
        </div>
      </div>
    </div>
  );
}
