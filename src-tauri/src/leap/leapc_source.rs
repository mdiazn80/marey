//! The real tracking source, backed by the installed LeapC library.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use leapc::{sys, Connection, Device, Event, Library};

use super::mapping::{self, policy};
use super::service::now_ms;
use super::{EventCallback, SourceError, SourceEvent, StereoImage, TrackingSource};
use crate::models::{
    Bone, DeviceSummary, Digit, Hand, HandSide, LinkState, LogEntry, LogSource, Palm, Settings,
    TrackingFrame, Vec3,
};

/// Environment variable overriding the SDK library directory.
pub const LIB_PATH_VAR: &str = "LEAPSDK_LIB_PATH";

/// Loads LeapC from `LEAPSDK_LIB_PATH` or the default location.
pub fn load_library() -> Result<Arc<Library>, SourceError> {
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
        let mut set = u64::from(policy::ALLOW_PAUSE_RESUME);
        let mut clear = 0_u64;
        let mut choose = |flag: u32, on: bool| {
            if on {
                set |= u64::from(flag);
            } else {
                clear |= u64::from(flag);
            }
        };
        choose(policy::IMAGES, settings.images_enabled);
        choose(policy::BACKGROUND_FRAMES, settings.background_frames);

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
            Event::Tracking(frame) if from_active => emit(SourceEvent::Frame(map_frame(frame))),
            Event::Image(images) if from_active => {
                if let Some(images) = map_images(images) {
                    emit(SourceEvent::Images(images));
                }
            }
            Event::Log {
                severity,
                timestamp: _,
                message,
            } => emit(SourceEvent::Log(map_log(severity, message))),
            Event::Logs(entries) => {
                for (severity, _, message) in entries {
                    emit(SourceEvent::Log(map_log(severity, message)));
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
            let info = device.info().map_err(call_error)?;
            let product = mapping::product_name(info.pid)
                .map(str::to_owned)
                .or_else(|| device.product_name(info.pid))
                .unwrap_or_else(|| format!("Unknown device (PID 0x{:04X})", info.pid));
            self.devices.insert(id, device);
            emit(SourceEvent::DeviceAdded(DeviceSummary {
                id,
                product,
                pid: info.pid,
                serial: info.serial,
                firmware: None,
                raw_status: info.status,
                flags: mapping::device_flags(info.status),
                baseline_mm: info.baseline_um as f32 / 1000.0,
                h_fov_deg: info.h_fov_rad.to_degrees(),
                v_fov_deg: info.v_fov_rad.to_degrees(),
                range_mm: info.range_um as f32 / 1000.0,
            }));
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

fn map_log(severity: u32, message: String) -> LogEntry {
    LogEntry {
        timestamp_ms: now_ms(),
        severity: mapping::log_severity(severity),
        source: LogSource::Leapc,
        message,
    }
}

fn vector(v: sys::LEAP_VECTOR) -> Vec3 {
    // SAFETY: both union members are the same three floats, and every bit pattern is valid.
    unsafe { v.__bindgen_anon_1.v }
}

fn bone(b: sys::LEAP_BONE) -> Bone {
    Bone {
        prev: vector(b.prev_joint),
        next: vector(b.next_joint),
        width: b.width,
    }
}

fn digit(d: sys::LEAP_DIGIT) -> Digit {
    // SAFETY: both union members describe the same four bones of plain data.
    let bones = unsafe { d.__bindgen_anon_1.bones };
    Digit {
        bones: bones.map(bone),
        extended: d.is_extended != 0,
    }
}

pub(crate) fn map_hand(h: &sys::LEAP_HAND) -> Hand {
    let palm = h.palm;
    // SAFETY: both union members describe the same five digits of plain data.
    let digits = unsafe { h.__bindgen_anon_1.digits };
    Hand {
        id: h.id,
        side: if h.type_ == sys::eLeapHandType_Left {
            HandSide::Left
        } else {
            HandSide::Right
        },
        confidence: h.confidence,
        grab_strength: h.grab_strength,
        pinch_strength: h.pinch_strength,
        pinch_distance: h.pinch_distance,
        visible_time_us: h.visible_time,
        palm: Palm {
            position: vector(palm.position),
            normal: vector(palm.normal),
            direction: vector(palm.direction),
            velocity: vector(palm.velocity),
            width: palm.width,
        },
        digits: digits.map(digit),
        arm: bone(h.arm),
    }
}

fn map_frame(event: &sys::LEAP_TRACKING_EVENT) -> TrackingFrame {
    let (count, pointer) = (event.nHands as usize, event.pHands);
    let hands = if count == 0 || pointer.is_null() {
        &[][..]
    } else {
        // SAFETY: LeapC points this at `nHands` hands, valid until the next poll, and the struct
        // is packed, so any address is suitably aligned.
        unsafe { std::slice::from_raw_parts(pointer as *const sys::LEAP_HAND, count) }
    };
    TrackingFrame {
        frame_id: event.tracking_frame_id,
        framerate: event.framerate,
        hands: hands.iter().map(map_hand).collect(),
    }
}

fn map_images(event: &sys::LEAP_IMAGE_EVENT) -> Option<StereoImage<'_>> {
    let (left, right) = (event.image[0], event.image[1]);
    let properties = left.properties;
    let info = event.info;
    // SAFETY: the event is borrowed from the connection, so its image data is still valid.
    let (left, right) = unsafe { (image_slice(&left)?, image_slice(&right)?) };
    Some(StereoImage {
        frame_id: info.frame_id,
        width: properties.width,
        height: properties.height,
        bytes_per_pixel: properties.bpp,
        left,
        right,
    })
}

/// # Safety
/// The image data pointer must be valid for the lifetime of the returned slice.
unsafe fn image_slice<'a>(image: &sys::LEAP_IMAGE) -> Option<&'a [u8]> {
    leapc::image_bytes(image).map(|bytes| std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()))
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

    #[test]
    fn hand_fields_map_from_raw_struct() {
        // SAFETY: LEAP_HAND is plain data, for which all zero bytes are valid.
        let mut raw: sys::LEAP_HAND = unsafe { std::mem::zeroed() };
        raw.id = 42;
        raw.type_ = sys::eLeapHandType_Left;
        raw.confidence = 0.5;
        raw.grab_strength = 0.25;
        raw.pinch_distance = 12.0;
        raw.visible_time = 1000;
        raw.palm.width = 80.0;
        raw.palm.position.__bindgen_anon_1.v = [1.0, 2.0, 3.0];
        raw.arm.next_joint.__bindgen_anon_1.v = [4.0, 5.0, 6.0];
        // SAFETY: writes plain data through the union.
        unsafe {
            raw.__bindgen_anon_1.digits[1].is_extended = 1;
            raw.__bindgen_anon_1.digits[1].__bindgen_anon_1.bones[3]
                .next_joint
                .__bindgen_anon_1
                .v = [7.0, 8.0, 9.0];
        }
        let hand = map_hand(&raw);
        assert_eq!(hand.id, 42);
        assert_eq!(hand.side, HandSide::Left);
        assert_eq!(hand.palm.position, [1.0, 2.0, 3.0]);
        assert_eq!(hand.arm.next, [4.0, 5.0, 6.0]);
        assert!(hand.digits[1].extended && !hand.digits[0].extended);
        assert_eq!(hand.digits[1].bones[3].next, [7.0, 8.0, 9.0]);
        assert_eq!(hand.visible_time_us, 1000);
    }
}
