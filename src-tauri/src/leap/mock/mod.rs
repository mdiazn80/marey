//! Simulated device for development without hardware, enabled only by `MAREY_MOCK=1`.
//!
//! Everything it produces is reported through the link status as simulated, and the UI shows it.

mod hands;
mod images;

use std::time::{Duration, Instant};

use self::hands::simulated_hands;
use self::images::{draw_view, IMAGE_HEIGHT, IMAGE_WIDTH};
use super::mapping::{self, policy, status};
use super::{EventCallback, SourceError, SourceEvent, StereoImage, TrackingSource};
use crate::models::{
    DeviceSummary, LinkState, LogEntry, LogSeverity, LogSource, Settings, TrackingFrame,
};

const FRAME_INTERVAL: Duration = Duration::from_micros(11_111);
const DEVICE_ID: u32 = 1;

/// Environment variable that enables the simulation.
pub const MOCK_VAR: &str = "MAREY_MOCK";

/// Whether the given variable value asks for simulation.
pub fn is_enabled(value: Option<&str>) -> bool {
    matches!(value, Some("1"))
}

pub struct MockSource {
    start: Instant,
    last_frame: Instant,
    frame_id: i64,
    announced: bool,
    settings: Option<Settings>,
    left: Vec<u8>,
    right: Vec<u8>,
    pending_policy: Option<u32>,
    pending_mode: Option<u32>,
    last_log: Instant,
}

impl MockSource {
    pub fn new() -> Self {
        let now = Instant::now();
        let size = (IMAGE_WIDTH * IMAGE_HEIGHT) as usize;
        Self {
            start: now,
            last_frame: now,
            frame_id: 0,
            announced: false,
            settings: None,
            left: vec![0; size],
            right: vec![0; size],
            pending_policy: None,
            pending_mode: None,
            last_log: now,
        }
    }

    fn device() -> DeviceSummary {
        DeviceSummary {
            id: DEVICE_ID,
            product: "Simulated Controller".to_owned(),
            pid: 0,
            serial: "MOCK-0001".to_owned(),
            firmware: None,
            raw_status: status::STREAMING,
            flags: mapping::device_flags(status::STREAMING),
            baseline_mm: 40.0,
            h_fov_deg: 140.0,
            v_fov_deg: 120.0,
            range_mm: 600.0,
        }
    }

    fn record(&mut self, settings: &Settings) {
        let mut raw = policy::ALLOW_PAUSE_RESUME;
        if settings.images_enabled {
            raw |= policy::IMAGES;
        }
        if settings.background_frames {
            raw |= policy::BACKGROUND_FRAMES;
        }
        self.pending_policy = Some(raw);
        self.pending_mode = mapping::tracking_mode_value(settings.tracking_mode);
        self.settings = Some(*settings);
    }
}

impl Default for MockSource {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingSource for MockSource {
    fn poll(&mut self, timeout: Duration, emit: EventCallback<'_>) -> Result<(), SourceError> {
        if !self.announced {
            self.announced = true;
            emit(SourceEvent::Link {
                state: LinkState::Connected,
                client_version: Some("simulated".to_owned()),
                service_version: Some("simulated".to_owned()),
            });
            emit(SourceEvent::DeviceAdded(Self::device()));
            return Ok(());
        }
        if let Some(raw) = self.pending_policy.take() {
            emit(SourceEvent::Policy(raw));
        }
        if let Some(raw) = self.pending_mode.take() {
            emit(SourceEvent::TrackingMode(raw));
        }

        let due = self.last_frame + FRAME_INTERVAL;
        let wait = due.saturating_duration_since(Instant::now());
        if wait > timeout {
            std::thread::sleep(timeout);
            return Ok(());
        }
        std::thread::sleep(wait);
        // Keep to the schedule despite sleep overshoot, without bursts after a long stall.
        let now = Instant::now();
        self.last_frame = if now.duration_since(due) < FRAME_INTERVAL {
            due
        } else {
            now
        };

        let Some(settings) = self.settings else {
            return Ok(());
        };
        if settings.paused {
            return Ok(());
        }

        self.frame_id += 1;
        let t = self.start.elapsed().as_secs_f32();
        let hands = simulated_hands(t);
        if settings.images_enabled {
            draw_view(&mut self.left, &hands, -20.0, t);
            draw_view(&mut self.right, &hands, 20.0, t);
        }
        emit(SourceEvent::Frame(TrackingFrame {
            frame_id: self.frame_id,
            framerate: 1.0 / FRAME_INTERVAL.as_secs_f32(),
            hands,
        }));
        if settings.images_enabled {
            emit(SourceEvent::Images(StereoImage {
                frame_id: self.frame_id,
                width: IMAGE_WIDTH,
                height: IMAGE_HEIGHT,
                bytes_per_pixel: 1,
                left: &self.left,
                right: &self.right,
            }));
        }
        Ok(())
    }

    fn refresh(&mut self, emit: EventCallback<'_>) {
        let status = match self.settings {
            Some(settings) if settings.paused => status::PAUSED,
            _ => status::STREAMING,
        };
        emit(SourceEvent::DeviceStatus {
            id: DEVICE_ID,
            status,
        });
        if self.last_log.elapsed() >= Duration::from_secs(10) {
            self.last_log = Instant::now();
            emit(SourceEvent::Log(LogEntry::now(
                LogSeverity::Info,
                LogSource::Leapc,
                "simulated log event".to_owned(),
            )));
        }
    }

    fn activate(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError> {
        self.apply(device_id, settings)
    }

    fn apply(&mut self, device_id: u32, settings: &Settings) -> Result<(), SourceError> {
        if device_id != DEVICE_ID {
            return Err(SourceError::Call(format!(
                "simulated device {device_id} does not exist"
            )));
        }
        self.record(settings);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_exact_value_enables_simulation() {
        assert!(is_enabled(Some("1")));
        for value in [None, Some("0"), Some(""), Some("true")] {
            assert!(!is_enabled(value));
        }
    }

    fn run_polls(source: &mut MockSource, count: usize) -> (usize, usize) {
        let (mut frames, mut images) = (0, 0);
        for _ in 0..count {
            source
                .poll(Duration::from_millis(20), &mut |event| match event {
                    SourceEvent::Frame(_) => frames += 1,
                    SourceEvent::Images(view) => {
                        assert_eq!(view.left.len(), (view.width * view.height) as usize);
                        images += 1;
                    }
                    _ => {}
                })
                .unwrap();
        }
        (frames, images)
    }

    #[test]
    fn emits_nothing_until_activated_then_honours_settings() {
        let mut source = MockSource::new();
        assert_eq!(run_polls(&mut source, 3), (0, 0));

        let settings = Settings::default();
        source.activate(DEVICE_ID, &settings).unwrap();
        let (frames, images) = run_polls(&mut source, 3);
        assert!(frames > 0);
        assert_eq!(images, 0, "images need the images policy");

        source
            .apply(
                DEVICE_ID,
                &Settings {
                    images_enabled: true,
                    ..settings
                },
            )
            .unwrap();
        let (frames, images) = run_polls(&mut source, 3);
        assert_eq!(frames, images);
        assert!(images > 0);

        source
            .apply(
                DEVICE_ID,
                &Settings {
                    paused: true,
                    ..settings
                },
            )
            .unwrap();
        assert_eq!(run_polls(&mut source, 3), (0, 0));
    }
}
