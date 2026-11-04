//! The launcher as crownui views: one transparent overlay over the whole output, holding a
//! Raycast-style glass window in its centre — the search field, the list of what it found and a
//! footer naming what Enter does. The overlay itself takes input only while shown, so a click
//! anywhere off the window dismisses it.

mod activate;
mod catcher;
mod follow;
mod footer;
mod glass;
mod images;
mod keys;
pub mod metrics;
mod motion;
mod palette;
mod query;
mod results;
mod search_bar;
mod state;
mod visibility;

use crownui::{
    kit::{self, Theme},
    prelude::{Capabilities, Listeners, Styled, Vector, View, bind, component, vstack, zstack},
};

pub use catcher::catcher;
use glass::glass;
pub use palette::Palette;
pub use state::Launcher;
pub use visibility::Observers;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::catalog::Catalog;
use metrics::{HAIRLINE, WINDOW_HEIGHT, WINDOW_RADIUS, WINDOW_WIDTH};

/// The launcher, following show and hide `requests` and keeping `observers` told; shown at once
/// unless `shown` is false.
pub fn root(requests: UnboundedReceiver<bool>, observers: Observers, shown: bool) -> impl View {
    component(move |cx| {
        let theme = kit::provide_configured_theme(cx);
        let launcher = Launcher::new(cx, theme, observers);
        follow::start(cx, launcher, requests, shown);
        zstack(centred(launcher))
            .pointer_events_none()
            .on_key_down(move |cx, event| keys::down(cx, launcher, event))
    })
}

fn hairline(launcher: Launcher) -> impl View {
    vstack(())
        .w_full()
        .h(HAIRLINE)
        .shrink(0.0)
        .bg(bind(move |runtime| launcher.palette(runtime).hairline))
}

fn window(launcher: Launcher) -> impl View {
    let content = vstack((
        search_bar::search_bar(launcher),
        hairline(launcher),
        results::results(launcher),
    ))
    .w_full()
    .flex_1()
    .min_h(0.0)
    .translate(bind(move |runtime| {
        Vector::new(0.0, motion::drop_offset(runtime, launcher))
    }));
    let body = vstack((content, hairline(launcher), footer::footer(launcher)))
        .w(WINDOW_WIDTH)
        .h(WINDOW_HEIGHT)
        .overflow_hidden()
        .pointer_events_auto();
    glass(body, launcher, WINDOW_RADIUS)
}

fn centred(launcher: Launcher) -> impl View {
    let panel = vstack(window(launcher))
        .pointer_events_none()
        .hidden(bind(move |runtime| !launcher.on_screen(runtime)));
    vstack(panel)
        .absolute()
        .inset(0.0)
        .items_center()
        .justify_center()
        .pointer_events_none()
}

/// One moment of the launcher, held still: for previews, screenshots and tests.
pub struct Staging {
    pub dark: bool,
    pub catalog: Catalog,
    pub query: String,
    /// How far the opening has come: the fade and the drop, 1 for at rest.
    pub opening: (f32, f32),
}

/// The launcher shown in the state `staging` describes, following nothing, its glass drawn as
/// if the compositor blurred behind it.
pub fn staged(staging: Staging) -> impl View {
    component(move |cx| {
        let theme = kit::provide_theme(
            cx,
            if staging.dark {
                Theme::dark()
            } else {
                Theme::light()
            },
        );
        let launcher = Launcher::new(cx, theme, Observers::default());
        let capabilities = launcher.capabilities.get(cx);
        launcher.capabilities.set(
            cx,
            Capabilities {
                materials: true,
                ..capabilities
            },
        );
        launcher.shown.set(cx, true);
        launcher.reveal.set(cx, staging.opening.0);
        launcher.settle.set(cx, staging.opening.1);
        launcher.catalog.set(cx, staging.catalog);
        launcher.query.set(cx, staging.query);
        query::list_apps(cx, launcher);
        zstack(centred(launcher)).absolute().inset(0.0)
    })
}
