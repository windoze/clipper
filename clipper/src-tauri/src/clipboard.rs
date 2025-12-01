use crate::state::AppState;
use arboard::Clipboard;
use chrono::Utc;
use gethostname::gethostname;
use image::{ImageBuffer, Rgba};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Get the hostname tag in the format `$host:<hostname>`
fn get_hostname_tag() -> String {
    let hostname = gethostname().to_string_lossy().to_string();
    format!("$host:{}", hostname)
}

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

/// Result of attempting to get clipboard content
enum ClipboardResult {
    /// Successfully retrieved content
    Content(ClipboardContent),
    /// Clipboard access failed - handle may need recreation
    AccessError(String),
}

/// Get current clipboard content (text or image)
/// Returns ClipboardResult to indicate whether the clipboard handle is still valid
fn get_clipboard_content(clipboard: &mut Clipboard) -> ClipboardResult {
    // Try to get image first (images take priority)
    match clipboard.get_image() {
        Ok(image_data) => {
            if let Some(png_bytes) = image_data_to_png(&image_data) {
                return ClipboardResult::Content(ClipboardContent::Image(png_bytes));
            }
        }
        Err(arboard::Error::ContentNotAvailable) => {
            // No image content, this is normal - try text
        }
        Err(e) => {
            // Other errors might indicate clipboard handle issues
            eprintln!("[clipboard] Image access error: {}", e);
        }
    }

    // Fall back to text
    match clipboard.get_text() {
        Ok(text) => {
            if !text.is_empty() {
                return ClipboardResult::Content(ClipboardContent::Text(text));
            }
            ClipboardResult::Content(ClipboardContent::Empty)
        }
        Err(arboard::Error::ContentNotAvailable) => {
            // No text content either
            ClipboardResult::Content(ClipboardContent::Empty)
        }
        Err(e) => {
            // Access error - clipboard handle may be stale
            ClipboardResult::AccessError(format!("Clipboard text access error: {}", e))
        }
    }
}

/// Try to create a new clipboard handle, with retry logic
fn create_clipboard() -> Option<Clipboard> {
    match Clipboard::new() {
        Ok(cb) => Some(cb),
        Err(e) => {
            eprintln!("[clipboard] Failed to create clipboard handle: {}", e);
            None
        }
    }
}

