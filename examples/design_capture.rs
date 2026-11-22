//! Captures every launcher state for the CrownOS-Design Figma file. Run both appearances with
//! `../crownui/capture/scripts/capture-app.sh .`; output lands in `$CROWN_DESIGN_DIR`.

use std::{
    any::Any,
    cell::Cell,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use capture::{CaptureApp, Carrier};
use crownlauncher::{
    catalog::{Catalog, scan},
    search::{Query, Results, Selection},
    ui::{Launcher, Observers, overlay},
    window_config,
};
use crownplugin_proto::{SearchHit, SearchResults};
use crownui::{kit::provide_configured_theme, prelude::component};
use view::{TaskExecutor, TaskFuture, TaskId};

const APP_RESULTS: usize = 12;
/// How long a capture waits for another icon to finish decoding.
const TASK_GRACE: Duration = Duration::from_millis(400);
/// A HiDPI output, so app icons are decoded sharp enough to scale up in Figma.
const ICON_DENSITY: f32 = 2.0;
const PLUGIN: &str = "crown.example.launcher-tools";

const FIXTURE_APPS: &[(&str, &str, &str, &str)] = &[
    (
        "blender",
        "Blender",
        "3D modeling, animation, rendering and post-production",
        "blender",
    ),
    (
        "org.gnome.Calculator",
        "Calculator",
        "Perform arithmetic, scientific or financial calculations",
        "org.gnome.Calculator",
    ),
    (
        "org.gnome.Calendar",
        "Calendar",
        "Access and manage your calendars",
        "org.gnome.Calendar",
    ),
    (
        "org.gnome.clocks",
        "Clocks",
        "Clocks for world times, plus alarms, stopwatch and a timer",
        "org.gnome.clocks",
    ),
    (
        "discord",
        "Discord",
        "All-in-one voice and text chat",
        "discord",
    ),
    (
        "org.gnome.Nautilus",
        "Files",
        "Access and organize files",
        "org.gnome.Nautilus",
    ),
    ("firefox", "Firefox", "Browse the World Wide Web", "firefox"),
    (
        "gimp",
        "GNU Image Manipulation Program",
        "Create images and edit photographs",
        "gimp",
    ),
    (
        "org.gnome.Loupe",
        "Image Viewer",
        "View images",
        "org.gnome.Loupe",
    ),
    ("inkscape", "Inkscape", "Vector graphics editor", "inkscape"),
    (
        "org.gnome.Maps",
        "Maps",
        "Find places around the world",
        "org.gnome.Maps",
    ),
    (
        "org.gnome.Music",
        "Music",
        "Play and organize your music collection",
        "org.gnome.Music",
    ),
    (
        "com.obsproject.Studio",
        "OBS Studio",
        "Free and open source software for video recording and live streaming",
        "com.obsproject.Studio",
    ),
    (
        "org.gnome.Settings",
        "Settings",
        "Change system settings",
        "org.gnome.Settings",
    ),
    ("spotify", "Spotify", "Music for everyone", "spotify"),
    (
        "steam",
        "Steam",
        "Application for managing and playing games on Steam",
        "steam",
    ),
    (
        "org.gnome.Terminal",
        "Terminal",
        "Use the command line",
        "org.gnome.Terminal",
    ),
    (
        "org.gnome.TextEditor",
        "Text Editor",
        "Edit text files",
        "org.gnome.TextEditor",
    ),
    (
        "thunderbird",
        "Thunderbird",
        "Send and receive mail",
        "thunderbird",
    ),
    (
        "code",
        "Visual Studio Code",
        "Code Editing. Redefined.",
        "visual-studio-code",
    ),
    (
        "vlc",
        "VLC media player",
        "Read, capture, broadcast your multimedia streams",
        "vlc",
    ),
    (
        "org.gnome.Weather",
        "Weather",
        "Show weather conditions and forecast",
        "org.gnome.Weather",
    ),
];

type Finished = (TaskId, Box<dyn Any + Send>);

/// Runs `cx.spawn` futures on a background runtime; long-lived ones (config watchers) never
/// finish, so their outputs are simply never delivered.
struct BackgroundTasks {
    runtime: tokio::runtime::Runtime,
    finished: Sender<Finished>,
}

impl TaskExecutor for BackgroundTasks {
    fn spawn(&mut self, id: TaskId, future: TaskFuture) {
        let finished = self.finished.clone();
        self.runtime.spawn(async move {
            let _ = finished.send((id, future.await));
        });
    }

    fn cancel(&mut self, _id: TaskId) {}
}

fn fixture_catalog() -> Result<Catalog> {
    let dir = std::env::temp_dir().join("crownlauncher-design-fixture/applications");
    std::fs::create_dir_all(&dir).context("creating the fixture app directory")?;
    for (id, name, comment, icon) in FIXTURE_APPS {
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={name}\nComment={comment}\nIcon={icon}\nExec={id}\n"
        );
        std::fs::write(dir.join(format!("{id}.desktop")), entry)
            .context("writing a fixture desktop entry")?;
    }
    Ok(Catalog::new(scan(&[dir])))
}

struct Session {
    app: CaptureApp,
    launcher: Launcher,
    finished: Receiver<Finished>,
}

