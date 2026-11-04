//! Thin safe wrapper over LeapC.
//!
//! [`Library::load`] resolves the installed SDK at runtime, so a machine without it can still run
//! the application and report the problem. A [`Connection`] must be used from one thread only.

use std::ffi::CStr;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Arc;

pub use leapc_sys as sys;
use sys::LeapCLibrary;

/// Default SDK library directory for this platform.
#[cfg(target_os = "macos")]
pub const DEFAULT_LIB_DIR: &str = "/Applications/Ultraleap Hand Tracking.app/Contents/LeapSDK/lib";
#[cfg(target_os = "windows")]
pub const DEFAULT_LIB_DIR: &str = r"C:\Program Files\Ultraleap\LeapSDK\lib\x64";
#[cfg(target_os = "linux")]
pub const DEFAULT_LIB_DIR: &str = "/usr/lib/ultraleap-hand-tracking-service";

#[cfg(target_os = "macos")]
const LIB_NAME: &str = "libLeapC.dylib";
#[cfg(target_os = "windows")]
const LIB_NAME: &str = "LeapC.dll";
#[cfg(target_os = "linux")]
const LIB_NAME: &str = "libLeapC.so";

/// Failure reported by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LeapC library not found or not loadable at {path}: {reason}")]
    LibraryNotFound { path: PathBuf, reason: String },
    #[error("LeapC call {call} failed: {}", result_name(*.code))]
    Call { call: &'static str, code: u32 },
    #[error("this LeapC version does not provide {0}")]
    MissingSymbol(&'static str),
}

/// Human-readable name of an `eLeapRS` code.
pub fn result_name(code: u32) -> String {
    let name = match code {
        sys::eLeapRS_Success => "Success",
        sys::eLeapRS_UnknownError => "UnknownError",
        sys::eLeapRS_InvalidArgument => "InvalidArgument",
        sys::eLeapRS_InsufficientResources => "InsufficientResources",
        sys::eLeapRS_InsufficientBuffer => "InsufficientBuffer",
        sys::eLeapRS_Timeout => "Timeout",
        sys::eLeapRS_NotConnected => "NotConnected",
        sys::eLeapRS_HandshakeIncomplete => "HandshakeIncomplete",
        sys::eLeapRS_BufferSizeOverflow => "BufferSizeOverflow",
        sys::eLeapRS_ProtocolError => "ProtocolError",
        sys::eLeapRS_InvalidClientID => "InvalidClientID",
        sys::eLeapRS_UnexpectedClosed => "UnexpectedClosed",
        sys::eLeapRS_UnknownImageFrameRequest => "UnknownImageFrameRequest",
        sys::eLeapRS_UnknownTrackingFrameID => "UnknownTrackingFrameID",
        sys::eLeapRS_RoutineIsNotSeer => "RoutineIsNotSeer",
        sys::eLeapRS_TimestampTooEarly => "TimestampTooEarly",
        sys::eLeapRS_ConcurrentPoll => "ConcurrentPoll",
        sys::eLeapRS_NotAvailable => "NotAvailable",
        sys::eLeapRS_NotStreaming => "NotStreaming",
        sys::eLeapRS_CannotOpenDevice => "CannotOpenDevice",
        sys::eLeapRS_Unsupported => "Unsupported",
        _ => return format!("0x{code:08X}"),
    };
    name.to_owned()
}

macro_rules! symbol {
    ($lib:expr, $name:ident) => {
        $lib.$name
            .as_ref()
            .map_err(|_| Error::MissingSymbol(stringify!($name)))?
    };
}

fn check(call: &'static str, code: u32) -> Result<(), Error> {
    if code == sys::eLeapRS_Success {
        Ok(())
    } else {
        Err(Error::Call { call, code })
    }
}

/// The loaded LeapC library.
pub struct Library {
    raw: LeapCLibrary,
    path: PathBuf,
}

impl Library {
    /// Resolves the SDK directory: an explicit override (`LEAPSDK_LIB_PATH`), then the default.
    pub fn resolve_path(override_dir: Option<&Path>) -> PathBuf {
        override_dir
            .unwrap_or_else(|| Path::new(DEFAULT_LIB_DIR))
            .join(LIB_NAME)
    }

    /// Loads LeapC from the given file.
    pub fn load(path: &Path) -> Result<Arc<Self>, Error> {
        // SAFETY: loading the vendor library runs its initializers. This is the documented way to
        // use it, and the path is either the vendor install location or a user override.
        let raw = unsafe { LeapCLibrary::new(path) }.map_err(|e| Error::LibraryNotFound {
            path: path.to_owned(),
            reason: e.to_string(),
        })?;
        Ok(Arc::new(Self {
            raw,
            path: path.to_owned(),
        }))
    }

    /// The file this library was loaded from.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Version triple reported by `LeapGetVersion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

/// Device details from `LeapGetDeviceInfo`.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub status: u32,
    pub caps: u32,
    pub pid: u32,
    pub baseline_um: u32,
    pub serial: String,
    pub h_fov_rad: f32,
    pub v_fov_rad: f32,
    pub range_um: u32,
}

/// A LeapC event, valid until the next poll on the same connection.
pub enum Event<'a> {
    Connection,
    ConnectionLost,
    Device {
        id: u32,
        status: u32,
        handle: sys::LEAP_DEVICE_REF,
    },
    DeviceLost {
        id: u32,
    },
    DeviceFailure {
        status: u32,
    },
    DeviceStatusChange {
        id: u32,
        status: u32,
    },
    Policy {
        flags: u32,
    },
    TrackingMode {
        mode: u32,
    },
    Tracking(&'a sys::LEAP_TRACKING_EVENT),
    Image(&'a sys::LEAP_IMAGE_EVENT),
    Log {
        severity: u32,
        timestamp: i64,
        message: String,
    },
    Logs(Vec<(u32, i64, String)>),
    DroppedFrame,
    Other(u32),
}

/// A connection to the tracking service, with a poll loop owned by one thread.
pub struct Connection {
    lib: Arc<Library>,
    handle: sys::LEAP_CONNECTION,
}

// SAFETY: the handle has no thread affinity. `Connection` is not `Sync`, so it is used from one
// thread at a time, which is what LeapC requires of `LeapPollConnection`.
unsafe impl Send for Connection {}

impl Connection {
    /// Creates and opens a multi-device-aware connection.
    pub fn open(lib: Arc<Library>) -> Result<Self, Error> {
        let mut config: sys::LEAP_CONNECTION_CONFIG = unsafe { zeroed() };
        config.size = size_of::<sys::LEAP_CONNECTION_CONFIG>() as u32;
        config.flags = sys::eLeapConnectionConfig_MultiDeviceAware;
        let mut handle: sys::LEAP_CONNECTION = ptr::null_mut();
        let create = symbol!(lib.raw, LeapCreateConnection);
        // SAFETY: both pointers refer to valid local values for the duration of the call.
        check("LeapCreateConnection", unsafe {
            create(&config, &mut handle)
        })?;
        let connection = Self { lib, handle };
        let open = symbol!(connection.lib.raw, LeapOpenConnection);
        // SAFETY: the handle was created above and is not closed yet.
        check("LeapOpenConnection", unsafe { open(handle) })?;
        Ok(connection)
    }

    /// Current state of the link to the service, as an `eLeapConnectionStatus` value.
    pub fn status(&self) -> Result<u32, Error> {
        let mut info: sys::LEAP_CONNECTION_INFO = unsafe { zeroed() };
        info.size = size_of::<sys::LEAP_CONNECTION_INFO>() as u32;
        let get = symbol!(self.lib.raw, LeapGetConnectionInfo);
        // SAFETY: the handle is valid and `info` outlives the call.
        check("LeapGetConnectionInfo", unsafe {
            get(self.handle, &mut info)
        })?;
        Ok(info.status)
    }

    /// Polls one event with the id of the device that sent it, or 0 for system-wide messages.
    /// Returns `Ok(None)` on timeout.
    pub fn poll(&mut self, timeout_ms: u32) -> Result<Option<(u32, Event<'_>)>, Error> {
        let mut msg: sys::LEAP_CONNECTION_MESSAGE = unsafe { zeroed() };
        msg.size = size_of::<sys::LEAP_CONNECTION_MESSAGE>() as u32;
        let poll = symbol!(self.lib.raw, LeapPollConnection);
        // SAFETY: the handle is valid and `&mut self` guarantees a single concurrent poller.
        let code = unsafe { poll(self.handle, timeout_ms, &mut msg) };
        if code == sys::eLeapRS_Timeout {
            return Ok(None);
        }
        check("LeapPollConnection", code)?;
        // SAFETY: LeapC set the union member matching `msg.type_`, and it stays valid until the
        // next poll, which the `&mut self` borrow on the returned event prevents.
        Ok(Some((msg.device_id, unsafe { decode(&msg) })))
    }

    pub fn version(&self, part: u32) -> Result<Version, Error> {
        let mut v = sys::LEAP_VERSION {
            major: 0,
            minor: 0,
            patch: 0,
        };
        let get = symbol!(self.lib.raw, LeapGetVersion);
        // SAFETY: the handle is valid and `v` outlives the call.
        check("LeapGetVersion", unsafe { get(self.handle, part, &mut v) })?;
        Ok(Version {
            major: v.major,
            minor: v.minor,
            patch: v.patch,
        })
    }

    pub fn open_device(&self, reference: sys::LEAP_DEVICE_REF) -> Result<Device, Error> {
        let mut handle: sys::LEAP_DEVICE = ptr::null_mut();
        let open = symbol!(self.lib.raw, LeapOpenDevice);
        // SAFETY: the reference came from a device event and `handle` outlives the call.
        check("LeapOpenDevice", unsafe { open(reference, &mut handle) })?;
        Ok(Device {
            lib: self.lib.clone(),
            handle,
        })
    }

    pub fn subscribe(&self, device: &Device) -> Result<(), Error> {
        let f = symbol!(self.lib.raw, LeapSubscribeEvents);
        // SAFETY: both handles are valid.
        check("LeapSubscribeEvents", unsafe {
            f(self.handle, device.handle)
        })
    }

    pub fn unsubscribe(&self, device: &Device) -> Result<(), Error> {
        let f = symbol!(self.lib.raw, LeapUnsubscribeEvents);
        // SAFETY: both handles are valid.
        check("LeapUnsubscribeEvents", unsafe {
            f(self.handle, device.handle)
        })
    }

    pub fn set_policy_flags(&self, device: &Device, set: u64, clear: u64) -> Result<(), Error> {
        let f = symbol!(self.lib.raw, LeapSetPolicyFlagsEx);
        // SAFETY: both handles are valid.
        check("LeapSetPolicyFlagsEx", unsafe {
            f(self.handle, device.handle, set, clear)
        })
    }

    pub fn set_tracking_mode(&self, device: &Device, mode: u32) -> Result<(), Error> {
        let f = symbol!(self.lib.raw, LeapSetTrackingModeEx);
        // SAFETY: both handles are valid.
        check("LeapSetTrackingModeEx", unsafe {
            f(self.handle, device.handle, mode)
        })
    }

    pub fn set_pause(&self, pause: bool) -> Result<(), Error> {
        let f = symbol!(self.lib.raw, LeapSetPause);
        // SAFETY: the handle is valid.
        check("LeapSetPause", unsafe { f(self.handle, pause) })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // SAFETY: the handle is valid and never used again. Close stops the service link first.
        unsafe {
            if let Ok(close) = self.lib.raw.LeapCloseConnection.as_ref() {
                close(self.handle);
            }
            if let Ok(destroy) = self.lib.raw.LeapDestroyConnection.as_ref() {
                destroy(self.handle);
            }
        }
    }
}

/// An open device handle.
pub struct Device {
    lib: Arc<Library>,
    handle: sys::LEAP_DEVICE,
}

// SAFETY: the handle has no thread affinity and is owned by the connection thread.
unsafe impl Send for Device {}

impl Device {
    pub fn info(&self) -> Result<DeviceInfo, Error> {
        let mut serial = [0 as std::os::raw::c_char; 256];
        let mut info: sys::LEAP_DEVICE_INFO = unsafe { zeroed() };
        info.size = size_of::<sys::LEAP_DEVICE_INFO>() as u32;
        info.serial = serial.as_mut_ptr();
        info.serial_length = serial.len() as u32;
        let get = symbol!(self.lib.raw, LeapGetDeviceInfo);
        // SAFETY: the handle is valid; `info` and its serial buffer outlive the call.
        check("LeapGetDeviceInfo", unsafe { get(self.handle, &mut info) })?;
        // SAFETY: the buffer was zero-initialized, so it stays NUL terminated.
        let serial = unsafe { CStr::from_ptr(serial.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        Ok(DeviceInfo {
            status: info.status,
            caps: info.caps,
            pid: info.pid,
            baseline_um: info.baseline,
            serial,
            h_fov_rad: info.h_fov,
            v_fov_rad: info.v_fov,
            range_um: info.range,
        })
    }

    /// Product name for this PID as reported by the library.
    pub fn product_name(&self, pid: u32) -> Option<String> {
        let f = self.lib.raw.LeapDevicePIDToString.as_ref().ok()?;
        // SAFETY: the function returns a static string or null for any PID value.
        let name = unsafe { f(pid) };
        if name.is_null() {
            return None;
        }
        // SAFETY: checked non-null above, and the library returns a NUL terminated string.
        Some(
            unsafe { CStr::from_ptr(name) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if let Ok(close) = self.lib.raw.LeapCloseDevice.as_ref() {
            // SAFETY: the handle is valid and never used again.
            unsafe { close(self.handle) };
        }
    }
}

/// # Safety
/// `msg` must have been filled in by a successful `LeapPollConnection` on a live connection.
unsafe fn decode<'a>(msg: &sys::LEAP_CONNECTION_MESSAGE) -> Event<'a> {
    let payload = msg.__bindgen_anon_1;
    match msg.type_ {
        sys::eLeapEventType_Connection => Event::Connection,
        sys::eLeapEventType_ConnectionLost => Event::ConnectionLost,
        sys::eLeapEventType_Device => {
            let event = &*payload.device_event;
            Event::Device {
                id: event.device.id,
                status: event.status,
                handle: event.device,
            }
        }
        sys::eLeapEventType_DeviceLost => {
            let event = &*payload.device_event;
            Event::DeviceLost {
                id: event.device.id,
            }
        }
        sys::eLeapEventType_DeviceFailure => Event::DeviceFailure {
            status: (*payload.device_failure_event).status,
        },
        sys::eLeapEventType_DeviceStatusChange => {
            let event = &*payload.device_status_change_event;
            Event::DeviceStatusChange {
                id: event.device.id,
                status: event.status,
            }
        }
        sys::eLeapEventType_Policy => Event::Policy {
            flags: (*payload.policy_event).current_policy,
        },
        sys::eLeapEventType_TrackingMode => Event::TrackingMode {
            mode: (*payload.tracking_mode_event).current_tracking_mode,
        },
        sys::eLeapEventType_Tracking => Event::Tracking(&*payload.tracking_event),
        sys::eLeapEventType_Image => Event::Image(&*payload.image_event),
        sys::eLeapEventType_LogEvent => {
            let event = &*payload.log_event;
            Event::Log {
                severity: event.severity,
                timestamp: event.timestamp,
                message: message_text(event.message),
            }
        }
        sys::eLeapEventType_LogEvents => {
            let events = &*payload.log_events;
            let count = events.nEvents as usize;
            let list = if count == 0 || events.events.is_null() {
                &[][..]
            } else {
                std::slice::from_raw_parts(events.events, count)
            };
            Event::Logs(
                list.iter()
                    .map(|e| {
                        let (severity, timestamp, message) = (e.severity, e.timestamp, e.message);
                        (severity, timestamp, message_text(message))
                    })
                    .collect(),
            )
        }
        sys::eLeapEventType_DroppedFrame => Event::DroppedFrame,
        other => Event::Other(other),
    }
}

/// # Safety
/// `message` must be null or a NUL terminated string.
unsafe fn message_text(message: *const std::os::raw::c_char) -> String {
    if message.is_null() {
        String::new()
    } else {
        CStr::from_ptr(message).to_string_lossy().into_owned()
    }
}

/// Copies one camera image out of an image event, using the offset LeapC reports.
///
/// # Safety
/// `image` must come from an image event that is still valid, which holds while the [`Event`]
/// borrow is alive.
pub unsafe fn image_bytes(image: &sys::LEAP_IMAGE) -> Option<&[u8]> {
    let props = image.properties;
    let (width, height, bpp, offset) = (
        props.width as usize,
        props.height as usize,
        props.bpp as usize,
        image.offset as usize,
    );
    let data = image.data as *const u8;
    if data.is_null() || width == 0 || height == 0 || bpp == 0 {
        return None;
    }
    Some(std::slice::from_raw_parts(
        data.add(offset),
        width * height * bpp,
    ))
}
