//! Parent process monitoring for bundled server mode.
//!
//! When the server is spawned as a child of the Clipper desktop app, the parent
//! passes a pipe handle via `--parent-pipe-handle`. This module monitors that pipe
//! and triggers a graceful shutdown when the parent exits (pipe is closed).

use std::io::Read;
use std::sync::OnceLock;
use std::thread;
use tokio::sync::broadcast;

#[cfg(unix)]
use std::os::unix::io::FromRawFd;

#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, RawHandle};

/// Global shutdown signal sender for parent exit notification
static PARENT_SHUTDOWN_TX: OnceLock<broadcast::Sender<()>> = OnceLock::new();

/// Initialize the parent monitor shutdown channel and return a receiver.
///
/// This should be called once at startup if parent monitoring is enabled.
/// Returns a receiver that will receive a message when the parent exits.
pub fn init_shutdown_channel() -> broadcast::Receiver<()> {
    let (tx, rx) = broadcast::channel(1);
    let _ = PARENT_SHUTDOWN_TX.set(tx);
    rx
}

/// Start monitoring the parent process via a pipe handle.
///
/// When the parent process exits (normally or abnormally), the pipe will be closed,
/// and this function will trigger the shutdown signal via the broadcast channel.
///
/// # Arguments
/// * `handle` - The pipe handle value passed from the parent process
///
/// # Note
/// `init_shutdown_channel()` must be called before this function to set up the channel.
pub fn start_parent_monitor(handle: u64) {
    thread::spawn(move || {
        tracing::info!("Starting parent process monitor (handle: {})", handle);

        // Create a PipeReader from the raw handle
        #[cfg(unix)]
        let mut pipe = unsafe { os_pipe::PipeReader::from_raw_fd(handle as i32) };

        #[cfg(windows)]
        let mut pipe = unsafe { os_pipe::PipeReader::from_raw_handle(handle as RawHandle) };

        // Try to read from the pipe - this will block until:
        // 1. Data is written (shouldn't happen, we only use this for monitoring)
        // 2. The pipe is closed (parent exited)
        // 3. An error occurs
        let mut buf = [0u8; 1];
        loop {
            match pipe.read(&mut buf) {
                Ok(0) => {
                    // EOF - pipe was closed (parent exited)
                    tracing::warn!("Parent process pipe closed - parent has exited");
                    break;
                }
                Ok(_) => {
                    // Unexpected data received - ignore and continue monitoring
                    tracing::debug!("Received unexpected data on parent monitor pipe");
                }
                Err(e) => {
                    // Error reading pipe - likely parent crashed
                    tracing::warn!(
                        "Error reading parent monitor pipe: {} - assuming parent exited",
                        e
                    );
                    break;
                }
            }
        }

        // Parent has exited - initiate graceful shutdown
        tracing::info!("Initiating graceful shutdown due to parent process exit");

        // Send shutdown signal via the broadcast channel
        if let Some(tx) = PARENT_SHUTDOWN_TX.get() {
            let _ = tx.send(());
        } else {
            // Fallback: if channel wasn't initialized, exit directly
            tracing::warn!("Shutdown channel not initialized, forcing exit");
            std::process::exit(0);
        }
    });
}
