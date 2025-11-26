interface FavoriteToggleProps {
  value: boolean;
  onChange: (value: boolean) => void;
}

export function FavoriteToggle({ value, onChange }: FavoriteToggleProps) {
  return (
    <div className="favorite-toggle">
      <label className="toggle-label">
        <input
          type="checkbox"
          checked={value}
          onChange={(e) => onChange(e.target.checked)}
          className="toggle-input"
        />
        <span className="toggle-switch"></span>
        <span className="toggle-text">Favorites only</span>
      </label>
    </div>
  );
}
