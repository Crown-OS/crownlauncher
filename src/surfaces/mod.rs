//! The launcher on every output, built once: its window and the click catcher under it, hidden
//! until asked to show. All of them share the catalog and the crownplugind connection; only the
//! one on the output asked to show it comes on screen.

mod host_output;

use anyhow::{Context, Result};
use crownui::{
    AppConfig, Surfaces, kit,
    prelude::{
        Anchor, AnyView, CornerShape, KeyboardInteractivity, Layer, LayerExtent, LayerOptions,
        TRANSPARENT, View,
    },
    surfaces::Output,
};
use tokio::sync::mpsc::unbounded_channel;

pub use host_output::HostOutput;

use crate::{
    catalog::SharedCatalog,
    ipc::{Remote, Shown, VisibilityRequests},
    plugins::{PluginCommand, PluginLink},
    ui::{
        self, Links, Observers,
        metrics::{SURFACE_HEIGHT, SURFACE_WIDTH},
    },
};

const NAMESPACE: &str = "crownlauncher";
const CATCHER_NAMESPACE: &str = "crownlauncher-catcher";
const FALLBACK_SIZE: (u32, u32) = (1280, 800);
/// Every output keeps its launcher whatever changes, so nothing is built twice.
const EVERY_OUTPUT: u64 = 0;

pub struct LauncherSurfaces {
    requests: VisibilityRequests,
    shown: Shown,
    host: HostOutput,
    show_at_start: bool,
    catalog: SharedCatalog,
    plugins: PluginLink,
}

impl LauncherSurfaces {
    /// Starts loading the catalog and connecting to crownplugind; the launcher shows on the host
    /// output as soon as there is one when `show_at_start` is set.
    pub fn new(remote: Remote, show_at_start: bool) -> Result<Self> {
        Ok(Self {
            requests: remote.requests,
            shown: remote.shown,
            host: remote.host,
            show_at_start,
            catalog: SharedCatalog::load().context("start loading the installed apps")?,
            plugins: PluginLink::start().context("start the crownplugind connection")?,
        })
    }

    /// A launcher shown on an output that went is hidden, so the next toggle shows it again.
    fn hide_if_output_gone(&self, outputs: &[Output]) {
        let gone = self
            .requests
            .asked_output()
            .is_some_and(|asked| outputs.iter().all(|output| output.id != asked));
        if gone && self.shown.get() {
            self.requests.hide();
            self.shown.set(false);
            self.plugins.send(PluginCommand::Shown(false));
        }
    }
}

impl Surfaces for LauncherSurfaces {
    fn outputs(&mut self, outputs: &[Output]) {
        self.host.follow(outputs);
        self.hide_if_output_gone(outputs);
        if let Some(host) = self.host.get().filter(|_| self.show_at_start) {
            self.show_at_start = false;
            self.requests.show_on(host);
            self.shown.set(true);
        }
    }

    fn placement(&mut self, _output: &Output) -> Option<u64> {
        Some(EVERY_OUTPUT)
    }

    fn build(&mut self, output: &Output) -> Vec<(AnyView, AppConfig)> {
        let (to_catcher, from_launcher) = unbounded_channel();
        let observers = Observers {
            ipc: self.shown.clone(),
            catcher: Some(to_catcher),
        };
        let links = Links {
            requests: self.requests.subscribe(output.id),
            catalog: self.catalog.clone(),
            plugins: self.plugins.clone(),
        };
        let shown = self.shown.get() && self.requests.asked_output() == Some(output.id);
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
        vec![
            (
                ui::catcher(from_launcher, self.requests.clone()).boxed(),
                catcher,
            ),
            (ui::root(links, observers, shown).boxed(), window),
        ]
    }
}

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
