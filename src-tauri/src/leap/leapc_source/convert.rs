//! Conversions from raw LeapC structs to models.

use leapc::{sys, Device};

use crate::leap::{mapping, StereoImage};
use crate::models::{
    Bone, DeviceSummary, Digit, Hand, HandSide, LogEntry, LogSource, Palm, TrackingFrame, Vec3,
};

/// Describes a device that was just opened.
pub(super) fn device_summary(id: u32, device: &Device) -> Result<DeviceSummary, leapc::Error> {
    let info = device.info()?;
    let product = mapping::product_name(info.pid)
        .map(str::to_owned)
        .or_else(|| device.product_name(info.pid))
        .unwrap_or_else(|| format!("Unknown device (PID 0x{:04X})", info.pid));
    Ok(DeviceSummary {
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
    })
}

pub(super) fn map_log(severity: u32, message: String) -> LogEntry {
    LogEntry::now(mapping::log_severity(severity), LogSource::Leapc, message)
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

fn map_hand(h: &sys::LEAP_HAND) -> Hand {
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

pub(super) fn map_frame(event: &sys::LEAP_TRACKING_EVENT) -> TrackingFrame {
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

pub(super) fn map_images(event: &sys::LEAP_IMAGE_EVENT) -> Option<StereoImage<'_>> {
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
