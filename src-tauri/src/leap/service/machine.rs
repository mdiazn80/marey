//! The service state machine, separate from the thread so tests can drive it directly.

use std::sync::mpsc::Sender;
use std::time::Instant;

use super::logs::LogBuffer;
use crate::leap::{mapping, SourceError, SourceEvent, StereoImage, TrackingSource};
use crate::models::{
    DeviceSummary, FrameStats, LinkState, LinkStatus, LogEntry, LogSeverity, LogSource, PanelState,
    PolicyState, Settings, TrackingFrame, TrackingMode,
};

/// Receives what the service produces. Implemented by the streaming layer and by test doubles.
pub trait EventSink: Send + 'static {
    fn state(&mut self, state: &PanelState);
    /// Offers a frame, and returns whether it was forwarded.
    fn frame(&mut self, frame: &TrackingFrame) -> bool;
    /// Offers an image, and returns whether it was forwarded.
    fn images(&mut self, images: &StereoImage<'_>) -> bool;
    fn log(&mut self, entry: &LogEntry);
    fn settings(&mut self, settings: &Settings);
}

type Reply<T> = Sender<Result<T, String>>;

/// A request to the service thread.
pub enum Command {
    GetState(Sender<PanelState>),
    SelectDevice(u32, Reply<()>),
    UpdateSettings(Settings, Reply<()>),
    Shutdown,
}

