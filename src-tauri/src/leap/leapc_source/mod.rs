//! The real tracking source, backed by the installed LeapC library.

mod convert;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use leapc::{sys, Connection, Device, Event, Library};

use super::mapping::{self, policy};
use super::{EventCallback, SourceError, SourceEvent, TrackingSource};
use crate::models::{LinkState, Settings};

/// Environment variable overriding the SDK library directory.
const LIB_PATH_VAR: &str = "LEAPSDK_LIB_PATH";

/// Loads LeapC from `LEAPSDK_LIB_PATH` or the default location.
fn load_library() -> Result<Arc<Library>, SourceError> {
    let override_dir = std::env::var_os(LIB_PATH_VAR).map(PathBuf::from);
    let path = Library::resolve_path(override_dir.as_deref());
    Library::load(&path).map_err(|e| SourceError::Unavailable(e.to_string()))
}

fn call_error(error: leapc::Error) -> SourceError {
    SourceError::Call(error.to_string())
}

/// Tracking data from the Ultraleap service.
pub struct LeapcSource {
    connection: Connection,
    devices: HashMap<u32, Device>,
    active: Option<u32>,
    paused: bool,
}

impl LeapcSource {
    pub fn open() -> Result<Self, SourceError> {
        let library = load_library()?;
        let connection =
            Connection::open(library).map_err(|e| SourceError::Unavailable(e.to_string()))?;
        Ok(Self {
            connection,
            devices: HashMap::new(),
            active: None,
            paused: false,
        })
    }

    fn versions(&self) -> (Option<String>, Option<String>) {
        let read = |part| {
            self.connection
                .version(part)
                .ok()
                .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
        };
        (
            read(sys::eLeapVersionPart_ClientLibrary),
            read(sys::eLeapVersionPart_ServerLibrary),
        )
    }

    fn link_event(&self, state: LinkState) -> SourceEvent<'static> {
        let (client_version, service_version) = if state == LinkState::Connected {
            self.versions()
        } else {
            (None, None)
        };
        SourceEvent::Link {
            state,
            client_version,
            service_version,
        }
    }

    fn device(&self, id: u32) -> Result<&Device, SourceError> {
        self.devices
            .get(&id)
            .ok_or_else(|| SourceError::Call(format!("device {id} is not open")))
    }

    fn apply_all(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError> {
        let (set, clear) = policy_changes(settings);

        // Attempt every step, so one unsupported option does not block the others.
        let device = self.device(device_id)?;
        let policies = self.connection.set_policy_flags(device, set, clear);
        let mode = match mapping::tracking_mode_value(settings.tracking_mode) {
            Some(value) => self.connection.set_tracking_mode(device, value),
            None => Ok(()),
        };
        let pause = if settings.paused != self.paused {
            self.connection.set_pause(settings.paused)
        } else {
            Ok(())
        };
        if pause.is_ok() {
            self.paused = settings.paused;
        }
        policies.and(mode).and(pause).map_err(call_error)
    }
}

/// The policy flags to set and to clear for these settings.
fn policy_changes(settings: &Settings) -> (u64, u64) {
    let mut set = u64::from(policy::ALLOW_PAUSE_RESUME);
    let mut clear = 0_u64;
    for (flag, on) in [
        (policy::IMAGES, settings.images_enabled),
        (policy::BACKGROUND_FRAMES, settings.background_frames),
    ] {
        if on {
            set |= u64::from(flag);
        } else {
            clear |= u64::from(flag);
        }
    }
    (set, clear)
}

impl TrackingSource for LeapcSource {
    fn poll(&mut self, timeout: Duration, emit: EventCallback<'_>) -> Result<(), SourceError> {
        let timeout_ms = timeout.as_millis() as u32;
        let active = self.active;

        // A device event needs the connection again, after the event borrow has ended.
        let mut opened = None;
        let mut connected = false;
        let polled = self.connection.poll(timeout_ms).map_err(call_error)?;
        let Some((device_id, event)) = polled else {
            return Ok(());
        };
        // Device-specific data from anything but the active device is not shown.
        let from_active = device_id == 0 || Some(device_id) == active;
        match event {
            Event::Connection => connected = true,
            Event::ConnectionLost => emit(SourceEvent::ConnectionLost),
            Event::Device { id, handle, .. } => opened = Some((id, handle)),
            Event::DeviceLost { id } => {
                self.devices.remove(&id);
                if self.active == Some(id) {
                    self.active = None;
                }
                emit(SourceEvent::DeviceLost(id));
            }
            Event::DeviceFailure { status } => emit(SourceEvent::DeviceFailure { status }),
            Event::DeviceStatusChange { id, status } => {
                emit(SourceEvent::DeviceStatus { id, status })
            }
            Event::Policy { flags } if from_active => emit(SourceEvent::Policy(flags)),
            Event::TrackingMode { mode } if from_active => emit(SourceEvent::TrackingMode(mode)),
            Event::Tracking(frame) if from_active => {
                emit(SourceEvent::Frame(convert::map_frame(frame)))
            }
            Event::Image(images) if from_active => {
                if let Some(images) = convert::map_images(images) {
                    emit(SourceEvent::Images(images));
                }
            }
            Event::Log {
                severity,
                timestamp: _,
                message,
            } => emit(SourceEvent::Log(convert::map_log(severity, message))),
            Event::Logs(entries) => {
                for (severity, _, message) in entries {
                    emit(SourceEvent::Log(convert::map_log(severity, message)));
                }
            }
            Event::DroppedFrame => emit(SourceEvent::DroppedFrame),
            _ => {}
        }

        if connected {
            emit(self.link_event(LinkState::Connected));
        }
        if let Some((id, handle)) = opened {
            let device = self.connection.open_device(handle).map_err(call_error)?;
            let summary = convert::device_summary(id, &device).map_err(call_error)?;
            self.devices.insert(id, device);
            emit(SourceEvent::DeviceAdded(summary));
        }
        Ok(())
    }

