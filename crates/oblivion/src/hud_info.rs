//! The game's own card of a thing, for the thing in a hand: Oblivion's HUD info
//! (`menus\main\hud_info_menu.xml`: the name of what the crosshair is on and its numbers beside
//! their icons: value, weight, damage, armour, condition, uses, quality, a lock), drawn by the game
//! in its font at the screen's bottom right. While a hand has one of the game's things
//! ([`crate::items`]), it's that thing's: the game's own setting of its target (0x5A4980, called
//! as the crosshair's target changes) is given the held thing instead. Where the card is on the
//! HUD is read from the game's own layout of it (its tiles' places and sizes), so the runtime can
//! put that piece of the HUD over the held thing ([`rustvr_protocol::HUD_HELD`]).
//!
//! Its tiles are read as [`crate::tiles`] reads any menu's.

pub use crate::tiles::{grow, uv_of};

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::mem;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const SET_TARGET: usize = 0x005A_4980;
    const HUD_INFO_MENU: usize = 0x00B3_B33C;
    const MENU_TILE: usize = 0x4;

    static REAL: AtomicUsize = AtomicUsize::new(0);
    /// The thing a hand has (a reference), or 0.
    static HELD: AtomicUsize = AtomicUsize::new(0);

    /// Where the game's card is on its HUD now (the part of it, 0..1), if it shows one.
    pub fn card_uv() -> Option<[f32; 4]> {
        let tile = mem::deref(mem::deref(HUD_INFO_MENU) + MENU_TILE);
        let screen = crate::tiles::screen()?;
        super::uv_of(crate::tiles::Tree::read(tile).shown()?, screen, 6.0)
    }

    unsafe extern "C" fn set_target(target: usize, a: u8, force: u8) -> u8 {
        let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        let held = HELD.load(Ordering::Acquire);
        unsafe { real(if held != 0 { held } else { target }, a, force) }
    }

    /// A hand has one of the game's things (the card is its).
    pub fn holding() -> bool {
        HELD.load(Ordering::Acquire) != 0
    }

    /// The thing a hand has now (0: none): the card is its.
    pub fn set_held(refr: usize) {
        if HELD.swap(refr, Ordering::AcqRel) != refr && REAL.load(Ordering::Acquire) != 0 {
            let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
            crate::hooks::guard_void("hud info", || unsafe {
                real(refr, 0, 0);
            });
        }
    }

    pub fn install() {
        if crate::hooks::install_into(SET_TARGET as *const (), set_target as *const (), "HUDInfoMenu::SetTarget", &REAL) {
            tracing::info!("hud: the game's card of a thing is the held thing's while a hand has one");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cards_box_is_its_part_of_the_hud() {
        let b = grow(grow(None, [600.0, 650.0, 640.0, 682.0]), [400.0, 660.0, 590.0, 690.0]).unwrap();
        assert_eq!(b, [400.0, 650.0, 640.0, 690.0]);
        let uv = uv_of(b, (800.0, 720.0), 0.0).unwrap();
        assert!((uv[0] - 0.5).abs() < 1e-6 && (uv[3] - 690.0 / 720.0).abs() < 1e-6);
        assert!(uv_of([5.0, 5.0, 5.0, 9.0], (800.0, 720.0), 0.0).is_none(), "nothing shown");
        assert_eq!(uv_of([790.0, 0.0, 800.0, 10.0], (800.0, 720.0), 20.0).unwrap()[2], 1.0, "kept on the HUD");
    }
}