/// The source, when one exists. Commands are also served while it is still missing.
type AvailableSource<'a> = Option<&'a mut (dyn TrackingSource + 'static)>;

/// Applies source events and commands to the panel state, and publishes the result.
pub struct Core<K: EventSink> {
    state: PanelState,
    sink: K,
    logs: LogBuffer,
    /// Device waiting to be activated once the current poll has returned.
    pending_activation: Option<u32>,
    window_start: Instant,
    window_frames: u32,
    window_images: u32,
    dirty: bool,
    last_poll_error: Option<String>,
}

impl<K: EventSink> Core<K> {
    pub fn new(sink: K, logs: LogBuffer, settings: Settings, mock: bool) -> Self {
        Self {
            state: PanelState {
                link: LinkStatus {
                    state: LinkState::Starting,
                    detail: None,
                    mock,
                    client_version: None,
                    service_version: None,
                },
                devices: Vec::new(),
                active_device_id: None,
                policies: PolicyState::default(),
                tracking_mode: TrackingMode::Unknown,
                settings,
                stats: FrameStats::default(),
            },
            sink,
            logs,
            pending_activation: None,
            window_start: Instant::now(),
            window_frames: 0,
            window_images: 0,
            dirty: true,
            last_poll_error: None,
        }
    }

    pub fn state(&self) -> &PanelState {
        &self.state
    }

    fn app_log(&mut self, severity: LogSeverity, message: String) {
        match severity {
            LogSeverity::Critical => log::error!("{message}"),
            LogSeverity::Warning => log::warn!("{message}"),
            _ => log::info!("{message}"),
        }
        self.push_log(LogEntry::now(severity, LogSource::App, message));
    }

    fn push_log(&mut self, entry: LogEntry) {
        self.sink.log(&entry);
        self.logs.push(entry);
    }

    fn clear_devices(&mut self) {
        self.state.devices.clear();
        self.state.active_device_id = None;
        self.pending_activation = None;
    }

    /// Applies one source event to the state and forwards what the UI needs. High-rate stream
    /// events only touch the counters, so they do not republish the state.
    pub fn handle(&mut self, event: SourceEvent<'_>) {
        match event {
            SourceEvent::Frame(frame) => self.on_frame(&frame),
            SourceEvent::Images(images) => self.on_images(&images),
            SourceEvent::DroppedFrame => self.state.stats.frames_dropped += 1,
            SourceEvent::Log(entry) => self.push_log(entry),
            SourceEvent::Link {
                state,
                client_version,
                service_version,
            } => self.on_link(state, client_version, service_version),
            SourceEvent::ConnectionLost => self.on_connection_lost(),
            SourceEvent::DeviceAdded(device) => self.on_device_added(device),
            SourceEvent::DeviceLost(id) => self.on_device_lost(id),
            SourceEvent::DeviceStatus { id, status } => self.on_device_status(id, status),
            SourceEvent::DeviceFailure { status } => {
                self.app_log(
                    LogSeverity::Critical,
                    format!("device failure 0x{status:08X}"),
                );
                self.dirty = true;
            }
            SourceEvent::Policy(raw) => {
                self.state.policies = mapping::policy_state(raw);
                self.dirty = true;
            }
            SourceEvent::TrackingMode(raw) => {
                self.state.tracking_mode = mapping::tracking_mode(raw);
                self.dirty = true;
            }
        }
    }

    fn on_frame(&mut self, frame: &TrackingFrame) {
        self.window_frames += 1;
        self.state.stats.frames_received += 1;
        self.state.stats.reported_fps = frame.framerate;
        if self.sink.frame(frame) {
            self.state.stats.frames_sent += 1;
        }
    }

    fn on_images(&mut self, images: &StereoImage<'_>) {
        self.window_images += 1;
        self.state.stats.images_received += 1;
        if self.sink.images(images) {
            self.state.stats.images_sent += 1;
        }
    }

    fn on_link(
        &mut self,
        state: LinkState,
        client_version: Option<String>,
        service_version: Option<String>,
    ) {
        let link = &mut self.state.link;
        let changed = link.state != state
            || (client_version.is_some() && link.client_version != client_version)
            || (service_version.is_some() && link.service_version != service_version);
        if !changed {
            return;
        }
        link.state = state;
        link.detail = None;
        if client_version.is_some() {
            link.client_version = client_version;
        }
        if service_version.is_some() {
            link.service_version = service_version;
        }
        if state != LinkState::Connected {
            self.clear_devices();
        }
        self.app_log(LogSeverity::Info, format!("service link is now {state:?}"));
        self.dirty = true;
    }

    fn on_connection_lost(&mut self) {
        self.state.link.state = LinkState::Connecting;
        self.state.link.service_version = None;
        self.clear_devices();
        self.app_log(
            LogSeverity::Warning,
            "connection to the tracking service lost; waiting for it to return".to_owned(),
        );
        self.dirty = true;
    }

    fn on_device_added(&mut self, device: DeviceSummary) {
        let id = device.id;
        self.app_log(
            LogSeverity::Info,
            format!("device connected: {} (id {id})", device.product),
        );
        match self.state.devices.iter_mut().find(|d| d.id == id) {
            Some(existing) => *existing = device,
            None => self.state.devices.push(device),
        }
        if self.state.active_device_id.is_none() {
            self.state.active_device_id = Some(id);
            self.pending_activation = Some(id);
        }
        self.dirty = true;
    }

    fn on_device_lost(&mut self, id: u32) {
        self.state.devices.retain(|d| d.id != id);
        self.app_log(LogSeverity::Warning, format!("device {id} disconnected"));
        if self.state.active_device_id == Some(id) {
            let next = self.state.devices.first().map(|d| d.id);
            self.state.active_device_id = next;
            self.pending_activation = next;
        }
        self.dirty = true;
    }

    fn on_device_status(&mut self, id: u32, status: u32) {
        let Some(device) = self.state.devices.iter_mut().find(|d| d.id == id) else {
            return;
        };
        if device.raw_status == status {
            return;
        }
        device.raw_status = status;
        device.flags = mapping::device_flags(status);
        self.app_log(
            LogSeverity::Info,
            format!("device {id} status is now 0x{status:08X}"),
        );
        self.dirty = true;
    }

    /// Runs work that needs the source, which is not reachable while it is polling.
    pub(super) fn run_pending(&mut self, source: &mut dyn TrackingSource) {
        if let Some(id) = self.pending_activation.take() {
            if let Err(error) = source.activate(id, &self.state.settings) {
                self.app_log(
                    LogSeverity::Warning,
                    format!("could not fully activate device {id}: {error}"),
                );
            }
            self.dirty = true;
        }
    }

    /// Updates the rate counters.
    pub fn tick(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.window_start).as_secs_f32();
        if elapsed <= 0.0 {
            return;
        }
        self.state.stats.tracking_fps = self.window_frames as f32 / elapsed;
        self.state.stats.image_fps = self.window_images as f32 / elapsed;
        self.window_frames = 0;
        self.window_images = 0;
        self.window_start = now;
        self.dirty = true;
    }

    /// Publishes the state if anything changed since the last call.
    pub fn flush(&mut self) {
        if self.dirty {
            self.dirty = false;
            self.sink.state(&self.state);
        }
    }

    pub(super) fn set_unavailable(&mut self, reason: String) {
        let link = &mut self.state.link;
        if link.state == LinkState::SdkMissing && link.detail.as_deref() == Some(&reason) {
            return;
        }
        link.state = LinkState::SdkMissing;
        link.detail = Some(reason.clone());
        link.client_version = None;
        link.service_version = None;
        self.clear_devices();
        self.dirty = true;
        self.app_log(LogSeverity::Warning, reason);
    }

    /// Logs a poll failure once, instead of on every poll while it lasts.
    pub(super) fn note_poll_error(&mut self, error: Option<&SourceError>) {
        let message = error.map(ToString::to_string);
        if message != self.last_poll_error {
            if let Some(text) = &message {
                self.app_log(LogSeverity::Warning, format!("poll failed: {text}"));
            }
        }
        self.last_poll_error = message;
    }

    fn select_device(&mut self, id: u32, source: AvailableSource<'_>) -> Result<(), String> {
        if !self.state.devices.iter().any(|d| d.id == id) {
            return Err(format!("device {id} is not connected"));
        }
        let source = source.ok_or_else(|| "the tracking source is not available".to_owned())?;
        source
            .activate(id, &self.state.settings)
            .map_err(|e| e.to_string())?;
        self.state.active_device_id = Some(id);
        self.dirty = true;
        Ok(())
    }

    fn update_settings(
        &mut self,
        settings: Settings,
        source: AvailableSource<'_>,
    ) -> Result<(), String> {
        let settings = settings.sanitized();
        self.state.settings = settings;
        self.sink.settings(&settings);
        self.dirty = true;
        match (source, self.state.active_device_id) {
            (Some(source), Some(id)) => source.apply(id, &settings).map_err(|e| e.to_string()),
            _ => Ok(()),
        }
    }

    /// Returns false when the service must stop.
    pub(super) fn command(&mut self, command: Command, source: AvailableSource<'_>) -> bool {
        match command {
            Command::GetState(reply) => {
                let _ = reply.send(self.state.clone());
            }
            Command::SelectDevice(id, reply) => {
                let _ = reply.send(self.select_device(id, source));
            }
            Command::UpdateSettings(settings, reply) => {
                let _ = reply.send(self.update_settings(settings, source));
            }
            Command::Shutdown => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::Duration;

    use super::*;
    use crate::leap::EventCallback;
    use crate::models::DeviceFlags;

    #[derive(Default)]
    struct RecordingSink {
        states: Vec<PanelState>,
        frames: usize,
        accept_frames: bool,
        logs: Vec<LogEntry>,
    }

    impl EventSink for RecordingSink {
        fn state(&mut self, state: &PanelState) {
            self.states.push(state.clone());
        }
        fn frame(&mut self, _: &TrackingFrame) -> bool {
            self.frames += 1;
            self.accept_frames
        }
        fn images(&mut self, _: &StereoImage<'_>) -> bool {
            false
        }
        fn log(&mut self, entry: &LogEntry) {
            self.logs.push(entry.clone());
        }
        fn settings(&mut self, _: &Settings) {}
    }

    /// A source that plays a script and records what the service asks of it.
    #[derive(Default)]
    struct ScriptedSource {
        script: VecDeque<SourceEvent<'static>>,
        activated: Vec<u32>,
        applied: Vec<Settings>,
    }

    impl TrackingSource for ScriptedSource {
        fn poll(&mut self, _: Duration, emit: EventCallback<'_>) -> Result<(), SourceError> {
            if let Some(event) = self.script.pop_front() {
                emit(event);
            }
            Ok(())
        }
        fn refresh(&mut self, _: EventCallback<'_>) {}
        fn activate(&mut self, id: u32, _: &Settings) -> Result<(), SourceError> {
            self.activated.push(id);
            Ok(())
        }
        fn apply(&mut self, _: u32, settings: &Settings) -> Result<(), SourceError> {
            self.applied.push(*settings);
            Ok(())
        }
    }

    fn device(id: u32) -> DeviceSummary {
        DeviceSummary {
            id,
            product: format!("Device {id}"),
            pid: 3,
            serial: format!("S{id}"),
            firmware: None,
            raw_status: 0,
            flags: DeviceFlags::default(),
            baseline_mm: 40.0,
            h_fov_deg: 140.0,
            v_fov_deg: 120.0,
            range_mm: 800.0,
        }
    }

    fn core() -> Core<RecordingSink> {
        Core::new(
            RecordingSink {
                accept_frames: true,
                ..Default::default()
            },
            LogBuffer::default(),
            Settings::default(),
            false,
        )
    }

    fn connected() -> SourceEvent<'static> {
        SourceEvent::Link {
            state: LinkState::Connected,
            client_version: Some("6.2.0".to_owned()),
            service_version: Some("6.2.0".to_owned()),
        }
    }

    fn frame() -> SourceEvent<'static> {
        SourceEvent::Frame(TrackingFrame {
            frame_id: 1,
            framerate: 90.0,
            hands: Vec::new(),
        })
    }

    #[test]
    fn first_device_becomes_active_and_is_activated() {
        let mut core = core();
        let mut source = ScriptedSource::default();
        core.handle(connected());
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.handle(SourceEvent::DeviceAdded(device(2)));
        core.run_pending(&mut source);
        assert_eq!(core.state().active_device_id, Some(1));
        assert_eq!(core.state().devices.len(), 2);
        assert_eq!(source.activated, vec![1]);
    }

    #[test]
    fn losing_the_active_device_falls_back_to_another() {
        let mut core = core();
        let mut source = ScriptedSource::default();
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.handle(SourceEvent::DeviceAdded(device(2)));
        core.run_pending(&mut source);
        core.handle(SourceEvent::DeviceLost(1));
        core.run_pending(&mut source);
        assert_eq!(core.state().active_device_id, Some(2));
        assert_eq!(source.activated, vec![1, 2]);
        core.handle(SourceEvent::DeviceLost(2));
        assert_eq!(core.state().active_device_id, None);
    }

    #[test]
    fn replugged_device_is_activated_again() {
        let mut core = core();
        let mut source = ScriptedSource::default();
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.run_pending(&mut source);
        core.handle(SourceEvent::DeviceLost(1));
        core.run_pending(&mut source);
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.run_pending(&mut source);
        assert_eq!(source.activated, vec![1, 1]);
    }

    #[test]
    fn connection_loss_clears_devices_and_reconnect_restores_them() {
        let mut core = core();
        let mut source = ScriptedSource::default();
        core.handle(connected());
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.run_pending(&mut source);
        core.handle(SourceEvent::ConnectionLost);
        assert_eq!(core.state().link.state, LinkState::Connecting);
        assert!(core.state().devices.is_empty());
        assert_eq!(core.state().active_device_id, None);
        core.handle(connected());
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.run_pending(&mut source);
        assert_eq!(core.state().link.state, LinkState::Connected);
        assert_eq!(source.activated, vec![1, 1]);
    }

    #[test]
    fn service_stopping_clears_devices() {
        let mut core = core();
        core.handle(connected());
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.handle(SourceEvent::Link {
            state: LinkState::ServiceNotRunning,
            client_version: None,
            service_version: None,
        });
        assert!(core.state().devices.is_empty());
    }

    #[test]
    fn device_status_updates_flags() {
        let mut core = core();
        core.handle(SourceEvent::DeviceAdded(device(1)));
        core.handle(SourceEvent::DeviceStatus {
            id: 1,
            status: mapping::status::STREAMING,
        });
        assert!(core.state().devices[0].flags.streaming);
        // Unknown devices are ignored.
        core.handle(SourceEvent::DeviceStatus { id: 9, status: 1 });
        assert_eq!(core.state().devices.len(), 1);
    }

    #[test]
    fn frames_count_only_when_forwarded_as_sent() {
        let mut core = core();
        core.handle(frame());
        core.sink.accept_frames = false;
        core.handle(frame());
        let stats = core.state().stats;
        assert_eq!(stats.frames_received, 2);
        assert_eq!(stats.frames_sent, 1);
        assert_eq!(stats.reported_fps, 90.0);
        assert_eq!(core.sink.frames, 2);
    }

    #[test]
    fn frames_do_not_republish_state() {
        let mut core = core();
        core.flush();
        let published = core.sink.states.len();
        core.handle(frame());
        core.flush();
        assert_eq!(core.sink.states.len(), published);
    }

    #[test]
    fn tick_computes_rates() {
        let mut core = core();
        let start = core.window_start;
        for _ in 0..30 {
            core.handle(frame());
        }
        core.tick(start + Duration::from_millis(500));
        assert_eq!(core.state().stats.tracking_fps, 60.0);
        core.tick(start + Duration::from_millis(1500));
        assert_eq!(core.state().stats.tracking_fps, 0.0);
    }

    #[test]
    fn selecting_unknown_device_fails_without_touching_the_source() {
        let mut core = core();
        let mut source: Box<dyn TrackingSource> = Box::new(ScriptedSource::default());
        assert!(core.select_device(5, Some(source.as_mut())).is_err());
    }

    #[test]
    fn settings_are_sanitized_and_kept_without_a_source() {
        let mut core = core();
        let settings = Settings {
            frame_rate_hz: 100_000,
            ..Settings::default()
        };
        assert!(core.update_settings(settings, None).is_ok());
        assert_eq!(core.state().settings.frame_rate_hz, 240);
    }

    #[test]
    fn missing_sdk_is_reported_once() {
        let mut core = core();
        core.set_unavailable("not installed".to_owned());
        core.set_unavailable("not installed".to_owned());
        assert_eq!(core.state().link.state, LinkState::SdkMissing);
        assert_eq!(core.state().link.detail.as_deref(), Some("not installed"));
        assert_eq!(core.sink.logs.len(), 1);
    }
}
