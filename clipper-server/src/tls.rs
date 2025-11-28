//! TLS/HTTPS server configuration and management.
//!
//! This module provides TLS support for the Clipper server using rustls.
//! It supports both manual certificate configuration and ACME automatic
//! certificate management.

#[cfg(feature = "tls")]
use std::path::Path;
#[cfg(feature = "tls")]
use std::sync::Arc;

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use thiserror::Error;

/// Errors that can occur during TLS configuration.
#[cfg(feature = "tls")]
#[derive(Error, Debug)]
pub enum TlsError {
    #[error("Failed to load certificate: {0}")]
    CertificateLoad(String),

    #[error("Failed to load private key: {0}")]
    KeyLoad(String),

    #[error("Invalid certificate format: {0}")]
    InvalidCertificate(String),

    #[error("Invalid key format: {0}")]
    InvalidKey(String),

    #[error("TLS configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "tls")]
pub type TlsResult<T> = Result<T, TlsError>;

/// TLS configuration manager.
///
/// Handles loading certificates and keys, and creating TLS configurations
/// for the HTTPS server.
#[cfg(feature = "tls")]
pub struct TlsManager {
    config: RustlsConfig,
}

#[cfg(feature = "tls")]
impl TlsManager {
    /// Create a new TLS manager from PEM files.
    pub async fn from_pem_files(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> TlsResult<Self> {
        let cert_path = cert_path.as_ref();
        let key_path = key_path.as_ref();

        tracing::info!(
            "Loading TLS certificate from {} and key from {}",
            cert_path.display(),
            key_path.display()
        );

        let config = RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| TlsError::Configuration(e.to_string()))?;

        Ok(Self { config })
    }

    /// Create a new TLS manager from PEM strings.
    pub async fn from_pem(cert_pem: &str, key_pem: &str) -> TlsResult<Self> {
        let cert_bytes = cert_pem.as_bytes().to_vec();
        let key_bytes = key_pem.as_bytes().to_vec();

        let config = RustlsConfig::from_pem(cert_bytes, key_bytes)
            .await
            .map_err(|e| TlsError::Configuration(e.to_string()))?;

        Ok(Self { config })
    }

    /// Get the rustls configuration for use with axum-server.
    pub fn config(&self) -> RustlsConfig {
        self.config.clone()
    }

    /// Reload certificates from PEM files.
    ///
    /// This allows hot-reloading certificates without restarting the server.
    pub async fn reload_from_pem_files(
        &self,
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> TlsResult<()> {
        let cert_path = cert_path.as_ref();
        let key_path = key_path.as_ref();

        tracing::info!(
            "Reloading TLS certificate from {} and key from {}",
            cert_path.display(),
            key_path.display()
        );

        self.config
            .reload_from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| TlsError::Configuration(e.to_string()))?;

        tracing::info!("TLS certificate reloaded successfully");
        Ok(())
    }

    /// Reload certificates from PEM strings.
    pub async fn reload_from_pem(&self, cert_pem: &str, key_pem: &str) -> TlsResult<()> {
        let cert_bytes = cert_pem.as_bytes().to_vec();
        let key_bytes = key_pem.as_bytes().to_vec();

        self.config
            .reload_from_pem(cert_bytes, key_bytes)
            .await
            .map_err(|e| TlsError::Configuration(e.to_string()))?;

        tracing::info!("TLS certificate reloaded successfully");
        Ok(())
    }
}

/// Generate a self-signed certificate for development/testing.
///
/// This is useful for local development when you don't have a real certificate.
#[cfg(feature = "acme")]
pub fn generate_self_signed_cert(domain: &str) -> TlsResult<(String, String)> {
    use rcgen::{CertificateParams, DnType, KeyPair};

    tracing::info!("Generating self-signed certificate for {}", domain);

    let mut params = CertificateParams::new(vec![domain.to_string()])
        .map_err(|e| TlsError::Configuration(e.to_string()))?;
    params
        .distinguished_name
        .push(DnType::CommonName, domain.to_string());
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Clipper Self-Signed");

    let key_pair = KeyPair::generate().map_err(|e| TlsError::Configuration(e.to_string()))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| TlsError::Configuration(e.to_string()))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    tracing::info!("Self-signed certificate generated successfully");

    Ok((cert_pem, key_pem))
}

/// Shared state for TLS configuration that can be hot-reloaded.
#[cfg(feature = "tls")]
#[derive(Clone)]
pub struct TlsState {
    inner: Arc<TlsStateInner>,
}

#[cfg(feature = "tls")]
struct TlsStateInner {
    config: RustlsConfig,
}

#[cfg(feature = "tls")]
impl TlsState {
    /// Create a new TLS state from a manager.
    pub fn new(manager: &TlsManager) -> Self {
        Self {
            inner: Arc::new(TlsStateInner {
                config: manager.config(),
            }),
        }
    }

    /// Get the rustls configuration.
    pub fn config(&self) -> RustlsConfig {
        self.inner.config.clone()
    }
}

#[cfg(test)]
#[cfg(feature = "acme")]
mod tests {
    use super::*;

    #[test]
    fn test_generate_self_signed_cert() {
        let (cert_pem, key_pem) = generate_self_signed_cert("localhost").unwrap();

        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(cert_pem.contains("END CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));
        assert!(key_pem.contains("END PRIVATE KEY"));
    }
}
