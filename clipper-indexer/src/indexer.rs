use crate::error::{IndexerError, Result};
use crate::models::{ClipboardEntry, PagedResult, PagingParams, SearchFilters};
use crate::storage::FileStorage;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

const TABLE_NAME: &str = "clipboard";
const NAMESPACE: &str = "clipper";
const DATABASE: &str = "library";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbClipboardEntry {
    id: surrealdb::sql::Thing,
    content: String,
    created_at: surrealdb::sql::Datetime,
    tags: Vec<String>,
    additional_notes: Option<String>,
    file_attachment: Option<String>,
    search_content: String,
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

        // Initialize file storage
        let storage = FileStorage::new(storage_path)?;

        Ok(Self { db, storage })
    }

    async fn initialize_schema(db: &Surreal<Db>) -> Result<()> {
        // Define the clipboard table schema
        let schema_query = format!(
            r#"
            DEFINE TABLE IF NOT EXISTS {} SCHEMAFULL;

            DEFINE FIELD IF NOT EXISTS content ON TABLE {} TYPE string;
            DEFINE FIELD IF NOT EXISTS created_at ON TABLE {} TYPE datetime;
            DEFINE FIELD IF NOT EXISTS tags ON TABLE {} TYPE array<string>;
            DEFINE FIELD IF NOT EXISTS additional_notes ON TABLE {} TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS file_attachment ON TABLE {} TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS search_content ON TABLE {} TYPE string;
            "#,
            TABLE_NAME, TABLE_NAME, TABLE_NAME, TABLE_NAME, TABLE_NAME, TABLE_NAME, TABLE_NAME
        );

        db.query(schema_query).await?;

        // Define indexes
        let index_query = format!(
            r#"
            DEFINE INDEX IF NOT EXISTS idx_created_at ON TABLE {} COLUMNS created_at;
            DEFINE INDEX IF NOT EXISTS idx_tags ON TABLE {} COLUMNS tags;
            DEFINE ANALYZER IF NOT EXISTS clipper_analyzer TOKENIZERS blank,class FILTERS lowercase,snowball(english);
            DEFINE INDEX IF NOT EXISTS idx_search_content ON TABLE {} COLUMNS search_content
                SEARCH ANALYZER clipper_analyzer BM25 HIGHLIGHTS;
            "#,
            TABLE_NAME, TABLE_NAME, TABLE_NAME
        );

        db.query(index_query).await?;

        Ok(())
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
                search_content: entry.search_content.clone(),
            })
            .await?;

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

        // Store the file using object_store
        let stored_file_key = self.storage.put_file(file_path).await?;

        // Read file content for search indexing
        let file_content = tokio::fs::read_to_string(file_path)
            .await
            .unwrap_or_else(|_| format!("Binary file: {}", file_path.display()));

        let mut entry = ClipboardEntry::new(file_content, tags);
        entry = entry.with_file_attachment(stored_file_key);

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
                search_content: entry.search_content.clone(),
            })
            .await?;

        Ok(entry)
    }

    pub async fn add_entry_from_file_content(
        &self,
        file_content: bytes::Bytes,
        original_filename: String,
        tags: Vec<String>,
        additional_notes: Option<String>,
    ) -> Result<ClipboardEntry> {
        // Store the file using object_store
        let stored_file_key = self
            .storage
            .put_file_bytes(file_content.clone(), &original_filename)
            .await?;

        // Try to read file content as text for search indexing
        let text_content = String::from_utf8(file_content.to_vec())
            .unwrap_or_else(|_| format!("Binary file: {}", original_filename));

        let mut entry = ClipboardEntry::new(text_content, tags);
        entry = entry.with_file_attachment(stored_file_key);

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
                search_content: entry.search_content.clone(),
            })
            .await?;

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
        let mut query_string = format!("UPDATE {}:{} SET ", TABLE_NAME, id);

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

        query_string.push_str(&updates.join(", "));
        query_string.push(';');

        let mut query = self.db.query(query_string);

        if let Some(t) = tags {
            query = query.bind(("tags", t));
        }

        if let Some(notes) = additional_notes {
            query = query.bind(("additional_notes", notes));
            query = query.bind(("search_content", new_search_content));
        }

        query.await?;

        // Return updated entry
        self.get_entry(id).await
    }

    pub async fn search_entries(
        &self,
        search_query: &str,
        filters: SearchFilters,
        paging: PagingParams,
    ) -> Result<PagedResult<ClipboardEntry>> {
        let mut where_clauses = vec![format!("search_content @@ '{}'", search_query)];

        if let Some(start_date) = filters.start_date {
            where_clauses.push(format!(
                "created_at >= <datetime>'{}'",
                start_date.to_rfc3339()
            ));
        }

        if let Some(end_date) = filters.end_date {
            where_clauses.push(format!(
                "created_at <= <datetime>'{}'",
                end_date.to_rfc3339()
            ));
        }

        if let Some(tags) = filters.tags {
            if !tags.is_empty() {
                let tag_conditions: Vec<String> = tags
                    .iter()
                    .map(|tag| format!("'{}' IN tags", tag))
                    .collect();
                where_clauses.push(format!("({})", tag_conditions.join(" OR ")));
            }
        }

        let where_clause = where_clauses.join(" AND ");

        // Get total count
        let count_query = format!(
            "SELECT count() FROM {} WHERE {} GROUP ALL;",
            TABLE_NAME, where_clause
        );
        let mut count_response = self.db.query(count_query).await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let total = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Get paginated results
        let query = format!(
            "SELECT * FROM {} WHERE {} ORDER BY created_at DESC LIMIT {} START {};",
            TABLE_NAME,
            where_clause,
            paging.page_size,
            paging.offset()
        );

        let mut response = self.db.query(query).await?;

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

    pub async fn list_entries(
        &self,
        filters: SearchFilters,
        paging: PagingParams,
    ) -> Result<PagedResult<ClipboardEntry>> {
        let mut where_clauses = Vec::new();

        if let Some(start_date) = filters.start_date {
            where_clauses.push(format!(
                "created_at >= <datetime>'{}'",
                start_date.to_rfc3339()
            ));
        }

        if let Some(end_date) = filters.end_date {
            where_clauses.push(format!(
                "created_at <= <datetime>'{}'",
                end_date.to_rfc3339()
            ));
        }

        if let Some(tags) = filters.tags {
            if !tags.is_empty() {
                let tag_conditions: Vec<String> = tags
                    .iter()
                    .map(|tag| format!("'{}' IN tags", tag))
                    .collect();
                where_clauses.push(format!("({})", tag_conditions.join(" OR ")));
            }
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

        let mut count_response = self.db.query(count_query).await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let count_results: Vec<CountResult> = count_response.take(0).unwrap_or_default();
        let total = count_results.first().map(|c| c.count as usize).unwrap_or(0);

        // Get paginated results
        let query = if where_clauses.is_empty() {
            format!(
                "SELECT * FROM {} ORDER BY created_at DESC LIMIT {} START {};",
                TABLE_NAME,
                paging.page_size,
                paging.offset()
            )
        } else {
            let where_clause = where_clauses.join(" AND ");
            format!(
                "SELECT * FROM {} WHERE {} ORDER BY created_at DESC LIMIT {} START {};",
                TABLE_NAME,
                where_clause,
                paging.page_size,
                paging.offset()
            )
        };

        let mut response = self.db.query(query).await?;

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
        let query = format!("DELETE {}:{};", TABLE_NAME, id);
        self.db.query(query).await?;

        Ok(())
    }
}
