use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static JIEBA: OnceCell<jieba_rs::Jieba> = OnceCell::new();

pub(crate) fn tokenize(text: &str) -> String {
    // Use jieba-rs for Chinese text segmentation
    let jieba = JIEBA.get_or_init(jieba_rs::Jieba::new);
    // Tokenize text and join tokens with zero-width space
    jieba
        .cut(text, false)
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\u{200B}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    #[serde(with = "datetime_conversion")]
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    #[serde(skip_serializing)]
    pub search_content: String,
}

mod datetime_conversion {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use surrealdb::sql::Datetime as SurrealDatetime;

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let surreal_dt = SurrealDatetime::from(*dt);
        surreal_dt.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let surreal_dt = SurrealDatetime::deserialize(deserializer)?;
        Ok(*surreal_dt)
    }
}

impl ClipboardEntry {
    pub fn new(content: String, tags: Vec<String>) -> Self {
        // Use UUID without hyphens for SurrealDB compatibility
        let id = uuid::Uuid::new_v4().simple().to_string();
        // Pre-tokenize content for search indexing
        let search_content = tokenize(&content);

        Self {
            id,
            content,
            created_at: Utc::now(),
            tags,
            additional_notes: None,
            file_attachment: None,
            original_filename: None,
            search_content,
        }
    }

    pub fn with_original_filename(mut self, filename: String) -> Self {
        self.original_filename = Some(filename);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.search_content = format!("{} {}", self.content, notes);
        self.additional_notes = Some(notes);
        self
    }

    pub fn with_file_attachment(mut self, file_path: String) -> Self {
        self.file_attachment = Some(file_path);
        self
    }

    pub fn update_search_content(&mut self) {
        self.search_content = match &self.additional_notes {
            Some(notes) => format!("{} {}", self.content, notes),
            None => self.content.clone(),
        };
    }
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

    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagingParams {
    /// Page number (starting from 1)
    pub page: usize,
    /// Number of items per page
    pub page_size: usize,
}

impl Default for PagingParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl PagingParams {
    pub fn new(page: usize, page_size: usize) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
        }
    }

    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

impl<T> PagedResult<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, page_size: usize) -> Self {
        let total_pages = total.div_ceil(page_size);
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// Represents a short URL that maps to a clipboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrl {
    pub id: String,
    pub clip_id: String,
    pub short_code: String,
    #[serde(with = "datetime_conversion")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "option_datetime_conversion")]
    pub expires_at: Option<DateTime<Utc>>,
}

mod option_datetime_conversion {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use surrealdb::sql::Datetime as SurrealDatetime;

    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => {
                let surreal_dt = SurrealDatetime::from(*dt);
                surreal_dt.serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<SurrealDatetime> = Option::deserialize(deserializer)?;
        Ok(opt.map(|surreal_dt| *surreal_dt))
    }
}

impl ShortUrl {
    /// Create a new ShortUrl with a generated short code
    pub fn new(clip_id: String, short_code: String, expires_at: Option<DateTime<Utc>>) -> Self {
        let id = uuid::Uuid::new_v4().simple().to_string();
        Self {
            id,
            clip_id,
            short_code,
            created_at: Utc::now(),
            expires_at,
        }
    }

    /// Check if this short URL has expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false, // No expiration set means never expires
        }
    }
}

/// Options for highlighting search results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HighlightOptions {
    /// The string to insert before matched text (e.g., "<mark>" or "**")
    pub prefix: Option<String>,
    /// The string to insert after matched text (e.g., "</mark>" or "**")
    pub suffix: Option<String>,
}

impl HighlightOptions {
    /// Create new highlight options with specified prefix and suffix
    pub fn new(prefix: String, suffix: String) -> Self {
        Self {
            prefix: Some(prefix),
            suffix: Some(suffix),
        }
    }

    /// Check if highlighting is enabled (both prefix and suffix are set)
    pub fn is_enabled(&self) -> bool {
        self.prefix.is_some() && self.suffix.is_some()
    }
}

/// A search result item with optional highlighted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// The clipboard entry
    #[serde(flatten)]
    pub entry: ClipboardEntry,
    /// Highlighted content (only present when highlight options are provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted_content: Option<String>,
}
