//! The launcher window's geometry in logical pixels: Raycast's layout, rounder.

pub const WINDOW_WIDTH: f32 = 750.0;
pub const WINDOW_HEIGHT: f32 = 474.0;
pub const WINDOW_RADIUS: f32 = 26.0;
/// Room around the window on its own surface for the halo the compositor casts and the drop it
/// opens with. The surface is no bigger, so each frame only the window's patch is redrawn.
pub const SURFACE_MARGIN: u32 = 64;
pub const SURFACE_WIDTH: u32 = WINDOW_WIDTH as u32 + SURFACE_MARGIN * 2;
pub const SURFACE_HEIGHT: u32 = WINDOW_HEIGHT as u32 + SURFACE_MARGIN * 2;
pub const RIM: f32 = 1.0;
pub const HAIRLINE: f32 = 1.0;
pub const SEARCH_HEIGHT: f32 = 58.0;
pub const FOOTER_HEIGHT: f32 = 42.0;
pub const LIST_PADDING: f32 = 8.0;
/// What the list's sections and rows have to fit in.
pub const LIST_HEIGHT: f32 =
    WINDOW_HEIGHT - SEARCH_HEIGHT - FOOTER_HEIGHT - HAIRLINE * 2.0 - LIST_PADDING * 2.0;
pub const ROW_RADIUS: f32 = 12.0;
pub const KEYCAP_RADIUS: f32 = 7.0;
