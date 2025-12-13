use crate::error::{IndexerError, Result};
use crate::export::{
    ExportBuilder, ExportedClip, ImportParser, ImportResult, calculate_content_hash,
};
use crate::models::{
    ClipboardEntry, HighlightOptions, PagedResult, PagingParams, SearchFilters, SearchResultItem,
    ShortUrl, Tag,
};
use crate::storage::FileStorage;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};

const TABLE_NAME: &str = "clipboard";
const SHORT_URL_TABLE: &str = "short_url";
const TAGS_TABLE: &str = "tags";
const CONFIG_TABLE: &str = "config";
const INDEX_VERSION_KEY: &str = "index_schema";
const SEARCH_ANALYZER_NAME: &str = "clipper_analyzer";
const TAGS_ANALYZER_NAME: &str = "clipper_tags_analyzer";
const SEARCH_INDEX_NAME: &str = "idx_search_content";
const TAGS_SEARCH_INDEX_NAME: &str = "idx_tag_text";
const NAMESPACE: &str = "clipper";
const DATABASE: &str = "library";
const CURRENT_INDEX_VERSION: i64 = 2;

/// Characters used for generating short codes (alphanumeric, excluding ambiguous characters)
const SHORT_CODE_CHARS: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz";
const SHORT_CODE_LENGTH: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbClipboardEntry {
    id: surrealdb::sql::Thing,
    content: String,
    created_at: surrealdb::sql::Datetime,
    tags: Vec<String>,
    additional_notes: Option<String>,
    file_attachment: Option<String>,
    original_filename: Option<String>,
    search_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexSchemaVersion {
    version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbShortUrl {
    id: surrealdb::sql::Thing,
    clip_id: String,
    short_code: String,
    created_at: surrealdb::sql::Datetime,
    expires_at: Option<surrealdb::sql::Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbTag {
    id: surrealdb::sql::Thing,
    text: String,
    created_at: surrealdb::sql::Datetime,
}

/// Generate a random short code using alphanumeric characters
fn generate_short_code() -> String {
    let mut rng = rand::rng();
    (0..SHORT_CODE_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..SHORT_CODE_CHARS.len());
            SHORT_CODE_CHARS[idx] as char
        })
        .collect()
}

pub struct ClipperIndexer {
    db: Surreal<Db>,
    storage: FileStorage,
}

impl ClipperIndexer {
    pub async fn new(db_path: impl AsRef<Path>, storage_path: impl AsRef<Path>) -> Result<Self> {
        // Initialize SurrealDB with RocksDB backend
        let db = Surreal::new::<RocksDb>(db_path.as_ref()).await?;

        // Select namespace and database
        db.use_ns(NAMESPACE).use_db(DATABASE).await?;

        // Initialize schema and indexes
        Self::initialize_schema(&db).await?;
        Self::run_migrations(&db).await?;

        // Initialize file storage
        let storage = FileStorage::new(storage_path)?;

        Ok(Self { db, storage })
    }

    async fn initialize_schema(db: &Surreal<Db>) -> Result<()> {
        // Define the clipboard table schema
        let schema_query = format!(
            r#"
            DEFINE TABLE IF NOT EXISTS {TABLE_NAME} SCHEMAFULL;

            DEFINE FIELD IF NOT EXISTS content ON TABLE {TABLE_NAME} TYPE string;
            DEFINE FIELD IF NOT EXISTS created_at ON TABLE {TABLE_NAME} TYPE datetime;
            DEFINE FIELD IF NOT EXISTS tags ON TABLE {TABLE_NAME} TYPE array<string>;
            DEFINE FIELD IF NOT EXISTS additional_notes ON TABLE {TABLE_NAME} TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS file_attachment ON TABLE {TABLE_NAME} TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS original_filename ON TABLE {TABLE_NAME} TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS search_content ON TABLE {TABLE_NAME} TYPE string;

            DEFINE TABLE IF NOT EXISTS {CONFIG_TABLE} SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS version ON TABLE {CONFIG_TABLE} TYPE int;

            DEFINE TABLE IF NOT EXISTS {SHORT_URL_TABLE} SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS clip_id ON TABLE {SHORT_URL_TABLE} TYPE string;
            DEFINE FIELD IF NOT EXISTS short_code ON TABLE {SHORT_URL_TABLE} TYPE string;
            DEFINE FIELD IF NOT EXISTS created_at ON TABLE {SHORT_URL_TABLE} TYPE datetime;
            DEFINE FIELD IF NOT EXISTS expires_at ON TABLE {SHORT_URL_TABLE} TYPE option<datetime>;

            DEFINE TABLE IF NOT EXISTS {TAGS_TABLE} SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS text ON TABLE {TAGS_TABLE} TYPE string;
            DEFINE FIELD IF NOT EXISTS created_at ON TABLE {TAGS_TABLE} TYPE datetime;
            "#
        );

        db.query(schema_query).await?;

        // Define indexes
        let index_query = format!(
            r#"
            DEFINE INDEX IF NOT EXISTS idx_created_at ON TABLE {TABLE_NAME} COLUMNS created_at;
            DEFINE INDEX IF NOT EXISTS idx_tags ON TABLE {TABLE_NAME} COLUMNS tags;
            DEFINE INDEX IF NOT EXISTS idx_short_code ON TABLE {SHORT_URL_TABLE} COLUMNS short_code UNIQUE;
            DEFINE INDEX IF NOT EXISTS idx_short_url_clip_id ON TABLE {SHORT_URL_TABLE} COLUMNS clip_id;
            DEFINE INDEX IF NOT EXISTS idx_short_url_expires_at ON TABLE {SHORT_URL_TABLE} COLUMNS expires_at;
            DEFINE INDEX IF NOT EXISTS idx_tag_text_unique ON TABLE {TAGS_TABLE} COLUMNS text UNIQUE;
            "#
        );

        db.query(index_query).await?;

        Ok(())
    }

    async fn run_migrations(db: &Surreal<Db>) -> Result<()> {
        let mut version = Self::get_index_schema_version(db).await?;

        if version >= CURRENT_INDEX_VERSION {
            return Ok(());
        }

        if version < 1 {
            Self::migrate_to_v1(db).await?;
            version = 1;
        }

        if version < 2 {
            Self::migrate_to_v2(db).await?;
        }

        // Always save the version after migrations complete
        Self::set_index_schema_version(db, CURRENT_INDEX_VERSION).await?;

        Ok(())
    }

    async fn get_index_schema_version(db: &Surreal<Db>) -> Result<i64> {
        let record: Option<IndexSchemaVersion> =
            db.select((CONFIG_TABLE, INDEX_VERSION_KEY)).await?;

        Ok(record.map(|r| r.version).unwrap_or(0))
    }

    async fn set_index_schema_version(db: &Surreal<Db>, version: i64) -> Result<()> {
        let _: Option<IndexSchemaVersion> = db
            .upsert((CONFIG_TABLE, INDEX_VERSION_KEY))
            .content(IndexSchemaVersion { version })
            .await?;

        Ok(())
    }

    async fn migrate_to_v1(db: &Surreal<Db>) -> Result<()> {
        let migration_query = format!(
            r#"
            REMOVE ANALYZER IF EXISTS {analyzer};
            REMOVE INDEX IF EXISTS {index} ON TABLE {table};

            DEFINE ANALYZER {analyzer} TOKENIZERS blank,class,camel FILTERS lowercase,snowball(english),ngram(1, 24);
            DEFINE INDEX {index} ON TABLE {table} COLUMNS search_content
                SEARCH ANALYZER {analyzer} BM25 HIGHLIGHTS;
            "#,
            analyzer = SEARCH_ANALYZER_NAME,
            index = SEARCH_INDEX_NAME,
            table = TABLE_NAME
        );

        db.query(migration_query).await?;

        Ok(())
    }

    async fn migrate_to_v2(db: &Surreal<Db>) -> Result<()> {
        // Define a new analyzer for tags using edgengram instead of ngram
        // edgengram is better for prefix/autocomplete matching on tag names
        let analyzer_query = format!(
            r#"
            REMOVE ANALYZER IF EXISTS {analyzer};
            DEFINE ANALYZER {analyzer} FILTERS lowercase,edgengram(1, 24);
            "#,
            analyzer = TAGS_ANALYZER_NAME
        );
        db.query(analyzer_query).await?;

        // Add FTS index for tags table
        let index_query = format!(
            r#"
            REMOVE INDEX IF EXISTS {index} ON TABLE {table};
            DEFINE INDEX {index} ON TABLE {table} COLUMNS text
                SEARCH ANALYZER {analyzer} BM25 HIGHLIGHTS;
            "#,
            index = TAGS_SEARCH_INDEX_NAME,
            table = TAGS_TABLE,
            analyzer = TAGS_ANALYZER_NAME
        );
        db.query(index_query).await?;

        // Collect existing tags from clipboard entries into the tags table
        let select_query = format!("SELECT tags FROM {};", TABLE_NAME);
        let mut response = db.query(select_query).await?;

        #[derive(Deserialize)]
        struct TagsOnly {
            tags: Vec<String>,
        }

        let entries: Vec<TagsOnly> = response.take(0).unwrap_or_default();

        // Collect all unique tags
        let mut unique_tags = std::collections::HashSet::new();
        for entry in entries {
            for tag in entry.tags {
                if !tag.is_empty() {
                    unique_tags.insert(tag);
                }
            }
        }

        // Insert each unique tag into the tags table
        let now = chrono::Utc::now();
        for tag in unique_tags {
            let tag_id = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                tag.hash(&mut hasher);
                format!("tag_{:x}", hasher.finish())
            };
            let insert_query = format!(
                "INSERT INTO {} {{ id: $id, text: $text, created_at: <datetime>$created_at }} ON DUPLICATE KEY UPDATE id = id;",
                TAGS_TABLE
            );
            db.query(insert_query)
                .bind(("id", tag_id))
                .bind(("text", tag))
                .bind(("created_at", now.to_rfc3339()))
                .await?;
        }

        Ok(())
    }

    /// Sync tags to the tags table. This ensures all tags from the given list
    /// exist in the tags table. Tags that already exist are skipped.
    async fn sync_tags(&self, tags: &[String]) -> Result<()> {
        if tags.is_empty() {
            return Ok(());
        }

        let now = chrono::Utc::now();
        for tag in tags {
            if tag.is_empty() {
                continue;
            }
            // Use UPSERT to create the tag if it doesn't exist, or do nothing if it does
            // We use the tag text as the record ID to ensure uniqueness
            let tag_id = Self::tag_text_to_id(tag);
            let query = format!(
                "INSERT INTO {} {{ id: $id, text: $text, created_at: <datetime>$created_at }} ON DUPLICATE KEY UPDATE id = id;",
                TAGS_TABLE
            );
            self.db
                .query(query)
                .bind(("id", tag_id))
                .bind(("text", tag.clone()))
                .bind(("created_at", now.to_rfc3339()))
                .await?;
        }

        Ok(())
    }

    /// Convert tag text to a valid record ID (lowercase, alphanumeric with underscores)
    fn tag_text_to_id(tag: &str) -> String {
        // Create a deterministic ID from the tag text using a hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        format!("tag_{:x}", hasher.finish())
    }

    /// Get the current index schema version.
    ///
    /// This version number indicates which features are available in the index:
    /// - Version 0: Initial schema (no FTS)
    /// - Version 1: Full-text search with ngram analyzer
    /// - Version 2: Tags table with edgengram FTS
    pub async fn get_index_version(&self) -> Result<i64> {
        Self::get_index_schema_version(&self.db).await
    }

    pub async fn add_entry_from_text(
        &self,
        content: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<ClipboardEntry> {
        let mut entry = ClipboardEntry::new(content, tags);

        if let Some(notes) = additional_notes {
            entry = entry.with_notes(notes);
        }

        // Insert into database using SDK method
        let record_id = (TABLE_NAME, entry.id.as_str());
        let _: Option<DbClipboardEntry> = self
            .db
            .create(record_id)
            .content(DbClipboardEntry {
                id: surrealdb::sql::Thing::from((TABLE_NAME.to_string(), entry.id.clone())),
                content: entry.content.clone(),
                created_at: surrealdb::sql::Datetime::from(entry.created_at),
                tags: entry.tags.clone(),
                additional_notes: entry.additional_notes.clone(),
                file_attachment: entry.file_attachment.clone(),
                original_filename: entry.original_filename.clone(),
                search_content: entry.search_content.clone(),
            })
            .await?;

        // Sync tags to the tags table
        self.sync_tags(&entry.tags).await?;

        Ok(entry)
    }

    pub async fn add_entry_from_file(
        &self,
        file_path: impl AsRef<Path>,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<ClipboardEntry> {
        let file_path = file_path.as_ref();

        // Validate file exists
        if !file_path.exists() {
            return Err(IndexerError::InvalidInput(format!(
                "File does not exist: {}",
                file_path.display()
            )));
        }

        // Extract original filename from path
        let original_filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        // Store the file using object_store
        let stored_file_key = self.storage.put_file(file_path).await?;

        // Read file content for search indexing
        let file_content = tokio::fs::read_to_string(file_path)
            .await
            .unwrap_or_else(|_| file_path.display().to_string());

        let mut entry = ClipboardEntry::new(file_content, tags);
        entry = entry.with_file_attachment(stored_file_key);

        if let Some(filename) = original_filename {
            entry = entry.with_original_filename(filename);
        }

        if let Some(notes) = additional_notes {
            entry = entry.with_notes(notes);
        }

        // Insert into database using SDK method
        let record_id = (TABLE_NAME, entry.id.as_str());
        let _: Option<DbClipboardEntry> = self
            .db
            .create(record_id)
            .content(DbClipboardEntry {
                id: surrealdb::sql::Thing::from((TABLE_NAME.to_string(), entry.id.clone())),
                content: entry.content.clone(),
                created_at: surrealdb::sql::Datetime::from(entry.created_at),
                tags: entry.tags.clone(),
                additional_notes: entry.additional_notes.clone(),
                file_attachment: entry.file_attachment.clone(),
                original_filename: entry.original_filename.clone(),
                search_content: entry.search_content.clone(),
            })
            .await?;

        // Sync tags to the tags table
        self.sync_tags(&entry.tags).await?;

        Ok(entry)
    }

    pub async fn add_entry_from_file_content(
        &self,
        file_content: bytes::Bytes,
        original_filename: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<ClipboardEntry> {
        self.add_entry_from_file_content_with_override(
            file_content,
            original_filename,
            tags,
            additional_notes,
            None,
        )
        .await
    }

    /// Add a new entry from file bytes with an optional content override.
    ///
    /// When `content_override` is provided, it will be used as the entry's content
    /// instead of trying to read the file as text or falling back to the filename.
    /// This is useful when you want to store the full file path as content.
    pub async fn add_entry_from_file_content_with_override(
        &self,
        file_content: bytes::Bytes,
        original_filename: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
        content_override: Option<String>,
    ) -> Result<ClipboardEntry> {
        // Store the file using object_store
        let stored_file_key = self
            .storage
            .put_file_bytes(file_content.clone(), &original_filename)
            .await?;

        // Use content_override if provided, otherwise try to read file content as text
        let text_content = content_override.unwrap_or_else(|| {
            String::from_utf8(file_content.to_vec()).unwrap_or_else(|_| original_filename.clone())
        });

        let mut entry = ClipboardEntry::new(text_content, tags);
        entry = entry.with_file_attachment(stored_file_key);
        entry = entry.with_original_filename(original_filename);

        if let Some(notes) = additional_notes {
            entry = entry.with_notes(notes);
        }

        // Insert into database using SDK method
        let record_id = (TABLE_NAME, entry.id.as_str());
        let _: Option<DbClipboardEntry> = self
            .db
            .create(record_id)
            .content(DbClipboardEntry {
                id: surrealdb::sql::Thing::from((TABLE_NAME.to_string(), entry.id.clone())),
                content: entry.content.clone(),
                created_at: surrealdb::sql::Datetime::from(entry.created_at),
                tags: entry.tags.clone(),
                additional_notes: entry.additional_notes.clone(),
                file_attachment: entry.file_attachment.clone(),
                original_filename: entry.original_filename.clone(),
                search_content: entry.search_content.clone(),
            })
            .await?;

        // Sync tags to the tags table
        self.sync_tags(&entry.tags).await?;

        Ok(entry)
    }

    pub async fn get_entry(&self, id: &str) -> Result<ClipboardEntry> {
        let record_id = (TABLE_NAME, id);
        let db_entry: Option<DbClipboardEntry> = self.db.select(record_id).await?;

        db_entry
            .map(|db_entry| ClipboardEntry {
                id: db_entry.id.id.to_string(),
                content: db_entry.content,
                created_at: *db_entry.created_at,
                tags: db_entry.tags,
                additional_notes: db_entry.additional_notes,
                file_attachment: db_entry.file_attachment,
                original_filename: db_entry.original_filename,
                search_content: db_entry.search_content,
            })
            .ok_or_else(|| IndexerError::NotFound(format!("Entry with id {} not found", id)))
    }

    pub async fn update_entry(
        &self,
        id: &str,
        tags: Option<Vec<String>>,
        additional_notes: Option<String>,
    ) -> Result<ClipboardEntry> {
        // First, retrieve the existing entry to get the content
        let existing_entry = self.get_entry(id).await?;

        // Calculate new search_content if additional_notes is being updated
        let new_search_content = match &additional_notes {
            Some(notes) => format!("{} {}", existing_entry.content, notes),
            None => match &existing_entry.additional_notes {
                Some(existing_notes) => format!("{} {}", existing_entry.content, existing_notes),
                None => existing_entry.content.clone(),
            },
        };

        // Build update query
        let mut updates = Vec::new();
        let query_string = "UPDATE type::thing($table, $id) SET ".to_string();

        let tags_to_sync = tags.clone();
        if tags.is_some() {
            updates.push("tags = $tags");
        }

        if additional_notes.is_some() {
            updates.push("additional_notes = $additional_notes");
            updates.push("search_content = $search_content");
        }

        if updates.is_empty() {
            return Ok(existing_entry);
        }

        let query_string = format!("{}{};", query_string, updates.join(", "));

        let mut query = self
            .db
            .query(query_string)
            .bind(("table", TABLE_NAME))
            .bind(("id", id.to_string()));

        if let Some(t) = tags {
            query = query.bind(("tags", t));
        }

        if let Some(notes) = additional_notes {
            query = query.bind(("additional_notes", notes));
            query = query.bind(("search_content", new_search_content));
        }

        query.await?;

        // Sync tags to the tags table if tags were updated
        if let Some(new_tags) = tags_to_sync {
            self.sync_tags(&new_tags).await?;
        }

        // Return updated entry
        self.get_entry(id).await
    }

    pub async fn search_entries(
        &self,
        search_query: &str,
        filters: SearchFilters,
        paging: PagingParams,
    ) -> Result<PagedResult<ClipboardEntry>> {
        let result = self
            .search_entries_with_highlight(search_query, filters, paging, None)
            .await?;

        // Convert SearchResultItem back to ClipboardEntry
        let items: Vec<ClipboardEntry> = result.items.into_iter().map(|item| item.entry).collect();

        Ok(PagedResult::new(
            items,
            result.total,
            result.page,
            result.page_size,
        ))
    }

    /// Search entries with optional highlighting support.
    ///
    /// When `highlight` is provided with both prefix and suffix, the returned
    /// `SearchResultItem` will include `highlighted_content` with matching terms
    /// wrapped by the prefix and suffix strings.
    ///
    /// # Arguments
    /// * `search_query` - The search query string
    /// * `filters` - Optional filters for date range and tags
    /// * `paging` - Pagination parameters
    /// * `highlight` - Optional highlight options (prefix/suffix for matched terms)
    ///
    /// # Returns
    /// A paged result containing search result items with optional highlighted content
    pub async fn search_entries_with_highlight(
        &self,
        search_query: &str,
        filters: SearchFilters,
        paging: PagingParams,
        highlight: Option<HighlightOptions>,
    ) -> Result<PagedResult<SearchResultItem>> {
        // Return all entries if search query is empty
        if search_query.trim().is_empty() {
            let result = self.list_entries(filters, paging).await?;
            let items: Vec<SearchResultItem> = result
                .items
                .into_iter()
                .map(|entry| SearchResultItem {
                    entry,
                    highlighted_content: None,
                })
                .collect();
            return Ok(PagedResult::new(
                items,
                result.total,
                result.page,
                result.page_size,
            ));
        }

        let highlight_enabled = highlight.as_ref().map(|h| h.is_enabled()).unwrap_or(false);

        // Pre-tokenize search query for better Chinese search
        let tokenized_query = crate::models::tokenize(search_query);

        // Use reference number 0 for the matches operator
        let match_operator = if highlight_enabled { "@0@" } else { "@@" };
        let mut where_clauses = vec![format!("search_content {} $query", match_operator)];

        if filters.start_date.is_some() {
            where_clauses.push("created_at >= <datetime>$start_date".to_string());
        }

        if filters.end_date.is_some() {
            where_clauses.push("created_at <= <datetime>$end_date".to_string());
        }

        // For tags, we need to check membership - use array contains
        let filter_tags = filters.tags.clone();
        if let Some(ref tags) = filter_tags
            && !tags.is_empty()
        {
            // Build tag conditions using indexed parameters
            let tag_conditions: Vec<String> = (0..tags.len())
                .map(|i| format!("$tag{} IN tags", i))
                .collect();
            where_clauses.push(format!("({})", tag_conditions.join(" AND ")));
        }

        let where_clause = where_clauses.join(" AND ");

        // Get total count
        let count_query = format!(
            "SELECT count() FROM {} WHERE {} GROUP ALL;",
            TABLE_NAME, where_clause
        );
        let mut count_query_builder = self
            .db
            .query(&count_query)
            .bind(("query", tokenized_query.clone()));

        if let Some(start_date) = filters.start_date {
            count_query_builder = count_query_builder.bind(("start_date", start_date.to_rfc3339()));
        }
        if let Some(end_date) = filters.end_date {
            count_query_builder = count_query_builder.bind(("end_date", end_date.to_rfc3339()));
        }
        if let Some(ref tags) = filter_tags {
            for (i, tag) in tags.iter().enumerate() {
                count_query_builder = count_query_builder.bind((format!("tag{}", i), tag.clone()));
            }
        }

        let mut count_response = count_query_builder.await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let total = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Build select clause with optional highlight
        let select_clause = if highlight_enabled {
            "*, search::highlight($hl_prefix, $hl_suffix, 0) AS highlighted_content".to_string()
        } else {
            "*".to_string()
        };

        // Get paginated results
        let query = format!(
            "SELECT {} FROM {} WHERE {} ORDER BY created_at DESC LIMIT $limit START $offset;",
            select_clause, TABLE_NAME, where_clause
        );

        let mut query_builder = self
            .db
            .query(&query)
            .bind(("query", tokenized_query))
            .bind(("limit", paging.page_size as i64))
            .bind(("offset", paging.offset() as i64));

        if let Some(start_date) = filters.start_date {
            query_builder = query_builder.bind(("start_date", start_date.to_rfc3339()));
        }
        if let Some(end_date) = filters.end_date {
            query_builder = query_builder.bind(("end_date", end_date.to_rfc3339()));
        }
        if let Some(ref tags) = filter_tags {
            for (i, tag) in tags.iter().enumerate() {
                query_builder = query_builder.bind((format!("tag{}", i), tag.clone()));
            }
        }
        if highlight_enabled {
            let h = highlight.as_ref().unwrap();
            query_builder = query_builder.bind(("hl_prefix", h.prefix.clone().unwrap_or_default()));
            query_builder = query_builder.bind(("hl_suffix", h.suffix.clone().unwrap_or_default()));
        }

        let mut response = query_builder.await?;

        // Use a different struct when highlight is enabled
        if highlight_enabled {
            #[derive(Deserialize)]
            struct DbClipboardEntryWithHighlight {
                id: surrealdb::sql::Thing,
                content: String,
                created_at: surrealdb::sql::Datetime,
                tags: Vec<String>,
                additional_notes: Option<String>,
                file_attachment: Option<String>,
                original_filename: Option<String>,
                search_content: String,
                highlighted_content: Option<String>,
            }

            let entries: Vec<DbClipboardEntryWithHighlight> = response
                .take(0)
                .map_err(|e| IndexerError::Serialization(e.to_string()))?;

            let items: Vec<SearchResultItem> = entries
                .into_iter()
                .map(|db_entry| SearchResultItem {
                    entry: ClipboardEntry {
                        id: db_entry.id.id.to_string(),
                        content: db_entry.content,
                        created_at: *db_entry.created_at,
                        tags: db_entry.tags,
                        additional_notes: db_entry.additional_notes,
                        file_attachment: db_entry.file_attachment,
                        original_filename: db_entry.original_filename,
                        search_content: db_entry.search_content,
                    },
                    highlighted_content: db_entry.highlighted_content,
                })
                .collect();

            Ok(PagedResult::new(
                items,
                total,
                paging.page,
                paging.page_size,
            ))
        } else {
            let entries: Vec<DbClipboardEntry> = response
                .take(0)
                .map_err(|e| IndexerError::Serialization(e.to_string()))?;

            let items: Vec<SearchResultItem> = entries
                .into_iter()
                .map(|db_entry| SearchResultItem {
                    entry: ClipboardEntry {
                        id: db_entry.id.id.to_string(),
                        content: db_entry.content,
                        created_at: *db_entry.created_at,
                        tags: db_entry.tags,
                        additional_notes: db_entry.additional_notes,
                        file_attachment: db_entry.file_attachment,
                        original_filename: db_entry.original_filename,
                        search_content: db_entry.search_content,
                    },
                    highlighted_content: None,
                })
                .collect();

            Ok(PagedResult::new(
                items,
                total,
                paging.page,
                paging.page_size,
            ))
        }
    }

    pub async fn list_entries(
        &self,
        filters: SearchFilters,
        paging: PagingParams,
    ) -> Result<PagedResult<ClipboardEntry>> {
        let mut where_clauses = Vec::new();

        if filters.start_date.is_some() {
            where_clauses.push("created_at >= <datetime>$start_date".to_string());
        }

        if filters.end_date.is_some() {
            where_clauses.push("created_at <= <datetime>$end_date".to_string());
        }

        let filter_tags = filters.tags.clone();
        if let Some(ref tags) = filter_tags
            && !tags.is_empty()
        {
            let tag_conditions: Vec<String> = (0..tags.len())
                .map(|i| format!("$tag{} IN tags", i))
                .collect();
            where_clauses.push(format!("({})", tag_conditions.join(" AND ")));
        }

        // Get total count
        let count_query = if where_clauses.is_empty() {
            format!("SELECT count() FROM {} GROUP ALL;", TABLE_NAME)
        } else {
            let where_clause = where_clauses.join(" AND ");
            format!(
                "SELECT count() FROM {} WHERE {} GROUP ALL;",
                TABLE_NAME, where_clause
            )
        };

        let mut count_query_builder = self.db.query(&count_query);
        if let Some(start_date) = filters.start_date {
            count_query_builder = count_query_builder.bind(("start_date", start_date.to_rfc3339()));
        }
        if let Some(end_date) = filters.end_date {
            count_query_builder = count_query_builder.bind(("end_date", end_date.to_rfc3339()));
        }
        if let Some(ref tags) = filter_tags {
            for (i, tag) in tags.iter().enumerate() {
                count_query_builder = count_query_builder.bind((format!("tag{}", i), tag.clone()));
            }
        }
        let mut count_response = count_query_builder.await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let total = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Get paginated results
        let query = if where_clauses.is_empty() {
            format!(
                "SELECT * FROM {} ORDER BY created_at DESC LIMIT $limit START $offset;",
                TABLE_NAME
            )
        } else {
            let where_clause = where_clauses.join(" AND ");
            format!(
                "SELECT * FROM {} WHERE {} ORDER BY created_at DESC LIMIT $limit START $offset;",
                TABLE_NAME, where_clause
            )
        };

        let mut query_builder = self
            .db
            .query(&query)
            .bind(("limit", paging.page_size as i64))
            .bind(("offset", paging.offset() as i64));
        if let Some(start_date) = filters.start_date {
            query_builder = query_builder.bind(("start_date", start_date.to_rfc3339()));
        }
        if let Some(end_date) = filters.end_date {
            query_builder = query_builder.bind(("end_date", end_date.to_rfc3339()));
        }
        if let Some(ref tags) = filter_tags {
            for (i, tag) in tags.iter().enumerate() {
                query_builder = query_builder.bind((format!("tag{}", i), tag.clone()));
            }
        }

        let mut response = query_builder.await?;

        let entries: Vec<DbClipboardEntry> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        let items: Vec<ClipboardEntry> = entries
            .into_iter()
            .map(|db_entry| ClipboardEntry {
                id: db_entry.id.id.to_string(),
                content: db_entry.content,
                created_at: *db_entry.created_at,
                tags: db_entry.tags,
                additional_notes: db_entry.additional_notes,
                file_attachment: db_entry.file_attachment,
                original_filename: db_entry.original_filename,
                search_content: db_entry.search_content,
            })
            .collect();

        Ok(PagedResult::new(
            items,
            total,
            paging.page,
            paging.page_size,
        ))
    }

    pub async fn get_file_content(&self, file_key: &str) -> Result<bytes::Bytes> {
        self.storage.get_file(file_key).await
    }

    pub async fn delete_entry(&self, id: &str) -> Result<()> {
        // Get the entry to check if it has a file attachment
        let entry = self.get_entry(id).await?;

        // Delete the file if it exists
        if let Some(file_key) = entry.file_attachment {
            let _ = self.storage.delete_file(&file_key).await;
        }

        // Delete the database entry
        let query = "DELETE type::thing($table, $id);";
        self.db
            .query(query)
            .bind(("table", TABLE_NAME))
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    /// Delete all clip entries without any tags (except host tags) within a given time range.
    ///
    /// This function finds entries where:
    /// - All tags start with "host:" (only host tags), OR
    /// - There are no tags at all
    ///
    /// And deletes them if they fall within the specified time range.
    ///
    /// # Arguments
    /// * `start_date` - Optional start of the time range (inclusive)
    /// * `end_date` - Optional end of the time range (inclusive)
    ///
    /// # Returns
    /// A vector of IDs of the deleted entries
    pub async fn cleanup_entries(
        &self,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<String>> {
        let mut where_clauses = Vec::new();

        // Entries with no tags OR all tags start with "$host:"
        // array::len(tags) == 0 OR all tags match "$host:*"
        where_clauses.push(
            "(array::len(tags) == 0 OR array::len(array::filter(tags, |$t| !string::starts_with($t, '$host:'))) == 0)".to_string()
        );

        if let Some(start) = start_date {
            where_clauses.push(format!("created_at >= <datetime>'{}'", start.to_rfc3339()));
        }

        if let Some(end) = end_date {
            where_clauses.push(format!("created_at <= <datetime>'{}'", end.to_rfc3339()));
        }

        let where_clause = where_clauses.join(" AND ");

        // First, get all entries that match the criteria to delete their files
        let select_query = format!("SELECT * FROM {} WHERE {};", TABLE_NAME, where_clause);

        let mut response = self.db.query(select_query).await?;
        let entries: Vec<DbClipboardEntry> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        // Collect the IDs of entries to be deleted
        let deleted_ids: Vec<String> = entries.iter().map(|e| e.id.id.to_string()).collect();

        // Delete all matching entries from the database
        let delete_query = format!("DELETE FROM {} WHERE {};", TABLE_NAME, where_clause);
        self.db.query(delete_query).await?;

        // Delete file attachments for all matching entries
        for entry in &entries {
            if let Some(ref file_key) = entry.file_attachment {
                let _ = self.storage.delete_file(file_key).await;
            }
        }

        Ok(deleted_ids)
    }

    // ==================== Short URL Functions ====================

    /// Create a short URL for a clip.
    ///
    /// Generates a unique short code and stores it in the database with the associated clip ID.
    /// If an expiration time is provided, the short URL will be invalid after that time.
    ///
    /// # Arguments
    /// * `clip_id` - The ID of the clip to create a short URL for
    /// * `expires_at` - Optional expiration time for the short URL
    ///
    /// # Returns
    /// The created ShortUrl
    pub async fn create_short_url(
        &self,
        clip_id: &str,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<ShortUrl> {
        // Verify the clip exists
        let _ = self.get_entry(clip_id).await?;

        // Generate a unique short code (retry if collision)
        let mut short_code = generate_short_code();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10;

        while attempts < MAX_ATTEMPTS {
            // Check if the short code already exists
            let check_query = format!(
                "SELECT * FROM {} WHERE short_code = $code;",
                SHORT_URL_TABLE
            );
            let mut response = self
                .db
                .query(check_query)
                .bind(("code", short_code.clone()))
                .await?;
            let existing: Vec<DbShortUrl> = response.take(0).unwrap_or_default();

            if existing.is_empty() {
                break;
            }

            short_code = generate_short_code();
            attempts += 1;
        }

        if attempts >= MAX_ATTEMPTS {
            return Err(IndexerError::InvalidInput(
                "Failed to generate unique short code after multiple attempts".to_string(),
            ));
        }

        let short_url = ShortUrl::new(clip_id.to_string(), short_code, expires_at);

        // Insert into database
        let record_id = (SHORT_URL_TABLE, short_url.id.as_str());
        let _: Option<DbShortUrl> = self
            .db
            .create(record_id)
            .content(DbShortUrl {
                id: surrealdb::sql::Thing::from((
                    SHORT_URL_TABLE.to_string(),
                    short_url.id.clone(),
                )),
                clip_id: short_url.clip_id.clone(),
                short_code: short_url.short_code.clone(),
                created_at: surrealdb::sql::Datetime::from(short_url.created_at),
                expires_at: short_url.expires_at.map(surrealdb::sql::Datetime::from),
            })
            .await?;

        Ok(short_url)
    }

    /// Get a short URL by its short code.
    ///
    /// Returns an error if the short URL is not found or has expired.
    ///
    /// # Arguments
    /// * `short_code` - The short code to look up
    ///
    /// # Returns
    /// The ShortUrl if found and not expired
    pub async fn get_short_url(&self, short_code: &str) -> Result<ShortUrl> {
        let query = format!(
            "SELECT * FROM {} WHERE short_code = $code;",
            SHORT_URL_TABLE
        );

        let mut response = self
            .db
            .query(query)
            .bind(("code", short_code.to_string()))
            .await?;
        let results: Vec<DbShortUrl> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        let db_short_url = results.into_iter().next().ok_or_else(|| {
            IndexerError::NotFound(format!("Short URL with code '{}' not found", short_code))
        })?;

        let short_url = ShortUrl {
            id: db_short_url.id.id.to_string(),
            clip_id: db_short_url.clip_id,
            short_code: db_short_url.short_code,
            created_at: *db_short_url.created_at,
            expires_at: db_short_url.expires_at.map(|dt| *dt),
        };

        // Check if expired
        if short_url.is_expired() {
            return Err(IndexerError::ShortUrlExpired(format!(
                "Short URL with code '{}' has expired",
                short_code
            )));
        }

        Ok(short_url)
    }

    /// Get all short URLs for a specific clip.
    ///
    /// # Arguments
    /// * `clip_id` - The ID of the clip
    ///
    /// # Returns
    /// A vector of ShortUrls associated with the clip
    pub async fn get_short_urls_for_clip(&self, clip_id: &str) -> Result<Vec<ShortUrl>> {
        let query = format!(
            "SELECT * FROM {} WHERE clip_id = $clip_id ORDER BY created_at DESC;",
            SHORT_URL_TABLE
        );

        let mut response = self
            .db
            .query(query)
            .bind(("clip_id", clip_id.to_string()))
            .await?;
        let results: Vec<DbShortUrl> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        let short_urls: Vec<ShortUrl> = results
            .into_iter()
            .map(|db| ShortUrl {
                id: db.id.id.to_string(),
                clip_id: db.clip_id,
                short_code: db.short_code,
                created_at: *db.created_at,
                expires_at: db.expires_at.map(|dt| *dt),
            })
            .collect();

        Ok(short_urls)
    }

    /// Delete a short URL by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the short URL to delete
    pub async fn delete_short_url(&self, id: &str) -> Result<()> {
        let query = "DELETE type::thing($table, $id);";
        self.db
            .query(query)
            .bind(("table", SHORT_URL_TABLE))
            .bind(("id", id.to_string()))
            .await?;
        Ok(())
    }

    /// Delete all short URLs for a specific clip.
    ///
    /// # Arguments
    /// * `clip_id` - The ID of the clip whose short URLs should be deleted
    ///
    /// # Returns
    /// The number of short URLs deleted
    pub async fn delete_short_urls_for_clip(&self, clip_id: &str) -> Result<usize> {
        // First count how many will be deleted
        let count_query = format!(
            "SELECT count() FROM {} WHERE clip_id = $clip_id GROUP ALL;",
            SHORT_URL_TABLE
        );
        let clip_id_owned = clip_id.to_string();
        let mut count_response = self
            .db
            .query(count_query)
            .bind(("clip_id", clip_id_owned.clone()))
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let count = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Delete the short URLs
        let delete_query = format!("DELETE FROM {} WHERE clip_id = $clip_id;", SHORT_URL_TABLE);
        self.db
            .query(delete_query)
            .bind(("clip_id", clip_id_owned))
            .await?;

        Ok(count)
    }

    /// Clean up all expired short URLs from the database.
    ///
    /// This function finds and deletes all short URLs where expires_at is in the past.
    ///
    /// # Returns
    /// The number of expired short URLs that were deleted
    pub async fn cleanup_expired_short_urls(&self) -> Result<usize> {
        let now = chrono::Utc::now();

        // Count expired short URLs
        let count_query = format!(
            "SELECT count() FROM {} WHERE expires_at != NONE AND expires_at < <datetime>'{}' GROUP ALL;",
            SHORT_URL_TABLE,
            now.to_rfc3339()
        );
        let mut count_response = self.db.query(count_query).await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let count = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Delete expired short URLs
        let delete_query = format!(
            "DELETE FROM {} WHERE expires_at != NONE AND expires_at < <datetime>'{}';",
            SHORT_URL_TABLE,
            now.to_rfc3339()
        );
        self.db.query(delete_query).await?;

        Ok(count)
    }

    // ==================== Tags Functions ====================

    /// List all tags with optional pagination.
    ///
    /// # Arguments
    /// * `paging` - Pagination parameters
    ///
    /// # Returns
    /// A paged result containing all tags ordered by creation date
    pub async fn list_tags(&self, paging: PagingParams) -> Result<PagedResult<Tag>> {
        // Get total count
        let count_query = format!("SELECT count() FROM {} GROUP ALL;", TAGS_TABLE);
        let mut count_response = self.db.query(count_query).await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let total = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Get paginated results - order by created_at instead of text to avoid
        // conflicts with the FTS SEARCH index on the text field
        let query = format!(
            "SELECT * FROM {} ORDER BY created_at DESC LIMIT {} START {};",
            TAGS_TABLE,
            paging.page_size,
            paging.offset()
        );

        let mut response = self.db.query(query).await?;
        let db_tags: Vec<DbTag> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        let items: Vec<Tag> = db_tags
            .into_iter()
            .map(|db_tag| Tag {
                id: db_tag.id.id.to_string(),
                text: db_tag.text,
                created_at: *db_tag.created_at,
            })
            .collect();

        Ok(PagedResult::new(
            items,
            total,
            paging.page,
            paging.page_size,
        ))
    }

    /// Search tags using full-text search.
    ///
    /// # Arguments
    /// * `search_query` - The search query string
    /// * `paging` - Pagination parameters
    ///
    /// # Returns
    /// A paged result containing matching tags
    pub async fn search_tags(
        &self,
        search_query: &str,
        paging: PagingParams,
    ) -> Result<PagedResult<Tag>> {
        // Return all tags if search query is empty
        if search_query.trim().is_empty() {
            return self.list_tags(paging).await;
        }

        let where_clause = "text @@ $query";
        let search_query_owned = search_query.to_string();

        // Get total count
        let count_query = format!(
            "SELECT count() FROM {} WHERE {} GROUP ALL;",
            TAGS_TABLE, where_clause
        );
        let mut count_response = self
            .db
            .query(count_query)
            .bind(("query", search_query_owned.clone()))
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let total = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Get paginated results
        let query = format!(
            "SELECT * FROM {} WHERE {} ORDER BY text ASC LIMIT $limit START $offset;",
            TAGS_TABLE, where_clause
        );

        let mut response = self
            .db
            .query(query)
            .bind(("query", search_query_owned))
            .bind(("limit", paging.page_size as i64))
            .bind(("offset", paging.offset() as i64))
            .await?;
        let db_tags: Vec<DbTag> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        let items: Vec<Tag> = db_tags
            .into_iter()
            .map(|db_tag| Tag {
                id: db_tag.id.id.to_string(),
                text: db_tag.text,
                created_at: *db_tag.created_at,
            })
            .collect();

        Ok(PagedResult::new(
            items,
            total,
            paging.page,
            paging.page_size,
        ))
    }

    /// Get a tag by its text.
    ///
    /// # Arguments
    /// * `text` - The tag text
    ///
    /// # Returns
    /// The Tag if found
    pub async fn get_tag_by_text(&self, text: &str) -> Result<Tag> {
        let query = format!("SELECT * FROM {} WHERE text = $text;", TAGS_TABLE);

        let mut response = self
            .db
            .query(query)
            .bind(("text", text.to_string()))
            .await?;
        let results: Vec<DbTag> = response
            .take(0)
            .map_err(|e| IndexerError::Serialization(e.to_string()))?;

        let db_tag = results
            .into_iter()
            .next()
            .ok_or_else(|| IndexerError::NotFound(format!("Tag '{}' not found", text)))?;

        Ok(Tag {
            id: db_tag.id.id.to_string(),
            text: db_tag.text,
            created_at: *db_tag.created_at,
        })
    }

    // ==================== Export/Import Functions ====================

    /// Export all clipboard entries to a tar.gz archive file.
    ///
    /// This is more memory-efficient for large exports as it writes directly
    /// to disk instead of building the entire archive in memory.
    ///
    /// The archive contains:
    /// - `manifest.json`: Metadata about the export and list of all clips
    /// - `files/`: Directory containing all file attachments
    ///
    /// Short URLs are NOT included in the export.
    ///
    /// # Arguments
    /// * `path` - Path where the tar.gz archive will be written
    pub async fn export_all_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let builder = self.build_export().await?;
        builder.build_to_file(path)
    }

    /// Build an ExportBuilder with all clips and their attachments.
    async fn build_export(&self) -> Result<ExportBuilder> {
        // Get all entries (no filters, large page size to get all)
        let mut all_entries = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let paging = PagingParams::new(page, page_size);
            let result = self.list_entries(SearchFilters::default(), paging).await?;

            if result.items.is_empty() {
                break;
            }

            all_entries.extend(result.items);

            if all_entries.len() >= result.total {
                break;
            }

            page += 1;
        }

        let mut builder = ExportBuilder::new();

        for entry in all_entries {
            let attachment_content = if let Some(ref file_key) = entry.file_attachment {
                self.storage.get_file(file_key).await.ok()
            } else {
                None
            };

            let exported_clip = ExportedClip::from(entry);
            builder.add_clip(exported_clip, attachment_content);
        }

        Ok(builder)
    }

    /// Import clips from a tar.gz archive with deduplication.
    ///
    /// Clips are deduplicated by:
    /// 1. Checking if the same ID already exists
    /// 2. Checking if the same content (hash) already exists
    ///
    /// # Arguments
    /// * `archive_data` - The tar.gz archive data as bytes
    ///
    /// # Returns
    /// An ImportResult containing statistics about the import operation
    pub async fn import_archive(&self, archive_data: &[u8]) -> Result<ImportResult> {
        let parser = ImportParser::from_bytes(archive_data)?;
        self.import_from_parser(parser).await
    }

    /// Import clips from a tar.gz archive file with deduplication.
    ///
    /// This is more memory-efficient for large archives as it streams from disk
    /// instead of requiring the entire archive to be loaded into memory first.
    ///
    /// Clips are deduplicated by:
    /// 1. Checking if the same ID already exists
    /// 2. Checking if the same content (hash) already exists
    ///
    /// # Arguments
    /// * `path` - Path to the tar.gz archive file
    ///
    /// # Returns
    /// An ImportResult containing statistics about the import operation
    pub async fn import_archive_from_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<ImportResult> {
        let parser = ImportParser::from_file(path)?;
        self.import_from_parser(parser).await
    }

    /// Import clips from a parsed archive with deduplication.
    async fn import_from_parser(&self, parser: ImportParser) -> Result<ImportResult> {
        // Get existing IDs and content hashes for deduplication
        let mut existing_ids = HashSet::new();
        let mut existing_content_hashes = HashSet::new();

        let mut page = 1;
        let page_size = 100;

        loop {
            let paging = PagingParams::new(page, page_size);
            let result = self.list_entries(SearchFilters::default(), paging).await?;

            if result.items.is_empty() {
                break;
            }

            for entry in &result.items {
                existing_ids.insert(entry.id.clone());
                let exported = ExportedClip::from(entry.clone());
                existing_content_hashes.insert(calculate_content_hash(&exported));
            }

            if existing_ids.len() >= result.total {
                break;
            }

            page += 1;
        }

        let mut imported_ids = Vec::new();
        let mut skipped_ids = Vec::new();
        let mut attachments_imported = 0;

        for clip in parser.clips() {
            // Check for duplicates
            let content_hash = calculate_content_hash(clip);

            if existing_ids.contains(&clip.id) || existing_content_hashes.contains(&content_hash) {
                skipped_ids.push(clip.id.clone());
                continue;
            }

            // Import the clip
            let has_attachment = clip.attachment_path.is_some();

            if let Some(ref attachment_path) = clip.attachment_path {
                if let Some(attachment_content) = parser.get_attachment(attachment_path) {
                    // Create entry with file attachment
                    let original_filename = clip
                        .original_filename
                        .clone()
                        .unwrap_or_else(|| "attachment".to_string());

                    let mut entry = ClipboardEntry {
                        id: clip.id.clone(),
                        content: clip.content.clone(),
                        created_at: clip.created_at,
                        tags: clip.tags.clone(),
                        additional_notes: clip.additional_notes.clone(),
                        file_attachment: None,
                        original_filename: Some(original_filename.clone()),
                        search_content: match &clip.additional_notes {
                            Some(notes) => format!("{} {}", clip.content, notes),
                            None => clip.content.clone(),
                        },
                    };

                    // Store the file
                    let stored_file_key = self
                        .storage
                        .put_file_bytes(attachment_content, &original_filename)
                        .await?;
                    entry.file_attachment = Some(stored_file_key);

                    // Insert into database
                    self.insert_entry_with_id(&entry).await?;
                    attachments_imported += 1;
                } else if has_attachment {
                    // Attachment expected but not found in archive, import without attachment
                    let entry = ClipboardEntry {
                        id: clip.id.clone(),
                        content: clip.content.clone(),
                        created_at: clip.created_at,
                        tags: clip.tags.clone(),
                        additional_notes: clip.additional_notes.clone(),
                        file_attachment: None,
                        original_filename: clip.original_filename.clone(),
                        search_content: match &clip.additional_notes {
                            Some(notes) => format!("{} {}", clip.content, notes),
                            None => clip.content.clone(),
                        },
                    };
                    self.insert_entry_with_id(&entry).await?;
                }
            } else {
                // No attachment, just insert the text entry
                let entry = ClipboardEntry {
                    id: clip.id.clone(),
                    content: clip.content.clone(),
                    created_at: clip.created_at,
                    tags: clip.tags.clone(),
                    additional_notes: clip.additional_notes.clone(),
                    file_attachment: None,
                    original_filename: None,
                    search_content: match &clip.additional_notes {
                        Some(notes) => format!("{} {}", clip.content, notes),
                        None => clip.content.clone(),
                    },
                };
                self.insert_entry_with_id(&entry).await?;
            }

            imported_ids.push(clip.id.clone());
            existing_ids.insert(clip.id.clone());
            existing_content_hashes.insert(content_hash);
        }

        Ok(ImportResult {
            imported_count: imported_ids.len(),
            skipped_count: skipped_ids.len(),
            attachments_imported,
            imported_ids,
            skipped_ids,
        })
    }

    /// Insert an entry with a specific ID (used during import)
    async fn insert_entry_with_id(&self, entry: &ClipboardEntry) -> Result<()> {
        let record_id = (TABLE_NAME, entry.id.as_str());
        let _: Option<DbClipboardEntry> = self
            .db
            .create(record_id)
            .content(DbClipboardEntry {
                id: surrealdb::sql::Thing::from((TABLE_NAME.to_string(), entry.id.clone())),
                content: entry.content.clone(),
                created_at: surrealdb::sql::Datetime::from(entry.created_at),
                tags: entry.tags.clone(),
                additional_notes: entry.additional_notes.clone(),
                file_attachment: entry.file_attachment.clone(),
                original_filename: entry.original_filename.clone(),
                search_content: entry.search_content.clone(),
            })
            .await?;

        // Sync tags to the tags table
        self.sync_tags(&entry.tags).await?;

        Ok(())
    }
}