    fn refresh(&mut self, emit: EventCallback<'_>) {
        if let Ok(raw) = self.connection.status() {
            emit(self.link_event(mapping::link_state(raw)));
        }
        for (id, device) in &self.devices {
            if let Ok(info) = device.info() {
                emit(SourceEvent::DeviceStatus {
                    id: *id,
                    status: info.status,
                });
            }
        }
    }

    fn activate(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError> {
        if let Some(previous) = self.active.filter(|id| *id != device_id) {
            if let Some(device) = self.devices.get(&previous) {
                // The device may already be gone, in which case there is nothing to undo.
                let _ = self.connection.unsubscribe(device);
            }
        }
        let device = self.device(device_id)?;
        self.connection.subscribe(device).map_err(call_error)?;
        self.active = Some(device_id);
        self.apply_all(device_id, settings)
    }

    fn apply(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError> {
        self.apply_all(device_id, settings)
    }
}

impl Drop for LeapcSource {
    fn drop(&mut self) {
        // Pausing may outlive this connection, so do not leave other clients paused.
        if self.paused {
            let _ = self.connection.set_pause(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leap::mapping::{connection, status};
    use crate::models::{LogSeverity, TrackingMode};

    /// The mapping module has its own copies of these values, so it works without the SDK.
    #[test]
    fn mapping_constants_match_the_installed_header() {
        assert_eq!(
            policy::BACKGROUND_FRAMES,
            sys::eLeapPolicyFlag_BackgroundFrames
        );
        assert_eq!(policy::IMAGES, sys::eLeapPolicyFlag_Images);
        assert_eq!(policy::OPTIMIZE_HMD, sys::eLeapPolicyFlag_OptimizeHMD);
        assert_eq!(
            policy::ALLOW_PAUSE_RESUME,
            sys::eLeapPolicyFlag_AllowPauseResume
        );
        assert_eq!(policy::MAP_POINTS, sys::eLeapPolicyFlag_MapPoints);
        assert_eq!(
            policy::OPTIMIZE_SCREEN_TOP,
            sys::eLeapPolicyFlag_OptimizeScreenTop
        );
        assert_eq!(status::STREAMING, sys::eLeapDeviceStatus_Streaming);
        assert_eq!(status::PAUSED, sys::eLeapDeviceStatus_Paused);
        assert_eq!(status::ROBUST, sys::eLeapDeviceStatus_Robust);
        assert_eq!(status::SMUDGED, sys::eLeapDeviceStatus_Smudged);
        assert_eq!(status::LOW_RESOURCE, sys::eLeapDeviceStatus_LowResource);
        assert_eq!(
            status::UNKNOWN_FAILURE,
            sys::eLeapDeviceStatus_UnknownFailure
        );
        assert_eq!(
            status::BAD_CALIBRATION,
            sys::eLeapDeviceStatus_BadCalibration
        );
        assert_eq!(status::BAD_FIRMWARE, sys::eLeapDeviceStatus_BadFirmware);
        assert_eq!(status::BAD_TRANSPORT, sys::eLeapDeviceStatus_BadTransport);
        assert_eq!(status::BAD_CONTROL, sys::eLeapDeviceStatus_BadControl);
        assert_eq!(
            connection::NOT_CONNECTED,
            sys::eLeapConnectionStatus_NotConnected
        );
        assert_eq!(connection::CONNECTED, sys::eLeapConnectionStatus_Connected);
        assert_eq!(
            connection::HANDSHAKE_INCOMPLETE,
            sys::eLeapConnectionStatus_HandshakeIncomplete
        );
        assert_eq!(
            connection::NOT_RUNNING,
            sys::eLeapConnectionStatus_NotRunning
        );
        assert_eq!(
            mapping::tracking_mode(sys::eLeapTrackingMode_Desktop),
            TrackingMode::Desktop
        );
        assert_eq!(
            mapping::tracking_mode(sys::eLeapTrackingMode_HMD),
            TrackingMode::Hmd
        );
        assert_eq!(
            mapping::tracking_mode(sys::eLeapTrackingMode_ScreenTop),
            TrackingMode::ScreenTop
        );
        assert_eq!(
            mapping::log_severity(sys::eLeapLogSeverity_Critical),
            LogSeverity::Critical
        );
        assert_eq!(
            mapping::log_severity(sys::eLeapLogSeverity_Warning),
            LogSeverity::Warning
        );
        assert_eq!(
            mapping::log_severity(sys::eLeapLogSeverity_Information),
            LogSeverity::Info
        );
    }
}