impl Session {
    fn open(catalog: Catalog) -> Result<Self> {
        let mounted = Rc::new(Cell::new(None));
        let root = component({
            let mounted = Rc::clone(&mounted);
            move |cx| {
                let theme = provide_configured_theme(cx);
                let launcher = Launcher::new(cx, theme, Observers::default());
                let capabilities = launcher.capabilities.get(cx);
                launcher.capabilities.set(
                    cx,
                    crownui::prelude::Capabilities {
                        materials: true,
                        ..capabilities
                    },
                );
                launcher.scale.set(cx, ICON_DENSITY);
                launcher.catalog.set(cx, catalog.clone());
                launcher.shown.set(cx, true);
                launcher.reveal.set(cx, 1.0);
                launcher.settle.set(cx, 1.0);
                mounted.set(Some(launcher));
                overlay(launcher)
            }
        });
        let mut app = capture::mount_app(root, &window_config(), None)?;
        let launcher = mounted
            .get()
            .ok_or_else(|| anyhow!("the launcher did not mount"))?;
        app.ui().set_emulate_materials(true);
        let (sender, finished) = channel();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;
        app.ui().set_task_executor(Box::new(BackgroundTasks {
            runtime,
            finished: sender,
        }));
        let mut session = Self {
            app,
            launcher,
            finished,
        };
        session.search("");
        Ok(session)
    }

    /// Types `query` as the launcher would: the apps re-listed, the top row highlighted.
    fn search(&mut self, query: &str) {
        let launcher = self.launcher;
        let cx = self.app.ui().cx();
        let found = launcher.catalog.with(cx, |catalog| {
            Results::new(query, &catalog.search(&Query::new(query), APP_RESULTS))
        });
        launcher.query.set(cx, query.to_owned());
        launcher.results.set(cx, found);
        launcher.selection.set(cx, Selection::default());
    }

    fn answer(&mut self, provider: &str, name: &str, hit: SearchHit) {
        let launcher = self.launcher;
        let cx = self.app.ui().cx();
        let query = launcher.query.get(cx);
        launcher.results.update(cx, |results| {
            results.answer(SearchResults {
                plugin: PLUGIN.to_owned(),
                provider: provider.to_owned(),
                name: name.to_owned(),
                query,
                hits: vec![hit],
            });
        });
    }

    fn select(&mut self, row: usize) {
        let cx = self.app.ui().cx();
        self.launcher.selection.set(cx, Selection(row));
    }

    fn capture(
        &mut self,
        carrier: &mut Carrier,
        group: &str,
        name: &str,
        description: &str,
    ) -> Result<()> {
        loop {
            self.app.settle();
            match self.finished.recv_timeout(TASK_GRACE) {
                Ok((id, output)) => self.app.ui().complete_task(id, output),
                Err(_) => break,
            }
        }
        self.app.settle();
        carrier.capture_state(&mut self.app, group, name, description)?;
        Ok(())
    }
}

fn hit(id: &str, title: &str, subtitle: &str, icon: &str) -> SearchHit {
    SearchHit {
        id: id.to_owned(),
        title: title.to_owned(),
        subtitle: subtitle.to_owned(),
        icon: Some(icon.to_owned()),
    }
}

fn capture_search_window(carrier: &mut Carrier, catalog: &Catalog) -> Result<()> {
    let mut session = Session::open(catalog.clone())?;
    session.capture(
        carrier,
        "Search window",
        "All apps",
        "Nothing typed: every app by name, the first highlighted",
    )?;
    session.select(3);
    session.capture(
        carrier,
        "Search window",
        "Row highlighted",
        "Arrow keys or the pointer moved the highlight down the list",
    )?;

    session.search("fi");
    session.capture(
        carrier,
        "Search results",
        "App matches",
        "\"fi\" typed: matching apps, best first",
    )?;

    session.search("12*7");
    session.answer(
        "calc",
        "Calculator",
        hit("84", "= 84", "12*7", "calculator"),
    );
    session.capture(
        carrier,
        "Search results",
        "Calculator",
        "A plugin answers an expression; Enter runs the command",
    )?;

    session.search("g crownos");
    session.answer(
        "web",
        "Web",
        hit(
            "crownos",
            "Search the web for \u{201c}crownos\u{201d}",
            "duckduckgo.com",
            "globe",
        ),
    );
    session.capture(
        carrier,
        "Search results",
        "Web search",
        "The g keyword hands the query to the web search provider",
    )?;

    session.search("xylophone");
    session.capture(
        carrier,
        "Empty & error states",
        "No results",
        "Nothing matches the query",
    )?;

    session.search("");
    let cx = session.app.ui().cx();
    session.launcher.reveal.set(cx, 0.5);
    session.launcher.settle.set(cx, 0.35);
    session.capture(
        carrier,
        "Motion",
        "Opening",
        "Mid-opening: glass half faded in, contents still dropping into place",
    )?;
    Ok(())
}

fn capture_without_blur(carrier: &mut Carrier, catalog: &Catalog) -> Result<()> {
    let mut session = Session::open(catalog.clone())?;
    let launcher = session.launcher;
    session.app.ui().set_emulate_materials(false);
    let cx = session.app.ui().cx();
    let capabilities = launcher.capabilities.get(cx);
    launcher.capabilities.set(
        cx,
        crownui::prelude::Capabilities {
            materials: false,
            ..capabilities
        },
    );
    session.capture(
        carrier,
        "Glass",
        "Without blur",
        "The compositor cannot blur: the window is drawn nearly solid",
    )?;
    Ok(())
}

fn capture_no_apps(carrier: &mut Carrier) -> Result<()> {
    let mut session = Session::open(Catalog::default())?;
    session.capture(
        carrier,
        "Empty & error states",
        "No apps",
        "No applications installed: an empty list and no action in the footer",
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let catalog = fixture_catalog()?;
    let mut carrier = Carrier::new(
        "Launcher",
        "crownlauncher: the Raycast-style glass search window (878x602 surface centred on the output), captured headlessly from the current source.",
    );
    capture_search_window(&mut carrier, &catalog)?;
    capture_without_blur(&mut carrier, &catalog)?;
    capture_no_apps(&mut carrier)?;
    let path: PathBuf = carrier.write(None)?;
    println!("{}", path.display());
    capture::finish()
}
