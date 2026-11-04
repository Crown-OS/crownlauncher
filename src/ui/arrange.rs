//! Where every tile goes: the configured rows, plugin tiles nobody placed packed in after them,
//! and the pane the results take while searching.

use crownplugin_proto::{LauncherSize, LauncherSnapshot};
use crownui::prelude::{Rect, Size};

use super::metrics::{COLUMNS, GAP_Y, PANEL_WIDTH, SHORT_ROW, TALL_ROW};
use crate::settings::{Row, RowHeight, Tile};

/// The least room the results get, even with nothing else on the grid.
const MIN_RESULTS: f32 = TALL_ROW + GAP_Y + SHORT_ROW;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placed {
    Builtin(Tile),
    Plugin {
        plugin: String,
        widget: String,
        size: LauncherSize,
    },
}

impl Placed {
    fn columns(&self) -> u8 {
        match self {
            Self::Builtin(tile) => tile.builtin_columns().unwrap_or(1),
            Self::Plugin { size, .. } => size.columns(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedRow {
    pub height: RowHeight,
    pub tiles: Vec<Placed>,
}

impl PlacedRow {
    fn new(height: RowHeight) -> Self {
        Self {
            height,
            tiles: Vec::new(),
        }
    }

    fn columns(&self) -> u8 {
        self.tiles.iter().map(Placed::columns).sum()
    }

    /// Adds `tile` if it fits the row's columns and height.
    fn take(&mut self, tile: Placed) -> bool {
        let tall = match &tile {
            Placed::Plugin { size, .. } => Some(size.is_tall()),
            Placed::Builtin(_) => None,
        };
        let fits_height = tall.is_none_or(|tall| tall == (self.height == RowHeight::Tall));
        let fits = fits_height && self.columns() + tile.columns() <= COLUMNS;
        if fits {
            self.tiles.push(tile);
        }
        fits
    }

    pub fn pixel_height(&self) -> f32 {
        match self.height {
            RowHeight::Short => SHORT_ROW,
            RowHeight::Tall => TALL_ROW,
        }
    }

    fn has_search(&self) -> bool {
        self.tiles.contains(&Placed::Builtin(Tile::Search))
    }
}

/// A tile some plugin draws, without its tree: what the grid's shape depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginTile {
    pub plugin: String,
    pub widget: String,
    pub size: LauncherSize,
}

pub fn plugin_tiles(snapshot: &LauncherSnapshot) -> Vec<PluginTile> {
    snapshot
        .tiles
        .iter()
        .map(|tile| PluginTile {
            plugin: tile.plugin.clone(),
            widget: tile.widget.clone(),
            size: tile.size,
        })
        .collect()
}

/// The grid as it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrangement {
    pub rows: Vec<PlacedRow>,
    pub results: Rect,
    pub size: Size,
}

pub fn arrange(layout: &[Row], plugins: &[PluginTile]) -> Arrangement {
    let size_of = |plugin: &str, widget: &str| {
        plugins
            .iter()
            .find(|tile| tile.plugin == plugin && tile.widget == widget)
            .map(|tile| tile.size)
    };
    let mut placed_plugins = Vec::new();
    let mut rows: Vec<PlacedRow> = layout
        .iter()
        .map(|row| {
            let mut placed = PlacedRow::new(row.height);
            for entry in &row.tiles {
                let tile = match Tile::parse(entry) {
                    Some(Tile::Plugin { plugin, widget }) => {
                        let Some(size) = size_of(&plugin, &widget) else {
                            continue;
                        };
                        placed_plugins.push((plugin.clone(), widget.clone()));
                        Placed::Plugin {
                            plugin,
                            widget,
                            size,
                        }
                    }
                    Some(tile) => Placed::Builtin(tile),
                    None => {
                        log::warn!("launcher.ron: `{entry}` is not a tile");
                        continue;
                    }
                };
                if !placed.take(tile) {
                    log::warn!("launcher.ron: `{entry}` does not fit its row");
                }
            }
            placed
        })
        .filter(|row| !row.tiles.is_empty())
        .collect();
    if !rows.iter().any(PlacedRow::has_search) {
        let mut search = PlacedRow::new(RowHeight::Short);
        search.take(Placed::Builtin(Tile::Search));
        rows.push(search);
    }
    let extra = pack(plugins.iter().filter_map(|tile| {
        let placed = placed_plugins
            .iter()
            .any(|(plugin, widget)| *plugin == tile.plugin && *widget == tile.widget);
        (!placed).then(|| Placed::Plugin {
            plugin: tile.plugin.clone(),
            widget: tile.widget.clone(),
            size: tile.size,
        })
    }));
    let search_last = rows.last().is_some_and(PlacedRow::has_search);
    let at = if search_last {
        rows.len() - 1
    } else {
        rows.len()
    };
    rows.splice(at..at, extra);
    measure(rows)
}

/// Packs tiles into rows of their own height, in order.
fn pack(tiles: impl Iterator<Item = Placed>) -> Vec<PlacedRow> {
    let mut rows: Vec<PlacedRow> = Vec::new();
    for tile in tiles {
        let height = match &tile {
            Placed::Plugin { size, .. } if size.is_tall() => RowHeight::Tall,
            _ => RowHeight::Short,
        };
        let open = rows
            .iter_mut()
            .rev()
            .find(|row| row.height == height && row.columns() + tile.columns() <= COLUMNS);
        match open {
            Some(row) => {
                row.take(tile);
            }
            None => {
                let mut row = PlacedRow::new(height);
                row.take(tile);
                rows.push(row);
            }
        }
    }
    rows
}

fn measure(rows: Vec<PlacedRow>) -> Arrangement {
    let tops: Vec<f32> = rows
        .iter()
        .scan(0.0, |top, row| {
            let this = *top;
            *top += row.pixel_height() + GAP_Y;
            Some(this)
        })
        .collect();
    let height = rows
        .last()
        .zip(tops.last())
        .map_or(0.0, |(row, top)| top + row.pixel_height());
    let search = rows.iter().position(PlacedRow::has_search).unwrap_or(0);
    let search_top = tops.get(search).copied().unwrap_or(0.0);
    let above = search_top - GAP_Y;
    let results = if above >= MIN_RESULTS {
        Rect::from_xywh(0.0, 0.0, PANEL_WIDTH, above)
    } else {
        let top = search_top + SHORT_ROW + GAP_Y;
        Rect::from_xywh(0.0, top, PANEL_WIDTH, (height - top).max(MIN_RESULTS))
    };
    let bottom = results.origin.y + results.size.height;
    Arrangement {
        rows,
        size: Size::new(PANEL_WIDTH, height.max(bottom)),
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::default_layout;

    fn snapshot(tiles: &[(&str, LauncherSize)]) -> Vec<PluginTile> {
        tiles
            .iter()
            .map(|(widget, size)| PluginTile {
                plugin: "dev.test".into(),
                widget: (*widget).into(),
                size: *size,
            })
            .collect()
    }

    #[test]
    fn the_default_grid_matches_the_reference_and_results_fill_the_rows_above_search() {
        let arrangement = arrange(&default_layout(), &[]);
        assert_eq!(arrangement.rows.len(), 3);
        assert_eq!(arrangement.size, Size::new(805.0, 523.0));
        assert_eq!(
            arrangement.results,
            Rect::from_xywh(0.0, 0.0, 805.0, 92.0 + 48.0 + 243.0)
        );
    }

    #[test]
    fn unplaced_plugin_tiles_are_packed_in_above_a_bottom_search_row() {
        let tiles = snapshot(&[
            ("a", LauncherSize::Small),
            ("b", LauncherSize::Wide),
            ("c", LauncherSize::Square),
        ]);
        let arrangement = arrange(&default_layout(), &tiles);
        let heights: Vec<_> = arrangement.rows.iter().map(|row| row.height).collect();
        assert_eq!(
            heights,
            [
                RowHeight::Short,
                RowHeight::Tall,
                RowHeight::Short,
                RowHeight::Tall,
                RowHeight::Short
            ]
        );
        assert_eq!(arrangement.rows[2].tiles.len(), 2, "a and b share a row");
        assert!(arrangement.rows[4].has_search());
    }

    #[test]
    fn a_layout_without_search_gets_one_and_results_go_below_a_lone_row() {
        let layout = vec![Row::new(
            RowHeight::Short,
            &["weather", "bogus", "shortcuts"],
        )];
        let arrangement = arrange(&layout, &[]);
        assert_eq!(arrangement.rows.len(), 2);
        assert!(arrangement.rows[1].has_search());
        assert_eq!(arrangement.results.origin.y, 92.0 + 48.0 + 92.0 + 48.0);
        assert_eq!(arrangement.results.size.height, MIN_RESULTS);
    }

    #[test]
    fn a_plugin_tile_listed_in_a_row_of_the_wrong_height_is_left_out() {
        let tiles = snapshot(&[("tall", LauncherSize::Square)]);
        let layout = vec![Row::new(
            RowHeight::Short,
            &["plugin:dev.test/tall", "search"],
        )];
        let arrangement = arrange(&layout, &tiles);
        assert_eq!(arrangement.rows.len(), 1);
        assert_eq!(arrangement.rows[0].tiles, [Placed::Builtin(Tile::Search)]);
    }
}
