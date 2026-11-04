//! The search field. Keys reach it from the launcher's root, so it never needs focus.

use crownui::prelude::{Runtime, Styled, View, bind, hstack, icon, text, vstack};

use super::{
    glass::{Glass, Showing, glass},
    metrics::{SHORT_ROW, span},
    state::Launcher,
};

pub const HEIGHT: f32 = 73.0;
pub const WIDTH: f32 = span(2);
const GLYPH: f32 = 26.0;
const TEXT: f32 = 22.0;
const CARET: (f32, f32) = (2.0, 26.0);
const PLACEHOLDER: &str = "Search";

pub fn search_bar(launcher: Launcher) -> impl View {
    let empty = move |runtime: &mut Runtime| launcher.query.with(runtime, String::is_empty);
    let glyph = icon(
        crownplugin_view::to_svg(
            crownplugin_view::lucide("search").unwrap_or_default(),
            GLYPH,
        ),
        bind(move |runtime| launcher.palette(runtime).on_clear),
    )
    .size(GLYPH);
    let label = text(bind(move |runtime| {
        let query = launcher.query.get(runtime);
        if query.is_empty() {
            PLACEHOLDER.to_owned()
        } else {
            query
        }
    }))
    .text_size(TEXT)
    .font_weight(450)
    .truncate()
    .text_color(bind(move |runtime| {
        let palette = launcher.palette(runtime);
        if empty(runtime) {
            palette.placeholder
        } else {
            palette.on_clear
        }
    }));
    let caret = vstack(())
        .w(CARET.0)
        .h(CARET.1)
        .ml(1.0)
        .rounded(1.0)
        .bg(bind(move |runtime| launcher.palette(runtime).on_clear))
        .hidden(bind(empty));
    let field = hstack((
        glyph,
        hstack((label, caret)).items_center().ml(22.0).min_w(0.0),
    ))
    .w(WIDTH)
    .h(HEIGHT)
    .pl(21.0)
    .pr(24.0)
    .items_center();
    vstack(glass(
        field,
        launcher,
        Glass::Clear,
        HEIGHT / 2.0,
        Showing::Always,
    ))
    .w(WIDTH)
    .h(SHORT_ROW)
    .justify_center()
}
