use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub const FALLBACK: &str = "application-x-executable";

static GTK_THEME: LazyLock<Option<String>> = LazyLock::new(freedesktop_icons::default_theme_gtk);

pub fn find(name: &str, edge: u32) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return path.is_file().then(|| path.to_owned());
    }
    let lookup = freedesktop_icons::lookup(name)
        .with_size(u16::try_from(edge).unwrap_or(u16::MAX))
        .with_cache();
    match GTK_THEME.as_deref() {
        Some(theme) => lookup.with_theme(theme).find(),
        None => lookup.find(),
    }
}
