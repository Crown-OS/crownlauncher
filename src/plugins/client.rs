//! crownplugind over crownos-ipc. The daemon may be absent, start late or restart under the
//! launcher: while it is down there are no plugin tiles or answers, and the connection is
//! retried with a growing pause. A restarted daemon knows nothing of the last session, so every
//! connection attaches afresh.

use std::time::Duration;

use crownos_ipc::adapter::tokio::AsyncClient;
use crownplugin_proto::plugind;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{PluginCommand, PluginEvent};

const FIRST_RETRY: Duration = Duration::from_secs(1);
const LAST_RETRY: Duration = Duration::from_secs(30);

/// Runs until the launcher drops its command sender.
pub async fn run(
    events: UnboundedSender<PluginEvent>,
    mut commands: UnboundedReceiver<PluginCommand>,
) {
    let mut retry = FIRST_RETRY;
    loop {
        if let Some((client, revision)) = connect(&events).await {
            retry = FIRST_RETRY;
            if !session(&events, &mut commands, client, revision).await {
                return;
            }
        }
        if events.send(PluginEvent::Offline).is_err() {
            return;
        }
        let pause = tokio::time::sleep(retry);
        tokio::pin!(pause);
        loop {
            tokio::select! {
                () = &mut pause => break,
                command = commands.recv() => if command.is_none() { return },
            }
        }
        retry = (retry * 2).min(LAST_RETRY);
    }
}

/// Subscribes before attaching, so no change lands between the two unseen.
async fn connect(events: &UnboundedSender<PluginEvent>) -> Option<(AsyncClient, u64)> {
    let mut client = AsyncClient::new(plugind::Client::connect().ok()?.into_inner()).ok()?;
    client.subscribe::<plugind::LauncherChanged>().await.ok()?;
    client.subscribe::<plugind::SearchAnswered>().await.ok()?;
    let snapshot = client
        .call::<plugind::attach_launcher>(&plugind::attach_launcher {}, Vec::new())
        .await
        .ok()?;
    let revision = snapshot.revision;
    events.send(PluginEvent::Tiles(snapshot)).ok()?;
    Some((client, revision))
}

/// Pumps one connection until it breaks; returns whether the launcher is still there.
async fn session(
    events: &UnboundedSender<PluginEvent>,
    commands: &mut UnboundedReceiver<PluginCommand>,
    mut client: AsyncClient,
    mut revision: u64,
) -> bool {
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(command) => if send(&mut client, command).await.is_err() {
                    return true;
                },
                None => return false,
            },
            pumped = client.pump() => {
                if pumped.is_err() {
                    return true;
                }
                if !drain(events, &mut client, &mut revision) {
                    return false;
                }
            }
        }
    }
}

/// Only the newest snapshot matters, and only if it is newer than the one the launcher holds: a
/// change queued before `attach_launcher` answered may be older.
fn drain(
    events: &UnboundedSender<PluginEvent>,
    client: &mut AsyncClient,
    revision: &mut u64,
) -> bool {
    let inbox = client.client_mut();
    let mut newest = None;
    while let Some(changed) = inbox.next_event::<plugind::LauncherChanged>() {
        newest = Some(changed.snapshot);
    }
    let tiles = newest
        .filter(|snapshot| snapshot.revision > *revision)
        .inspect(|snapshot| *revision = snapshot.revision)
        .map(PluginEvent::Tiles);
    std::iter::from_fn(|| inbox.next_event::<plugind::SearchAnswered>())
        .map(|answered| PluginEvent::Answer(answered.results))
        .chain(tiles)
        .all(|event| events.send(event).is_ok())
}

async fn send(client: &mut AsyncClient, command: PluginCommand) -> Result<(), crownos_ipc::Error> {
    match command {
        PluginCommand::Search(query) => {
            client
                .notify::<plugind::search>(&plugind::search { query }, Vec::new())
                .await
        }
        PluginCommand::Activate(hit) => {
            client
                .notify::<plugind::activate>(&plugind::activate { hit }, Vec::new())
                .await
        }
        PluginCommand::Shown(shown) => {
            client
                .notify::<plugind::launcher_shown>(&plugind::launcher_shown { shown }, Vec::new())
                .await
        }
        PluginCommand::Event {
            surface,
            rev,
            event,
        } => {
            client
                .notify::<plugind::ui_event>(
                    &plugind::ui_event {
                        surface,
                        rev,
                        event,
                    },
                    Vec::new(),
                )
                .await
        }
    }
}
