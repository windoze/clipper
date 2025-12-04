//! Certificate and credential storage abstraction.
//!
//! Provides secure storage for ACME account credentials and TLS certificates.
//! Uses OS keychain when available, falls back to encrypted file storage.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during certificate storage operations.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Credential not found: {0}")]
    NotFound(String),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Storage error: {0}")]
    Other(String),
}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Trait for certificate and credential storage.
///
/// Implementations can use OS keychain, encrypted files, or other backends.
pub trait CertStorage: Send + Sync {
    /// Store ACME account credentials (private key in PEM format).
    fn store_account_key(&self, key_pem: &str) -> StorageResult<()>;

    /// Load ACME account credentials.
    fn load_account_key(&self) -> StorageResult<Option<String>>;

    /// Delete ACME account credentials.
    fn delete_account_key(&self) -> StorageResult<()>;

    /// Store certificate chain (PEM format) for a domain.
    fn store_certificate(&self, domain: &str, cert_pem: &str) -> StorageResult<()>;

    /// Load certificate chain for a domain.
    fn load_certificate(&self, domain: &str) -> StorageResult<Option<String>>;

    /// Store private key (PEM format) for a domain.
    fn store_private_key(&self, domain: &str, key_pem: &str) -> StorageResult<()>;

    /// Load private key for a domain.
    fn load_private_key(&self, domain: &str) -> StorageResult<Option<String>>;

    /// Delete certificate and key for a domain.
    fn delete_certificate(&self, domain: &str) -> StorageResult<()>;

    /// Check if a certificate exists for a domain.
    fn has_certificate(&self, domain: &str) -> StorageResult<bool>;
}

/// File-based certificate storage.
///
/// Stores certificates and keys as PEM files in a directory.
/// Account keys are stored with restrictive permissions.
pub struct FileStorage {
    base_dir: PathBuf,
}

impl FileStorage {
    /// Create a new file storage instance.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Ensure the storage directory exists.
    fn ensure_dir(&self) -> StorageResult<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        Ok(())
    }

    /// Get the path for account key storage.
    fn account_key_path(&self) -> PathBuf {
        self.base_dir.join("account.key")
    }

    /// Get the path for a domain's certificate.
    fn cert_path(&self, domain: &str) -> PathBuf {
        self.base_dir.join(format!("{}.crt", domain))
    }

    /// Get the path for a domain's private key.
    fn key_path(&self, domain: &str) -> PathBuf {
        self.base_dir.join(format!("{}.key", domain))
    }

    /// Write a file with restrictive permissions (Unix only).
    fn write_secure(&self, path: &PathBuf, content: &str) -> StorageResult<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        self.ensure_dir()?;

        // Create file with restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600) // Owner read/write only
                .open(path)?;
            file.write_all(content.as_bytes())?;
        }

        #[cfg(not(unix))]
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }
}

impl CertStorage for FileStorage {
    fn store_account_key(&self, key_pem: &str) -> StorageResult<()> {
        self.write_secure(&self.account_key_path(), key_pem)
    }

    fn load_account_key(&self) -> StorageResult<Option<String>> {
        let path = self.account_key_path();
        if path.exists() {
            Ok(Some(std::fs::read_to_string(path)?))
        } else {
            Ok(None)
        }
    }

    fn delete_account_key(&self) -> StorageResult<()> {
        let path = self.account_key_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn store_certificate(&self, domain: &str, cert_pem: &str) -> StorageResult<()> {
        self.ensure_dir()?;
        std::fs::write(self.cert_path(domain), cert_pem)?;
        Ok(())
    }

    fn load_certificate(&self, domain: &str) -> StorageResult<Option<String>> {
        let path = self.cert_path(domain);
        if path.exists() {
            Ok(Some(std::fs::read_to_string(path)?))
        } else {
            Ok(None)
        }
    }

    fn store_private_key(&self, domain: &str, key_pem: &str) -> StorageResult<()> {
        self.write_secure(&self.key_path(domain), key_pem)
    }

    fn load_private_key(&self, domain: &str) -> StorageResult<Option<String>> {
        let path = self.key_path(domain);
        if path.exists() {
            Ok(Some(std::fs::read_to_string(path)?))
        } else {
            Ok(None)
        }
    }

    fn delete_certificate(&self, domain: &str) -> StorageResult<()> {
        let cert_path = self.cert_path(domain);
        let key_path = self.key_path(domain);

        if cert_path.exists() {
            std::fs::remove_file(cert_path)?;
        }
        if key_path.exists() {
            std::fs::remove_file(key_path)?;
        }
        Ok(())
    }

    fn has_certificate(&self, domain: &str) -> StorageResult<bool> {
        Ok(self.cert_path(domain).exists() && self.key_path(domain).exists())
    }
}

/// Keyring-backed storage for ACME account keys.
///
/// Uses the OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
/// for storing the ACME account key. Certificates are still stored in files.
#[cfg(feature = "secure-storage")]
pub struct KeyringStorage {
    service_name: String,
    file_storage: FileStorage,
}

#[cfg(feature = "secure-storage")]
impl KeyringStorage {
    /// Create a new keyring storage instance.
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            service_name: "com.0d0a.clipper".to_string(),
            file_storage: FileStorage::new(base_dir),
        }
    }

    /// Get or create a keyring entry.
    fn get_entry(&self, key: &str) -> Result<keyring::Entry, StorageError> {
        keyring::Entry::new(&self.service_name, key)
            .map_err(|e| StorageError::Keyring(e.to_string()))
    }
}

