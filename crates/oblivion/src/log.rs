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

/// Access violations, logged where they happen (first chance: the game's own handlers may still
/// deal with them): the instruction pointer, the address it touched and the return addresses on
/// the stack that are in the game's code or this DLL's. The first 20.
#[cfg(all(windows, target_arch = "x86"))]
pub fn watch_access_violations() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use windows::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION;
    use windows::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};
    static SEEN: AtomicU32 = AtomicU32::new(0);
    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        const CONTINUE_SEARCH: i32 = 0;
        let (rec, ctx) = unsafe { (&*(*info).ExceptionRecord, &*(*info).ContextRecord) };
        if rec.ExceptionCode != EXCEPTION_ACCESS_VIOLATION || SEEN.fetch_add(1, Ordering::Relaxed) >= 20 {
            return CONTINUE_SEARCH;
        }
        let ours = handler as *const () as usize;
        let code = |v: u32| (0x0040_1000..0x00A3_0000).contains(&v) || (v as usize).abs_diff(ours) < 0x0100_0000;
        let mut stack = Vec::new();
        for i in 0..512usize {
            let at = ctx.Esp as usize + i * 4;
            if !crate::mem::is_readable(at as *const u8, 4) {
                break;
            }
            let v = unsafe { *(at as *const u32) };
            if code(v) {
                stack.push(format!("{v:#x}"));
                if stack.len() >= 24 {
                    break;
                }
            }
        }
        let what = if rec.NumberParameters >= 2 { format!("{} {:#x}", if rec.ExceptionInformation[0] == 0 { "reading" } else if rec.ExceptionInformation[0] == 8 { "executing" } else { "writing" }, rec.ExceptionInformation[1]) } else { String::new() };
        tracing::error!("access violation at eip {:#x} ({what}), esp {:#x}; code on the stack: {}", ctx.Eip, ctx.Esp, stack.join(" "));
        CONTINUE_SEARCH
    }
    unsafe { AddVectoredExceptionHandler(1, Some(handler)) };
    tracing::info!("log: access violations are logged (this DLL's code is near {:#x})", handler as *const () as usize);
}
