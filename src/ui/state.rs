use std::collections::{HashMap, HashSet};

use crownui::{
    ext::ImageId,
    kit::{Theme, ThemeMode},
    prelude::{Capabilities, Cx, Runtime, Signal, Store},
};
use tokio::sync::mpsc::UnboundedSender;

use super::{
    motion::{Motion, opacity},
    palette::Palette,
    visibility::Observers,
};
use crate::{
    catalog::{Catalog, DesktopId},
    plugins::PluginCommand,
    search::{Results, Selection},
};

/// Below this much revealed the launcher is gone: nothing drawn, no glass, no input.
const HIDDEN: f32 = 0.001;

/// An app's icon, decoded `edge` pixels square.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppIcon {
    pub id: DesktopId,
    pub edge: u32,
}

/// Everything the launcher's views read, as copyable handles.
#[derive(Clone, Copy)]
pub struct Launcher {
    pub catalog: Signal<Catalog>,
    pub theme: Signal<Theme>,
    pub capabilities: Signal<Capabilities>,
    pub scale: Signal<f32>,
    pub shown: Signal<bool>,
    /// How far the glass has faded in, 0 to 1.
    pub reveal: Signal<f32>,
    /// How far the window has dropped into place, 0 to 1 and a little past while it bounces.
    pub settle: Signal<f32>,
    pub motion: Store<Motion>,
    pub query: Signal<String>,
    pub results: Signal<Results>,
    pub selection: Signal<Selection>,
    pub icons: Signal<HashMap<AppIcon, ImageId>>,
    pub requested: Store<HashSet<AppIcon>>,
    pub plugins: Store<Option<UnboundedSender<PluginCommand>>>,
    pub observers: Store<Observers>,
}

impl Launcher {
    pub fn new(cx: &mut Cx, theme: Signal<Theme>, observers: Observers) -> Self {
        let capabilities = cx.platform_capabilities();
        let scale = cx.scale_factor();
        Self {
            catalog: cx.signal(Catalog::default()),
            theme,
            capabilities,
            scale,
            shown: cx.signal(false),
            reveal: cx.signal(0.0),
            settle: cx.signal(0.0),
            motion: cx.store(Motion::default()),
            query: cx.signal(String::new()),
            results: cx.signal(Results::default()),
            selection: cx.signal(Selection::default()),
            icons: cx.signal(HashMap::new()),
            requested: cx.store(HashSet::new()),
            plugins: cx.store(None),
            observers: cx.store(observers),
        }
    }

    /// The colours to paint with now, faded as far as the window has faded in.
    pub fn palette(self, runtime: &mut Runtime) -> Palette {
        let dark = self
            .theme
            .with(runtime, |theme| theme.mode == ThemeMode::Dark);
        Palette::new(dark).faded(self.presence(runtime))
    }

    /// How far the window has faded in, at full strength once nearly there.
    pub fn presence(self, runtime: &mut Runtime) -> f32 {
        opacity(self.reveal.get(runtime))
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

    pub fn icon(self, runtime: &mut Runtime, icon: &AppIcon) -> Option<ImageId> {
        self.icons.with(runtime, |icons| icons.get(icon).copied())
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
