//! Delivers service output to the frontend: throttled channels for streams, events for the rest.

pub mod throttle;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter};
use throttle::Throttle;

use crate::leap::service::EventSink;
use crate::leap::StereoImage;
use crate::models::{LogEntry, PanelState, Settings, TrackingFrame};

pub const STATE_EVENT: &str = "marey://state";
pub const LOG_EVENT: &str = "marey://log";

/// Bytes before the pixel data in an image message.
pub const IMAGE_HEADER_LEN: usize = 16;

/// The frontend's stream endpoints, replaced whenever the page subscribes again.
#[derive(Clone, Default)]
pub struct StreamChannels(Arc<Mutex<Channels>>);

#[derive(Default)]
struct Channels {
    frames: Option<Channel<InvokeResponseBody>>,
    images: Option<Channel<InvokeResponseBody>>,
}

impl StreamChannels {
    pub fn subscribe(
        &self,
        frames: Channel<InvokeResponseBody>,
        images: Channel<InvokeResponseBody>,
    ) {
        if let Ok(mut channels) = self.0.lock() {
            channels.frames = Some(frames);
            channels.images = Some(images);
        }
    }
}

/// Encodes a stereo image as a little-endian header followed by the left and right planes:
/// width, height, bytes per pixel and the low 32 bits of the frame id, each as a `u32`.
pub fn encode_images(images: &StereoImage<'_>) -> Vec<u8> {
    let mut out = Vec::with_capacity(IMAGE_HEADER_LEN + images.left.len() + images.right.len());
    out.extend_from_slice(&images.width.to_le_bytes());
    out.extend_from_slice(&images.height.to_le_bytes());
    out.extend_from_slice(&images.bytes_per_pixel.to_le_bytes());
    out.extend_from_slice(&(images.frame_id as u32).to_le_bytes());
    out.extend_from_slice(images.left);
    out.extend_from_slice(images.right);
    out
}

/// Sends service output through Tauri.
pub struct TauriSink {
    app: AppHandle,
    channels: StreamChannels,
    frames: Throttle,
    images: Throttle,
}

impl TauriSink {
    pub fn new(app: AppHandle, channels: StreamChannels, settings: &Settings) -> Self {
        Self {
            app,
            channels,
            frames: Throttle::new(settings.frame_rate_hz),
            images: Throttle::new(settings.image_rate_hz),
        }
    }
}

impl EventSink for TauriSink {
    fn state(&mut self, state: &PanelState) {
        if let Err(error) = self.app.emit(STATE_EVENT, state) {
            log::warn!("could not publish the state: {error}");
        }
    }

    fn frame(&mut self, frame: &TrackingFrame) -> bool {
        if !self.frames.allow(Instant::now()) {
            return false;
        }
        let Ok(channels) = self.channels.0.lock() else {
            return false;
        };
        let Some(channel) = &channels.frames else {
            return false;
        };
        // Serializing here avoids cloning the frame just to hand it to the channel.
        match serde_json::to_string(frame) {
            Ok(json) => channel.send(InvokeResponseBody::Json(json)).is_ok(),
            Err(_) => false,
        }
    }

    fn images(&mut self, images: &StereoImage<'_>) -> bool {
        // The throttle comes first, because the copy below is the expensive part.
        if !self.images.allow(Instant::now()) {
            return false;
        }
        let Ok(channels) = self.channels.0.lock() else {
            return false;
        };
        let Some(channel) = &channels.images else {
            return false;
        };
        channel
            .send(InvokeResponseBody::Raw(encode_images(images)))
            .is_ok()
    }

    fn log(&mut self, entry: &LogEntry) {
        let _ = self.app.emit(LOG_EVENT, entry);
    }

    fn settings(&mut self, settings: &Settings) {
        self.frames.set_rate(settings.frame_rate_hz);
        self.images.set_rate(settings.image_rate_hz);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_message_layout_is_stable() {
        let left = [1_u8, 2, 3, 4];
        let right = [5_u8, 6, 7, 8];
        let message = encode_images(&StereoImage {
            frame_id: 0x1_0000_0002,
            width: 2,
            height: 2,
            bytes_per_pixel: 1,
            left: &left,
            right: &right,
        });
        assert_eq!(message.len(), IMAGE_HEADER_LEN + 8);
        assert_eq!(&message[0..4], &2_u32.to_le_bytes());
        assert_eq!(&message[4..8], &2_u32.to_le_bytes());
        assert_eq!(&message[8..12], &1_u32.to_le_bytes());
        assert_eq!(&message[12..16], &2_u32.to_le_bytes());
        assert_eq!(&message[16..], &[1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
