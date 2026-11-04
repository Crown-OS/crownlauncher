//! The reveal, stepped on the UI thread so the compositor's blur fades with the glass.

use std::time::Duration;

use crownui::prelude::{Cx, Spring};
use motion::{Animation, Curve};

use super::state::Launcher;

fn spring() -> Spring {
    Spring::new(0.32, 0.88)
}

#[derive(Debug, Clone, Copy)]
pub struct Motion {
    animation: Animation<f32>,
    aim: Option<f32>,
    scheduled: bool,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            animation: Animation::new(0.0, 0.0, 0.0, Duration::ZERO, Curve::Spring(spring())),
            aim: None,
            scheduled: false,
        }
    }
}

impl Motion {
    fn target(&self) -> f32 {
        self.aim.unwrap_or_else(|| self.animation.target())
    }

    fn step(&mut self, now: Duration) -> (bool, f32) {
        if let Some(target) = self.aim.take() {
            self.animation = self
                .animation
                .retarget(now, target, Curve::Spring(spring()));
        }
        let sample = self.animation.sample(now);
        (!sample.finished, sample.value)
    }
}

/// Springs the reveal towards `target`, 0 hidden and 1 shown.
pub fn reveal(cx: &mut Cx, launcher: Launcher, target: f32) {
    let schedule = launcher.motion.update(cx, |motion| {
        if motion.target() != target {
            motion.aim = Some(target);
        }
        !std::mem::replace(&mut motion.scheduled, true)
    });
    if schedule {
        cx.request_animation_frame(move |cx, now| frame(cx, launcher, now));
    }
}

fn frame(cx: &mut Cx, launcher: Launcher, now: Duration) {
    let (moving, value) = launcher.motion.update(cx, |motion| {
        motion.scheduled = false;
        motion.step(now)
    });
    launcher.reveal.set(cx, value);
    if moving {
        let target = launcher.motion.with(cx, Motion::target);
        reveal(cx, launcher, target);
    }
}
