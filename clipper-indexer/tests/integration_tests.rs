use chrono::{Duration, Utc};
use clipper_indexer::{ClipperIndexer, IndexerError, PagingParams, SearchFilters};
use std::fs;
use tempfile::TempDir;

async fn setup_test_indexer() -> (ClipperIndexer, TempDir, TempDir) {
    let db_dir = TempDir::new().unwrap();
    let storage_dir = TempDir::new().unwrap();

    let indexer = ClipperIndexer::new(db_dir.path(), storage_dir.path())
        .await
        .expect("Failed to create indexer");

    (indexer, db_dir, storage_dir)
}

#[tokio::test]
async fn test_add_entry_from_text() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let entry = indexer
        .add_entry_from_text(
            "Hello, World!".to_string(),
            vec!["greeting".to_string()],
            Some("This is a test note".to_string()),
        )
        .await
        .expect("Failed to add entry");

    assert_eq!(entry.content, "Hello, World!");
    assert_eq!(entry.tags, vec!["greeting"]);
    assert_eq!(
        entry.additional_notes,
        Some("This is a test note".to_string())
    );
    assert_eq!(entry.search_content, "Hello, World! This is a test note");
}

#[tokio::test]
async fn test_get_entry() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let entry = indexer
        .add_entry_from_text("Test content".to_string(), vec!["test".to_string()], None)
        .await
        .expect("Failed to add entry");

    let retrieved = indexer
        .get_entry(&entry.id)
        .await
        .expect("Failed to get entry");

    assert_eq!(retrieved.id, entry.id);
    assert_eq!(retrieved.content, "Test content");
    assert_eq!(retrieved.tags, vec!["test"]);
}

#[tokio::test]
async fn test_update_entry() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let entry = indexer
        .add_entry_from_text(
            "Original content".to_string(),
            vec!["original".to_string()],
            None,
        )
        .await
        .expect("Failed to add entry");

    let updated = indexer
        .update_entry(
            &entry.id,
            Some(vec!["updated".to_string(), "test".to_string()]),
            Some("Updated notes".to_string()),
        )
        .await
        .expect("Failed to update entry");

    assert_eq!(updated.tags, vec!["updated", "test"]);
    assert_eq!(updated.additional_notes, Some("Updated notes".to_string()));
    assert_eq!(updated.search_content, "Original content Updated notes");
}

#[tokio::test]
async fn test_add_entry_from_file() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a temporary file
    let temp_file = TempDir::new().unwrap();
    let file_path = temp_file.path().join("test.txt");
    fs::write(&file_path, "File content for testing").unwrap();

    let entry = indexer
        .add_entry_from_file(&file_path, vec!["file".to_string()], None)
        .await
        .expect("Failed to add entry from file");

    assert!(entry.file_attachment.is_some());
    assert_eq!(entry.content, "File content for testing");
    assert_eq!(entry.tags, vec!["file"]);

    // Verify file can be retrieved
    let file_content = indexer
        .get_file_content(entry.file_attachment.as_ref().unwrap())
        .await
        .expect("Failed to get file content");

    assert_eq!(
        String::from_utf8(file_content.to_vec()).unwrap(),
        "File content for testing"
    );
}

#[tokio::test]
async fn test_search_entries() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add some test entries
    indexer
        .add_entry_from_text(
            "Rust programming is fun".to_string(),
            vec!["rust".to_string()],
            None,
        )
        .await
        .unwrap();

    indexer
        .add_entry_from_text(
            "Python is also fun".to_string(),
            vec!["python".to_string()],
            None,
        )
        .await
        .unwrap();

    indexer
        .add_entry_from_text(
            "Programming languages comparison".to_string(),
            vec!["comparison".to_string()],
            Some("Rust vs Python".to_string()),
        )
        .await
        .unwrap();

    // Wait a bit for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Search for "programming"
    let paging_params = PagingParams::default();
    let results = indexer
        .search_entries("programming", SearchFilters::new(), paging_params)
        .await
        .expect("Failed to search");

    assert!(
        results.total >= 2,
        "Expected at least 2 results, got {}",
        results.total
    );
}

