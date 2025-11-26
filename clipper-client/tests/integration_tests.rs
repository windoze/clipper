use clipper_client::{ClipNotification, ClipperClient, SearchFilters};
use std::time::Duration;
use tokio::sync::mpsc;

// Helper to get test server URL from environment or use default
fn test_server_url() -> String {
    std::env::var("TEST_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

// Helper to wait for server to be ready
async fn wait_for_server() {
    let client = reqwest::Client::new();
    let url = format!("{}/health", test_server_url());

    for _ in 0..30 {
        if client.get(&url).send().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("Server did not start in time");
}

#[tokio::test]
async fn test_create_clip() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    let clip = client
        .create_clip(
            "Test content".to_string(),
            vec!["test".to_string(), "example".to_string()],
            Some("Test notes".to_string()),
        )
        .await
        .expect("Failed to create clip");

    assert_eq!(clip.content, "Test content");
    assert_eq!(clip.tags, vec!["test", "example"]);
    assert_eq!(clip.additional_notes, Some("Test notes".to_string()));
    assert!(!clip.id.is_empty());
}

#[tokio::test]
async fn test_create_clip_without_notes() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    let clip = client
        .create_clip(
            "Simple content".to_string(),
            vec!["simple".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    assert_eq!(clip.content, "Simple content");
    assert_eq!(clip.tags, vec!["simple"]);
    assert_eq!(clip.additional_notes, None);
}

#[tokio::test]
async fn test_get_clip() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a clip first
    let created = client
        .create_clip("Get me".to_string(), vec!["findme".to_string()], None)
        .await
        .expect("Failed to create clip");

    // Get the clip
    let retrieved = client
        .get_clip(&created.id)
        .await
        .expect("Failed to get clip");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.content, "Get me");
    assert_eq!(retrieved.tags, vec!["findme"]);
}

#[tokio::test]
async fn test_get_nonexistent_clip() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    let result = client.get_clip("nonexistent123").await;

    assert!(result.is_err());
    match result {
        Err(clipper_client::ClientError::NotFound(_)) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_update_clip() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a clip
    let created = client
        .create_clip(
            "Original content".to_string(),
            vec!["original".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    // Update the clip
    let updated = client
        .update_clip(
            &created.id,
            Some(vec!["updated".to_string(), "new".to_string()]),
            Some("Updated notes".to_string()),
        )
        .await
        .expect("Failed to update clip");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.tags, vec!["updated", "new"]);
    assert_eq!(updated.additional_notes, Some("Updated notes".to_string()));
    assert_eq!(updated.content, "Original content"); // Content unchanged
}

#[tokio::test]
async fn test_update_clip_tags_only() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a clip
    let created = client
        .create_clip("Content".to_string(), vec!["old".to_string()], None)
        .await
        .expect("Failed to create clip");

    // Update only tags
    let updated = client
        .update_clip(&created.id, Some(vec!["new".to_string()]), None)
        .await
        .expect("Failed to update clip");

    assert_eq!(updated.tags, vec!["new"]);
}

#[tokio::test]
async fn test_delete_clip() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a clip
    let created = client
        .create_clip("Delete me".to_string(), vec!["temporary".to_string()], None)
        .await
        .expect("Failed to create clip");

    // Delete the clip
    client
        .delete_clip(&created.id)
        .await
        .expect("Failed to delete clip");

    // Verify it's deleted
    let result = client.get_clip(&created.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_clips() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a few clips
    client
        .create_clip("Clip 1".to_string(), vec!["test".to_string()], None)
        .await
        .expect("Failed to create clip");

    client
        .create_clip("Clip 2".to_string(), vec!["test".to_string()], None)
        .await
        .expect("Failed to create clip");

    // List all clips
    let clips = client
        .list_clips(SearchFilters::new(), 1, 20)
        .await
        .expect("Failed to list clips");

    assert!(clips.items.len() >= 2);
}

#[tokio::test]
async fn test_list_clips_with_tag_filter() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create clips with different tags
    client
        .create_clip(
            "Important clip".to_string(),
            vec!["important".to_string(), "work".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    client
        .create_clip(
            "Personal clip".to_string(),
            vec!["personal".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    // List clips filtered by tag
    let filters = SearchFilters::new().with_tags(vec!["important".to_string()]);
    let clips = client
        .list_clips(filters, 1, 20)
        .await
        .expect("Failed to list clips");

    assert!(clips.items.len() >= 1);
    assert!(clips.items.iter().any(|c| c.content == "Important clip"));
}

#[tokio::test]
async fn test_search_clips() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create clips with searchable content
    client
        .create_clip(
            "The quick brown fox".to_string(),
            vec!["animals".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    client
        .create_clip(
            "The lazy dog".to_string(),
            vec!["animals".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    // Search for clips
    let clips = client
        .search_clips("fox", SearchFilters::new(), 1, 20)
        .await
        .expect("Failed to search clips");

    assert!(clips.items.len() >= 1);
    assert!(clips
        .items
        .iter()
        .any(|c| c.content == "The quick brown fox"));
}

#[tokio::test]
async fn test_search_clips_with_tag_filter() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create clips
    client
        .create_clip(
            "Work document about meetings".to_string(),
            vec!["work".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    client
        .create_clip(
            "Personal notes about meetings".to_string(),
            vec!["personal".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    // Search with tag filter
    let filters = SearchFilters::new().with_tags(vec!["work".to_string()]);
    let clips = client
        .search_clips("meetings", filters, 1, 20)
        .await
        .expect("Failed to search clips");

    assert!(clips.items.len() >= 1);
    assert!(clips
        .items
        .iter()
        .any(|c| c.content == "Work document about meetings"));
}

#[tokio::test]
async fn test_websocket_notifications() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a channel to receive notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to notifications
    let _handle = client
        .subscribe_notifications(tx)
        .await
        .expect("Failed to subscribe to notifications");

    // Give WebSocket time to connect
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create a clip
    let created = client
        .create_clip(
            "Notification test".to_string(),
            vec!["notify".to_string()],
            None,
        )
        .await
        .expect("Failed to create clip");

    // Wait for notification with timeout
    let notification = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for notification")
        .expect("Channel closed");

    match notification {
        ClipNotification::NewClip { id, content, tags } => {
            assert_eq!(id, created.id);
            assert_eq!(content, "Notification test");
            assert_eq!(tags, vec!["notify"]);
        }
        _ => panic!("Expected NewClip notification"),
    }
}

#[tokio::test]
async fn test_websocket_update_notification() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a channel to receive notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to notifications
    let _handle = client
        .subscribe_notifications(tx)
        .await
        .expect("Failed to subscribe to notifications");

    // Give WebSocket time to connect
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create a clip
    let created = client
        .create_clip("Update test".to_string(), vec!["test".to_string()], None)
        .await
        .expect("Failed to create clip");

    // Consume the creation notification
    let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for creation notification");

    // Update the clip
    client
        .update_clip(&created.id, Some(vec!["updated".to_string()]), None)
        .await
        .expect("Failed to update clip");

    // Wait for update notification
    let notification = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for update notification")
        .expect("Channel closed");

    match notification {
        ClipNotification::UpdatedClip { id } => {
            assert_eq!(id, created.id);
        }
        _ => panic!("Expected UpdatedClip notification"),
    }
}

#[tokio::test]
async fn test_websocket_delete_notification() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a channel to receive notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to notifications
    let _handle = client
        .subscribe_notifications(tx)
        .await
        .expect("Failed to subscribe to notifications");

    // Give WebSocket time to connect
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create a clip
    let created = client
        .create_clip("Delete test".to_string(), vec!["test".to_string()], None)
        .await
        .expect("Failed to create clip");

    // Consume the creation notification
    let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for creation notification");

    // Delete the clip
    client
        .delete_clip(&created.id)
        .await
        .expect("Failed to delete clip");

    // Wait for delete notification
    let notification = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for delete notification")
        .expect("Channel closed");

    match notification {
        ClipNotification::DeletedClip { id } => {
            assert_eq!(id, created.id);
        }
        _ => panic!("Expected DeletedClip notification"),
    }
}

#[tokio::test]
async fn test_upload_file() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create file content as a reader
    let file_content = b"This is test file content for upload";
    let reader = std::io::Cursor::new(file_content.to_vec());

    // Upload the file
    let clip = client
        .upload_file(
            reader,
            "test_upload.txt".to_string(),
            vec!["test".to_string(), "upload".to_string()],
            Some("Test file upload".to_string()),
        )
        .await
        .expect("Failed to upload file");

    assert_eq!(clip.content, "This is test file content for upload");
    assert_eq!(clip.tags, vec!["test", "upload"]);
    assert_eq!(clip.additional_notes, Some("Test file upload".to_string()));
    assert!(clip.file_attachment.is_some());
    assert!(!clip.id.is_empty());
}

#[tokio::test]
async fn test_upload_file_without_optional_fields() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create file content as a reader
    let file_content = b"Simple file upload";
    let reader = std::io::Cursor::new(file_content.to_vec());

    // Upload the file without optional fields
    let clip = client
        .upload_file(reader, "simple.txt".to_string(), vec![], None)
        .await
        .expect("Failed to upload file");

    assert_eq!(clip.content, "Simple file upload");
    assert_eq!(clip.tags, Vec::<String>::new());
    assert_eq!(clip.additional_notes, None);
    assert!(clip.file_attachment.is_some());
}

#[tokio::test]
async fn test_upload_binary_file() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create binary content (not valid UTF-8) as a reader
    let file_content = vec![0xFF, 0xFE, 0xFD, 0xFC, 0x00, 0x01, 0x02, 0x03];
    let reader = std::io::Cursor::new(file_content);

    // Upload the binary file
    let clip = client
        .upload_file(
            reader,
            "binary_data.bin".to_string(),
            vec!["binary".to_string()],
            Some("Binary file test".to_string()),
        )
        .await
        .expect("Failed to upload binary file");

    // Content should indicate it's a binary file
    assert!(clip.content.contains("Binary file") || clip.content.contains("binary_data.bin"));
    assert_eq!(clip.tags, vec!["binary"]);
    assert!(clip.file_attachment.is_some());
}

#[tokio::test]
async fn test_upload_file_with_websocket_notification() {
    wait_for_server().await;

    let client = ClipperClient::new(test_server_url());

    // Create a channel to receive notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to notifications
    let _handle = client
        .subscribe_notifications(tx)
        .await
        .expect("Failed to subscribe to notifications");

    // Give WebSocket time to connect
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Upload a file
    let file_content = b"File upload notification test";
    let reader = std::io::Cursor::new(file_content.to_vec());
    let clip = client
        .upload_file(
            reader,
            "notify_test.txt".to_string(),
            vec!["notify".to_string()],
            None,
        )
        .await
        .expect("Failed to upload file");

    // Wait for notification with timeout
    let notification = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for notification")
        .expect("Channel closed");

    match notification {
        ClipNotification::NewClip { id, content, tags } => {
            assert_eq!(id, clip.id);
            assert_eq!(content, "File upload notification test");
            assert_eq!(tags, vec!["notify"]);
        }
        _ => panic!("Expected NewClip notification"),
    }
}
