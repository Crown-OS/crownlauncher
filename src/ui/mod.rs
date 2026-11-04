//! The launcher as crownui views: one transparent overlay over the whole output, holding the
//! home grid of glass tiles in its centre. The overlay itself takes input only while shown, so
//! a click anywhere off the grid dismisses it.

mod activate;
mod arrange;
mod follow;
mod glass;
mod grid;
mod images;
mod keys;
mod metrics;
mod motion;
mod palette;
mod query;
mod results;
mod search_bar;
mod state;
mod tiles;
mod visibility;

use crownui::{
    kit::{self, Theme},
    prelude::{Capabilities, Listeners, Styled, View, bind, component, vstack, zstack},
};
use jiff::Zoned;

pub use palette::Palette;
pub use state::Launcher;

use crate::{
    catalog::Catalog,
    ipc::{Remote, Shown},
    settings::LauncherSettings,
    weather::Report,
};

/// How far the grid grows into place as it appears.
const ENTRY_SCALE: f32 = 0.94;

pub fn root(settings: LauncherSettings, remote: Remote, shown: bool) -> impl View {
    component(move |cx| {
        let theme = kit::provide_configured_theme(cx);
        let Remote {
            requests,
            shown: report,
        } = remote;
        let launcher = Launcher::new(cx, settings, theme, report);
        follow::start(cx, launcher, requests, shown);
        zstack((dismiss(launcher), centred(launcher)))
            .pointer_events_none()
            .on_key_down(move |cx, event| keys::down(cx, launcher, event))
    })
}

fn dismiss(launcher: Launcher) -> impl View {
    vstack(())
        .absolute()
        .inset(0.0)
        .pointer_events_auto()
        .hidden(bind(move |runtime| !launcher.shown.get(runtime)))
        .on_click(move |cx, _| visibility::set(cx, launcher, false))
}

fn centred(launcher: Launcher) -> impl View {
    let panel = grid::grid(launcher)
        .opacity(bind(move |runtime| {
            launcher.reveal.get(runtime).clamp(0.0, 1.0)
        }))
        .scale(bind(move |runtime| {
            let reveal = launcher.reveal.get(runtime).clamp(0.0, 1.0);
            ENTRY_SCALE + (1.0 - ENTRY_SCALE) * reveal
        }))
        .pointer_events_auto()
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
    pub now: Zoned,
    pub weather: Vec<Report>,
    pub catalog: Catalog,
    pub query: String,
}

/// The launcher shown in the state `staging` describes, following nothing, its glass drawn as
/// if the compositor blurred behind it.
pub fn staged(settings: LauncherSettings, staging: Staging) -> impl View {
    component(move |cx| {
        let theme = kit::provide_theme(
            cx,
            if staging.dark {
                Theme::dark()
            } else {
                Theme::light()
            },
        );
        let launcher = Launcher::new(cx, settings, theme, Shown::default());
        let capabilities = launcher.capabilities.get(cx);
        launcher.capabilities.set(
            cx,
            Capabilities {
                materials: true,
                ..capabilities
            },
        );
        launcher.shown.set(cx, true);
        launcher.reveal.set(cx, 1.0);
        launcher.now.set(cx, staging.now);
        launcher.weather.set(cx, staging.weather);
        launcher.catalog.set(cx, staging.catalog);
        query::edit(cx, launcher, |query| *query = staging.query);
        zstack(centred(launcher)).absolute().inset(0.0)
    })
}
