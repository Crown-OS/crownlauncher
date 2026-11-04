use crownui::prelude::{AnyView, Runtime, Styled, View, bind, hstack, keyed, vstack, zstack};

use super::{
    arrange::{Arrangement, Placed, PlacedRow, arrange, plugin_tiles},
    metrics::{GAP_X, GAP_Y},
    results::results,
    search_bar::search_bar,
    state::Launcher,
    tiles,
};
use crate::settings::Tile;

pub fn arrangement(runtime: &mut Runtime, launcher: Launcher) -> Arrangement {
    let plugins = launcher.tiles.with(runtime, plugin_tiles);
    launcher
        .settings
        .with(runtime, |settings| arrange(&settings.layout, &plugins))
}

/// The home grid, rebuilt only when its shape changes.
pub fn grid(launcher: Launcher) -> impl View + Styled {
    keyed(
        move |runtime| arrangement(runtime, launcher),
        move |_, arrangement| {
            let rows: Vec<AnyView> = arrangement
                .rows
                .iter()
                .map(|row| row_view(launcher, row))
                .collect();
            zstack((
                vstack(rows).gap(GAP_Y),
                results(launcher, arrangement.results),
            ))
            .w(arrangement.size.width)
            .h(arrangement.size.height)
        },
    )
}

fn row_view(launcher: Launcher, row: &PlacedRow) -> AnyView {
    let tiles: Vec<AnyView> = row
        .tiles
        .iter()
        .map(|tile| tile_view(launcher, tile))
        .collect();
    hstack(tiles)
        .gap(GAP_X)
        .h(row.pixel_height())
        .items_center()
        .boxed()
}

fn tile_view(launcher: Launcher, tile: &Placed) -> AnyView {
    let view = match tile {
        Placed::Builtin(Tile::Search) => return search_bar(launcher).boxed(),
        Placed::Builtin(Tile::Weather) => tiles::weather(launcher).boxed(),
        Placed::Builtin(Tile::Shortcuts) => tiles::shortcuts(launcher).boxed(),
        Placed::Builtin(Tile::Clock) => tiles::clock(launcher).boxed(),
        Placed::Builtin(Tile::Events) => tiles::events(launcher).boxed(),
        Placed::Builtin(Tile::Countdown) => tiles::countdown(launcher).boxed(),
        Placed::Builtin(Tile::Pinned) => tiles::pinned(launcher).boxed(),
        Placed::Builtin(Tile::Plugin { .. }) => return vstack(()).boxed(),
        Placed::Plugin {
            plugin,
            widget,
            size,
        } => tiles::remote(launcher, plugin.clone(), widget.clone(), *size).boxed(),
    };
    vstack(view)
        .hidden(bind(move |runtime| launcher.searching(runtime)))
        .boxed()
}
