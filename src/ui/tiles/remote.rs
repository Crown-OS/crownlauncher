//! A plugin's tile: its tree, drawn by crownplugin-render in the kit theme, on the launcher's
//! glass. The view is keyed by the tree's revision, so a new tree is a new view and an unchanged
//! one costs nothing; what the user does goes back tagged with the revision it was done to.

use std::rc::Rc;

use crownplugin_proto::{LauncherSize, LauncherSurface, SurfaceKey};
use crownplugin_render::{EventSink, Resolver, render};
use crownui::prelude::{AnyView, Styled, View, keyed, vstack};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    plugins::PluginCommand,
    ui::{
        glass::{Glass, Showing, glass},
        metrics::{RADIUS, SHORT_ROW, TALL_ROW, span},
        state::Launcher,
    },
};

const PADDING: f32 = 20.0;

struct TileHost {
    plugins: UnboundedSender<PluginCommand>,
    surface: SurfaceKey,
    rev: u32,
}

impl Resolver for TileHost {
    fn events(&self) -> Option<EventSink> {
        let (plugins, surface, rev) = (self.plugins.clone(), self.surface.clone(), self.rev);
        Some(Rc::new(move |event| {
            let _ = plugins.send(PluginCommand::Event {
                surface: surface.clone(),
                rev,
                event,
            });
        }))
    }
}

pub fn size_of(size: LauncherSize) -> (f32, f32) {
    let height = if size.is_tall() { TALL_ROW } else { SHORT_ROW };
    (span(size.columns()), height)
}

pub fn remote(launcher: Launcher, plugin: String, widget: String, size: LauncherSize) -> impl View {
    let (width, height) = size_of(size);
    let body = keyed(
        move |runtime| {
            let session = launcher.session.get(runtime);
            launcher.tiles.with(runtime, |snapshot| {
                snapshot
                    .tiles
                    .iter()
                    .find(|tile| tile.plugin == plugin && tile.widget == widget)
                    .cloned()
                    .map(|tile| (session, tile))
            })
        },
        move |cx, tile: Option<(u64, LauncherSurface)>| -> AnyView {
            let plugins = launcher.plugins.with(cx, Option::clone);
            let (Some((_, tile)), Some(plugins)) = (tile, plugins) else {
                return vstack(()).boxed();
            };
            let host = TileHost {
                plugins,
                surface: tile.key(),
                rev: tile.tree.rev,
            };
            render(cx, &tile.tree, &host)
        },
    )
    .w(width)
    .h(height)
    .p(PADDING)
    .overflow_hidden();
    glass(body, launcher, Glass::Milky, RADIUS, Showing::Home)
}
