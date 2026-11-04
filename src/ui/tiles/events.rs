//! The soonest three agenda entries, each on its own glass pill.

use crownui::prelude::{AnyView, Runtime, Styled, View, bind, hstack, keyed, text, vstack};
use jiff::fmt::strtime;

use crate::{
    settings::{Upcoming, agenda},
    ui::{
        glass::{Glass, Showing, glass},
        metrics::{CELL, TALL_ROW},
        state::Launcher,
    },
};

const SHOWN: usize = 3;
const GAP: f32 = 9.0;
const PILL: f32 = (TALL_ROW - GAP * (SHOWN as f32 - 1.0)) / SHOWN as f32;
const TITLE: f32 = 22.0;
const DETAIL: f32 = 18.0;
const NOTHING: &str = "No upcoming events";

/// What a pill says under the title, and at its end.
pub fn describe(event: &Upcoming) -> (String, Option<String>) {
    let detail = match (event.days, event.time) {
        (0, Some(time)) => format!(
            "{} - {}",
            strtime::format("%H:%M", time.start).unwrap_or_default(),
            strtime::format("%H:%M", time.end).unwrap_or_default()
        ),
        (0, None) => "Today".to_owned(),
        _ => strtime::format("%B %-d", event.date).unwrap_or_default(),
    };
    let countdown = match event.days {
        0 => None,
        1 => Some("1 day".to_owned()),
        days => Some(format!("{days} days")),
    };
    (detail, countdown)
}

pub fn events(launcher: Launcher) -> impl View {
    keyed(
        move |runtime| {
            let now = launcher.now.with(runtime, |now| now.datetime());
            launcher
                .settings
                .with(runtime, |settings| agenda(&settings.events, now, SHOWN))
        },
        move |_, upcoming| {
            let pills: Vec<AnyView> = if upcoming.is_empty() {
                vec![pill(launcher, NOTHING.to_owned(), String::new(), None)]
            } else {
                upcoming
                    .iter()
                    .map(|event| {
                        let (detail, countdown) = describe(event);
                        pill(launcher, event.title.clone(), detail, countdown)
                    })
                    .collect()
            };
            vstack(pills).gap(GAP)
        },
    )
    .w(CELL)
    .h(TALL_ROW)
}

fn pill(launcher: Launcher, title: String, detail: String, countdown: Option<String>) -> AnyView {
    let ink = move |runtime: &mut Runtime| launcher.palette(runtime).text;
    let muted = move |runtime: &mut Runtime| launcher.palette(runtime).muted;
    let heading = vstack((
        text(title)
            .text_size(TITLE)
            .font_weight(500)
            .truncate()
            .text_color(bind(ink)),
        text(detail).text_size(DETAIL).text_color(bind(muted)),
    ))
    .flex_1()
    .min_w(0.0);
    let trailing = countdown.map(|countdown| {
        text(countdown)
            .text_size(TITLE)
            .text_color(bind(muted))
            .ml(4.0)
            .mt(6.0)
    });
    let row = hstack((heading, trailing))
        .w(CELL)
        .h(PILL)
        .pl(25.0)
        .pr(22.0)
        .pt(3.0)
        .items_center();
    glass(row, launcher, Glass::Milky, PILL / 2.0, Showing::Home).boxed()
}

#[cfg(test)]
mod tests {
    use jiff::civil::{date, time};

    use super::*;
    use crate::settings::TimeRange;

    fn upcoming(days: i32, time: Option<TimeRange>) -> Upcoming {
        Upcoming {
            title: "x".into(),
            date: date(2026, 6, 2),
            days,
            time,
        }
    }

    #[test]
    fn pills_read_like_the_reference() {
        let meeting = TimeRange {
            start: time(17, 0, 0, 0),
            end: time(18, 0, 0, 0),
        };
        assert_eq!(
            describe(&upcoming(0, Some(meeting))),
            ("17:00 - 18:00".to_owned(), None)
        );
        assert_eq!(
            describe(&upcoming(9, None)),
            ("June 2".to_owned(), Some("9 days".to_owned()))
        );
        assert_eq!(describe(&upcoming(1, None)).1.as_deref(), Some("1 day"));
        assert_eq!(describe(&upcoming(0, None)).0, "Today");
    }
}
