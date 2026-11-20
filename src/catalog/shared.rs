use std::{io, sync::Arc, thread};

use tokio::sync::watch;

use super::Catalog;

/// The installed apps, loaded once for every launcher window and loaded again when one is shown.
#[derive(Clone)]
pub struct SharedCatalog(Arc<watch::Sender<Catalog>>);

impl Default for SharedCatalog {
    fn default() -> Self {
        Self(Arc::new(watch::Sender::new(Catalog::default())))
    }
}

impl SharedCatalog {
    /// Starts loading the installed apps in the background.
    pub fn load() -> io::Result<Self> {
        let shared = Self::default();
        let loader = shared.clone();
        thread::Builder::new()
            .name("crownlauncher-catalog".into())
            .spawn(move || loader.publish(Catalog::load()))?;
        Ok(shared)
    }

    /// Hands `catalog` to every window, unless it is the one they have.
    pub fn publish(&self, catalog: Catalog) {
        self.0.send_if_modified(|current| {
            let changed = *current != catalog;
            if changed {
                *current = catalog;
            }
            changed
        });
    }

    /// The catalog now, and each one published after it.
    pub fn subscribe(&self) -> watch::Receiver<Catalog> {
        let mut catalogs = self.0.subscribe();
        catalogs.mark_changed();
        catalogs
    }
}
