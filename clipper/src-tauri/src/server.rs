use crate::settings::SettingsManager;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, RwLock};

#[cfg(unix)]
use std::os::unix::io::IntoRawFd;

#[cfg(windows)]
use std::os::windows::io::IntoRawHandle;

/// Manages the bundled clipper-server sidecar process
pub struct ServerManager {
    /// The server process child handle
    child: Mutex<Option<Child>>,
    /// The write end of the parent monitor pipe (kept alive while server runs)
    _pipe_writer: Mutex<Option<os_pipe::PipeWriter>>,
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
            _pipe_writer: Mutex::new(None),
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

    /// Find the sidecar binary path
    /// Tauri places sidecars in Contents/MacOS/ on macOS (plain name in production,
    /// with target triple suffix during development)
    fn find_sidecar_path(app: &AppHandle) -> Result<PathBuf, String> {
        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?;

        // Get possible target triples for the sidecar name
        let target_triples = Self::get_target_triples();

        // Build list of possible sidecar names to try
        // Production builds use plain name, development builds use target triple suffix
        let mut sidecar_names: Vec<String> = vec![];

        // Plain name first (production builds)
        if cfg!(windows) {
            sidecar_names.push("clipper-server.exe".to_string());
        } else {
            sidecar_names.push("clipper-server".to_string());
        }

        // Then try with target triple suffix (development builds)
        for target_triple in &target_triples {
            if cfg!(windows) {
                sidecar_names.push(format!("clipper-server-{}.exe", target_triple));
            } else {
                sidecar_names.push(format!("clipper-server-{}", target_triple));
            }
        }

        for sidecar_name in &sidecar_names {
            // Try next to the main executable (Contents/MacOS/ on macOS)
            if let Ok(exe_path) = std::env::current_exe()
                && let Some(exe_dir) = exe_path.parent()
            {
                let sidecar_path = exe_dir.join(sidecar_name);
                if sidecar_path.exists() {
                    return Ok(sidecar_path);
                }
            }

            // Try resource_dir (some Tauri configurations)
            let sidecar_path = resource_dir.join(sidecar_name);
            if sidecar_path.exists() {
                return Ok(sidecar_path);
            }

            // Try resource_dir/binaries
            let sidecar_path = resource_dir.join("binaries").join(sidecar_name);
            if sidecar_path.exists() {
                return Ok(sidecar_path);
            }

            // Try the binaries directory in development (CWD)
            let dev_path = PathBuf::from("binaries").join(sidecar_name);
            if dev_path.exists() {
                return Ok(dev_path);
            }
        }

        Err(format!(
            "Sidecar binary not found. Looked for {:?} in exe_dir and resource_dir: {:?}",
            sidecar_names, resource_dir
        ))
    }

