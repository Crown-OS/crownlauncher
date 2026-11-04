//! What the search found, grouped by source, on one pane of glass.

use crownui::prelude::{
    AnyView, Listeners, Rect, Runtime, Styled, View, bind, hstack, icon, keyed, text, vstack,
};

use super::{
    activate::activate,
    glass::{Glass, Showing, glass},
    images::picture,
    metrics::{PANEL_WIDTH, RADIUS},
    state::{Art, Launcher},
};
use crate::search::{Row, RowIcon, Section, Selection};

pub const ROW: f32 = 52.0;
pub const HEADER: f32 = 30.0;
const PADDING: f32 = 12.0;
const ICON: f32 = 34.0;
const SYMBOL: f32 = 22.0;
const EMPTY: &str = "No results";

/// The sections that fit a pane `height` tall.
pub fn fitted(runtime: &mut Runtime, launcher: Launcher, height: f32) -> Vec<Section> {
    launcher.results.with(runtime, |results| {
        results.fit(height - PADDING * 2.0, HEADER, ROW)
    })
}

pub fn results(launcher: Launcher, frame: Rect) -> impl View {
    let content = keyed(
        move |runtime| {
            let query = launcher.query.get(runtime);
            (query, fitted(runtime, launcher, frame.size.height))
        },
        move |_, (_, sections)| -> AnyView {
            if sections.is_empty() {
                return text(EMPTY)
                    .text_size(18.0)
                    .text_color(bind(move |runtime| launcher.palette(runtime).muted))
                    .p(16.0)
                    .boxed();
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
            vstack(groups).boxed()
        },
    )
    .w_full();
    let pane = vstack(content)
        .absolute()
        .left(frame.origin.x)
        .top(frame.origin.y)
        .w(PANEL_WIDTH)
        .h(frame.size.height)
        .p(PADDING)
        .overflow_hidden();
    glass(pane, launcher, Glass::Milky, RADIUS, Showing::Search)
}

fn header(launcher: Launcher, name: String) -> impl View {
    text(name)
        .text_size(14.0)
        .font_weight(600)
        .text_color(bind(move |runtime| launcher.palette(runtime).muted))
        .h(HEADER)
        .pl(14.0)
        .pt(8.0)
}

fn row_icon(launcher: Launcher, icon_of: RowIcon) -> AnyView {
    match icon_of {
        RowIcon::App(id) => picture(launcher, ICON, move |_, edge| {
            Some(Art::App(id.clone(), edge))
        })
        .boxed(),
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
    let words = vstack((
        text(row.title)
            .text_size(18.0)
            .font_weight(500)
            .truncate()
            .text_color(bind(move |runtime| launcher.palette(runtime).text)),
        (!row.subtitle.is_empty()).then(|| {
            text(row.subtitle)
                .text_size(14.0)
                .truncate()
                .text_color(bind(move |runtime| launcher.palette(runtime).muted))
        }),
    ))
    .flex_1()
    .min_w(0.0)
    .ml(14.0);
    let hit = row.hit;
    hstack((row_icon(launcher, row.icon), words))
        .h(ROW)
        .px(14.0)
        .items_center()
        .rounded(14.0)
        .bg(bind(move |runtime| {
            let palette = launcher.palette(runtime);
            if selected(runtime) {
                palette.selected
            } else {
                crownui::prelude::TRANSPARENT
            }
        }))
        .on_pointer_enter(move |cx, _| launcher.selection.set(cx, Selection(index)))
        .on_click(move |cx, event| {
            event.stop_propagation();
            activate(cx, launcher, &hit);
        })
        .boxed()
}
