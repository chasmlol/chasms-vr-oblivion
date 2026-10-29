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
//! | a count, a question, haggling | where the eyes look, the menu under it where it was | the box |
//! | a service from dialogue (spells, training, repair, recharge, persuasion) | beside the speaker | the menu, cut from the screen |
//! | anything else | ahead of the player | the menu, cut from the screen |
//!
//! **Transparency**: the pieces are as opaque as the game draws them, and the rest of the screen is
//! clear. The menus' backgrounds (their parchment: each menu's image tiles named `..._background`)
//! are drawn a little see-through (`menu_background` in `rustvr-oblivion.toml`, 0.8), so the world
//! shows through them while the text and icons on them stay solid: the game's own `alpha` of each,
//! set through the game's own update of a tile's value (so what it draws follows) whenever the
//! game sets it afresh.
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
//! | the health, magicka and fatigue bars, the weapon and the spell | `hud_main_menu.xml` | the back of the off-hand wrist (Alyx's glove) |
//! | the spells and potions working on the player | `hud_main_menu.xml` | up the forearm from the bars |
//! | the compass | `hud_main_menu.xml` | low in front of the body |
//! | the crosshair, what it can activate, the sneak eye | `hud_reticle.xml` | where the head looks |
//! | the card of what it's on (name and numbers) | `hud_info_menu.xml` ([`crate::hud_info`]) | under that; over the thing while a hand has one |
//! | a foe's health | `hud_reticle.xml` | over the foe |
//! | spoken lines | `hud_subtitle_menu.xml` | the caption strip |
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

    // The menus by type (`kMenuType_*`).
    const MENU_MESSAGE: u32 = 0x3E9;
    const MENU_INVENTORY: u32 = 0x3EA;
    const MENU_STATS: u32 = 0x3EB;
    const MENU_HUD: u32 = 0x3EC;
    const MENU_CONTAINER: u32 = 0x3F0;
    const MENU_SUBTITLES: u32 = 0x3F2;
    /// The reticle's tile (`hud_reticle.xml`), not a menu of its own: a pseudo-type for [`Trees`].
    const RETICLE: u32 = 0;
    /// `Tile::UpdateFloat(this, id, value)` (thiscall): sets a value and updates what's drawn from it.
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    /// `HUDInfoMenu`: what the crosshair is on (a reference).
    const HUD_INFO_MENU: usize = 0x00B3_B33C;
    const CROSSHAIR_REF: usize = 0x54;
    const MENU_DIALOG: u32 = 0x3F1;
    const MENU_QUANTITY: u32 = 0x3F8;
    /// Haggling over a price (from barter).
    const MENU_NEGOTIATE: u32 = 0x401;
    const MENU_MAGIC: u32 = 0x3FE;
    /// The description of a spell pointed at in the magic menu, a menu of its own.
    const MENU_MAGIC_POPUP: u32 = 0x400;
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
                let flags = HUDE_SOFT | if p.point { HUDE_POINT } else { 0 };
                layout.push(HudElement::piece(n, uv, offset, flags));
            }
        }
        boxes.into_iter().fold(None, tiles::grow)
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
        static OURS: Mutex<Vec<(usize, f32)>> = Mutex::new(Vec::new());
        let k = crate::settings::get().menu_background.clamp(0.0, 1.0);
        if k >= 1.0 {
            return;
        }
        let mut ours = OURS.lock().unwrap();
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
        let mut trees = Trees::default();
        if !crate::menus::menu_mode() {
            hud(&mut l, screen, &mut trees);
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
            MENU_QUANTITY | MENU_MESSAGE | MENU_NEGOTIATE => {
                // A count, a question or a haggle over another menu: where the eyes look, the menu
                // under it where it was.
                let under = if tiles::menu_tile(MENU_CONTAINER) != 0 { container(&mut l, &mut trees) } else { big_four(&mut l, &mut trees) };
                let popup = add_group(&mut l, HudGroup { anchor: ANCHOR_POPUP, id: top, ..Default::default() }, top, WHOLE, screen, &mut trees);
                if let (Some((kind, u)), Some(p)) = (under, popup) {
                    make_room((tiles::menu_tile(kind), u), (tiles::menu_tile(top), p), screen);
                }
            }
            0 => {}
            kind => {
                // (The runtime's own settings float beside the game's pause menu.)
                if kind == MENU_PAUSE {
                    l.flags |= HUDL_SETTINGS;
                }
                // A menu opened from dialogue (a service: spells, training, repair, recharge,
                // persuasion) beside the one who offers it; any other ahead of the player.
                let speaker = mem::deref(tiles::menu(MENU_DIALOG) + DIALOG_SPEAKER);
                let group = if speaker != 0 && tiles::menu_tile(MENU_DIALOG) != 0 { beside(speaker, kind, SIDE_RIGHT, false) } else { player(kind) };
                add_group(&mut l, group, kind, WHOLE, screen, &mut trees);
            }
        }
        // (A pop-up stays solid: what's behind it in the world is the menu it's over.)
        for (_, tree) in trees.0.iter().filter(|(k, _)| !matches!(**k, MENU_QUANTITY | MENU_MESSAGE | MENU_NEGOTIATE)) {
            see_through(tree);
        }
        l
    }

    /// The HUD's pieces (out of menus): see the module's table.
    fn hud(l: &mut HudLayout, screen: (f32, f32), trees: &mut Trees) {
        let piece = |l: &mut HudLayout, trees: &mut Trees, role: u32, menu: u32, from: &'static [From]| -> Option<[f32; 4]> {
            let b = piece_box(trees.get(menu), &Piece::of(from, false))?;
            l.push(HudElement::new(role, tiles::uv_of(b, screen, MARGIN)?));
            Some(b)
        };
        piece(l, trees, HUD_STATUS, MENU_HUD, &[From::Drawn("hudmain_statusbars"), From::Drawn("hudmain_Weapon_Icon"), From::Drawn("hudmain_Magic_Icon")]);
        piece(l, trees, HUD_STATUS, MENU_HUD, &[From::Drawn("magic_icons")]);
        if let Some(i) = piece(l, trees, HUD_COMPASS, MENU_HUD, &[From::Drawn("hudmain_compass_layout")]).map(|_| l.count as usize - 1) {
            // (A little bigger than the game's text: its heading letters are small.)
            l.elements[i].scale = 1.4;
        }
        piece(l, trees, HUD_SUBTITLES, MENU_SUBTITLES, &[From::Drawn("hudsubtitle_text")]);
        piece(l, trees, HUD_HINTS, MENU_SUBTITLES, &[From::Drawn("hudsubtitle_icon"), From::Drawn("hudsubtitle_notice")]);
        piece(l, trees, HUD_HINTS, MENU_HUD, &[From::Drawn("hudmain_region")]);
        // At gaze: the reticle where it is from the screen's middle (the crosshair on the point
        // looked at), and the card of what it's on under it.
        let (w, h) = screen;
        let offset = |b: [f32; 4]| [((b[0] + b[2]) * 0.5 - w * 0.5) / h, -((b[1] + b[3]) * 0.5 - h * 0.5) / h, 0.0];
        let reticle = piece_box(trees.get(RETICLE), &Piece::of(&[From::Drawn("hudreticle_reticle"), From::Drawn("hudreticle_sneak")], false));
        if let Some((b, uv)) = reticle.and_then(|b| Some((b, tiles::uv_of(b, screen, MARGIN)?))) {
            l.push(HudElement { role: HUD_GAZE, uv, offset: offset(b), ..Default::default() });
        }
        let crosshair = mem::deref(mem::deref(HUD_INFO_MENU) + CROSSHAIR_REF);
        if let Some(uv) = crate::hud_info::card_uv() {
            if crate::hud_info::holding() {
                // (Over the thing in a hand.)
                l.push(HudElement::new(HUD_HELD, uv));
            } else {
                let below = reticle.map_or(0.0, |r| (r[3] - r[1]) * 0.5 / h) + (uv[3] - uv[1]) * 0.5 + 0.01;
                let above_middle = reticle.map_or([0.0; 3], offset);
                l.push(HudElement { role: HUD_GAZE, uv, offset: [0.0, above_middle[1] - below, 0.0], ..Default::default() });
            }
        }
        // A foe's health over the foe (what the crosshair is on), or at gaze with nobody there.
        if let Some(b) = piece_box(trees.get(RETICLE), &Piece::of(&[From::Drawn("hudreticle_enemy_health")], false))
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
