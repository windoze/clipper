use crate::settings::SettingsManager;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};

const APP_NAME: &str = "clipper-slint";

/// Manages the bundled clipper-server child process
pub struct ServerManager {
    /// The server process child handle
    child: Mutex<Option<Child>>,
    /// The port the server is running on
    port: RwLock<Option<u16>>,
    /// The base URL of the server
    server_url: RwLock<Option<String>>,
    /// Path to the database
    db_path: PathBuf,
    /// Path to file storage
    storage_path: PathBuf,
    /// Reference to settings manager
    settings: Arc<SettingsManager>,
    /// Flag to signal shutdown (stops restart attempts)
    shutdown: AtomicBool,
    /// Path to the server binary
    server_binary: PathBuf,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new(settings: Arc<SettingsManager>) -> Result<Self, String> {
        let data_dir = get_data_dir()?;

        let db_path = data_dir.join("db");
        let storage_path = data_dir.join("storage");

        // Find the server binary
        let server_binary = find_server_binary()?;

        Ok(Self {
            child: Mutex::new(None),
            port: RwLock::new(None),
            server_url: RwLock::new(None),
            db_path,
            storage_path,
            settings,
            shutdown: AtomicBool::new(false),
            server_binary,
        })
    }

    /// Check if a port is available
    fn is_port_available(port: u16) -> bool {
        std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Get the server URL if the server is running
    pub async fn server_url(&self) -> Option<String> {
        self.server_url.read().await.clone()
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        self.child.lock().await.is_some()
    }

    /// Internal method to spawn the server process
    async fn spawn_server(&self) -> Result<String, String> {
        // Try to reuse saved port, or pick a new one
        let port = if let Some(saved_port) = self.settings.get_server_port() {
            if Self::is_port_available(saved_port) {
                eprintln!("[clipper-server] Reusing saved port: {}", saved_port);
                saved_port
            } else {
                eprintln!(
                    "[clipper-server] Saved port {} is in use, picking new port",
                    saved_port
                );
                portpicker::pick_unused_port().ok_or("Failed to find available port")?
            }
        } else {
            portpicker::pick_unused_port().ok_or("Failed to find available port")?
        };

        // Ensure data directories exist
        tokio::fs::create_dir_all(&self.db_path)
            .await
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
        tokio::fs::create_dir_all(&self.storage_path)
            .await
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;

        let db_path_str = self
            .db_path
            .to_str()
            .ok_or("Invalid database path")?
            .to_string();
        let storage_path_str = self
            .storage_path
            .to_str()
            .ok_or("Invalid storage path")?
            .to_string();

        // Determine listen address based on settings
        let listen_on_all = self.settings.get_listen_on_all_interfaces();
        let listen_addr = if listen_on_all {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        };
        eprintln!(
            "[clipper-server] Binding to {} (listen_on_all_interfaces: {})",
            listen_addr, listen_on_all
        );

        // Spawn the server process
        let mut child = Command::new(&self.server_binary)
            .args([
                "--db-path",
                &db_path_str,
                "--storage-path",
                &storage_path_str,
                "--listen-addr",
                listen_addr,
                "--port",
                &port.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // Ensures child is killed when handle is dropped
            .spawn()
            .map_err(|e| format!("Failed to spawn server: {}", e))?;

        // Get stdout and stderr handles
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Store the child process
        *self.child.lock().await = Some(child);

        // Store the port and URL
        let server_url = format!("http://127.0.0.1:{}", port);
        *self.port.write().await = Some(port);
        *self.server_url.write().await = Some(server_url.clone());

        // Save the port to settings for next startup
        if let Err(e) = self.settings.set_server_port(port) {
            eprintln!("[clipper-server] Warning: Failed to save port: {}", e);
        }

        // Spawn task to monitor stdout
        if let Some(stdout) = stdout {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("[clipper-server] {}", line);
                }
            });
        }

        // Spawn task to monitor stderr
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("[clipper-server] {}", line);
                }
            });
        }

        Ok(server_url)
    }

    /// Wait for the server to become healthy
    async fn wait_for_health(&self, server_url: &str) -> bool {
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", server_url);
        let mut retries = 10;

        // Initial wait
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        while retries > 0 {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    eprintln!("[clipper-server] Server is healthy at {}", server_url);
                    return true;
                }
                _ => {
                    retries -= 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }
            }
        }

        eprintln!(
            "[clipper-server] Server started at {} (health check pending)",
            server_url
        );
        false
    }

    /// Start the bundled server and begin monitoring for restarts
    pub async fn start(self: &Arc<Self>) -> Result<String, String> {
        // Check if already running
        if self.is_running().await
            && let Some(url) = self.server_url().await
        {
            return Ok(url);
        }

        // Clear shutdown flag
        self.shutdown.store(false, Ordering::SeqCst);

        // Spawn the server
        let server_url = self.spawn_server().await?;

        // Wait for health
        self.wait_for_health(&server_url).await;

        // Spawn background task to monitor and restart
        let manager = Arc::clone(self);
        tokio::spawn(async move {
            manager.monitor_loop().await;
        });

        Ok(server_url)
    }

    /// Background loop that monitors the server and restarts if needed
    async fn monitor_loop(self: Arc<Self>) {
        loop {
            // Check if shutdown was requested
            if self.shutdown.load(Ordering::SeqCst) {
                eprintln!("[clipper-server] Shutdown requested, stopping monitor");
                break;
            }

            // Check the process status
            let needs_restart = {
                let mut child_guard = self.child.lock().await;
                if let Some(ref mut child) = *child_guard {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Process has exited
                            eprintln!("[clipper-server] Server exited with status: {:?}", status);
                            *child_guard = None;

                            // Clear state
                            *self.port.write().await = None;
                            *self.server_url.write().await = None;

                            true
                        }
                        Ok(None) => {
                            // Process still running
                            false
                        }
                        Err(e) => {
                            eprintln!("[clipper-server] Error checking process status: {}", e);
                            false
                        }
                    }
                } else {
                    // No child process, exit the monitor loop
                    break;
                }
            };

            if needs_restart {
                // Check if shutdown was requested
                if self.shutdown.load(Ordering::SeqCst) {
                    eprintln!("[clipper-server] Shutdown requested, not restarting");
                    break;
                }

                // Wait before restart
                eprintln!("[clipper-server] Restarting server in 1 second...");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // Check shutdown flag again after sleep
                if self.shutdown.load(Ordering::SeqCst) {
                    eprintln!("[clipper-server] Shutdown requested, not restarting");
                    break;
                }

                // Attempt to restart
                match self.spawn_server().await {
                    Ok(url) => {
                        self.wait_for_health(&url).await;
                        eprintln!("[clipper-server] Server restarted at {}", url);
                    }
                    Err(e) => {
                        eprintln!("[clipper-server] Failed to restart server: {}", e);
                        // Wait before retrying
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            } else {
                // Process still running, check again in a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }

    /// Stop the server gracefully
    pub async fn stop(&self) -> Result<(), String> {
        // Set shutdown flag to prevent restart
        self.shutdown.store(true, Ordering::SeqCst);

        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            // Kill the process
            if let Err(e) = child.kill().await {
                eprintln!("[clipper-server] Warning: Failed to kill server: {}", e);
            }
            eprintln!("[clipper-server] Server stopped");

            // Wait for the process to fully terminate
            let _ = child.wait().await;

            // Wait for port to be released
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        *self.port.write().await = None;
        *self.server_url.write().await = None;

        Ok(())
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        // Set shutdown flag
        self.shutdown.store(true, Ordering::SeqCst);

        // Try to stop the server synchronously on drop
        // Note: kill_on_drop(true) on the Command should handle this,
        // but we do a best-effort cleanup here as well
        if let Ok(mut child_guard) = self.child.try_lock()
            && let Some(child) = child_guard.take()
        {
            // The child will be killed automatically due to kill_on_drop(true)
            drop(child);
            eprintln!("[clipper-server] Server process cleanup on drop");
        }
    }
}

/// Get the application data directory for storing server data
pub fn get_data_dir() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join(APP_NAME);
    Ok(data_dir)
}

/// Find the clipper-server binary
fn find_server_binary() -> Result<PathBuf, String> {
    // First, check if it's in the same directory as the current executable
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let server_path = exe_dir.join("clipper-server");
        if server_path.exists() {
            eprintln!("[clipper-server] Found server binary at: {:?}", server_path);
            return Ok(server_path);
        }

        // Also check for .exe on Windows
        #[cfg(target_os = "windows")]
        {
            let server_path = exe_dir.join("clipper-server.exe");
            if server_path.exists() {
                eprintln!("[clipper-server] Found server binary at: {:?}", server_path);
                return Ok(server_path);
            }
        }
    }

    // Next, check if it's in PATH
    if let Ok(path) = which::which("clipper-server") {
        eprintln!("[clipper-server] Found server binary in PATH: {:?}", path);
        return Ok(path);
    }

    // Finally, check in the cargo target directory (for development)
    let cargo_target_paths = [
        // Debug build
        PathBuf::from("../target/debug/clipper-server"),
        // Release build
        PathBuf::from("../target/release/clipper-server"),
    ];

    for path in &cargo_target_paths {
        if path.exists() {
            let canonical = path
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize path: {}", e))?;
            eprintln!("[clipper-server] Found server binary at: {:?}", canonical);
            return Ok(canonical);
        }
    }

    Err("Could not find clipper-server binary. Please ensure it is built and either in the same directory as this executable, in PATH, or in the cargo target directory.".to_string())
}
