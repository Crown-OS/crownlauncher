use crownui::prelude::{Styled, View, hstack, keyed};

use super::app_icon::app_icon;
use crate::ui::{
    glass::{Glass, Showing, glass},
    metrics::{RADIUS, SHORT_ROW, span},
    state::Launcher,
};

const LARGE_ICON: f32 = 92.0;
const LARGE_INSET: f32 = 4.0;
const SMALL_ICON: f32 = 64.0;
const SMALL_GAP: f32 = 12.0;
const MAX_SHORTCUTS: usize = 2;
const MAX_PINNED: usize = 3;

/// Large icons standing on the desktop itself, at either end of their column.
pub fn shortcuts(launcher: Launcher) -> impl View {
    keyed(
        move |runtime| {
            launcher
                .settings
                .with(runtime, |settings| settings.shortcuts.clone())
        },
        move |_, ids| {
            let icons: Vec<_> = ids
                .into_iter()
                .take(MAX_SHORTCUTS)
                .map(|id| app_icon(launcher, id, LARGE_ICON))
                .collect();
            hstack(icons)
                .w(span(1))
                .h(SHORT_ROW)
                .px(LARGE_INSET)
                .items_center()
                .justify_between()
        },
    )
}

/// A glass tray of small icons.
pub fn pinned(launcher: Launcher) -> impl View {
    let tray = keyed(
        move |runtime| {
            launcher
                .settings
                .with(runtime, |settings| settings.pinned.clone())
        },
        move |_, ids| {
            let icons: Vec<_> = ids
                .into_iter()
                .take(MAX_PINNED)
                .map(|id| app_icon(launcher, id, SMALL_ICON))
                .collect();
            hstack(icons).gap(SMALL_GAP).items_center().justify_center()
        },
    )
    .w(span(1))
    .h(SHORT_ROW)
    .items_center()
    .justify_center();
    glass(tray, launcher, Glass::Milky, RADIUS, Showing::Home)
}
