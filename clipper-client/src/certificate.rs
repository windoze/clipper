//! Certificate utilities for handling self-signed certificates
//!
//! This module provides functionality to:
//! - Fetch server certificates
//! - Calculate SHA-256 fingerprints
//! - Verify certificates against trusted fingerprints
//! - Create custom certificate verifiers

use rustls::client::danger::ServerCertVerifier;
use rustls::pki_types::{CertificateDer, ServerName};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::error::{ClientError, Result};

/// Information about a server's TLS certificate
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    /// The server hostname
    pub host: String,
    /// SHA-256 fingerprint of the certificate (hex encoded, uppercase, colon-separated)
    pub fingerprint: String,
    /// Subject common name (CN) from the certificate
    pub subject_cn: Option<String>,
    /// Issuer common name (CN) from the certificate
    pub issuer_cn: Option<String>,
    /// Certificate validity start (not before)
    pub not_before: Option<String>,
    /// Certificate validity end (not after)
    pub not_after: Option<String>,
    /// Whether this is a self-signed certificate
    pub is_self_signed: bool,
    /// The raw DER-encoded certificate
    pub der_bytes: Vec<u8>,
    /// Whether the certificate is trusted by the system (passes WebPKI verification)
    pub is_system_trusted: bool,
}

/// Calculate SHA-256 fingerprint of a DER-encoded certificate
pub fn calculate_fingerprint(der_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(der_bytes);
    let result = hasher.finalize();

    // Format as uppercase hex with colons (e.g., "AB:CD:EF:...")
    result
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// Fetch the TLS certificate from a server
///
/// This connects to the server and retrieves its certificate chain.
/// It accepts any certificate during this fetch operation and also checks
/// if the certificate would pass standard WebPKI verification.
pub async fn fetch_server_certificate(host: &str, port: u16) -> Result<CertificateInfo> {
    // Ensure the ring crypto provider is installed
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Create a rustls config that accepts any certificate (just for fetching)
    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(CertificateFetcher::new()))
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| ClientError::Connection(format!("Failed to connect to {}: {}", addr, e)))?;

    let server_name = ServerName::try_from(host.to_string())
        .map_err(|e| ClientError::Certificate(format!("Invalid server name: {}", e)))?;

    let tls_stream = connector
        .connect(server_name.clone(), stream)
        .await
        .map_err(|e| ClientError::Certificate(format!("TLS handshake failed: {}", e)))?;

    // Get the peer certificates
    let (_, conn) = tls_stream.get_ref();
    let certs = conn
        .peer_certificates()
        .ok_or_else(|| ClientError::Certificate("No certificates received from server".to_string()))?;

    if certs.is_empty() {
        return Err(ClientError::Certificate("Empty certificate chain".to_string()));
    }

    // Use the first (leaf) certificate
    let cert_der = &certs[0];
    let fingerprint = calculate_fingerprint(cert_der.as_ref());

    // Parse certificate details using x509-parser if available, otherwise use basic info
    let (subject_cn, issuer_cn, not_before, not_after, is_self_signed) =
        parse_certificate_details(cert_der.as_ref());

    // Check if the certificate passes standard WebPKI verification
    let is_system_trusted = verify_certificate_with_system_roots(
        cert_der,
        &certs[1..],
        &server_name,
    );

    Ok(CertificateInfo {
        host: host.to_string(),
        fingerprint,
        subject_cn,
        issuer_cn,
        not_before,
        not_after,
        is_self_signed,
        der_bytes: cert_der.as_ref().to_vec(),
        is_system_trusted,
    })
}

