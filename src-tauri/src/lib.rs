mod commands;
mod error;
pub mod leap;
pub mod models;
mod settings;
pub mod streaming;
#[cfg(desktop)]
mod tray;

use leap::mock::{self, MockSource};
use leap::service::{self, ServiceHandle};
use leap::{SourceError, TrackingSource};
use streaming::{StreamChannels, TauriSink};
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_log::log::LevelFilter;

/// Largest size of a log file before it is rotated, in bytes.
const MAX_LOG_FILE_SIZE: u128 = 5_000_000;

/// Creates the real tracking source, or explains why it is not available.
fn open_real_source() -> Result<Box<dyn TrackingSource>, SourceError> {
    #[cfg(feature = "leapc")]
    {
        Ok(Box::new(leap::leapc_source::LeapcSource::open()?))
    }
    #[cfg(not(feature = "leapc"))]
    {
        Err(SourceError::Unavailable(
            "this build has no LeapC support; run with MAREY_MOCK=1".to_owned(),
        ))
    }
}

fn start_service(app: &tauri::AppHandle) -> std::io::Result<ServiceHandle> {
    let mock = mock::is_enabled(std::env::var(mock::MOCK_VAR).ok().as_deref());
    if mock {
        log::warn!("simulation mode is on: all device data is synthetic");
    }
    let settings = settings::load(app);
    let channels = StreamChannels::default();
    let sink = TauriSink::new(app.clone(), channels.clone(), &settings);
    app.manage(channels);
    service::spawn(
        Box::new(move || {
            if mock {
                Ok(Box::new(MockSource::new()))
            } else {
                open_real_source()
            }
        }),
        sink,
        settings,
        mock,
    )
}

/// Brings the running instance to the front when the app is launched again.
#[cfg(desktop)]
pub(crate) fn focus_main_window(app: &tauri::AppHandle) {
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .on_window_event(|window, event| {
            // Closing the window keeps the app running. It quits from the tray or the app menu.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(error) = window.hide() {
                    log::warn!("could not hide the window: {error}");
                }
            }
        })
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
            RunEvent::Reopen { .. } => focus_main_window(app),
            _ => {}
        });
}
