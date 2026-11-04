//! Synthetic camera images.

use std::f32::consts::TAU;

use crate::models::{Hand, Vec3};

pub(super) const IMAGE_WIDTH: u32 = 640;
pub(super) const IMAGE_HEIGHT: u32 = 240;

/// Draws a recognizably synthetic infrared view: a faint striped background and bright joints.
pub(super) fn draw_view(plane: &mut [u8], hands: &[Hand], eye_offset: f32, t: f32) {
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