#[tokio::test]
async fn test_list_entries_with_date_range() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let now = Utc::now();

    // Add entries
    indexer
        .add_entry_from_text("Recent entry".to_string(), vec!["recent".to_string()], None)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    indexer
        .add_entry_from_text(
            "Another entry".to_string(),
            vec!["another".to_string()],
            None,
        )
        .await
        .unwrap();

    // List all entries
    let paging_params = PagingParams::default();
    let all_entries = indexer
        .list_entries(SearchFilters::new(), paging_params)
        .await
        .expect("Failed to list entries");

    assert_eq!(all_entries.total, 2);

    // List with date range
    let filters =
        SearchFilters::new().with_date_range(now - Duration::hours(1), now + Duration::hours(1));
    let paging_params = PagingParams::default();

    let filtered_entries = indexer
        .list_entries(filters, paging_params)
        .await
        .expect("Failed to list entries with filter");

    assert_eq!(filtered_entries.total, 2);
}

#[tokio::test]
async fn test_list_entries_with_tag_filter() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add entries with different tags
    indexer
        .add_entry_from_text(
            "Entry 1".to_string(),
            vec!["tag1".to_string(), "common".to_string()],
            None,
        )
        .await
        .unwrap();

    indexer
        .add_entry_from_text(
            "Entry 2".to_string(),
            vec!["tag2".to_string(), "common".to_string()],
            None,
        )
        .await
        .unwrap();

    indexer
        .add_entry_from_text("Entry 3".to_string(), vec!["tag3".to_string()], None)
        .await
        .unwrap();

    // Filter by specific tag
    let filters = SearchFilters::new().with_tags(vec!["tag1".to_string()]);
    let paging_params = PagingParams::default();

    let filtered = indexer
        .list_entries(filters, paging_params)
        .await
        .expect("Failed to list entries");

    assert_eq!(filtered.items.len(), 1);
    assert_eq!(filtered.items[0].content, "Entry 1");

    // Filter by common tag
    let filters = SearchFilters::new().with_tags(vec!["common".to_string()]);
    let paging_params = PagingParams::default();

    let filtered = indexer
        .list_entries(filters, paging_params)
        .await
        .expect("Failed to list entries");

    assert_eq!(filtered.total, 2);
}

#[tokio::test]
async fn test_delete_entry() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let entry = indexer
        .add_entry_from_text(
            "To be deleted".to_string(),
            vec!["delete".to_string()],
            None,
        )
        .await
        .unwrap();

    // Verify entry exists
    let retrieved = indexer.get_entry(&entry.id).await;
    assert!(retrieved.is_ok());

    // Delete the entry
    indexer
        .delete_entry(&entry.id)
        .await
        .expect("Failed to delete entry");

    // Verify entry no longer exists
    let result = indexer.get_entry(&entry.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_with_combined_filters() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let now = Utc::now();

    // Add entries
    indexer
        .add_entry_from_text(
            "Rust is awesome".to_string(),
            vec!["rust".to_string()],
            None,
        )
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    indexer
        .add_entry_from_text(
            "Rust programming tips".to_string(),
            vec!["rust".to_string(), "tips".to_string()],
            None,
        )
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    indexer
        .add_entry_from_text(
            "Python programming".to_string(),
            vec!["python".to_string()],
            None,
        )
        .await
        .unwrap();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Search with tag filter
    let filters = SearchFilters::new()
        .with_tags(vec!["rust".to_string()])
        .with_date_range(now - Duration::hours(1), now + Duration::hours(1));
    let paging_params = PagingParams::default();

    let results = indexer
        .search_entries("programming", filters, paging_params)
        .await
        .expect("Failed to search");

    assert!(!results.items.is_empty());
    assert!(results
        .items
        .iter()
        .any(|e| e.tags.contains(&"rust".to_string())));
}

