//! Everything the launcher keeps up with: requests to show or hide, crownplugind, its own
//! settings, the forecast, the clock and the installed apps.

use std::time::Duration;

use crownui::prelude::Cx;
use jiff::Zoned;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::{state::Launcher, visibility};
use crate::{
    catalog::Catalog,
    plugins::{self, PluginEvent},
    settings::LauncherSettings,
    weather,
};

const MIN_WEATHER_REFRESH: u64 = 5;

pub fn start(cx: &mut Cx, launcher: Launcher, requests: UnboundedReceiver<bool>, shown: bool) {
    remote(cx, launcher, requests);
    connect_plugins(cx, launcher);
    rescan(cx, launcher);
    refresh_weather(cx, launcher);
    let initial = launcher.settings.get(cx);
    // Watching starts a thread, which must wait until `crownui::run` has set up the GPU driver
    // through the process environment.
    cx.request_animation_frame(move |cx, _| {
        settings(
            cx,
            launcher,
            crownconfig::watch_section(LauncherSettings::SECTION, initial),
        );
        if shown {
            visibility::set(cx, launcher, true);
        }
    });
}

/// What showing brings up to date.
pub fn on_show(cx: &mut Cx, launcher: Launcher) {
    rescan(cx, launcher);
    let idle = launcher
        .ticking
        .update(cx, |ticking| !std::mem::replace(ticking, true));
    if idle {
        tick(cx, launcher);
    }
}

fn remote(cx: &mut Cx, launcher: Launcher, mut requests: UnboundedReceiver<bool>) {
    let next = async move {
        let request = requests.recv().await;
        (requests, request)
    };
    cx.spawn(next, move |cx, (requests, request)| {
        let Some(shown) = request else {
            return;
        };
        visibility::set(cx, launcher, shown);
        remote(cx, launcher, requests);
    });
}

fn connect_plugins(cx: &mut Cx, launcher: Launcher) {
    let (events, inbox) = unbounded_channel();
    let (commands, outbox) = unbounded_channel();
    launcher
        .plugins
        .update(cx, |plugins| *plugins = Some(commands));
    cx.spawn(plugins::run(events, outbox), |_, ()| {});
    follow_plugins(cx, launcher, inbox);
}

fn follow_plugins(cx: &mut Cx, launcher: Launcher, mut inbox: UnboundedReceiver<PluginEvent>) {
    let next = async move {
        let event = inbox.recv().await;
        (inbox, event)
    };
    cx.spawn(next, move |cx, (inbox, event)| {
        let Some(event) = event else {
            return;
        };
        match event {
            PluginEvent::Tiles(snapshot) => launcher.tiles.set(cx, snapshot),
            PluginEvent::Answer(answer) => {
                let mut results = launcher.results.get(cx);
                if results.answer(answer) {
                    launcher.results.set(cx, results);
                }
            }
            PluginEvent::Offline => {
                launcher.tiles.set(cx, Default::default());
                launcher.session.update(cx, |session| *session += 1);
            }
        }
        follow_plugins(cx, launcher, inbox);
    });
}

fn settings(
    cx: &mut Cx,
    launcher: Launcher,
    mut watch: crownconfig::SectionWatch<LauncherSettings>,
) {
    let next = async move {
        let settings = watch.changed().await;
        (watch, settings)
    };
    cx.spawn(next, move |cx, (watch, settings)| {
        let weather_changed = launcher.settings.with(cx, |current| {
            current.weather != settings.weather || current.units != settings.units
        });
        launcher.settings.set(cx, settings);
        if weather_changed {
            refresh_weather(cx, launcher);
        }
        self::settings(cx, launcher, watch);
    });
}

fn rescan(cx: &mut Cx, launcher: Launcher) {
    let scan = async { tokio::task::spawn_blocking(Catalog::load).await.ok() };
    cx.spawn(scan, move |cx, catalog| {
        if let Some(catalog) = catalog
            && launcher.catalog.with(cx, |current| *current != catalog)
        {
            launcher.catalog.set(cx, catalog);
        }
    });
}

/// Fetches the forecast now and again every `weather_refresh` minutes.
fn refresh_weather(cx: &mut Cx, launcher: Launcher) {
    let (cities, units, every) = launcher.settings.with(cx, |settings| {
        (
            settings.weather.clone(),
            settings.units,
            settings.weather_refresh.max(MIN_WEATHER_REFRESH),
        )
    });
    let generation = launcher.forecast.update(cx, |generation| {
        *generation += 1;
        *generation
    });
    let fetch = async move {
        tokio::task::spawn_blocking(move || weather::forecasts(&cities, units))
            .await
            .unwrap_or_default()
    };
    cx.spawn(fetch, move |cx, reports| {
        if !reports.is_empty() {
            launcher.weather.set(cx, reports);
        }
        cx.spawn(sleep(Duration::from_secs(every * 60)), move |cx, ()| {
            if launcher.forecast.with(cx, |current| *current == generation) {
                refresh_weather(cx, launcher);
            }
        });
    });
}

/// Moves the clock on to the next whole second while the launcher is shown.
fn tick(cx: &mut Cx, launcher: Launcher) {
    let now = Zoned::now();
    let to_next_second =
        Duration::from_nanos(1_000_000_000 - u64::from(now.subsec_nanosecond().unsigned_abs()));
    launcher.now.set(cx, now);
    if !launcher.shown.get(cx) {
        launcher.ticking.update(cx, |ticking| *ticking = false);
        return;
    }
    cx.spawn(sleep(to_next_second), move |cx, ()| tick(cx, launcher));
}

/// A pause made on the task runtime: tokio's timers can only be created inside it.
async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
