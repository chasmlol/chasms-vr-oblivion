//! Diagnostic logging: `tracing` events go to `rustvr-oblivion.log` beside the DLL and to the
//! debugger via `OutputDebugString`. Installed once from `DllMain`.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

static FILE: Mutex<Option<File>> = Mutex::new(None);

/// A `tracing` writer that appends to the log file and echoes to the debugger.
struct Sink;

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(mut g) = FILE.lock() {
            if let Some(f) = g.as_mut() {
                let _ = f.write_all(buf);
                let _ = f.flush();
            }
        }
        #[cfg(windows)]
        {
            if let Ok(s) = std::ffi::CString::new(buf) {
                unsafe {
                    windows::Win32::System::Diagnostics::Debug::OutputDebugStringA(windows::core::PCSTR(s.as_ptr() as *const u8));
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Opens the log file and installs the subscriber. Safe to call once; later calls are ignored.
pub fn init(path: &Path) {
    // Appended: a game that restarts itself (Steam's DRM) keeps the first start's lines.
    if let Ok(f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        *FILE.lock().unwrap() = Some(f);
    }
    let sub = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_level(true)
        .with_target(false)
        .with_writer(|| Sink)
        .finish();
    let _ = tracing::subscriber::set_global_default(sub);
    // A panic aborts the game (release builds): say where first.
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("panic: {info}");
    }));
    tracing::info!(
        "oblivion adapter log started (pid {}, SteamAppId {:?}, {:?})",
        std::process::id(),
        std::env::var("SteamAppId").ok(),
        std::env::args().collect::<Vec<_>>()
    );
}
