//! The home grid's geometry in logical pixels, measured off the reference design.

pub const COLUMNS: u8 = 3;
pub const CELL: f32 = 243.0;
pub const GAP_X: f32 = 38.0;
pub const GAP_Y: f32 = 48.0;
pub const SHORT_ROW: f32 = 92.0;
pub const TALL_ROW: f32 = 243.0;
pub const RADIUS: f32 = 28.0;
pub const RIM: f32 = 1.5;
pub const PANEL_WIDTH: f32 = span(COLUMNS);

/// The width of a tile `columns` wide.
pub const fn span(columns: u8) -> f32 {
    let columns = columns as f32;
    CELL * columns + GAP_X * (columns - 1.0)
}
