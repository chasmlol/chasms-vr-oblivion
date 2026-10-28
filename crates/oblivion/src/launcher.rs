//! Starts the RustVR runtime in the background when the game starts, so playing Oblivion in VR is just
//! starting Oblivion: the runtime (`rustvr-play --no-range`, no window) takes the headset and the
//! adapter passes the game through to it.
//!
//! `rustvr-runtime.txt` next to the DLL names the runtime's executable (the installer writes it). If
//! a runtime is already up (its shared memory exists), or the file is missing, nothing is started.

use std::path::Path;

/// The installer's pointer to the runtime.
pub const RUNTIME_FILE: &str = "rustvr-runtime.txt";

/// The runtime's executable, from `dir/rustvr-runtime.txt` (first non-empty line, quotes allowed).
pub fn runtime_path(dir: &Path) -> Option<std::path::PathBuf> {
    let text = std::fs::read_to_string(dir.join(RUNTIME_FILE)).ok()?;
    let line = text.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#'))?;
    Some(std::path::PathBuf::from(line.trim_matches('"')))
}

/// Starts the runtime unless one is already running. Call from a worker thread (not `DllMain`).
#[cfg(windows)]
pub fn ensure_runtime(dir: &Path) {
    use std::os::windows::process::CommandExt;
    if rustvr_protocol::link::Link::open(rustvr_protocol::DEFAULT_MAPPING).is_ok() {
        tracing::info!("launcher: a RustVR runtime is already running");
        return;
    }
    let Some(exe) = runtime_path(dir) else {
        tracing::info!("launcher: no {RUNTIME_FILE}: start the runtime yourself");
        return;
    };
    // Detached, in its own process group, and out of the game's job if it may (Steam waits for
    // everything in the game's job before it lets the game start again); it also quits by itself a
    // few seconds after the game has gone.
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
    let start = |flags: u32| {
        std::process::Command::new(&exe)
            .args(["--no-range", "--exit-with-game", "--game-pid", &std::process::id().to_string()])
            .current_dir(exe.parent().unwrap_or(Path::new(".")))
            .creation_flags(flags)
            .spawn()
    };
    let flags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;
    match start(flags | CREATE_BREAKAWAY_FROM_JOB).or_else(|_| start(flags)) {
        Ok(child) => tracing::info!("launcher: started the runtime {} (pid {})", exe.display(), child.id()),
        Err(e) => tracing::error!("launcher: couldn't start {}: {e}", exe.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_runtime_path() {
        let dir = std::env::temp_dir().join(format!("rustvr-launcher-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(runtime_path(&dir).is_none());
        std::fs::write(dir.join(RUNTIME_FILE), "# RustVR\n\"X:\\rustvr\\target\\release\\rustvr-play.exe\"\n").unwrap();
        assert_eq!(runtime_path(&dir).unwrap(), std::path::PathBuf::from("X:\\rustvr\\target\\release\\rustvr-play.exe"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
