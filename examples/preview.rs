//! Renders the launcher through Impeller, offscreen, over a wallpaper, to a PNG — the same
//! frame the compositor would blur behind, with the blur done in-app.
//!
//! `cargo run --example preview -- <out.png> [wallpaper.png] [--dark] [--query <text>]`

use std::{
    any::Any,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use anyhow::{Context, Result};
use crownlauncher::{
    catalog::Catalog,
    settings::LauncherSettings,
    ui::{Staging, staged},
    weather::{Condition, Report},
};
use crownui::{
    ext::TextSystem,
    kit,
    prelude::{Size, Styled, View, image, zstack},
};
use impeller::{ImpellerBackend, Offscreen};
use jiff::civil::date;
use runtime::InlineApp;
use taffy_layout::TaffyEngine;
use view::{TaskExecutor, TaskFuture, TaskId};

const SURFACE: Size = Size::new(1022.0, 625.0);
const SCALE: f32 = 2.0;
const SETTLE_ROUNDS: usize = 40;

type Finished = (TaskId, Box<dyn Any + Send>);

/// Runs the launcher's `cx.spawn` futures, which an offscreen app leaves unstarted.
struct Tasks {
    runtime: tokio::runtime::Runtime,
    finished: Sender<Finished>,
}

impl TaskExecutor for Tasks {
    fn spawn(&mut self, id: TaskId, future: TaskFuture) {
        let finished = self.finished.clone();
        self.runtime.spawn(async move {
            let _ = finished.send((id, future.await));
        });
    }

    fn cancel(&mut self, _id: TaskId) {}
}

fn run_tasks(app: &mut InlineApp<ImpellerBackend, TaffyEngine>) -> Result<Receiver<Finished>> {
    let (finished, outputs) = channel();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()?;
    app.ui()
        .set_task_executor(Box::new(Tasks { runtime, finished }));
    Ok(outputs)
}

/// The default apps where they are installed, and otherwise whichever are.
fn installed(wanted: &[String], catalog: &Catalog, count: usize) -> Vec<String> {
    let present: Vec<String> = wanted
        .iter()
        .filter(|id| catalog.get(id).is_some())
        .cloned()
        .collect();
    if present.len() >= count {
        return present;
    }
    catalog
        .apps()
        .iter()
        .filter(|app| app.icon.is_some())
        .map(|app| app.id.as_str().to_owned())
        .take(count)
        .collect()
}

fn report(city: &str, temperature: f32, high: f32, low: f32, condition: Condition) -> Report {
    Report {
        city: city.to_owned(),
        temperature,
        high,
        low,
        condition,
        night: false,
    }
}

fn wallpaper(path: Option<&PathBuf>) -> Result<Option<impl View>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let picture = ::image::open(path)
        .with_context(|| format!("open {}", path.display()))?
        .into_rgba8();
    let (width, height) = picture.dimensions();
    Ok(Some(
        image(width, height, picture.into_raw())
            .absolute()
            .inset(0.0),
    ))
}

fn main() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    let mut positional = Vec::new();
    let (mut dark, mut query) = (false, String::new());
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--dark" => dark = true,
            "--query" => query = arguments.next().unwrap_or_default(),
            _ => positional.push(PathBuf::from(argument)),
        }
    }
    let output = positional
        .first()
        .cloned()
        .unwrap_or_else(|| "launcher.png".into());

    let staging = Staging {
        dark,
        now: date(2026, 5, 24)
            .at(8, 16, 30, 0)
            .in_tz("Europe/Berlin")
            .context("the zone database")?,
        weather: vec![
            report("Munich", 18.0, 26.0, 3.0, Condition::PartlyCloudy),
            report("Paris", 16.0, 24.0, 5.0, Condition::Rain),
        ],
        catalog: Catalog::load(),
        query,
    };
    let mut settings = LauncherSettings::default();
    settings.shortcuts = installed(&settings.shortcuts, &staging.catalog, 2);
    settings.pinned = installed(&settings.pinned, &staging.catalog, 3);
    let view = zstack((wallpaper(positional.get(1))?, staged(settings, staging)));
    let surface = Offscreen {
        width: (SURFACE.width * SCALE) as u32,
        height: (SURFACE.height * SCALE) as u32,
        scale: SCALE,
    };
    let mut app = InlineApp::<ImpellerBackend, TaffyEngine>::with_surface(
        view,
        TaffyEngine::new(),
        SURFACE,
        SCALE,
        &surface,
    )
    .context("creating the offscreen renderer")?;
    app.ui().set_emulate_materials(true);
    app.text_cache()
        .system_mut()
        .register_font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY))?;
    let tasks = run_tasks(&mut app)?;
    for _ in 0..SETTLE_ROUNDS {
        while let Ok((id, output)) = tasks.try_recv() {
            app.ui().complete_task(id, output);
        }
        app.settle()?;
        std::thread::sleep(Duration::from_millis(25));
    }
    let pixels = app.renderer_mut().read_pixels()?;
    std::fs::write(&output, impeller::encode_png(&pixels)?)
        .with_context(|| format!("writing {}", output.display()))?;
    println!("{}", output.display());
    Ok(())
}
