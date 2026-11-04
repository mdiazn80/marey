//! Behaviour of the main window.

#[cfg(desktop)]
use tauri::{AppHandle, Manager};
use tauri::{Window, WindowEvent};

/// Closing the window keeps the app running. It quits from the tray or the app menu.
pub fn hide_on_close(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        if let Err(error) = window.hide() {
            log::warn!("could not hide the window: {error}");
        }
    }
}

/// Brings the main window to the front, showing it again if it was hidden.
#[cfg(desktop)]
pub fn focus_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("the main window does not exist");
        return;
    };

    let result = window
        .unminimize()
        .and_then(|_| window.show())
        .and_then(|_| window.set_focus());

    if let Err(error) = result {
        log::warn!("could not focus the main window: {error}");
    }
}
