//! The system tray icon and its menu.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{include_image, AppHandle};

const SHOW_ID: &str = "show";
const QUIT_ID: &str = "quit";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, SHOW_ID, "Show Marey", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit Marey", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    // macOS tints a template image to suit the menu bar. Other platforms get the light variant,
    // which suits their usual dark bars.
    #[cfg(target_os = "macos")]
    let icon = include_image!("icons/tray/tray-icon-template@2x.png");
    #[cfg(not(target_os = "macos"))]
    let icon = include_image!("icons/tray/tray-icon-light@2x.png");

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Marey")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_ID => crate::focus_main_window(app),
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
