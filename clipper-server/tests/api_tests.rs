use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use clipper_indexer::ClipperIndexer;
use clipper_server::{api, AppState};
use http_body_util::BodyExt;
use serde_json::json;
use tempfile::TempDir;
use tower::ServiceExt;

/// Helper function to create a test app with a temporary database
async fn create_test_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("db");
    let storage_path = temp_dir.path().join("storage");

    let indexer = ClipperIndexer::new(&db_path, &storage_path)
        .await
        .expect("Failed to create indexer");

    let state = AppState::new(indexer);
    let app = Router::new().merge(api::routes()).with_state(state);

    (app, temp_dir)
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_create_clip() {
    let (app, _temp_dir) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Test content",
                        "tags": ["test", "example"],
                        "additional_notes": "Test notes"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert_eq!(body["content"], "Test content");
    assert_eq!(body["tags"], json!(["test", "example"]));
    assert_eq!(body["additional_notes"], "Test notes");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn test_create_clip_without_notes() {
    let (app, _temp_dir) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Simple content",
                        "tags": ["simple"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert_eq!(body["content"], "Simple content");
    assert_eq!(body["tags"], json!(["simple"]));
    assert!(body["additional_notes"].is_null());
}

#[tokio::test]
async fn test_get_clip() {
    let (app, _temp_dir) = create_test_app().await;

    // Create a clip first
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Get me",
                        "tags": ["findme"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_body = response_json(create_response).await;
    let clip_id = create_body["id"].as_str().unwrap();

    // Get the clip
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/clips/{}", clip_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["id"], clip_id);
    assert_eq!(body["content"], "Get me");
    assert_eq!(body["tags"], json!(["findme"]));
}

#[tokio::test]
async fn test_get_nonexistent_clip() {
    let (app, _temp_dir) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/nonexistent123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_clip() {
    let (app, _temp_dir) = create_test_app().await;

    // Create a clip
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Original content",
                        "tags": ["original"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_body = response_json(create_response).await;
    let clip_id = create_body["id"].as_str().unwrap();

    // Update the clip
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/clips/{}", clip_id))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "tags": ["updated", "new"],
                        "additional_notes": "Updated notes"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["id"], clip_id);
    assert_eq!(body["tags"], json!(["updated", "new"]));
    assert_eq!(body["additional_notes"], "Updated notes");
    assert_eq!(body["content"], "Original content");
}

#[tokio::test]
async fn test_delete_clip() {
    let (app, _temp_dir) = create_test_app().await;

    // Create a clip
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Delete me",
                        "tags": ["temporary"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_body = response_json(create_response).await;
    let clip_id = create_body["id"].as_str().unwrap().to_string();

    // Delete the clip
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/clips/{}", clip_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify it's deleted
    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/clips/{}", clip_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_clips() {
    let (app, _temp_dir) = create_test_app().await;

    // Create multiple clips
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Clip 1",
                        "tags": ["test"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Clip 2",
                        "tags": ["test"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // List all clips
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert!(body.is_object());
    let items = body["items"].as_array().unwrap();
    assert!(items.len() >= 2);
    assert!(body["total"].as_u64().unwrap() >= 2);
    assert_eq!(body["page"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn test_search_clips() {
    let (app, _temp_dir) = create_test_app().await;

    // Create clips with searchable content
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "The quick brown fox",
                        "tags": ["animals"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "The lazy dog",
                        "tags": ["animals"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Search for clips
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=fox")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["content"], "The quick brown fox");
}

// ============================================================================
// Search Combination Tests
// ============================================================================

/// Helper to create multiple clips with different tags for testing search combinations
async fn create_test_clips_for_search(app: &Router) {
    // Clip 1: rust, programming
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Rust programming language",
                        "tags": ["rust", "programming"],
                        "additional_notes": "A systems programming language"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Clip 2: python, programming
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Python scripting language",
                        "tags": ["python", "programming"],
                        "additional_notes": "A dynamic programming language"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Clip 3: rust, webdev
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Rust web development with Axum",
                        "tags": ["rust", "webdev"],
                        "additional_notes": "Building web apps in Rust"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Clip 4: no tags
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "Untagged content about programming",
                        "tags": []
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Clip 5: favorite tag only
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "content": "My favorite Rust snippet",
                        "tags": ["favorite"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_search_no_filters() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with query only, no tags filter
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=programming")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find all clips containing "programming" (clips 1, 2, 4)
    assert!(items.len() >= 3, "Expected at least 3 clips, got {}", items.len());
}

#[tokio::test]
async fn test_search_with_empty_tags_parameter() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with empty tags parameter - should behave same as no tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=programming&tags=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find all clips containing "programming" (same as no tags filter)
    assert!(items.len() >= 3, "Expected at least 3 clips, got {}", items.len());
}

#[tokio::test]
async fn test_search_with_single_tag() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with single tag filter
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=Rust&tags=rust")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find clips 1 and 3 (both have "rust" tag and contain "Rust")
    assert_eq!(items.len(), 2, "Expected 2 clips with rust tag, got {}", items.len());
    for item in items {
        let tags = item["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "rust"), "Expected rust tag in {:?}", tags);
    }
}

#[tokio::test]
async fn test_search_with_multiple_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with multiple tags (OR logic) - clips must have ANY of the tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=Rust&tags=rust,programming")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // With OR logic: clips 1 (rust, programming) and 3 (rust, webdev) match "Rust" and have at least one of the tags
    assert_eq!(items.len(), 2, "Expected 2 clips with rust OR programming tags, got {}", items.len());
    for item in items {
        let tags = item["tags"].as_array().unwrap();
        assert!(
            tags.iter().any(|t| t == "rust") || tags.iter().any(|t| t == "programming"),
            "Expected rust or programming tag in {:?}",
            tags
        );
    }
}

#[tokio::test]
async fn test_search_with_nonexistent_tag() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with a tag that doesn't exist
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=programming&tags=nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find no clips
    assert_eq!(items.len(), 0, "Expected 0 clips with nonexistent tag, got {}", items.len());
}

#[tokio::test]
async fn test_list_no_filters() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with no filters
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should return all 5 clips
    assert_eq!(items.len(), 5, "Expected 5 clips, got {}", items.len());
}

#[tokio::test]
async fn test_list_with_empty_tags_parameter() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with empty tags parameter - should behave same as no tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should return all 5 clips (empty tags = no filter)
    assert_eq!(items.len(), 5, "Expected 5 clips with empty tags filter, got {}", items.len());
}

#[tokio::test]
async fn test_list_with_single_tag() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with single tag filter
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=programming")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find clips 1 and 2 (both have "programming" tag)
    assert_eq!(items.len(), 2, "Expected 2 clips with programming tag, got {}", items.len());
    for item in items {
        let tags = item["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "programming"), "Expected programming tag in {:?}", tags);
    }
}

#[tokio::test]
async fn test_list_with_multiple_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with multiple tags (OR logic) - clips must have ANY of the tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=rust,webdev")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // With OR logic: clips 1 (rust, programming), 3 (rust, webdev) have at least one of the tags
    assert_eq!(items.len(), 2, "Expected 2 clips with rust OR webdev tags, got {}", items.len());
    for item in items {
        let tags = item["tags"].as_array().unwrap();
        assert!(
            tags.iter().any(|t| t == "rust") || tags.iter().any(|t| t == "webdev"),
            "Expected rust or webdev tag in {:?}",
            tags
        );
    }
}

#[tokio::test]
async fn test_list_with_nonexistent_tag() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with a tag that doesn't exist
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find no clips
    assert_eq!(items.len(), 0, "Expected 0 clips with nonexistent tag, got {}", items.len());
}

#[tokio::test]
async fn test_search_empty_query_with_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with empty query but with tags filter
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=&tags=rust")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // With empty query, should still filter by tag
    // Clips 1, 3, and 5 have "rust" tag
    for item in items {
        let tags = item["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "rust"), "Expected rust tag in {:?}", tags);
    }
}

#[tokio::test]
async fn test_search_with_whitespace_in_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with whitespace around tags (should be trimmed)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=Rust&tags=%20rust%20,%20programming%20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // With OR logic and trimmed tags: clips 1 (rust, programming) and 3 (rust, webdev) match
    assert_eq!(items.len(), 2, "Expected 2 clips with whitespace-trimmed tags, got {}", items.len());
}

#[tokio::test]
async fn test_list_with_whitespace_only_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with whitespace-only tags (should behave like empty/no tags after trimming)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=%20%20%20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // After trimming whitespace and filtering empty strings, should return all 5 clips
    assert_eq!(items.len(), 5, "Expected 5 clips with whitespace-only tags filter, got {}", items.len());
}

#[tokio::test]
async fn test_list_with_comma_only_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with only commas (should behave like no tags filter after filtering empty strings)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=,,,")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // After splitting by comma and filtering empty strings, should return all 5 clips
    assert_eq!(items.len(), 5, "Expected 5 clips with comma-only tags filter, got {}", items.len());
}

#[tokio::test]
async fn test_search_with_comma_only_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with only commas (should behave like no tags filter)
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=programming&tags=,,,")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Should find all clips containing "programming" (same as no tags filter)
    assert!(items.len() >= 3, "Expected at least 3 clips with comma-only tags filter, got {}", items.len());
}

#[tokio::test]
async fn test_search_empty_query_and_empty_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with empty query and empty tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=&tags=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    // Empty query with no tags filter - depends on search implementation
    // Should return results (possibly all) since no filtering is applied
    assert!(body["total"].as_u64().is_some());
}

#[tokio::test]
async fn test_list_with_mixed_valid_and_empty_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with mix of valid tags and empty strings
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=rust,,programming,")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    // Empty strings should be filtered out, leaving [rust, programming]
    // With OR logic: clips 1 (rust, programming), 2 (python, programming), 3 (rust, webdev) match
    assert_eq!(items.len(), 3, "Expected 3 clips with mixed tags filter, got {}", items.len());
}

