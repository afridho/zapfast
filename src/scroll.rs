//! Linux scroll response: direct touchpad input and short eased wheel steps.
//! This is a local feel adjustment, not an emulation of Firefox's physics.

use egui::{Context, MouseWheelUnit, TouchPhase, Vec2};

const TOUCHPAD_GAIN: f32 = 1.3;
const WHEEL_GAIN: f32 = 3.0;

#[derive(Default)]
pub(crate) struct Scroll {
    pending: Vec2,
    trackpad: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_is_eased_and_preserves_distance() {
        let mut scroll = Scroll::default();
        let first = scroll.step((
            Vec2::new(0.0, 40.0),
            0.0,
            0.016,
            false,
            true,
            false,
            false,
            false,
        ));
        assert!(first.y > 0.0 && first.y < 120.0);
        let mut total = first.y;
        for frame in 1..100 {
            total += scroll
                .step((
                    Vec2::ZERO,
                    frame as f64 * 0.016,
                    0.016,
                    false,
                    false,
                    false,
                    false,
                    false,
                ))
                .y;
        }
        assert!((total - 120.0).abs() < 0.01);
        assert!(!scroll.moving());
    }

    #[test]
    fn touchpad_stops_when_input_stops() {
        let mut scroll = Scroll::default();
        let raw = Vec2::new(0.0, 12.0);
        assert_eq!(
            scroll.step((raw, 0.0, 0.016, true, false, true, false, false)),
            raw * TOUCHPAD_GAIN
        );
        scroll.step((Vec2::ZERO, 0.016, 0.016, true, false, false, true, false));
        let first = scroll.step((Vec2::ZERO, 0.032, 0.016, false, false, false, false, false));
        let next = scroll.step((Vec2::ZERO, 0.048, 0.016, false, false, false, false, false));
        assert_eq!(first, Vec2::ZERO);
        assert_eq!(next, Vec2::ZERO);
        assert_eq!(
            scroll.step((Vec2::ZERO, 0.064, 0.016, false, false, false, false, true)),
            Vec2::ZERO
        );
        assert!(!scroll.moving());
    }
}

impl Scroll {
    pub(crate) fn moving(&self) -> bool {
        self.pending.length() > 0.1
    }

    pub(crate) fn from_trackpad(&self) -> bool {
        self.trackpad
    }

    pub(crate) fn apply(&mut self, ctx: &Context) {
        let (raw, now, dt, touch, wheel, began, ended, cancel, pressed) = ctx.input(|i| {
            let mut touch = false;
            let mut wheel = false;
            let mut began = false;
            let mut ended = false;
            let mut cancel = false;
            let mut raw = Vec2::ZERO;
            for event in &i.events {
                if let egui::Event::MouseWheel {
                    unit,
                    phase,
                    delta,
                    modifiers,
                } = event
                {
                    let mut delta = *delta
                        * if *unit == MouseWheelUnit::Line {
                            40.0
                        } else {
                            1.0
                        };
                    if modifiers.shift {
                        delta = Vec2::new(delta.x + delta.y, 0.0);
                    }
                    if !modifiers.ctrl && !modifiers.command {
                        raw += delta;
                    }
                    touch |= *unit == MouseWheelUnit::Point;
                    wheel |= *unit != MouseWheelUnit::Point;
                    began |= *phase == TouchPhase::Start;
                    ended |= *phase == TouchPhase::End;
                    cancel |= *phase == TouchPhase::Cancel;
                }
            }
            (
                raw,
                i.time,
                i.stable_dt.clamp(0.001, 0.05),
                touch,
                wheel,
                began,
                ended,
                cancel,
                i.pointer.any_pressed(),
            )
        });
        let delta = self.step((raw, now, dt, touch, wheel, began, ended, cancel || pressed));
        ctx.input_mut(|i| i.smooth_scroll_delta = delta);
        if self.moving() {
            ctx.request_repaint();
        }
    }

    fn step(&mut self, input: (Vec2, f64, f32, bool, bool, bool, bool, bool)) -> Vec2 {
        let (raw, _now, dt, touch, wheel, began, _ended, cancel) = input;
        if cancel || began {
            self.pending = Vec2::ZERO;
        }
        if cancel {
            return Vec2::ZERO;
        }
        if wheel && raw != Vec2::ZERO {
            self.trackpad = false;
            // Three default egui lines per notch with a short easing tail.
            if self.pending.dot(raw) < 0.0 {
                self.pending = Vec2::ZERO;
            }
            self.pending += raw * WHEEL_GAIN;
        } else if touch {
            self.trackpad = true;
            self.pending = Vec2::ZERO;
            // Do not invent velocity after release: platform point deltas
            // already describe the gesture, including any platform momentum.
            return raw * TOUCHPAD_GAIN;
        }
        let wheel_fraction = 1.0 - (-dt / 0.025).exp();
        let mut delta = self.pending * wheel_fraction;
        self.pending -= delta;
        if self.pending.length() <= 0.1 {
            delta += self.pending;
            self.pending = Vec2::ZERO;
        }
        delta
    }
}
