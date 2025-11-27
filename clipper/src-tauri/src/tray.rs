#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{
    include_image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use crate::tray_i18n::{t, Language};

pub fn setup_tray(app: &AppHandle, language: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lang = Language::from_str(language);

    let show_hide_item = MenuItem::with_id(
        app,
        "show_hide",
        t(lang, "tray.showHide"),
        true,
        None::<&str>,
    )?;
    let settings_item = MenuItem::with_id(
        app,
        "settings",
        t(lang, "tray.settings"),
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", t(lang, "tray.quit"), true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show_hide_item, &settings_item, &separator, &quit_item],
    )?;

    // Use the tray icon embedded at compile time via include_image! macro
    let tray_icon = include_image!("icons/tray-icon.png");

    let _tray = TrayIconBuilder::new()
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                        // Hide dock icon on macOS
                        #[cfg(target_os = "macos")]
                        let _ = app.set_activation_policy(ActivationPolicy::Accessory);
                    } else {
                        // Show dock icon on macOS before showing window
                        #[cfg(target_os = "macos")]
                        let _ = app.set_activation_policy(ActivationPolicy::Regular);
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "settings" => {
                // Show the window first if hidden
                if let Some(window) = app.get_webview_window("main") {
                    if !window.is_visible().unwrap_or(false) {
                        #[cfg(target_os = "macos")]
                        let _ = app.set_activation_policy(ActivationPolicy::Regular);
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                // Emit event to open settings dialog in the frontend
                let _ = app.emit("open-settings", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                        // Hide dock icon on macOS
                        #[cfg(target_os = "macos")]
                        let _ = app.set_activation_policy(ActivationPolicy::Accessory);
                    } else {
                        // Show dock icon on macOS before showing window
                        #[cfg(target_os = "macos")]
                        let _ = app.set_activation_policy(ActivationPolicy::Regular);
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Update the tray menu language
pub fn update_tray_language(
    app: &AppHandle,
    language: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let lang = Language::from_str(language);

    // Get all tray icons and update the first one (there should only be one)
    let trays = app.tray_by_id("main");
    if let Some(tray) = trays {
        // Create new menu items with updated language
        let show_hide_item = MenuItem::with_id(
            app,
            "show_hide",
            t(lang, "tray.showHide"),
            true,
            None::<&str>,
        )?;
        let settings_item = MenuItem::with_id(
            app,
            "settings",
            t(lang, "tray.settings"),
            true,
            None::<&str>,
        )?;
        let separator = PredefinedMenuItem::separator(app)?;
        let quit_item = MenuItem::with_id(app, "quit", t(lang, "tray.quit"), true, None::<&str>)?;

        let menu = Menu::with_items(
            app,
            &[&show_hide_item, &settings_item, &separator, &quit_item],
        )?;

        tray.set_menu(Some(menu))?;
    }

    Ok(())
}
