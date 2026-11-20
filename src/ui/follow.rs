//! Everything the launcher keeps up with: requests to show or hide, crownplugind's answers and
//! the installed apps.

use crownplugin_proto::SearchResults;
use crownui::prelude::Cx;
use tokio::sync::{
    broadcast::{self, error::RecvError},
    watch,
};

use super::{query, state::Launcher, visibility};
use crate::{
    catalog::{Catalog, SharedCatalog},
    ipc::OutputRequests,
    plugins::PluginLink,
};

/// What a launcher window follows from outside its UI.
pub struct Links {
    pub requests: OutputRequests,
    pub catalog: SharedCatalog,
    pub plugins: PluginLink,
}

pub fn start(cx: &mut Cx, launcher: Launcher, links: Links, shown: bool) {
    remote(cx, launcher, links.requests);
    follow_catalog(cx, launcher, links.catalog.subscribe());
    launcher
        .shared_catalog
        .update(cx, |shared| *shared = links.catalog);
    launcher
        .plugins
        .update(cx, |plugins| *plugins = Some(links.plugins.commands()));
    follow_answers(cx, launcher, links.plugins.subscribe());
    if shown {
        cx.request_animation_frame(move |cx, _| visibility::set(cx, launcher, true));
    }
}

fn remote(cx: &mut Cx, launcher: Launcher, requests: OutputRequests) {
    cx.spawn(requests.next(), move |cx, request| {
        let Some((requests, shown)) = request else {
            return;
        };
        visibility::set(cx, launcher, shown);
        remote(cx, launcher, requests);
    });
}

/// Takes the answers meant for the shown window; the hidden ones ignore them.
fn follow_answers(
    cx: &mut Cx,
    launcher: Launcher,
    mut answers: broadcast::Receiver<SearchResults>,
) {
    let next = async move {
        let answer = loop {
            match answers.recv().await {
                Ok(answer) => break Some(answer),
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break None,
            }
        };
        (answers, answer)
    };
    cx.spawn(next, move |cx, (answers, answer)| {
        let Some(answer) = answer else {
            return;
        };
        if launcher.shown.get(cx) {
            let mut results = launcher.results.get(cx);
            if results.answer(answer) {
                launcher.results.set(cx, results);
            }
        }
        follow_answers(cx, launcher, answers);
    });
}

/// Takes each catalog published for every window. The list of every app follows; a search under
/// way keeps its answers.
fn follow_catalog(cx: &mut Cx, launcher: Launcher, mut catalogs: watch::Receiver<Catalog>) {
    let next = async move {
        let open = catalogs.changed().await.is_ok();
        let catalog = open.then(|| catalogs.borrow_and_update().clone());
        (catalogs, catalog)
    };
    cx.spawn(next, move |cx, (catalogs, catalog)| {
        let Some(catalog) = catalog else {
            return;
        };
        launcher.catalog.set(cx, catalog);
        if !launcher.searching(cx) {
            query::list_apps(cx, launcher);
        }
        follow_catalog(cx, launcher, catalogs);
    });
}

/// Loads the installed apps again for every window; nothing changes when they are the same.
pub fn rescan(cx: &mut Cx, launcher: Launcher) {
    let shared = launcher.shared_catalog.with(cx, SharedCatalog::clone);
    let scan = async move {
        let _ = tokio::task::spawn_blocking(move || shared.publish(Catalog::load())).await;
    };
    cx.spawn(scan, |_, ()| {});
}
