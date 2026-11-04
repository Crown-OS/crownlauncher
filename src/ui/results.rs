//! The list under the search field: what the search found, grouped by source, each row an icon,
//! a title with its subtitle beside it, and the kind of thing it is at the far end.

use crownui::prelude::{
    AnyView, Listeners, Runtime, Styled, TRANSPARENT, View, bind, hstack, icon, keyed, text, vstack,
};

use super::{
    activate::activate,
    images::app_icon,
    metrics::{LIST_HEIGHT, LIST_PADDING, ROW_RADIUS},
    state::Launcher,
};
use crate::search::{Hit, Row, RowIcon, Section, Selection};

const ROW: f32 = 44.0;
const HEADER: f32 = 32.0;
const ICON: f32 = 22.0;
const SYMBOL: f32 = 17.0;
const TITLE: f32 = 14.5;
const DETAIL: f32 = 13.5;
const EMPTY: &str = "No Results";

/// The sections that fit the list.
pub fn fitted(runtime: &mut Runtime, launcher: Launcher) -> Vec<Section> {
    launcher
        .results
        .with(runtime, |results| results.fit(LIST_HEIGHT, HEADER, ROW))
}

/// The hit of the highlighted row, if any row is shown.
pub fn selected_hit(runtime: &mut Runtime, launcher: Launcher) -> Option<Hit> {
    let index = launcher.selection.get(runtime).0;
    fitted(runtime, launcher)
        .into_iter()
        .flat_map(|section| section.rows)
        .nth(index)
        .map(|row| row.hit)
}

/// What a hit is, as the row's far end and the footer name it.
pub fn kind(hit: &Hit) -> &'static str {
    match hit {
        Hit::App(_) => "Application",
        Hit::Plugin(_) => "Command",
    }
}

pub fn results(launcher: Launcher) -> impl View {
    keyed(
        move |runtime| {
            let query = launcher.query.get(runtime);
            (query, fitted(runtime, launcher))
        },
        move |_, (_, sections)| -> AnyView {
            if sections.is_empty() {
                return empty(launcher);
            }
            let mut index = 0;
            let groups: Vec<AnyView> = sections
                .into_iter()
                .map(|section| {
                    let rows: Vec<AnyView> = section
                        .rows
                        .into_iter()
                        .map(|row| {
                            index += 1;
                            result_row(launcher, row, index - 1)
                        })
                        .collect();
                    vstack((header(launcher, section.name), vstack(rows))).boxed()
                })
                .collect();
            vstack(groups).w_full().boxed()
        },
    )
    .w_full()
    .flex_1()
    .min_h(0.0)
    .p(LIST_PADDING)
    .overflow_hidden()
}

fn empty(launcher: Launcher) -> AnyView {
    vstack(
        text(EMPTY)
            .text_size(TITLE)
            .text_color(bind(move |runtime| launcher.palette(runtime).muted)),
    )
    .w_full()
    .h(LIST_HEIGHT)
    .items_center()
    .justify_center()
    .boxed()
}

fn header(launcher: Launcher, name: String) -> impl View {
    hstack(
        text(name)
            .text_size(12.5)
            .font_weight(600)
            .text_color(bind(move |runtime| launcher.palette(runtime).muted)),
    )
    .h(HEADER)
    .px(10.0)
    .pt(6.0)
    .items_center()
}

fn row_icon(launcher: Launcher, icon_of: RowIcon) -> AnyView {
    match icon_of {
        RowIcon::App(id) => app_icon(launcher, ICON, id).boxed(),
        RowIcon::Symbol(name) => {
            let body = crownplugin_view::lucide(&name)
                .or_else(|| crownplugin_view::lucide("circle"))
                .unwrap_or_default();
            vstack(
                icon(
                    crownplugin_view::to_svg(body, SYMBOL),
                    bind(move |runtime| launcher.palette(runtime).text),
                )
                .size(SYMBOL),
            )
            .size(ICON)
            .items_center()
            .justify_center()
            .boxed()
        }
        RowIcon::Blank => vstack(()).size(ICON).boxed(),
    }
}

fn result_row(launcher: Launcher, row: Row, index: usize) -> AnyView {
    let selected = move |runtime: &mut Runtime| launcher.selection.get(runtime) == Selection(index);
    let muted = move |runtime: &mut Runtime| launcher.palette(runtime).muted;
    let title = text(row.title)
        .text_size(TITLE)
        .font_weight(500)
        .truncate()
        .shrink(0.0)
        .max_w(360.0)
        .text_color(bind(move |runtime| launcher.palette(runtime).text));
    let subtitle = (!row.subtitle.is_empty()).then(|| {
        text(row.subtitle)
            .text_size(DETAIL)
            .truncate()
            .min_w(0.0)
            .ml(10.0)
            .text_color(bind(muted))
    });
    let accessory = text(kind(&row.hit))
        .text_size(DETAIL)
        .shrink(0.0)
        .ml(16.0)
        .text_color(bind(muted));
    let words = hstack((title, subtitle))
        .flex_1()
        .min_w(0.0)
        .ml(12.0)
        .items_center();
    let hit = row.hit;
    hstack((row_icon(launcher, row.icon), words, accessory))
        .h(ROW)
        .px(10.0)
        .items_center()
        .rounded(ROW_RADIUS)
        .bg(bind(move |runtime| {
            if selected(runtime) {
                launcher.palette(runtime).selected
            } else {
                TRANSPARENT
            }
        }))
        .on_pointer_enter(move |cx, _| launcher.selection.set(cx, Selection(index)))
        .on_click(move |cx, event| {
            event.stop_propagation();
            activate(cx, launcher, &hit);
        })
        .boxed()
}