#[tokio::test]
async fn test_cleanup_entries_no_tags() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add entry with no tags
    let entry_no_tags = indexer
        .add_entry_from_text("No tags entry".to_string(), vec![], None)
        .await
        .unwrap();

    // Add entry with meaningful tag
    let entry_with_tag = indexer
        .add_entry_from_text(
            "With tag entry".to_string(),
            vec!["important".to_string()],
            None,
        )
        .await
        .unwrap();

    // Cleanup should delete the entry with no tags
    let deleted_ids = indexer.cleanup_entries(None, None).await.unwrap();

    assert_eq!(deleted_ids.len(), 1);
    assert!(deleted_ids.contains(&entry_no_tags.id));

    // Verify entry with no tags is deleted
    let result = indexer.get_entry(&entry_no_tags.id).await;
    assert!(result.is_err());

    // Verify entry with tag still exists
    let result = indexer.get_entry(&entry_with_tag.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cleanup_entries_only_host_tag() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add entry with only host tag
    let entry_host_only = indexer
        .add_entry_from_text(
            "Host only entry".to_string(),
            vec!["$host:my-machine".to_string()],
            None,
        )
        .await
        .unwrap();

    // Add entry with host tag and other tag
    let entry_host_and_other = indexer
        .add_entry_from_text(
            "Host and other entry".to_string(),
            vec!["$host:my-machine".to_string(), "important".to_string()],
            None,
        )
        .await
        .unwrap();

    // Cleanup should delete entry with only host tag
    let deleted_ids = indexer.cleanup_entries(None, None).await.unwrap();

    assert_eq!(deleted_ids.len(), 1);
    assert!(deleted_ids.contains(&entry_host_only.id));

    // Verify entry with only host tag is deleted
    let result = indexer.get_entry(&entry_host_only.id).await;
    assert!(result.is_err());

    // Verify entry with host + other tag still exists
    let result = indexer.get_entry(&entry_host_and_other.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cleanup_entries_multiple_host_tags() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add entry with multiple host tags but no other tags
    let entry_multi_host = indexer
        .add_entry_from_text(
            "Multiple hosts entry".to_string(),
            vec![
                "$host:machine1".to_string(),
                "$host:machine2".to_string(),
            ],
            None,
        )
        .await
        .unwrap();

    // Add entry with meaningful tag
    let entry_with_tag = indexer
        .add_entry_from_text(
            "With tag entry".to_string(),
            vec!["favorite".to_string()],
            None,
        )
        .await
        .unwrap();

    // Cleanup should delete entry with only host tags
    let deleted_ids = indexer.cleanup_entries(None, None).await.unwrap();

    assert_eq!(deleted_ids.len(), 1);
    assert!(deleted_ids.contains(&entry_multi_host.id));

    // Verify entry with only host tags is deleted
    let result = indexer.get_entry(&entry_multi_host.id).await;
    assert!(result.is_err());

    // Verify entry with meaningful tag still exists
    let result = indexer.get_entry(&entry_with_tag.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cleanup_entries_with_date_range() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    let now = Utc::now();

    // Add entry with no tags
    let entry1 = indexer
        .add_entry_from_text("Entry 1 no tags".to_string(), vec![], None)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mid_time = Utc::now();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Add another entry with no tags
    let entry2 = indexer
        .add_entry_from_text("Entry 2 no tags".to_string(), vec![], None)
        .await
        .unwrap();

    // Cleanup only entries before mid_time
    let deleted_ids = indexer
        .cleanup_entries(Some(now - Duration::hours(1)), Some(mid_time))
        .await
        .unwrap();

    assert_eq!(deleted_ids.len(), 1);
    assert!(deleted_ids.contains(&entry1.id));

    // Verify first entry is deleted
    let result = indexer.get_entry(&entry1.id).await;
    assert!(result.is_err());

    // Verify second entry still exists (outside date range)
    let result = indexer.get_entry(&entry2.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cleanup_entries_with_file_attachment() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a temporary file
    let temp_file = TempDir::new().unwrap();
    let file_path = temp_file.path().join("test.txt");
    fs::write(&file_path, "File content to be cleaned up").unwrap();

    // Add file entry with only host tag
    let entry = indexer
        .add_entry_from_file(&file_path, vec!["$host:test-machine".to_string()], None)
        .await
        .unwrap();

    let file_key = entry.file_attachment.clone().unwrap();

    // Verify file exists in storage
    let file_content = indexer.get_file_content(&file_key).await;
    assert!(file_content.is_ok());

    // Cleanup should delete the entry and its file
    let deleted_ids = indexer.cleanup_entries(None, None).await.unwrap();

    assert_eq!(deleted_ids.len(), 1);
    assert!(deleted_ids.contains(&entry.id));

    // Verify entry is deleted
    let result = indexer.get_entry(&entry.id).await;
    assert!(result.is_err());

    // Verify file is also deleted from storage
    let file_content = indexer.get_file_content(&file_key).await;
    assert!(file_content.is_err());
}

#[tokio::test]
async fn test_cleanup_entries_none_to_delete() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add entries all with meaningful tags
    indexer
        .add_entry_from_text(
            "Entry 1".to_string(),
            vec!["important".to_string()],
            None,
        )
        .await
        .unwrap();

    indexer
        .add_entry_from_text(
            "Entry 2".to_string(),
            vec!["$host:machine".to_string(), "favorite".to_string()],
            None,
        )
        .await
        .unwrap();

    // Cleanup should delete nothing
    let deleted_ids = indexer.cleanup_entries(None, None).await.unwrap();

    assert!(deleted_ids.is_empty());

    // Verify all entries still exist
    let paging = PagingParams::default();
    let all_entries = indexer
        .list_entries(SearchFilters::new(), paging)
        .await
        .unwrap();

    assert_eq!(all_entries.total, 2);
}

// ==================== Short URL Tests ====================

#[tokio::test]
async fn test_create_short_url() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip first
    let entry = indexer
        .add_entry_from_text(
            "Test content for short URL".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create a short URL for the clip
    let short_url = indexer
        .create_short_url(&entry.id, None)
        .await
        .expect("Failed to create short URL");

    assert_eq!(short_url.clip_id, entry.id);
    assert!(!short_url.short_code.is_empty());
    assert_eq!(short_url.short_code.len(), 8);
    assert!(short_url.expires_at.is_none());
}

#[tokio::test]
async fn test_create_short_url_with_expiration() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip first
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create a short URL with expiration
    let expires_at = Utc::now() + Duration::hours(24);
    let short_url = indexer
        .create_short_url(&entry.id, Some(expires_at))
        .await
        .expect("Failed to create short URL");

    assert_eq!(short_url.clip_id, entry.id);
    assert!(short_url.expires_at.is_some());
    assert!(!short_url.is_expired());
}

#[tokio::test]
async fn test_create_short_url_for_nonexistent_clip() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Try to create a short URL for a nonexistent clip
    let result = indexer
        .create_short_url("nonexistent-clip-id", None)
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IndexerError::NotFound(_)));
}

