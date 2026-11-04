use anyhow::{Result, bail};
use crownlauncher::ipc::{self, Request};

const USAGE: &str = "\
crownlauncher [toggle | show | hide]

Without a command, starts the launcher hidden, for the session's autostart.
The commands show or hide the running launcher, starting it first when it is not running.";

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,resvg=error,usvg=error"),
    )
    .init();
    let request = match std::env::args().nth(1).as_deref() {
        None => None,
        Some("toggle") => Some(Request::Toggle),
        Some("show") => Some(Request::Show),
        Some("hide") => Some(Request::Hide),
        Some("--help" | "-h") => {
            println!("{USAGE}");
            return Ok(());
        }
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}"),
    };
    match request {
        Some(request) if ipc::forward(request) => Ok(()),
        Some(Request::Hide) => Ok(()),
        Some(_) => crownlauncher::app(true),
        None if ipc::is_running() => Ok(()),
        None => crownlauncher::app(false),
    }
}
