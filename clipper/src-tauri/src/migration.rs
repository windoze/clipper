use std::path::{Path, PathBuf};
use tokio::fs;

const OLD_APP_IDENTIFIER: &str = "com.0d0a.clipper";

/// Migrate data from old app identifier location to new location.
/// This runs once on first startup if old data exists and new location is empty.
/// Data is moved (not copied) to avoid duplication.
pub async fn migrate_from_old_location(
    new_config_dir: &Path,
    new_data_dir: &Path,
) -> Result<(), String> {
    // Get old locations based on platform
    let (old_config_dir, old_data_dir) = get_old_directories()?;

    // Check if old config exists
    let old_settings_file = old_config_dir.join("settings.json");
    let new_settings_file = new_config_dir.join("settings.json");

    // Only migrate if old data exists AND new data doesn't exist yet
    let should_migrate_config = old_settings_file.exists() && !new_settings_file.exists();
    let should_migrate_data = old_data_dir.exists() && !new_data_dir.exists();

    if !should_migrate_config && !should_migrate_data {
        return Ok(());
    }

    eprintln!(
        "[migration] Detected old app data at {}",
        old_config_dir.display()
    );

    // Migrate config directory (settings.json)
    if should_migrate_config {
        eprintln!(
            "[migration] Migrating config from {} to {}",
            old_config_dir.display(),
            new_config_dir.display()
        );

        // Ensure new config directory exists
        fs::create_dir_all(new_config_dir)
            .await
            .map_err(|e| format!("Failed to create new config directory: {}", e))?;

        // Move settings.json
        if old_settings_file.exists() {
            move_file(&old_settings_file, &new_settings_file).await?;
            eprintln!("[migration] Moved settings.json");
        }

        // Move certs directory if it exists (for ACME certificates)
        let old_certs_dir = old_config_dir.join("certs");
        let new_certs_dir = new_config_dir.join("certs");
        if old_certs_dir.exists() {
            move_dir(&old_certs_dir, &new_certs_dir).await?;
            eprintln!("[migration] Moved certs directory");
        }
    }

    // Migrate data directory (db/, storage/)
    if should_migrate_data {
        eprintln!(
            "[migration] Migrating data from {} to {}",
            old_data_dir.display(),
            new_data_dir.display()
        );

        // Move db directory
        let old_db_dir = old_data_dir.join("db");
        let new_db_dir = new_data_dir.join("db");
        if old_db_dir.exists() {
            move_dir(&old_db_dir, &new_db_dir).await?;
            eprintln!("[migration] Moved db directory");
        }

        // Move storage directory
        let old_storage_dir = old_data_dir.join("storage");
        let new_storage_dir = new_data_dir.join("storage");
        if old_storage_dir.exists() {
            move_dir(&old_storage_dir, &new_storage_dir).await?;
            eprintln!("[migration] Moved storage directory");
        }
    }

    // Clean up empty old directories
    cleanup_empty_dir(&old_data_dir).await;
    cleanup_empty_dir(&old_config_dir).await;

    eprintln!("[migration] Migration completed successfully");

    Ok(())
}

/// Get the old config and data directories based on platform
fn get_old_directories() -> Result<(PathBuf, PathBuf), String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Failed to get home directory")?;
        let old_config = home
            .join("Library")
            .join("Application Support")
            .join(OLD_APP_IDENTIFIER);
        let old_data = old_config.clone(); // macOS uses same dir for config and data
        Ok((old_config, old_data))
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Failed to get home directory")?;
        let old_config = home.join(".config").join(OLD_APP_IDENTIFIER);
        let old_data = home.join(".local/share").join(OLD_APP_IDENTIFIER);
        Ok((old_config, old_data))
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = dirs::data_dir().ok_or("Failed to get AppData directory")?;
        let old_config = app_data.join(OLD_APP_IDENTIFIER);
        let old_data = old_config.clone(); // Windows uses same dir
        Ok((old_config, old_data))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Unsupported platform for migration".to_string())
    }
}

/// Move a file from src to dst, falling back to copy+delete if rename fails (cross-device)
async fn move_file(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    // Try rename first (fast, same filesystem)
    match fs::rename(src, dst).await {
        Ok(()) => Ok(()),
        Err(_) => {
            // Fall back to copy + delete (cross-device move)
            fs::copy(src, dst)
                .await
                .map_err(|e| format!("Failed to copy {}: {}", src.display(), e))?;
            fs::remove_file(src)
                .await
                .map_err(|e| format!("Failed to remove {}: {}", src.display(), e))?;
            Ok(())
        }
    }
}

/// Move a directory from src to dst
async fn move_dir(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    // Try rename first (fast, same filesystem)
    match fs::rename(src, dst).await {
        Ok(()) => Ok(()),
        Err(_) => {
            // Fall back to recursive copy + delete (cross-device move)
            copy_dir_recursive(src, dst).await?;
            fs::remove_dir_all(src)
                .await
                .map_err(|e| format!("Failed to remove {}: {}", src.display(), e))?;
            Ok(())
        }
    }
}

/// Recursively copy a directory (used as fallback for cross-device moves)
async fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dst)
        .await
        .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;

    let mut entries = fs::read_dir(src)
        .await
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let file_type = entry
            .file_type()
            .await
            .map_err(|e| format!("Failed to get file type: {}", e))?;

        if file_type.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            fs::copy(&src_path, &dst_path)
                .await
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}

/// Remove a directory if it's empty
async fn cleanup_empty_dir(dir: &PathBuf) {
    if !dir.exists() {
        return;
    }

    // Check if directory is empty
    if let Ok(mut entries) = fs::read_dir(dir).await
        && entries.next_entry().await.ok().flatten().is_none()
    {
        // Directory is empty, remove it
        let _ = fs::remove_dir(dir).await;
    }
}
