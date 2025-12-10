//! GTK3 Header Bar integration for Linux
//!
//! This module provides native GTK client-side decorations (CSD) for the Linux platform.
//! It accesses Tauri's underlying GTK window and installs a custom HeaderBar with
//! status indicators and window controls that integrate with Tauri via events.

use gtk::glib;
use gtk::prelude::*;
use gtk::{HeaderBar, Image, Label, Button, Box as GtkBox, Orientation};
use log::{error, info};
use std::cell::RefCell;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// Global reference to the header bar components for status updates
thread_local! {
    static HEADER_COMPONENTS: RefCell<Option<HeaderComponents>> = const { RefCell::new(None) };
}

struct HeaderComponents {
    server_indicator: Image,
    ws_indicator: Image,
    maximize_btn: Button,
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
    let header_bar = create_headerbar(app, &gtk_window);

    // Set the header bar as the window's titlebar
    gtk_window.set_titlebar(Some(&header_bar));

    // Show all widgets
    header_bar.show_all();

    info!("GTK HeaderBar installed successfully");

    Ok(())
}

/// Create the HeaderBar widget with all controls
fn create_headerbar(app: &AppHandle, gtk_window: &gtk::ApplicationWindow) -> HeaderBar {
    let header_bar = HeaderBar::new();

    // Enable showing window controls (minimize, maximize, close)
    header_bar.set_show_close_button(true);
    header_bar.set_decoration_layout(Some("menu:minimize,maximize,close"));

    // Set title and subtitle
    header_bar.set_title(Some("Clipper"));
    header_bar.set_subtitle(Some("Clipboard Manager"));

    // === Left side: Status indicators ===
    let status_box = GtkBox::new(Orientation::Horizontal, 8);

    // Server connection indicator
    let server_indicator = Image::from_icon_name(Some("network-server-symbolic"), gtk::IconSize::SmallToolbar);
    server_indicator.set_opacity(0.3);
    server_indicator.set_tooltip_text(Some("Server: Disconnected"));
    status_box.pack_start(&server_indicator, false, false, 0);

    // WebSocket indicator
    let ws_indicator = Image::from_icon_name(Some("network-transmit-receive-symbolic"), gtk::IconSize::SmallToolbar);
    ws_indicator.set_opacity(0.3);
    ws_indicator.set_tooltip_text(Some("WebSocket: Disconnected"));
    status_box.pack_start(&ws_indicator, false, false, 0);

    header_bar.pack_start(&status_box);

    // === Right side: Settings button ===
    let settings_button = Button::from_icon_name(Some("emblem-system-symbolic"), gtk::IconSize::SmallToolbar);
    settings_button.set_tooltip_text(Some("Settings"));
    settings_button.style_context().add_class("flat");

    let app_for_settings = app.clone();
    settings_button.connect_clicked(move |_| {
        info!("GTK Settings button clicked");
        let _ = app_for_settings.emit("gtk-settings-clicked", ());
    });

    header_bar.pack_end(&settings_button);

    // Create a dummy maximize button reference for state tracking
    // The actual buttons are handled by GTK's show_close_button
    let maximize_btn = Button::new();

    // Store references for status updates
    HEADER_COMPONENTS.with(|components| {
        *components.borrow_mut() = Some(HeaderComponents {
            server_indicator: server_indicator.clone(),
            ws_indicator: ws_indicator.clone(),
            maximize_btn,
        });
    });

    header_bar
}

/// Update the server connection status indicator
pub fn update_server_indicator(connected: bool) {
    glib::idle_add_local_once(move || {
        HEADER_COMPONENTS.with(|components| {
            if let Some(ref comp) = *components.borrow() {
                comp.server_indicator.set_opacity(if connected { 1.0 } else { 0.3 });
                comp.server_indicator.set_tooltip_text(Some(if connected {
                    "Server: Connected"
                } else {
                    "Server: Disconnected"
                }));

                // Update style class for color
                let ctx = comp.server_indicator.style_context();
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

/// Update the WebSocket connection status indicator
pub fn update_websocket_indicator(connected: bool) {
    glib::idle_add_local_once(move || {
        HEADER_COMPONENTS.with(|components| {
            if let Some(ref comp) = *components.borrow() {
                comp.ws_indicator.set_opacity(if connected { 1.0 } else { 0.3 });
                comp.ws_indicator.set_tooltip_text(Some(if connected {
                    "WebSocket: Connected"
                } else {
                    "WebSocket: Disconnected"
                }));

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
