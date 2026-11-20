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

use crownui::surfaces::OutputId;

use crate::{protocol::protocol, surfaces::HostOutput};

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

/// The output the launcher is asked to show on, or `None` while it is asked to hide. Each
/// launcher window follows the requests made after it was built.
#[derive(Clone)]
pub struct VisibilityRequests(Arc<watch::Sender<Option<OutputId>>>);

impl Default for VisibilityRequests {
    fn default() -> Self {
        Self(Arc::new(watch::Sender::new(None)))
    }
}

impl VisibilityRequests {
    pub fn show_on(&self, output: OutputId) {
        self.0.send_replace(Some(output));
    }

    pub fn hide(&self) {
        self.0.send_replace(None);
    }

    /// The output last asked to show the launcher, if the last request was to show it.
    pub fn asked_output(&self) -> Option<OutputId> {
        *self.0.borrow()
    }

    /// The requests made after this call, as the window on `output` follows them.
    pub fn subscribe(&self, output: OutputId) -> OutputRequests {
        OutputRequests {
            requests: self.0.subscribe(),
            output,
        }
    }
}

/// Show and hide requests for the launcher window on one output.
pub struct OutputRequests {
    requests: watch::Receiver<Option<OutputId>>,
    output: OutputId,
}

impl OutputRequests {
    /// Waits for the next request and tells whether it shows this window; `None` once no more
    /// requests can come.
    pub async fn next(mut self) -> Option<(Self, bool)> {
        self.requests.changed().await.ok()?;
        let shown = *self.requests.borrow_and_update() == Some(self.output);
        Some((self, shown))
    }
}

/// Show and hide requests for the UI, where it reports back, and where a hidden launcher shows.
pub struct Remote {
    pub requests: VisibilityRequests,
    pub shown: Shown,
    pub host: HostOutput,
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
    let host = HostOutput::default();
    let mut handler = Handler {
        shown: shown.clone(),
        requests: requests.clone(),
        host: host.clone(),
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
    Ok(Remote {
        requests,
        shown,
        host,
    })
}

struct Handler {
    shown: Shown,
    requests: VisibilityRequests,
    host: HostOutput,
}

impl Handler {
    /// Shows a hidden launcher on the host output, leaves a shown one where it is, or hides it.
    fn request(&mut self, show: bool) -> Result<bool, RemoteError> {
        let shown = match (show, self.shown.get(), self.host.get()) {
            (true, true, _) => true,
            (true, false, Some(host)) => {
                self.requests.show_on(host);
                true
            }
            (true, false, None) => false,
            (false, ..) => {
                self.requests.hide();
                false
            }
        };
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
