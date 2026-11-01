//! The game's own card of a thing, for the thing in a hand: Oblivion's HUD info
//! (`menus\main\hud_info_menu.xml`: the name of what the crosshair is on and its numbers beside
//! their icons: value, weight, damage, armour, condition, uses, quality, a lock), drawn by the game
//! in its font at the screen's bottom right. While a hand has one of the game's things
//! ([`crate::items`]), it's that thing's: the game's own setting of its target (0x5A4980, called
//! as the crosshair's target changes) is given the held thing instead. A thing that can be taken,
//! only looked at, is given as nothing: it has no card until a hand has it. Where the card is on
//! the HUD is read from the game's own layout of it (its tiles' places and sizes, as
//! [`crate::tiles`] reads any menu's), so the runtime can put that piece of the HUD over the held
//! thing ([`rustvr_protocol::HUD_HELD`]). A thing a hand's fingers are on in someone's pocket has
//! its card too ([`crate::pockets`]).
//!
//! **Both hands:** the card is of one thing at a time, so the runtime draws each hand's card from a
//! picture of it, cut from the HUD while it was of that hand's thing (`HudLayout::held` says which,
//! once the game has laid it out). With a thing in each hand, the card is of each in turn till both
//! have their picture, then stays: of a thing new to a hand first.
//!
//! **A hand put up:** what the game picks to use is along a free hand put up in front
//! ([`crate::pointing`]), and the card of what it picks floats over that hand: a gate's, a
//! person's. Nothing's card shows where the eyes look.
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

