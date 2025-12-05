use crate::state::AppState;
use arboard::Clipboard;
use chrono::Utc;
use gethostname::gethostname;
use image::{ImageBuffer, Rgba};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
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
    Image(Vec<u8>),      // PNG-encoded bytes
    Files(Vec<PathBuf>), // File paths from clipboard (e.g., copied from Finder/Explorer)
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

/// Get current clipboard content (text, image, or files)
/// Returns ClipboardResult to indicate whether the clipboard handle is still valid
/// Priority: files > images > text (files take highest priority since copying files
/// in Finder/Explorer also provides text fallback with filenames)
fn get_clipboard_content(clipboard: &mut Clipboard) -> ClipboardResult {
    // Try to get file list first (highest priority)
    // When copying files in Finder/Explorer, the clipboard contains both file URIs and text fallback
    match clipboard.get().file_list() {
        Ok(files) => {
            if !files.is_empty() {
                return ClipboardResult::Content(ClipboardContent::Files(files));
            }
        }
        Err(arboard::Error::ContentNotAvailable) => {
            // No file content, this is normal - try image
        }
        Err(e) => {
            // Other errors might indicate clipboard handle issues
            eprintln!("[clipboard] File list access error: {}", e);
        }
    }

    // Try to get image (second priority)
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

    // Fall back to text (lowest priority)
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
    let last_synced = Arc::clone(&state.last_synced_content);
    let last_synced_image = Arc::clone(&state.last_synced_image);
    let last_content = Arc::new(std::sync::Mutex::new(ClipboardContent::Empty));
    // Get a reference to the max upload size (AtomicU64 wrapped in Arc)
    let max_upload_size_arc = state.max_upload_size_arc();

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

            // For image content, check if it was just synced from server (avoid loop)
            if let ClipboardContent::Image(ref png_bytes) = current_content {
                let synced_image = match last_synced_image.lock() {
                    Ok(guard) => guard.clone(),
                    Err(poisoned) => {
                        eprintln!("[clipboard] last_synced_image mutex was poisoned, recovering");
                        poisoned.into_inner().clone()
                    }
                };
                if *png_bytes == synced_image {
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

            // Get a fresh client from the app state each time to pick up URL changes
            let client = app.state::<AppState>().client();
            let app_handle = app.clone();

            match current_content {
                ClipboardContent::Text(text) => {
                    let hostname_tag = get_hostname_tag();
                    rt.spawn(async move {
                        match client
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
                    // Check size limit before uploading
                    let max_size = max_upload_size_arc.load(Ordering::SeqCst);
                    if png_bytes.len() as u64 > max_size {
                        let max_size_mb = max_size as f64 / (1024.0 * 1024.0);
                        let file_size_mb = png_bytes.len() as f64 / (1024.0 * 1024.0);
                        eprintln!(
                            "[clipboard] Image size ({:.2} MB) exceeds maximum allowed size ({:.2} MB), skipping upload",
                            file_size_mb, max_size_mb
                        );
                        continue;
                    }
                    let filename =
                        format!("screenshot-{}.png", Utc::now().format("%Y-%m-%d-%H-%M-%S"));
                    let hostname_tag = get_hostname_tag();
                    rt.spawn(async move {
                        match client
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
                ClipboardContent::Files(paths) => {
                    // Upload files copied from Finder/Explorer
                    let max_size = max_upload_size_arc.load(Ordering::SeqCst);
                    let hostname_tag = get_hostname_tag();
                    rt.spawn(async move {
                        for path in paths {
                            // Check if file exists and get metadata
                            let metadata = match tokio::fs::metadata(&path).await {
                                Ok(m) => m,
                                Err(e) => {
                                    eprintln!(
                                        "[clipboard] Failed to read file metadata for {}: {}",
                                        path.display(),
                                        e
                                    );
                                    continue;
                                }
                            };

                            // Skip directories
                            if metadata.is_dir() {
                                eprintln!(
                                    "[clipboard] Skipping directory: {}",
                                    path.display()
                                );
                                continue;
                            }

                            // Check file size
                            if metadata.len() > max_size {
                                let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                                let max_size_mb = max_size / (1024 * 1024);
                                eprintln!(
                                    "[clipboard] File {} ({:.2} MB) exceeds maximum allowed size ({} MB), skipping",
                                    path.display(),
                                    file_size_mb,
                                    max_size_mb
                                );
                                // Emit error event for toast notification
                                let _ = app_handle.emit(
                                    "file-upload-error",
                                    serde_json::json!({
                                        "path": path.to_string_lossy(),
                                        "error": "file_too_large",
                                        "size_mb": file_size_mb,
                                        "max_size_mb": max_size_mb
                                    }),
                                );
                                continue;
                            }

                            // Read file bytes
                            let bytes = match tokio::fs::read(&path).await {
                                Ok(b) => b,
                                Err(e) => {
                                    eprintln!(
                                        "[clipboard] Failed to read file {}: {}",
                                        path.display(),
                                        e
                                    );
                                    continue;
                                }
                            };

                            let filename = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let full_path = path.to_string_lossy().to_string();

                            match client
                                .upload_file_bytes_with_content(
                                    bytes,
                                    filename.clone(),
                                    vec!["$file".to_string(), hostname_tag.clone()],
                                    None,
                                    Some(full_path.clone()),
                                )
                                .await
                            {
                                Ok(clip) => {
                                    let _ = app_handle.emit("clip-created", &clip);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[clipboard] Failed to upload file {}: {}",
                                        filename, e
                                    );
                                }
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
