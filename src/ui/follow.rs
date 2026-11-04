//! Everything the launcher keeps up with: requests to show or hide, crownplugind's answers and
//! the installed apps.

use crownplugin_proto::SearchResults;
use crownui::prelude::Cx;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::{query, state::Launcher, visibility};
use crate::{catalog::Catalog, plugins};

pub fn start(cx: &mut Cx, launcher: Launcher, requests: UnboundedReceiver<bool>, shown: bool) {
    remote(cx, launcher, requests);
    connect_plugins(cx, launcher);
    rescan(cx, launcher);
    if shown {
        cx.request_animation_frame(move |cx, _| visibility::set(cx, launcher, true));
    }
}

fn remote(cx: &mut Cx, launcher: Launcher, mut requests: UnboundedReceiver<bool>) {
    let next = async move {
        let request = requests.recv().await;
        (requests, request)
    };
    cx.spawn(next, move |cx, (requests, request)| {
        let Some(shown) = request else {
            return;
        };
        visibility::set(cx, launcher, shown);
        remote(cx, launcher, requests);
    });
}

fn connect_plugins(cx: &mut Cx, launcher: Launcher) {
    let (answers, inbox) = unbounded_channel();
    let (commands, outbox) = unbounded_channel();
    launcher
        .plugins
        .update(cx, |plugins| *plugins = Some(commands));
    cx.spawn(plugins::run(answers, outbox), |_, ()| {});
    follow_answers(cx, launcher, inbox);
}

fn follow_answers(cx: &mut Cx, launcher: Launcher, mut inbox: UnboundedReceiver<SearchResults>) {
    let next = async move {
        let answer = inbox.recv().await;
        (inbox, answer)
    };
    cx.spawn(next, move |cx, (inbox, answer)| {
        let Some(answer) = answer else {
            return;
        };
        let mut results = launcher.results.get(cx);
        if results.answer(answer) {
            launcher.results.set(cx, results);
        }
        follow_answers(cx, launcher, inbox);
    });
}

/// Reloads the installed apps, keeping the old catalog when nothing changed. The list of every
/// app follows; a search under way keeps its answers.
pub fn rescan(cx: &mut Cx, launcher: Launcher) {
    let scan = async { tokio::task::spawn_blocking(Catalog::load).await.ok() };
    cx.spawn(scan, move |cx, catalog| {
        if let Some(catalog) = catalog
            && launcher.catalog.with(cx, |current| *current != catalog)
        {
            launcher.catalog.set(cx, catalog);
            if !launcher.searching(cx) {
                query::list_apps(cx, launcher);
            }
        }
    });
}
