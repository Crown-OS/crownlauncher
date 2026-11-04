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
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::protocol::protocol;

const FORWARD_TIMEOUT: Duration = Duration::from_secs(2);

/// Whether the launcher is shown, as the UI last reported it.
#[derive(Clone, Default)]
pub struct Shown(Arc<AtomicBool>);

impl Shown {
    pub fn set(&self, shown: bool) {
        self.0.store(shown, Ordering::Relaxed);
    }

    fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Show (`true`) and hide (`false`) requests for the UI, and where it reports back.
pub struct Remote {
    pub requests: UnboundedReceiver<bool>,
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
    let (sender, requests) = unbounded_channel();
    let shown = Shown::default();
    let mut handler = Handler {
        shown: shown.clone(),
        sender,
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
    sender: UnboundedSender<bool>,
}

impl Handler {
    fn request(&mut self, shown: bool) -> Result<bool, RemoteError> {
        self.sender
            .send(shown)
            .map_err(|_| RemoteError::handler("the launcher is shutting down"))?;
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
