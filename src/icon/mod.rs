mod decode;
mod lookup;

/// Straight-alpha RGBA, ready for `Cx::register_image`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// The themed icon `name` (or the generic program icon) as an `edge`-pixel square.
pub fn load(name: Option<&str>, edge: u32) -> Option<Pixels> {
    let file = name
        .and_then(|name| lookup::find(name, edge))
        .or_else(|| lookup::find(lookup::FALLBACK, edge))?;
    decode::decode(&file, edge)
        .inspect_err(|error| log::warn!("icon {}: {error:#}", file.display()))
        .ok()
}
