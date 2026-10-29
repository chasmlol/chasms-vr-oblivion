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
//! | dialogue | the speaker: their line under their face, the topics beside them | the name and line; the topics and services |
//! | the inventory, magic, the map, the character | ahead of the player | the page; the HUD's switcher between them (bars: character, weapon: inventory, spell: magic, compass: map) |
//! | a count, a question | where the eyes look | the box |
//! | anything else | ahead of the player | the menu, cut from the screen |
//!
//! A piece is named by its tiles in the game's own menu file (`menus\container_menu.xml`...): its
//! rectangle is the box of what the game shows under them now, so it follows the game (a button the
//! game hides leaves its piece). The pieces keep the game's own arrangement, drawn apart a little so
//! each floats on its own; the first is the one the others are arranged round. The laser points at
//! the ones that take clicks.
//!
//! **The HUD** (`menus\main\hud_main_menu.xml`) keeps its pieces anchored to the screen's corners
//! and centre at any aspect: the health, magicka and fatigue bars and the equipped weapon and spell
//! icons in the lower left, the compass at the bottom middle, the crosshair (and what it can
//! activate: "Open Door", a name) at the centre, the sneak eye above it; the card of what it's on
//! (its name and numbers) at the bottom right ([`crate::hud_info`]).

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

/// A piece of a menu: the tiles it's made of (in its group's menu, or in another: the HUD's), and
/// whether the laser points at it.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub menu: u32,
    pub from: &'static [From],
    pub point: bool,
}

