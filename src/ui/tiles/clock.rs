//! An analog clock in the configured city's time zone. Its face is milky enough to stand without
//! the compositor's glass, which draws continuous (squircle) corners and so cannot be round.

use std::f32::consts::TAU;

use crownui::{
    ext::{Brush, DrawCommand, DrawList, GradientStop, Paint, Path, Stroke},
    prelude::{
        Color, CornerRadii, Point, Rect, Runtime, Size, Styled, TRANSPARENT, View, WHITE, bind,
        canvas, hstack, text, vstack, zstack,
    },
};
use jiff::{Zoned, tz::TimeZone};

use crate::ui::{
    glass::Showing,
    metrics::{CELL, RIM},
    palette::Palette,
    state::Launcher,
};

const RADIUS: f32 = CELL / 2.0;
const TICK_OUTER: f32 = 114.0;
const HOUR_TICK: f32 = 12.0;
const MINUTE_TICK: f32 = 6.0;
const NUMERAL_RADIUS: f32 = 89.0;
const NUMERAL_SIZE: f32 = 20.0;
const NUMERAL_BOX: Size = Size::new(30.0, 24.0);
const HOUR_HAND: (f32, f32) = (72.0, 7.0);
const MINUTE_HAND: (f32, f32) = (117.0, 5.0);
const SECOND_HAND: (f32, f32) = (118.0, 2.5);
const SECOND_TAIL: f32 = 32.0;
const HUB: f32 = 5.5;
const HUB_HOLE: f32 = 2.5;
const LABEL_OFFSET: f32 = 44.0;

/// Clockwise from twelve, in turns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hands {
    pub hour: f32,
    pub minute: f32,
    pub second: f32,
}

impl Hands {
    pub fn at(time: &Zoned) -> Self {
        let second = f32::from(time.second());
        let minute = f32::from(time.minute()) + second / 60.0;
        let hour = f32::from(time.hour() % 12) + minute / 60.0;
        Self {
            hour: hour / 12.0,
            minute: minute / 60.0,
            second: second / 60.0,
        }
    }
}

fn local_time(runtime: &mut Runtime, launcher: Launcher) -> Zoned {
    let zone = launcher
        .settings
        .with(runtime, |settings| settings.clock.time_zone.clone());
    let now = launcher.now.get(runtime);
    match TimeZone::get(&zone) {
        Ok(zone) => now.with_time_zone(zone),
        Err(_) => now,
    }
}

/// The point `radius` from the centre at `turns` clockwise from twelve.
fn polar(turns: f32, radius: f32) -> Point {
    let angle = turns * TAU;
    Point::new(RADIUS + radius * angle.sin(), RADIUS - radius * angle.cos())
}

fn line(list: &mut DrawList, from: Point, to: Point, width: f32, color: Color) {
    list.push(DrawCommand::StrokePath(
        Path::new().move_to(from).line_to(to),
        Stroke::rounded(width),
        Paint::solid(color),
    ));
}

fn disc(list: &mut DrawList, radius: f32, color: Color) {
    let rect = Rect::from_xywh(RADIUS - radius, RADIUS - radius, radius * 2.0, radius * 2.0);
    list.fill(rect, CornerRadii::uniform(radius), color);
}

fn face(list: &mut DrawList, palette: &Palette) {
    let size = RADIUS * 2.0;
    let stops = [
        (0.0, TRANSPARENT),
        (0.30, TRANSPARENT),
        (0.44, palette.face_band),
        (0.60, TRANSPARENT),
        (1.0, TRANSPARENT),
    ];
    let band = Paint {
        brush: Brush::LinearGradient {
            start: Point::new(size * 0.62, size * 0.02),
            end: Point::new(size * 0.38, size * 0.98),
            stops: stops
                .iter()
                .map(|&(offset, color)| GradientStop { offset, color })
                .collect(),
        },
        blur: None,
    };
    list.fill(
        Rect::from_xywh(0.0, 0.0, size, size),
        CornerRadii::uniform(RADIUS),
        band,
    );
    for tick in 0..60 {
        let turns = tick as f32 / 60.0;
        let (length, width, color) = if tick % 5 == 0 {
            (HOUR_TICK, 2.0, palette.hour_tick)
        } else {
            (MINUTE_TICK, 1.2, palette.minute_tick)
        };
        line(
            list,
            polar(turns, TICK_OUTER),
            polar(turns, TICK_OUTER - length),
            width,
            color,
        );
    }
}

