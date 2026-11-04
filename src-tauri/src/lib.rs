#[cfg(desktop)]
use tauri::Manager;
use tauri_plugin_log::log::{self, LevelFilter};

/// Largest size of a log file before it is rotated, in bytes.
const MAX_LOG_FILE_SIZE: u128 = 5_000_000;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Brings the running instance to the front when the app is launched again.
#[cfg(desktop)]
fn focus_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("second launch detected, but the main window does not exist");
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Single instance must be the first plugin registered.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main_window(app);
        }));
    }

    // Logging comes next so the plugins below can log during their setup.
    builder = builder.plugin(
        tauri_plugin_log::Builder::new()
            .level(if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            })
            .max_file_size(MAX_LOG_FILE_SIZE)
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(5))
            .build(),
    );

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_window_state::Builder::new().build())
            .plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
