//! Oblivion's HUD and menus in VR, piece by piece (`rustvr_protocol::hud`). The game draws them as
//! ever, into the HUD texture with their alpha ([`crate::render`]); this says which part of it is
//! what and where it goes, read every frame from where the game has put its own tiles
//! ([`crate::tiles`]). Nothing here draws: a piece is a rectangle of what the game drew.
//!
//! **Menus** are cut into the parts that matter and float in the world as groups (`HudGroup`),
//! each where it belongs:
//!
//! | Menu | Where | Pieces |
//! |---|---|---|
//! | a container | beside it | its list (heads, scroll bar, gold, weight), its tabs, its buttons |
//! | barter | beside the merchant | the same, with the merchant's gold and haggle |
//! | dialogue | the speaker: their name over their head, their line under their face, the topics beside them | the name; the line; the topics and services |
//! | the inventory, magic, the map, the character | ahead of the player | the page or list; its tabs; the HUD's switcher between them (bars: character, weapon: inventory, spell: magic, compass: map); a description over the page |
//! | a count, a question, haggling | where the eyes look, the menu under it where it was | the box |
//! | a service from dialogue (spells, training, repair, recharge, persuasion) | beside the speaker | the menu, cut from the screen |
//! | a thing's own (a lock being picked, an altar's enchanting or spellmaking, a sigil stone) | beside the thing | the menu, cut from the screen |
//! | the race menu (a new character) | beside the mirror (the runtime's: `GS_MENU_IN_WORLD`, [`crate::render`]) | none here: the runtime shows the part of the picture it's in |
//! | anything else | ahead of the player | the menu, cut from the screen |
//! | a book or a note being read | over the book in the hand, turned to the eyes (or held in the off hand: one read from the inventory or a shelf) | a book's two pages, a note's scroll |
//! | the map drawn from the belt | held in the hand like paper | its page (the local or world map in its frame), its tabs |
//!
//! **Transparency**: the pieces are as opaque as the game draws them, and the rest of the screen is
//! clear, so the parchment is solid and cut out. A menu's pieces are pages (`HUDE_SOLID`): the
//! runtime draws them opaque wherever the game drew half or more, so no blending of the game's
//! leaves its parchment see-through. Half a second after a menu opens, how much of each piece the
//! UI covers in the HUD copy goes into the log ("hud: piece N ... is covered"). The menus' backgrounds (each menu's image tiles
//! named `..._background`) can be made see-through (`menu_background` in `rustvr-oblivion.toml`,
//! 1 by default: solid), so the world shows through them while the text and icons on them stay
//! solid: the game's own `alpha` of each, set through the game's own update of a tile's value (so
//! what it draws follows) whenever the game sets it afresh.
//!
//! A piece is named by its tiles in the game's own menu file (`menus\container_menu.xml`...): its
//! rectangle is the box of what the game shows under them now, so it follows the game (a button the
//! game hides leaves its piece). The pieces keep the game's own arrangement, drawn apart a little so
//! each floats on its own; the first is the one the others are arranged round. The laser points at
//! the ones that take clicks.
//!
//! **The HUD** is cut the same way, from its own menus' tiles, each piece where VR games put it:
//!
//! | Piece | From | Where |
//! |---|---|---|
//! | the health, magicka and fatigue bars, the weapon and the spell | `hud_main_menu.xml` | over the inside of the off-hand wrist while its palm is turned up |
//! | the compass | `hud_main_menu.xml` | over the bars |
//! | the spells and potions working on the player | `hud_main_menu.xml` | over the compass |
//! | what the reticle can activate (its crosshair is hidden, [`crate::reticle`]; nothing for a thing that can be taken until a hand has it), the sneak eye | `hud_reticle.xml` | where the head looks |
//! | the card of what it's on (name and numbers) | `hud_info_menu.xml` ([`crate::hud_info`]) | under that, just in front of the thing, for a person, a door, a container (not a thing that can be taken); over each hand's thing while hands have them (a picture of the card for each, `HudLayout::card_of`; a thing under the fingers in a pocket too) |
//! | a foe's health | `hud_reticle.xml` | over the foe |
//! | spoken lines | `hud_subtitle_menu.xml` | the caption strip |
//! | the breath meter, under water | `breath_menu.xml` ([`crate::swim`]) | where the game has it, at the top of the view |
//! | notices ("Iron Arrow added"), a region's name | `hud_subtitle_menu.xml`, `hud_main_menu.xml` | over the captions |

use rustvr_protocol::*;

/// Where a piece of a menu comes from: the box a tile has (a list's window, whatever it shows), the
/// box of what the game draws under a tile (its images and text that show), or of everything the
/// menu draws.
#[derive(Clone, Copy, Debug)]
pub enum From {
    Box(&'static str),
    Drawn(&'static str),
    All,
}

/// A piece of a menu: the tiles it's made of (in its group's menu, or in another: the HUD's),
/// whether the laser points at it, and whether it lies over the others (a description that pops up
/// over the page it's about) rather than beside them.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub menu: u32,
    pub from: &'static [From],
    pub point: bool,
    pub over: bool,
}

impl Piece {
    /// A piece of the group's own menu.
    pub const fn of(from: &'static [From], point: bool) -> Piece {
        Piece { menu: 0, from, point, over: false }
    }