#[tokio::test]
async fn test_get_short_url() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip and short URL
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    let created_short_url = indexer
        .create_short_url(&entry.id, None)
        .await
        .unwrap();

    // Get the short URL by short code
    let retrieved_short_url = indexer
        .get_short_url(&created_short_url.short_code)
        .await
        .expect("Failed to get short URL");

    assert_eq!(retrieved_short_url.id, created_short_url.id);
    assert_eq!(retrieved_short_url.clip_id, entry.id);
    assert_eq!(retrieved_short_url.short_code, created_short_url.short_code);
}

#[tokio::test]
async fn test_get_short_url_not_found() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Try to get a nonexistent short URL
    let result = indexer.get_short_url("nonexistent").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IndexerError::NotFound(_)));
}

#[tokio::test]
async fn test_get_expired_short_url() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create a short URL that's already expired
    let expires_at = Utc::now() - Duration::hours(1);
    let short_url = indexer
        .create_short_url(&entry.id, Some(expires_at))
        .await
        .unwrap();

    // Try to get the expired short URL
    let result = indexer.get_short_url(&short_url.short_code).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IndexerError::ShortUrlExpired(_)));
}

#[tokio::test]
async fn test_get_short_urls_for_clip() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create multiple short URLs for the same clip
    let short_url1 = indexer.create_short_url(&entry.id, None).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let short_url2 = indexer.create_short_url(&entry.id, None).await.unwrap();

    // Get all short URLs for the clip
    let short_urls = indexer
        .get_short_urls_for_clip(&entry.id)
        .await
        .expect("Failed to get short URLs");

    assert_eq!(short_urls.len(), 2);
    // Should be ordered by created_at DESC, so short_url2 should be first
    assert_eq!(short_urls[0].id, short_url2.id);
    assert_eq!(short_urls[1].id, short_url1.id);
}

