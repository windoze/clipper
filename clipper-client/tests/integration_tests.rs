use axum::{middleware, routing::get, Router};
use clipper_indexer::ClipperIndexer;
use clipper_client::{ClipNotification, ClipperClient, SearchFilters};
use clipper_server::{api, auth_middleware, websocket, AppState, ServerConfig};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

struct TestServer {
    base_url: String,
    bypass_proxy: bool,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
    _temp_dir: Option<TempDir>,
}

impl TestServer {
    async fn start() -> Self {
        if let Ok(base_url) = std::env::var("TEST_SERVER_URL") {
            let server = Self {
                base_url: base_url.trim_end_matches('/').to_string(),
                bypass_proxy: false,
                shutdown_tx: None,
                handle: None,
                _temp_dir: None,
            };
            wait_for_server_url(&server.base_url, server.bypass_proxy).await;
            return server;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("db");
        let storage_path = temp_dir.path().join("storage");

        let indexer = ClipperIndexer::new(&db_path, &storage_path)
            .await
            .expect("Failed to create indexer");

        let config = ServerConfig::default();
        let state = AppState::new(indexer, config.clone());
        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .merge(api::routes(config.upload.max_size_bytes))
            .merge(websocket::routes())
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind test server");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("Failed to read test server addr")
        );

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
            {
                eprintln!("Test server failed: {}", error);
            }
        });

        let server = Self {
            base_url,
            bypass_proxy: true,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
            _temp_dir: Some(temp_dir),
        };
        wait_for_server_url(&server.base_url, server.bypass_proxy).await;
        server
    }

    fn url(&self) -> String {
        self.base_url.clone()
    }

    fn client(&self) -> ClipperClient {
        if self.bypass_proxy {
            let http_client = reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("Failed to create no-proxy reqwest client");
            ClipperClient::new_with_http_client(self.url(), http_client)
        } else {
            ClipperClient::new(self.url())
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        if let Some(handle) = &self.handle {
            let deadline = Instant::now() + Duration::from_millis(500);
            while !handle.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if !handle.is_finished() {
                handle.abort();
            }
        }
    }
}

async fn test_client() -> (TestServer, ClipperClient) {
    let server = TestServer::start().await;
    let client = server.client();
    (server, client)
}

