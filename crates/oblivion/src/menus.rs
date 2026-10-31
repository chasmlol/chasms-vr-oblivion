//! The game's menus under the runtime's laser: while a menu is open the runtime shows the game's
//! menus as a panel and points at it (`Intents::cursor_uv`), and the game's own cursor goes where
//! the laser is; the trigger clicks (the left mouse button, [`crate::input`]).
//!
//! The cursor is a menu tile (`InterfaceManager`, `*0xB3A6E0`, +0x1C) whose node (+0x24) the game
//! moves by the mouse's movement (0x57E7C0): x across the screen (+0x54) and its height (+0x5C),
//! in menu units: the screen is 960 high and as wide as its shape makes it, centred on 0 (0x57E2D0
//! and its three siblings). Put there, the game sees the cursor move and works out what's under it.

use crate::mem;

const INTERFACE_MANAGER: usize = 0x00B3_A6E0;
const IS_MENU_MODE: usize = 0x0057_8F60;
const CURSOR: usize = 0x1C;
const TILE_NODE: usize = 0x24;
const NODE_X: usize = 0x54;
const NODE_HEIGHT: usize = 0x5C;
/// The game's screen size (`nWidth`, `nHeight`).
/// The game's open menus by type (`g_TileMenuArray`: an `NiTArray` of `TileMenu*`, its data at +4,
/// the first entry `kMenuType_Message`, 0x3E9); an entry is there while its menu is.
const TILE_MENUS: usize = 0x00B1_3970;
const MENU_FIRST: u32 = 0x3E9;
pub const MENU_LOADING: u32 = 0x3EF;
pub const MENU_MAIN: u32 = 0x414;
/// The race menu (`RaceSexMenu`: a new game's character creation, or `showracemenu`).
pub const MENU_RACE_SEX: u32 = 0x40C;
/// Dialogue (`DialogMenu`), its pieces with the speaker ([`crate::hud`]).
pub const MENU_DIALOG: u32 = 0x3F1;
/// `PlayerCharacter*`, and a reference's cell (`TESObjectREFR::parentCell`).
const PLAYER: usize = 0x00B3_33C4;
const PARENT_CELL: usize = 0x40;

/// The interface's own 3D scene (`InterfaceManager` +4, "Menu3DRoot"): its render (0x57F170)
/// draws it under the menus whenever its bound isn't empty.
const MENU_3D: usize = 4;
/// `NiAVObject`'s world bound's radius.
const BOUND_RADIUS: usize = 0x2C;
/// A reference's 3D (`TESObjectREFR` +0x3C): the player's, their body as seen in third person.
const REFR_NODE: usize = 0x3C;

/// Under the big four the interface's own 3D holds a figure of the player (as big as a person:
/// its bound is about 88 units across), drawn with the menus' own camera into the UI, which the
/// flat game shows beside its inventory. In VR it lands in the HUD texture under the floating
/// pages, a dark silhouette behind them, so it's culled while the UI is drawn (`hide` as the UI
/// starts, after the eyes; not, at Present, once it's drawn). Under any other menu (the race
/// menu's face) it's the game's as ever.
#[cfg(all(windows, target_arch = "x86"))]
pub fn hide_menu_3d(hide: bool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use crate::engine::av::{APP_CULLED, FLAGS};
    /// It was culled here, and wasn't before.
    static CULLED: AtomicBool = AtomicBool::new(false);
    let ui3d = mem::deref(mem::deref(INTERFACE_MANAGER) + MENU_3D);
    let Some(flags) = (ui3d != 0).then(|| mem::at::<u16>(ui3d + FLAGS)).flatten() else { return };
    if hide {
        if flags & APP_CULLED == 0 && crate::tiles::top_menu() == crate::tiles::BIG_FOUR {
            unsafe { ((ui3d + FLAGS) as *mut u16).write(flags | APP_CULLED) };
            CULLED.store(true, Ordering::Relaxed);
        }
    } else if CULLED.swap(false, Ordering::Relaxed) {
        unsafe { ((ui3d + FLAGS) as *mut u16).write(flags & !APP_CULLED) };
    }
}

/// Testing what's drawn behind an open menu besides the world: how big the interface's own 3D is
/// (None or 0: it draws nothing), and whether the player's body is hidden (in first person it
/// should be).
pub fn log_drawn_under() {
    let ui3d = mem::deref(mem::deref(INTERFACE_MANAGER) + MENU_3D);
    let radius = (ui3d != 0).then(|| mem::at::<f32>(ui3d + BOUND_RADIUS)).flatten();
    let body = mem::deref(mem::deref(PLAYER) + REFR_NODE);
    let hidden = (body != 0).then(|| mem::at::<u16>(body + crate::engine::av::FLAGS)).flatten().map(|f| f & crate::engine::av::APP_CULLED != 0);
    tracing::info!("menus: under the menu, the game's own 3D (Menu3DRoot) is {radius:?} across; the player's body hidden {hidden:?}");
}

