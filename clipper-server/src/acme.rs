//! ACME (Automatic Certificate Management Environment) integration.
//!
//! This module provides automatic TLS certificate provisioning via Let's Encrypt
//! using the ACME protocol. It handles:
//! - Account registration and key management
//! - Certificate ordering and validation (HTTP-01 challenge)
//! - Certificate storage and renewal

#[cfg(feature = "acme")]
use std::sync::Arc;
#[cfg(feature = "acme")]
use std::time::Duration;

#[cfg(feature = "acme")]
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt,
    NewAccount, NewOrder, OrderStatus, RetryPolicy,
};
#[cfg(feature = "acme")]
use thiserror::Error;
#[cfg(feature = "acme")]
use tokio::sync::RwLock;
#[cfg(feature = "acme")]
use x509_parser::prelude::*;

#[cfg(feature = "acme")]
use crate::cert_storage::{CertStorage, StorageError};
#[cfg(feature = "acme")]
use crate::config::AcmeConfig;

/// Errors that can occur during ACME operations.
#[cfg(feature = "acme")]
#[derive(Error, Debug)]
pub enum AcmeError {
    #[error("ACME protocol error: {0}")]
    Protocol(String),

    #[error("Challenge failed: {0}")]
    ChallengeFailed(String),

    #[error("Order failed: {0}")]
    OrderFailed(String),

    #[error("Certificate generation error: {0}")]
    CertificateGeneration(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Certificate parsing error: {0}")]
    CertificateParsing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "acme")]
pub type AcmeResult<T> = Result<T, AcmeError>;

/// Pending HTTP-01 challenge token and authorization.
#[cfg(feature = "acme")]
#[derive(Clone)]
pub struct PendingChallenge {
    pub token: String,
    pub key_authorization: String,
}

/// ACME certificate manager.
///
/// Handles certificate provisioning and renewal via Let's Encrypt.
#[cfg(feature = "acme")]
pub struct AcmeManager {
    config: AcmeConfig,
    storage: Box<dyn CertStorage>,
    account: RwLock<Option<Account>>,
    /// Pending challenges for HTTP-01 validation.
    /// Maps token -> key_authorization
    pending_challenges: Arc<RwLock<std::collections::HashMap<String, String>>>,
}

