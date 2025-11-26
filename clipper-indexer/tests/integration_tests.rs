use chrono::{Duration, Utc};
use clipper_indexer::{ClipperIndexer, PagingParams, SearchFilters};
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

    assert!(results.items.len() >= 1);
    assert!(results
        .items
        .iter()
        .any(|e| e.tags.contains(&"rust".to_string())));
}
