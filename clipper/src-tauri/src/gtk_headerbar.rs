//! GTK3 Header Bar integration for Linux
//!
//! This module provides native GTK client-side decorations (CSD) for the Linux platform.
//! It accesses Tauri's underlying GTK window and installs a custom HeaderBar with
//! status indicators and window controls that integrate with Tauri via events.

use gtk::glib;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, HeaderBar, Image, Label, Orientation};
use log::info;
use std::cell::RefCell;
use tauri::{AppHandle, Emitter, Manager};

/// Global reference to the header bar components for status updates
thread_local! {
    static HEADER_COMPONENTS: RefCell<Option<HeaderComponents>> = const { RefCell::new(None) };
}

struct HeaderComponents {
    ws_indicator: Image,
    clip_count_label: Label,
}

/// Initialize and setup the GTK header bar for the main window.
/// This should be called during app setup on Linux.
pub fn setup_gtk_headerbar(app: &AppHandle) -> Result<(), String> {
    info!("Setting up GTK3 header bar for Linux");

    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get the GTK window from Tauri
    let gtk_window = window
        .gtk_window()
        .map_err(|e| format!("Failed to get GTK window: {}", e))?;

    // Create and configure the header bar
    let header_bar = create_headerbar(app);

    // Set the header bar as the window's titlebar
    gtk_window.set_titlebar(Some(&header_bar));

    // Show all widgets
    header_bar.show_all();

    info!("GTK HeaderBar installed successfully");

    Ok(())
}

/// Create the HeaderBar widget with all controls
fn create_headerbar(app: &AppHandle) -> HeaderBar {
    let header_bar = HeaderBar::new();

    // Enable showing window controls (minimize, maximize, close)
    header_bar.set_show_close_button(true);
    header_bar.set_decoration_layout(Some(":minimize,maximize,close"));

    // Set title only (no subtitle to save space)
    header_bar.set_title(Some("Clipper"));
    header_bar.set_has_subtitle(false);

    // === Right side controls (pack_end adds from right to left) ===

    // Refresh button (rightmost after window controls)
    let refresh_button =
        Button::from_icon_name(Some("view-refresh-symbolic"), gtk::IconSize::SmallToolbar);
    refresh_button.set_tooltip_text(Some("Refresh (Ctrl+R)"));
    refresh_button.style_context().add_class("flat");

    let app_for_refresh = app.clone();
    refresh_button.connect_clicked(move |_| {
        info!("GTK Refresh button clicked");
        let _ = app_for_refresh.emit("gtk-refresh-clicked", ());
    });
    header_bar.pack_end(&refresh_button);

    // Settings button
    let settings_button =
        Button::from_icon_name(Some("emblem-system-symbolic"), gtk::IconSize::SmallToolbar);
    settings_button.set_tooltip_text(Some("Settings (Ctrl+,)"));
    settings_button.style_context().add_class("flat");

    let app_for_settings = app.clone();
    settings_button.connect_clicked(move |_| {
        info!("GTK Settings button clicked");
        let _ = app_for_settings.emit("gtk-settings-clicked", ());
    });
    header_bar.pack_end(&settings_button);

    // Status box with connection indicator and clip count
    let status_box = GtkBox::new(Orientation::Horizontal, 6);
    status_box.set_margin_end(8);

    // WebSocket/connection indicator (single indicator for simplicity)
    let ws_indicator = Image::from_icon_name(
        Some("network-transmit-receive-symbolic"),
        gtk::IconSize::SmallToolbar,
    );
    ws_indicator.set_opacity(0.3);
    ws_indicator.set_tooltip_text(Some("Disconnected"));
    status_box.pack_start(&ws_indicator, false, false, 0);

    // Clip count label
    let clip_count_label = Label::new(Some("0"));
    clip_count_label.set_tooltip_text(Some("Total clips"));
    clip_count_label.style_context().add_class("dim-label");
    status_box.pack_start(&clip_count_label, false, false, 0);

    // Clipboard icon next to count
    let clip_icon =
        Image::from_icon_name(Some("edit-paste-symbolic"), gtk::IconSize::SmallToolbar);
    clip_icon.set_opacity(0.7);
    status_box.pack_start(&clip_icon, false, false, 0);

    header_bar.pack_end(&status_box);

    // Store references for status updates
    HEADER_COMPONENTS.with(|components| {
        *components.borrow_mut() = Some(HeaderComponents {
            ws_indicator: ws_indicator.clone(),
            clip_count_label: clip_count_label.clone(),
        });
    });

    header_bar
}

/// Update the WebSocket connection status indicator
pub fn update_websocket_indicator(connected: bool) {
    glib::idle_add_local_once(move || {
        HEADER_COMPONENTS.with(|components| {
            if let Some(ref comp) = *components.borrow() {
                comp.ws_indicator.set_opacity(if connected { 1.0 } else { 0.3 });
                comp.ws_indicator
                    .set_tooltip_text(Some(if connected { "Connected" } else { "Disconnected" }));

                let ctx = comp.ws_indicator.style_context();
                if connected {
                    ctx.add_class("success");
                    ctx.remove_class("dim-label");
                } else {
                    ctx.remove_class("success");
                    ctx.add_class("dim-label");
                }
            }
        });
    });
}

/// Update the clip count displayed in the header bar
pub fn update_clip_count(count: u64) {
    glib::idle_add_local_once(move || {
        HEADER_COMPONENTS.with(|components| {
            if let Some(ref comp) = *components.borrow() {
                comp.clip_count_label.set_text(&count.to_string());
            }
        });
    });
}

/// Update the server connection status indicator (alias for websocket indicator)
pub fn update_server_indicator(connected: bool) {
    update_websocket_indicator(connected);
}
