//! Pure conversions from LeapC numeric values to models.
//!
//! The values are from `LeapC.h` and are checked against the generated bindings in
//! `leapc_source::tests`, so this module needs neither the SDK nor a device.

use crate::models::{DeviceFlags, LinkState, LogSeverity, PolicyState, TrackingMode};

pub mod policy {
    pub const BACKGROUND_FRAMES: u32 = 0x1;
    pub const IMAGES: u32 = 0x2;
    pub const OPTIMIZE_HMD: u32 = 0x4;
    pub const ALLOW_PAUSE_RESUME: u32 = 0x8;
    pub const MAP_POINTS: u32 = 0x80;
    pub const OPTIMIZE_SCREEN_TOP: u32 = 0x100;
}

pub mod status {
    pub const STREAMING: u32 = 0x1;
    pub const PAUSED: u32 = 0x2;
    pub const ROBUST: u32 = 0x4;
    pub const SMUDGED: u32 = 0x8;
    pub const LOW_RESOURCE: u32 = 0x10;
    pub const UNKNOWN_FAILURE: u32 = 0xE801_0000;
    pub const BAD_CALIBRATION: u32 = 0xE801_0001;
    pub const BAD_FIRMWARE: u32 = 0xE801_0002;
    pub const BAD_TRANSPORT: u32 = 0xE801_0003;
    pub const BAD_CONTROL: u32 = 0xE801_0004;
}

pub mod connection {
    pub const NOT_CONNECTED: u32 = 0;
    pub const CONNECTED: u32 = 1;
    pub const HANDSHAKE_INCOMPLETE: u32 = 2;
    pub const NOT_RUNNING: u32 = 0xE703_0004;
}

pub fn policy_state(raw: u32) -> PolicyState {
    let has = |flag| raw & flag != 0;
    PolicyState {
        raw,
        background_frames: has(policy::BACKGROUND_FRAMES),
        images: has(policy::IMAGES),
        optimize_hmd: has(policy::OPTIMIZE_HMD),
        optimize_screen_top: has(policy::OPTIMIZE_SCREEN_TOP),
        allow_pause_resume: has(policy::ALLOW_PAUSE_RESUME),
        map_points: has(policy::MAP_POINTS),
    }
}

/// Decodes a device status. Values from `0xE8010000` up are single failure codes, whose low bits
/// must not be read as flags.
pub fn device_flags(raw: u32) -> DeviceFlags {
    let mut flags = DeviceFlags::default();
    if raw >= status::UNKNOWN_FAILURE {
        match raw {
            status::BAD_CALIBRATION => flags.bad_calibration = true,
            status::BAD_FIRMWARE => flags.bad_firmware = true,
            status::BAD_TRANSPORT => flags.bad_transport = true,
            status::BAD_CONTROL => flags.bad_control = true,
            _ => flags.unknown_failure = true,
        }
        return flags;
    }
    flags.streaming = raw & status::STREAMING != 0;
    flags.paused = raw & status::PAUSED != 0;
    flags.robust = raw & status::ROBUST != 0;
    flags.smudged = raw & status::SMUDGED != 0;
    flags.low_resource = raw & status::LOW_RESOURCE != 0;
    flags
}

pub fn tracking_mode(raw: u32) -> TrackingMode {
    match raw {
        0 => TrackingMode::Desktop,
        1 => TrackingMode::Hmd,
        2 => TrackingMode::ScreenTop,
        _ => TrackingMode::Unknown,
    }
}

/// The LeapC value for a mode, or `None` for a mode that cannot be requested.
pub fn tracking_mode_value(mode: TrackingMode) -> Option<u32> {
    match mode {
        TrackingMode::Desktop => Some(0),
        TrackingMode::Hmd => Some(1),
        TrackingMode::ScreenTop => Some(2),
        TrackingMode::Unknown => None,
    }
}

pub fn log_severity(raw: u32) -> LogSeverity {
    match raw {
        1 => LogSeverity::Critical,
        2 => LogSeverity::Warning,
        3 => LogSeverity::Info,
        _ => LogSeverity::Unknown,
    }
}

pub fn link_state(raw: u32) -> LinkState {
    match raw {
        connection::CONNECTED => LinkState::Connected,
        connection::NOT_RUNNING => LinkState::ServiceNotRunning,
        connection::NOT_CONNECTED | connection::HANDSHAKE_INCOMPLETE => LinkState::Connecting,
        _ => LinkState::Connecting,
    }
}

/// Product name for a device PID. Unlisted products fall back to the name LeapC reports.
pub fn product_name(pid: u32) -> Option<&'static str> {
    match pid {
        0x0003 => Some("Leap Motion Controller"),
        0x1203 => Some("Ultraleap Stereo IR 170"),
        0x1204 => Some("Ultraleap 3Di"),
        0x1206 => Some("Leap Motion Controller 2"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_flags_decode_independently() {
        let state = policy_state(policy::IMAGES | policy::ALLOW_PAUSE_RESUME);
        assert!(state.images && state.allow_pause_resume);
        assert!(!state.background_frames && !state.optimize_hmd);
        assert_eq!(state.raw, 0xA);
    }

    #[test]
    fn device_status_flags_combine() {
        let flags = device_flags(status::STREAMING | status::SMUDGED);
        assert!(flags.streaming && flags.smudged);
        assert!(!flags.paused && !flags.bad_control);
    }

    #[test]
    fn device_failures_are_codes_not_flags() {
        // 0xE8010001 has bit 0 set, which must not read as streaming.
        let flags = device_flags(status::BAD_CALIBRATION);
        assert!(flags.bad_calibration);
        assert!(!flags.streaming);
        assert_eq!(
            device_flags(status::UNKNOWN_FAILURE),
            DeviceFlags {
                unknown_failure: true,
                ..Default::default()
            }
        );
        assert!(device_flags(0xE801_0099).unknown_failure);
    }

    #[test]
    fn idle_device_has_no_flags() {
        assert_eq!(device_flags(0), DeviceFlags::default());
    }

    #[test]
    fn tracking_modes_round_trip() {
        for mode in [
            TrackingMode::Desktop,
            TrackingMode::Hmd,
            TrackingMode::ScreenTop,
        ] {
            assert_eq!(tracking_mode(tracking_mode_value(mode).unwrap()), mode);
        }
        assert_eq!(tracking_mode(3), TrackingMode::Unknown);
        assert_eq!(tracking_mode_value(TrackingMode::Unknown), None);
    }

    #[test]
    fn link_states_map() {
        assert_eq!(link_state(connection::CONNECTED), LinkState::Connected);
        assert_eq!(
            link_state(connection::NOT_RUNNING),
            LinkState::ServiceNotRunning
        );
        assert_eq!(
            link_state(connection::HANDSHAKE_INCOMPLETE),
            LinkState::Connecting
        );
    }

    #[test]
    fn log_severity_maps_unknown_values() {
        assert_eq!(log_severity(2), LogSeverity::Warning);
        assert_eq!(log_severity(99), LogSeverity::Unknown);
    }
}