    /// Get possible target triples for the current platform
    /// Returns a list of triples to try, in order of preference
    /// On macOS, includes universal binary first, then architecture-specific
    fn get_target_triples() -> Vec<&'static str> {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            vec![
                "universal-apple-darwin",
                "aarch64-apple-darwin",
            ]
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            vec![
                "universal-apple-darwin",
                "x86_64-apple-darwin",
            ]
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            vec!["x86_64-unknown-linux-gnu"]
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            vec!["aarch64-unknown-linux-gnu"]
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            vec!["x86_64-pc-windows-msvc"]
        }
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            vec!["aarch64-pc-windows-msvc"]
        }
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
        if self.is_running().await
            && let Some(url) = self.server_url().await
        {
            return Ok(url);
        }

        // Get settings manager for port persistence
        let settings_manager = app
            .try_state::<SettingsManager>()
            .ok_or("Settings manager not initialized")?;

        // Try to reuse saved port, or pick a new one
        let port = if let Some(saved_port) = settings_manager.get_server_port() {
            if Self::is_port_available(saved_port) {
                log::debug!("[clipper-server] Reusing saved port: {}", saved_port);
                saved_port
            } else {
                log::debug!(
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

        // Secure the data directories and fix any incorrect permissions
        // On Unix: checks and fixes permissions to 0700/0600
        // On Windows: sets DACL to grant access only to current user
        match clipper_security::secure_directory_recursive(&self.db_path, |msg| {
            log::warn!("{}", msg)
        }) {
            Ok(count) if count > 0 => {
                log::info!("Fixed permissions on {} items in database directory", count);
            }
            Err(e) => log::warn!("Failed to secure database directory: {}", e),
            _ => {}
        }

        match clipper_security::secure_directory_recursive(&self.storage_path, |msg| {
            log::warn!("{}", msg)
        }) {
            Ok(count) if count > 0 => {
                log::info!("Fixed permissions on {} items in storage directory", count);
            }
            Err(e) => log::warn!("Failed to secure storage directory: {}", e),
            _ => {}
        }

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
        let listen_addr = if listen_on_all {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        };

        // Get cleanup settings
        let cleanup_enabled = settings_manager.get_cleanup_enabled();
        let cleanup_retention_days = settings_manager.get_cleanup_retention_days();

        // Get bundled server token for authentication
        // Always use if set - this ensures client and server have consistent auth configuration
        let bundled_server_token = settings_manager.get_bundled_server_token();

        // Get max upload size setting
        let max_upload_size_mb = settings_manager.get_max_upload_size_mb();

        // Get memory configuration settings
        let memory_threshold_mb = settings_manager.get_memory_threshold_mb();
        let rocksdb_block_cache_mb = settings_manager.get_rocksdb_block_cache_mb();
        let rocksdb_write_buffer_mb = settings_manager.get_rocksdb_write_buffer_mb();
        let rocksdb_max_write_buffer_number = settings_manager.get_rocksdb_max_write_buffer_number();

        // Log all parameters
        log::debug!(
            "[clipper-server] Starting bundled server with parameters:\n  \
             db_path: {}\n  \
             storage_path: {}\n  \
             listen_addr: {}\n  \
             port: {}\n  \
             cleanup_enabled: {}\n  \
             cleanup_retention_days: {}\n  \
             auth_enabled: {}\n  \
             max_upload_size_mb: {}\n  \
             memory_threshold_mb: {}\n  \
             rocksdb_block_cache_mb: {}\n  \
             rocksdb_write_buffer_mb: {}\n  \
             rocksdb_max_write_buffer_number: {}",
            db_path_str,
            storage_path_str,
            listen_addr,
            port,
            cleanup_enabled,
            cleanup_retention_days,
            bundled_server_token.is_some(),
            max_upload_size_mb,
            memory_threshold_mb,
            rocksdb_block_cache_mb,
            rocksdb_write_buffer_mb,
            rocksdb_max_write_buffer_number
        );

        // Build environment variables for server configuration
        // Using env vars instead of CLI args to avoid exposing sensitive data (like tokens) in process listings
        let mut env_vars: Vec<(String, String)> = vec![
            ("CLIPPER_DB_PATH".to_string(), db_path_str),
            ("CLIPPER_STORAGE_PATH".to_string(), storage_path_str),
            ("CLIPPER_LISTEN_ADDR".to_string(), listen_addr.to_string()),
            ("PORT".to_string(), port.to_string()),
            (
                "CLIPPER_CLEANUP_ENABLED".to_string(),
                cleanup_enabled.to_string(),
            ),
            (
                "CLIPPER_CLEANUP_RETENTION_DAYS".to_string(),
                cleanup_retention_days.to_string(),
            ),
            (
                "CLIPPER_MAX_UPLOAD_SIZE_MB".to_string(),
                max_upload_size_mb.to_string(),
            ),
            // SurrealDB memory threshold (format: "256mb")
            (
                "SURREAL_MEMORY_THRESHOLD".to_string(),
                format!("{}mb", memory_threshold_mb),
            ),
            // RocksDB block cache size in bytes
            (
                "SURREAL_ROCKSDB_BLOCK_CACHE_SIZE".to_string(),
                (rocksdb_block_cache_mb * 1024 * 1024).to_string(),
            ),
            // RocksDB write buffer size in bytes
            (
                "SURREAL_ROCKSDB_WRITE_BUFFER_SIZE".to_string(),
                (rocksdb_write_buffer_mb * 1024 * 1024).to_string(),
            ),
            // RocksDB max write buffer number
            (
                "SURREAL_ROCKSDB_MAX_WRITE_BUFFER_NUMBER".to_string(),
                rocksdb_max_write_buffer_number.to_string(),
            ),
        ];

        // Add bearer token if external access is enabled and token is set
        if let Some(ref token) = bundled_server_token {
            env_vars.push(("CLIPPER_BEARER_TOKEN".to_string(), token.clone()));
        }

        // Only arg needed is the parent pipe handle (not sensitive)
        let mut args: Vec<String> = vec![];

        // Create a pipe for parent process monitoring
        // The child will monitor the read-end; when parent exits, the pipe closes
        let (pipe_reader, pipe_writer) =
            os_pipe::pipe().map_err(|e| format!("Failed to create monitor pipe: {}", e))?;

        // Get the raw handle/fd to pass to the child
        #[cfg(unix)]
        let pipe_handle = {
            use std::os::unix::io::AsRawFd;
            pipe_reader.as_raw_fd() as u64
        };

        #[cfg(windows)]
        let pipe_handle = {
            use std::os::windows::io::AsRawHandle;
            pipe_reader.as_raw_handle() as u64
        };

        // Add the parent pipe handle argument
        args.push("--parent-pipe-handle".to_string());
        args.push(pipe_handle.to_string());

        // Find the sidecar binary path
        let sidecar_path = Self::find_sidecar_path(app)?;
        log::debug!("[clipper-server] Sidecar path: {:?}", sidecar_path);

        // Spawn the server process using std::process::Command
        // This gives us control over handle inheritance
        let mut command = std::process::Command::new(&sidecar_path);
        command
            .args(&args)
            .envs(env_vars.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On Unix, we need to prevent the pipe from being closed on exec
        // and pass it to the child. The fd is inherited by default.
        #[cfg(unix)]
        let pipe_reader_fd = {
            use std::os::unix::process::CommandExt;
            // The pipe_reader fd will be inherited by the child process
            // We need to keep it open until after spawn
            let fd = pipe_reader.into_raw_fd();
            // Update args with the actual fd value (in case it changed)
            let args_len = args.len();
            args[args_len - 1] = fd.to_string();
            // Rebuild command with updated args
            command = std::process::Command::new(&sidecar_path);
            command
                .args(&args)
                .envs(env_vars.clone())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            // Pre-exec hook to ensure the fd is not closed on exec
            unsafe {
                command.pre_exec(move || {
                    // Clear the close-on-exec flag for the pipe fd
                    let flags = libc::fcntl(fd, libc::F_GETFD);
                    if flags != -1 {
                        libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC);
                    }
                    Ok(())
                });
            }
            fd
        };

        #[cfg(windows)]
        let pipe_reader_handle = {
            use std::os::windows::process::CommandExt;
            // On Windows, we need to make the handle inheritable explicitly
            // os_pipe creates non-inheritable handles by default
            let handle = pipe_reader.into_raw_handle();

            // Make the handle inheritable using SetHandleInformation
            const HANDLE_FLAG_INHERIT: u32 = 0x00000001;
            let result = unsafe {
                windows_sys::Win32::Foundation::SetHandleInformation(
                    handle as _,
                    HANDLE_FLAG_INHERIT,
                    HANDLE_FLAG_INHERIT,
                )
            };
            if result == 0 {
                log::warn!(
                    "[clipper-server] Failed to make pipe handle inheritable"
                );
            }

            // Update args with the actual handle value
            let args_len = args.len();
            args[args_len - 1] = (handle as u64).to_string();
            // Rebuild command with updated args
            command = std::process::Command::new(&sidecar_path);
            command
                .args(&args)
                .envs(env_vars.clone())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            // CREATE_NO_WINDOW to avoid console window popup
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
            handle
        };

        let mut child = command
            .spawn()
            .map_err(|e| format!("Failed to spawn server: {}", e))?;

        // Close the parent's copy of the pipe read end after spawning.
        // The child has inherited this fd/handle, so we must close our copy.
        // Otherwise, the pipe won't signal EOF when the parent exits (because
        // there would still be an open read end in the parent process).
        #[cfg(unix)]
        unsafe {
            libc::close(pipe_reader_fd);
        }

        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(pipe_reader_handle as _);
        }

        // Spawn tasks to forward stdout/stderr with appropriate log levels
        // The bundled server uses tracing with format like:
        // "2024-01-15T10:30:00.000Z  INFO clipper_server: message"
        // "2024-01-15T10:30:00.000Z  WARN clipper_server: message"
        // "2024-01-15T10:30:00.000Z ERROR clipper_server: message"
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // Parse log level from tracing output and forward appropriately
                    forward_server_log(&line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    // Parse log level from tracing output and forward appropriately
                    forward_server_log(&line);
                }
            });
        }

        // Store the child process and pipe writer
        // The pipe writer must be kept alive - when it's dropped, the pipe closes
        *self.child.lock().await = Some(child);
        *self._pipe_writer.lock().await = Some(pipe_writer);

        // Store the port and URL
        let server_url = format!("http://127.0.0.1:{}", port);
        *self.port.write().await = Some(port);
        *self.server_url.write().await = Some(server_url.clone());

        // Save the port to settings for next startup
        if let Err(e) = settings_manager.set_server_port(port).await {
            log::warn!("[clipper-server] Failed to save port: {}", e);
        }

        // Wait a bit for the server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Try to verify the server is up by checking health endpoint
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", server_url);
        let mut retries = 10;
        while retries > 0 {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    log::info!("Bundled server started at {}", server_url);
                    return Ok(server_url);
                }
                _ => {
                    retries -= 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }
            }
        }

        // Server might still be starting, return URL anyway
        log::info!(
            "Bundled server started at {} (health check pending)",
            server_url
        );
        Ok(server_url)
    }

    /// Stop the server gracefully
    pub async fn stop(&self) -> Result<(), String> {
        // Drop the pipe writer first - this signals the child that parent is shutting down
        // But we also need to kill it explicitly for immediate shutdown
        *self._pipe_writer.lock().await = None;

        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            // Kill the process
            child
                .kill()
                .map_err(|e| format!("Failed to kill server: {}", e))?;
            // Wait for the process to exit
            let _ = child.wait();
            log::info!("Bundled server stopped");

            // Wait for the process to fully terminate and port to be released
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        *self.port.write().await = None;
        *self.server_url.write().await = None;

        Ok(())
    }

    /// Restart the server (stop and start)
    pub async fn restart(&self, app: &AppHandle) -> Result<String, String> {
        log::info!("Restarting bundled server...");
        self.stop().await?;
        self.start(app).await
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
            log::debug!("[clipper-server] Database directory cleared");
        }

        // Remove storage directory
        if self.storage_path.exists() {
            tokio::fs::remove_dir_all(&self.storage_path)
                .await
                .map_err(|e| format!("Failed to remove storage directory: {}", e))?;
            log::debug!("[clipper-server] Storage directory cleared");
        }

        Ok(())
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        // Try to stop the server synchronously on drop
        // This is a best-effort cleanup
        // First drop the pipe writer to signal the child
        if let Ok(mut pipe_guard) = self._pipe_writer.try_lock() {
            *pipe_guard = None;
        }
        // Then kill the child process
        if let Ok(mut child_guard) = self.child.try_lock()
            && let Some(mut child) = child_guard.take()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Forward a log line from the bundled server with the appropriate log level.
