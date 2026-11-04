//! A Spotlight-style launcher for CrownOS. One resident process keeps a full-screen overlay
//! ready; `crownlauncher` run again only asks it to show or hide, so it opens at once.

pub mod catalog;
pub mod icon;
pub mod ipc;
pub mod plugins;
pub mod protocol;
pub mod search;
pub mod settings;
pub mod ui;
mod util;
pub mod weather;

use anyhow::{Context, Result};
use crownui::{
    AppConfig, kit,
    prelude::{Anchor, KeyboardInteractivity, Layer, LayerOptions, TRANSPARENT},
};

use settings::LauncherSettings;

const NAMESPACE: &str = "crownlauncher";
const FALLBACK_SIZE: (u32, u32) = (1280, 800);

/// Runs the launcher, shown at once unless `shown` is false.
pub fn app(shown: bool) -> Result<()> {
    let remote = ipc::serve()?;
    let settings: LauncherSettings = crownconfig::load_section(LauncherSettings::SECTION);
    let layer = LayerOptions::new(NAMESPACE, Anchor::ALL)
        .layer(Layer::Overlay)
        .exclusive_zone(-1)
        .keyboard(KeyboardInteractivity::None);
    let config = AppConfig::new("CrownLauncher")
        .app_id(NAMESPACE)
        .size(FALLBACK_SIZE.0, FALLBACK_SIZE.1)
        .clear_color(TRANSPARENT)
        .accessibility(false)
        .font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY))
        .layer(layer);
    crownui::run(ui::root(settings, remote, shown), config).context("the launcher stopped")
}