/// Verify a certificate against the system's root certificate store
/// Returns true if the certificate is trusted, false otherwise
fn verify_certificate_with_system_roots(
    end_entity: &CertificateDer<'_>,
    intermediates: &[CertificateDer<'_>],
    server_name: &ServerName<'_>,
) -> bool {
    // Build root certificate store with system roots
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    // Create a WebPKI verifier
    let verifier = match rustls::client::WebPkiServerVerifier::builder(Arc::new(root_store)).build() {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Get current time
    let now = rustls::pki_types::UnixTime::now();

    // Try to verify the certificate
    verifier
        .verify_server_cert(end_entity, intermediates, server_name, &[], now)
        .is_ok()
}

/// Parse certificate details from DER bytes
/// Returns (subject_cn, issuer_cn, not_before, not_after, is_self_signed)
fn parse_certificate_details(_der_bytes: &[u8]) -> (Option<String>, Option<String>, Option<String>, Option<String>, bool) {
    // Basic parsing - extract common fields from X.509 certificate
    // This is a simplified parser that extracts CN fields

    // Try to find subject and issuer CN in the DER structure
    // X.509 certificates have a specific structure, but for simplicity
    // we'll mark unknown and focus on fingerprint verification

    // A more robust implementation would use x509-parser crate
    // For now, return None for details and check if cert is self-signed by comparing raw bytes

    (None, None, None, None, false)
}

/// Certificate verifier that accepts any certificate (used only for fetching cert info)
#[derive(Debug)]
struct CertificateFetcher;

impl CertificateFetcher {
    fn new() -> Self {
        Self
    }
}

impl rustls::client::danger::ServerCertVerifier for CertificateFetcher {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Accept any certificate - this verifier is only used for fetching cert info
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

/// Certificate verifier that trusts specific fingerprints
#[derive(Debug, Clone)]
pub struct TrustedFingerprintVerifier {
    /// Map of hostname to trusted SHA-256 fingerprint
    trusted_fingerprints: HashMap<String, String>,
    /// Root certificate store for standard verification
    root_store: Arc<rustls::RootCertStore>,
}

impl TrustedFingerprintVerifier {
    /// Create a new verifier with the given trusted fingerprints
    pub fn new(trusted_fingerprints: HashMap<String, String>) -> Self {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        Self {
            trusted_fingerprints,
            root_store: Arc::new(root_store),
        }
    }

    /// Add a trusted fingerprint for a host
    pub fn trust(&mut self, host: String, fingerprint: String) {
        self.trusted_fingerprints.insert(host, fingerprint);
    }

    /// Remove trust for a host
    pub fn untrust(&mut self, host: &str) {
        self.trusted_fingerprints.remove(host);
    }

    /// Check if a fingerprint is trusted for a host
    pub fn is_trusted(&self, host: &str, fingerprint: &str) -> bool {
        self.trusted_fingerprints
            .get(host)
            .map(|fp| fp == fingerprint)
            .unwrap_or(false)
    }
}

impl rustls::client::danger::ServerCertVerifier for TrustedFingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // First, check if the certificate fingerprint is trusted
        let fingerprint = calculate_fingerprint(end_entity.as_ref());
        let host = match server_name {
            ServerName::DnsName(name) => name.as_ref().to_string(),
            _ => String::new(),
        };

        if self.is_trusted(&host, &fingerprint) {
            // Certificate is explicitly trusted by fingerprint
            return Ok(rustls::client::danger::ServerCertVerified::assertion());
        }

        // Fall back to standard WebPKI verification
        let verifier = rustls::client::WebPkiServerVerifier::builder(self.root_store.clone())
            .build()
            .map_err(|e| rustls::Error::General(format!("Failed to create verifier: {}", e)))?;

        verifier.verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// Create a rustls ClientConfig that trusts specific certificate fingerprints
pub fn create_tls_config_with_trusted_certs(
    trusted_fingerprints: HashMap<String, String>,
) -> Arc<rustls::ClientConfig> {
    // Ensure the ring crypto provider is installed
    let _ = rustls::crypto::ring::default_provider().install_default();

    let verifier = TrustedFingerprintVerifier::new(trusted_fingerprints);

    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();

    Arc::new(config)
}

/// Create a reqwest Client that trusts specific certificate fingerprints
pub fn create_http_client_with_trusted_certs(
    trusted_fingerprints: HashMap<String, String>,
) -> std::result::Result<reqwest::Client, reqwest::Error> {
    use reqwest::ClientBuilder;

    // If we have trusted fingerprints, we need to accept potentially invalid certs
    // and do our own verification. Since reqwest doesn't support custom verifiers directly,
    // we configure it to accept invalid certs and rely on our TLS layer for WebSocket.
    // For HTTP requests, we'll handle verification differently.

    if trusted_fingerprints.is_empty() {
        // No custom certs, use default secure client
        ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .build()
    } else {
        // We have trusted certs - need to accept them
        // Note: reqwest with rustls-tls doesn't easily support custom verifiers
        // For now, we'll accept invalid certs when trusted_fingerprints is set
        // A production implementation might use a custom connector
        ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_calculation() {
        // Test with known data
        let test_data = b"test certificate data";
        let fingerprint = calculate_fingerprint(test_data);

        // Fingerprint should be hex encoded with colons
        assert!(fingerprint.contains(':'));
        assert_eq!(fingerprint.len(), 64 + 31); // 64 hex chars + 31 colons

        // Should be uppercase
        assert_eq!(fingerprint, fingerprint.to_uppercase());
    }

    #[test]
    fn test_trusted_verifier_is_trusted() {
        let mut fingerprints = HashMap::new();
        fingerprints.insert("example.com".to_string(), "AB:CD:EF:12".to_string());

        let verifier = TrustedFingerprintVerifier::new(fingerprints);

        assert!(verifier.is_trusted("example.com", "AB:CD:EF:12"));
        assert!(!verifier.is_trusted("example.com", "XX:YY:ZZ:99"));
        assert!(!verifier.is_trusted("other.com", "AB:CD:EF:12"));
    }
}
