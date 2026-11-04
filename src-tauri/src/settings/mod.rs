//! Persistence of user preferences through the Tauri store plugin.

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::error::AppError;
use crate::models::Settings;

const STORE_FILE: &str = "settings.json";
const KEY: &str = "settings";

/// Reads the saved preferences, falling back to defaults when they are absent or invalid.
pub fn load(app: &AppHandle) -> Settings {
    let stored = app
        .store(STORE_FILE)
        .map_err(|e| log::warn!("could not open the settings store: {e}"))
        .ok()
        .and_then(|store| store.get(KEY));
    parse(stored)
}

fn parse(stored: Option<serde_json::Value>) -> Settings {
    stored
        .and_then(|value| {
            serde_json::from_value::<Settings>(value)
                .map_err(|e| log::warn!("ignoring invalid saved settings: {e}"))
                .ok()
        })
        .unwrap_or_default()
        .sanitized()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Settings(e.to_string()))?;
    let value = serde_json::to_value(settings).map_err(|e| AppError::Settings(e.to_string()))?;
    store.set(KEY, value);
    store.save().map_err(|e| AppError::Settings(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TrackingMode;
    use serde_json::json;

    #[test]
    fn absent_settings_give_defaults() {
        assert_eq!(parse(None), Settings::default());
    }

    #[test]
    fn invalid_settings_give_defaults() {
        assert_eq!(
            parse(Some(json!({"trackingMode": "nope"}))),
            Settings::default()
        );
    }

    #[test]
    fn partial_settings_keep_other_defaults() {
        let settings = parse(Some(json!({"trackingMode": "hmd", "imageRateHz": 9999})));
        assert_eq!(settings.tracking_mode, TrackingMode::Hmd);
        assert_eq!(settings.image_rate_hz, 120);
        assert_eq!(settings.frame_rate_hz, 60);
    }

    #[test]
    fn settings_round_trip_as_json() {
        let settings = Settings {
            paused: true,
            images_enabled: true,
            ..Settings::default()
        };
        let value = serde_json::to_value(settings).unwrap();
        assert_eq!(parse(Some(value)), settings);
    }
}
