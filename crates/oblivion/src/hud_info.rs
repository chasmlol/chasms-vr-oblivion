//! The game's own card of a thing, for the thing in a hand: Oblivion's HUD info
//! (`menus\main\hud_info_menu.xml`: the name of what the crosshair is on and its numbers beside
//! their icons: value, weight, damage, armour, condition, uses, quality, a lock), drawn by the game
//! in its font at the screen's bottom right. While a hand has one of the game's things
//! ([`crate::items`]), it's that thing's: the game's own setting of its target (0x5A4980, called
//! as the crosshair's target changes) is given the held thing instead. Where the card is on the
//! HUD is read from the game's own layout of it (its tiles' places and sizes), so the runtime can
//! put that piece of the HUD over the held thing ([`rustvr_protocol::HUD_HELD`]). A thing a hand's
//! fingers are on in someone's pocket has its card too ([`crate::pockets`]).
//!
//! **Taking it is a crime:** the game's own sign is the reticle's activate icon in its stealing red
//! (`hud_reticle.xml`: its `user1`, set by 0x578D50 when 0x4DEBF0 says activating what the
//! crosshair's on is a crime: someone's thing, or a living person while sneaking, which is their
//! pickpocketing). While a hand has such a thing, or its fingers are on one in a pocket, the card
//! says so on its own action line: the button icon and word over its name that only the Xbox shows
//! (`hudinfo_action_icon`, `hudinfo_action_text`, shown by the card's `user2`). It's given the
//! reticle's own icon for the thing (its activate icons' sheet, the one 0x4DE980 picks: a hand for
//! most things) and the game's word (`sSteal`, or `sPickpocket` for someone's pocket), both in the
//! reticle's stealing red (its `_red_stealing`, `_green_stealing`, `_blue_stealing`).
//!
//! Tiles (xOBSE's `GameTiles.h`): a tile's values at +0x14 (a list: +4 its first node; a node is
//! next, previous, the value), a value's number at +4 and its id at +0x18 (u16); its children at
//! +0x30 (the same list). Ids: visible 0xFA1 (1 no, 2 yes), locus 0xFA6, y 0xFAC, x 0xFAD, height
//! 0xFCA, width 0xFCB. A tile's place is from its nearest ancestor with `locus` (the menu itself).
//! A value's text is at +8 (`BSStringT`: the characters, their count); a tile's name at +8 (the
//! same). Other values' ids by their names in the menus' files: `Tile::StrToStrID` (0x588EF0).

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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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
    /// Room left of the card for a longer name than it was measured with (the HUD's pixels).
    const NAME_ROOM: f32 = 96.0;
    /// How wide the card's part of the HUD is at least (of the HUD's width), from its right edge,
    /// where its lines end (they're right-aligned): the same for every thing, so its lines stay
    /// where they are as the thing changes or the red line comes and goes. Clear of the main HUD's
    /// pieces (its left half: the bars and the compass).
    const CARD_WIDTH: f32 = 0.34;

    /// The HUD reticle's `hudreticle_reticle` rect: its stealing red, and its activate icons.
    const RETICLE_RECT: usize = 0x00B3_B350;
    const RETICLE_ACTIVATE: &str = "hudreticle_reticle_activate";
    const ACTION_ICON: &str = "hudinfo_action_icon";
    const ACTION_TEXT: &str = "hudinfo_action_text";
    /// `TESObjectREFR` (thiscall): the reticle's icon for a reference (0x4DE980: 1 a hand, 6 a book,
    /// 7 talking...; the n-th of its sheet), and whether activating it is a crime (0x4DEBF0).
    const ICON_OF: usize = 0x004D_E980;
    const IS_CRIME: usize = 0x004D_EBF0;
    /// The icon for talking, never red (as 0x578D50 has it).
    const ICON_TALK: u32 = 7;
    /// `Tile::UpdateFloat` and `Tile::UpdateString` (thiscall: the tile, a value's id, the value: set,
    /// and what's drawn from it updated); `Tile::StrToStrID` (cdecl: a value's name to its id).
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const UPDATE_STRING: usize = 0x0058_CED0;
    const TRAIT_ID: usize = 0x0058_8EF0;
    /// The game settings `sSteal` and `sPickpocket` (their text at +0).
    const S_STEAL: usize = 0x00B3_8D68;
    const S_PICKPOCKET: usize = 0x00B3_8D70;
    const TILE_NAME: usize = 0x08;
    const VALUE_TEXT: usize = 0x08;
    const USER0: u16 = 0xFAE;
    const USER2: u16 = 0xFB0;
    const RED: u16 = 0xFCC;
    const GREEN: u16 = 0xFCD;
    const BLUE: u16 = 0xFCE;
    const FILENAME: u16 = 0xFE6;
    const FALSE: f32 = 1.0;

    static REAL: AtomicUsize = AtomicUsize::new(0);
    /// The thing of the world's a hand has (a reference), or 0; a thing out of a pocket a hand has,
    /// or its fingers are on (a reference made for its card, [`crate::pockets`]), or 0, and whose
    /// pocket it's in (the person, until it's the player's), or 0; and the one the card is now.
    static HELD: AtomicUsize = AtomicUsize::new(0);
    static POCKET_HELD: AtomicUsize = AtomicUsize::new(0);
    static POCKET_FROM: AtomicUsize = AtomicUsize::new(0);
    static SHOWN: AtomicUsize = AtomicUsize::new(0);
    /// The card's action line as it's to be: the icon and the word, or none ([`card_now`]).
    static ACTION: Mutex<Option<(u32, usize)>> = Mutex::new(None);
    /// How long the game takes to show a new thing on its card (seconds): it lays out the card's
    /// lines over a few frames (a red line's box settled 0.1 s after it was set), and meanwhile the
    /// card is of what it was: at first what the crosshair's on, a person's name. And how long the
    /// card stays over a hand after the hand has nothing (it fades out there).
    const SETTLE: f64 = 0.15;
    const LINGER: f64 = 0.3;
    /// When the card last became a hand's thing's, and when a hand last had one (the link's clock);
    /// whether it was over the hand when the layout was last told.
    static TIMES: Mutex<(f64, f64)> = Mutex::new((f64::MIN, f64::MIN));
    static OVER_HAND: AtomicBool = AtomicBool::new(false);

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

    /// A value's text (its characters, as the game keeps them: 0 for none).
    fn text_ptr(tile: usize, id: u16) -> usize {
        list(tile + TILE_VALUES).into_iter().find(|&v| mem::at::<u16>(v + VALUE_ID) == Some(id)).map_or(0, |v| mem::deref(v + VALUE_TEXT))
    }

    fn text(tile: usize, id: u16) -> String {
        let p = text_ptr(tile, id);
        if p == 0 { String::new() } else { crate::combat::read_str(p) }
    }

    fn child(tile: usize, name: &str) -> Option<usize> {
        list(tile + TILE_CHILDREN).into_iter().find(|&c| {
            let (p, n) = (mem::deref(c + TILE_NAME), mem::at::<u16>(c + TILE_NAME + 4).unwrap_or(0) as usize);
            n == name.len() && p != 0 && mem::is_readable(p as *const u8, n) && unsafe { std::slice::from_raw_parts(p as *const u8, n) } == name.as_bytes()
        })
    }

    /// The id of a value by its name in the menus' files (0: no such value).
    fn trait_id(name: &std::ffi::CStr) -> u16 {
        type Lookup = unsafe extern "C" fn(*const std::ffi::c_char) -> u32;
        (unsafe { std::mem::transmute::<usize, Lookup>(TRAIT_ID)(name.as_ptr()) }) as u16
    }

    /// Sets a tile's number, through the game, if it isn't that already.
    fn set_value(tile: usize, id: u16, v: f32) {
        if id != 0 && value(tile, id) != Some(v) {
            type Update = unsafe extern "thiscall" fn(usize, u32, f32);
            unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(tile, id as u32, v) };
        }
    }

    /// Sets a tile's text to the characters at `p` (the game copies them), if it isn't that already.
    fn set_text(tile: usize, id: u16, p: usize) {
        if p != 0 && text(tile, id) != crate::combat::read_str(p) {
            type Update = unsafe extern "thiscall" fn(usize, u32, usize);
            unsafe { std::mem::transmute::<usize, Update>(UPDATE_STRING)(tile, id as u32, p) };
        }
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
        // (Its name is right-aligned and grows to the left as the game sets it, a frame or more
        // after the box is measured: "Gold (50)" was cut to "ld (50)". Room to the left for it;
        // what's there is the HUD's black, nothing.)
        let b = b?;
        let uv = super::uv_of([b[0] - NAME_ROOM, b[1], b[2], b[3]], screen, 6.0)?;
        Some([uv[0].min(uv[2] - CARD_WIDTH).max(0.0), uv[1], uv[2], uv[3]])
    }

    unsafe extern "C" fn set_target(target: usize, a: u8, force: u8) -> u8 {
        let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        let held = held();
        let shown = unsafe { real(if held != 0 { held } else { target }, a, force) };
        crate::hooks::guard_void("hud info action", || card_now(target));
        shown
    }

    /// The thing of the world's a hand has now (0: none): the card is its.
    pub fn set_held(refr: usize) {
        HELD.store(refr, Ordering::Release);
        show();
    }

    /// The thing out of a pocket a hand has now, or its fingers are on (a reference made for it; 0:
    /// none), and the person whose pocket it's in (0: it's the player's): the card is its, while no
    /// hand has a thing of the world's.
    pub fn set_pocket_held(refr: usize, from: usize) {
        POCKET_FROM.store(from, Ordering::Release);
        POCKET_HELD.store(refr, Ordering::Release);
        show();
    }

    /// The card's action line now: the reticle's icon for the thing and the game's word for taking
    /// it, when that's a crime (the reticle's own test: someone's thing, someone's pocket).
    fn action() -> Option<(u32, usize)> {
        type Of = unsafe extern "thiscall" fn(usize) -> u32;
        type Crime = unsafe extern "thiscall" fn(usize) -> u8;
        let (refr, word) = match (HELD.load(Ordering::Acquire), POCKET_HELD.load(Ordering::Acquire), POCKET_FROM.load(Ordering::Acquire)) {
            (0, 0, _) | (0, _, 0) => return None,
            (0, _, from) => (from, S_PICKPOCKET),
            (held, _, _) => (held, S_STEAL),
        };
        let icon = unsafe { std::mem::transmute::<usize, Of>(ICON_OF)(refr) } & 0xFF;
        let crime = icon != 0 && icon != ICON_TALK && unsafe { std::mem::transmute::<usize, Crime>(IS_CRIME)(refr) } != 0;
        crime.then_some((icon, word))
    }

    /// The card now shows `target` (0: nothing, as it fades out): its action line decided with it, and
    /// set in the same frame, so the card is never drawn without it first. A thing a hand has, as
    /// [`action`] says; a thing the crosshair's on, none (the reticle's red is the game's own sign
    /// there); nothing, the line as it was, fading out with the card it's on.
    fn card_now(target: usize) {
        if held() != 0 {
            *ACTION.lock().unwrap() = action();
        } else if target != 0 {
            *ACTION.lock().unwrap() = None;
        }
        keep_action();
    }

    /// Before each picture (and as the card's thing changes): the card's action line shown as
    /// [`card_now`] decided, or hidden, whenever the game's tiles aren't so already.
    pub fn keep_action() {
        let menu = mem::deref(mem::deref(HUD_INFO_MENU) + MENU_TILE);
        if menu == 0 {
            return;
        }
        let action = *ACTION.lock().unwrap();
        let Some((kind, word)) = action else {
            set_value(menu, USER2, FALSE);
            return;
        };
        let reticle = mem::deref(RETICLE_RECT);
        let (Some(icon), Some(words), Some(activate)) = (child(menu, ACTION_ICON), child(menu, ACTION_TEXT), (reticle != 0).then(|| child(reticle, RETICLE_ACTIVATE)).flatten()) else { return };
        let [Some(r), Some(g), Some(b)] = [c"_red_stealing", c"_green_stealing", c"_blue_stealing"].map(|n| value(reticle, trait_id(n))) else { return };
        // The word, in the stealing red.
        set_text(menu, USER0, mem::deref(word));
        for (id, v) in [(RED, r), (GREEN, g), (BLUE, b)] {
            set_value(words, id, v);
            set_value(icon, id, v);
        }
        // The reticle's icon: its sheet and size, the thing's icon on it (the n-th, as the reticle's
        // own crop has it), in the stealing red.
        set_text(icon, FILENAME, text_ptr(activate, FILENAME));
        let (Some(w), Some(h)) = (value(activate, WIDTH), value(activate, HEIGHT)) else { return };
        set_value(icon, WIDTH, w);
        set_value(icon, HEIGHT, h);
        set_value(icon, trait_id(c"cropx"), w * (kind - 1) as f32);
        if value(menu, USER2) != Some(TRUE) {
            set_value(menu, USER2, TRUE);
            tracing::info!("hud: the card says \"{}\" in the reticle's stealing red ({r} {g} {b}), with its icon {kind}", crate::combat::read_str(mem::deref(word)));
        }
    }

    fn show() {
        let refr = held();
        let was = SHOWN.swap(refr, Ordering::AcqRel);
        if was == 0 && refr != 0 {
            TIMES.lock().unwrap().0 = crate::link::now();
        }
        if was != refr && REAL.load(Ordering::Acquire) != 0 {
            let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
            crate::hooks::guard_void("hud info", || unsafe {
                real(refr, 0, 0);
            });
            crate::hooks::guard_void("hud info action", || card_now(refr));
        }
    }

    /// Whether the card is of a hand's thing now (the game has had a moment to show it), or was a
    /// moment ago (it's fading out over the hand).
    fn over_hand(now: f64) -> bool {
        let mut t = TIMES.lock().unwrap();
        if held() != 0 {
            t.1 = now;
            return now - t.0 >= SETTLE;
        }
        now - t.1 < LINGER
    }

    /// Once a frame: whether the layout's to be told again, as the card has moved on the HUD or come
    /// over a hand or gone from it. Its place only changes while it's a hand's thing's (and the first
    /// time it shows anything): what else it shows (the crosshair's thing, or nothing as it fades)
    /// isn't over a hand, and the card over the hand, fading, keeps the place it had.
    pub fn moved() -> bool {
        let over = over_hand(crate::link::now());
        let mut changed = OVER_HAND.swap(over, Ordering::AcqRel) != over;
        let mut last = LAST.lock().unwrap();
        if (held() != 0 || last.is_none())
            && let Some(now) = card_uv()
            && last.is_none_or(|l| l.iter().zip(now).any(|(a, b)| (a - b).abs() > 0.01))
        {
            *last = Some(now);
            tracing::info!("hud: the game's card of a thing is at {now:.3?} of its HUD");
            changed = true;
        }
        changed
    }

    /// The card isn't of the thing in a hand yet (or there's none).
    pub fn pending() -> bool {
        !OVER_HAND.load(Ordering::Acquire)
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
