//! crownplugind over crownos-ipc. The daemon may be absent, start late or restart under the
//! launcher: while it is down there are no plugin answers, and the connection is retried with a
//! growing pause.

use std::time::Duration;

use crownos_ipc::adapter::tokio::AsyncClient;
use crownplugin_proto::{SearchResults, plugind};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::PluginCommand;

const FIRST_RETRY: Duration = Duration::from_secs(1);
const LAST_RETRY: Duration = Duration::from_secs(30);

/// Runs until the launcher drops its command sender or its answer receiver.
pub async fn run(
    answers: UnboundedSender<SearchResults>,
    mut commands: UnboundedReceiver<PluginCommand>,
) {
    let mut retry = FIRST_RETRY;
    loop {
        if let Some(client) = connect().await {
            retry = FIRST_RETRY;
            if !session(&answers, &mut commands, client).await {
                return;
            }
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

async fn connect() -> Option<AsyncClient> {
    let mut client = AsyncClient::new(plugind::Client::connect().ok()?.into_inner()).ok()?;
    client.subscribe::<plugind::SearchAnswered>().await.ok()?;
    Some(client)
}

/// Pumps one connection until it breaks; returns whether the launcher is still there.
async fn session(
    answers: &UnboundedSender<SearchResults>,
    commands: &mut UnboundedReceiver<PluginCommand>,
    mut client: AsyncClient,
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
                let inbox = client.client_mut();
                let delivered = std::iter::from_fn(|| inbox.next_event::<plugind::SearchAnswered>())
                    .all(|answered| answers.send(answered.results).is_ok());
                if !delivered {
                    return false;
                }
            }
        }
    }
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
    }
}
