use crownui::prelude::{Cx, KeyboardInteractivity};
use tokio::sync::mpsc::UnboundedSender;

use super::{follow, motion, query, state::Launcher};
use crate::{ipc::Shown, plugins::PluginCommand};

/// Everything outside the UI that follows whether the launcher is shown.
#[derive(Default)]
pub struct Observers {
    /// What IPC answers a toggle with.
    pub ipc: Shown,
    /// The click catcher under the window.
    pub catcher: Option<UnboundedSender<bool>>,
}

impl Observers {
    fn tell(&self, shown: bool) {
        self.ipc.set(shown);
        if let Some(catcher) = &self.catcher {
            let _ = catcher.send(shown);
        }
    }
}

pub fn set(cx: &mut Cx, launcher: Launcher, shown: bool) {
    if launcher.shown.get(cx) == shown {
        return;
    }
    launcher.shown.set(cx, shown);
    launcher
        .observers
        .with(cx, |observers| observers.tell(shown));
    launcher.send(cx, PluginCommand::Shown(shown));
    if shown {
        cx.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        follow::rescan(cx, launcher);
    } else {
        cx.set_keyboard_interactivity(KeyboardInteractivity::None);
        launcher.query.set(cx, String::new());
        query::list_apps(cx, launcher);
    }
    motion::reveal(cx, launcher, shown);
}
