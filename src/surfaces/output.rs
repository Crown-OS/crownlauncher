//! The output the launcher is on. While hidden it follows the focused output, so it is already
//! there when it is shown; while on screen it stays put. Without `crownos_output_info_v1` it stays
//! on the first output it is put on, until that output goes.

use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use crownui::surfaces::{Output, OutputId, Waker};

use crate::util::lock::lock;

#[derive(Clone, Default)]
pub struct LauncherOutput(Arc<Shared>);

#[derive(Default)]
struct Shared {
    held: Mutex<Option<OutputId>>,
    on_screen: AtomicBool,
    runner_waker: OnceLock<Waker>,
}

impl LauncherOutput {
    pub fn attach(&self, runner_waker: Waker) {
        let _ = self.0.runner_waker.set(runner_waker);
    }

    pub fn hosts(&self, output: &Output) -> bool {
        let on_screen = self.0.on_screen.load(Ordering::Relaxed);
        let following_focus = output
            .crown
            .as_ref()
            .map(|crown| crown.focused)
            .filter(|_| !on_screen);
        hosts(&mut lock(&self.0.held), output.id, following_focus)
    }

    /// Puts the launcher on `output` for as long as the returned seat lives.
    pub fn seat(&self, output: OutputId) -> LauncherSeat {
        *lock(&self.0.held) = Some(output);
        LauncherSeat {
            launcher: self.clone(),
            output,
        }
    }

    fn set_on_screen(&self, on_screen: bool) {
        let was_on_screen = self.0.on_screen.swap(on_screen, Ordering::Relaxed);
        if was_on_screen && !on_screen {
            self.wake_runner();
        }
    }

    fn wake_runner(&self) {
        if let Some(wake_runner) = self.0.runner_waker.get() {
            wake_runner();
        }
    }
}

/// Whether `output` hosts a surface that follows the focus: `following_focus` is whether `output`
/// is focused, or `None` when the surface stays where it is held. A surface held nowhere is held
/// by the first output asked.
pub fn hosts<Id: Copy + PartialEq>(
    held: &mut Option<Id>,
    output: Id,
    following_focus: Option<bool>,
) -> bool {
    match following_focus {
        Some(focused) => focused,
        None => *held.get_or_insert(output) == output,
    }
}

/// The launcher's hold on the output its surfaces were built for. Once they go, so does the hold,
/// and the launcher is put on another output.
pub struct LauncherSeat {
    launcher: LauncherOutput,
    output: OutputId,
}

impl LauncherSeat {
    /// Whether anything of the launcher shows; once it has faded out, it may follow the focus.
    pub fn set_on_screen(&self, on_screen: bool) {
        self.launcher.set_on_screen(on_screen);
    }
}

impl Drop for LauncherSeat {
    fn drop(&mut self) {
        let released = {
            let mut held = lock(&self.launcher.0.held);
            let ours = *held == Some(self.output);
            if ours {
                *held = None;
            }
            ours
        };
        if released {
            self.launcher.0.on_screen.store(false, Ordering::Relaxed);
            self.launcher.wake_runner();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::hosts;

    #[test]
    fn the_first_output_asked_holds_the_launcher_until_it_is_released() {
        let mut held = None;
        assert!(hosts(&mut held, 2, None));
        assert!(!hosts(&mut held, 1, None));
        held = None;
        assert!(hosts(&mut held, 1, None));
        assert!(!hosts(&mut held, 2, None));
    }

    #[test]
    fn a_hidden_launcher_is_on_the_focused_output_and_a_shown_one_stays_where_it_is() {
        let mut held = Some(1);
        let hidden: Vec<i32> = [1, 2, 3]
            .into_iter()
            .filter(|&output| hosts(&mut held, output, Some(output == 2)))
            .collect();
        assert_eq!(hidden, [2]);
        let shown: Vec<i32> = [3, 2, 1]
            .into_iter()
            .filter(|&output| hosts(&mut held, output, None))
            .collect();
        assert_eq!(shown, [1]);
    }
}
