use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
        let search_content = content.clone();

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
