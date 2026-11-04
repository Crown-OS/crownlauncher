//! Opening and closing as two springs, stepped on the UI thread so the compositor's blur follows
//! the glass. `presence` fades the glass and its blur in quickly without overshooting, and
//! `settle` drops the contents into place with a little bounce. Retargeting keeps each
//! spring's velocity, so reversing mid-way swings back instead of jumping.
//!
//! The glass itself never moves: the compositor redraws the blur behind any glass whose shape
//! changes, which it cannot do at the display's rate. Its contents drop in instead, a paint-only
//! move inside it.

use std::time::Duration;

use crownui::prelude::{Cx, Runtime, Spring};
use motion::{Animation, Curve};

use super::state::Launcher;

/// How far above their place the window's contents start.
const ENTRY_DROP: f32 = 12.0;

/// Past this much revealed the window is at full strength, so the fade does not linger on the
/// spring's long tail.
const OPAQUE: f32 = 0.98;

/// How strongly the window is drawn at `reveal`.
pub fn opacity(reveal: f32) -> f32 {
    if reveal > OPAQUE {
        1.0
    } else {
        reveal.clamp(0.0, 1.0)
    }
}

/// The contents' offset from their place: negative above it, a touch below while bouncing.
pub fn drop_offset(runtime: &mut Runtime, launcher: Launcher) -> f32 {
    -ENTRY_DROP * (1.0 - launcher.settle.get(runtime))
}

#[derive(Debug, Clone, Copy)]
struct Springs {
    opening: Spring,
    closing: Spring,
}

impl Springs {
    fn presence() -> Self {
        Self {
            opening: Spring::new(0.14, 1.0),
            closing: Spring::new(0.12, 1.0),
        }
    }

    fn settle() -> Self {
        Self {
            opening: Spring::new(0.5, 0.66),
            closing: Spring::new(0.24, 1.0),
        }
    }

    fn toward(self, target: f32) -> Curve {
        Curve::Spring(if target > 0.0 {
            self.opening
        } else {
            self.closing
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Channel {
    animation: Animation<f32>,
    aim: Option<f32>,
    springs: Springs,
}

impl Channel {
    fn new(springs: Springs) -> Self {
        Self {
            animation: Animation::new(0.0, 0.0, 0.0, Duration::ZERO, springs.toward(0.0)),
            aim: None,
            springs,
        }
    }

    fn target(&self) -> f32 {
        self.aim.unwrap_or_else(|| self.animation.target())
    }

    fn aim(&mut self, target: f32) {
        if self.target() != target {
            self.aim = Some(target);
        }
    }

    /// The value at `now`, and whether it is still moving.
    fn step(&mut self, now: Duration) -> (f32, bool) {
        if let Some(target) = self.aim.take() {
            self.animation = self
                .animation
                .retarget(now, target, self.springs.toward(target));
        }
        let sample = self.animation.sample(now);
        (sample.value, !sample.finished)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Motion {
    presence: Channel,
    settle: Channel,
    scheduled: bool,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            presence: Channel::new(Springs::presence()),
            settle: Channel::new(Springs::settle()),
            scheduled: false,
        }
    }
}

/// Springs the launcher towards shown or hidden.
pub fn reveal(cx: &mut Cx, launcher: Launcher, shown: bool) {
    let target = if shown { 1.0 } else { 0.0 };
    let schedule = launcher.motion.update(cx, |motion| {
        motion.presence.aim(target);
        motion.settle.aim(target);
        !std::mem::replace(&mut motion.scheduled, true)
    });
    if schedule {
        cx.request_animation_frame(move |cx, now| frame(cx, launcher, now));
    }
}

fn frame(cx: &mut Cx, launcher: Launcher, now: Duration) {
    let ((presence, fading), (settle, settling)) = launcher.motion.update(cx, |motion| {
        motion.scheduled = false;
        (motion.presence.step(now), motion.settle.step(now))
    });
    launcher.reveal.set(cx, presence);
    launcher.settle.set(cx, settle);
    if fading || settling {
        launcher.motion.update(cx, |motion| motion.scheduled = true);
        cx.request_animation_frame(move |cx, now| frame(cx, launcher, now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Duration = Duration::from_millis(8);

    /// Every value `channel` passes through opening, and when it came to rest.
    fn open(springs: Springs) -> (Vec<f32>, Duration) {
        let mut channel = Channel::new(springs);
        channel.aim(1.0);
        let mut now = Duration::ZERO;
        let mut values = Vec::new();
        loop {
            let (value, moving) = channel.step(now);
            values.push(value);
            if !moving {
                return (values, now);
            }
            now += FRAME;
        }
    }

    #[test]
    fn the_glass_fades_in_without_overshooting() {
        let (values, rest) = open(Springs::presence());
        assert!(values.iter().all(|value| (0.0..=1.0).contains(value)));
        assert!(rest < Duration::from_millis(500), "{rest:?}");
    }

    #[test]
    fn the_bar_bounces_a_little_past_its_place_and_settles() {
        let (values, rest) = open(Springs::settle());
        let peak = values.iter().copied().fold(0.0, f32::max);
        assert!((1.02..1.1).contains(&peak), "{peak}");
        assert!(rest < Duration::from_millis(1400), "{rest:?}");
    }

    #[test]
    fn reversing_mid_way_keeps_moving_smoothly() {
        let mut channel = Channel::new(Springs::settle());
        channel.aim(1.0);
        let (before, _) = channel.step(Duration::ZERO);
        let (opened, _) = channel.step(Duration::from_millis(120));
        channel.aim(0.0);
        let (reversed, _) = channel.step(Duration::from_millis(120));
        let (after, _) = channel.step(Duration::from_millis(128));
        assert!(before < opened);
        assert_eq!(opened, reversed);
        assert!((after - reversed).abs() < 0.1, "{reversed} -> {after}");
    }
}
