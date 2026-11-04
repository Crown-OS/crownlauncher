mod events;
mod layout;

use jiff::civil::{date, time};
use serde::{Deserialize, Serialize};

pub use events::{Countdown, Event, Repeat, TimeRange, Upcoming, agenda};
pub use layout::{Row, RowHeight, Tile, default_layout};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl City {
    fn new(name: &str, latitude: f64, longitude: f64) -> Self {
        Self {
            name: name.to_owned(),
            latitude,
            longitude,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    #[default]
    Celsius,
    Fahrenheit,
}

/// The analog clock's place: its label and IANA time zone (empty for the system's own).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldClock {
    pub label: String,
    pub time_zone: String,
}

/// `~/.config/crownos/launcher.ron`, followed live.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherSettings {
    /// The home grid, top to bottom. Plugin tiles left out are added below.
    pub layout: Vec<Row>,
    /// The weather tile shows the first two.
    pub weather: Vec<City>,
    pub units: Units,
    /// Minutes between forecasts.
    pub weather_refresh: u64,
    pub clock: WorldClock,
    /// The agenda shows the soonest three.
    pub events: Vec<Event>,
    pub countdown: Countdown,
    /// `.desktop` ids drawn as large icons on their own.
    pub shortcuts: Vec<String>,
    /// `.desktop` ids on the glass tray; a calendar shows today's date.
    pub pinned: Vec<String>,
}

impl LauncherSettings {
    pub const SECTION: &'static str = "launcher";
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            layout: default_layout(),
            weather: vec![
                City::new("Munich", 48.137, 11.575),
                City::new("Paris", 48.857, 2.352),
            ],
            units: Units::Celsius,
            weather_refresh: 30,
            clock: WorldClock {
                label: "Munich".to_owned(),
                time_zone: "Europe/Berlin".to_owned(),
            },
            events: vec![
                Event {
                    title: "Weekly Meeting".to_owned(),
                    date: date(2026, 5, 24),
                    repeat: Repeat::Weekly,
                    time: Some(TimeRange {
                        start: time(17, 0, 0, 0),
                        end: time(18, 0, 0, 0),
                    }),
                },
                Event {
                    title: "John's Birthday".to_owned(),
                    date: date(2026, 6, 2),
                    repeat: Repeat::Yearly,
                    time: None,
                },
                Event {
                    title: "Anniversary".to_owned(),
                    date: date(2026, 6, 9),
                    repeat: Repeat::Yearly,
                    time: None,
                },
            ],
            countdown: Countdown {
                title: "Till Birthday".to_owned(),
                date: date(2026, 8, 18),
                repeat: Repeat::Yearly,
            },
            shortcuts: vec![
                "crownos-myphone.desktop".to_owned(),
                "crownos-messages.desktop".to_owned(),
            ],
            pinned: vec![
                "org.gnome.Calendar.desktop".to_owned(),
                "org.gnome.TextEditor.desktop".to_owned(),
                "org.gnome.Nautilus.desktop".to_owned(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_survive_a_round_trip_through_ron() {
        let defaults = LauncherSettings::default();
        let text = match ron::ser::to_string_pretty(&defaults, ron::ser::PrettyConfig::default()) {
            Ok(text) => text,
            Err(error) => panic!("{error}"),
        };
        let parsed: LauncherSettings = match ron::from_str(&text) {
            Ok(parsed) => parsed,
            Err(error) => panic!("{error}\n{text}"),
        };
        assert_eq!(parsed, defaults);
    }

    #[test]
    fn a_partial_file_keeps_the_other_defaults() {
        let parsed: LauncherSettings = match ron::from_str(
            r#"(shortcuts: ["kitty.desktop"], events: [(title: "Dentist", date: "2026-12-01")])"#,
        ) {
            Ok(parsed) => parsed,
            Err(error) => panic!("{error}"),
        };
        assert_eq!(parsed.shortcuts, ["kitty.desktop"]);
        assert_eq!(parsed.events[0].repeat, Repeat::Never);
        assert_eq!(parsed.layout, default_layout());
    }
}
