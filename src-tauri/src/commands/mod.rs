//! Tauri command handlers. They are `async` so waiting for a reply never blocks the main thread.

use std::sync::mpsc;
use std::time::Duration;

use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

use crate::error::AppError;
use crate::leap::service::{Command, ServiceHandle};
use crate::models::{LogEntry, PanelState, Settings};
use crate::settings;
use crate::streaming::StreamChannels;

const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

fn ask<T>(
    service: &ServiceHandle,
    make: impl FnOnce(mpsc::Sender<T>) -> Command,
) -> Result<T, AppError> {
    let (reply, answer) = mpsc::channel();
    if !service.send(make(reply)) {
        return Err(AppError::ServiceUnavailable);
    }
    answer
        .recv_timeout(REPLY_TIMEOUT)
        .map_err(|_| AppError::ServiceUnavailable)
}

#[tauri::command(async)]
pub fn get_state(service: State<'_, ServiceHandle>) -> Result<PanelState, AppError> {
    ask(&service, Command::GetState)
}

#[tauri::command(async)]
pub fn get_logs(service: State<'_, ServiceHandle>) -> Vec<LogEntry> {
    service.logs()
}

/// Registers the frontend's stream endpoints. The latest subscription replaces earlier ones.
#[tauri::command(async)]
pub fn subscribe_streams(
    channels: State<'_, StreamChannels>,
    frames: Channel<InvokeResponseBody>,
    images: Channel<InvokeResponseBody>,
) {
    channels.subscribe(frames, images);
}

#[tauri::command(async)]
pub fn select_device(service: State<'_, ServiceHandle>, id: u32) -> Result<(), AppError> {
    ask(&service, |reply| Command::SelectDevice(id, reply))?.map_err(AppError::Source)
}

/// Applies and saves preferences. They are saved even if the device rejects one of them, so the
/// choice is kept for a device that supports it. The error still reaches the UI.
#[tauri::command(async)]
pub fn update_settings(
    app: AppHandle,
    service: State<'_, ServiceHandle>,
    settings: Settings,
) -> Result<(), AppError> {
    let applied = ask(&service, |reply| Command::UpdateSettings(settings, reply))?;
    settings::save(&app, &settings.sanitized())?;
    applied.map_err(AppError::Source)
}

#[tauri::command(async)]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), AppError> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| AppError::Clipboard(e.to_string()))
}

/// Translates the tray menu. The frontend owns all translations, so it sends the text to show.
#[cfg(desktop)]
#[tauri::command(async)]
pub fn set_tray_labels(
    menu: State<'_, crate::tray::TrayMenu>,
    show: String,
    quit: String,
) -> Result<(), AppError> {
    menu.set_labels(&show, &quit)?;
    Ok(())
}

/// Asks where to save the text. Returns the chosen path, or `None` if the user cancelled.
#[tauri::command(async)]
pub fn export_text(
    app: AppHandle,
    file_name: String,
    contents: String,
) -> Result<Option<String>, AppError> {
    let Some(path) = app
        .dialog()
        .file()
        .set_file_name(file_name)
        .blocking_save_file()
        .and_then(|path| path.into_path().ok())
    else {
        return Ok(None);
    };
    std::fs::write(&path, contents)?;
    Ok(Some(path.display().to_string()))
}
