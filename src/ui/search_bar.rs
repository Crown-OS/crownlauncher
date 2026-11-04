//! The search field across the top of the window. Keys reach it from the launcher's root, so it
//! never needs focus; the caret sits before the prompt while empty and after the query once
//! typing starts.

use crownui::prelude::{Styled, View, bind, hstack, show, text, vstack};

use super::{metrics::SEARCH_HEIGHT, state::Launcher};

const INSET: f32 = 20.0;
const TEXT: f32 = 19.0;
const CARET: (f32, f32) = (2.0, 22.0);
const CARET_GAP: f32 = 2.0;
const PROMPT: &str = "Search for apps and commands...";

pub fn search_bar(launcher: Launcher) -> impl View {
    let line = show(
        bind(move |runtime| launcher.query.with(runtime, String::is_empty)),
        move || hstack((caret(launcher), prompt(launcher))).items_center(),
        move || hstack((typed(launcher), caret(launcher))).items_center(),
    )
    .min_w(0.0);
    hstack(line)
        .w_full()
        .h(SEARCH_HEIGHT)
        .px(INSET)
        .items_center()
}

fn caret(launcher: Launcher) -> impl View {
    vstack(())
        .w(CARET.0)
        .h(CARET.1)
        .mx(CARET_GAP / 2.0)
        .rounded(CARET.0 / 2.0)
        .bg(bind(move |runtime| launcher.palette(runtime).caret))
}

fn prompt(launcher: Launcher) -> impl View {
    text(PROMPT)
        .text_size(TEXT)
        .text_color(bind(move |runtime| launcher.palette(runtime).placeholder))
}

fn typed(launcher: Launcher) -> impl View {
    text(bind(move |runtime| launcher.query.get(runtime)))
        .text_size(TEXT)
        .truncate()
        .min_w(0.0)
        .text_color(bind(move |runtime| launcher.palette(runtime).text))
}
