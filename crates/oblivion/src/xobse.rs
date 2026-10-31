//! xOBSE (the script extender) and the plugins it loads, such as Oblivion Stutter Remover, when the
//! player has it installed beside the game. Launched through Steam, the game gets
//! `obse_steam_loader.dll` loaded for it (Valve's arrangement with OBSE). The loader checks the exe and
//! loads `obse_1_2_416.dll`, which patches the game's code as it loads. Started from its exe, as the
//! adapter's launcher starts it, nothing loads them.
//!
//! So the adapter loads the same loader where xOBSE's own launcher loads xOBSE into a retail exe: on
//! the game's main thread as its C runtime starts, after every DLL has loaded and before the game's
//! `WinMain`. That is the runtime's one call of `GetCommandLineA` (0x9877A9, in its start at 0x98769C,
//! which Steam's stub at the exe's entry point hands over to): the import is ours until then. This
//! Steam exe's code isn't encrypted (the adapter reads it 16 µs after it loads), so xOBSE's patches
//! there stay.

use crate::hooks;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const LOADER: &str = "obse_steam_loader.dll";
const OBSE: &str = "obse_1_2_416.dll";

static DIR: OnceLock<PathBuf> = OnceLock::new();
static REAL: AtomicUsize = AtomicUsize::new(0);
static DONE: AtomicBool = AtomicBool::new(false);

/// Testing, `RUSTVR_OBLIVION_NO_XOBSE=1`: the game without xOBSE, as before.
fn enabled() -> bool {
    !std::env::var("RUSTVR_OBLIVION_NO_XOBSE").is_ok_and(|v| v == "1")
}

/// At the adapter's load: the game's `GetCommandLineA` import is ours until the C runtime calls it,
/// if xOBSE is installed in the game's folder (`dir`) and nothing has loaded it yet.
#[cfg(windows)]
pub fn install_early(dir: &Path) {
    if !dir.join(LOADER).exists() {
        return;
    }
    if !enabled() {
        tracing::info!("xobse: testing: not loaded (RUSTVR_OBLIVION_NO_XOBSE)");
        return;
    }
    if loaded() {
        tracing::info!("xobse: already loaded (the game was started through Steam)");
        return;
    }
    let _ = DIR.set(dir.to_path_buf());
    match crate::iat::patch("KERNEL32.dll", "GetCommandLineA", get_command_line as *const () as usize) {
        Some(real) => REAL.store(real, Ordering::Release),
        None => tracing::error!("xobse: the game imports no GetCommandLineA: xOBSE isn't loaded"),
    }
}

#[cfg(windows)]
fn loaded() -> bool {
    let name: Vec<u16> = OBSE.encode_utf16().chain([0]).collect();
    unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(windows::core::PCWSTR(name.as_ptr())) }.is_ok()
}

#[cfg(windows)]
unsafe extern "system" fn get_command_line() -> *const u8 {
    let real: unsafe extern "system" fn() -> *const u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
    if !DONE.swap(true, Ordering::AcqRel) {
        hooks::guard_void("xOBSE's loader", || {
            // The import is the game's again, then xOBSE comes in.
            let _ = crate::iat::patch("KERNEL32.dll", "GetCommandLineA", REAL.load(Ordering::Acquire));
            load();
        });
    }
    unsafe { real() }
}

#[cfg(windows)]
fn load() {
    use std::os::windows::ffi::OsStrExt;
    let Some(dir) = DIR.get() else { return };
    let path: Vec<u16> = dir.join(LOADER).as_os_str().encode_wide().chain([0]).collect();
    match unsafe { windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::PCWSTR(path.as_ptr())) } {
        Ok(_) if loaded() => tracing::info!("xobse: loaded (its own log: obse.log in the game's folder)"),
        Ok(_) => tracing::error!("xobse: its loader didn't load it (obse_steam_loader.log in the game's folder says why)"),
        Err(e) => tracing::error!("xobse: {LOADER}: {e}"),
    }
}
