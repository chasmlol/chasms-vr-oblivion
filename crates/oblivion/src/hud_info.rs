//! The game's own card of a thing, for the thing in a hand: Oblivion's HUD info
//! (`menus\main\hud_info_menu.xml`: the name of what the crosshair is on and its numbers beside
//! their icons: value, weight, damage, armour, condition, uses, quality, a lock), drawn by the game
//! in its font at the screen's bottom right. While a hand has one of the game's things
//! ([`crate::items`]), it's that thing's: the game's own setting of its target (0x5A4980, called
//! as the crosshair's target changes) is given the held thing instead. Where the card is on the
//! HUD is read from the game's own layout of it (its tiles' places and sizes), so the runtime can
//! put that piece of the HUD over the held thing ([`rustvr_protocol::HUD_HELD`]).
//!
//! Tiles (xOBSE's `GameTiles.h`): a tile's values at +0x14 (a list: +4 its first node; a node is
//! next, previous, the value), a value's number at +4 and its id at +0x18 (u16); its children at
//! +0x30 (the same list). Ids: visible 0xFA1 (1 no, 2 yes), locus 0xFA6, y 0xFAC, x 0xFAD, height
//! 0xFCA, width 0xFCB. A tile's place is from its nearest ancestor with `locus` (the menu itself).

/// A box `[x0, y0, x1, y1]` grown to take in another.
pub fn grow(b: Option<[f32; 4]>, r: [f32; 4]) -> Option<[f32; 4]> {
    Some(match b {
        None => r,
        Some(b) => [b[0].min(r[0]), b[1].min(r[1]), b[2].max(r[2]), b[3].max(r[3])],
    })
}

