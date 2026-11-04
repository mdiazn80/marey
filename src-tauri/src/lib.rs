mod commands;
mod error;
pub mod leap;
pub mod models;
mod settings;
pub mod streaming;
#[cfg(desktop)]
mod tray;
mod window;

use leap::mock;
use leap::service::{self, ServiceHandle};
use streaming::{StreamChannels, TauriSink};
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, RunEvent, Runtime};
use tauri_plugin_log::log::LevelFilter;
use tauri_plugin_log::RotationStrategy;

/// Largest size of a log file before it is rotated, in bytes.
const MAX_LOG_FILE_SIZE: u128 = 5_000_000;
const LOG_FILES_KEPT: usize = 5;

fn log_plugin<R: Runtime>() -> TauriPlugin<R> {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    tauri_plugin_log::Builder::new()
        .level(level)
        .max_file_size(MAX_LOG_FILE_SIZE)
        .rotation_strategy(RotationStrategy::KeepSome(LOG_FILES_KEPT))
        .build()
}

fn start_service(app: &AppHandle) -> std::io::Result<ServiceHandle> {
    let mock = mock::is_enabled(std::env::var(mock::MOCK_VAR).ok().as_deref());
    if mock {
        log::warn!("simulation mode is on: all device data is synthetic");
    }
    let settings = settings::load(app);
    let channels = StreamChannels::default();
    let sink = TauriSink::new(app.clone(), channels.clone(), &settings);
    app.manage(channels);
    service::spawn(
        Box::new(move || leap::open_source(mock)),
        sink,
        settings,
        mock,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Single instance must be the first plugin registered.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::focus_main(app);
        }));
    }

    // Logging comes next so the plugins below can log during their setup.
    builder = builder.plugin(log_plugin());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_window_state::Builder::new().build())
            .plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .on_window_event(window::hide_on_close)
        .setup(|app| {
            #[cfg(desktop)]
            tray::create(app.handle())?;
            let service = start_service(app.handle())?;
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_logs,
            commands::subscribe_streams,
            commands::select_device,
            commands::update_settings,
            commands::copy_text,
            commands::export_text,
            #[cfg(desktop)]
            commands::set_tray_labels,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the application")
        .run(|app, event| match event {
            RunEvent::Exit => {
                if let Some(service) = app.try_state::<ServiceHandle>() {
                    service.shutdown();
                }
            }
            // Clicking the Dock icon brings back a hidden window.
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => window::focus_main(app),
            _ => {}
        });
}
