use crownplugin_proto::{HitKey, SearchResults};

use crate::catalog::{AppEntry, DesktopId};

const APPS_SECTION: &str = "Applications";

/// What picking a row does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hit {
    App(DesktopId),
    Plugin(HitKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowIcon {
    App(DesktopId),
    /// A Lucide icon a plugin named.
    Symbol(String),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub hit: Hit,
    pub title: String,
    pub subtitle: String,
    pub icon: RowIcon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// `plugin/provider`, or empty for the applications.
    pub source: String,
    pub name: String,
    pub rows: Vec<Row>,
}

/// Everything found for one query: the applications first, then each plugin provider in the
/// order its first answer arrived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Results {
    pub query: String,
    pub sections: Vec<Section>,
}

impl Results {
    pub fn new(query: &str, apps: &[&AppEntry]) -> Self {
        let rows: Vec<Row> = apps
            .iter()
            .map(|app| Row {
                hit: Hit::App(app.id.clone()),
                title: app.name.clone(),
                subtitle: app.comment.clone().unwrap_or_default(),
                icon: RowIcon::App(app.id.clone()),
            })
            .collect();
        Self {
            query: query.to_owned(),
            sections: (!rows.is_empty())
                .then(|| Section {
                    source: String::new(),
                    name: APPS_SECTION.to_owned(),
                    rows,
                })
                .into_iter()
                .collect(),
        }
    }

    /// Takes a provider's answer if it is for this query; returns whether anything changed.
    pub fn answer(&mut self, answer: SearchResults) -> bool {
        if answer.query != self.query {
            return false;
        }
        let source = format!("{}/{}", answer.plugin, answer.provider);
        let rows: Vec<Row> = answer
            .hits
            .into_iter()
            .map(|hit| Row {
                icon: hit.icon.map_or(RowIcon::Blank, RowIcon::Symbol),
                hit: Hit::Plugin(HitKey {
                    plugin: answer.plugin.clone(),
                    provider: answer.provider.clone(),
                    id: hit.id,
                }),
                title: hit.title,
                subtitle: hit.subtitle,
            })
            .collect();
        let existing = self
            .sections
            .iter()
            .position(|section| section.source == source);
        match (existing, rows.is_empty()) {
            (Some(index), true) => {
                self.sections.remove(index);
            }
            (Some(index), false) if self.sections[index].rows == rows => return false,
            (Some(index), false) => self.sections[index].rows = rows,
            (None, true) => return false,
            (None, false) => self.sections.push(Section {
                source,
                name: answer.name,
                rows,
            }),
        }
        true
    }

    /// The sections cut down to what fits `budget` pixels, each header costing `header` and
    /// each row `row`. Every source gets its first row before any gets a second, so a long list
    /// of apps never crowds out a plugin's answer.
    pub fn fit(&self, budget: f32, header: f32, row: f32) -> Vec<Section> {
        let mut left = budget;
        let mut shown: Vec<usize> = Vec::with_capacity(self.sections.len());
        for _ in &self.sections {
            if left < header + row {
                break;
            }
            left -= header + row;
            shown.push(1);
        }
        let mut grew = true;
        while grew {
            grew = false;
            for (count, section) in shown.iter_mut().zip(&self.sections) {
                if left >= row && *count < section.rows.len() {
                    *count += 1;
                    left -= row;
                    grew = true;
                }
            }
        }
        shown
            .into_iter()
            .zip(&self.sections)
            .map(|(count, section)| Section {
                rows: section.rows[..count].to_vec(),
                ..section.clone()
            })
            .collect()
    }
}

/// The highlighted row of the shown results, by position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Selection(pub usize);

impl Selection {
    pub fn step(self, by: isize, rows: usize) -> Self {
        match rows {
            0 => Self(0),
            rows => Self(self.0.saturating_add_signed(by).min(rows - 1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crownplugin_proto::SearchHit;

    use super::*;

    fn answer(query: &str, provider: &str, titles: &[&str]) -> SearchResults {
        SearchResults {
            plugin: "dev.test".into(),
            provider: provider.into(),
            name: provider.to_uppercase(),
            query: query.into(),
            hits: titles
                .iter()
                .map(|title| SearchHit {
                    id: (*title).into(),
                    title: (*title).into(),
                    subtitle: String::new(),
                    icon: None,
                })
                .collect(),
        }
    }

    #[test]
    fn answers_to_another_query_are_ignored_and_repeats_change_nothing() {
        let mut results = Results::new("2+2", &[]);
        assert!(!results.answer(answer("2+", "calc", &["2"])));
        assert!(results.answer(answer("2+2", "calc", &["4"])));
        assert!(!results.answer(answer("2+2", "calc", &["4"])));
        assert_eq!(results.sections.len(), 1);
        assert!(results.answer(answer("2+2", "calc", &[])));
        assert!(
            results.sections.is_empty(),
            "an empty answer withdraws the section"
        );
    }

    #[test]
    fn fitting_keeps_whole_headers_and_as_many_rows_as_there_is_room_for() {
        let mut results = Results::new("x", &[]);
        results.answer(answer("x", "a", &["1", "2", "3", "4"]));
        results.answer(answer("x", "b", &["5", "6"]));
        let counts = |budget| -> Vec<usize> {
            results
                .fit(budget, 20.0, 20.0)
                .iter()
                .map(|section| section.rows.len())
                .collect()
        };
        assert_eq!(
            counts(100.0),
            [2, 1],
            "both sources show before either grows"
        );
        assert_eq!(counts(200.0), [4, 2]);
        assert_eq!(counts(50.0), [1]);
        assert!(counts(30.0).is_empty());
    }

    #[test]
    fn the_selection_stays_on_the_rows() {
        assert_eq!(Selection(0).step(-1, 3), Selection(0));
        assert_eq!(Selection(1).step(5, 3), Selection(2));
        assert_eq!(Selection(2).step(1, 0), Selection(0));
    }
}
