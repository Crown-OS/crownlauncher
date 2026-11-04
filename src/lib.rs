//! A Raycast-style launcher for CrownOS. One resident process keeps its window ready;
//! `crownlauncher` run again only asks it to show or hide, so it opens at once.

pub mod catalog;
pub mod icon;
pub mod ipc;
pub mod plugins;
pub mod protocol;
pub mod search;
pub mod ui;
mod util;

use anyhow::{Context, Result};
use crownui::{
    AppConfig, kit,
    prelude::{
        Anchor, CornerShape, KeyboardInteractivity, Layer, LayerExtent, LayerOptions, TRANSPARENT,
        View,
    },
};
use tokio::sync::mpsc::unbounded_channel;

use ui::{
    Observers,
    metrics::{SURFACE_HEIGHT, SURFACE_WIDTH},
};

const NAMESPACE: &str = "crownlauncher";
const CATCHER_NAMESPACE: &str = "crownlauncher-catcher";
const FALLBACK_SIZE: (u32, u32) = (1280, 800);

fn surface(title: &str, layer: LayerOptions, size: (u32, u32)) -> AppConfig {
    AppConfig::new(title)
        .app_id(NAMESPACE)
        .size(size.0, size.1)
        .clear_color(TRANSPARENT)
        .accessibility(false)
        .corner_shape(CornerShape::Squircle)
        .font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY))
        .layer(
            layer
                .layer(Layer::Overlay)
                .exclusive_zone(-1)
                .keyboard(KeyboardInteractivity::None),
        )
}

/// Runs the launcher, shown at once unless `shown` is false: a window-sized surface in the
/// middle of the output, over an invisible full-output one that catches clicks off the window.
pub fn app(shown: bool) -> Result<()> {
    let remote = ipc::serve()?;
    let (to_catcher, from_launcher) = unbounded_channel();
    let catcher = surface(
        "CrownLauncher click catcher",
        LayerOptions::new(CATCHER_NAMESPACE, Anchor::ALL),
        FALLBACK_SIZE,
    );
    let window = surface(
        "CrownLauncher",
        LayerOptions::new(NAMESPACE, Anchor::NONE)
            .width(LayerExtent::Fixed(SURFACE_WIDTH))
            .height(LayerExtent::Fixed(SURFACE_HEIGHT)),
        (SURFACE_WIDTH, SURFACE_HEIGHT),
    );
    let observers = Observers {
        ipc: remote.shown,
        catcher: Some(to_catcher),
    };
    crownui::run_windows(vec![
        (
            ui::catcher(from_launcher, remote.requester).boxed(),
            catcher,
        ),
        (ui::root(remote.requests, observers, shown).boxed(), window),
    ])
    .context("the launcher stopped")
}
