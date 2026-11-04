use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

use super::{AppEntry, DESKTOP_EXTENSION, DesktopId, entry::Environment};
use crate::util::locale;

const MAX_DEPTH: usize = 4;

/// Every application shown in launchers, the first directory shadowing the later ones.
pub fn scan(dirs: &[PathBuf]) -> Vec<AppEntry> {
    let locale = locale::current();
    let desktops: Vec<String> = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .filter(|desktop| !desktop.is_empty())
        .map(str::to_owned)
        .collect();
    let environment = Environment {
        locale: locale.as_deref(),
        desktops: &desktops,
    };
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    for dir in dirs {
        collect(dir, dir, 0, &mut files);
    }
    files
        .into_iter()
        .filter(|(id, _)| seen.insert(id.clone()))
        .filter_map(|(id, path)| {
            let bytes = fs::read(&path).ok()?;
            AppEntry::parse(id, path, bytes, &environment)
        })
        .collect()
}

fn collect(root: &Path, dir: &Path, depth: usize, files: &mut Vec<(DesktopId, PathBuf)>) {
    let Ok(listing) = fs::read_dir(dir) else {
        return;
    };
    for path in listing.filter_map(Result::ok).map(|item| item.path()) {
        if path.is_dir() {
            if depth < MAX_DEPTH {
                collect(root, &path, depth + 1, files);
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == DESKTOP_EXTENSION)
            && let Some(id) = path
                .strip_prefix(root)
                .ok()
                .and_then(DesktopId::from_relative)
        {
            files.push((id, path));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Installed(PathBuf);

    impl Installed {
        fn new(name: &str, files: &[(&str, &str)]) -> Self {
            let dir =
                env::temp_dir().join(format!("crownlauncher-scan-{name}-{}", std::process::id()));
            for (file, text) in files {
                let path = dir.join(file);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::write(path, text).ok();
            }
            Self(dir)
        }
    }

    impl Drop for Installed {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).ok();
        }
    }

    const APP: &str = "[Desktop Entry]\nType=Application\nName=App\nExec=app\n";

    #[test]
    fn earlier_directories_shadow_later_ones_and_subdirectories_are_scanned() {
        let user = Installed::new("user", &[("app.desktop", APP)]);
        let system = Installed::new(
            "system",
            &[("app.desktop", APP), ("vendor/tool.desktop", APP)],
        );
        let entries = scan(&[user.0.clone(), system.0.clone()]);
        let ids: Vec<_> = entries.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(entries.len(), 2);
        assert!(ids.contains(&"vendor-tool.desktop"));
        let app = entries
            .iter()
            .find(|entry| entry.id.as_str() == "app.desktop");
        assert_eq!(app.map(|entry| entry.path.starts_with(&user.0)), Some(true));
    }
}