#[tokio::test]
async fn test_search_pagination_with_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // Search with pagination and tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips/search?q=Rust&tags=rust&page=1&page_size=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "Expected 1 clip per page");
    assert_eq!(body["page"].as_u64().unwrap(), 1);
    assert_eq!(body["page_size"].as_u64().unwrap(), 1);
    // Total should be 2 (clips 1 and 3 have rust tag)
    assert_eq!(body["total"].as_u64().unwrap(), 2, "Expected total of 2 clips with rust tag");
    assert_eq!(body["total_pages"].as_u64().unwrap(), 2);
}

#[tokio::test]
async fn test_list_pagination_with_tags() {
    let (app, _temp_dir) = create_test_app().await;
    create_test_clips_for_search(&app).await;

    // List with pagination and tags
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/clips?tags=programming&page=1&page_size=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "Expected 1 clip per page");
    assert_eq!(body["page"].as_u64().unwrap(), 1);
    // Total should be 2 (clips 1 and 2 have programming tag)
    assert_eq!(body["total"].as_u64().unwrap(), 2, "Expected total of 2 clips with programming tag");
}

#[tokio::test]
async fn test_upload_file() {
    let (app, _temp_dir) = create_test_app().await;

    let file_content = b"This is test file content";
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    let body_str = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"tags\"\r\n\
         \r\n\
         document,test\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"additional_notes\"\r\n\
         \r\n\
         Test upload\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content)
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/clips/upload")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body_str))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert_eq!(body["content"], "This is test file content");
    assert_eq!(body["tags"], json!(["document", "test"]));
    assert_eq!(body["additional_notes"], "Test upload");
    assert!(body["file_attachment"].is_string());
    assert_eq!(body["original_filename"], "test.txt");
}