impl Piece {
    /// A piece of the group's own menu.
    pub const fn of(from: &'static [From], point: bool) -> Piece {
        Piece { menu: 0, from, point }
    }
}

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

    // The menus by type (`kMenuType_*`).
    const MENU_MESSAGE: u32 = 0x3E9;
    const MENU_INVENTORY: u32 = 0x3EA;
    const MENU_STATS: u32 = 0x3EB;
    const MENU_HUD: u32 = 0x3EC;
    const MENU_CONTAINER: u32 = 0x3F0;
    const MENU_DIALOG: u32 = 0x3F1;
    const MENU_QUANTITY: u32 = 0x3F8;
    const MENU_MAGIC: u32 = 0x3FE;
    const MENU_MAP: u32 = 0x3FF;
    const MENU_LAST: u32 = 0x41B;
    use tiles::BIG_FOUR;
    const MENU_PAUSE: u32 = 0x3F5;

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
    const SWITCHER: Piece = Piece { menu: MENU_HUD, from: &[From::Drawn("hudmain_background")], point: true };

    /// The inventory (`main\inventory_menu.xml`): its list as the container's (heads, scroll bar,
    /// the weight, armour and gold under it), its tabs, and the switcher.
    const INVENTORY: &[Piece] = &[
        Piece::of(&[From::Box("inv_window"), From::Drawn("inv_headers_top"), From::Drawn("inv_scroll_bar"), From::Drawn("inv_extra_data")], true),
        Piece::of(&[From::Drawn("inv_tabs")], true),
        SWITCHER,
    ];
    /// Magic (`main\magic_menu.xml`): its list of spells (heads, scroll bar, the cost and
    /// magnitude marks), its tabs, and the switcher.
    const MAGIC: &[Piece] = &[
        Piece::of(&[From::Box("magic_window"), From::Drawn("magic_headers_top"), From::Drawn("magic_scroll_bar"), From::Drawn("magic_misc_elements")], true),
        Piece::of(&[From::Drawn("magic_tabs")], true),
        SWITCHER,
    ];
    /// The character (`main\stats_menu.xml`): its page, its tabs, and the switcher.
    const STATS: &[Piece] = &[Piece::of(&[From::Drawn("stat_background")], true), Piece::of(&[From::Drawn("stat_tabs")], true), SWITCHER];
    /// The map and the journal (`main\map_menu.xml`): its page, its tabs, and the switcher.
    const MAP: &[Piece] = &[Piece::of(&[From::Drawn("map_background")], true), Piece::of(&[From::Drawn("map_tabs")], true), SWITCHER];

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

    /// The menus' trees, each read once a frame as it's needed.
    #[derive(Default)]
    struct Trees(HashMap<u32, tiles::Tree>);

    impl Trees {
        fn get(&mut self, kind: u32) -> &tiles::Tree {
            self.0.entry(kind).or_insert_with(|| tiles::Tree::read(tiles::menu_tile(kind)))
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

    /// Menu `kind`'s pieces as a group of `layout`.
    fn add_group(layout: &mut HudLayout, group: HudGroup, kind: u32, pieces: &[Piece], screen: (f32, f32), trees: &mut Trees) {
        let found: Vec<(&Piece, [f32; 4])> = pieces.iter().filter_map(|p| piece_box(trees.get(if p.menu != 0 { p.menu } else { kind }), p).map(|b| (p, b))).collect();
        if found.is_empty() {
            return;
        }
        let mut boxes: Vec<[f32; 4]> = found.iter().map(|(_, b)| *b).collect();
        carve(&mut boxes);
        let offsets = arrange(&boxes, screen);
        let n = layout.add_group(group);
        for ((p, _), (b, offset)) in found.iter().zip(boxes.iter().zip(offsets)) {
            if let Some(uv) = tiles::uv_of(*b, screen, MARGIN) {
                let flags = HUDE_SOFT | if p.point { HUDE_POINT } else { 0 };
                layout.push(HudElement::piece(n, uv, offset, flags));
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
        let Some(screen) = tiles::screen() else { return l };
        // A line of the game's ordinary text (a list row's name: 38 units with its leading).
        l.text_height = 36.0 / screen.1;
        if !crate::menus::menu_mode() {
            hud(&mut l);
            return l;
        }
        let mut trees = Trees::default();
        // (Tests: each menu's tiles into the log, once.)
        for kind in MENU_MESSAGE..=MENU_LAST {
            if tiles::menu_tile(kind) != 0 {
                tiles::dump(kind);
            }
        }
        let player = |kind: u32| HudGroup { anchor: ANCHOR_PLAYER, id: kind, ..Default::default() };
        let top = tiles::top_menu();
        let container = |l: &mut HudLayout, trees: &mut Trees| {
            let refr = mem::deref(tiles::menu(MENU_CONTAINER) + CONTAINER_REF);
            add_group(l, beside(refr, MENU_CONTAINER, SIDE_RIGHT, false), MENU_CONTAINER, CONTAINER, screen, trees);
        };
        let big_four = |l: &mut HudLayout, trees: &mut Trees| {
            // (All four are there, hidden, from when a game loads: the one that shows.)
            if let Some(kind) = [MENU_INVENTORY, MENU_STATS, MENU_MAGIC, MENU_MAP].into_iter().find(|&k| tiles::shows(k)) {
                let pieces = match kind {
                    MENU_INVENTORY => INVENTORY,
                    MENU_MAGIC => MAGIC,
                    MENU_STATS => STATS,
                    _ => MAP,
                };
                // (The big four are one place: switching between them keeps it.)
                add_group(l, player(BIG_FOUR), kind, pieces, screen, trees);
            }
        };
        match top {
            MENU_CONTAINER => container(&mut l, &mut trees),
            MENU_DIALOG => {
                let speaker = mem::deref(tiles::menu(MENU_DIALOG) + DIALOG_SPEAKER);
                add_group(&mut l, beside(speaker, MENU_DIALOG, SIDE_BELOW, true), MENU_DIALOG, DIALOG_LINE, screen, &mut trees);
                add_group(&mut l, beside(speaker, MENU_DIALOG, SIDE_ABOVE, true), MENU_DIALOG, DIALOG_NAME, screen, &mut trees);
                add_group(&mut l, beside(speaker, MENU_DIALOG, SIDE_RIGHT, true), MENU_DIALOG, DIALOG_TOPICS, screen, &mut trees);
            }
            BIG_FOUR => big_four(&mut l, &mut trees),
            MENU_QUANTITY | MENU_MESSAGE => {
                // A count or a question over another menu: where the eyes look, the menu under it
                // where it was.
                if tiles::menu_tile(MENU_CONTAINER) != 0 {
                    container(&mut l, &mut trees);
                } else {
                    big_four(&mut l, &mut trees);
                }
                add_group(&mut l, HudGroup { anchor: ANCHOR_POPUP, id: top, ..Default::default() }, top, WHOLE, screen, &mut trees);
            }
            0 => {}
            kind => {
                // (The runtime's own settings float beside the game's pause menu.)
                if kind == MENU_PAUSE {
                    l.flags |= HUDL_SETTINGS;
                }
                add_group(&mut l, player(kind), kind, WHOLE, screen, &mut trees)
            }
        }
        l
    }

    /// The HUD's pieces (out of menus).
    fn hud(l: &mut HudLayout) {
        // The bars, the equipped weapon and spell, the compass: on the off-hand wrist.
        l.push(HudElement::new(HUD_STATUS, [0.0, 0.86, 0.5, 1.0]));
        // The crosshair and what it can activate: at gaze.
        l.push(HudElement::new(HUD_CENTER, [0.3, 0.38, 0.7, 0.62]));
        // Spoken lines.
        l.push(HudElement::new(HUD_SUBTITLES, [0.1, 0.62, 0.9, 0.86]));
        // Messages ("You have learned..."), top left.
        l.push(HudElement::new(HUD_HINTS, [0.0, 0.0, 0.5, 0.16]));
        // The card of what the crosshair's on (or what a hand has): over the thing in a hand.
        if let Some(uv) = crate::hud_info::card_uv() {
            l.push(HudElement::new(HUD_HELD, uv));
        }
    }

    /// The layout, when it isn't what was last sent (or every two seconds, for a runtime that has
    /// come back).
    pub fn changed() -> Option<HudLayout> {
        static LAST: Mutex<Option<(HudLayout, f64)>> = Mutex::new(None);
        let now = crate::link::now();
        let l = layout();
        let mut last = LAST.lock().unwrap();
        let same = last.as_ref().is_some_and(|(p, at)| bytemuck::bytes_of(p) == bytemuck::bytes_of(&l) && now - at < 2.0);
        if same {
            return None;
        }
        if last.as_ref().is_none_or(|(p, _)| p.count != l.count || p.group_count != l.group_count) {
            tracing::info!("hud: {} pieces in {} groups (the menu on top {:#x})", l.count, l.group_count, if crate::menus::menu_mode() { tiles::top_menu() } else { 0 });
        }
        *last = Some((l, now));
        Some(l)
    }
}
