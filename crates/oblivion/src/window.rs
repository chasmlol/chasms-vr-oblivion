//! The game window, kept out of VR's way.
//!
//! * **Focus**: Oblivion skips its frames while its window isn't the active one (it sleeps 50 ms
//!   instead). In VR the player looks at the headset, not the window, so the game is told its window
//!   is always active ([`install`], once the game has made its device). And it never hears its window
//!   was deactivated ([`install_focus`]): its window procedure (0x4060F0) pauses the game's clock on
//!   `WM_ACTIVATE`'s inactive (the time globals' pause count, 0xB33E90), and paused, every frame is
//!   no time at all: the world stands still (no walking, no swing, no menu's fade) while the
//!   runtime goes on drawing the hands. Another window in front (the desktop view, OBS) is no
//!   reason for the game to stop, nor for its sounds to go quiet ([`install_sound_focus`]).
//! * **Background** (the `background` setting, for testing): the game's window is made off-screen
//!   and never activated or brought to the front, and message boxes are logged and answered (a
//!   player isn't there to), so the game runs without touching the desktop. (A desktop of its own
//!   doesn't do: Direct3D 9 can't make a device off the interactive desktop.)
//!
//! * **Size**: the game renders each eye into its back buffer, which is as big as its screen. When
//!   it makes its window (after reading its screen size, before making its renderer, 0x498340) its
//!   screen size becomes the headset's eye size as the runtime asks for it ([`crate::resolution`]),
//!   so the device and every render target are made at that size.
//!
//! Nothing here touches the game's import table before the game is running: Steam's DRM checks it
//! when the game starts, and restarts the game through Steam if it was changed. The background
//! hooks are on `user32.dll` itself, and only act on the game's own window.

#[cfg(windows)]
pub use win::*;

#[cfg(windows)]
mod win {
    use crate::{engine, hooks, iat, settings};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows::core::{PCSTR, w};

    static REAL_ACTIVE: AtomicUsize = AtomicUsize::new(0);
    static REAL_CREATE: AtomicUsize = AtomicUsize::new(0);
    static REAL_SHOW: AtomicUsize = AtomicUsize::new(0);
    static REAL_POS: AtomicUsize = AtomicUsize::new(0);
    static REAL_FOREGROUND: AtomicUsize = AtomicUsize::new(0);
    static REAL_BOX: AtomicUsize = AtomicUsize::new(0);
    /// The game's window, once made.
    static GAME: AtomicUsize = AtomicUsize::new(0);

    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const SW_SHOWNOACTIVATE: i32 = 4;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const IDOK: i32 = 1;
    /// Where the window goes in the background: off every monitor (not -32000, where Windows puts
    /// minimized windows: there the device counts as lost and the game stops its frames).
    const OFF: i32 = -20000;

    fn hook_user32(func: &str, hook: *const (), slot: &AtomicUsize) -> bool {
        let Ok(user32) = (unsafe { LoadLibraryW(w!("user32.dll")) }) else { return false };
        let name = std::ffi::CString::new(func).unwrap();
        let Some(f) = (unsafe { GetProcAddress(user32, PCSTR(name.as_ptr() as *const u8)) }) else { return false };
        hooks::install_into(f as *const (), hook, func, slot)
    }

    /// Before the game makes its window.
    pub fn install_early() {
        let c = hook_user32("CreateWindowExA", create_window as *const (), &REAL_CREATE);
        if !settings::get().background {
            return;
        }
        let s = hook_user32("ShowWindow", show_window as *const (), &REAL_SHOW);
        let p = hook_user32("SetWindowPos", set_window_pos as *const (), &REAL_POS);
        let f = hook_user32("SetForegroundWindow", set_foreground_window as *const (), &REAL_FOREGROUND);
        let m = hook_user32("MessageBoxA", message_box as *const (), &REAL_BOX);
        tracing::info!("window: background mode (create {c}, show {s}, move {p}, foreground {f}, message box {m})");
    }

    /// The game's window procedure, and `WM_ACTIVATE` with the window deactivated (`WA_INACTIVE`).
    #[cfg(target_arch = "x86")]
    const WND_PROC: usize = 0x0040_60F0;
    #[cfg(target_arch = "x86")]
    const WM_ACTIVATE: u32 = 6;
    #[cfg(target_arch = "x86")]
    const WA_INACTIVE: usize = 0;
    #[cfg(target_arch = "x86")]
    static REAL_WND_PROC: AtomicUsize = AtomicUsize::new(0);