pub use crate::tiles::{grow, uv_of};

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::mem;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    const SET_TARGET: usize = 0x005A_4980;
    const HUD_INFO_MENU: usize = 0x00B3_B33C;
    const MENU_TILE: usize = 0x4;
    const TILE_VALUES: usize = 0x14;
    const TILE_CHILDREN: usize = 0x30;
    const LIST_FIRST: usize = 0x4;
    const NODE_NEXT: usize = 0x0;
    const NODE_DATA: usize = 0x8;
    const VALUE_NUM: usize = 0x4;
    const VALUE_ID: usize = 0x18;
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
    /// What the crosshair is on, as the game set it (0: nothing); the card may be another's.
    static ASKED: AtomicUsize = AtomicUsize::new(0);
    /// The last thing the crosshair was on could be taken.
    static LAST_TAKEABLE: AtomicBool = AtomicBool::new(false);
    /// The card's target (`HUDInfoMenu` +0x54), for when the setting of it isn't hooked.
    const TARGET: usize = 0x54;
    /// A thing on a hand the card can be of: the id the runtime knows its card by (an object's own,
    /// or a pocket thing's template), the reference the game's card is given, and whose taking it is
    /// a crime against: the thing itself (`world`: the world's, someone's or not), the person whose
    /// pocket it's in (`from`), or nobody (0: the player's).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Thing {
        pub key: u32,
        pub refr: usize,
        pub from: usize,
        pub world: bool,
    }

    /// How long the game takes to show a new thing on its card (seconds): it lays out the card's
    /// lines over a few frames (a red line's box settled 0.1 s after it was set), and meanwhile the
    /// card is of what it was (at first, what the crosshair's on: a person's name). And how long the
    /// runtime has it after that to cut its picture (it does every frame the layout says).
    const SETTLE: f64 = 0.15;
    const PICTURE: f64 = 0.1;

    struct Card {
        /// The things of the world's in hands ([`crate::items`]), and the pockets' in hands or under
        /// fingers ([`crate::pockets`]).
        world: Vec<Thing>,
        pockets: Vec<Thing>,
        /// The thing the card is of, and since when (the link's clock).
        shown: Option<Thing>,
        since: f64,
        /// When each thing on a hand came to it, and those the runtime has a picture of by now.
        came: Vec<(u32, f64)>,
        pictured: Vec<u32>,
        /// The thing the layout last said the card is of (0: none).
        told: u32,
    }

    static CARD: Mutex<Card> = Mutex::new(Card { world: Vec::new(), pockets: Vec::new(), shown: None, since: 0.0, came: Vec::new(), pictured: Vec::new(), told: 0 });
    /// The card's action line: the reticle's icon for the thing (the n-th of its sheet), the game's
    /// word for it (its setting's text, 0: none) and whether using it is a crime (in the reticle's
    /// stealing red).
    #[derive(Clone, Copy, PartialEq)]
    struct Action {
        icon: u32,
        word: usize,
        crime: bool,
    }

    /// The card's action line as it's to be, or none ([`card_now`]).
    static ACTION: Mutex<Option<Action>> = Mutex::new(None);

    /// The reference the card's given for a thing on a hand, or 0.
    fn held() -> usize {
        CARD.lock().unwrap().shown.map_or(0, |t| t.refr)
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
            n == name.len() && p != 0 && mem::bytes(p as *const u8, n).is_some_and(|b| b == name.as_bytes())
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

    /// Where the game's card is on its HUD now (the part of it, 0..1), if it shows one.
    pub fn card_uv() -> Option<[f32; 4]> {
        let tile = mem::deref(mem::deref(HUD_INFO_MENU) + MENU_TILE);
        let screen = crate::tiles::screen()?;
        // (Its name is right-aligned and grows to the left as the game sets it, a frame or more
        // after the box is measured: "Gold (50)" was cut to "ld (50)". Room to the left for it;
        // what's there is the HUD's black, nothing.)
        let b = crate::tiles::Tree::read(tile).shown()?;
        let uv = super::uv_of([b[0] - NAME_ROOM, b[1], b[2], b[3]], screen, 6.0)?;
        Some([uv[0].min(uv[2] - CARD_WIDTH).max(0.0), uv[1], uv[2], uv[3]])
    }

    /// Where an open menu (`kMenuType_*`) is on the game's picture (the part of it, 0..1): the box of
    /// its tiles that draw something (images and text, not the rects that lay them out, which may
    /// reach past them: the race menu's for dragging the player round).
    pub fn menu_uv(kind: u32) -> Option<[f32; 4]> {
        let tree = crate::tiles::Tree::read(crate::tiles::menu_tile(kind));
        let top = tree.0.first()?;
        super::uv_of(tree.drawn(&top.name)?, crate::tiles::screen()?, 8.0)
    }

    unsafe extern "C" fn set_target(target: usize, a: u8, force: u8) -> u8 {
        let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        ASKED.store(target, Ordering::Release);
        let takeable = target != 0 && crate::items::takeable_ref(target).is_some();
        if target != 0 {
            LAST_TAKEABLE.store(takeable, Ordering::Release);
        }
        // A thing that can be taken gets no card while it's only looked at, not even for a moment:
        // the game fades its card out over 0.15 s once the crosshair leaves, so a card left to the
        // HUD's cutting flashes up as the gaze passes over things.
        let held = held();
        let shown = unsafe { real(if held != 0 { held } else if takeable { 0 } else { target }, a, force) };
        crate::hooks::guard_void("hud info action", || card_now(target));
        shown
    }

    /// What the crosshair is on (0: nothing), whatever the card is of.
    pub fn looked_at() -> usize {
        if REAL.load(Ordering::Acquire) == 0 {
            return mem::deref(mem::deref(HUD_INFO_MENU) + TARGET);
        }
        ASKED.load(Ordering::Acquire)
    }

    /// Nothing of what the crosshair is on should show (no card, no activate icon): it's a thing
    /// that can be taken, only looked at, or nothing just after one (while the game's icon of it
    /// fades out).
    pub fn quiet() -> bool {
        let on = looked_at();
        !holding() && if on == 0 { LAST_TAKEABLE.load(Ordering::Acquire) } else { crate::items::takeable_ref(on).is_some() }
    }

    /// A thing on a hand has the card (one held, or under the fingers in a pocket).
    pub fn holding() -> bool {
        held() != 0
    }

    /// The things of the world's in hands now.
    pub fn set_world(things: Vec<Thing>) {
        let changed = {
            let mut c = CARD.lock().unwrap();
            let changed = c.world != things;
            c.world = things;
            changed
        };
        show(changed);
    }

    /// The things out of pockets in hands now, or under their fingers in one.
    pub fn set_pockets(things: Vec<Thing>) {
        let changed = {
            let mut c = CARD.lock().unwrap();
            let changed = c.pockets != things;
            c.pockets = things;
            changed
        };
        show(changed);
    }

    /// The reticle's icon for a reference (0: none), and whether using it is a crime (the reticle's
    /// own test: someone's thing, a living person while sneaking: their pocket).
    fn icon_of(refr: usize) -> (u32, bool) {
        type Of = unsafe extern "thiscall" fn(usize) -> u32;
        type Crime = unsafe extern "thiscall" fn(usize) -> u8;
        let icon = unsafe { std::mem::transmute::<usize, Of>(ICON_OF)(refr) } & 0xFF;
        (icon, icon != 0 && icon != ICON_TALK && unsafe { std::mem::transmute::<usize, Crime>(IS_CRIME)(refr) } != 0)
    }

    /// The card's action line for a thing on a hand: the reticle's icon for it and the game's word
    /// for taking it, when that's a crime (someone's thing, someone's pocket).
    fn action(t: Thing) -> Option<Action> {
        let (refr, word) = match t {
            Thing { world: true, refr, .. } => (refr, S_STEAL),
            Thing { from, .. } if from != 0 => (from, S_PICKPOCKET),
            _ => return None,
        };
        let (icon, crime) = icon_of(refr);
        crime.then_some(Action { icon, word, crime })
    }

    /// The card's action line for what a hand points at: the reticle's icon for it (a hand to search
    /// a chest or a body, a door, talking) beside its name, in the reticle's stealing red with the
    /// game's word when using it is a crime (taking from someone's chest, from a person's pocket).
    fn pointed(refr: usize) -> Option<Action> {
        let (icon, crime) = icon_of(refr);
        let word = if !crime { 0 } else if crate::combat::is_actor(refr) { S_PICKPOCKET } else { S_STEAL };
        (icon != 0).then_some(Action { icon, word, crime })
    }

    /// The card now shows `target` (0: nothing, as it fades out): its action line decided with it, and
    /// set in the same frame, so the card is never drawn without it first. A thing a hand has, or one
    /// a hand's put up towards ([`crate::pointing`]), as [`action`] says (the reticle's red, its
    /// own sign of a crime, isn't shown); anything else, none; nothing, the line as it was, fading
    /// out with the card it's on.
    fn card_now(target: usize) {
        let shown = CARD.lock().unwrap().shown;
        if let Some(t) = shown {
            *ACTION.lock().unwrap() = action(t);
        } else if target != 0 {
            *ACTION.lock().unwrap() = crate::pointing::hand().and_then(|_| pointed(target));
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
        let Some(Action { icon: kind, word, crime }) = action else {
            set_value(menu, USER2, FALSE);
            return;
        };
        let reticle = mem::deref(RETICLE_RECT);
        let (Some(icon), Some(words), Some(activate)) = (child(menu, ACTION_ICON), child(menu, ACTION_TEXT), (reticle != 0).then(|| child(reticle, RETICLE_ACTIVATE)).flatten()) else { return };
        // A crime in the reticle's stealing red, else its white.
        let [Some(r), Some(g), Some(b)] = (if crime { [c"_red_stealing", c"_green_stealing", c"_blue_stealing"].map(|n| value(reticle, trait_id(n))) } else { [Some(255.0); 3] }) else { return };
        // The word (or none).
        static NONE: [u8; 1] = [0];
        set_text(menu, USER0, if word != 0 { mem::deref(word) } else { NONE.as_ptr() as usize });
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
            if crime {
                tracing::info!("hud: the card says \"{}\" in the reticle's stealing red ({r} {g} {b}), with its icon {kind}", crate::combat::read_str(mem::deref(word)));
            } else {
                tracing::info!("hud: the card has the reticle's icon {kind} beside its name");
            }
        }
    }

    /// Which thing on a hand the card's to be of: one without its picture yet (the one it's of, else
    /// the newest to a hand), else the one it's of while a hand has it, else the newest.
    fn choose(c: &mut Card, now: f64) -> Option<Thing> {
        let mut things = c.world.clone();
        things.extend(c.pockets.iter().filter(|t| !c.world.iter().any(|w| w.key == t.key)));
        c.came.retain(|(k, _)| things.iter().any(|t| t.key == *k));
        for t in &things {
            if !c.came.iter().any(|(k, _)| *k == t.key) {
                c.came.push((t.key, now));
            }
        }
        // The one it's of has its picture by now (and keeps it while it's on a hand).
        c.pictured.retain(|k| things.iter().any(|t| t.key == *k));
        if let Some(s) = c.shown
            && now - c.since >= SETTLE + PICTURE
            && !c.pictured.contains(&s.key)
        {
            c.pictured.push(s.key);
        }
        let came = |t: &Thing| c.came.iter().find(|(k, _)| *k == t.key).map_or(now, |(_, at)| *at);
        let newest = |without: bool| things.iter().filter(|t| !without || !c.pictured.contains(&t.key)).max_by(|a, b| came(a).total_cmp(&came(b))).copied();
        let current = c.shown.and_then(|s| things.iter().find(|t| t.key == s.key).copied());
        match current {
            Some(t) if !c.pictured.contains(&t.key) => Some(t),
            _ => newest(true).or(current).or_else(|| newest(false)),
        }
    }

    /// The things on the hands changed (`changed`), or a frame went by: the card of the one it's to
    /// be of ([`choose`]), set through the game, its action line with it.
    fn show(changed: bool) {
        let now = crate::link::now();
        let (next, was) = {
            let mut c = CARD.lock().unwrap();
            let next = choose(&mut c, now);
            let was = c.shown;
            if next.map(|t| (t.key, t.refr)) != was.map(|t| (t.key, t.refr)) {
                c.since = now;
            }
            c.shown = next;
            (next, was)
        };
        let refr = next.map_or(0, |t| t.refr);
        if refr != was.map_or(0, |t| t.refr) && REAL.load(Ordering::Acquire) != 0 {
            let real: unsafe extern "C" fn(usize, u8, u8) -> u8 = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
            crate::hooks::guard_void("hud info", || unsafe {
                real(refr, 0, 0);
            });
            crate::hooks::guard_void("hud info action", || card_now(refr));
        } else if changed && next != was {
            // The same thing, now taken (or not): its action line again.
            crate::hooks::guard_void("hud info action", || card_now(refr));
        }
    }

    /// What the layout says the card's of: the thing on a hand it's of, once the game has had a
    /// moment to lay it out (0: none).
    fn held_now(now: f64) -> u32 {
        let c = CARD.lock().unwrap();
        c.shown.filter(|_| now - c.since >= SETTLE).map_or(0, |t| t.key)
    }

    /// Once a frame: whether the layout's to be told again, as the card has moved on the HUD or what
    /// it's of has changed. Its place only changes while it's of a thing on a hand (and the first
    /// time it shows anything): what else it shows (the crosshair's thing) isn't over a hand.
    pub fn moved() -> bool {
        let now = crate::link::now();
        show(false);
        let told = held_now(now);
        let mut changed = {
            let mut c = CARD.lock().unwrap();
            std::mem::replace(&mut c.told, told) != told
        };
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

    /// The thing on a hand the card's of, for the layout (0: none yet).
    pub fn told() -> u32 {
        CARD.lock().unwrap().told
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
