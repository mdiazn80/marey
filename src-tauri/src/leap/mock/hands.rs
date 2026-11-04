//! Synthetic hands that move and curl over time.

use std::f32::consts::PI;

use crate::models::{Bone, Digit, Hand, HandSide, Palm, Vec3};

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

pub(super) fn simulated_hands(t: f32) -> Vec<Hand> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
