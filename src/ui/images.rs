//! Pictures decoded off the UI thread, once per pixel size, and kept as GPU images.

use crownui::{
    ext::ImageId,
    prelude::{AnyView, Cx, Runtime, Size, Styled, View, cover_image, keyed, vstack},
};

use super::state::{Art, Launcher};
use crate::{icon, weather};

/// Starts decoding `art` unless it is loaded or on its way.
pub fn request(cx: &mut Cx, launcher: Launcher, art: Art) {
    if !launcher
        .requested
        .update(cx, |requested| requested.insert(art.clone()))
    {
        return;
    }
    let icon_name = match &art {
        Art::App(id, _) => launcher.catalog.with(cx, |catalog| {
            catalog.get(id.as_str()).and_then(|app| app.icon.clone())
        }),
        Art::Weather(..) => None,
    };
    let job = art.clone();
    let decode = async move {
        tokio::task::spawn_blocking(move || match job {
            Art::App(_, edge) => icon::load(icon_name.as_deref(), edge),
            Art::Weather(condition, night, edge) => {
                icon::rasterize_svg(weather::svg(condition, night).as_bytes(), edge, edge)
                    .inspect_err(|error| log::warn!("weather art: {error:#}"))
                    .ok()
            }
        })
        .await
        .ok()
        .flatten()
    };
    cx.spawn(decode, move |cx, pixels| {
        let Some(pixels) = pixels else {
            return;
        };
        let image = cx.register_image(pixels.width, pixels.height, pixels.rgba);
        launcher.images.update(cx, |images| {
            images.insert(art, image);
        });
    });
}

/// `art` drawn `size` logical pixels square, once it is decoded.
pub fn picture(
    launcher: Launcher,
    size: f32,
    art: impl Fn(&mut Runtime, u32) -> Option<Art> + 'static,
) -> impl View + Styled {
    keyed(
        move |runtime| {
            let edge = launcher.pixels(runtime, size);
            let art = art(runtime, edge)?;
            Some((art.clone(), launcher.image(runtime, &art)))
        },
        move |cx, wanted| match wanted {
            Some((_, Some(image))) => shown(image, size),
            Some((art, None)) => {
                request(cx, launcher, art);
                blank(size)
            }
            None => blank(size),
        },
    )
}

fn shown(image: ImageId, size: f32) -> AnyView {
    cover_image(image, Size::new(size, size), 0.0)
        .size(size)
        .boxed()
}

fn blank(size: f32) -> AnyView {
    vstack(()).size(size).boxed()
}
