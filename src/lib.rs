//! A Raycast-style launcher for CrownOS. One resident process keeps its window ready;
//! `crownlauncher` run again only asks it to show or hide, so it opens at once.

pub mod catalog;
pub mod icon;
pub mod ipc;
pub mod plugins;
pub mod protocol;
pub mod search;
mod surfaces;
pub mod ui;
mod util;

use anyhow::{Context, Result};

use surfaces::LauncherSurfaces;
pub use surfaces::window_config;

/// Runs the launcher, shown at once unless `shown` is false: on every output a window-sized
/// surface in the middle, over an invisible full-output one that catches clicks off the window.
pub fn app(shown: bool) -> Result<()> {
    let surfaces = LauncherSurfaces::new(ipc::serve()?, shown)?;
    crownui::run_surfaces(surfaces).context("the launcher stopped")
}
