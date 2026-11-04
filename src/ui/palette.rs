use crownui::prelude::{BLACK, Color, WHITE};

const fn rgba(red: u8, green: u8, blue: u8, alpha: f32) -> Color {
    Color::new(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        alpha,
    )
}

const fn ink(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// Every colour the launcher paints, for the desktop's light or dark mode. The inks are
/// translucent so the blurred desktop tints them, as vibrancy does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// The window's glass over the compositor's blur.
    pub glass: Color,
    /// Glass seen through when the compositor cannot blur.
    pub solid_glass: Color,
    pub rim: Color,
    /// The light, or the shade, the glass spills onto the desktop around it.
    pub halo: Color,
    /// The rules under the search field and over the footer.
    pub hairline: Color,
    /// The footer's band.
    pub footer: Color,
    pub placeholder: Color,
    pub caret: Color,
    pub text: Color,
    pub muted: Color,
    pub selected: Color,
    pub keycap: Color,
}

impl Palette {
    pub const LIGHT: Self = Self {
        glass: ink(WHITE, 0.62),
        solid_glass: ink(WHITE, 0.92),
        rim: ink(WHITE, 0.6),
        halo: rgba(18, 30, 60, 0.18),
        hairline: ink(BLACK, 0.08),
        footer: ink(WHITE, 0.32),
        placeholder: ink(BLACK, 0.38),
        caret: ink(BLACK, 0.7),
        text: ink(BLACK, 0.86),
        muted: ink(BLACK, 0.5),
        selected: ink(BLACK, 0.07),
        keycap: ink(BLACK, 0.07),
    };

    pub const DARK: Self = Self {
        glass: rgba(26, 27, 30, 0.72),
        solid_glass: rgba(26, 27, 30, 0.95),
        rim: ink(WHITE, 0.14),
        halo: ink(BLACK, 0.45),
        hairline: ink(WHITE, 0.08),
        footer: ink(BLACK, 0.18),
        placeholder: ink(WHITE, 0.4),
        caret: ink(WHITE, 0.85),
        text: ink(WHITE, 0.94),
        muted: ink(WHITE, 0.5),
        selected: ink(WHITE, 0.08),
        keycap: ink(WHITE, 0.1),
    };

    pub const fn new(dark: bool) -> Self {
        if dark { Self::DARK } else { Self::LIGHT }
    }

    /// Every colour at `presence` of its strength, to fade the window in without drawing it
    /// through an offscreen layer.
    pub fn faded(self, presence: f32) -> Self {
        let fade = |color: Color| Color {
            a: color.a * presence,
            ..color
        };
        Self {
            glass: fade(self.glass),
            solid_glass: fade(self.solid_glass),
            rim: fade(self.rim),
            halo: self.halo,
            hairline: fade(self.hairline),
            footer: fade(self.footer),
            placeholder: fade(self.placeholder),
            caret: fade(self.caret),
            text: fade(self.text),
            muted: fade(self.muted),
            selected: fade(self.selected),
            keycap: fade(self.keycap),
        }
    }
}