/// A box in the screen's units as the part of the HUD it is (0..1), a little margin round it.
pub fn uv_of(b: [f32; 4], screen: (f32, f32), margin: f32) -> Option<[f32; 4]> {
    let (w, h) = screen;
    if !(w > 0.0 && h > 0.0) || b[2] <= b[0] || b[3] <= b[1] {
        return None;
    }
    let uv = [(b[0] - margin) / w, (b[1] - margin) / h, (b[2] + margin) / w, (b[3] + margin) / h];
    Some(uv.map(|v| v.clamp(0.0, 1.0)))
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::mem;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const SET_TARGET: usize = 0x005A_4980;
    const HUD_INFO_MENU: usize = 0x00B3_B33C;
    const INTERFACE: usize = 0x0058_2160;
    const MENU_ROOT: usize = 0x68;
    const MENU_TILE: usize = 0x4;
    const TILE_VALUES: usize = 0x14;
    const TILE_CHILDREN: usize = 0x30;
    const LIST_FIRST: usize = 0x4;
    const NODE_NEXT: usize = 0x0;
    const NODE_DATA: usize = 0x8;
    const VALUE_NUM: usize = 0x4;
    const VALUE_ID: usize = 0x18;
    const VISIBLE: u16 = 0xFA1;
    const LOCUS: u16 = 0xFA6;
    const Y: u16 = 0xFAC;
    const X: u16 = 0xFAD;
    const HEIGHT: u16 = 0xFCA;
    const WIDTH: u16 = 0xFCB;
    const TRUE: f32 = 2.0;

    static REAL: AtomicUsize = AtomicUsize::new(0);
    /// The thing of the world's a hand has (a reference), or 0; a thing out of a pocket a hand has
    /// (a reference made for its card, [`crate::pockets`]), or 0; and the one the card is now.
    static HELD: AtomicUsize = AtomicUsize::new(0);
    static POCKET_HELD: AtomicUsize = AtomicUsize::new(0);
    static SHOWN: AtomicUsize = AtomicUsize::new(0);

    /// The card's thing: the world's thing in a hand, else a pocket's.
    fn held() -> usize {
        let h = HELD.load(Ordering::Acquire);
        if h != 0 { h } else { POCKET_HELD.load(Ordering::Acquire) }
    }
    /// Where the card was last found on the HUD.
    static LAST: Mutex<Option<[f32; 4]>> = Mutex::new(None);

    fn list(at: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = mem::deref(at + LIST_FIRST);
        while node != 0 && out.len() < 256 {
            let data = mem::deref(node + NODE_DATA);
            if data != 0 {
                out.push(data);
            }
            node = mem::deref(node + NODE_NEXT);
        }
        out
    }

    fn value(tile: usize, id: u16) -> Option<f32> {
        list(tile + TILE_VALUES).into_iter().find(|&v| mem::at::<u16>(v + VALUE_ID) == Some(id)).and_then(|v| mem::at::<f32>(v + VALUE_NUM)).filter(|v| v.is_finite())
    }

    /// The box of what's shown under `tile` (its place from `origin`, its locus's).
    fn shown(tile: usize, origin: (f32, f32), depth: u32, out: &mut Option<[f32; 4]>) {
        if depth > 8 || value(tile, VISIBLE).is_some_and(|v| v != TRUE) {
            return;
        }
        let (x, y) = (origin.0 + value(tile, X).unwrap_or(0.0), origin.1 + value(tile, Y).unwrap_or(0.0));
        let (w, h) = (value(tile, WIDTH).unwrap_or(0.0), value(tile, HEIGHT).unwrap_or(0.0));
        if w > 0.0 && h > 0.0 {
            *out = super::grow(*out, [x, y, x + w, y + h]);
        }
        let inner = if value(tile, LOCUS) == Some(TRUE) { (x, y) } else { origin };
        for c in list(tile + TILE_CHILDREN) {
            shown(c, inner, depth + 1, out);
        }
    }

    /// Where the game's card is on its HUD now (the part of it, 0..1), if it shows one.
    pub fn card_uv() -> Option<[f32; 4]> {
        type Interface = unsafe extern "C" fn(u8, u8) -> usize;
        let menu = mem::deref(HUD_INFO_MENU);
        let tile = mem::deref(menu + MENU_TILE);
        let ui = unsafe { std::mem::transmute::<usize, Interface>(INTERFACE)(0, 1) };
        let root = mem::deref(ui + MENU_ROOT);
        if tile == 0 || root == 0 {
            return None;
        }
        let screen = (value(root, WIDTH)?, value(root, HEIGHT)?);
        let mut b = None;
        shown(tile, (0.0, 0.0), 0, &mut b);
        super::uv_of(b?, screen, 6.0)
    }

    unsafe extern "C" fn set_target(target: usize, a: u8, force: u8) -> u8 {
        let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        let held = held();
        unsafe { real(if held != 0 { held } else { target }, a, force) }
    }

    /// The thing of the world's a hand has now (0: none): the card is its.
    pub fn set_held(refr: usize) {
        HELD.store(refr, Ordering::Release);
        show();
    }

    /// The thing out of a pocket a hand has now (a reference made for it; 0: none): the card is its,
    /// while no hand has a thing of the world's.
    pub fn set_pocket_held(refr: usize) {
        POCKET_HELD.store(refr, Ordering::Release);
        show();
    }

    fn show() {
        let refr = held();
        if SHOWN.swap(refr, Ordering::AcqRel) != refr && REAL.load(Ordering::Acquire) != 0 {
            let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
            crate::hooks::guard_void("hud info", || unsafe {
                real(refr, 0, 0);
            });
        }
    }

    /// The card's place on the HUD, when it has moved since it was last told (for the layout).
    pub fn moved() -> Option<[f32; 4]> {
        let now = card_uv()?;
        let mut last = LAST.lock().unwrap();
        let far = last.is_none_or(|l| l.iter().zip(now).any(|(a, b)| (a - b).abs() > 0.01));
        if far {
            *last = Some(now);
            tracing::info!("hud: the game's card of a thing is at {now:.3?} of its HUD");
        }
        far.then_some(now)
    }

    pub fn last() -> Option<[f32; 4]> {
        *LAST.lock().unwrap()
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
