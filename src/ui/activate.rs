use crownui::prelude::Cx;

use super::{state::Launcher, visibility};
use crate::{catalog, plugins::PluginCommand, search::Hit};

/// Starts the app `id` and gets out of the way.
pub fn open_app(cx: &mut Cx, launcher: Launcher, id: &str) {
    let found = launcher
        .catalog
        .with(cx, |catalog| catalog.get(id).map(catalog::launch).is_some());
    if !found {
        log::warn!("{id} is not installed");
        return;
    }
    visibility::set(cx, launcher, false);
}

pub fn activate(cx: &mut Cx, launcher: Launcher, hit: &Hit) {
    match hit {
        Hit::App(id) => open_app(cx, launcher, id.as_str()),
        Hit::Plugin(key) => {
            launcher.send(cx, PluginCommand::Activate(key.clone()));
            visibility::set(cx, launcher, false);
        }
    }
}