    /// A description that pops up over the page (`menu`'s; 0: the group's own): just where the
    /// game puts it, lifted a little towards the eye, so it covers itself in the page under it.
    pub const fn over(menu: u32, from: &'static [From]) -> Piece {
        Piece { menu, from, point: false, over: true }
    }
}

/// How far a piece that lies over the others is lifted towards the eye (the group's units).
pub const LIFT: f32 = 0.02;

/// How far apart the pieces of a menu float (the HUD texture's height).
pub const GAP: f32 = 0.018;
/// A margin round each piece's box (the game's units), so nothing it draws is cut at its edge.
pub const MARGIN: f32 = 3.0;

/// The pieces' places in their group from their boxes on the screen (`[x0, y0, x1, y1]`, the game's
/// units, `screen` wide and high): the first in the middle, the others where the game has them
/// round it, each moved out from it by a `GAP` for every piece it's beyond (across, or up or down,
/// whichever way it's further out). Offsets are in the group's units (the screen's height), y up.
pub fn arrange(boxes: &[[f32; 4]], screen: (f32, f32)) -> Vec<[f32; 3]> {
    let Some(main) = boxes.first() else { return Vec::new() };
    let h = screen.1.max(1.0);
    let centre = |b: &[f32; 4]| ((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5);
    let half = |b: &[f32; 4]| ((b[2] - b[0]) * 0.5, (b[3] - b[1]) * 0.5);
    let (mc, mh) = (centre(main), half(main));
    // Which way each is from the first (0 across, 1 up or down), and how far out.
    let way = |b: &[f32; 4]| {
        let (c, hb) = (centre(b), half(b));
        let across = (c.0 - mc.0).abs() / (mh.0 + hb.0).max(1.0);
        let down = (c.1 - mc.1).abs() / (mh.1 + hb.1).max(1.0);
        if across > down { (0, (c.0 - mc.0).signum(), (c.0 - mc.0).abs()) } else { (1, (c.1 - mc.1).signum(), (c.1 - mc.1).abs()) }
    };
    boxes
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let c = centre(b);
            let (mut x, mut y) = ((c.0 - mc.0) / h, -(c.1 - mc.1) / h);
            if i > 0 {
                let (axis, sign, far) = way(b);
                // (A gap for it, and one for each piece between it and the first.)
                let between = boxes[1..].iter().filter(|o| {
                    let (a, s, f) = way(o);
                    a == axis && s == sign && f < far
                });
                let gap = GAP * (1 + between.count()) as f32;
                if axis == 0 {
                    x += gap * sign;
                } else {
                    y -= gap * sign;
                }
            }
            [x, y, 0.0]
        })
        .collect()
}

