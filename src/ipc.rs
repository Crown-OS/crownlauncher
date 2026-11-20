//! The running launcher, reachable as the `crownlauncher` crownos-ipc service: a keybinding
//! runs `crownlauncher` again, which only asks this one to show or hide.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result};
use crownos_ipc::{RemoteError, ServiceBuilder, blocking};
use tokio::sync::watch;

use crate::protocol::protocol;

const FORWARD_TIMEOUT: Duration = Duration::from_secs(2);

/// Whether the launcher is shown, as the UI last reported it.
#[derive(Clone, Default)]
pub struct Shown(Arc<AtomicBool>);

impl Shown {
    pub fn set(&self, shown: bool) {
        self.0.store(shown, Ordering::Relaxed);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Show (`true`) and hide (`false`) requests, which each launcher window built after another
/// follows from then on.
#[derive(Clone)]
pub struct VisibilityRequests(Arc<watch::Sender<bool>>);

impl Default for VisibilityRequests {
    fn default() -> Self {
        Self(Arc::new(watch::Sender::new(false)))
    }
}

impl VisibilityRequests {
    pub fn ask(&self, shown: bool) {
        self.0.send_replace(shown);
    }

    /// The requests made after this call.
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.0.subscribe()
    }
}

/// Show and hide requests for the UI, and where it reports back.
pub struct Remote {
    pub requests: VisibilityRequests,
    pub shown: Shown,
}

/// What another invocation asks of the running launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Show,
    Hide,
    Toggle,
}

pub fn is_running() -> bool {
    protocol::Client::connect().is_ok()
}

/// Hands `request` to a launcher that is already running; `false` when there is none.
pub fn forward(request: Request) -> bool {
    let Ok(mut client) = protocol::Client::connect() else {
        return false;
    };
    let pending = match request {
        Request::Show => client.show(),
        Request::Hide => client.hide(),
        Request::Toggle => client.toggle(),
    };
    pending
        .and_then(|pending| client.wait(pending, Some(FORWARD_TIMEOUT)))
        .is_ok()
}

pub fn serve() -> Result<Remote> {
    let mut server = ServiceBuilder::new(protocol::SERVICE)
        .build()
        .context("bind the crownlauncher socket; is another launcher starting?")?;
    let requests = VisibilityRequests::default();
    let shown = Shown::default();
    let mut handler = Handler {
        shown: shown.clone(),
        requests: requests.clone(),
    };
    std::thread::Builder::new()
        .name("crownlauncher-ipc".into())
        .spawn(move || {
            let served = blocking::serve(&mut server, |server, peer, message| {
                protocol::dispatch(&mut handler, server, peer, message).map(|()| true)
            });
            if let Err(error) = served {
                log::error!("the launcher's IPC stopped: {error}");
            }
        })
        .context("start the IPC thread")?;
    Ok(Remote { requests, shown })
}

struct Handler {
    shown: Shown,
    requests: VisibilityRequests,
}

impl Handler {
    fn request(&mut self, shown: bool) -> Result<bool, RemoteError> {
        self.requests.ask(shown);
        self.shown.set(shown);
        Ok(shown)
    }
}

impl protocol::Handler for Handler {
    fn show(&mut self) -> Result<bool, RemoteError> {
        self.request(true)
    }

    fn hide(&mut self) -> Result<bool, RemoteError> {
        self.request(false)
    }

    fn toggle(&mut self) -> Result<bool, RemoteError> {
        let shown = !self.shown.get();
        self.request(shown)
    }
}
