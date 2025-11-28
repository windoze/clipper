import { useI18n } from "../i18n";

interface FavoriteToggleProps {
  value: boolean;
  onChange: (value: boolean) => void;
}

export function FavoriteToggle({ value, onChange }: FavoriteToggleProps) {
  const { t } = useI18n();

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
        <span className="toggle-text">{t("filter.favorites")}</span>
      </label>
    </div>
  );
}
