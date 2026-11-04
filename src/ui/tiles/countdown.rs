//! Days left to one date, on a solid accent card.

use crownui::{
    ext::{Brush, GradientStop, Paint},
    prelude::{
        CornerRadii, Point, Rect, Runtime, Styled, View, bind, canvas, hstack, spacer, text,
        vstack, zstack,
    },
};
use jiff::{civil::Date, fmt::strtime};

use crate::ui::{
    glass::{Showing, material},
    metrics::{CELL, RADIUS, RIM, TALL_ROW},
    state::Launcher,
};

const TITLE: f32 = 22.0;
const NUMBER: f32 = 90.0;
const UNIT: f32 = 19.0;
const DATE: f32 = 21.0;

fn remaining(runtime: &mut Runtime, launcher: Launcher) -> Option<(Date, i32)> {
    let today = launcher.now.with(runtime, |now| now.date());
    launcher
        .settings
        .with(runtime, |settings| settings.countdown.remaining(today))
}

/// `2026/8/18` and `Tuesday`.
pub fn date_lines(date: Date) -> (String, String) {
    (
        format!("{}/{}/{}", date.year(), date.month(), date.day()),
        strtime::format("%A", date).unwrap_or_default(),
    )
}

pub fn countdown(launcher: Launcher) -> impl View {
    let on_accent = move |runtime: &mut Runtime| launcher.palette(runtime).on_accent;
    let line = move |text_of: fn(Date) -> String| {
        text(bind(move |runtime| {
            remaining(runtime, launcher).map_or_else(String::new, |(date, _)| text_of(date))
        }))
        .text_size(DATE)
        .text_color(bind(on_accent))
    };
    let title = text(bind(move |runtime| {
        launcher
            .settings
            .with(runtime, |settings| settings.countdown.title.clone())
    }))
    .text_size(TITLE)
    .truncate()
    .text_color(bind(on_accent));
    let days = hstack((
        text(bind(move |runtime| {
            remaining(runtime, launcher)
                .map_or_else(|| "–".to_owned(), |(_, days)| days.to_string())
        }))
        .text_size(NUMBER)
        .leading(1.0)
        .tracking(-2.0)
        .text_color(bind(on_accent)),
        text(bind(move |runtime| {
            let one = remaining(runtime, launcher).is_some_and(|(_, days)| days == 1);
            if one { "Day" } else { "Days" }.to_owned()
        }))
        .text_size(UNIT)
        .ml(2.0)
        .mb(13.0)
        .text_color(bind(on_accent)),
    ))
    .items_end()
    .mt(15.0);
    let content = vstack((
        title,
        days,
        spacer(),
        line(|date| date_lines(date).0),
        line(|date| date_lines(date).1),
    ))
    .absolute()
    .inset(0.0)
    .pl(28.0)
    .pr(20.0)
    .pt(21.0)
    .pb(22.0);
    let card = canvas(move |runtime, size, list| {
        let palette = launcher.palette(runtime);
        let gradient = Paint {
            brush: Brush::LinearGradient {
                start: Point::ZERO,
                end: Point::new(size.width, size.height),
                stops: [(0.0, palette.accent_start), (1.0, palette.accent_end)]
                    .iter()
                    .map(|&(offset, color)| GradientStop { offset, color })
                    .collect(),
            },
            blur: None,
        };
        let bounds = Rect::new(Point::ZERO, size);
        list.fill(bounds, CornerRadii::uniform(RADIUS), gradient);
        list.stroke(
            bounds,
            CornerRadii::uniform(RADIUS),
            RIM,
            palette.accent_rim,
        );
    })
    .absolute()
    .inset(0.0);
    zstack((card, content))
        .w(CELL)
        .h(TALL_ROW)
        .rounded(RADIUS)
        .hidden(bind(move |runtime| Showing::Home.hidden(runtime, launcher)))
        .material(bind(move |runtime| material(runtime, launcher)))
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn the_date_reads_like_the_reference() {
        assert_eq!(
            date_lines(date(2026, 8, 18)),
            ("2026/8/18".to_owned(), "Tuesday".to_owned())
        );
    }
}