#[tokio::test]
async fn test_delete_short_url() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip and short URL
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    let short_url = indexer.create_short_url(&entry.id, None).await.unwrap();

    // Delete the short URL
    indexer
        .delete_short_url(&short_url.id)
        .await
        .expect("Failed to delete short URL");

    // Verify it's deleted
    let result = indexer.get_short_url(&short_url.short_code).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_short_urls_for_clip() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create multiple short URLs
    let short_url1 = indexer.create_short_url(&entry.id, None).await.unwrap();
    let short_url2 = indexer.create_short_url(&entry.id, None).await.unwrap();

    // Delete all short URLs for the clip
    let deleted_count = indexer
        .delete_short_urls_for_clip(&entry.id)
        .await
        .expect("Failed to delete short URLs");

    assert_eq!(deleted_count, 2);

    // Verify they're deleted
    let result1 = indexer.get_short_url(&short_url1.short_code).await;
    let result2 = indexer.get_short_url(&short_url2.short_code).await;
    assert!(result1.is_err());
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_cleanup_expired_short_urls() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create an expired short URL
    let expired_at = Utc::now() - Duration::hours(1);
    let expired_short_url = indexer
        .create_short_url(&entry.id, Some(expired_at))
        .await
        .unwrap();

    // Create a non-expired short URL
    let future_at = Utc::now() + Duration::hours(24);
    let valid_short_url = indexer
        .create_short_url(&entry.id, Some(future_at))
        .await
        .unwrap();

    // Create a short URL with no expiration
    let no_expiry_short_url = indexer.create_short_url(&entry.id, None).await.unwrap();

    // Cleanup expired short URLs
    let cleaned_up = indexer
        .cleanup_expired_short_urls()
        .await
        .expect("Failed to cleanup expired short URLs");

    assert_eq!(cleaned_up, 1);

    // Verify expired one is deleted (will error because not found)
    let result = indexer.get_short_url(&expired_short_url.short_code).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IndexerError::NotFound(_)));

    // Verify valid one still exists
    let result = indexer.get_short_url(&valid_short_url.short_code).await;
    assert!(result.is_ok());

    // Verify no expiry one still exists
    let result = indexer.get_short_url(&no_expiry_short_url.short_code).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_short_url_unique_codes() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Create a clip
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test".to_string()],
            None,
        )
        .await
        .unwrap();

    // Create multiple short URLs and verify all codes are unique
    let mut short_codes = Vec::new();
    for _ in 0..10 {
        let short_url = indexer.create_short_url(&entry.id, None).await.unwrap();
        assert!(!short_codes.contains(&short_url.short_code));
        short_codes.push(short_url.short_code);
    }

    assert_eq!(short_codes.len(), 10);
}

// ==================== Tags Tests ====================

#[tokio::test]
async fn test_tags_synced_on_add_entry() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add an entry with tags
    indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["rust".to_string(), "programming".to_string()],
            None,
        )
        .await
        .expect("Failed to add entry");

    // List tags and verify they were synced
    let tags = indexer
        .list_tags(PagingParams::default())
        .await
        .expect("Failed to list tags");

    assert_eq!(tags.total, 2);
    let tag_texts: Vec<&str> = tags.items.iter().map(|t| t.text.as_str()).collect();
    assert!(tag_texts.contains(&"rust"));
    assert!(tag_texts.contains(&"programming"));
}

