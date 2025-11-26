use clipper_indexer::{ClipperIndexer, SearchFilters};
use chrono::{Duration, Utc};
use std::fs;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directories for this example
    let db_dir = TempDir::new()?;
    let storage_dir = TempDir::new()?;

    println!("Initializing ClipperIndexer...");
    let indexer = ClipperIndexer::new(db_dir.path(), storage_dir.path()).await?;

    // Example 1: Add a text entry
    println!("\n1. Adding a text entry...");
    let entry1 = indexer
        .add_entry_from_text(
            "Rust is a systems programming language".to_string(),
            vec!["rust".to_string(), "programming".to_string()],
            Some("Great for performance-critical applications".to_string()),
        )
        .await?;
    println!("Created entry with ID: {}", entry1.id);

    // Example 2: Add another text entry
    println!("\n2. Adding another text entry...");
    let entry2 = indexer
        .add_entry_from_text(
            "Python is a versatile programming language".to_string(),
            vec!["python".to_string(), "programming".to_string()],
            Some("Great for rapid development".to_string()),
        )
        .await?;
    println!("Created entry with ID: {}", entry2.id);

    // Example 3: Add entry from file
    println!("\n3. Adding entry from file...");
    let temp_file_dir = TempDir::new()?;
    let file_path = temp_file_dir.path().join("example.txt");
    fs::write(&file_path, "This is a sample file content for testing.")?;

    let entry3 = indexer
        .add_entry_from_file(
            &file_path,
            vec!["file".to_string(), "example".to_string()],
            Some("Test file upload".to_string()),
        )
        .await?;
    println!("Created entry with ID: {} (with file attachment)", entry3.id);

    // Example 4: Retrieve an entry by ID
    println!("\n4. Retrieving entry by ID...");
    let retrieved = indexer.get_entry(&entry1.id).await?;
    println!("Retrieved entry: {}", retrieved.content);
    println!("Tags: {:?}", retrieved.tags);
    println!("Notes: {:?}", retrieved.additional_notes);

    // Example 5: Update an entry
    println!("\n5. Updating entry tags and notes...");
    let updated = indexer
        .update_entry(
            &entry1.id,
            Some(vec![
                "rust".to_string(),
                "systems".to_string(),
                "updated".to_string(),
            ]),
            Some("Updated with new information".to_string()),
        )
        .await?;
    println!("Updated entry tags: {:?}", updated.tags);
    println!("Updated notes: {:?}", updated.additional_notes);

    // Wait a bit for full-text indexing to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Example 6: Full-text search
    println!("\n6. Performing full-text search for 'programming'...");
    let search_results = indexer
        .search_entries("programming", SearchFilters::new())
        .await?;
    println!("Found {} entries matching 'programming':", search_results.len());
    for result in &search_results {
        println!("  - {} (tags: {:?})", result.content, result.tags);
    }

    // Example 7: List entries with tag filter
    println!("\n7. Listing entries with 'rust' tag...");
    let tag_filter = SearchFilters::new().with_tags(vec!["rust".to_string()]);
    let tagged_entries = indexer.list_entries(tag_filter).await?;
    println!("Found {} entries with 'rust' tag:", tagged_entries.len());
    for entry in &tagged_entries {
        println!("  - {}", entry.content);
    }

    // Example 8: List entries with date range
    println!("\n8. Listing entries from the last hour...");
    let now = Utc::now();
    let date_filter =
        SearchFilters::new().with_date_range(now - Duration::hours(1), now + Duration::hours(1));
    let recent_entries = indexer.list_entries(date_filter).await?;
    println!("Found {} entries from the last hour", recent_entries.len());

    // Example 9: Search with combined filters
    println!("\n9. Searching 'programming' with tag filter...");
    let combined_filter = SearchFilters::new()
        .with_tags(vec!["python".to_string()])
        .with_date_range(now - Duration::hours(1), now + Duration::hours(1));
    let filtered_search = indexer.search_entries("programming", combined_filter).await?;
    println!(
        "Found {} entries matching 'programming' with 'python' tag:",
        filtered_search.len()
    );
    for entry in &filtered_search {
        println!("  - {}", entry.content);
    }

    // Example 10: Retrieve file content
    if let Some(file_key) = &entry3.file_attachment {
        println!("\n10. Retrieving file content...");
        let file_content = indexer.get_file_content(file_key).await?;
        let content_str = String::from_utf8_lossy(&file_content);
        println!("File content: {}", content_str);
    }

    // Example 11: Delete an entry
    println!("\n11. Deleting entry...");
    indexer.delete_entry(&entry2.id).await?;
    println!("Deleted entry with ID: {}", entry2.id);

    // Verify deletion
    match indexer.get_entry(&entry2.id).await {
        Ok(_) => println!("Entry still exists (unexpected)"),
        Err(_) => println!("Entry successfully deleted"),
    }

    println!("\n✓ All examples completed successfully!");

    Ok(())
}
