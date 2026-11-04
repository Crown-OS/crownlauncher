//! The band along the bottom of the window: the crown on the left, and on the right what Enter
//! does to the highlighted row.

use crownui::prelude::{Runtime, Styled, View, bind, hstack, icon, text, vstack};

use super::{
    metrics::{FOOTER_HEIGHT, KEYCAP_RADIUS},
    results::{kind, selected_hit},
    state::Launcher,
};
use crate::search::Hit;

const INSET: f32 = 14.0;
const MARK: f32 = 17.0;
const LABEL: f32 = 13.0;
const KEYCAP: f32 = 22.0;
const KEY_GLYPH: f32 = 12.0;

fn primary_action(hit: &Hit) -> String {
    let verb = match hit {
        Hit::App(_) => "Open",
        Hit::Plugin(_) => "Run",
    };
    format!("{verb} {}", kind(hit))
}

fn lucide(launcher: Launcher, name: &str, size: f32) -> impl View + Styled {
    icon(
        crownplugin_view::to_svg(crownplugin_view::lucide(name).unwrap_or_default(), size),
        bind(move |runtime| launcher.palette(runtime).muted),
    )
    .size(size)
}

pub fn footer(launcher: Launcher) -> impl View {
    let action = move |runtime: &mut Runtime| selected_hit(runtime, launcher);
    let label = text(bind(move |runtime| {
        action(runtime)
            .as_ref()
            .map(primary_action)
            .unwrap_or_default()
    }))
    .text_size(LABEL)
    .font_weight(500)
    .text_color(bind(move |runtime| launcher.palette(runtime).text));
    let enter = vstack(lucide(launcher, "corner-down-left", KEY_GLYPH))
        .size(KEYCAP)
        .ml(8.0)
        .rounded(KEYCAP_RADIUS)
        .items_center()
        .justify_center()
        .bg(bind(move |runtime| launcher.palette(runtime).keycap));
    let primary = hstack((label, enter))
        .items_center()
        .hidden(bind(move |runtime| action(runtime).is_none()));
    hstack((
        lucide(launcher, "crown", MARK),
        vstack(()).flex_1(),
        primary,
    ))
    .w_full()
    .h(FOOTER_HEIGHT)
    .px(INSET)
    .items_center()
    .bg(bind(move |runtime| launcher.palette(runtime).footer))
}
