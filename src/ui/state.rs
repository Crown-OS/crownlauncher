use std::collections::{HashMap, HashSet};

use crownplugin_proto::LauncherSnapshot;
use crownui::{
    ext::ImageId,
    kit::{Theme, ThemeMode},
    prelude::{Capabilities, Cx, Runtime, Signal, Store},
};
use jiff::Zoned;
use tokio::sync::mpsc::UnboundedSender;

use super::{motion::Motion, palette::Palette};
use crate::{
    catalog::{Catalog, DesktopId},
    ipc::Shown,
    plugins::PluginCommand,
    search::{Results, Selection},
    settings::LauncherSettings,
    weather::{Condition, Report},
};

/// Below this much revealed the launcher is gone: nothing drawn, no glass, no input.
const HIDDEN: f32 = 0.001;

/// Which picture an image was decoded for, at which pixel size.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Art {
    App(DesktopId, u32),
    Weather(Condition, bool, u32),
}

/// Everything the launcher's views read, as copyable handles.
#[derive(Clone, Copy)]
pub struct Launcher {
    pub settings: Signal<LauncherSettings>,
    pub catalog: Signal<Catalog>,
    pub theme: Signal<Theme>,
    pub capabilities: Signal<Capabilities>,
    pub scale: Signal<f32>,
    pub shown: Signal<bool>,
    pub reveal: Signal<f32>,
    pub motion: Store<Motion>,
    pub query: Signal<String>,
    pub results: Signal<Results>,
    pub selection: Signal<Selection>,
    pub now: Signal<Zoned>,
    pub weather: Signal<Vec<Report>>,
    pub images: Signal<HashMap<Art, ImageId>>,
    pub requested: Store<HashSet<Art>>,
    pub tiles: Signal<LauncherSnapshot>,
    /// Counts crownplugind connections: tree revisions only compare within one.
    pub session: Signal<u64>,
    pub plugins: Store<Option<UnboundedSender<PluginCommand>>>,
    pub report: Store<Shown>,
    pub ticking: Store<bool>,
    /// Bumped by every forecast refresh; a scheduled one that finds it moved on stands down.
    pub forecast: Store<u64>,
}

impl Launcher {
    pub fn new(
        cx: &mut Cx,
        settings: LauncherSettings,
        theme: Signal<Theme>,
        report: Shown,
    ) -> Self {
        let capabilities = cx.platform_capabilities();
        let scale = cx.scale_factor();
        Self {
            settings: cx.signal(settings),
            catalog: cx.signal(Catalog::default()),
            theme,
            capabilities,
            scale,
            shown: cx.signal(false),
            reveal: cx.signal(0.0),
            motion: cx.store(Motion::default()),
            query: cx.signal(String::new()),
            results: cx.signal(Results::default()),
            selection: cx.signal(Selection::default()),
            now: cx.signal(Zoned::now()),
            weather: cx.signal(Vec::new()),
            images: cx.signal(HashMap::new()),
            requested: cx.store(HashSet::new()),
            tiles: cx.signal(LauncherSnapshot::default()),
            session: cx.signal(0),
            plugins: cx.store(None),
            report: cx.store(report),
            ticking: cx.store(false),
            forecast: cx.store(0),
        }
    }

    pub fn palette(self, runtime: &mut Runtime) -> Palette {
        Palette::new(
            self.theme
                .with(runtime, |theme| theme.mode == ThemeMode::Dark),
        )
    }

    /// Whether the compositor blurs what is behind the glass.
    pub fn blurs(self, runtime: &mut Runtime) -> bool {
        self.capabilities.get(runtime).materials
    }

    /// Shown, or still fading out.
    pub fn on_screen(self, runtime: &mut Runtime) -> bool {
        self.shown.get(runtime) || self.reveal.get(runtime) > HIDDEN
    }

    pub fn searching(self, runtime: &mut Runtime) -> bool {
        self.query.with(runtime, |query| !query.trim().is_empty())
    }

    pub fn image(self, runtime: &mut Runtime, art: &Art) -> Option<ImageId> {
        self.images.with(runtime, |images| images.get(art).copied())
    }

    /// Device pixels for `logical` at the current scale.
    pub fn pixels(self, runtime: &mut Runtime, logical: f32) -> u32 {
        (logical * self.scale.get(runtime)).round().max(1.0) as u32
    }

    pub fn send(self, cx: &mut Cx, command: PluginCommand) {
        self.plugins.with(cx, |plugins| {
            if let Some(plugins) = plugins {
                let _ = plugins.send(command);
            }
        });
    }
}