pub fn start_clipboard_monitor(app: AppHandle) {
    let state = app.state::<AppState>();
    let client = state.client().clone();
    let last_synced = Arc::clone(&state.last_synced_content);
    let last_content = Arc::new(std::sync::Mutex::new(ClipboardContent::Empty));

    // Spawn clipboard monitoring task
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[clipboard] Failed to create tokio runtime: {}", e);
                return;
            }
        };

        let mut clipboard: Option<Clipboard> = create_clipboard();
        let mut consecutive_errors: u32 = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;
        const ERROR_BACKOFF_MS: u64 = 1000;

        // Initialize with current clipboard content if we have a handle
        if let Some(ref mut cb) = clipboard
            && let ClipboardResult::Content(content) = get_clipboard_content(cb)
        {
            // Handle potential mutex poisoning gracefully
            if let Ok(mut guard) = last_content.lock() {
                *guard = content;
            }
        }

        loop {
            // Use longer sleep if we're experiencing errors
            let sleep_duration = if consecutive_errors > 0 {
                Duration::from_millis(ERROR_BACKOFF_MS * consecutive_errors as u64)
            } else {
                Duration::from_millis(POLL_INTERVAL_MS)
            };
            std::thread::sleep(sleep_duration);

            // Ensure we have a valid clipboard handle
            if clipboard.is_none() {
                eprintln!("[clipboard] Attempting to recreate clipboard handle...");
                clipboard = create_clipboard();
                if clipboard.is_some() {
                    eprintln!("[clipboard] Successfully recreated clipboard handle after error");
                    consecutive_errors = 0;
                } else {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        eprintln!(
                            "[clipboard] Failed to recreate clipboard handle after {} attempts, backing off",
                            consecutive_errors
                        );
                    }
                    continue;
                }
            }

            let cb = clipboard.as_mut().unwrap();
            let current_content = match get_clipboard_content(cb) {
                ClipboardResult::Content(content) => {
                    consecutive_errors = 0;
                    content
                }
                ClipboardResult::AccessError(err) => {
                    consecutive_errors = consecutive_errors.saturating_add(1);
                    eprintln!(
                        "[clipboard] Access error (attempt {}): {}",
                        consecutive_errors, err
                    );

                    // Invalidate the clipboard handle so it gets recreated
                    clipboard = None;

                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        eprintln!(
                            "[clipboard] Access failing repeatedly ({} times), will keep retrying with backoff",
                            consecutive_errors
                        );
                    }
                    continue;
                }
            };

            // Skip if clipboard is empty
            if current_content == ClipboardContent::Empty {
                continue;
            }

            // Handle potential mutex poisoning gracefully
            let last = match last_content.lock() {
                Ok(guard) => guard.clone(),
                Err(poisoned) => {
                    eprintln!("[clipboard] last_content mutex was poisoned, recovering");
                    poisoned.into_inner().clone()
                }
            };

            // Skip if content hasn't changed
            if current_content == last {
                continue;
            }

            // For text content, check if it was just synced from server (avoid loop)
            if let ClipboardContent::Text(ref text) = current_content {
                let synced = match last_synced.lock() {
                    Ok(guard) => guard.clone(),
                    Err(poisoned) => {
                        eprintln!("[clipboard] last_synced mutex was poisoned, recovering");
                        poisoned.into_inner().clone()
                    }
                };
                if *text == synced {
                    match last_content.lock() {
                        Ok(mut guard) => *guard = current_content,
                        Err(poisoned) => *poisoned.into_inner() = current_content,
                    }
                    continue;
                }
            }

            // Content changed, update last content
            match last_content.lock() {
                Ok(mut guard) => *guard = current_content.clone(),
                Err(poisoned) => *poisoned.into_inner() = current_content.clone(),
            }

            let client_clone = client.clone();
            let app_handle = app.clone();

            match current_content {
                ClipboardContent::Text(text) => {
                    let hostname_tag = get_hostname_tag();
                    rt.spawn(async move {
                        match client_clone
                            .create_clip(text, vec![hostname_tag], None)
                            .await
                        {
                            Ok(clip) => {
                                let _ = app_handle.emit("clip-created", &clip);
                            }
                            Err(e) => {
                                eprintln!("[clipboard] Failed to create clip from text: {}", e);
                            }
                        }
                    });
                }
                ClipboardContent::Image(png_bytes) => {
                    let filename =
                        format!("screenshot-{}.png", Utc::now().format("%Y-%m-%d-%H-%M-%S"));
                    let hostname_tag = get_hostname_tag();
                    rt.spawn(async move {
                        match client_clone
                            .upload_file_bytes(
                                png_bytes,
                                filename,
                                vec!["$image".to_string(), hostname_tag],
                                None,
                            )
                            .await
                        {
                            Ok(clip) => {
                                let _ = app_handle.emit("clip-created", &clip);
                            }
                            Err(e) => {
                                eprintln!("[clipboard] Failed to create clip from image: {}", e);
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

/// Set image content to the system clipboard from PNG bytes
pub fn set_clipboard_image(png_bytes: &[u8]) -> Result<(), String> {
    use std::io::Cursor;

    // Decode PNG bytes to get image dimensions and RGBA data
    let img = image::ImageReader::new(Cursor::new(png_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Create arboard ImageData
    let image_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: rgba.into_raw().into(),
    };

    // Set to clipboard
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_image(image_data)
        .map_err(|e| format!("Failed to set clipboard image: {}", e))
}