#[tokio::test]
async fn test_tags_synced_on_update_entry() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add an entry with initial tags
    let entry = indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["initial".to_string()],
            None,
        )
        .await
        .expect("Failed to add entry");

    // Update with new tags
    indexer
        .update_entry(
            &entry.id,
            Some(vec!["initial".to_string(), "updated".to_string()]),
            None,
        )
        .await
        .expect("Failed to update entry");

    // List tags and verify new tag was synced
    let tags = indexer
        .list_tags(PagingParams::default())
        .await
        .expect("Failed to list tags");

    assert_eq!(tags.total, 2);
    let tag_texts: Vec<&str> = tags.items.iter().map(|t| t.text.as_str()).collect();
    assert!(tag_texts.contains(&"initial"));
    assert!(tag_texts.contains(&"updated"));
}

#[tokio::test]
async fn test_tags_deduplication() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add multiple entries with overlapping tags
    indexer
        .add_entry_from_text(
            "Test 1".to_string(),
            vec!["common".to_string(), "unique1".to_string()],
            None,
        )
        .await
        .expect("Failed to add entry 1");

    indexer
        .add_entry_from_text(
            "Test 2".to_string(),
            vec!["common".to_string(), "unique2".to_string()],
            None,
        )
        .await
        .expect("Failed to add entry 2");

    // List tags - "common" should only appear once
    let tags = indexer
        .list_tags(PagingParams::default())
        .await
        .expect("Failed to list tags");

    assert_eq!(tags.total, 3); // common, unique1, unique2
    let tag_texts: Vec<&str> = tags.items.iter().map(|t| t.text.as_str()).collect();
    assert!(tag_texts.contains(&"common"));
    assert!(tag_texts.contains(&"unique1"));
    assert!(tag_texts.contains(&"unique2"));
}

#[tokio::test]
async fn test_search_tags() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add entries with various tags
    indexer
        .add_entry_from_text(
            "Test".to_string(),
            vec![
                "rust-programming".to_string(),
                "typescript".to_string(),
                "database".to_string(),
            ],
            None,
        )
        .await
        .expect("Failed to add entry");

    // Search for tags containing "rust"
    let results = indexer
        .search_tags("rust", PagingParams::default())
        .await
        .expect("Failed to search tags");

    assert!(results.total >= 1);
    let tag_texts: Vec<&str> = results.items.iter().map(|t| t.text.as_str()).collect();
    assert!(tag_texts.contains(&"rust-programming"));
}

#[tokio::test]
async fn test_get_tag_by_text() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add an entry with a tag
    indexer
        .add_entry_from_text(
            "Test content".to_string(),
            vec!["test-tag".to_string()],
            None,
        )
        .await
        .expect("Failed to add entry");

    // Get the tag by text
    let tag = indexer
        .get_tag_by_text("test-tag")
        .await
        .expect("Failed to get tag");

    assert_eq!(tag.text, "test-tag");
}

#[tokio::test]
async fn test_get_tag_by_text_not_found() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Try to get a nonexistent tag
    let result = indexer.get_tag_by_text("nonexistent").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IndexerError::NotFound(_)));
}

#[tokio::test]
async fn test_list_tags_pagination() {
    let (indexer, _db_dir, _storage_dir) = setup_test_indexer().await;

    // Add an entry with many tags
    let tags: Vec<String> = (0..25).map(|i| format!("tag{:02}", i)).collect();
    indexer
        .add_entry_from_text("Test".to_string(), tags, None)
        .await
        .expect("Failed to add entry");

    // Test pagination
    let page1 = indexer
        .list_tags(PagingParams::new(1, 10))
        .await
        .expect("Failed to list tags page 1");

    assert_eq!(page1.items.len(), 10);
    assert_eq!(page1.total, 25);
    assert_eq!(page1.page, 1);
    assert_eq!(page1.total_pages, 3);

    let page2 = indexer
        .list_tags(PagingParams::new(2, 10))
        .await
        .expect("Failed to list tags page 2");

    assert_eq!(page2.items.len(), 10);
    assert_eq!(page2.page, 2);

    let page3 = indexer
        .list_tags(PagingParams::new(3, 10))
        .await
        .expect("Failed to list tags page 3");

    assert_eq!(page3.items.len(), 5);
    assert_eq!(page3.page, 3);
}
