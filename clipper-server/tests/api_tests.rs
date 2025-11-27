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