    /// Once the game's code is there, before it makes its window: its window procedure hooked, so
    /// the game is never told its window was deactivated.
    #[cfg(target_arch = "x86")]
    pub fn install_focus() {
        if hooks::install_into(WND_PROC as *const (), wnd_proc as *const (), "the game's window procedure", &REAL_WND_PROC) {
            tracing::info!("window: the game plays on with another window in front (its clock never paused for it)");
        }
    }

    #[cfg(target_arch = "x86")]
    unsafe extern "system" fn wnd_proc(h: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
        if msg == WM_ACTIVATE && wparam & 0xFFFF == WA_INACTIVE {
            // (The game's own answer to it is 0.)
            return 0;
        }
        let real: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize = unsafe { std::mem::transmute(REAL_WND_PROC.load(Ordering::Acquire)) };
        unsafe { real(h, msg, wparam, lparam) }
    }

    /// The game's sounds go on with another window in front: DirectSound mutes a buffer made
    /// without `DSBCAPS_GLOBALFOCUS` while its window (the game's, at `DSSCL_PRIORITY`) isn't the
    /// foreground one, and the game makes none with it. `DirectSoundCreate8` (dsound.dll's own) is
    /// hooked before the game makes its sound, and `IDirectSound8::CreateSoundBuffer` (slot 3) on
    /// what it makes; a duplicated buffer has its original's flags.
    #[cfg(target_arch = "x86")]
    static REAL_DS_CREATE: AtomicUsize = AtomicUsize::new(0);
    #[cfg(target_arch = "x86")]
    static REAL_CREATE_BUFFER: AtomicUsize = AtomicUsize::new(0);
    #[cfg(target_arch = "x86")]
    const DSBCAPS_PRIMARYBUFFER: u32 = 0x1;
    #[cfg(target_arch = "x86")]
    const DSBCAPS_GLOBALFOCUS: u32 = 0x8000;
    /// `DSBUFFERDESC`'s size (a 32-bit process's: `DSBUFFERDESC1`, the older one, is 20).
    #[cfg(target_arch = "x86")]
    const BUFFER_DESC: usize = 36;

    #[cfg(target_arch = "x86")]
    pub fn install_sound_focus() {
        let Ok(dsound) = (unsafe { LoadLibraryW(w!("dsound.dll")) }) else { return };
        let Some(f) = (unsafe { GetProcAddress(dsound, PCSTR(c"DirectSoundCreate8".as_ptr() as *const u8)) }) else { return };
        if hooks::install_into(f as *const (), ds_create as *const (), "DirectSoundCreate8", &REAL_DS_CREATE) {
            tracing::info!("window: the game's sounds play on with another window in front");
        }
    }

    #[cfg(target_arch = "x86")]
    unsafe extern "system" fn ds_create(guid: *const u8, out: *mut *mut u8, outer: *mut u8) -> i32 {
        let real: unsafe extern "system" fn(*const u8, *mut *mut u8, *mut u8) -> i32 = unsafe { std::mem::transmute(REAL_DS_CREATE.load(Ordering::Acquire)) };
        let hr = unsafe { real(guid, out, outer) };
        hooks::guard_void("DirectSoundCreate8", || {
            if hr < 0 || out.is_null() || unsafe { (*out).is_null() } || REAL_CREATE_BUFFER.load(Ordering::Acquire) != 0 {
                return;
            }
            let create = unsafe { *(*(*out as *const *const *const ())).add(3) };
            hooks::install_into(create, create_buffer as *const (), "IDirectSound8::CreateSoundBuffer", &REAL_CREATE_BUFFER);
        });
        hr
    }

    #[cfg(target_arch = "x86")]
    unsafe extern "system" fn create_buffer(ds: *mut u8, desc: *const u8, out: *mut *mut u8, outer: *mut u8) -> i32 {
        let real: unsafe extern "system" fn(*mut u8, *const u8, *mut *mut u8, *mut u8) -> i32 = unsafe { std::mem::transmute(REAL_CREATE_BUFFER.load(Ordering::Acquire)) };
        // (A copy with the flag: the game's own description stays as it made it.)
        let mut copy = [0u8; BUFFER_DESC];
        let size = if desc.is_null() { 0 } else { unsafe { (desc as *const u32).read_unaligned() as usize } };
        if (8..=BUFFER_DESC).contains(&size) {
            unsafe { std::ptr::copy_nonoverlapping(desc, copy.as_mut_ptr(), size) };
            let flags = u32::from_le_bytes(copy[4..8].try_into().unwrap());
            if flags & DSBCAPS_PRIMARYBUFFER == 0 {
                copy[4..8].copy_from_slice(&(flags | DSBCAPS_GLOBALFOCUS).to_le_bytes());
                return unsafe { real(ds, copy.as_ptr(), out, outer) };
            }
        }
        unsafe { real(ds, desc, out, outer) }
    }

