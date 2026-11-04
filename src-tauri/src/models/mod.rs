//! Serializable data shared with the frontend. TypeScript types are generated from these
//! definitions into `src/types/generated` by `task types`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub type Vec3 = [f32; 3];

/// One bone segment, in millimetres in the device coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Bone {
    pub prev: Vec3,
    pub next: Vec3,
    pub width: f32,
}

/// One finger, with bones ordered metacarpal, proximal, intermediate, distal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Digit {
    pub bones: [Bone; 4],
    pub extended: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Palm {
    pub position: Vec3,
    pub normal: Vec3,
    pub direction: Vec3,
    pub velocity: Vec3,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum HandSide {
    Left,
    Right,
}

/// One tracked hand. Digits are ordered thumb, index, middle, ring, pinky.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Hand {
    pub id: u32,
    pub side: HandSide,
    pub confidence: f32,
    pub grab_strength: f32,
    pub pinch_strength: f32,
    pub pinch_distance: f32,
    #[ts(type = "number")]
    pub visible_time_us: u64,
    pub palm: Palm,
    pub digits: [Digit; 5],
    pub arm: Bone,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct TrackingFrame {
    #[ts(type = "number")]
    pub frame_id: i64,
    /// Device frame rate reported by the source, in Hz.
    pub framerate: f32,
    pub hands: Vec<Hand>,
}

/// Decoded device status. Failure states are exclusive codes in LeapC, not bit flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DeviceFlags {
    pub streaming: bool,
    pub paused: bool,
    pub robust: bool,
    pub smudged: bool,
    pub low_resource: bool,
    pub unknown_failure: bool,
    pub bad_calibration: bool,
    pub bad_firmware: bool,
    pub bad_transport: bool,
    pub bad_control: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DeviceSummary {
    pub id: u32,
    pub product: String,
    pub pid: u32,
    pub serial: String,
    /// Firmware version, when the source exposes one. LeapC 6.2 has no public call for it.
    pub firmware: Option<String>,
    pub raw_status: u32,
    pub flags: DeviceFlags,
    pub baseline_mm: f32,
    pub h_fov_deg: f32,
    pub v_fov_deg: f32,
    pub range_mm: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum LinkState {
    /// The app is starting and has not tried to reach LeapC yet.
    Starting,
    /// The LeapC library could not be loaded.
    SdkMissing,
    /// The library loaded, and it reports that the tracking service is not running.
    ServiceNotRunning,
    /// Waiting for the link to the service to be established.
    Connecting,
    Connected,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct LinkStatus {
    pub state: LinkState,
    pub detail: Option<String>,
    /// True only when the explicit simulation mode is active.
    pub mock: bool,
    pub client_version: Option<String>,
    pub service_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct PolicyState {
    pub raw: u32,
    pub background_frames: bool,
    pub images: bool,
    pub optimize_hmd: bool,
    pub optimize_screen_top: bool,
    pub allow_pause_resume: bool,
    pub map_points: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum TrackingMode {
    Desktop,
    Hmd,
    ScreenTop,
    Unknown,
}

/// User preferences, persisted and re-applied on startup and on every device connection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Settings {
    pub tracking_mode: TrackingMode,
    pub paused: bool,
    pub images_enabled: bool,
    pub background_frames: bool,
    pub frame_rate_hz: u32,
    pub image_rate_hz: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tracking_mode: TrackingMode::Desktop,
            paused: false,
            images_enabled: false,
            background_frames: true,
            frame_rate_hz: 60,
            image_rate_hz: 30,
        }
    }
}

impl Settings {
    /// Returns a copy with the stream rates limited to the range the UI can use.
    pub fn sanitized(mut self) -> Self {
        self.frame_rate_hz = self.frame_rate_hz.clamp(1, 240);
        self.image_rate_hz = self.image_rate_hz.clamp(1, 120);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct FrameStats {
    /// Tracking frames received from the source per second.
    pub tracking_fps: f32,
    /// Frame rate reported inside the latest tracking frame.
    pub reported_fps: f32,
    pub image_fps: f32,
    #[ts(type = "number")]
    pub frames_received: u64,
    #[ts(type = "number")]
    pub frames_sent: u64,
    #[ts(type = "number")]
    pub images_received: u64,
    #[ts(type = "number")]
    pub images_sent: u64,
    #[ts(type = "number")]
    pub frames_dropped: u64,
}

/// Everything the UI shows except the high-rate streams.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct PanelState {
    pub link: LinkStatus,
    pub devices: Vec<DeviceSummary>,
    pub active_device_id: Option<u32>,
    pub policies: PolicyState,
    pub tracking_mode: TrackingMode,
    pub settings: Settings,
    pub stats: FrameStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum LogSeverity {
    Unknown,
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum LogSource {
    Leapc,
    App,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct LogEntry {
    /// Milliseconds since the Unix epoch.
    #[ts(type = "number")]
    pub timestamp_ms: u64,
    pub severity: LogSeverity,
    pub source: LogSource,
    pub message: String,
}
