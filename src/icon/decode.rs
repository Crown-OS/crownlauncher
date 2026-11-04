use std::path::Path;

use anyhow::{Context, Result};
use image::{ImageFormat, RgbaImage, imageops};
use resvg::{tiny_skia, usvg};

use super::Pixels;

pub fn decode(path: &Path, edge: u32) -> Result<Pixels> {
    let bytes = std::fs::read(path).context("read")?;
    let is_svg = path
        .extension()
        .is_some_and(|extension| extension == "svg" || extension == "svgz");
    if is_svg {
        rasterize_svg(&bytes, edge, edge)
    } else {
        resize_raster(&bytes, edge)
    }
}

/// Renders SVG `bytes` centred and fitted into `width` × `height` pixels.
pub fn rasterize_svg(bytes: &[u8], width: u32, height: u32) -> Result<Pixels> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default()).context("parse svg")?;
    let size = tree.size();
    let scale = (width as f32 / size.width().max(1.0)).min(height as f32 / size.height().max(1.0));
    let offset_x = (width as f32 - size.width() * scale) / 2.0;
    let offset_y = (height as f32 - size.height() * scale) / 2.0;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).context("allocate pixmap")?;
    let transform =
        tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let straight = pixel.demultiply();
            [
                straight.red(),
                straight.green(),
                straight.blue(),
                straight.alpha(),
            ]
        })
        .collect();
    Ok(Pixels {
        width,
        height,
        rgba,
    })
}

fn resize_raster(bytes: &[u8], edge: u32) -> Result<Pixels> {
    let source = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .context("decode png")?
        .into_rgba8();
    let (width, height) = source.dimensions();
    let scale = edge as f32 / width.max(height).max(1) as f32;
    let fitted = |length: u32| ((length as f32 * scale).round() as u32).clamp(1, edge);
    let (fit_width, fit_height) = (fitted(width), fitted(height));
    let resized = if (fit_width, fit_height) == (width, height) {
        source
    } else {
        imageops::resize(
            &source,
            fit_width,
            fit_height,
            imageops::FilterType::Triangle,
        )
    };
    let mut canvas = RgbaImage::new(edge, edge);
    imageops::overlay(
        &mut canvas,
        &resized,
        i64::from((edge - fit_width) / 2),
        i64::from((edge - fit_height) / 2),
    );
    Ok(Pixels {
        width: edge,
        height: edge,
        rgba: canvas.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TALL_RED: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="20"><rect width="10" height="20" fill="#ff0000"/></svg>"##;

    #[test]
    fn a_tall_svg_is_centred_in_its_box() {
        let pixels = match rasterize_svg(TALL_RED, 8, 8) {
            Ok(pixels) => pixels,
            Err(error) => panic!("{error:#}"),
        };
        assert_eq!(pixels.rgba.len(), 8 * 8 * 4);
        let alpha = |x: usize, y: usize| pixels.rgba[(y * 8 + x) * 4 + 3];
        assert_eq!(alpha(0, 4), 0);
        assert_eq!(alpha(4, 4), 255);
        assert_eq!(&pixels.rgba[(4 * 8 + 4) * 4..][..3], [255, 0, 0]);
    }
}
