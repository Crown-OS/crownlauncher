use crownui::prelude::{Cx, KeyboardInteractivity};

use super::{follow, motion, state::Launcher};
use crate::{plugins::PluginCommand, search::Results};

pub fn set(cx: &mut Cx, launcher: Launcher, shown: bool) {
    if launcher.shown.get(cx) == shown {
        return;
    }
    launcher.shown.set(cx, shown);
    launcher.report.with(cx, |report| report.set(shown));
    launcher.send(cx, PluginCommand::Shown(shown));
    if shown {
        cx.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        follow::on_show(cx, launcher);
    } else {
        cx.set_keyboard_interactivity(KeyboardInteractivity::None);
        launcher.query.set(cx, String::new());
        launcher.results.set(cx, Results::default());
    }
    motion::reveal(cx, launcher, if shown { 1.0 } else { 0.0 });
}
