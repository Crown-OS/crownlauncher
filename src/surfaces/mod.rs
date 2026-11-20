//! The launcher's two surfaces, its window and the click catcher under it, kept together on one
//! output.

mod output;

use crownui::{
    AppConfig, Surfaces, kit,
    prelude::{
        Anchor, AnyView, CornerShape, KeyboardInteractivity, Layer, LayerExtent, LayerOptions,
        TRANSPARENT, View,
    },
    surfaces::{Output, Waker},
};
use tokio::sync::mpsc::unbounded_channel;

pub use output::{LauncherOutput, LauncherSeat};

use crate::{
    ipc::{Remote, Shown, VisibilityRequests},
    ui::{
        self, Observers,
        metrics::{SURFACE_HEIGHT, SURFACE_WIDTH},
    },
};

const NAMESPACE: &str = "crownlauncher";
const CATCHER_NAMESPACE: &str = "crownlauncher-catcher";
const FALLBACK_SIZE: (u32, u32) = (1280, 800);

pub struct LauncherSurfaces {
    requests: VisibilityRequests,
    shown: Shown,
    output: LauncherOutput,
}

impl LauncherSurfaces {
    pub fn new(remote: Remote) -> Self {
        Self {
            requests: remote.requests,
            shown: remote.shown,
            output: LauncherOutput::default(),
        }
    }
}

impl Surfaces for LauncherSurfaces {
    fn attach(&mut self, runner_waker: Waker) {
        self.output.attach(runner_waker);
    }

    fn placement(&mut self, output: &Output) -> Option<u64> {
        self.output.hosts(output).then_some(0)
    }

    fn build(&mut self, output: &Output) -> Vec<(AnyView, AppConfig)> {
        let (to_catcher, from_launcher) = unbounded_channel();
        let observers = Observers {
            ipc: self.shown.clone(),
            catcher: Some(to_catcher),
            seat: Some(self.output.seat(output.id)),
        };
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
        let root = ui::root(self.requests.subscribe(), observers, self.shown.get());
        vec![
            (
                ui::catcher(from_launcher, self.requests.clone()).boxed(),
                catcher,
            ),
            (root.boxed(), window),
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