async fn wait_for_server_url(base_url: &str, bypass_proxy: bool) {
    let client = if bypass_proxy {
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("Failed to create no-proxy readiness client")
    } else {
        reqwest::Client::new()
    };
    let url = format!("{}/health", base_url);
    let mut last_error = String::from("no attempts made");

    for _ in 0..100 {
        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                if status.is_success() && body.trim() == "OK" {
                    return;
                }
                last_error = format!("last response was {status} with body {body:?}");
            }
            Err(error) => {
                last_error = error.to_string();
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!(
        "Clipper server at {} did not become ready: {}",
        base_url, last_error
    );
}

#[tokio::test]
async fn test_create_clip() {
    let (_server, client) = test_client().await;

    let clip = client
        .create_clip(
            "Test content".to_string(),
            vec!["test".to_string(), "example".to_string()],
            Some("Test notes".to_string()),
            None,
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
    let (_server, client) = test_client().await;

    let clip = client
        .create_clip(
            "Simple content".to_string(),
            vec!["simple".to_string()],
            None,
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
    let (_server, client) = test_client().await;

    // Create a clip first
    let created = client
        .create_clip("Get me".to_string(), vec!["findme".to_string()], None, None)
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
    let (_server, client) = test_client().await;

    let result = client.get_clip("nonexistent123").await;

    assert!(result.is_err());
    match result {
        Err(clipper_client::ClientError::NotFound(_)) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_update_clip() {
    let (_server, client) = test_client().await;

    // Create a clip
    let created = client
        .create_clip(
            "Original content".to_string(),
            vec!["original".to_string()],
            None,
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
            None,
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
    let (_server, client) = test_client().await;

    // Create a clip
    let created = client
        .create_clip("Content".to_string(), vec!["old".to_string()], None, None)
        .await
        .expect("Failed to create clip");

    // Update only tags
    let updated = client
        .update_clip(&created.id, Some(vec!["new".to_string()]), None, None)
        .await
        .expect("Failed to update clip");

    assert_eq!(updated.tags, vec!["new"]);
}

#[tokio::test]
async fn test_delete_clip() {
    let (_server, client) = test_client().await;

    // Create a clip
    let created = client
        .create_clip("Delete me".to_string(), vec!["temporary".to_string()], None, None)
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
    let (_server, client) = test_client().await;

    // Create a few clips
    client
        .create_clip("Clip 1".to_string(), vec!["test".to_string()], None, None)
        .await
        .expect("Failed to create clip");

    client
        .create_clip("Clip 2".to_string(), vec!["test".to_string()], None, None)
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
    let (_server, client) = test_client().await;

    // Create clips with different tags
    client
        .create_clip(
            "Important clip".to_string(),
            vec!["important".to_string(), "work".to_string()],
            None,
            None,
        )
        .await
        .expect("Failed to create clip");

    client
        .create_clip(
            "Personal clip".to_string(),
            vec!["personal".to_string()],
            None,
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

    assert!(!clips.items.is_empty());
    assert!(clips.items.iter().any(|c| c.content == "Important clip"));
}

#[tokio::test]
async fn test_search_clips() {
    let (_server, client) = test_client().await;

    // Create clips with searchable content
    client
        .create_clip(
            "The quick brown fox".to_string(),
            vec!["animals".to_string()],
            None,
            None,
        )
        .await
        .expect("Failed to create clip");

    client
        .create_clip(
            "The lazy dog".to_string(),
            vec!["animals".to_string()],
            None,
            None,
        )
        .await
        .expect("Failed to create clip");

    // Search for clips
    let clips = client
        .search_clips("fox", SearchFilters::new(), 1, 20)
        .await
        .expect("Failed to search clips");

    assert!(!clips.items.is_empty());
    assert!(clips
        .items
        .iter()
        .any(|c| c.content == "The quick brown fox"));
}

#[tokio::test]
async fn test_search_clips_with_tag_filter() {
    let (_server, client) = test_client().await;

    // Create clips
    client
        .create_clip(
            "Work document about meetings".to_string(),
            vec!["work".to_string()],
            None,
            None,
        )
        .await
        .expect("Failed to create clip");

    client
        .create_clip(
            "Personal notes about meetings".to_string(),
            vec!["personal".to_string()],
            None,
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

    assert!(!clips.items.is_empty());
    assert!(clips
        .items
        .iter()
        .any(|c| c.content == "Work document about meetings"));
}

#[tokio::test]
async fn test_websocket_notifications() {
    let (_server, client) = test_client().await;

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
    let (_server, client) = test_client().await;

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
        .create_clip("Update test".to_string(), vec!["test".to_string()], None, None)
        .await
        .expect("Failed to create clip");

    // Consume the creation notification
    let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for creation notification");

    // Update the clip
    client
        .update_clip(&created.id, Some(vec!["updated".to_string()]), None, None)
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
    let (_server, client) = test_client().await;

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
        .create_clip("Delete test".to_string(), vec!["test".to_string()], None, None)
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
    let (_server, client) = test_client().await;

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
    let (_server, client) = test_client().await;

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
    let (_server, client) = test_client().await;

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
    let (_server, client) = test_client().await;

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

// ==================== Language Persistence Tests ====================

#[tokio::test]
async fn test_create_clip_with_language() {
    let (_server, client) = test_client().await;

    let clip = client
        .create_clip(
            "fn main() { println!(\"Hello\"); }".to_string(),
            vec!["code".to_string()],
            None,
            Some("rust".to_string()),
        )
        .await
        .expect("Failed to create clip");

    assert_eq!(clip.language, Some("rust".to_string()));

    // Verify language is persisted by retrieving the clip
    let retrieved = client
        .get_clip(&clip.id)
        .await
        .expect("Failed to get clip");

    assert_eq!(retrieved.language, Some("rust".to_string()));
}

#[tokio::test]
async fn test_create_clip_without_language() {
    let (_server, client) = test_client().await;

    let clip = client
        .create_clip(
            "Some plain text".to_string(),
            vec!["text".to_string()],
            None,
            None,
        )
        .await
        .expect("Failed to create clip");

    assert_eq!(clip.language, None);

    // Verify language is None when retrieved
    let retrieved = client
        .get_clip(&clip.id)
        .await
        .expect("Failed to get clip");

    assert_eq!(retrieved.language, None);
}

#[tokio::test]
async fn test_update_clip_add_language() {
    let (_server, client) = test_client().await;

    // Create a clip without a language
    let created = client
        .create_clip(
            "console.log('hello')".to_string(),
            vec!["code".to_string()],
            None,
            None,
        )
        .await
        .expect("Failed to create clip");

    assert_eq!(created.language, None);

    // Update to add a language
    let updated = client
        .update_clip(&created.id, None, None, Some("javascript".to_string()))
        .await
        .expect("Failed to update clip");

    assert_eq!(updated.language, Some("javascript".to_string()));

    // Verify language is persisted
    let retrieved = client
        .get_clip(&created.id)
        .await
        .expect("Failed to get clip");

    assert_eq!(retrieved.language, Some("javascript".to_string()));
}

#[tokio::test]
async fn test_update_clip_change_language() {
    let (_server, client) = test_client().await;

    // Create a clip with a language
    let created = client
        .create_clip(
            "print('hello')".to_string(),
            vec!["code".to_string()],
            None,
            Some("python".to_string()),
        )
        .await
        .expect("Failed to create clip");

    assert_eq!(created.language, Some("python".to_string()));

    // Update to change the language
    let updated = client
        .update_clip(&created.id, None, None, Some("ruby".to_string()))
        .await
        .expect("Failed to update clip");

    assert_eq!(updated.language, Some("ruby".to_string()));
}

#[tokio::test]
async fn test_update_clip_language_preserves_other_fields() {
    let (_server, client) = test_client().await;

    // Create a clip with all fields
    let created = client
        .create_clip(
            "Some code content".to_string(),
            vec!["tag1".to_string(), "tag2".to_string()],
            Some("Important notes".to_string()),
            Some("typescript".to_string()),
        )
        .await
        .expect("Failed to create clip");

    // Update only the language
    let updated = client
        .update_clip(&created.id, None, None, Some("javascript".to_string()))
        .await
        .expect("Failed to update clip");

    // Verify language changed
    assert_eq!(updated.language, Some("javascript".to_string()));
    // Verify other fields are preserved
    assert_eq!(updated.tags, vec!["tag1", "tag2"]);
    assert_eq!(updated.additional_notes, Some("Important notes".to_string()));
    assert_eq!(updated.content, "Some code content");
}

#[tokio::test]
async fn test_update_clip_tags_preserves_language() {
    let (_server, client) = test_client().await;

    // Create a clip with a language
    let created = client
        .create_clip(
            "package main".to_string(),
            vec!["original".to_string()],
            None,
            Some("go".to_string()),
        )
        .await
        .expect("Failed to create clip");

    // Update only the tags (pass None for language)
    let updated = client
        .update_clip(&created.id, Some(vec!["updated".to_string()]), None, None)
        .await
        .expect("Failed to update clip");

    // Verify tags changed
    assert_eq!(updated.tags, vec!["updated"]);
    // Verify language is preserved
    assert_eq!(updated.language, Some("go".to_string()));
}
