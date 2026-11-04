mod desktop_id;
mod entry;
mod exec;
mod scan;

use std::sync::Arc;

pub use desktop_id::DesktopId;
pub use entry::AppEntry;
pub use exec::command;
pub use scan::scan;

use crate::{
    search::Query,
    util::{process::spawn_detached, xdg},
};

pub const DESKTOP_EXTENSION: &str = "desktop";
const TERMINAL_LAUNCHER: &str = "xdg-terminal-exec";
/// What matching a generic name or keyword costs against matching the app's own name.
const ALIAS_PENALTY: u32 = 300;

/// The installed applications, shared cheaply between the UI and search.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    apps: Arc<[AppEntry]>,
}

impl Catalog {
    pub fn load() -> Self {
        Self::new(scan(&xdg::application_dirs()))
    }

    /// The apps sorted by name, which is how an empty query lists them.
    pub fn new(mut apps: Vec<AppEntry>) -> Self {
        apps.sort_by_cached_key(|app| app.name.to_lowercase());
        Self { apps: apps.into() }
    }

    pub fn get(&self, id: &str) -> Option<&AppEntry> {
        let wanted = id.strip_suffix(".desktop").unwrap_or(id);
        self.apps.iter().find(|app| app.id.stem() == wanted)
    }

    pub fn apps(&self) -> &[AppEntry] {
        &self.apps
    }

    /// The apps matching `query`, best first, at most `limit`; every app by name when it is empty.
    pub fn search(&self, query: &Query, limit: usize) -> Vec<&AppEntry> {
        if query.is_empty() {
            return self.apps.iter().take(limit).collect();
        }
        let mut scored: Vec<(u32, &AppEntry)> = self
            .apps
            .iter()
            .filter_map(|app| {
                let best = app
                    .keys
                    .iter()
                    .enumerate()
                    .filter_map(|(index, key)| {
                        let penalty = if index == 0 { 0 } else { ALIAS_PENALTY };
                        Some(query.score(key)?.saturating_sub(penalty))
                    })
                    .max()?;
                Some((best, app))
            })
            .collect();
        scored.sort_by(|(left, a), (right, b)| right.cmp(left).then_with(|| a.name.cmp(&b.name)));
        scored.into_iter().take(limit).map(|(_, app)| app).collect()
    }
}

/// Starts `app` detached from the launcher.
pub fn launch(app: &AppEntry) {
    let Some(mut argv) = app.exec.as_deref().and_then(command) else {
        log::warn!("{} has nothing to run", app.path.display());
        return;
    };
    if app.terminal {
        argv.insert(0, TERMINAL_LAUNCHER.to_owned());
    }
    spawn_detached(argv);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::search::SearchKey;

    #[test]
    fn a_name_match_outranks_a_keyword_match() {
        let mut terminal = app("xterm", "XTerm");
        terminal.keys.push(SearchKey::new("console"));
        let catalog = Catalog::new(vec![terminal, app("contacts", "Contacts")]);
        let names: Vec<_> = catalog
            .search(&Query::new("co"), 2)
            .into_iter()
            .map(|app| app.name.as_str())
            .collect();
        assert_eq!(names, ["Contacts", "XTerm"]);
    }

    fn app(id: &str, name: &str) -> AppEntry {
        AppEntry {
            id: DesktopId::new(format!("{id}.desktop")),
            path: PathBuf::from(format!("/apps/{id}.desktop")),
            name: name.to_owned(),
            comment: None,
            icon: None,
            exec: Some(id.to_owned()),
            terminal: false,
            categories: Vec::new(),
            keys: vec![SearchKey::new(name)],
        }
    }

    #[test]
    fn apps_are_found_with_or_without_their_extension() {
        let catalog = Catalog::new(vec![app("org.gnome.Nautilus", "Files")]);
        assert!(catalog.get("org.gnome.Nautilus.desktop").is_some());
        assert!(catalog.get("org.gnome.Nautilus").is_some());
        assert!(catalog.get("org.gnome.Calendar").is_none());
    }

    #[test]
    fn search_ranks_the_closest_names_first_and_honours_the_limit() {
        let catalog = Catalog::new(vec![
            app("firefox", "Firefox"),
            app("files", "Files"),
            app("profiler", "Profiler"),
        ]);
        let names: Vec<_> = catalog
            .search(&Query::new("fi"), 2)
            .into_iter()
            .map(|app| app.name.as_str())
            .collect();
        assert_eq!(names, ["Files", "Firefox"]);
    }

    #[test]
    fn an_empty_query_lists_every_app_by_name() {
        let catalog = Catalog::new(vec![
            app("zed", "Zed"),
            app("blender", "blender"),
            app("atom", "Atom"),
        ]);
        let names: Vec<_> = catalog
            .search(&Query::new("  "), 2)
            .into_iter()
            .map(|app| app.name.as_str())
            .collect();
        assert_eq!(names, ["Atom", "blender"]);
    }
}
