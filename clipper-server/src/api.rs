use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use clipper_indexer::{ClipboardEntry, PagedResult, PagingParams, SearchFilters, ShortUrl};
use serde::{Deserialize, Serialize};

use crate::{error::Result, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/check", get(check_auth))
        .route("/version", get(get_version))
        .route("/clips", post(create_clip))
        .route("/clips/upload", post(upload_clip_file))
        .route("/clips", get(list_clips))
        .route("/clips/search", get(search_clips))
        .route("/clips/{id}", get(get_clip))
        .route("/clips/{id}", put(update_clip))
        .route("/clips/{id}", delete(delete_clip))
        .route("/clips/{id}/file", get(get_clip_file))
        // Short URL endpoints
        .route("/clips/{id}/short-url", post(create_short_url))
        .route("/short/{code}", get(get_short_url_redirect))
        // Public short URL resolver (no auth required)
        .route("/s/{code}", get(resolve_short_url))
        // Static assets for shared clip page (no auth required)
        .route("/shared-assets/{filename}", get(serve_asset))
}

/// Version information response
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    /// Server version string
    pub version: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Number of active WebSocket connections
    pub active_ws_connections: usize,
    /// Configuration info
    pub config: ConfigInfo,
}

/// Configuration information (subset of server config)
#[derive(Debug, Serialize)]
pub struct ConfigInfo {
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
    pub max_upload_size_bytes: u64,
    /// Whether short URL functionality is enabled
    pub short_url_enabled: bool,
    /// Short URL base URL (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_url_base: Option<String>,
}

/// Authentication check response
#[derive(Debug, Serialize)]
pub struct AuthCheckResponse {
    /// Whether authentication is required
    pub auth_required: bool,
}

/// Check if authentication is required
async fn check_auth(State(state): State<AppState>) -> Json<AuthCheckResponse> {
    Json(AuthCheckResponse {
        auth_required: state.config.auth.is_enabled(),
    })
}

/// Get server version and status information
async fn get_version(State(state): State<AppState>) -> Json<VersionResponse> {
    let config = &state.config;

    let config_info = ConfigInfo {
        port: config.server.port,
        tls_enabled: config.tls_available(),
        tls_port: if config.tls_available() {
            Some(config.tls.port)
        } else {
            None
        },
        acme_enabled: config.acme_available(),
        acme_domain: if config.acme_available() {
            config.acme.domain.clone()
        } else {
            None
        },
        cleanup_enabled: config.cleanup.enabled,
        cleanup_interval_mins: if config.cleanup.enabled {
            Some(config.cleanup.interval_hours * 60)
        } else {
            None
        },
        cleanup_retention_days: if config.cleanup.enabled {
            Some(config.cleanup.retention_days)
        } else {
            None
        },
        auth_required: config.auth.is_enabled(),
        max_upload_size_bytes: config.upload.max_size_bytes,
        short_url_enabled: config.short_url.is_enabled(),
        short_url_base: if config.short_url.is_enabled() {
            config.short_url.base_url.clone()
        } else {
            None
        },
    };

    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.uptime_secs(),
        active_ws_connections: state.active_ws_connections(),
        config: config_info,
    })
}