#[cfg(feature = "acme")]
impl AcmeManager {
    /// Create a new ACME manager.
    pub fn new(config: AcmeConfig, storage: Box<dyn CertStorage>) -> Self {
        Self {
            config,
            storage,
            account: RwLock::new(None),
            pending_challenges: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get the ACME directory URL.
    fn directory_url(&self) -> String {
        if self.config.staging {
            LetsEncrypt::Staging.url().to_owned()
        } else {
            LetsEncrypt::Production.url().to_owned()
        }
    }

    /// Get or create an ACME account.
    pub async fn get_or_create_account(&self) -> AcmeResult<Account> {
        // Check if we already have an account loaded
        {
            let account = self.account.read().await;
            if let Some(ref acc) = *account {
                return Ok(acc.clone());
            }
        }

        // Try to load existing account from storage
        if let Some(credentials_json) = self.storage.load_account_key()? {
            let credentials: AccountCredentials = serde_json::from_str(&credentials_json)
                .map_err(|e| AcmeError::Storage(StorageError::Serialization(e.to_string())))?;

            let account = Account::builder()
                .map_err(|e| AcmeError::Protocol(e.to_string()))?
                .from_credentials(credentials)
                .await
                .map_err(|e| AcmeError::Protocol(e.to_string()))?;

            tracing::info!("Loaded existing ACME account");

            let mut acc = self.account.write().await;
            *acc = Some(account.clone());
            return Ok(account);
        }

        // Create a new account
        let contact_email = self.config.contact_email.as_ref().ok_or_else(|| {
            AcmeError::Configuration("Contact email is required for ACME".to_string())
        })?;

        tracing::info!(
            "Creating new ACME account with email {} ({})",
            contact_email,
            if self.config.staging {
                "staging"
            } else {
                "production"
            }
        );

        let contact = format!("mailto:{}", contact_email);
        let (account, credentials) = Account::builder()
            .map_err(|e| AcmeError::Protocol(e.to_string()))?
            .create(
                &NewAccount {
                    contact: &[&contact],
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                self.directory_url(),
                None,
            )
            .await
            .map_err(|e| AcmeError::Protocol(e.to_string()))?;

        // Store credentials
        let credentials_json = serde_json::to_string(&credentials)
            .map_err(|e| AcmeError::Storage(StorageError::Serialization(e.to_string())))?;
        self.storage.store_account_key(&credentials_json)?;

        tracing::info!("Created and stored new ACME account");

        let mut acc = self.account.write().await;
        *acc = Some(account.clone());
        Ok(account)
    }

    /// Get the pending challenges map for the HTTP-01 challenge handler.
    pub fn pending_challenges(&self) -> Arc<RwLock<std::collections::HashMap<String, String>>> {
        self.pending_challenges.clone()
    }

    /// Provision a certificate for the configured domain.
    ///
    /// Returns (certificate_pem, private_key_pem).
    pub async fn provision_certificate(&self) -> AcmeResult<(String, String)> {
        let domain = self.config.domain.as_ref().ok_or_else(|| {
            AcmeError::Configuration("Domain is required for certificate provisioning".to_string())
        })?;

        tracing::info!("Provisioning certificate for {}", domain);

        // Check if we have a valid cached certificate
        if let Some((cert_pem, key_pem)) = self.load_cached_certificate(domain).await? {
            if !self.certificate_needs_renewal(&cert_pem)? {
                tracing::info!("Using cached certificate for {}", domain);
                return Ok((cert_pem, key_pem));
            }
            tracing::info!("Cached certificate needs renewal");
        }

        // Get or create account
        let account = self.get_or_create_account().await?;

        // Create order
        let identifiers = vec![Identifier::Dns(domain.to_string())];
        let mut order = account
            .new_order(&NewOrder::new(identifiers.as_slice()))
            .await
            .map_err(|e| AcmeError::Protocol(e.to_string()))?;

        // Process authorizations and HTTP-01 challenges
        let mut authorizations = order.authorizations();
        while let Some(result) = authorizations.next().await {
            let mut authz = result.map_err(|e| AcmeError::Protocol(e.to_string()))?;

            match authz.status {
                AuthorizationStatus::Valid => continue,
                AuthorizationStatus::Pending => {}
                _ => {
                    return Err(AcmeError::ChallengeFailed(format!(
                        "Unexpected authorization status: {:?}",
                        authz.status
                    )));
                }
            }

            let mut challenge = authz.challenge(ChallengeType::Http01).ok_or_else(|| {
                AcmeError::ChallengeFailed("No HTTP-01 challenge found".to_string())
            })?;

            let token = challenge.identifier().to_string();
            let key_auth = challenge.key_authorization().as_str().to_string();

            tracing::info!("Setting up HTTP-01 challenge for token: {}", token);

            // Store the challenge for the HTTP handler
            {
                let mut challenges = self.pending_challenges.write().await;
                challenges.insert(token.clone(), key_auth.clone());
            }

            // Notify ACME server that we're ready
            challenge
                .set_ready()
                .await
                .map_err(|e| AcmeError::Protocol(e.to_string()))?;

            // Note: Do NOT remove the challenge here - the ACME server will validate asynchronously
            // and needs to be able to fetch the key authorization from our HTTP endpoint
        }

        // Wait for order to be ready using poll_ready with retry policy
        let status = order
            .poll_ready(&RetryPolicy::default())
            .await
            .map_err(|e| AcmeError::Protocol(e.to_string()))?;

        // Clean up pending challenges after validation is complete
        {
            let mut challenges = self.pending_challenges.write().await;
            challenges.clear();
        }

        if status != OrderStatus::Ready {
            return Err(AcmeError::OrderFailed(format!(
                "Unexpected order status: {:?}",
                status
            )));
        }

        // Finalize order - this generates the private key and returns it
        let key_pem = order
            .finalize()
            .await
            .map_err(|e| AcmeError::Protocol(e.to_string()))?;

        // Get the certificate chain
        let cert_chain = order
            .poll_certificate(&RetryPolicy::default())
            .await
            .map_err(|e| AcmeError::Protocol(e.to_string()))?;

        // Store certificate
        self.storage.store_certificate(domain, &cert_chain)?;
        self.storage.store_private_key(domain, &key_pem)?;

        tracing::info!("Certificate provisioned and stored for {}", domain);

        Ok((cert_chain, key_pem))
    }

    /// Load cached certificate from storage.
    async fn load_cached_certificate(&self, domain: &str) -> AcmeResult<Option<(String, String)>> {
        if !self.storage.has_certificate(domain)? {
            return Ok(None);
        }

        let cert_pem = self
            .storage
            .load_certificate(domain)?
            .ok_or_else(|| AcmeError::Storage(StorageError::NotFound("Certificate".to_string())))?;

        let key_pem = self
            .storage
            .load_private_key(domain)?
            .ok_or_else(|| AcmeError::Storage(StorageError::NotFound("Private key".to_string())))?;

        Ok(Some((cert_pem, key_pem)))
    }

    /// Check if a certificate needs renewal (less than 30 days validity).
    fn certificate_needs_renewal(&self, cert_pem: &str) -> AcmeResult<bool> {
        // Parse the first certificate from the PEM chain
        let (_, pem) = x509_parser::pem::parse_x509_pem(cert_pem.as_bytes())
            .map_err(|e| AcmeError::CertificateParsing(e.to_string()))?;

        let (_, cert) = X509Certificate::from_der(&pem.contents)
            .map_err(|e| AcmeError::CertificateParsing(e.to_string()))?;

        let not_after = cert.validity().not_after;
        let now = chrono::Utc::now();

        // Convert ASN1Time to timestamp
        let expiry_timestamp = not_after.timestamp();
        let now_timestamp = now.timestamp();

        // Check if less than 30 days remaining
        let days_remaining = (expiry_timestamp - now_timestamp) / 86400;
        let needs_renewal = days_remaining < 30;

        if needs_renewal {
            tracing::info!(
                "Certificate expires in {} days, renewal needed",
                days_remaining
            );
        } else {
            tracing::debug!(
                "Certificate expires in {} days, no renewal needed",
                days_remaining
            );
        }

        Ok(needs_renewal)
    }

    /// Renew the certificate if needed.
    pub async fn renew_if_needed(&self) -> AcmeResult<Option<(String, String)>> {
        let domain = match &self.config.domain {
            Some(d) => d.clone(),
            None => return Ok(None),
        };

        if let Some((cert_pem, key_pem)) = self.load_cached_certificate(&domain).await? {
            if self.certificate_needs_renewal(&cert_pem)? {
                tracing::info!("Renewing certificate for {}", domain);
                // Delete old certificate and provision new one
                self.storage.delete_certificate(&domain)?;
                return Ok(Some(self.provision_certificate().await?));
            }
            return Ok(Some((cert_pem, key_pem)));
        }

        Ok(None)
    }
}

/// Axum handler for ACME HTTP-01 challenges.
///
/// This handler responds to requests at `/.well-known/acme-challenge/{token}`
/// with the corresponding key authorization.
#[cfg(feature = "acme")]
pub mod challenge_handler {
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// State for the ACME challenge handler.
    #[derive(Clone)]
    pub struct AcmeChallengeState {
        pub challenges: Arc<RwLock<HashMap<String, String>>>,
    }

    /// Handle ACME HTTP-01 challenge requests.
    pub async fn handle_challenge(
        State(state): State<AcmeChallengeState>,
        Path(token): Path<String>,
    ) -> impl IntoResponse {
        let challenges = state.challenges.read().await;

        if let Some(key_auth) = challenges.get(&token) {
            tracing::debug!("Responding to ACME challenge for token: {}", token);
            (StatusCode::OK, key_auth.clone())
        } else {
            tracing::warn!("Unknown ACME challenge token: {}", token);
            (StatusCode::NOT_FOUND, "Challenge not found".to_string())
        }
    }
}

/// Background task for certificate renewal.
#[cfg(feature = "acme")]
pub async fn certificate_renewal_task(
    manager: Arc<AcmeManager>,
    on_renewal: impl Fn(String, String) + Send + Sync + 'static,
) {
    let check_interval = Duration::from_secs(24 * 60 * 60); // Check daily

    loop {
        tokio::time::sleep(check_interval).await;

        match manager.renew_if_needed().await {
            Ok(Some((cert_pem, key_pem))) => {
                tracing::info!("Certificate renewed or loaded successfully");
                on_renewal(cert_pem, key_pem);
            }
            Ok(None) => {
                tracing::debug!("No certificate renewal needed");
            }
            Err(e) => {
                tracing::error!("Certificate renewal check failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "acme")]
mod tests {
    use super::*;

    #[test]
    fn test_directory_urls() {
        assert!(LetsEncrypt::Production.url().contains("acme-v02"));
        assert!(LetsEncrypt::Staging.url().contains("staging"));
    }
}
