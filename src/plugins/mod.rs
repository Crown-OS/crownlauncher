mod client;
mod link;

use crownplugin_proto::HitKey;

pub use client::run;
pub use link::PluginLink;

/// What the launcher tells crownplugind.
#[derive(Debug, Clone)]
pub enum PluginCommand {
    Search(String),
    Activate(HitKey),
    Shown(bool),
}
