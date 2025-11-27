use crate::state::AppState;
use arboard::Clipboard;
use chrono::Utc;
use image::{ImageBuffer, Rgba};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const POLL_INTERVAL_MS: u64 = 500;

/// Represents the type of clipboard content
#[derive(Clone, PartialEq)]
enum ClipboardContent {
    Text(String),
    Image(Vec<u8>), // PNG-encoded bytes
    Empty,
}

/// Convert arboard ImageData to PNG bytes
fn image_data_to_png(image_data: &arboard::ImageData) -> Option<Vec<u8>> {
    let width = image_data.width as u32;
    let height = image_data.height as u32;

    // Create an image buffer from RGBA bytes
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, image_data.bytes.to_vec())?;

    // Encode to PNG
    let mut png_bytes = Cursor::new(Vec::new());
    img.write_to(&mut png_bytes, image::ImageFormat::Png).ok()?;

    Some(png_bytes.into_inner())
}

/// Get current clipboard content (text or image)
fn get_clipboard_content(clipboard: &mut Clipboard) -> ClipboardContent {
    // Try to get image first (images take priority)
    if let Ok(image_data) = clipboard.get_image() {
        if let Some(png_bytes) = image_data_to_png(&image_data) {
            return ClipboardContent::Image(png_bytes);
        }
    }

    // Fall back to text
    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            return ClipboardContent::Text(text);
        }
    }

    ClipboardContent::Empty
}

pub fn start_clipboard_monitor(app: AppHandle) {
    let state = app.state::<AppState>();
    let client = state.client().clone();
    let last_synced = Arc::clone(&state.last_synced_content);
    let last_content = Arc::new(std::sync::Mutex::new(ClipboardContent::Empty));

    // Spawn clipboard monitoring task
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                eprintln!("Failed to initialize clipboard: {}", e);
                return;
            }
        };

        // Initialize with current clipboard content
        let initial_content = get_clipboard_content(&mut clipboard);
        *last_content.lock().unwrap() = initial_content;

        loop {
            std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));

            let current_content = get_clipboard_content(&mut clipboard);

            // Skip if clipboard is empty
            if current_content == ClipboardContent::Empty {
                continue;
            }

            let last = last_content.lock().unwrap().clone();

            // Skip if content hasn't changed
            if current_content == last {
                continue;
            }

            // For text content, check if it was just synced from server (avoid loop)
            if let ClipboardContent::Text(ref text) = current_content {
                let synced = last_synced.lock().unwrap().clone();
                if *text == synced {
                    *last_content.lock().unwrap() = current_content;
                    continue;
                }
            }

            // Content changed, update last content
            *last_content.lock().unwrap() = current_content.clone();

            let client_clone = client.clone();
            let app_handle = app.clone();

            match current_content {
                ClipboardContent::Text(text) => {
                    rt.spawn(async move {
                        match client_clone.create_clip(text, vec![], None).await {
                            Ok(clip) => {
                                let _ = app_handle.emit("clip-created", &clip);
                            }
                            Err(e) => {
                                eprintln!("Failed to create clip from clipboard text: {}", e);
                            }
                        }
                    });
                }
                ClipboardContent::Image(png_bytes) => {
                    let filename =
                        format!("screenshot-{}.png", Utc::now().format("%Y-%m-%d-%H-%M-%S"));
                    rt.spawn(async move {
                        match client_clone
                            .upload_file_bytes(
                                png_bytes,
                                filename,
                                vec!["$image".to_string()],
                                None,
                            )
                            .await
                        {
                            Ok(clip) => {
                                let _ = app_handle.emit("clip-created", &clip);
                            }
                            Err(e) => {
                                eprintln!("Failed to create clip from clipboard image: {}", e);
                            }
                        }
                    });
                }
                ClipboardContent::Empty => {}
            }
        }
    });
}

pub fn set_clipboard_content(content: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(content).map_err(|e| e.to_string())
}
