use std::process::{Command, Stdio};

/// Runs `argv` in a session of its own, off the UI thread. `setsid --fork` returns at once, so
/// the program outlives the launcher and never lingers as its zombie.
pub fn spawn_detached(argv: Vec<String>) {
    let spawned = std::thread::Builder::new()
        .name("crownlauncher-spawn".into())
        .spawn(move || {
            let status = Command::new("setsid")
                .arg("--fork")
                .args(&argv)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if let Err(error) = status {
                log::warn!("spawn {argv:?}: {error}");
            }
        });
    if let Err(error) = spawned {
        log::warn!("could not start the spawn thread: {error}");
    }
}
