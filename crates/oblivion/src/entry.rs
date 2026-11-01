//! The DLL's entry point. On load: the log, the settings, Oblivion.ini set for VR, xOBSE's loader
//! readied (if it's installed: [`crate::xobse`]), and a thread that hooks `Direct3DCreate9` straight
//! away (the game makes its device seconds later) and the engine once Steam's DRM has decrypted the
//! game's code.

use crate::{console, d3d9, engine, game_settings, launcher, log, mem, render, settings, window};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, GetModuleFileNameW};

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

static MOD_DIR: OnceLock<PathBuf> = OnceLock::new();

fn module_dir(inst: HMODULE) -> PathBuf {
    let mut buf = [0u16; 512];
    let n = unsafe { GetModuleFileNameW(Some(inst), &mut buf) } as usize;
    let path = PathBuf::from(String::from_utf16_lossy(&buf[..n]));
    path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
}

fn startup() {
    d3d9::install();
    if let Some(dir) = MOD_DIR.get() {
        launcher::ensure_runtime(dir);
    }
    let t0 = Instant::now();
    while !engine::supported() {
        if t0.elapsed() > Duration::from_secs(60) {
            tracing::error!("startup: not Oblivion 1.2.0.416 (or its code never decrypted): VR is off");
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    tracing::info!("startup: the game's code is there after {:?}", t0.elapsed());
    render::install();
    #[cfg(target_arch = "x86")]
    window::install_focus();
    #[cfg(target_arch = "x86")]
    window::install_sound_focus();
    #[cfg(target_arch = "x86")]
    crate::worldmenu::install();
    #[cfg(target_arch = "x86")]
    crate::clock::install();
    #[cfg(target_arch = "x86")]
    crate::grass::install();
    #[cfg(target_arch = "x86")]
    crate::movement::install();
    crate::books::install();
    #[cfg(target_arch = "x86")]
    crate::swim::install();
    #[cfg(target_arch = "x86")]
    crate::hud_info::install();
    crate::pointing::install();
    crate::combat::install();
    crate::console::install();
    crate::slot_menu::install();
    crate::keyboard::install();
    crate::voice::install();
    crate::look::install();
    tracing::info!("startup: hooks installed");
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(inst: HMODULE, reason: u32, _reserved: *mut core::ffi::c_void) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(inst).ok() };
        let dir = module_dir(inst);
        let _ = MOD_DIR.set(dir.clone());
        log::init(&dir.join("rustvr-oblivion.log"));
        log::watch_access_violations();
        mem::guard_reads();
        settings::load_from(&dir);
        game_settings::restore(&dir);
        let s = settings::get();
        game_settings::apply(&dir, (s.eye_width > 0 && s.eye_height > 0).then_some((s.eye_width, s.eye_height)), &s.save_folder);
        console::load_test(&dir);
        window::install_early();
        crate::xobse::install_early(&dir);
        std::thread::spawn(startup);
    } else if reason == DLL_PROCESS_DETACH
        && let Some(dir) = MOD_DIR.get()
    {
        game_settings::restore(dir);
    }
    1
}
