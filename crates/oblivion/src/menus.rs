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

/// One of the game's menus made for a moment, to work its own logic without its menu mode (as
/// alchemy's bench works the alchemy menu, a hammer's strike the repair menu): `open` makes it (the
/// game's own maker, which returns its tile); the interface is put back at once as it was (making a
/// menu pushes it on the stack of open menus, so the world would stop, and takes the cursor's
/// tiles), and the menu is unregistered (its id cleared: its own close then takes nothing off the
/// stack). `work` has the menu and its tile; then the menu is gone again (its fade off the
/// interface's list, its end). None: it didn't open as menu `kind`.
#[cfg(target_arch = "x86")]
pub fn for_a_moment<R>(kind: u32, open: impl FnOnce() -> usize, work: impl FnOnce(usize, usize) -> R) -> Option<R> {
    type This = unsafe extern "thiscall" fn(usize) -> usize;
    type Unfade = unsafe extern "C" fn(usize);
    type Destroy = unsafe extern "thiscall" fn(usize, u8) -> usize;
    let im = mem::deref(INTERFACE_MANAGER);
    if im == 0 {
        return None;
    }
    let stack: [u32; 10] = mem::at(im + STACK)?;
    let mode: u8 = mem::at(im + MODE)?;
    let under: Vec<usize> = UNDER.iter().map(|o| mem::deref(im + o)).collect();
    let background: u8 = mem::at(MENU_BACKGROUND)?;
    let tile = open();
    unsafe {
        ((im + STACK) as *mut [u32; 10]).write(stack);
        ((im + MODE) as *mut u8).write(mode);
        for (o, v) in UNDER.iter().zip(&under) {
            ((im + o) as *mut usize).write(*v);
        }
        (MENU_BACKGROUND as *mut u8).write(background);
    }
    let menu = if tile != 0 { unsafe { std::mem::transmute::<usize, This>(MENU_OF)(tile) } } else { 0 };
    // (Its id: the menu's `GetID`, vtable +0x34.)
    let id = if menu != 0 { unsafe { std::mem::transmute::<usize, This>(mem::deref(mem::deref(menu) + 0x34))(menu) } } else { 0 };
    if menu == 0 || id != kind as usize {
        tracing::warn!("menus: the game's menu {kind:#x} didn't open for a moment");
        return None;
    }
    unsafe { ((menu + MENU_ID) as *mut u32).write(0) };
    let r = work(menu, tile);
    unsafe {
        ((menu + MENU_ID) as *mut u32).write(0);
        std::mem::transmute::<usize, Unfade>(UNFADE)(menu);
        let destroy = mem::deref(mem::deref(menu));
        std::mem::transmute::<usize, Destroy>(destroy)(menu, 1);
    }
    Some(r)
}

/// The open menu of a type (`Menu*`), or 0 (cdecl: the game's own, as a menu's maker checks for one
/// already open).
#[cfg(target_arch = "x86")]
pub fn open_menu(kind: u32) -> usize {
    type ByType = unsafe extern "C" fn(u32) -> usize;
    unsafe { std::mem::transmute::<usize, ByType>(MENU_BY_TYPE)(kind) }
}

/// The message box (`kMenuType_Message`): its buttons (+0x34, ten tiles) and its `HandleClick`
/// (thiscall: the tile's id, the tile; its vtable's fourth): button n is id 4 + n.
const MESSAGE: u32 = 0x3E9;
const MESSAGE_BUTTONS: usize = 0x34;
const MESSAGE_CLICK: usize = 0x005B_CDD0;
const FIRST_BUTTON: u32 = 4;
/// The game's corner notice (cdecl: the words, how long, 0, 0).
const NOTICE: usize = 0x0057_ADD0;

/// The message box open now (its `Menu*`), and how many buttons it has.
#[cfg(target_arch = "x86")]
pub fn message_box() -> Option<(usize, usize)> {
    let menu = crate::tiles::menu(MESSAGE);
    (menu != 0).then(|| (menu, (0..10).filter(|k| mem::deref(menu + MESSAGE_BUTTONS + 4 * k) != 0).count()))
}

/// Button `n` of the message box `menu` clicked, as the player's click does: what asked (a script,
/// the game) has its answer.
#[cfg(target_arch = "x86")]
pub fn click_message(menu: usize, n: u32) {
    type Click = unsafe extern "thiscall" fn(usize, u32, usize);
    unsafe { std::mem::transmute::<usize, Click>(MESSAGE_CLICK)(menu, FIRST_BUTTON + n, 0) };
}

/// Words in the game's corner notice for a couple of seconds, as its own messages are.
#[cfg(target_arch = "x86")]
pub fn notice(text: &str) {
    type Notice = unsafe extern "C" fn(*const u8, f32, u32, u32) -> u8;
    if let Ok(t) = std::ffi::CString::new(text) {
        unsafe { std::mem::transmute::<usize, Notice>(NOTICE)(t.as_ptr() as *const u8, 2.0, 0, 0) };
    }
}

/// `InterfaceManager` (`INTERFACE_MANAGER`): its mode (+8: 1 the game, 3 a menu), the tiles and menu
/// under the cursor (+0x88, +0x98, +0x9C), its stack of open menus (+0xE0, ten ids); the menu
/// background's flag (a menu opening sets it); a tile's menu (thiscall) and the open menu of a type
/// (cdecl); a menu's id once registered (+0x14) and its fade off the interface's list (cdecl).
const MODE: usize = 0x8;
const UNDER: [usize; 3] = [0x88, 0x98, 0x9C];
const STACK: usize = 0xE0;
const MENU_BACKGROUND: usize = 0x00B4_2D54;
const MENU_OF: usize = 0x0058_98F0;
const MENU_BY_TYPE: usize = 0x0058_9B70;
const MENU_ID: usize = 0x14;
const UNFADE: usize = 0x0058_3CF0;

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