/// Parses the tracing format: "2024-01-15T10:30:00.000Z  INFO clipper_server: message"
/// or "2024-01-15T10:30:00.000Z DEBUG request{...}: tower_http::trace: message"
fn forward_server_log(line: &str) {
    // Tracing format: ISO 8601 timestamp followed by log level
    // Examples:
    //   "2024-01-15T10:30:00.000000Z  INFO clipper_server: message"
    //   "2024-01-15T10:30:00.000000Z DEBUG request{...}: tower_http::trace: message"
    //   "2024-01-15T10:30:00.000000Z  WARN clipper_server: message"
    //
    // Pattern: timestamp ends with "Z " and then the level follows

    // Find the 'Z' that ends the ISO 8601 timestamp, then check what follows
    fn extract_level(line: &str) -> Option<&'static str> {
        // Look for "Z " or "Z  " (timestamp end followed by space(s))
        if let Some(z_pos) = line.find('Z') {
            // Get the part after "Z"
            let after_z = &line[z_pos + 1..];
            // Skip leading spaces
            let trimmed = after_z.trim_start();
            // Check which level it starts with
            if trimmed.starts_with("ERROR") {
                return Some("ERROR");
            } else if trimmed.starts_with("WARN") {
                return Some("WARN");
            } else if trimmed.starts_with("DEBUG") {
                return Some("DEBUG");
            } else if trimmed.starts_with("TRACE") {
                return Some("TRACE");
            } else if trimmed.starts_with("INFO") {
                return Some("INFO");
            }
        }
        None
    }

    match extract_level(line) {
        Some("ERROR") => log::error!("[clipper-server] {}", line),
        Some("WARN") => log::warn!("[clipper-server] {}", line),
        Some("DEBUG") => log::debug!("[clipper-server] {}", line),
        Some("TRACE") => log::trace!("[clipper-server] {}", line),
        Some("INFO") => log::info!("[clipper-server] {}", line),
        _ => {
            // Default to INFO for unrecognized formats
            log::info!("[clipper-server] {}", line);
        }
    }
}

/// Get the application data directory for storing server data
pub fn get_server_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))
}
