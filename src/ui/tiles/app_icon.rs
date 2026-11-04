//! An app's icon that opens it. A calendar app's icon is today's date, drawn live.

use crownui::prelude::{
    AnyView, BLACK, Listeners, Spring, Styled, View, WHITE, bind, keyed, text, vstack,
};
use jiff::fmt::strtime;

use crate::{
    catalog::DesktopId,
    ui::{
        activate::open_app,
        images::picture,
        state::{Art, Launcher},
    },
};

const HOVER_SCALE: f32 = 1.05;
const PRESS_SCALE: f32 = 0.95;
/// The calendar face's corner, day label and date, as fractions of its edge.
const CALENDAR_RADIUS: f32 = 0.235;
const CALENDAR_DAY: f32 = 0.22;
const CALENDAR_DATE: f32 = 0.55;

/// The icon of app `id`, `size` square; nothing while the app is not installed.
pub fn app_icon(launcher: Launcher, id: String, size: f32) -> impl View {
    let installed = id.clone();
    keyed(
        move |runtime| {
            launcher.catalog.with(runtime, |catalog| {
                catalog.get(&installed).map(|app| app.is_calendar())
            })
        },
        move |_, calendar| match calendar {
            Some(calendar) => face(launcher, id.clone(), size, calendar),
            None => vstack(()).display_none().boxed(),
        },
    )
}

fn face(launcher: Launcher, id: String, size: f32, calendar: bool) -> AnyView {
    let art = if calendar {
        calendar_face(launcher, size).boxed()
    } else {
        let desktop_id = DesktopId::new(id.clone());
        picture(launcher, size, move |_, edge| {
            Some(Art::App(desktop_id.clone(), edge))
        })
        .boxed()
    };
    vstack(art)
        .size(size)
        .hover(|style| style.scale(HOVER_SCALE))
        .active(|style| style.scale(PRESS_SCALE))
        .transition(Spring::snappy())
        .on_click(move |cx, event| {
            event.stop_propagation();
            open_app(cx, launcher, &id);
        })
        .boxed()
}

fn calendar_face(launcher: Launcher, size: f32) -> impl View {
    let weekday = text(bind(move |runtime| {
        launcher.now.with(runtime, |now| {
            strtime::format("%a", now).map_or_else(|_| String::new(), |day| day.to_uppercase())
        })
    }))
    .text_size(size * CALENDAR_DAY)
    .font_weight(600)
    .text_color(bind(move |runtime| launcher.palette(runtime).second_hand));
    let day = text(bind(move |runtime| {
        launcher.now.with(runtime, |now| now.day().to_string())
    }))
    .text_size(size * CALENDAR_DATE)
    .font_weight(400)
    .leading(1.0)
    .text_color(BLACK);
    vstack((weekday, day))
        .size(size)
        .pt(size * 0.08)
        .items_center()
        .rounded(size * CALENDAR_RADIUS)
        .bg(WHITE)
}
