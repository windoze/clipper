use crate::settings::SettingsManager;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::{Mutex, RwLock};

/// Manages the bundled clipper-server sidecar process
pub struct ServerManager {
    /// The server process child handle
    child: Mutex<Option<CommandChild>>,
    /// The port the server is running on
    port: RwLock<Option<u16>>,
    /// The base URL of the server
    server_url: RwLock<Option<String>>,
    /// Path to the database
    db_path: PathBuf,
    /// Path to file storage
    storage_path: PathBuf,
}

impl ServerManager {
    /// Create a new server manager with system-preferred data paths
    pub fn new(app_data_dir: PathBuf) -> Self {
        let db_path = app_data_dir.join("db");
        let storage_path = app_data_dir.join("storage");

        Self {
            child: Mutex::new(None),
            port: RwLock::new(None),
            server_url: RwLock::new(None),
            db_path,
            storage_path,
        }
    }

    /// Check if a port is available
    fn is_port_available(port: u16) -> bool {
        std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Get the server URL if the server is running
    pub async fn server_url(&self) -> Option<String> {
        self.server_url.read().await.clone()
    }

    /// Get the port the server is running on
    #[allow(dead_code)]
    pub async fn port(&self) -> Option<u16> {
        *self.port.read().await
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        self.child.lock().await.is_some()
    }

    /// Start the bundled server
    pub async fn start(&self, app: &AppHandle) -> Result<String, String> {
        // Check if already running
        if self.is_running().await {
            if let Some(url) = self.server_url().await {
                return Ok(url);
            }
        }

        // Get settings manager for port persistence
        let settings_manager = app
            .try_state::<SettingsManager>()
            .ok_or("Settings manager not initialized")?;

        // Try to reuse saved port, or pick a new one
        let port = if let Some(saved_port) = settings_manager.get_server_port() {
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
        let listen_on_all = settings_manager.get_listen_on_all_interfaces();
        let listen_addr = if listen_on_all { "0.0.0.0" } else { "127.0.0.1" };
        eprintln!(
            "[clipper-server] Binding to {} (listen_on_all_interfaces: {})",
            listen_addr, listen_on_all
        );

        // Spawn the sidecar process
        let sidecar_command = app
            .shell()
            .sidecar("clipper-server")
            .map_err(|e| format!("Failed to create sidecar command: {}", e))?
            .args([
                "--db-path",
                &db_path_str,
                "--storage-path",
                &storage_path_str,
                "--listen-addr",
                listen_addr,
                "--port",
                &port.to_string(),
            ]);

        let (mut rx, child) = sidecar_command
            .spawn()
            .map_err(|e| format!("Failed to spawn server: {}", e))?;

        // Store the child process
        *self.child.lock().await = Some(child);

        // Store the port and URL
        let server_url = format!("http://127.0.0.1:{}", port);
        *self.port.write().await = Some(port);
        *self.server_url.write().await = Some(server_url.clone());

        // Save the port to settings for next startup
        if let Err(e) = settings_manager.set_server_port(port).await {
            eprintln!("[clipper-server] Warning: Failed to save port: {}", e);
        }

        // Spawn a task to monitor the server output
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line_bytes) => {
                        let line = String::from_utf8_lossy(&line_bytes);
                        eprintln!("[clipper-server] {}", line);
                    }
                    CommandEvent::Stderr(line_bytes) => {
                        let line = String::from_utf8_lossy(&line_bytes);
                        eprintln!("[clipper-server] {}", line);
                    }
                    CommandEvent::Error(err) => {
                        eprintln!("[clipper-server] Error: {}", err);
                    }
                    CommandEvent::Terminated(payload) => {
                        eprintln!(
                            "[clipper-server] Terminated with code: {:?}, signal: {:?}",
                            payload.code, payload.signal
                        );
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Wait a bit for the server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Try to verify the server is up by checking health endpoint
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", server_url);
        let mut retries = 10;
        while retries > 0 {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    eprintln!("[clipper-server] Server is healthy at {}", server_url);
                    return Ok(server_url);
                }
                _ => {
                    retries -= 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }
            }
        }

        // Server might still be starting, return URL anyway
        eprintln!(
            "[clipper-server] Server started at {} (health check pending)",
            server_url
        );
        Ok(server_url)
    }

    /// Stop the server gracefully
    pub async fn stop(&self) -> Result<(), String> {
        let mut child_guard = self.child.lock().await;
        if let Some(child) = child_guard.take() {
            // Kill the process
            child
                .kill()
                .map_err(|e| format!("Failed to kill server: {}", e))?;
            eprintln!("[clipper-server] Server stopped");

            // Wait for the process to fully terminate and port to be released
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        *self.port.write().await = None;
        *self.server_url.write().await = None;

        Ok(())
    }

    /// Clear all data (database and storage) - server must be stopped first
    pub async fn clear_data(&self) -> Result<(), String> {
        // Ensure server is stopped
        if self.is_running().await {
            return Err("Server must be stopped before clearing data".to_string());
        }

        // Remove database directory
        if self.db_path.exists() {
            tokio::fs::remove_dir_all(&self.db_path)
                .await
                .map_err(|e| format!("Failed to remove database directory: {}", e))?;
            eprintln!("[clipper-server] Database directory cleared");
        }

        // Remove storage directory
        if self.storage_path.exists() {
            tokio::fs::remove_dir_all(&self.storage_path)
                .await
                .map_err(|e| format!("Failed to remove storage directory: {}", e))?;
            eprintln!("[clipper-server] Storage directory cleared");
        }

        Ok(())
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        // Try to stop the server synchronously on drop
        // This is a best-effort cleanup
        if let Ok(mut child_guard) = self.child.try_lock() {
            if let Some(child) = child_guard.take() {
                let _ = child.kill();
            }
        }
    }
}

/// Get the application data directory for storing server data
pub fn get_server_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))
}
