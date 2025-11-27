use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use clipper_indexer::{ClipboardEntry, PagedResult, PagingParams, SearchFilters};
use serde::{Deserialize, Serialize};

use crate::{error::Result, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/clips", post(create_clip))
        .route("/clips/upload", post(upload_clip_file))
        .route("/clips", get(list_clips))
        .route("/clips/search", get(search_clips))
        .route("/clips/{id}", get(get_clip))
        .route("/clips/{id}", put(update_clip))
        .route("/clips/{id}", delete(delete_clip))
        .route("/clips/{id}/file", get(get_clip_file))
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
        let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        filters = filters.with_tags(tags);
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
        let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        filters = filters.with_tags(tags);
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
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate required fields
    let file_data = file_data
        .ok_or_else(|| crate::error::ServerError::InvalidInput("Missing file field".to_string()))?;

    let original_filename = original_filename.unwrap_or_else(|| "uploaded_file".to_string());

    // Create entry from file content
    let entry = state
        .indexer
        .add_entry_from_file_content(
            file_data,
            original_filename.clone(),
            tags.clone(),
            additional_notes,
        )
        .await?;

    // Notify WebSocket clients
    state.notify_new_clip(entry.id.clone(), entry.content.clone(), entry.tags.clone());

    Ok((StatusCode::CREATED, Json(entry.into())))
}
