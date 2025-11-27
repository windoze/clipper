// Not used under Windows
#[allow(unused_imports)]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
const LAUNCH_AGENT_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.chenxu.clipper</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#;

/// Enable or disable auto-launch on login
pub async fn set_auto_launch(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        set_auto_launch_macos(enabled).await
    }

    #[cfg(target_os = "linux")]
    {
        set_auto_launch_linux(enabled).await
    }

    #[cfg(target_os = "windows")]
    {
        set_auto_launch_windows(enabled).await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Auto-launch is not supported on this platform".to_string())
    }
}

/// Check if auto-launch is currently enabled
pub fn is_auto_launch_enabled() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        is_auto_launch_enabled_macos()
    }

    #[cfg(target_os = "linux")]
    {
        is_auto_launch_enabled_linux()
    }

    #[cfg(target_os = "windows")]
    {
        is_auto_launch_enabled_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
fn get_launch_agent_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("com.chenxu.clipper.plist"))
}

#[cfg(target_os = "macos")]
fn get_app_executable_path() -> Result<PathBuf, String> {
    // Get the path to the running executable
    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    // In development, use the exe path directly
    // In production (inside .app bundle), use the .app path
    if let Some(app_path) = exe_path
        .ancestors()
        .find(|p| p.extension().is_some_and(|e| e == "app"))
    {
        Ok(app_path.to_path_buf())
    } else {
        // Development mode - use the binary directly
        Ok(exe_path)
    }
}

#[cfg(target_os = "macos")]
async fn set_auto_launch_macos(enabled: bool) -> Result<(), String> {
    let launch_agent_path = get_launch_agent_path()?;

    if enabled {
        // Create LaunchAgents directory if it doesn't exist
        if let Some(parent) = launch_agent_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;
        }

        // Get the app executable path
        let app_path = get_app_executable_path()?;
        let plist_content = LAUNCH_AGENT_PLIST.replace("{}", &app_path.to_string_lossy());

        tokio::fs::write(&launch_agent_path, plist_content)
            .await
            .map_err(|e| format!("Failed to write launch agent: {}", e))?;
    } else {
        // Remove the launch agent if it exists
        if launch_agent_path.exists() {
            tokio::fs::remove_file(&launch_agent_path)
                .await
                .map_err(|e| format!("Failed to remove launch agent: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn is_auto_launch_enabled_macos() -> Result<bool, String> {
    let launch_agent_path = get_launch_agent_path()?;
    Ok(launch_agent_path.exists())
}

#[cfg(target_os = "linux")]
fn get_autostart_path() -> Result<PathBuf, String> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    Ok(config_dir.join("autostart").join("clipper.desktop"))
}

#[cfg(target_os = "linux")]
async fn set_auto_launch_linux(enabled: bool) -> Result<(), String> {
    let autostart_path = get_autostart_path()?;

    if enabled {
        // Create autostart directory if it doesn't exist
        if let Some(parent) = autostart_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create autostart directory: {}", e))?;
        }

        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

        let desktop_content = format!(
            r#"[Desktop Entry]
Type=Application
Name=Clipper
Exec={}
X-GNOME-Autostart-enabled=true
"#,
            exe_path.to_string_lossy()
        );

        tokio::fs::write(&autostart_path, desktop_content)
            .await
            .map_err(|e| format!("Failed to write desktop entry: {}", e))?;
    } else {
        if autostart_path.exists() {
            tokio::fs::remove_file(&autostart_path)
                .await
                .map_err(|e| format!("Failed to remove desktop entry: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn is_auto_launch_enabled_linux() -> Result<bool, String> {
    let autostart_path = get_autostart_path()?;
    Ok(autostart_path.exists())
}

#[cfg(target_os = "windows")]
fn get_registry_key() -> &'static str {
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run"
}

#[cfg(target_os = "windows")]
async fn set_auto_launch_windows(enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(get_registry_key(), KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    if enabled {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
        run_key
            .set_value("Clipper", &exe_path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
    } else {
        // Ignore error if value doesn't exist
        let _ = run_key.delete_value("Clipper");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn is_auto_launch_enabled_windows() -> Result<bool, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(get_registry_key(), KEY_READ)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    match run_key.get_value::<String, _>("Clipper") {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
