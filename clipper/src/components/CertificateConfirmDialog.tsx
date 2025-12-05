import { useI18n } from "@unwritten-codes/clipper-ui";
import "./CertificateConfirmDialog.css";

export interface CertificateInfo {
  host: string;
  fingerprint: string;
  isTrusted: boolean;
}

interface CertificateConfirmDialogProps {
  isOpen: boolean;
  certificate: CertificateInfo | null;
  onConfirm: () => void;
  onCancel: () => void;
  loading?: boolean;
}

export function CertificateConfirmDialog({
  isOpen,
  certificate,
  onConfirm,
  onCancel,
  loading = false,
}: CertificateConfirmDialogProps) {
  const { t } = useI18n();

  if (!isOpen || !certificate) return null;

  // Format fingerprint for display (show in groups of 4 for readability)
  const formatFingerprint = (fp: string) => {
    // Fingerprint is already in format "AB:CD:EF:..."
    // Split into groups of 4 pairs for better readability
    const pairs = fp.split(":");
    const groups: string[] = [];
    for (let i = 0; i < pairs.length; i += 4) {
      groups.push(pairs.slice(i, i + 4).join(":"));
    }
    return groups;
  };

  const fingerprintGroups = formatFingerprint(certificate.fingerprint);

  return (
    <div className="cert-dialog-backdrop" onClick={onCancel}>
      <div className="cert-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="cert-dialog-header">
          <div className="cert-dialog-icon">
            <svg
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
              <path d="M12 8v4" />
              <path d="M12 16h.01" />
            </svg>
          </div>
          <h2>{t("certificate.title")}</h2>
        </div>

        <div className="cert-dialog-content">
          <p className="cert-dialog-warning">{t("certificate.warning")}</p>

          <div className="cert-dialog-reasons">
            <p className="cert-dialog-explanation">{t("certificate.explanation")}</p>
            <ul className="cert-dialog-reason-list">
              <li>{t("certificate.reason1")}</li>
              <li>{t("certificate.reason2")}</li>
              <li>{t("certificate.reason3")}</li>
            </ul>
          </div>

          <div className="cert-dialog-info">
            <div className="cert-info-row">
              <span className="cert-info-label">{t("certificate.host")}</span>
              <span className="cert-info-value cert-host">{certificate.host}</span>
            </div>

            <div className="cert-info-row">
              <span className="cert-info-label">{t("certificate.fingerprint")}</span>
              <div className="cert-fingerprint">
                {fingerprintGroups.map((group, index) => (
                  <code key={index} className="cert-fingerprint-group">
                    {group}
                  </code>
                ))}
              </div>
              <span className="cert-fingerprint-hint">{t("certificate.fingerprintHint")}</span>
            </div>
          </div>

          <p className="cert-dialog-hint">{t("certificate.hint")}</p>
        </div>

        <div className="cert-dialog-footer">
          <button
            type="button"
            className="cert-btn secondary"
            onClick={onCancel}
            disabled={loading}
          >
            {t("common.cancel")}
          </button>
          <button
            type="button"
            className="cert-btn primary"
            onClick={onConfirm}
            disabled={loading}
          >
            {loading ? t("certificate.trusting") : t("certificate.trust")}
          </button>
        </div>
      </div>
    </div>
  );
}