/// The menu of this type is open (`kMenuType_*`).
pub fn menu_open(kind: u32) -> bool {
    tile_menu(kind) != 0
}

/// The open menu of this type's tile (`TileMenu*`), or 0.
pub fn tile_menu(kind: u32) -> usize {
    let data = mem::deref(TILE_MENUS + 4);
    if kind >= MENU_FIRST && data != 0 { mem::deref(data + 4 * (kind - MENU_FIRST) as usize) } else { 0 }
}

/// A game is loaded and the player is in it: in a cell, not at the main menu or a loading screen.
pub fn in_world() -> bool {
    let player = mem::deref(PLAYER);
    player != 0 && mem::deref(player + PARENT_CELL) != 0 && !menu_open(MENU_MAIN) && !menu_open(MENU_LOADING)
}

const SCREEN_W: usize = 0x00B0_6C4C;
const SCREEN_H: usize = 0x00B0_6C50;

/// A menu is open (the game's `IsMenuMode`).
#[cfg(target_arch = "x86")]
pub fn menu_mode() -> bool {
    unsafe { std::mem::transmute::<usize, unsafe extern "C" fn() -> u8>(IS_MENU_MODE)() != 0 }
}

#[cfg(not(target_arch = "x86"))]
pub fn menu_mode() -> bool {
    false
}

/// The pause menu and its own (options and their pages, load, save; the main menu): Escape closes
/// them. Any other menu (a container, dialogue, the big four...) closes with Menu Mode, and Escape
/// would open the pause menu over it.
const ESCAPE_CLOSES: [u32; 10] = [0x3F5, 0x3F7, 0x3F9, 0x3FA, 0x3FB, 0x3FC, 0x3FD, 0x40E, 0x40F, MENU_MAIN];

/// A menu in the world is on top, one the game's Menu Mode closes.
#[cfg(target_arch = "x86")]
pub fn tab_closes() -> bool {
    menu_mode() && in_world() && !ESCAPE_CLOSES.contains(&crate::tiles::top_menu())
}

#[cfg(not(target_arch = "x86"))]
pub fn tab_closes() -> bool {
    false
}

/// Where a point on the screen (0..1 from the top left) is in menu units (x, height).
pub fn menu_point(uv: [f32; 2], width: f32, height: f32) -> (f32, f32) {
    let (w, h) = if height >= width { (1280.0, height / width * 1280.0) } else { (width / height * 960.0, 960.0) };
    ((uv[0] - 0.5) * w, (0.5 - uv[1]) * h)
}

/// The game's cursor to `uv` on the screen.
pub fn point(uv: [f32; 2]) {
    let node = mem::deref(mem::deref(mem::deref(INTERFACE_MANAGER) + CURSOR) + TILE_NODE);
    let (Some(w), Some(h)) = (mem::at::<u32>(SCREEN_W), mem::at::<u32>(SCREEN_H)) else { return };
    if node == 0 || w == 0 || h == 0 || !mem::is_readable((node + NODE_X) as *const u8, 12) {
        return;
    }
    let (x, z) = menu_point(uv, w as f32, h as f32);
    unsafe {
        ((node + NODE_X) as *mut f32).write(x);
        ((node + NODE_HEIGHT) as *mut f32).write(z);
    }
    hover_tick();
}

/// The tile the game's cursor is over (`InterfaceManager` +0x98), as the game worked it out.
const ACTIVE_TILE: usize = 0x98;

/// A light tick in the laser's hand as the game's own hover moves onto another thing (a row, a tab,
/// a button): the laser is felt crossing what it can click, as the game shows it.
fn hover_tick() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static LAST: AtomicUsize = AtomicUsize::new(0);
    let active = mem::deref(mem::deref(INTERFACE_MANAGER) + ACTIVE_TILE);
    if LAST.swap(active, Ordering::Relaxed) != active && active != 0 {
        crate::link::with_adapter(|a| a.haptic(rustvr_protocol::HAPTIC_FIRING_HAND, 0.008, 200.0, 0.12));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_screen_is_960_menu_units_high() {
        let (x, z) = super::menu_point([0.5, 0.5], 2560.0, 1440.0);
        assert_eq!((x, z), (0.0, 0.0));
        let (x, z) = super::menu_point([0.0, 0.0], 2560.0, 1440.0);
        assert!((x + 2560.0 / 1440.0 * 480.0).abs() < 1e-3 && (z - 480.0).abs() < 1e-3);
    }
}
