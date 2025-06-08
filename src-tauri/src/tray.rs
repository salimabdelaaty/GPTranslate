use anyhow::Result;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

fn get_theme_icon() -> Result<Image<'static>> {
    // Use the high-quality icon file for the tray
    let icon_bytes = include_bytes!("../icons/icon.ico");
    Image::from_bytes(icon_bytes).map_err(|e| anyhow::anyhow!("Failed to load icon: {}", e))
}

pub fn create_tray(app: &AppHandle) -> Result<()> {
    // Create tray menu
    let open_item = MenuItem::with_id(app, "open", "Open GPTranslate", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    // Get icon for tray
    let icon = match get_theme_icon() {
        Ok(icon) => icon,
        Err(e) => {
            log::warn!("Failed to load tray icon: {}", e);
            // Fallback to default window icon
            app.default_window_icon()
                .ok_or_else(|| anyhow::anyhow!("No default window icon available"))?
                .clone()
        }
    };

    // Create tray icon
    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("GPTranslate")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let Some(app) = tray.app_handle().get_webview_window("main") {
                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let _ = app.show();
                        let _ = app.set_focus();
                        log::info!("Tray icon left clicked - showing main window");
                    }
                    TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let _ = app.show();
                        let _ = app.set_focus();
                        log::info!("Tray icon double clicked - showing main window");
                    }
                    _ => {}
                }
            }
        })
        .build(app)?;

    Ok(())
}
