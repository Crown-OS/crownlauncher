mod client;

use crownplugin_proto::{HitKey, LauncherSnapshot, SearchResults, SurfaceKey, UiEvent};

pub use client::run;

/// What crownplugind tells the launcher.
#[derive(Debug)]
pub enum PluginEvent {
    /// Every plugin tile, after (re)connecting and whenever one changes.
    Tiles(LauncherSnapshot),
    Answer(SearchResults),
    /// The daemon went away, and its tiles with it.
    Offline,
}

/// What the launcher tells crownplugind.
#[derive(Debug, Clone)]
pub enum PluginCommand {
    Search(String),
    Activate(HitKey),
    Shown(bool),
    /// The user acted on revision `rev` of a tile.
    Event {
        surface: SurfaceKey,
        rev: u32,
        event: UiEvent,
    },
}
