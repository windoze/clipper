import { useI18n } from "@unwritten-codes/clipper-ui";
import "./CertificateMismatchDialog.css";

export interface CertificateMismatchInfo {
  host: string;
  fingerprint: string;
  storedFingerprint: string;
}

interface CertificateMismatchDialogProps {
  isOpen: boolean;
  mismatchInfo: CertificateMismatchInfo | null;
  onAcceptRisk: () => void;
  onReject: () => void;
  loading?: boolean;
}

export function CertificateMismatchDialog({
  isOpen,
  mismatchInfo,
  onAcceptRisk,
  onReject,
  loading = false,
}: CertificateMismatchDialogProps) {
  const { t } = useI18n();

  if (!isOpen || !mismatchInfo) return null;

  // Format fingerprint for display (show in groups of 4 for readability)
  const formatFingerprint = (fp: string) => {
    const pairs = fp.split(":");
    const groups: string[] = [];
    for (let i = 0; i < pairs.length; i += 4) {
      groups.push(pairs.slice(i, i + 4).join(":"));
    }
    return groups;
  };

  const newFingerprintGroups = formatFingerprint(mismatchInfo.fingerprint);
  const storedFingerprintGroups = formatFingerprint(mismatchInfo.storedFingerprint);

  return (
    <div className="mismatch-dialog-backdrop">
      <div className="mismatch-dialog">
        <div className="mismatch-dialog-header">
          <div className="mismatch-dialog-icon">
            <svg
              width="56"
              height="56"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
          </div>
          <h2>{t("certificateMismatch.title")}</h2>
        </div>

        <div className="mismatch-dialog-content">
          <div className="mismatch-critical-warning">
            <strong>{t("certificateMismatch.criticalWarning")}</strong>
          </div>

          <p className="mismatch-dialog-warning">
            {t("certificateMismatch.warning")}
          </p>

          <div className="mismatch-dialog-explanation">
            <p>{t("certificateMismatch.explanation")}</p>
            <ul className="mismatch-reason-list">
              <li className="mismatch-reason-danger">{t("certificateMismatch.reason1")}</li>
              <li>{t("certificateMismatch.reason2")}</li>
              <li>{t("certificateMismatch.reason3")}</li>
            </ul>
          </div>

          <div className="mismatch-dialog-info">
            <div className="mismatch-info-row">
              <span className="mismatch-info-label">{t("certificateMismatch.host")}</span>
              <span className="mismatch-info-value mismatch-host">{mismatchInfo.host}</span>
            </div>

            <div className="mismatch-info-row">
              <span className="mismatch-info-label mismatch-label-stored">
                {t("certificateMismatch.storedFingerprint")}
              </span>
              <div className="mismatch-fingerprint mismatch-fingerprint-stored">
                {storedFingerprintGroups.map((group, index) => (
                  <code key={index} className="mismatch-fingerprint-group">
                    {group}
                  </code>
                ))}
              </div>
            </div>

            <div className="mismatch-info-row">
              <span className="mismatch-info-label mismatch-label-new">
                {t("certificateMismatch.newFingerprint")}
              </span>
              <div className="mismatch-fingerprint mismatch-fingerprint-new">
                {newFingerprintGroups.map((group, index) => (
                  <code key={index} className="mismatch-fingerprint-group">
                    {group}
                  </code>
                ))}
              </div>
            </div>
          </div>

          <div className="mismatch-recommendation">
            <p>{t("certificateMismatch.recommendation")}</p>
          </div>
        </div>

        <div className="mismatch-dialog-footer">
          <button
            type="button"
            className="mismatch-btn reject"
            onClick={onReject}
            disabled={loading}
          >
            {t("certificateMismatch.reject")}
          </button>
          <button
            type="button"
            className="mismatch-btn danger"
            onClick={onAcceptRisk}
            disabled={loading}
          >
            {loading ? t("certificateMismatch.accepting") : t("certificateMismatch.acceptRisk")}
          </button>
        </div>
      </div>
    </div>
  );
}