    /// Once the game is running (its device made).
    pub fn install() {
        if let Some(o) = iat::patch("USER32.dll", "GetActiveWindow", get_active_window as *const () as usize) {
            REAL_ACTIVE.store(o, Ordering::Release);
            tracing::info!("window: the game's window is always the active one");
        }
    }

    fn is_game(h: HWND) -> bool {
        let g = GAME.load(Ordering::Acquire);
        g != 0 && h.0 as usize == g
    }

    unsafe extern "system" fn get_active_window() -> HWND {
        let game = engine::window();
        if game != 0 {
            return HWND(game as *mut _);
        }
        let real: unsafe extern "system" fn() -> HWND = unsafe { std::mem::transmute(REAL_ACTIVE.load(Ordering::Acquire)) };
        unsafe { real() }
    }

    type CreateFn = unsafe extern "system" fn(u32, *const u8, *const u8, u32, i32, i32, i32, i32, HWND, isize, isize, *const ()) -> HWND;

    unsafe extern "system" fn create_window(ex: u32, cls: *const u8, name: *const u8, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: isize, inst: isize, param: *const ()) -> HWND {
        let real: CreateFn = unsafe { std::mem::transmute(REAL_CREATE.load(Ordering::Acquire)) };
        // The game's main window: its class is "Oblivion" (a class atom isn't a name: not ours).
        let ours = parent.0.is_null() && (cls as usize) > 0xFFFF && cstr(cls) == "Oblivion";
        if ours {
            crate::resolution::apply();
        }
        let background = ours && settings::get().background;
        // The game makes its window where it was when the game last quit (`Oblivion.ini`
        // `iLocation X`/`Y`): quit while minimised, that's -32000, off every monitor, where it can't
        // be brought to the front on the desktop (it plays on in the headset all the same: it's
        // always the active window to the game). It goes to the screen's corner instead.
        let (x, y) = if ours && !background && off_screen(x, y) { (0, 0) } else { (x, y) };
        let (ex, x, y) = if background { (ex | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, OFF, OFF) } else { (ex, x, y) };
        let hwnd = unsafe { real(ex, cls, name, style, x, y, w, h, parent, menu, inst, param) };
        if background {
            GAME.store(hwnd.0 as usize, Ordering::Release);
            tracing::info!("window: the game's window made off-screen, never activated (background)");
        }
        hwnd
    }

    /// A window's corner at (x, y) is on no monitor (Windows' own default place, `CW_USEDEFAULT`,
    /// isn't a place).
    fn off_screen(x: i32, y: i32) -> bool {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONULL, MonitorFromPoint};
        const CW_USEDEFAULT: i32 = i32::MIN;
        x != CW_USEDEFAULT && y != CW_USEDEFAULT && unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONULL) }.is_invalid()
    }

    unsafe extern "system" fn show_window(h: HWND, cmd: i32) -> i32 {
        let real: unsafe extern "system" fn(HWND, i32) -> i32 = unsafe { std::mem::transmute(REAL_SHOW.load(Ordering::Acquire)) };
        let cmd = if is_game(h) && matches!(cmd, 1 | 3 | 5 | 9 | 10) { SW_SHOWNOACTIVATE } else { cmd };
        unsafe { real(h, cmd) }
    }

    unsafe extern "system" fn set_window_pos(h: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32 {
        let real: unsafe extern "system" fn(HWND, HWND, i32, i32, i32, i32, u32) -> i32 = unsafe { std::mem::transmute(REAL_POS.load(Ordering::Acquire)) };
        if is_game(h) {
            return unsafe { real(h, after, OFF, OFF, cx, cy, flags | SWP_NOACTIVATE) };
        }
        unsafe { real(h, after, x, y, cx, cy, flags) }
    }

    unsafe extern "system" fn set_foreground_window(h: HWND) -> i32 {
        if is_game(h) {
            return 1;
        }
        let real: unsafe extern "system" fn(HWND) -> i32 = unsafe { std::mem::transmute(REAL_FOREGROUND.load(Ordering::Acquire)) };
        unsafe { real(h) }
    }

    unsafe extern "system" fn message_box(_h: HWND, text: *const u8, caption: *const u8, _kind: u32) -> i32 {
        tracing::warn!("window: a message box '{}': {} (answered OK)", cstr(caption), cstr(text));
        IDOK
    }

    fn cstr(p: *const u8) -> String {
        if p.is_null() {
            return String::new();
        }
        unsafe { std::ffi::CStr::from_ptr(p as *const std::ffi::c_char) }.to_string_lossy().into_owned()
    }
}