#[derive(Debug, Deserialize)]
struct CreateClipRequest {
    content: String,
    tags: Vec<String>,
    #[serde(default)]
    additional_notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct ClipResponse {
    id: String,
    content: String,
    created_at: String,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_filename: Option<String>,
}

impl From<ClipboardEntry> for ClipResponse {
    fn from(entry: ClipboardEntry) -> Self {
        Self {
            id: entry.id,
            content: entry.content,
            created_at: entry.created_at.to_rfc3339(),
            tags: entry.tags,
            additional_notes: entry.additional_notes,
            file_attachment: entry.file_attachment,
            original_filename: entry.original_filename,
        }
    }
}

#[derive(Debug, Serialize)]
struct PagedClipResponse {
    items: Vec<ClipResponse>,
    total: usize,
    page: usize,
    page_size: usize,
    total_pages: usize,
}

impl From<PagedResult<ClipboardEntry>> for PagedClipResponse {
    fn from(result: PagedResult<ClipboardEntry>) -> Self {
        Self {
            items: result.items.into_iter().map(ClipResponse::from).collect(),
            total: result.total,
            page: result.page,
            page_size: result.page_size,
            total_pages: result.total_pages,
        }
    }
}

async fn create_clip(
    State(state): State<AppState>,
    Json(payload): Json<CreateClipRequest>,
) -> Result<(StatusCode, Json<ClipResponse>)> {
    let entry = state
        .indexer
        .add_entry_from_text(
            payload.content.clone(),
            payload.tags.clone(),
            payload.additional_notes,
        )
        .await?;

    // Notify WebSocket clients
    state.notify_new_clip(entry.id.clone(), entry.content.clone(), entry.tags.clone());

    Ok((StatusCode::CREATED, Json(entry.into())))
}

#[derive(Debug, Deserialize)]
struct ListClipsQuery {
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    tags: Option<String>,
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
}

fn default_page() -> usize {
    1
}

fn default_page_size() -> usize {
    20
}

async fn list_clips(
    State(state): State<AppState>,
    Query(query): Query<ListClipsQuery>,
) -> Result<Json<PagedClipResponse>> {
    let mut filters = SearchFilters::new();

    if let Some(start_date) = query.start_date {
        let start = chrono::DateTime::parse_from_rfc3339(&start_date)
            .map_err(|e| {
                crate::error::ServerError::InvalidInput(format!("Invalid start_date: {}", e))
            })?
            .with_timezone(&chrono::Utc);

        let end = if let Some(end_date) = query.end_date {
            chrono::DateTime::parse_from_rfc3339(&end_date)
                .map_err(|e| {
                    crate::error::ServerError::InvalidInput(format!("Invalid end_date: {}", e))
                })?
                .with_timezone(&chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        filters = filters.with_date_range(start, end);
    }

    if let Some(tags_str) = query.tags {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tags.is_empty() {
            filters = filters.with_tags(tags);
        }
    }

    let paging = PagingParams::new(query.page, query.page_size);
    let result = state.indexer.list_entries(filters, paging).await?;
    Ok(Json(result.into()))
}

#[derive(Debug, Deserialize)]
struct SearchClipsQuery {
    q: String,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    tags: Option<String>,
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
}

async fn search_clips(
    State(state): State<AppState>,
    Query(query): Query<SearchClipsQuery>,
) -> Result<Json<PagedClipResponse>> {
    let mut filters = SearchFilters::new();

    if let Some(start_date) = query.start_date {
        let start = chrono::DateTime::parse_from_rfc3339(&start_date)
            .map_err(|e| {
                crate::error::ServerError::InvalidInput(format!("Invalid start_date: {}", e))
            })?
            .with_timezone(&chrono::Utc);

        let end = if let Some(end_date) = query.end_date {
            chrono::DateTime::parse_from_rfc3339(&end_date)
                .map_err(|e| {
                    crate::error::ServerError::InvalidInput(format!("Invalid end_date: {}", e))
                })?
                .with_timezone(&chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        filters = filters.with_date_range(start, end);
    }

    if let Some(tags_str) = query.tags {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tags.is_empty() {
            filters = filters.with_tags(tags);
        }
    }

    let paging = PagingParams::new(query.page, query.page_size);
    let result = state
        .indexer
        .search_entries(&query.q, filters, paging)
        .await?;
    Ok(Json(result.into()))
}

async fn get_clip(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClipResponse>> {
    let entry = state.indexer.get_entry(&id).await?;
    Ok(Json(entry.into()))
}

#[derive(Debug, Deserialize)]
struct UpdateClipRequest {
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    additional_notes: Option<String>,
}

async fn update_clip(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateClipRequest>,
) -> Result<Json<ClipResponse>> {
    let entry = state
        .indexer
        .update_entry(&id, payload.tags, payload.additional_notes)
        .await?;

    // Notify WebSocket clients
    state.notify_updated_clip(id);

    Ok(Json(entry.into()))
}

async fn delete_clip(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode> {
    state.indexer.delete_entry(&id).await?;

    // Notify WebSocket clients
    state.notify_deleted_clip(id);

    Ok(StatusCode::NO_CONTENT)
}

async fn get_clip_file(State(state): State<AppState>, Path(id): Path<String>) -> Result<Vec<u8>> {
    let entry = state.indexer.get_entry(&id).await?;

    let file_key = entry.file_attachment.ok_or_else(|| {
        crate::error::ServerError::NotFound("No file attachment for this clip".to_string())
    })?;

    let bytes = state.indexer.get_file_content(&file_key).await?;
    Ok(bytes.to_vec())
}

async fn upload_clip_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ClipResponse>)> {
    let mut file_data: Option<bytes::Bytes> = None;
    let mut original_filename: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut additional_notes: Option<String> = None;
    let mut content_override: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| crate::error::ServerError::InvalidInput(format!("Multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                original_filename = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    crate::error::ServerError::InvalidInput(format!("Failed to read file: {}", e))
                })?);
            }
            "tags" => {
                let tags_str = field.text().await.map_err(|e| {
                    crate::error::ServerError::InvalidInput(format!("Failed to read tags: {}", e))
                })?;
                tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
            }
            "additional_notes" => {
                additional_notes = Some(field.text().await.map_err(|e| {
                    crate::error::ServerError::InvalidInput(format!("Failed to read notes: {}", e))
                })?);
            }
            "content" => {
                content_override = Some(field.text().await.map_err(|e| {
                    crate::error::ServerError::InvalidInput(format!("Failed to read content: {}", e))
                })?);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate required fields
    let file_data = file_data
        .ok_or_else(|| crate::error::ServerError::InvalidInput("Missing file field".to_string()))?;

    // Check file size limit
    let max_size = state.config.upload.max_size_bytes;
    if file_data.len() as u64 > max_size {
        let max_size_mb = max_size as f64 / (1024.0 * 1024.0);
        let file_size_mb = file_data.len() as f64 / (1024.0 * 1024.0);
        return Err(crate::error::ServerError::PayloadTooLarge(format!(
            "File size ({:.2} MB) exceeds maximum allowed size ({:.2} MB)",
            file_size_mb, max_size_mb
        )));
    }

    let original_filename = original_filename.unwrap_or_else(|| "uploaded_file".to_string());

    // Create entry from file content with optional content override
    let entry = state
        .indexer
        .add_entry_from_file_content_with_override(
            file_data,
            original_filename.clone(),
            tags.clone(),
            additional_notes,
            content_override,
        )
        .await?;

    // Notify WebSocket clients
    state.notify_new_clip(entry.id.clone(), entry.content.clone(), entry.tags.clone());

    Ok((StatusCode::CREATED, Json(entry.into())))
}

// ==================== Short URL Endpoints ====================

#[derive(Debug, Deserialize)]
struct CreateShortUrlRequest {
    /// Optional expiration time in hours (overrides server default)
    #[serde(default)]
    expires_in_hours: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ShortUrlResponse {
    id: String,
    clip_id: String,
    short_code: String,
    full_url: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
}

impl ShortUrlResponse {
    fn from_short_url(short_url: ShortUrl, base_url: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        Self {
            id: short_url.id,
            clip_id: short_url.clip_id,
            short_code: short_url.short_code.clone(),
            full_url: format!("{}/s/{}", base, short_url.short_code),
            created_at: short_url.created_at.to_rfc3339(),
            expires_at: short_url.expires_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Create a short URL for a clip
async fn create_short_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateShortUrlRequest>,
) -> Result<(StatusCode, Json<ShortUrlResponse>)> {
    // Check if short URL feature is enabled
    if !state.config.short_url.is_enabled() {
        return Err(crate::error::ServerError::FeatureDisabled(
            "Short URL functionality is disabled. Set CLIPPER_SHORT_URL_BASE to enable.".to_string(),
        ));
    }

    // Calculate expiration time
    let expires_at = match payload.expires_in_hours {
        Some(0) => None, // Explicit no expiration
        Some(hours) => Some(chrono::Utc::now() + chrono::Duration::hours(hours as i64)),
        None => {
            // Use server default
            if state.config.short_url.default_expiration_hours > 0 {
                Some(
                    chrono::Utc::now()
                        + chrono::Duration::hours(
                            state.config.short_url.default_expiration_hours as i64,
                        ),
                )
            } else {
                None
            }
        }
    };

    let short_url = state.indexer.create_short_url(&id, expires_at).await?;

    let base_url = state.config.short_url.base_url.as_ref().unwrap();
    let response = ShortUrlResponse::from_short_url(short_url, base_url);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Redirect from short URL to the clip
/// This endpoint returns the clip ID which can be used to fetch the clip content
async fn get_short_url_redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ShortUrlRedirectResponse>> {
    let short_url = state.indexer.get_short_url(&code).await?;

    Ok(Json(ShortUrlRedirectResponse {
        clip_id: short_url.clip_id,
        short_code: short_url.short_code,
    }))
}

#[derive(Debug, Serialize)]
struct ShortUrlRedirectResponse {
    clip_id: String,
    short_code: String,
}

// ==================== Public Short URL Resolver ====================

/// JSON response for short URL content (minimal metadata)
#[derive(Debug, Serialize)]
struct ShortUrlContentResponse {
    id: String,
    content: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_filename: Option<String>,
}

impl From<ClipboardEntry> for ShortUrlContentResponse {
    fn from(entry: ClipboardEntry) -> Self {
        // For file attachments, show only the filename (not the full path/key)
        let content = if entry.file_attachment.is_some() {
            entry
                .original_filename
                .clone()
                .unwrap_or_else(|| entry.content.clone())
        } else {
            entry.content
        };

        Self {
            id: entry.id,
            content,
            created_at: entry.created_at.to_rfc3339(),
            file_attachment: entry.file_attachment,
            original_filename: entry.original_filename,
        }
    }
}

/// Query parameters for short URL resolution
#[derive(Debug, Deserialize)]
struct ResolveShortUrlQuery {
    /// Override content type (useful for download links in HTML)
    #[serde(default)]
    accept: Option<String>,
}

/// Resolve short URL and return content based on Accept header or query parameter
///
/// Content negotiation (via Accept header or ?accept= query parameter):
/// - `text/html`: HTML representation of the clip
/// - `text/plain`: Plain text content
/// - `application/json`: JSON with minimal metadata (no tags/notes)
/// - `application/octet-stream`: File attachment if exists, otherwise error
async fn resolve_short_url(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(query): Query<ResolveShortUrlQuery>,
    headers: HeaderMap,
) -> Result<Response> {
    // Get short URL and check if expired
    let short_url = state.indexer.get_short_url(&code).await?;

    // Get the clip
    let entry = state.indexer.get_entry(&short_url.clip_id).await?;

    // Determine content type from query parameter first, then Accept header
    let accept = query.accept.as_deref().unwrap_or_else(|| {
        headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
    });

    // Parse accept header and find best match
    let response = if accept.contains("application/octet-stream") {
        // Return file attachment
        if let Some(file_key) = &entry.file_attachment {
            let bytes = state.indexer.get_file_content(file_key).await?;
            let filename = entry
                .original_filename
                .as_deref()
                .unwrap_or("attachment");

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(Body::from(bytes.to_vec()))
                .unwrap()
        } else {
            return Err(crate::error::ServerError::NotFound(
                "This clip has no file attachment".to_string(),
            ));
        }
    } else if accept.contains("application/json") {
        // Return JSON with minimal metadata
        let response: ShortUrlContentResponse = entry.into();
        Json(response).into_response()
    } else if accept.contains("text/plain") {
        // Return plain text content
        let content = if entry.file_attachment.is_some() {
            entry
                .original_filename
                .unwrap_or_else(|| entry.content.clone())
        } else {
            entry.content
        };

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(content))
            .unwrap()
    } else {
        // Default to HTML representation
        let original_filename = entry.original_filename.clone();
        let is_image = original_filename
            .as_ref()
            .map(|f| is_image_file(f))
            .unwrap_or(false);

        let content = if entry.file_attachment.is_some() {
            original_filename
                .clone()
                .unwrap_or_else(|| entry.content.clone())
        } else {
            entry.content.clone()
        };

        // Store original content for copy button (unescaped)
        let original_content = if entry.file_attachment.is_some() {
            original_filename
                .clone()
                .unwrap_or_else(|| entry.content.clone())
        } else {
            entry.content.clone()
        };

        // Build image HTML if it's an image file
        let image_html = if is_image {
            format!(
                r#"<div class="image-container"><img src="/s/{}?accept=application/octet-stream" alt="{}" class="shared-image" /></div>"#,
                code,
                html_escape(&original_filename.clone().unwrap_or_default())
            )
        } else {
            String::new()
        };

        // Build download link if file attachment exists
        // Use id="download-btn" so JavaScript can localize the text
        let download_link = if entry.file_attachment.is_some() {
            format!(
                r#"<a class="btn" id="download-btn" href="/s/{}?accept=application/octet-stream">Download File</a>"#,
                code
            )
        } else {
            String::new()
        };

        // Build expiration info
        let (expiration_html, expires_at_json) = match short_url.expires_at {
            Some(expires_at) => (
                format!(
                    r#"Expires: <span class="expires" title="{}">loading...</span>"#,
                    expires_at.format("%Y-%m-%d %H:%M:%S UTC")
                ),
                serde_json::to_string(&expires_at.to_rfc3339())
                    .unwrap_or_else(|_| "null".to_string()),
            ),
            None => (
                r#"Expires: <span class="no-expiry">never</span>"#.to_string(),
                "null".to_string(),
            ),
        };

        // Check if this is a file attachment
        let is_file = entry.file_attachment.is_some();

        // Load template and substitute placeholders
        let html = include_str!("templates/shared_clip.html")
            .replace("{{BUILD_VERSION}}", build_version())
            .replace("{{CONTENT}}", &html_escape(&content))
            .replace("{{IMAGE_HTML}}", &image_html)
            .replace("{{IS_IMAGE}}", if is_image { "true" } else { "false" })
            .replace("{{IS_FILE}}", if is_file { "true" } else { "false" })
            .replace("{{DOWNLOAD_LINK}}", &download_link)
            .replace(
                "{{ORIGINAL_CONTENT_JSON}}",
                &serde_json::to_string(&original_content).unwrap_or_else(|_| "\"\"".to_string()),
            )
            .replace("{{EXPIRATION_HTML}}", &expiration_html)
            .replace("{{EXPIRES_AT_JSON}}", &expires_at_json);

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(html))
            .unwrap()
    };

    Ok(response)
}

/// Simple HTML escaping for content display
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Check if a filename has an image extension
fn is_image_file(filename: &str) -> bool {
    const IMAGE_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".svg"];
    let lower = filename.to_lowercase();
    IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Get build timestamp for cache busting
fn build_version() -> &'static str {
    env!("BUILD_TIMESTAMP")
}

// ==================== Static Assets ====================

/// Serve static assets for shared clip page with browser caching
/// Supports versioned filenames like shared_clip-1733318400.css
async fn serve_asset(Path(filename): Path<String>) -> Result<Response> {
    // Extract base name and extension, stripping version suffix
    // e.g., "shared_clip-1733318400.css" -> ("shared_clip", "css")
    let (content, content_type) = if filename.starts_with("shared_clip-") && filename.ends_with(".css") {
        (
            include_str!("assets/shared_clip.css"),
            "text/css; charset=utf-8",
        )
    } else if filename.starts_with("shared_clip-") && filename.ends_with(".js") {
        (
            include_str!("assets/shared_clip.js"),
            "application/javascript; charset=utf-8",
        )
    } else if filename == "favicon.svg" {
        (
            include_str!("assets/favicon.svg"),
            "image/svg+xml",
        )
    } else {
        return Err(crate::error::ServerError::NotFound(format!(
            "Asset not found: {}",
            filename
        )));
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        // Cache for 1 year (assets are versioned in filename)
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(content))
        .unwrap())
}
