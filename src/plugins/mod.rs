mod client;

use crownplugin_proto::HitKey;

pub use client::run;

/// What the launcher tells crownplugind.
#[derive(Debug, Clone)]
pub enum PluginCommand {
    Search(String),
    Activate(HitKey),
    Shown(bool),
}
