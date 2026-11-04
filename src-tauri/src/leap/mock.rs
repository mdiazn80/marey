//! Simulated device for development without hardware, enabled only by `MAREY_MOCK=1`.
//!
//! Everything it produces is reported through the link status as simulated, and the UI shows it.

use std::f32::consts::{PI, TAU};
use std::time::{Duration, Instant};

use super::mapping::{self, policy, status};
use super::service::now_ms;
use super::{EventCallback, SourceError, SourceEvent, StereoImage, TrackingSource};
use crate::models::{
    Bone, DeviceSummary, Digit, Hand, HandSide, LinkState, LogEntry, LogSeverity, LogSource, Palm,
    Settings, TrackingFrame, TrackingMode, Vec3,
};

const FRAME_INTERVAL: Duration = Duration::from_micros(11_111);
const IMAGE_WIDTH: u32 = 640;
const IMAGE_HEIGHT: u32 = 240;
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
        self.pending_mode = Some(match settings.tracking_mode {
            TrackingMode::Hmd => 1,
            TrackingMode::ScreenTop => 2,
            _ => 0,
        });
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
            emit(SourceEvent::Log(LogEntry {
                timestamp_ms: now_ms(),
                severity: LogSeverity::Info,
                source: LogSource::Leapc,
                message: "simulated log event".to_owned(),
            }));
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

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn mix(a: Vec3, b: Vec3, along: f32, across: f32) -> Vec3 {
    add(scale(a, along), scale(b, across))
}

/// Bone lengths in mm for metacarpal, proximal, intermediate and distal.
const FINGER_LENGTHS: [[f32; 4]; 5] = [
    [0.0, 46.0, 32.0, 25.0],
    [66.0, 40.0, 24.0, 18.0],
    [62.0, 44.0, 28.0, 19.0],
    [58.0, 40.0, 26.0, 18.0],
    [54.0, 32.0, 18.0, 16.0],
];

fn simulated_hand(side: HandSide, t: f32) -> Hand {
    let sign = if side == HandSide::Left { -1.0 } else { 1.0 };
    let phase = if side == HandSide::Left { 0.0 } else { PI };
    let position = [
        sign * 110.0 + 35.0 * (t * 0.7 + phase).sin(),
        210.0 + 45.0 * (t * 0.9 + phase).cos(),
        20.0 * (t * 0.5 + phase).sin(),
    ];
    // Fingers point away from the user, and the palm faces the device.
    let yaw = 0.25 * (t * 0.6 + phase).sin();
    let direction = [yaw.sin(), 0.0, -yaw.cos()];
    let normal = [0.0, -1.0, 0.0];
    // Points from the pinky side toward the thumb side, which differs between hands.
    let inward = scale(cross(direction, normal), sign);
    let curl = 0.5 - 0.5 * (t * 1.3 + phase).cos();

    let wrist = add(position, scale(direction, -55.0));
    let digits = std::array::from_fn(|finger| {
        let spread = finger as f32 - 2.0;
        let lengths = FINGER_LENGTHS[finger];
        let is_thumb = finger == 0;
        let lateral = -spread * 19.0;
        let mut joint = if is_thumb {
            add(wrist, mix(direction, inward, 25.0, 38.0))
        } else {
            add(wrist, scale(inward, lateral * 0.4))
        };
        let knuckle = add(position, mix(direction, inward, 45.0, lateral));
        let bones = std::array::from_fn(|index| {
            let start = joint;
            let end = if index == 0 {
                if is_thumb {
                    start
                } else {
                    knuckle
                }
            } else {
                let bend = curl * 1.35 * index as f32 * if is_thumb { 0.5 } else { 1.0 };
                let forward = if is_thumb {
                    mix(direction, inward, 0.75, 0.65)
                } else {
                    direction
                };
                let segment = mix(forward, normal, bend.cos(), bend.sin());
                add(start, scale(segment, lengths[index]))
            };
            joint = end;
            Bone {
                prev: start,
                next: end,
                width: if is_thumb { 18.0 } else { 15.0 },
            }
        });
        Digit {
            bones,
            extended: curl < 0.5,
        }
    });

    Hand {
        id: if side == HandSide::Left { 1 } else { 2 },
        side,
        confidence: 1.0,
        grab_strength: curl,
        pinch_strength: curl * curl,
        pinch_distance: 75.0 * (1.0 - curl),
        visible_time_us: (t * 1_000_000.0) as u64,
        palm: Palm {
            position,
            normal,
            direction,
            velocity: [0.0; 3],
            width: 85.0,
        },
        digits,
        arm: Bone {
            prev: add(wrist, scale(direction, -250.0)),
            next: wrist,
            width: 55.0,
        },
    }
}

pub(crate) fn simulated_hands(t: f32) -> Vec<Hand> {
    // Hands leave the view briefly, which exercises the empty state in the UI.
    let cycle = (t / 12.0).fract();
    if cycle > 0.92 {
        return Vec::new();
    }
    vec![
        simulated_hand(HandSide::Left, t),
        simulated_hand(HandSide::Right, t),
    ]
}

/// Draws a recognizably synthetic infrared view: a faint striped background and bright joints.
fn draw_view(plane: &mut [u8], hands: &[Hand], eye_offset: f32, t: f32) {
    let (width, height) = (IMAGE_WIDTH as i32, IMAGE_HEIGHT as i32);
    let band = (t * 40.0) as i32;
    for (y, row) in plane.chunks_exact_mut(width as usize).enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            *pixel = 10 + (((x as i32 + y as i32 + band) >> 4) & 1) as u8 * 8;
        }
    }
    let mut point = |p: Vec3, radius: i32, value: u8| {
        let depth = p[1].max(40.0);
        let px = width / 2 + ((p[0] - eye_offset) / depth * 150.0) as i32;
        let py = height / 2 + (p[2] / depth * 150.0) as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let (x, y) = (px + dx, py + dy);
                if dx * dx + dy * dy <= radius * radius
                    && (0..width).contains(&x)
                    && (0..height).contains(&y)
                {
                    plane[(y * width + x) as usize] = value;
                }
            }
        }
    };
    for hand in hands {
        point(hand.palm.position, 16, 170);
        for digit in &hand.digits {
            for bone in &digit.bones {
                point(bone.next, 5, 235);
            }
        }
    }
    // A moving marker shows the stream is live even with no hands in view.
    let angle = (t * 0.8) % TAU;
    point([70.0 * angle.cos(), 150.0, 30.0 * angle.sin()], 3, 120);
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

    #[test]
    fn simulated_hand_is_anatomically_connected() {
        let hands = simulated_hands(1.0);
        assert_eq!(hands.len(), 2);
        assert_eq!(hands[0].side, HandSide::Left);
        for hand in &hands {
            assert_eq!(
                hand.arm.next,
                add(hand.palm.position, scale(hand.palm.direction, -55.0))
            );
            for digit in &hand.digits {
                for pair in digit.bones.windows(2) {
                    assert_eq!(pair[0].next, pair[1].prev);
                }
            }
        }
    }

    #[test]
    fn thumbs_are_on_the_inside_of_each_hand() {
        for t in [0.5, 2.0, 4.0] {
            let hands = simulated_hands(t);
            let thumb_tip = |h: &Hand| h.digits[0].bones[3].next[0] - h.palm.position[0];
            let pinky_tip = |h: &Hand| h.digits[4].bones[3].next[0] - h.palm.position[0];
            assert!(thumb_tip(&hands[0]) > pinky_tip(&hands[0]), "left at {t}");
            assert!(thumb_tip(&hands[1]) < pinky_tip(&hands[1]), "right at {t}");
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
