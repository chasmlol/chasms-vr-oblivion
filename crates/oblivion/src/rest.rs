//! Rest from the big four (the inventory, magic, the map, the character: Menu Mode, the off hand's
//! B): a button of the game's own (`button_short.xml`, its parchment button and brown ink) beside
//! the HUD's frame, which floats with the big four as their switcher ([`crate::hud`]). Clicked,
//! Menu Mode closes and the game's own Rest control is tapped: its own Rest/Wait menu, by its own
//! rules (not with enemies about, not in the air). (An empty hand's held trigger is its fist.)
//!
//! It's read by the game's own `Tile::ReadXML` from a file of ours (written beside the runtime's
//! settings at start) into the HUD's tiles, so the game draws it, lights it under the cursor and
//! clicks it as its own; the HUD's click handler (its vtable's `HandleClick`, hooked once the HUD
//! is there) sees its id first.

/// Its tile's name, and its id (clear of the HUD's own, under 100, and the loot list's).
pub const REST: &str = "rustvr_rest";
pub const REST_ID: i32 = 990;

/// The button's file.
pub fn xml() -> String {
    format!(
        "<rect name=\"rustvr_rest_layout\"> <locus> &true; </locus> <visible> &false; </visible> <depth> 30 </depth> <x> 0 </x> <y> 0 </y>\n\
         <image name=\"{REST}\">\n<include src=\"button_short.xml\"/>\n<id> {REST_ID} </id> <target> &true; </target> <depth> 3 </depth> <clicksound> 1 </clicksound>\n<user0> Rest </user0>\n<x> 0 </x> <y> 0 </y>\n</image>\n\
         </rect>\n"
    )
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn frame(_now: f64) {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn tap(_now: f64) -> Option<&'static str> {
    None
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{hooks, mem, menus, tiles};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// `Tile::ReadXML(parent, path)`, `Tile::UpdateFloat` (thiscall).
    const READ_XML: usize = 0x0059_0420;
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    /// The HUD (`kMenuType_HUDMain`), its click (vtable slot 3: the id and the clicked tile).
    const MENU_HUD: u32 = 0x3EC;
    const VT_HANDLE_CLICK: usize = 3 * 4;
    const FALSE: f32 = 1.0;
    /// The HUD's frame (`hud_main_menu.xml`): its background, and how far its gilt frame reaches
    /// right of it (761 wide from 70 left of it); the button sits this far right of that, beside
    /// its middle (the button's 44 high).
    const BACKGROUND: &str = "hudmain_background";
    const FRAME_RIGHT: f32 = 691.0;
    const GAP: f32 = 12.0;
    const BUTTON_H: f32 = 44.0;
    /// How long Menu Mode is given to close before the Rest is given up (seconds).
    const CLOSING: f64 = 2.0;

    static PATH: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    static REAL_CLICK: AtomicUsize = AtomicUsize::new(0);
    static HOOKED: AtomicBool = AtomicBool::new(false);
    static CLICKED: AtomicBool = AtomicBool::new(false);

    #[derive(Clone, Copy, PartialEq)]
    enum Step {
        None,
        /// Menu Mode closing (tapped yet, and when it was clicked).
        Closing { tapped: bool, at: f64 },
        Resting,
    }

    struct State {
        tile: usize,
        parent: usize,
        shown: bool,
        step: Step,
        failed: bool,
    }

    static STATE: Mutex<State> = Mutex::new(State { tile: 0, parent: 0, shown: false, step: Step::None, failed: false });

    fn update_float(tile: usize, id: u32, v: f32) {
        type F = unsafe extern "thiscall" fn(usize, u32, f32);
        if tile != 0 {
            hooks::guard_void("rest button", || unsafe { std::mem::transmute::<usize, F>(UPDATE_FLOAT)(tile, id, v) });
        }
    }

    /// The HUD's click: the Rest's id is ours, the rest the game's.
    unsafe extern "thiscall" fn click(menu: usize, id: i32, tile: usize) {
        if id == REST_ID && menu == tiles::menu(MENU_HUD) {
            CLICKED.store(true, Ordering::Relaxed);
            return;
        }
        let real = REAL_CLICK.load(Ordering::Relaxed);
        if real != 0 {
            type F = unsafe extern "thiscall" fn(usize, i32, usize);
            unsafe { std::mem::transmute::<usize, F>(real)(menu, id, tile) };
        }
    }

    /// The button's file written (beside the runtime's settings).
    pub fn install() {
        let dir = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("RustVR");
        let path = dir.join("oblivion_rest.xml");
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, xml())) {
            tracing::warn!("rest: can't write its button {}: {e}", path.display());
            return;
        }
        let _ = PATH.set(std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default());
    }

    fn alive(s: &State) -> bool {
        s.tile != 0 && tiles::menu_tile(MENU_HUD) == s.parent && tiles::children(s.parent).contains(&s.tile)
    }

    /// The button made in the HUD's tiles, right of its frame, and the HUD's click hooked.
    fn make(s: &mut State) -> bool {
        let Some(path) = PATH.get() else { return false };
        let parent = tiles::menu_tile(MENU_HUD);
        let menu = tiles::menu(MENU_HUD);
        if parent == 0 || menu == 0 {
            return false;
        }
        if !HOOKED.swap(true, Ordering::Relaxed) {
            let at = mem::deref(mem::deref(menu) + VT_HANDLE_CLICK);
            if at == 0 || !hooks::install_into(at as *const (), click as *const (), "HUDMainMenu::HandleClick", &REAL_CLICK) {
                tracing::warn!("rest: the HUD's clicks can't be had ({at:#x}): no Rest button");
                s.failed = true;
                return false;
            }
        }
        type Read = unsafe extern "thiscall" fn(usize, *const u8) -> usize;
        let tile = hooks::guard("rest button", 0, || unsafe { std::mem::transmute::<usize, Read>(READ_XML)(parent, path.as_ptr() as *const u8) });
        if tile == 0 {
            tracing::warn!("rest: the game made no button from {path:?}");
            s.failed = true;
            return false;
        }
        let tree = tiles::Tree::read(parent);
        let Some(bg) = tree.find(BACKGROUND).map(|p| p.rect) else {
            tracing::warn!("rest: the HUD has no {BACKGROUND}: no Rest button");
            s.failed = true;
            return false;
        };
        let (x, y) = (bg[0] + FRAME_RIGHT + GAP, (bg[1] + bg[3] - BUTTON_H) * 0.5);
        update_float(tile, tiles::X as u32, x);
        update_float(tile, tiles::Y as u32, y);
        (s.tile, s.parent, s.shown) = (tile, parent, false);
        tracing::info!("rest: its button made in the HUD of the game's own art, at ({x:.0}, {y:.0}) beside its frame");
        true
    }

    /// Once a game frame: the button shown while the big four are up; clicked, Menu Mode closes,
    /// then the game's Rest.
    pub fn frame(now: f64) {
        let mut s = STATE.lock().unwrap();
        let big_four = tiles::top_menu() == tiles::BIG_FOUR;
        if big_four && !s.failed && !alive(&s) && !make(&mut s) {
            return;
        }
        let on = big_four && alive(&s);
        if s.shown != on && alive(&s) {
            update_float(s.tile, tiles::VISIBLE as u32, if on { tiles::TRUE } else { FALSE });
        }
        s.shown = on;
        if CLICKED.swap(false, Ordering::Relaxed) && s.step == Step::None {
            tracing::info!("rest: clicked: Menu Mode closes, then the game's Rest");
            s.step = Step::Closing { tapped: false, at: now };
        }
        if let Step::Closing { at, .. } = s.step {
            if !menus::menu_mode() {
                s.step = Step::Resting;
            } else if now - at > CLOSING {
                tracing::warn!("rest: Menu Mode didn't close in {CLOSING} s: no Rest");
                s.step = Step::None;
            }
        }
    }

    /// The control to tap now for a Rest clicked (Menu Mode to close the big four, then Rest), once.
    pub fn tap(_now: f64) -> Option<&'static str> {
        let mut s = STATE.lock().unwrap();
        match s.step {
            Step::Closing { tapped: false, at } => {
                s.step = Step::Closing { tapped: true, at };
                Some("Menu Mode")
            }
            Step::Resting => {
                s.step = Step::None;
                tracing::info!("rest: the game's Rest tapped");
                Some("Rest")
            }
            _ => None,
        }
    }
}
