//! Tracking backend: sources, event mapping and the background service thread.
//!
//! This module has no Tauri dependency, so a future CLI or OSC bridge can reuse it.

pub mod mapping;
pub mod mock;
pub mod service;

#[cfg(feature = "leapc")]
pub mod leapc_source;

use std::time::Duration;

use crate::models::{DeviceSummary, LinkState, LogEntry, Settings, TrackingFrame};

/// A stereo camera image, borrowed from the source until the next poll.
///
/// Planes hold `width * height * bytes_per_pixel` bytes.
#[derive(Debug, Clone, Copy)]
pub struct StereoImage<'a> {
    pub frame_id: i64,
    pub width: u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
    pub left: &'a [u8],
    pub right: &'a [u8],
}

/// Something that happened in a tracking source.
#[derive(Debug)]
pub enum SourceEvent<'a> {
    Link {
        state: LinkState,
        client_version: Option<String>,
        service_version: Option<String>,
    },
    ConnectionLost,
    DeviceAdded(DeviceSummary),
    DeviceLost(u32),
    DeviceStatus {
        id: u32,
        status: u32,
    },
    DeviceFailure {
        status: u32,
    },
    Policy(u32),
    TrackingMode(u32),
    Frame(TrackingFrame),
    Images(StereoImage<'a>),
    DroppedFrame,
    Log(LogEntry),
}

/// Why a source could not be used.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    /// The backend itself is missing, so creating the source may succeed later.
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Call(String),
}

pub type EventCallback<'c> = &'c mut dyn for<'e> FnMut(SourceEvent<'e>);

/// A provider of tracking data: LeapC, the mock, or later a recording or another backend.
///
/// All methods run on the service thread, and none may block for longer than the poll timeout.
pub trait TrackingSource: Send {
    /// Waits for up to `timeout` and forwards whatever arrived.
    fn poll(&mut self, timeout: Duration, emit: EventCallback<'_>) -> Result<(), SourceError>;

    /// Called about once per second to report slowly changing state.
    fn refresh(&mut self, emit: EventCallback<'_>);

    /// Makes a device the one whose data is emitted, and applies the settings to it.
    fn activate(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError>;

    /// Applies new settings to the active device.
    fn apply(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError>;
}
