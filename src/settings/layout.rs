use serde::{Deserialize, Serialize};

const PLUGIN_PREFIX: &str = "plugin:";

/// How tall a row of the home grid is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RowHeight {
    /// A strip: weather, app shortcuts, the search field.
    #[default]
    Short,
    /// A square: the clock, the agenda, the countdown.
    Tall,
}

/// One row of the home grid, its tiles named as in `bar.ron`: a built-in's name or
/// `plugin:<plugin id>/<tile id>`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Row {
    pub height: RowHeight,
    pub tiles: Vec<String>,
}

impl Row {
    pub fn new(height: RowHeight, tiles: &[&str]) -> Self {
        Self {
            height,
            tiles: tiles.iter().map(|tile| (*tile).to_owned()).collect(),
        }
    }
}

/// A tile of the home grid, parsed from its entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Weather,
    Shortcuts,
    Clock,
    Events,
    Countdown,
    Pinned,
    Search,
    Plugin { plugin: String, widget: String },
}

impl Tile {
    pub fn parse(entry: &str) -> Option<Self> {
        Some(match entry {
            "weather" => Self::Weather,
            "shortcuts" => Self::Shortcuts,
            "clock" => Self::Clock,
            "events" => Self::Events,
            "countdown" => Self::Countdown,
            "pinned" => Self::Pinned,
            "search" => Self::Search,
            other => {
                let (plugin, widget) = other.strip_prefix(PLUGIN_PREFIX)?.split_once('/')?;
                Self::Plugin {
                    plugin: plugin.to_owned(),
                    widget: widget.to_owned(),
                }
            }
        })
    }

    /// Columns a built-in spans; a plugin tile spans what its manifest declares.
    pub fn builtin_columns(&self) -> Option<u8> {
        match self {
            Self::Weather | Self::Search => Some(2),
            Self::Plugin { .. } => None,
            _ => Some(1),
        }
    }
}

pub fn default_layout() -> Vec<Row> {
    vec![
        Row::new(RowHeight::Short, &["weather", "shortcuts"]),
        Row::new(RowHeight::Tall, &["clock", "events", "countdown"]),
        Row::new(RowHeight::Short, &["pinned", "search"]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_name_builtins_and_plugin_tiles() {
        assert_eq!(Tile::parse("clock"), Some(Tile::Clock));
        assert_eq!(
            Tile::parse("plugin:crown.example.tools/system"),
            Some(Tile::Plugin {
                plugin: "crown.example.tools".into(),
                widget: "system".into()
            })
        );
        assert_eq!(Tile::parse("plugin:no-widget"), None);
        assert_eq!(Tile::parse("calendar"), None);
    }

    #[test]
    fn every_default_row_fills_the_three_columns() {
        for row in default_layout() {
            let columns: u8 = row
                .tiles
                .iter()
                .filter_map(|entry| Tile::parse(entry)?.builtin_columns())
                .sum();
            assert_eq!(columns, 3, "{row:?}");
        }
    }
}
