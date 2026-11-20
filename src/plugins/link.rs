use std::{io, thread};

use crownplugin_proto::SearchResults;
use tokio::sync::{
    broadcast,
    mpsc::{UnboundedSender, unbounded_channel},
};

use super::{PluginCommand, run};

/// How many answers a window may fall behind before it misses some.
const ANSWER_BACKLOG: usize = 64;

/// One crownplugind connection for every launcher window: commands from any of them, answers to
/// all of them.
#[derive(Clone)]
pub struct PluginLink {
    commands: UnboundedSender<PluginCommand>,
    answers: broadcast::Sender<SearchResults>,
}

impl PluginLink {
    /// Connects on a thread of its own, which runs as long as the process.
    pub fn start() -> io::Result<Self> {
        let (commands, outbox) = unbounded_channel();
        let (answers, _) = broadcast::channel(ANSWER_BACKLOG);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let delivered = answers.clone();
        thread::Builder::new()
            .name("crownlauncher-plugins".into())
            .spawn(move || runtime.block_on(run(delivered, outbox)))?;
        Ok(Self { commands, answers })
    }

    pub fn send(&self, command: PluginCommand) {
        let _ = self.commands.send(command);
    }

    pub fn commands(&self) -> UnboundedSender<PluginCommand> {
        self.commands.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SearchResults> {
        self.answers.subscribe()
    }
}
