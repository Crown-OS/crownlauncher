use std::{fmt, path::Path, sync::Arc};

use super::DESKTOP_EXTENSION;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DesktopId(Arc<str>);

impl DesktopId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn from_relative(relative: &Path) -> Option<Self> {
        if relative.extension()? != DESKTOP_EXTENSION {
            return None;
        }
        let id = relative
            .components()
            .map(|component| component.as_os_str().to_str())
            .collect::<Option<Vec<_>>>()?
            .join("-");
        Some(Self::new(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn stem(&self) -> &str {
        self.0.strip_suffix(".desktop").unwrap_or(&self.0)
    }
}

impl fmt::Display for DesktopId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_entries_join_their_directories_with_dashes() {
        let id = DesktopId::from_relative(Path::new("kde4/kate.desktop"));
        assert_eq!(id, Some(DesktopId::new("kde4-kate.desktop")));
    }

    #[test]
    fn other_files_have_no_desktop_id() {
        assert_eq!(DesktopId::from_relative(Path::new("mimeinfo.cache")), None);
    }

    #[test]
    fn stem_drops_the_extension() {
        assert_eq!(
            DesktopId::new("org.gnome.Nautilus.desktop").stem(),
            "org.gnome.Nautilus"
        );
    }
}
