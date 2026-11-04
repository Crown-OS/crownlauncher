use jiff::{
    ToSpan,
    civil::{Date, DateTime, Time},
};
use serde::{Deserialize, Serialize};

/// How often a date comes round again.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Repeat {
    #[default]
    Never,
    Weekly,
    Monthly,
    Yearly,
}

impl Repeat {
    /// The first time `anchor` falls on or after `from`.
    pub fn next(self, anchor: Date, from: Date) -> Option<Date> {
        if anchor >= from {
            return Some(anchor);
        }
        match self {
            Self::Never => None,
            Self::Weekly => {
                let ahead = (anchor.weekday().to_monday_zero_offset() + 7
                    - from.weekday().to_monday_zero_offset())
                    % 7;
                from.checked_add(i64::from(ahead).days()).ok()
            }
            Self::Monthly => (0..=12).find_map(|months| {
                let month = from.first_of_month().checked_add(months.months()).ok()?;
                let date = Date::new(month.year(), month.month(), anchor.day()).ok()?;
                (date >= from).then_some(date)
            }),
            Self::Yearly => (0..=4).find_map(|years| {
                let year = from.year().checked_add(years)?;
                let date = Date::new(year, anchor.month(), anchor.day()).ok()?;
                (date >= from).then_some(date)
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Time,
    pub end: Time,
}

/// An agenda entry of `launcher.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub title: String,
    pub date: Date,
    #[serde(default)]
    pub repeat: Repeat,
    #[serde(default)]
    pub time: Option<TimeRange>,
}

/// An event's next occurrence, as the agenda shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upcoming {
    pub title: String,
    pub date: Date,
    pub days: i32,
    pub time: Option<TimeRange>,
}

impl Event {
    /// The next occurrence that has not ended by `now`.
    pub fn upcoming(&self, now: DateTime) -> Option<Upcoming> {
        let today = now.date();
        let over_today = self.time.is_some_and(|time| time.end <= now.time());
        let from = if over_today {
            today.tomorrow().ok()?
        } else {
            today
        };
        let date = self.repeat.next(self.date, from)?;
        Some(Upcoming {
            title: self.title.clone(),
            date,
            days: days_between(today, date),
            time: self.time,
        })
    }
}

/// The soonest `limit` occurrences, soonest first.
pub fn agenda(events: &[Event], now: DateTime, limit: usize) -> Vec<Upcoming> {
    let mut upcoming: Vec<Upcoming> = events
        .iter()
        .filter_map(|event| event.upcoming(now))
        .collect();
    upcoming.sort_by_key(|event| (event.date, event.time.map(|time| time.start)));
    upcoming.truncate(limit);
    upcoming
}

/// A day counted down to on its own tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Countdown {
    pub title: String,
    pub date: Date,
    #[serde(default)]
    pub repeat: Repeat,
}

impl Countdown {
    /// The day counted down to and how many days are left, or `None` once it has passed.
    pub fn remaining(&self, today: Date) -> Option<(Date, i32)> {
        let date = self.repeat.next(self.date, today)?;
        Some((date, days_between(today, date)))
    }
}

fn days_between(from: Date, to: Date) -> i32 {
    from.until(to).map_or(0, |span| span.get_days())
}

#[cfg(test)]
mod tests {
    use jiff::civil::{date, time};

    use super::*;

    #[test]
    fn repeats_find_the_next_date() {
        let from = date(2026, 5, 24);
        assert_eq!(Repeat::Never.next(date(2026, 5, 1), from), None);
        assert_eq!(
            Repeat::Weekly.next(date(2026, 5, 6), from),
            Some(date(2026, 5, 27))
        );
        assert_eq!(
            Repeat::Monthly.next(date(2026, 1, 31), from),
            Some(date(2026, 5, 31))
        );
        assert_eq!(
            Repeat::Yearly.next(date(1990, 6, 2), from),
            Some(date(2026, 6, 2))
        );
        assert_eq!(
            Repeat::Yearly.next(date(1988, 2, 29), date(2026, 3, 1)),
            Some(date(2028, 2, 29))
        );
    }

    #[test]
    fn the_agenda_skips_what_is_over_and_counts_the_days() {
        let now = date(2026, 5, 24).at(17, 30, 0, 0);
        let meeting = Event {
            title: "Weekly Meeting".into(),
            date: date(2026, 5, 3),
            repeat: Repeat::Weekly,
            time: Some(TimeRange {
                start: time(17, 0, 0, 0),
                end: time(18, 0, 0, 0),
            }),
        };
        let birthday = Event {
            title: "John's Birthday".into(),
            date: date(1990, 6, 2),
            repeat: Repeat::Yearly,
            time: None,
        };
        let shown = agenda(&[birthday.clone(), meeting.clone()], now, 3);
        assert_eq!(shown[0].title, "Weekly Meeting");
        assert_eq!(shown[0].days, 0);
        assert_eq!((shown[1].date, shown[1].days), (date(2026, 6, 2), 9));

        let later = date(2026, 5, 24).at(18, 0, 0, 0);
        assert_eq!(meeting.upcoming(later).map(|event| event.days), Some(7));
    }

    #[test]
    fn a_countdown_rolls_over_to_next_year() {
        let birthday = Countdown {
            title: "Till Birthday".into(),
            date: date(2026, 8, 18),
            repeat: Repeat::Yearly,
        };
        assert_eq!(
            birthday.remaining(date(2026, 5, 24)),
            Some((date(2026, 8, 18), 86))
        );
        assert_eq!(
            birthday.remaining(date(2026, 8, 19)).map(|(date, _)| date),
            Some(date(2027, 8, 18))
        );
    }
}
