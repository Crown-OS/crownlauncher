use crownui::prelude::Cx;

use super::state::Launcher;
use crate::{
    plugins::PluginCommand,
    search::{Query, Results, Selection},
};

/// More than the list ever shows.
const APP_RESULTS: usize = 12;

/// Changes the query and searches again: the apps at once, the plugins as they answer.
pub fn edit(cx: &mut Cx, launcher: Launcher, change: impl FnOnce(&mut String)) {
    let before = launcher.query.get(cx);
    launcher.query.update(cx, change);
    let query = launcher.query.get(cx);
    if query == before {
        return;
    }
    list_apps(cx, launcher);
    launcher.send(cx, PluginCommand::Search(query));
}

/// Lists the apps matching the query afresh, dropping any plugin answers, with the top row
/// selected.
pub fn list_apps(cx: &mut Cx, launcher: Launcher) {
    let query = launcher.query.get(cx);
    let found = launcher.catalog.with(cx, |catalog| {
        let apps = catalog.search(&Query::new(&query), APP_RESULTS);
        Results::new(&query, &apps)
    });
    launcher.results.set(cx, found);
    launcher.selection.set(cx, Selection::default());
}