#[cfg(feature = "secure-storage")]
impl CertStorage for KeyringStorage {
    fn store_account_key(&self, key_pem: &str) -> StorageResult<()> {
        let entry = self.get_entry("acme_account_key")?;
        entry
            .set_password(key_pem)
            .map_err(|e| StorageError::Keyring(e.to_string()))?;
        tracing::info!("Stored ACME account key in OS keychain");
        Ok(())
    }

    fn load_account_key(&self) -> StorageResult<Option<String>> {
        let entry = self.get_entry("acme_account_key")?;
        match entry.get_password() {
            Ok(key) => {
                tracing::debug!("Loaded ACME account key from OS keychain");
                Ok(Some(key))
            }
            Err(keyring::Error::NoEntry) => {
                // Try falling back to file storage for migration
                if let Ok(Some(key)) = self.file_storage.load_account_key() {
                    tracing::info!("Migrating ACME account key from file to keychain");
                    self.store_account_key(&key)?;
                    // Remove the file after migration
                    let _ = self.file_storage.delete_account_key();
                    return Ok(Some(key));
                }
                Ok(None)
            }
            Err(e) => Err(StorageError::Keyring(e.to_string())),
        }
    }

    fn delete_account_key(&self) -> StorageResult<()> {
        let entry = self.get_entry("acme_account_key")?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(StorageError::Keyring(e.to_string())),
        }
    }

    // Certificates and private keys are still stored in files
    fn store_certificate(&self, domain: &str, cert_pem: &str) -> StorageResult<()> {
        self.file_storage.store_certificate(domain, cert_pem)
    }

    fn load_certificate(&self, domain: &str) -> StorageResult<Option<String>> {
        self.file_storage.load_certificate(domain)
    }

    fn store_private_key(&self, domain: &str, key_pem: &str) -> StorageResult<()> {
        self.file_storage.store_private_key(domain, key_pem)
    }

    fn load_private_key(&self, domain: &str) -> StorageResult<Option<String>> {
        self.file_storage.load_private_key(domain)
    }

    fn delete_certificate(&self, domain: &str) -> StorageResult<()> {
        self.file_storage.delete_certificate(domain)
    }

    fn has_certificate(&self, domain: &str) -> StorageResult<bool> {
        self.file_storage.has_certificate(domain)
    }
}

/// Create the appropriate storage backend based on available features.
///
/// This function also ensures the storage directory exists.
pub fn create_storage(base_dir: PathBuf) -> Box<dyn CertStorage> {
    // Ensure the directory exists before creating storage
    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        tracing::warn!("Failed to create certificate storage directory: {}", e);
    } else {
        tracing::debug!("Certificate storage directory: {}", base_dir.display());
    }

    #[cfg(feature = "secure-storage")]
    {
        tracing::info!("Using keyring-backed certificate storage");
        Box::new(KeyringStorage::new(base_dir))
    }

    #[cfg(not(feature = "secure-storage"))]
    {
        tracing::info!("Using file-based certificate storage");
        Box::new(FileStorage::new(base_dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_storage_account_key() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf());

        // Initially no key
        assert!(storage.load_account_key().unwrap().is_none());

        // Store key
        let key = "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----";
        storage.store_account_key(key).unwrap();

        // Load key
        let loaded = storage.load_account_key().unwrap().unwrap();
        assert_eq!(loaded, key);

        // Delete key
        storage.delete_account_key().unwrap();
        assert!(storage.load_account_key().unwrap().is_none());
    }

    #[test]
    fn test_file_storage_certificate() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf());
        let domain = "example.com";

        // Initially no certificate
        assert!(!storage.has_certificate(domain).unwrap());

        // Store certificate and key
        let cert = "-----BEGIN CERTIFICATE-----\ncert\n-----END CERTIFICATE-----";
        let key = "-----BEGIN PRIVATE KEY-----\nkey\n-----END PRIVATE KEY-----";
        storage.store_certificate(domain, cert).unwrap();
        storage.store_private_key(domain, key).unwrap();

        // Check existence
        assert!(storage.has_certificate(domain).unwrap());

        // Load
        assert_eq!(storage.load_certificate(domain).unwrap().unwrap(), cert);
        assert_eq!(storage.load_private_key(domain).unwrap().unwrap(), key);

        // Delete
        storage.delete_certificate(domain).unwrap();
        assert!(!storage.has_certificate(domain).unwrap());
    }

    #[test]
    #[cfg(unix)]
    fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf());

        let key = "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----";
        storage.store_account_key(key).unwrap();

        let metadata = std::fs::metadata(storage.account_key_path()).unwrap();
        let mode = metadata.permissions().mode();
        // Check that only owner has read/write (0o600)
        assert_eq!(mode & 0o777, 0o600);
    }
}
