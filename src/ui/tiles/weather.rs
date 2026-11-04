use crownui::prelude::{Runtime, Styled, View, bind, hstack, spacer, text, vstack};

use crate::{
    ui::{
        glass::{Glass, Showing, glass},
        images::picture,
        metrics::{RADIUS, SHORT_ROW, span},
        state::{Art, Launcher},
    },
    weather::Report,
};

const CITIES: usize = 2;
const TEMPERATURE: f32 = 64.0;
const CITY: f32 = 21.0;
const DEGREE: f32 = 46.0;
const RANGE: f32 = 18.0;
const ICON: f32 = 52.0;
const UNKNOWN: &str = "–";

/// Two cities side by side: temperature, name, today's range and the sky.
pub fn weather(launcher: Launcher) -> impl View {
    let divider = vstack(())
        .w(1.0)
        .h(40.0)
        .mx(17.0)
        .shrink(0.0)
        .bg(bind(move |runtime| launcher.palette(runtime).hairline));
    let row = hstack((city(launcher, 0, false), divider, city(launcher, 1, true)))
        .w(span(CITIES as u8))
        .h(SHORT_ROW)
        .pl(18.0)
        .pr(24.0)
        .items_center();
    glass(row, launcher, Glass::Milky, RADIUS, Showing::Home)
}

fn report(runtime: &mut Runtime, launcher: Launcher, index: usize) -> Option<Report> {
    launcher
        .weather
        .with(runtime, |reports| reports.get(index).cloned())
}

/// One city; the last one `fills` the tile, its sky against the far edge.
fn city(launcher: Launcher, index: usize, fills: bool) -> impl View {
    let ink = move |runtime: &mut Runtime| launcher.palette(runtime).text;
    let muted = move |runtime: &mut Runtime| launcher.palette(runtime).muted;
    let digits = text(bind(move |runtime| {
        report(runtime, launcher, index)
            .map_or_else(|| UNKNOWN.to_owned(), |report| report.degrees().0)
    }))
    .text_size(TEMPERATURE)
    .font_weight(400)
    .tracking(-1.0)
    .text_color(bind(ink));
    let degree = text("°")
        .text_size(DEGREE)
        .text_color(bind(ink))
        .ml(1.0)
        .mt(4.0);
    let temperature = hstack((digits, degree)).items_start().shrink(0.0);
    let name = text(bind(move |runtime| {
        launcher.settings.with(runtime, |settings| {
            settings
                .weather
                .get(index)
                .map(|city| city.name.clone())
                .unwrap_or_default()
        })
    }))
    .text_size(CITY)
    .font_weight(500)
    .truncate()
    .text_color(bind(ink));
    let range = text(bind(move |runtime| {
        report(runtime, launcher, index).map_or_else(
            || format!("{UNKNOWN}/{UNKNOWN}"),
            |report| report.degrees().1,
        )
    }))
    .text_size(RANGE)
    .text_color(bind(muted));
    let sky = picture(launcher, ICON, move |runtime, edge| {
        let report = report(runtime, launcher, index)?;
        Some(Art::Weather(report.condition, report.night, edge))
    });
    let row = hstack((
        temperature,
        vstack((name, range)).gap(4.0).ml(10.0).min_w(0.0),
        fills.then(spacer),
        sky.ml(8.0).shrink(0.0),
    ))
    .items_center();
    if fills { row.flex_1() } else { row }
}
