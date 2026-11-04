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

/// Every colour the launcher paints, for the desktop's light or dark mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    /// Milky glass over the compositor's blur.
    pub glass: Color,
    /// Glass seen through when the compositor cannot blur.
    pub solid_glass: Color,
    pub rim: Color,
    /// The search field: nearly clear glass, so the blurred desktop shows through vivid.
    pub clear_glass: Color,
    pub clear_rim: Color,
    /// The search field when the compositor cannot blur.
    pub solid_clear: Color,
    pub on_clear: Color,
    pub placeholder: Color,
    pub text: Color,
    pub muted: Color,
    pub hairline: Color,
    pub face: Color,
    pub face_band: Color,
    pub hour_tick: Color,
    pub minute_tick: Color,
    pub hand: Color,
    pub second_hand: Color,
    pub label: Color,
    pub accent_start: Color,
    pub accent_end: Color,
    pub accent_rim: Color,
    pub on_accent: Color,
    pub selected: Color,
    pub shadow: Color,
}

impl Palette {
    pub const LIGHT: Self = Self {
        dark: false,
        glass: ink(WHITE, 0.42),
        solid_glass: ink(WHITE, 0.82),
        rim: ink(WHITE, 0.80),
        clear_glass: ink(WHITE, 0.10),
        clear_rim: ink(WHITE, 0.62),
        solid_clear: rgba(40, 132, 226, 0.92),
        on_clear: WHITE,
        placeholder: ink(WHITE, 0.92),
        text: BLACK,
        muted: ink(BLACK, 0.70),
        hairline: ink(BLACK, 0.14),
        face: ink(WHITE, 0.74),
        face_band: rgba(176, 164, 255, 0.34),
        hour_tick: ink(BLACK, 0.72),
        minute_tick: ink(BLACK, 0.36),
        hand: rgba(8, 12, 18, 1.0),
        second_hand: rgba(47, 125, 251, 1.0),
        label: rgba(110, 140, 170, 0.22),
        accent_start: rgba(136, 124, 255, 1.0),
        accent_end: rgba(120, 106, 253, 1.0),
        accent_rim: ink(WHITE, 0.34),
        on_accent: WHITE,
        selected: ink(WHITE, 0.55),
        shadow: rgba(18, 44, 90, 0.16),
    };

    pub const DARK: Self = Self {
        dark: true,
        glass: rgba(30, 32, 40, 0.45),
        solid_glass: rgba(30, 32, 40, 0.88),
        rim: ink(WHITE, 0.16),
        clear_glass: ink(WHITE, 0.06),
        clear_rim: ink(WHITE, 0.24),
        solid_clear: rgba(44, 48, 60, 0.92),
        on_clear: WHITE,
        placeholder: ink(WHITE, 0.78),
        text: ink(WHITE, 0.96),
        muted: ink(WHITE, 0.62),
        hairline: ink(WHITE, 0.14),
        face: rgba(40, 44, 56, 0.84),
        face_band: rgba(150, 140, 255, 0.30),
        hour_tick: ink(WHITE, 0.78),
        minute_tick: ink(WHITE, 0.34),
        hand: ink(WHITE, 0.96),
        second_hand: rgba(64, 140, 255, 1.0),
        label: ink(WHITE, 0.12),
        accent_start: rgba(126, 112, 250, 1.0),
        accent_end: rgba(104, 88, 236, 1.0),
        accent_rim: ink(WHITE, 0.22),
        on_accent: WHITE,
        selected: ink(WHITE, 0.14),
        shadow: ink(BLACK, 0.35),
    };

    pub const fn new(dark: bool) -> Self {
        if dark { Self::DARK } else { Self::LIGHT }
    }
}
