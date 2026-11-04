//! The frosted glass every tile sits on: the compositor blurs and shadows each tile's shape,
//! and the tile lays its own milky or clear tint and bright rim over that.

use crownui::prelude::{
    Color, Material, MaterialBlur, MaterialShadow, Runtime, Styled, TRANSPARENT, Vector, bind,
};

use super::{metrics::RIM, palette::Palette, state::Launcher};

const BLUR_RADIUS: f32 = 30.0;
const VIBRANCY: f32 = 1.7;
const SHADOW_RADIUS: f32 = 28.0;
const SHADOW_OFFSET: f32 = 10.0;

/// When a piece of glass is on screen. The compositor blurs every shape not hidden itself,
/// whatever its parents, so each pane hides on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// The home grid, gone while searching.
    Home,
    /// The results, there only while searching.
    Search,
    /// The search field.
    Always,
}

impl Showing {
    pub fn hidden(self, runtime: &mut Runtime, launcher: Launcher) -> bool {
        if !launcher.on_screen(runtime) {
            return true;
        }
        match self {
            Self::Home => launcher.searching(runtime),
            Self::Search => !launcher.searching(runtime),
            Self::Always => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glass {
    /// Frosted white: the widgets.
    Milky,
    /// Almost clear, the desktop vivid through it: the search field.
    Clear,
}

impl Glass {
    fn fill(self, palette: &Palette, blurred: bool) -> Color {
        match (self, blurred) {
            (Self::Milky, true) => palette.glass,
            (Self::Milky, false) => palette.solid_glass,
            (Self::Clear, true) => palette.clear_glass,
            (Self::Clear, false) => palette.solid_clear,
        }
    }

    fn rim(self, palette: &Palette) -> Color {
        match self {
            Self::Milky => palette.rim,
            Self::Clear => palette.clear_rim,
        }
    }
}

/// The compositor's part at `reveal`: blur and shadow come up with the launcher, squared so they
/// are gone before it is and never frost an empty patch of desktop.
pub fn material(runtime: &mut Runtime, launcher: Launcher) -> Material {
    let strength = launcher.reveal.get(runtime).clamp(0.0, 1.0).powi(2);
    let shadow = launcher.palette(runtime).shadow;
    Material {
        corner_radius: 0.0,
        border_width: 1.0,
        blur: Some(MaterialBlur {
            radius: BLUR_RADIUS * strength,
            tint: TRANSPARENT,
            saturation: 1.0 + (VIBRANCY - 1.0) * strength,
        }),
        shadow: Some(MaterialShadow {
            radius: SHADOW_RADIUS,
            offset: Vector::new(0.0, SHADOW_OFFSET),
            color: Color {
                a: shadow.a * strength,
                ..shadow
            },
        }),
    }
}

/// Puts `element` on glass with corners of `radius`.
pub fn glass<E: Styled>(
    element: E,
    launcher: Launcher,
    kind: Glass,
    radius: f32,
    showing: Showing,
) -> E {
    element
        .hidden(bind(move |runtime| showing.hidden(runtime, launcher)))
        .rounded(radius)
        .border(RIM)
        .border_color(bind(move |runtime| kind.rim(&launcher.palette(runtime))))
        .bg(bind(move |runtime| {
            let blurred = launcher.blurs(runtime);
            kind.fill(&launcher.palette(runtime), blurred)
        }))
        .material(bind(move |runtime| material(runtime, launcher)))
}
