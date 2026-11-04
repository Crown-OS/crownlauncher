//! The frosted glass the window sits on: the compositor blurs its shape and casts a soft halo
//! around it, and the launcher lays its tint and bright rim over that.

use crownui::prelude::{
    Color, Material, MaterialBlur, MaterialShadow, Runtime, Styled, TRANSPARENT, Vector, bind,
};

use super::{metrics::RIM, state::Launcher};

const BLUR_RADIUS: f32 = 30.0;
const VIBRANCY: f32 = 1.7;
const HALO_RADIUS: f32 = 30.0;
const HALO_OFFSET: f32 = 12.0;

/// The compositor's part at `reveal`: blur and halo come up with the launcher, squared so they
/// are gone before it is and never frost an empty patch of desktop.
fn material(runtime: &mut Runtime, launcher: Launcher) -> Material {
    let strength = launcher.reveal.get(runtime).clamp(0.0, 1.0).powi(2);
    let halo = launcher.palette(runtime).halo;
    Material {
        corner_radius: 0.0,
        border_width: RIM,
        blur: Some(MaterialBlur {
            radius: BLUR_RADIUS * strength,
            tint: TRANSPARENT,
            saturation: 1.0 + (VIBRANCY - 1.0) * strength,
        }),
        shadow: Some(MaterialShadow {
            radius: HALO_RADIUS,
            offset: Vector::new(0.0, HALO_OFFSET),
            color: Color {
                a: halo.a * strength,
                ..halo
            },
        }),
    }
}

/// Puts `element` on glass with corners of `radius`. The compositor blurs every shape not hidden
/// itself, whatever its parents, so the glass hides on its own once the launcher is gone.
pub fn glass<E: Styled>(element: E, launcher: Launcher, radius: f32) -> E {
    element
        .hidden(bind(move |runtime| !launcher.on_screen(runtime)))
        .rounded(radius)
        .border(RIM)
        .border_color(bind(move |runtime| launcher.palette(runtime).rim))
        .bg(bind(move |runtime| {
            let palette = launcher.palette(runtime);
            if launcher.blurs(runtime) {
                palette.glass
            } else {
                palette.solid_glass
            }
        }))
        .material(bind(move |runtime| material(runtime, launcher)))
}