/// Pieces never overlap: where another piece lies over the edge of the first (the HUD's switcher
/// over the foot of the big four's page), the first is cut back to clear it, by twice the margin.
pub fn carve(boxes: &mut [[f32; 4]]) {
    let Some((first, rest)) = boxes.split_first_mut() else { return };
    for b in rest.iter() {
        let over = first[0] < b[2] && b[0] < first[2] && first[1] < b[3] && b[1] < first[3];
        if !over {
            continue;
        }
        let (fc, bc) = (((first[0] + first[2]) * 0.5, (first[1] + first[3]) * 0.5), ((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5));
        let across = (bc.0 - fc.0).abs() / (first[2] - first[0] + b[2] - b[0]).max(1.0);
        let down = (bc.1 - fc.1).abs() / (first[3] - first[1] + b[3] - b[1]).max(1.0);
        let mut cut = *first;
        if across > down {
            if bc.0 > fc.0 { cut[2] = b[0] - 2.0 * MARGIN } else { cut[0] = b[2] + 2.0 * MARGIN }
        } else if bc.1 > fc.1 {
            cut[3] = b[1] - 2.0 * MARGIN
        } else {
            cut[1] = b[3] + 2.0 * MARGIN
        }
        if cut[2] > cut[0] && cut[3] > cut[1] {
            *first = cut;
        }
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::{camera, engine, mem, tiles};
    use glam::Vec3;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tiles::BIG_FOUR;

    // The menus by type (`kMenuType_*`).
    const MENU_MESSAGE: u32 = 0x3E9;
    const MENU_INVENTORY: u32 = 0x3EA;
    const MENU_STATS: u32 = 0x3EB;
    const MENU_HUD: u32 = 0x3EC;
    const MENU_CONTAINER: u32 = 0x3F0;
    const MENU_DIALOG: u32 = 0x3F1;
    const MENU_SUBTITLES: u32 = 0x3F2;
    const MENU_PAUSE: u32 = 0x3F5;
    const MENU_LOCKPICK: u32 = 0x3F6;
    const MENU_QUANTITY: u32 = 0x3F8;
    const MENU_MAGIC: u32 = 0x3FE;
    const MENU_MAP: u32 = 0x3FF;
    /// The description of a spell pointed at in the magic menu, a menu of its own.
    const MENU_MAGIC_POPUP: u32 = 0x400;
    /// Haggling over a price (from barter).
    const MENU_NEGOTIATE: u32 = 0x401;
    const MENU_SPELLMAKING: u32 = 0x411;
    const MENU_ENCHANTMENT: u32 = 0x412;
    const MENU_SIGIL_STONE: u32 = 0x418;
    const MENU_BOOK: u32 = crate::books::MENU_BOOK;
    /// The breath meter (`BreathMenu`), shown while the player's head is under water.
    const MENU_BREATH: u32 = 0x415;
    /// Menus a thing in the world opens as it's activated (a lock, an altar, a sigil stone): they
    /// float beside it.
    const THINGS_OWN: [u32; 4] = [MENU_LOCKPICK, MENU_SPELLMAKING, MENU_ENCHANTMENT, MENU_SIGIL_STONE];
    const MENU_LAST: u32 = 0x41B;
    /// The reticle's tile (`hud_reticle.xml`), not a menu of its own: a pseudo-type for [`Trees`].
    const RETICLE: u32 = 0;

    /// `Tile::UpdateFloat(this, id, value)` (thiscall): sets a value and updates what's drawn from it.
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    /// The laser in the colours of the game's own cursor (`textures\menus\misc\cursor.dds`, a gold
    /// arrow): the cream of its highlight for the beam's thread, the gold of its blade for the glow
    /// round it and its dot.
    const POINTER: [u32; 2] = [0xFFF7_DEFF, 0xFFCB_39FF];
    /// `ContainerMenu`: the container (or the merchant, bartering).
    const CONTAINER_REF: usize = 0x44;
    /// `DialogMenu`: the speaker.
    const DIALOG_SPEAKER: usize = 0x60;
    /// A reference's form type (`TESForm`), 3D, and a node's world bound (centre, radius; game units).
    const FORM_TYPE: usize = 0x4;
    const REFR_NODE: usize = 0x3C;
    const NODE_BOUND: usize = 0x20;
    /// `kFormType_ACHR`, `kFormType_ACRE`: a character or a creature.
    const ACTORS: [u8; 2] = [0x32, 0x33];

    /// A container's list beside the container (`container_menu.xml`), three strips of its box: the
    /// list on its parchment (the column heads, the scroll bar, and under it the gold and the weight,
    /// or the merchant's name and gold), the category tabs, and the buttons (take all or haggle, the
    /// switch between the container's things and the player's, exit).
    const CONTAINER: &[Piece] = &[
        Piece::of(
            &[
                From::Box("cont_window"),
                From::Drawn("cont_headers_top"),
                From::Drawn("cont_scroll_bar"),
                From::Drawn("cont_player_money_icon"),
                From::Drawn("cont_npc_money_icon"),
                From::Drawn("cont_encumberance_icon"),
            ],
            true,
        ),
        Piece::of(&[From::Drawn("cont_tab_layout")], true),
        Piece::of(&[From::Drawn("cont_button_take_all"), From::Drawn("cont_button_haggle"), From::Drawn("cont_button_exit"), From::Drawn("cont_select_frame")], true),
    ];

    /// The HUD's frame in the big four (`hud_main_menu.xml`): the bars, the weapon, the spell and
    /// the compass, each the way to its menu (character, inventory, magic, the map).
    const SWITCHER: Piece = Piece { menu: MENU_HUD, from: &[From::Drawn("hudmain_background")], point: true, over: false };

    /// The inventory (`main\inventory_menu.xml`): its list as the container's (heads, scroll bar,
    /// the weight, armour and gold under it), its tabs, and the switcher.
    const INVENTORY: &[Piece] = &[
        Piece::of(&[From::Box("inv_window"), From::Drawn("inv_headers_top"), From::Drawn("inv_scroll_bar"), From::Drawn("inv_extra_data")], true),
        Piece::of(&[From::Drawn("inv_tabs")], true),
        SWITCHER,
    ];
    /// Magic (`main\magic_menu.xml`): its list of spells (heads, scroll bar, the cost and
    /// magnitude marks, the spell effectiveness under it), its tabs, and the switcher.
    const MAGIC: &[Piece] = &[
        Piece::of(&[From::Box("magic_window"), From::Drawn("magic_headers_top"), From::Drawn("magic_scroll_bar"), From::Drawn("magic_misc_elements"), From::Drawn("magic_effectivness")], true),
        // (The tabs' own icons: the page art they're drawn on reaches up under the spell
        // effectiveness, which is the list's.)
        Piece::of(MAGIC_TABS, true),
        SWITCHER,
        Piece::over(MENU_MAGIC_POPUP, &[From::All]),
    ];
    /// The character (`main\stats_menu.xml`): its page, its tabs, the switcher, and the description
    /// of what's pointed at over the page.
    const STATS: &[Piece] = &[
        Piece::of(&[From::Drawn("stat_background")], true),
        Piece::of(&[From::Drawn("stat_tabs")], true),
        SWITCHER,
        Piece::over(0, &[From::Drawn("stat_pop_background")]),
    ];
    /// The map and the journal (`main\map_menu.xml`): its page, its tabs, and the switcher.
    const MAP: &[Piece] = &[Piece::of(&[From::Drawn("map_background")], true), Piece::of(MAP_TABS, true), SWITCHER];
    const MAGIC_TABS: &[From] = &[From::Drawn("magic_tabs_p1"), From::Drawn("magic_tabs_p2"), From::Drawn("magic_tabs_p3"), From::Drawn("magic_tabs_p4"), From::Drawn("magic_tabs_p5")];
    const MAP_TABS: &[From] = &[From::Drawn("map_tabs_p1"), From::Drawn("map_tabs_p2"), From::Drawn("map_tabs_p3"), From::Drawn("map_tabs_p4"), From::Drawn("map_tabs_p5")];

    /// Dialogue (`dialog\dialog_menu.xml`) at the speaker: their line under their face (the game
    /// shows it while they speak, and the topics after)...
    const DIALOG_LINE: &[Piece] = &[Piece::of(&[From::Drawn("dialog_text")], true)];
    /// ...their name over their head...
    const DIALOG_NAME: &[Piece] = &[Piece::of(&[From::Drawn("dialog_name")], false)];
    /// ...and the topics and the services (persuade, barter, spells, repair, training, recharge,
    /// goodbye) beside them.
    const DIALOG_TOPICS: &[Piece] = &[Piece::of(&[From::Drawn("dialog_topics")], true)];

    /// A menu nobody has cut yet: the whole of what it draws, as one piece.
    const WHOLE: &[Piece] = &[Piece::of(&[From::All], true)];

    /// A lock picked at the keyhole (`lockpick_menu.xml`, [`crate::lock`]): its tumblers stand in the
    /// world, so only what it says floats beside them, on its own art: the lock's level, the picks
    /// left and the player's Security. (Its buttons are the controller's: A, B.)
    const LOCK: &[Piece] = &[
        Piece::of(&[From::Drawn("lockpick_level"), From::Drawn("lockpick_level_levelText")], false),
        Piece::of(&[From::Drawn("lockpick_picks_left_text"), From::Drawn("lockpick_picks_left")], false),
        Piece::of(&[From::Drawn("lockpick_playerlevel_text"), From::Drawn("lockpick_playerlevel")], false),
    ];
    /// How far round the keyhole the lock's tumblers reach (metres): what floats beside it stays
    /// clear of them.
    const LOCK_REACH: f32 = 0.08;

    /// The map drawn from the belt, in the hand: its page (the local or the world map in its frame,
    /// or a page of the journal, on its parchment) and its tabs; the big four's switcher stays
    /// behind.
    const MAP_HELD: &[Piece] = &[Piece::of(&[From::Drawn("map_background")], true), Piece::of(MAP_TABS, true)];

    /// How far an open book's two pages tip towards the eye about its spine (radians).
    const BOOK_FOLD: f32 = 0.12;

    /// The holsters' editor (`crate::slot_menu`), in strips as a container's list is: the
    /// categories of the slot picked in the middle, the slots on their left (with what each gives
    /// now), the buttons on their right, and its title over them.
    const HOLSTERS: &[Piece] = &[
        Piece::of(&[From::Drawn("hol_categories")], true),
        Piece::of(&[From::Drawn("hol_slots")], true),
        Piece::of(&[From::Drawn("hol_buttons")], true),
        Piece::of(&[From::Drawn("hol_title")], false),
    ];

    /// The menus' trees, each read once a frame as it's needed.
    #[derive(Default)]
    struct Trees(HashMap<u32, tiles::Tree>);

    impl Trees {
        fn get(&mut self, kind: u32) -> &tiles::Tree {
            self.0.entry(kind).or_insert_with(|| tiles::Tree::read(if kind == RETICLE { tiles::reticle_tile() } else { tiles::menu_tile(kind) }))
        }
    }

    /// The box of a piece on the screen now (the game's units), if any of it shows.
    fn piece_box(tree: &tiles::Tree, p: &Piece) -> Option<[f32; 4]> {
        p.from
            .iter()
            .filter_map(|f| match f {
                From::Box(name) => tree.rect(name),
                From::Drawn(name) => tree.drawn(name),
                From::All => tree.0.first().and_then(|top| tree.drawn(&top.name)),
            })
            .fold(None, tiles::grow)
    }

    /// Menu `kind`'s pieces as a group of `layout`; the box they take on the screen.
    fn add_group(layout: &mut HudLayout, group: HudGroup, kind: u32, pieces: &[Piece], screen: (f32, f32), trees: &mut Trees) -> Option<[f32; 4]> {
        let found: Vec<(&Piece, [f32; 4])> = pieces.iter().filter_map(|p| piece_box(trees.get(if p.menu != 0 { p.menu } else { kind }), p).map(|b| (p, b))).collect();
        if found.is_empty() {
            return None;
        }
        // (The pieces beside each other are cut apart and drawn apart; one over them is where the
        // game has it, nearer.)
        let mut boxes: Vec<[f32; 4]> = found.iter().map(|(_, b)| *b).collect();
        let beside: Vec<usize> = (0..found.len()).filter(|&i| !found[i].0.over).collect();
        let mut apart: Vec<[f32; 4]> = beside.iter().map(|&i| boxes[i]).collect();
        carve(&mut apart);
        for (&i, b) in beside.iter().zip(&apart) {
            boxes[i] = *b;
        }
        let mut offsets = vec![[0.0; 3]; boxes.len()];
        for (&i, o) in beside.iter().zip(arrange(&apart, screen)) {
            offsets[i] = o;
        }
        if let Some(&first) = beside.first() {
            let f = boxes[first];
            for i in (0..found.len()).filter(|&i| found[i].0.over) {
                let b = boxes[i];
                let h = screen.1.max(1.0);
                offsets[i] = [((b[0] + b[2]) - (f[0] + f[2])) * 0.5 / h, -((b[1] + b[3]) - (f[1] + f[3])) * 0.5 / h, LIFT];
            }
        }
        let n = layout.add_group(group);
        for ((p, _), (b, offset)) in found.iter().zip(boxes.iter().zip(offsets)) {
            if let Some(uv) = tiles::uv_of(*b, screen, MARGIN) {
                // (A menu is a page: solid where the game drew it, its cut edges soft.)
                let flags = HUDE_SOFT | HUDE_SOLID | if p.point { HUDE_POINT } else { 0 };
                layout.push(HudElement::piece(n, uv, offset, flags));
            }
        }
        boxes.into_iter().fold(None, tiles::grow)
    }

    /// Menu `kind`'s pieces as a group held in the hands, as `held` says. (Held, they're drawn as
    /// solid as what the game drew: it draws its book and map pages as light added, with no
    /// coverage.)
    fn add_held_group(layout: &mut HudLayout, group: HudGroup, held: HudHeld, kind: u32, pieces: &[Piece], screen: (f32, f32), trees: &mut Trees) -> Option<[f32; 4]> {
        let b = add_group(layout, group, kind, pieces, screen, trees)?;
        let n = layout.group_count as usize;
        layout.groups[n - 1].anchor = ANCHOR_HELD;
        layout.held[n - 1] = held;
        let count = layout.count as usize;
        for e in layout.elements[..count].iter_mut().filter(|e| e.group == n as u32) {
            e.flags |= HUDE_OPAQUE;
        }
        Some(b)
    }

    /// The book or note being read (`menus\book_menu.xml`), held: over the book in the hand when a
    /// hand has it, turned to the eyes however the hand turns the book, else in the off hand like a
    /// sheet (one read from the inventory, or from a shelf by looking at it until it's in the hand).
    /// A book is its art's two halves, its pages, tipped a little about the spine as an open book's
    /// are (the buttons, Take and Exit and the page turns, are on its pages); a note is its scroll,
    /// rollers and all. Its pages turn by a flick of the free hand.
    fn book(l: &mut HudLayout, screen: (f32, f32), trees: &mut Trees) {
        let Some((menu, refr)) = crate::books::open() else { return };
        // (Let go of, it's gone at once: as it fades out no hand has it, and it would be held as a
        // sheet in the off hand as one from the pack is.)
        if crate::books::let_go() {
            return;
        }
        let scroll = crate::books::is_scroll(menu);
        // (Over the book in the hand; a book read where it lies comes to the off hand and opens
        // there; one from the pack is held in the off hand as a sheet.)
        let (object, flags) = match (crate::books::held_book(), crate::books::lying_book()) {
            (Some(id), _) => (id, HELD_FACING | HELD_PAGES),
            (None, Some(id)) => (id, HELD_FACING | HELD_PAGES | HELD_BRING),
            _ => (0, HELD_SHEET | HELD_PAGES),
        };
        // (Its pages centred on its model, not on its collision box, which can be off it.)
        let model = (object != 0).then(|| crate::items::visual_centre(refr)).flatten();
        let flags = flags | if model.is_some() { HELD_CENTRE } else { 0 };
        let held = HudHeld { object, hand: HELD_OFF_HAND, flags, fold: if scroll { 0.0 } else { BOOK_FOLD }, centre: model.unwrap_or_default().to_array(), _pad: 0 };
        let id = MENU_BOOK ^ refr as u32 ^ object.rotate_left(8) ^ if scroll { 0x5C } else { 0 };
        let group = HudGroup { id, ..Default::default() };
        let tree = trees.get(MENU_BOOK);
        let h = screen.1.max(1.0);
        let boxes: Vec<[f32; 4]> = if scroll {
            tree.drawn("book_background_scroll").into_iter().collect()
        } else {
            // (Its two pages, split down the spine: nothing between them.)
            tree.rect("book_background")
                .map(|b| {
                    let mid = (b[0] + b[2]) * 0.5;
                    vec![[b[0], b[1], mid, b[3]], [mid, b[1], b[2], b[3]]]
                })
                .unwrap_or_default()
        };
        let Some(all) = boxes.iter().copied().fold(None, tiles::grow) else { return };
        let centre = ((all[0] + all[2]) * 0.5, (all[1] + all[3]) * 0.5);
        let n = l.add_held(group, held);
        if n == 0 {
            return;
        }
        // (A book's pages, as the map's, are drawn by the game as light added, with no coverage,
        // though they're solid paper: they're as solid as what the game drew.)
        let flags = HUDE_POINT | HUDE_OPAQUE;
        for b in boxes {
            let margin = if scroll { MARGIN } else { 0.0 };
            if let Some(uv) = tiles::uv_of(b, screen, margin) {
                let offset = [((b[0] + b[2]) * 0.5 - centre.0) / h, -((b[1] + b[3]) * 0.5 - centre.1) / h, 0.0];
                l.push(HudElement::piece(n, uv, offset, flags));
            }
        }
        static SAID: Mutex<u32> = Mutex::new(0);
        let mut said = SAID.lock().unwrap();
        if *said != id {
            *said = id;
            tracing::info!("hud: {} being read, {} (its model's middle {model:.3?})", if scroll { "a note" } else { "a book" }, if object != 0 { "over the book in the hand" } else { "in the off hand" });
        }
    }

    /// A menu moved on the screen to make room for a pop-up over it: its top tile, and how far down.
    static MOVED: Mutex<Option<(usize, f32)>> = Mutex::new(None);

    /// Moves a menu's top tile (and so all of it) down the screen by `dy` (the game's own update of
    /// its `y`, so what it draws and where it takes clicks follow).
    fn move_menu(tile: usize, dy: f32) {
        let y = tiles::value(tile, tiles::Y).unwrap_or(0.0);
        type Update = unsafe extern "thiscall" fn(usize, u32, f32);
        crate::hooks::guard_void("menu room", || unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(tile, tiles::Y as u32, y + dy) });
    }

    /// A pop-up (a count, a question) comes in the middle of the screen over the menu it's for, so
    /// cut out, part of it would show again in that menu's pieces. So they're put apart on the
    /// screen (each where it floats in the world is its own): the menu under it to the screen's
    /// foot, the pop-up in the room above. `under` and `popup`: their top tiles and the boxes their
    /// pieces take now.
    fn make_room(under: (usize, [f32; 4]), popup: (usize, [f32; 4]), screen: (f32, f32)) {
        let ((ut, u), (pt, p)) = (under, popup);
        let over = u[0] < p[2] && p[0] < u[2] && u[1] < p[3] && p[1] < u[3];
        let gap = 3.0 * MARGIN;
        if !over || (u[3] - u[1]) + (p[3] - p[1]) + 3.0 * gap > screen.1 {
            return;
        }
        let mut moved = MOVED.lock().unwrap();
        let down = (screen.1 - gap - u[3]).max(0.0);
        move_menu(ut, down);
        let room = u[1] + down - gap;
        move_menu(pt, (room - (p[3] - p[1])) * 0.5 - p[1]);
        let before = moved.filter(|&(t, _)| t == ut).map_or(0.0, |(_, d)| d);
        *moved = Some((ut, before + down));
        tracing::info!("hud: a pop-up made room for: the menu under it {down:.0} down the screen, the pop-up over it");
    }

    /// The menu moved for a pop-up back where it was (the pop-up gone).
    fn put_back() {
        let mut moved = MOVED.lock().unwrap();
        if let Some((tile, dy)) = moved.take()
            && [MENU_CONTAINER, MENU_INVENTORY, MENU_STATS, MENU_MAGIC, MENU_MAP].iter().any(|&k| tiles::menu_tile(k) == tile)
        {
            move_menu(tile, -dy);
        }
    }

    /// A menu's background image (its parchment), by the game's names for them.
    fn is_background(name: &str) -> bool {
        name.ends_with("_background") || name.ends_with("_background_2")
    }

    /// The menu's backgrounds a little see-through (see the module's notes): each time the game sets
    /// a background's alpha afresh, it's set to that much of it.
    fn see_through(tree: &tiles::Tree) {
        let k = crate::settings::get().menu_background.clamp(0.0, 1.0);
        if k >= 1.0 {
            return;
        }
        let mut ours = SEE_THROUGH.lock().unwrap();
        for p in tree.0.iter().filter(|p| p.kind == tiles::KIND_IMAGE && p.shown && is_background(&p.name)) {
            let Some(a) = p.alpha.filter(|&a| a > 0.0) else { continue };
            if ours.iter().any(|&(t, v)| t == p.tile && (v - a).abs() < 0.5) {
                continue;
            }
            let v = a * k;
            type Update = unsafe extern "thiscall" fn(usize, u32, f32);
            crate::hooks::guard_void("menu background", || unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(p.tile, tiles::ALPHA as u32, v) });
            ours.retain(|&(t, _)| t != p.tile);
            if ours.len() >= 64 {
                ours.remove(0);
            }
            ours.push((p.tile, v));
        }
    }

    /// The backgrounds made see-through ([`see_through`]): each tile, and the alpha it was given.
    static SEE_THROUGH: Mutex<Vec<(usize, f32)>> = Mutex::new(Vec::new());

    /// A menu's backgrounds as opaque as the game made them again (the map, drawn from the belt
    /// after it floated ahead: a sheet of paper isn't see-through).
    fn solid(tree: &tiles::Tree) {
        let k = crate::settings::get().menu_background.clamp(0.01, 1.0);
        let mut ours = SEE_THROUGH.lock().unwrap();
        for p in tree.0.iter().filter(|p| p.kind == tiles::KIND_IMAGE && is_background(&p.name)) {
            let Some(i) = ours.iter().position(|&(t, _)| t == p.tile) else { continue };
            let (_, v) = ours.remove(i);
            if p.alpha.is_some_and(|a| (a - v).abs() < 0.5) {
                type Update = unsafe extern "thiscall" fn(usize, u32, f32);
                crate::hooks::guard_void("menu background", || unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(p.tile, tiles::ALPHA as u32, (v / k).min(255.0)) });
            }
        }
    }

    fn is_actor(refr: usize) -> bool {
        mem::at::<u8>(refr + FORM_TYPE).is_some_and(|t| ACTORS.contains(&t))
    }

    /// A reference's 3D bound in the protocol's world (metres): its centre and radius.
    fn bound(refr: usize) -> Option<(Vec3, f32)> {
        let node = mem::deref(refr + REFR_NODE);
        let b: [f32; 4] = mem::at(node + NODE_BOUND)?;
        (node != 0 && b.iter().all(|v| v.is_finite()) && b[3] > 0.0).then(|| (camera::basis().point_from_game(Vec3::new(b[0], b[1], b[2])), b[3] / camera::UNITS_PER_METER))
    }

    /// An actor's head (its skeleton's head bone), in the protocol's world.
    fn head(actor: usize) -> Option<Vec3> {
        let node = mem::deref(actor + REFR_NODE);
        let head = *crate::combat::bones(node).get("Bip01 Head")?;
        let p: [f32; 3] = mem::at(head + engine::av::WORLD_TRANSLATE)?;
        Some(camera::basis().point_from_game(Vec3::from(p)))
    }

    /// A group beside a thing: a container (its bound), or a person (as wide as a body, as tall as
    /// they are; or at their head, `at_head`).
    fn beside(refr: usize, kind: u32, side: u32, at_head: bool) -> HudGroup {
        let id = refr as u32 ^ kind ^ side;
        let player = HudGroup { anchor: ANCHOR_PLAYER, id, ..Default::default() };
        let Some((centre, radius)) = bound(refr) else { return player };
        if is_actor(refr) {
            let at = if at_head { head(refr).unwrap_or(centre + Vec3::Y * radius * 0.8) } else { centre };
            // (A head reaches a hand's width round itself; a body a shoulder's, and up and down
            // its height.)
            let (across, tall) = if at_head { (0.15, 0.15) } else { (0.3, radius.min(1.2)) };
            return HudGroup { anchor: ANCHOR_WORLD, side, at: at.to_array(), radius: across, id, height: tall };
        }
        HudGroup { anchor: ANCHOR_WORLD, side, at: centre.to_array(), radius: radius.min(1.0), id, height: 0.0 }
    }

    /// The HUD and the menus this frame.
    pub fn layout() -> HudLayout {
        let mut l = HudLayout::new(HUDL_ALPHA | HUDL_REST_HIDDEN, &[]);
        l.pointer = POINTER;
        // (The thing on a hand the game's card is of now: the runtime keeps a picture of it for that
        // hand, `crate::hud_info`.)
        l.card_of = crate::hud_info::told();
        let Some(screen) = tiles::screen() else { return l };
        // A line of the game's ordinary text (a list row's name: 38 units with its leading).
        l.text_height = 36.0 / screen.1;
        let mut trees = Trees::default();
        // (What the crosshair was last on out of menus: what a menu opened by activating it is about.)
        static LOOKED_AT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        if !crate::menus::menu_mode() {
            LOOKED_AT.store(crate::hud_info::looked_at(), std::sync::atomic::Ordering::Relaxed);
            // A book read as the game plays on (`crate::books`): its pages held over it as ever, and
            // the HUD's pieces it covers in the game's picture of the HUD (it's drawn over them)
            // left out until it's shut.
            let covered = crate::books::open().and_then(|_| piece_box(trees.get(MENU_BOOK), &Piece::of(&[From::All], false)));
            hud(&mut l, screen, &mut trees, covered);
            if covered.is_some() {
                book(&mut l, screen, &mut trees);
                solid(trees.get(MENU_BOOK));
            }
            return l;
        }
        // Testing (`RUSTVR_OBLIVION_DUMP_TILES=1`): each menu's tiles into the log, once, for cutting
        // a menu into pieces.
        static DUMP: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *DUMP.get_or_init(|| std::env::var("RUSTVR_OBLIVION_DUMP_TILES").is_ok_and(|v| v == "1")) {
            for kind in MENU_MESSAGE..=MENU_LAST {
                if tiles::menu_tile(kind) != 0 {
                    tiles::dump(kind);
                }
            }
        }
        let player = |kind: u32| HudGroup { anchor: ANCHOR_PLAYER, id: kind, ..Default::default() };
        let top = tiles::top_menu();
        let container = |l: &mut HudLayout, trees: &mut Trees| {
            let refr = mem::deref(tiles::menu(MENU_CONTAINER) + CONTAINER_REF);
            add_group(l, beside(refr, MENU_CONTAINER, SIDE_RIGHT, false), MENU_CONTAINER, CONTAINER, screen, trees).map(|b| (MENU_CONTAINER, b))
        };
        let big_four = |l: &mut HudLayout, trees: &mut Trees| {
            // (All four are there, hidden, from when a game loads: the one that shows.)
            let kind = [MENU_INVENTORY, MENU_STATS, MENU_MAGIC, MENU_MAP].into_iter().find(|&k| tiles::shows(k))?;
            // The map drawn from the belt: in the hand, its own page and tabs (a grip on a map page
            // drags it; the journal's aren't dragged). Going back to the belt: nothing.
            if kind == MENU_MAP && crate::map::put_away() {
                return None;
            }
            if kind == MENU_MAP && crate::map::held_open() {
                let drag = if crate::map::held_map_page() { HELD_DRAG } else { 0 };
                let held = HudHeld { object: crate::map::MAP_ID, flags: HELD_SHEET | drag, ..Default::default() };
                return add_held_group(l, HudGroup { id: BIG_FOUR ^ crate::map::MAP_ID, ..Default::default() }, held, kind, MAP_HELD, screen, trees).map(|b| (kind, b));
            }
            let pieces = match kind {
                MENU_INVENTORY => INVENTORY,
                MENU_MAGIC => MAGIC,
                MENU_STATS => STATS,
                _ => MAP,
            };
            // (The big four are one place: switching between them keeps it.)
            add_group(l, player(BIG_FOUR), kind, pieces, screen, trees).map(|b| (kind, b))
        };
        if !matches!(top, MENU_QUANTITY | MENU_MESSAGE | MENU_NEGOTIATE) {
            put_back();
        }
        match top {
            MENU_CONTAINER => {
                container(&mut l, &mut trees);
            }
            MENU_DIALOG => {
                let speaker = mem::deref(tiles::menu(MENU_DIALOG) + DIALOG_SPEAKER);
                add_group(&mut l, beside(speaker, MENU_DIALOG, SIDE_BELOW, true), MENU_DIALOG, DIALOG_LINE, screen, &mut trees);
                add_group(&mut l, beside(speaker, MENU_DIALOG, SIDE_ABOVE, true), MENU_DIALOG, DIALOG_NAME, screen, &mut trees);
                add_group(&mut l, beside(speaker, MENU_DIALOG, SIDE_RIGHT, true), MENU_DIALOG, DIALOG_TOPICS, screen, &mut trees);
            }
            BIG_FOUR => {
                big_four(&mut l, &mut trees);
            }
            MENU_BOOK => book(&mut l, screen, &mut trees),
            MENU_QUANTITY | MENU_MESSAGE | MENU_NEGOTIATE => {
                // A count, a question or a haggle over another menu: where the eyes look, the menu
                // under it where it was.
                let under = if tiles::menu_tile(MENU_CONTAINER) != 0 { container(&mut l, &mut trees) } else { big_four(&mut l, &mut trees) };
                let popup = add_group(&mut l, HudGroup { anchor: ANCHOR_POPUP, id: top, ..Default::default() }, top, WHOLE, screen, &mut trees);
                if let (Some((kind, u)), Some(p)) = (under, popup) {
                    make_room((tiles::menu_tile(kind), u), (tiles::menu_tile(top), p), screen);
                }
            }
            // (The race menu stands beside the mirror the player is made before: the runtime puts
            // it there, the part of the picture it's in, and pieces of it here would float where
            // the mirror stands. What it asks over itself still floats, above.)
            0 | crate::menus::MENU_RACE_SEX => {}
            // The holsters' editor ahead of the player, as their own menus are.
            crate::slot_menu::MENU_HOLSTERS => {
                add_group(&mut l, player(top), top, HOLSTERS, screen, &mut trees);
            }
            // A lock picked at its keyhole: what it says beside its tumblers, and the hands go on
            // under it (they hold the pick).
            MENU_LOCKPICK if crate::lock::keyhole().is_some() => {
                let k = crate::lock::keyhole().unwrap();
                let group = HudGroup { anchor: ANCHOR_WORLD, side: SIDE_RIGHT, at: k.at.to_array(), radius: LOCK_REACH, id: MENU_LOCKPICK, height: 0.0 };
                add_group(&mut l, group, MENU_LOCKPICK, LOCK, screen, &mut trees);
                l.flags |= HUDL_HANDS;
            }
            kind => {
                // (The runtime's own settings float beside the game's pause menu.)
                if kind == MENU_PAUSE {
                    l.flags |= HUDL_SETTINGS;
                }
                // A menu opened from dialogue (a service: spells, training, repair, recharge,
                // persuasion) beside the one who offers it; a thing's own beside the thing; any
                // other ahead of the player.
                let speaker = mem::deref(tiles::menu(MENU_DIALOG) + DIALOG_SPEAKER);
                let thing = LOOKED_AT.load(std::sync::atomic::Ordering::Relaxed);
                let group = if speaker != 0 && tiles::menu_tile(MENU_DIALOG) != 0 {
                    beside(speaker, kind, SIDE_RIGHT, false)
                } else if THINGS_OWN.contains(&kind) && thing != 0 && !is_actor(thing) {
                    beside(thing, kind, SIDE_RIGHT, false)
                } else {
                    player(kind)
                };
                add_group(&mut l, group, kind, WHOLE, screen, &mut trees);
            }
        }
        // (A pop-up stays solid: what's behind it in the world is the menu it's over. So does what's
        // held in the hands: a book's pages, a map.)
        let held_map = crate::map::held_open();
        for (k, tree) in trees.0.iter() {
            if matches!(*k, MENU_QUANTITY | MENU_MESSAGE | MENU_NEGOTIATE | MENU_BOOK) || (*k == MENU_MAP && held_map) {
                solid(tree);
            } else {
                see_through(tree);
            }
        }
        l
    }

    /// The HUD's pieces (out of menus): see the module's table. None of the ones under `covered` (a
    /// book's area, which is drawn over them).
    fn hud(l: &mut HudLayout, screen: (f32, f32), trees: &mut Trees, covered: Option<[f32; 4]>) {
        let under = |b: [f32; 4]| covered.is_some_and(|c| b[0] < c[2] && b[2] > c[0] && b[1] < c[3] && b[3] > c[1]);
        let piece = |l: &mut HudLayout, trees: &mut Trees, role: u32, menu: u32, from: &'static [From]| -> Option<[f32; 4]> {
            let b = piece_box(trees.get(menu), &Piece::of(from, false)).filter(|b| !under(*b))?;
            l.push(HudElement::new(role, tiles::uv_of(b, screen, MARGIN)?));
            Some(b)
        };
        // Over the off-hand wrist, stacked upwards: the bars with the weapon and the spell, the
        // compass, and the effects working on the player (which come and go, so they're last).
        let status = l.count as usize;
        piece(l, trees, HUD_STATUS, MENU_HUD, &[From::Drawn("hudmain_statusbars"), From::Drawn("hudmain_Weapon_Icon"), From::Drawn("hudmain_Magic_Icon")]);
        let compass = piece(l, trees, HUD_STATUS, MENU_HUD, &[From::Drawn("hudmain_compass_layout")]).map(|_| l.count as usize - 1);
        piece(l, trees, HUD_STATUS, MENU_HUD, &[From::Drawn("magic_icons")]);
        // (The bars and icons a little bigger than a watch's face: they're read at a glance.)
        for e in &mut l.elements[status..l.count as usize] {
            e.scale = 1.3;
        }
        if let Some(i) = compass {
            // (Its heading letters are small: on a wrist raised to the eyes, as big to them as when
            // it floated in front.)
            l.elements[i].scale = 1.8;
        }
        piece(l, trees, HUD_SUBTITLES, MENU_SUBTITLES, &[From::Drawn("hudsubtitle_text")]);
        piece(l, trees, HUD_HINTS, MENU_SUBTITLES, &[From::Drawn("hudsubtitle_icon"), From::Drawn("hudsubtitle_notice")]);
        piece(l, trees, HUD_HINTS, MENU_HUD, &[From::Drawn("hudmain_region")]);
        // The breath meter, under water (`crate::swim`): where the game has it, at the top of the
        // view. (What it draws: its menu is as big as the screen.)
        let breath = trees.get(MENU_BREATH);
        if let Some(b) = breath.0.first().and_then(|top| breath.drawn(&top.name)).filter(|b| !under(*b))
            && let Some(uv) = tiles::uv_of(b, screen, MARGIN)
        {
            l.push(HudElement::new(HUD_CENTER, uv));
        }
        // At gaze: the reticle where it is from the screen's middle (its activate icon on the point
        // looked at; its crosshair is hidden, as the hands aim), and the card of what it's on under it.
        let (w, h) = screen;
        let offset = |b: [f32; 4]| [((b[0] + b[2]) * 0.5 - w * 0.5) / h, -((b[1] + b[3]) * 0.5 - h * 0.5) / h, 0.0];
        let crosshair = crate::hud_info::looked_at();
        // A thing that can be taken says nothing while it's only looked at: neither its card (the
        // game never gets it, `crate::hud_info`) nor its activate icon (a hand, red when taking it
        // would be stealing), nor that icon fading out after the gaze leaves it. Once a hand has
        // it, the game's card is the held thing's and floats over it, and the icon is under the
        // gaze again, as the game sets it for its target.
        let quiet = crate::hud_info::quiet();
        let icons: &'static [From] = if quiet { &[From::Drawn("hudreticle_sneak")] } else { &[From::Drawn("hudreticle_reticle"), From::Drawn("hudreticle_sneak")] };
        let reticle = piece_box(trees.get(RETICLE), &Piece::of(icons, false)).filter(|b| !under(*b));
        if let Some((b, uv)) = reticle.and_then(|b| Some((b, tiles::uv_of(b, screen, MARGIN)?))) {
            l.push(HudElement { role: HUD_GAZE, uv, offset: offset(b), ..Default::default() });
        }
        // (What they're about: they float just in front of it, a person's head or a thing's bound.)
        if crosshair != 0
            && let Some((centre, radius)) = bound(crosshair)
        {
            let (at, radius) = if is_actor(crosshair) { (head(crosshair).unwrap_or(centre), 0.12) } else { (centre, radius) };
            l.gaze = at.to_array();
            l.gaze_radius = radius.max(0.05);
        }
        if let Some(uv) = crate::hud_info::card_uv().filter(|uv| !under([uv[0] * w, uv[1] * h, uv[2] * w, uv[3] * h])) {
            if crate::hud_info::holding() {
                // (Over the thing in a hand.)
                l.push(HudElement::new(HUD_HELD, uv));
            } else if quiet {
                // (Its card comes once a hand has it: a card for every thing looked at is too much.)
            } else {
                let below = reticle.map_or(0.0, |r| (r[3] - r[1]) * 0.5 / h) + (uv[3] - uv[1]) * 0.5 + 0.01;
                let above_middle = reticle.map_or([0.0; 3], offset);
                l.push(HudElement { role: HUD_GAZE, uv, offset: [0.0, above_middle[1] - below, 0.0], ..Default::default() });
            }
        }
        // A foe's health over the foe (what the crosshair is on), or at gaze with nobody there.
        if let Some(b) = piece_box(trees.get(RETICLE), &Piece::of(&[From::Drawn("hudreticle_enemy_health")], false)).filter(|b| !under(*b))
            && let Some(uv) = tiles::uv_of(b, screen, MARGIN)
        {
            match (crosshair != 0 && is_actor(crosshair)).then(|| head(crosshair)).flatten() {
                Some(head) => {
                    l.target = (head + Vec3::Y * 0.35).to_array();
                    l.has_target = 1;
                    l.push(HudElement::new(HUD_TARGET, uv));
                }
                None => l.push(HudElement { role: HUD_GAZE, uv, offset: offset(b), ..Default::default() }),
            }
        }
    }

    /// A menu's pieces to measure (when, and the layout): see [`measure_due`].
    static MEASURE: Mutex<Option<(f64, HudLayout)>> = Mutex::new(None);

    /// Half a second after a menu's pieces come (it has faded in), the layout whose pieces to
    /// measure: how much of each the game's UI covers in the HUD copy, for the log
    /// ([`crate::d3d9::coverage`]: what the runtime is given to draw solid).
    pub fn measure_due() -> Option<HudLayout> {
        let mut m = MEASURE.lock().unwrap();
        if m.as_ref().is_some_and(|(at, _)| crate::link::now() >= *at) { m.take().map(|(_, l)| l) } else { None }
    }

    /// Out of the world (the main menu, loading): the game's screen whole, as before; only the
    /// laser's colours said.
    pub fn outside() -> HudLayout {
        let mut l = HudLayout::new(0, &[]);
        l.pointer = POINTER;
        l
    }

    /// `l` ([`layout`] or [`outside`]), when it isn't what was last sent (or every two seconds, for
    /// a runtime that has come back).
    pub fn changed(l: HudLayout) -> Option<HudLayout> {
        static LAST: Mutex<Option<(HudLayout, f64)>> = Mutex::new(None);
        let now = crate::link::now();
        let mut last = LAST.lock().unwrap();
        let same = last.as_ref().is_some_and(|(p, at)| bytemuck::bytes_of(p) == bytemuck::bytes_of(&l) && now - at < 2.0);
        if same {
            return None;
        }
        if last.as_ref().is_none_or(|(p, _)| p.count != l.count || p.group_count != l.group_count) {
            tracing::info!("hud: {} pieces in {} groups (the menu on top {:#x})", l.count, l.group_count, if crate::menus::menu_mode() { tiles::top_menu() } else { 0 });
            *MEASURE.lock().unwrap() = (l.group_count > 0).then_some((now + 0.5, l));
        }
        *last = Some((l, now));
        Some(l)
    }
}
