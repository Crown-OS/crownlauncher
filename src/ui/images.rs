//! App icons decoded off the UI thread, once per pixel size, and kept as GPU images.

use crownui::{
    ext::ImageId,
    prelude::{AnyView, Cx, Size, Styled, View, bind, cover_image, keyed, vstack},
};

use super::state::{AppIcon, Launcher};
use crate::{catalog::DesktopId, icon};

/// Starts decoding `wanted` unless it is loaded or on its way.
fn request(cx: &mut Cx, launcher: Launcher, wanted: AppIcon) {
    if !launcher
        .requested
        .update(cx, |requested| requested.insert(wanted.clone()))
    {
        return;
    }
    let icon_name = launcher.catalog.with(cx, |catalog| {
        catalog
            .get(wanted.id.as_str())
            .and_then(|app| app.icon.clone())
    });
    let edge = wanted.edge;
    let decode = async move {
        tokio::task::spawn_blocking(move || icon::load(icon_name.as_deref(), edge))
            .await
            .ok()
            .flatten()
    };
    cx.spawn(decode, move |cx, pixels| {
        let Some(pixels) = pixels else {
            return;
        };
        let image = cx.register_image(pixels.width, pixels.height, pixels.rgba);
        launcher.icons.update(cx, |icons| {
            icons.insert(wanted, image);
        });
    });
}

/// The icon of app `id` drawn `size` logical pixels square, once it is decoded.
pub fn app_icon(launcher: Launcher, size: f32, id: DesktopId) -> impl View + Styled {
    let icon = keyed(
        move |runtime| {
            let wanted = AppIcon {
                id: id.clone(),
                edge: launcher.pixels(runtime, size),
            };
            let image = launcher.icon(runtime, &wanted);
            (wanted, image)
        },
        move |cx, (wanted, image)| match image {
            Some(image) => shown(image, size),
            None => {
                request(cx, launcher, wanted);
                blank(size)
            }
        },
    );
    vstack(icon).opacity(bind(move |runtime| launcher.presence(runtime)))
}

fn shown(image: ImageId, size: f32) -> AnyView {
    cover_image(image, Size::new(size, size), 0.0)
        .size(size)
        .boxed()
}

fn blank(size: f32) -> AnyView {
    vstack(()).size(size).boxed()
}
