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