fn hands(list: &mut DrawList, hands: Hands, palette: &Palette) {
    let centre = Point::new(RADIUS, RADIUS);
    line(
        list,
        centre,
        polar(hands.hour, HOUR_HAND.0),
        HOUR_HAND.1,
        palette.hand,
    );
    line(
        list,
        centre,
        polar(hands.minute, MINUTE_HAND.0),
        MINUTE_HAND.1,
        palette.hand,
    );
    line(
        list,
        polar(hands.second + 0.5, SECOND_TAIL),
        polar(hands.second, SECOND_HAND.0),
        SECOND_HAND.1,
        palette.second_hand,
    );
    disc(list, HUB, palette.second_hand);
    disc(list, HUB_HOLE, WHITE);
}

fn numeral(launcher: Launcher, hour: u8) -> impl View {
    let centre = polar(f32::from(hour) / 12.0, NUMERAL_RADIUS);
    let digits = text(hour.to_string())
        .text_size(NUMERAL_SIZE)
        .font_weight(600)
        .text_color(bind(move |runtime| launcher.palette(runtime).text));
    hstack(digits)
        .justify_center()
        .items_center()
        .absolute()
        .left(centre.x - NUMERAL_BOX.width / 2.0)
        .top(centre.y - NUMERAL_BOX.height / 2.0)
        .w(NUMERAL_BOX.width)
        .h(NUMERAL_BOX.height)
}

fn label(launcher: Launcher) -> impl View {
    let city = text(bind(move |runtime| {
        launcher
            .settings
            .with(runtime, |settings| settings.clock.label.clone())
    }))
    .text_size(17.0)
    .text_color(bind(move |runtime| launcher.palette(runtime).muted))
    .px(6.0)
    .py(1.0)
    .rounded(6.0)
    .bg(bind(move |runtime| launcher.palette(runtime).label));
    vstack(city)
        .absolute()
        .left(0.0)
        .right(0.0)
        .top(RADIUS + LABEL_OFFSET - 12.0)
        .items_center()
}

pub fn clock(launcher: Launcher) -> impl View {
    let dial = canvas(move |runtime, _, list| {
        let palette = launcher.palette(runtime);
        face(list, &palette);
    })
    .absolute()
    .inset(0.0);
    let pointer = canvas(move |runtime, _, list| {
        let palette = launcher.palette(runtime);
        let time = local_time(runtime, launcher);
        hands(list, Hands::at(&time), &palette);
    })
    .absolute()
    .inset(0.0);
    let numerals: Vec<_> = (1..=12).map(|hour| numeral(launcher, hour)).collect();
    zstack((
        dial,
        zstack(numerals).absolute().inset(0.0),
        label(launcher),
        pointer,
    ))
    .size(RADIUS * 2.0)
    .rounded(RADIUS)
    .bg(bind(move |runtime| launcher.palette(runtime).face))
    .border(RIM)
    .border_color(bind(move |runtime| launcher.palette(runtime).rim))
    .hidden(bind(move |runtime| Showing::Home.hidden(runtime, launcher)))
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn the_hands_point_where_the_reference_shows_them() {
        let Ok(time) = date(2026, 5, 24).at(20, 16, 30, 0).in_tz("Europe/Berlin") else {
            panic!("the zone exists");
        };
        let hands = Hands::at(&time);
        assert!((hands.hour - (8.275 / 12.0)).abs() < 1e-4);
        assert!((hands.minute - (16.5 / 60.0)).abs() < 1e-4);
        assert!((hands.second - 0.5).abs() < 1e-6);
    }

    #[test]
    fn twelve_is_straight_up_and_three_to_the_right() {
        let top = polar(0.0, 10.0);
        let right = polar(0.25, 10.0);
        assert!((top.x - RADIUS).abs() < 1e-4 && (top.y - (RADIUS - 10.0)).abs() < 1e-4);
        assert!((right.x - (RADIUS + 10.0)).abs() < 1e-4 && (right.y - RADIUS).abs() < 1e-4);
    }
}
