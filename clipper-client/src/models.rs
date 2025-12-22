use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    /// Optional language identifier for the clip content (e.g., "en", "zh", "rust", "python")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Highlighted content with search terms wrapped by highlight markers.
    /// Only present in search results when highlight params are provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClipRequest {
    pub content: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClipRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl SearchFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_start_date(mut self, date: DateTime<Utc>) -> Self {
        self.start_date = Some(date);
        self
    }

    pub fn with_end_date(mut self, date: DateTime<Utc>) -> Self {
        self.end_date = Some(date);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClipNotification {
    NewClip {
        id: String,
        content: String,
        tags: Vec<String>,
    },
    UpdatedClip {
        id: String,
    },
    DeletedClip {
        id: String,
    },
    ClipsCleanedUp {
        ids: Vec<String>,
        count: usize,
    },
}

/// WebSocket authentication request message sent by client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsAuthRequest {
    #[serde(rename = "auth")]
    Auth { token: String },
}

/// WebSocket authentication response message from server
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum WsAuthResponse {
    #[serde(rename = "auth_success")]
    AuthSuccess,
    #[serde(rename = "auth_error")]
    AuthError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult {
    pub items: Vec<Clip>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

/// Server configuration information returned by /version API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigInfo {
    /// HTTP listening port
    pub port: u16,
    /// Whether TLS is enabled
    pub tls_enabled: bool,
    /// HTTPS port (if TLS is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_port: Option<u16>,
    /// Whether ACME is enabled
    pub acme_enabled: bool,
    /// ACME domain (if ACME is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acme_domain: Option<String>,
    /// Whether auto-cleanup is enabled
    pub cleanup_enabled: bool,
    /// Auto-cleanup interval in minutes (if cleanup is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_interval_mins: Option<u32>,
    /// Auto-cleanup retention in days (if cleanup is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_retention_days: Option<u32>,
    /// Whether authentication is required
    pub auth_required: bool,
    /// Maximum upload size in bytes
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size_bytes: u64,
    /// Whether short URL functionality is enabled
    #[serde(default)]
    pub short_url_enabled: bool,
    /// Short URL base URL (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_url_base: Option<String>,
    /// Short URL default expiration in hours (if enabled, 0 = no expiration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_url_expiration_hours: Option<u32>,
    /// Whether export/import functionality is enabled
    #[serde(default)]
    pub export_import_enabled: bool,
}

fn default_max_upload_size() -> u64 {
    10 * 1024 * 1024 // 10MB default
}

/// Server version and status information returned by /version API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server version string
    pub version: String,
    /// Index schema version (indicates available features)
    #[serde(default)]
    pub index_version: i64,
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Number of active WebSocket connections
    pub active_ws_connections: usize,
    /// Configuration info
    pub config: ServerConfigInfo,
}

/// Request to create a short URL for a clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShortUrlRequest {
    /// Optional expiration time in hours (overrides server default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_hours: Option<u32>,
}

/// Short URL response from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrl {
    /// Short URL record ID
    pub id: String,
    /// Associated clip ID
    pub clip_id: String,
    /// Short code (used in URL path)
    pub short_code: String,
    /// Full short URL
    pub full_url: String,
    /// Creation timestamp (RFC3339)
    pub created_at: String,
    /// Expiration timestamp (RFC3339), if set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of clips imported
    pub imported_count: usize,
    /// Number of clips skipped (already existed)
    pub skipped_count: usize,
    /// Number of file attachments imported
    pub attachments_imported: usize,
    /// IDs of newly imported clips
    pub imported_ids: Vec<String>,
    /// IDs of skipped clips (duplicates)
    pub skipped_ids: Vec<String>,
}

/// A tag that has been used by clip entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag record ID
    pub id: String,
    /// Tag text
    pub text: String,
    /// Creation timestamp (RFC3339)
    pub created_at: String,
}

/// Paged result for tag queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedTagResult {
    pub items: Vec<Tag>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}
