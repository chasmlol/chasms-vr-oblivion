//! Oblivion's HUD and menus in VR, piece by piece (`rustvr_protocol::hud`). The game draws them as
//! ever, into the HUD texture with their alpha ([`crate::render`]); this says which part of it is
//! what and where it goes, read every frame from where the game has put its own tiles
//! ([`crate::tiles`]). Nothing here draws: a piece is a rectangle of what the game drew.
//!
//! **Menus** are cut into the parts that matter and float in the world as groups
//! (`HudGroup`), each beside what it's about: a container's list beside the container, the
//! merchant's wares beside the merchant. A piece is named by its tiles in the game's own menu file
//! (`menus\container_menu.xml`...): its rectangle is the box of what the game shows under them now,
//! so it follows the game (a button the game hides leaves its piece). The pieces keep the game's
//! own arrangement, drawn apart a little so each floats on its own; the first is the one the
//! others are arranged round. The laser points at the ones that take clicks.
//!
//! **The HUD** (`menus\main\hud_main_menu.xml`) keeps its pieces anchored to the screen's corners
//! and centre at any aspect: the health, magicka and fatigue bars and the equipped weapon and spell
//! icons in the lower left, the compass at the bottom middle, the crosshair (and what it can
//! activate: "Open Door", a name) at the centre, the sneak eye above it; the card of what it's on
//! (its name and numbers) at the bottom right ([`crate::hud_info`]).

use rustvr_protocol::*;

/// Where a piece of a menu comes from: the box a tile has (a list's window, whatever it shows), or
/// the box of what the game draws under a tile (its images and text that show).
#[derive(Clone, Copy, Debug)]
pub enum From {
    Box(&'static str),
    Drawn(&'static str),
}

/// A piece of a menu: the tiles it's made of, and whether the laser points at it.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub from: &'static [From],
    pub point: bool,
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

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::{camera, mem, tiles};
    use glam::Vec3;
    use std::sync::Mutex;

    const MENU_CONTAINER: u32 = 0x3F0;
    /// `ContainerMenu`: the container (or the merchant, bartering).
    const CONTAINER_REF: usize = 0x44;
    /// A reference's 3D, and a node's world bound (centre, radius; game units).
    const REFR_NODE: usize = 0x3C;
    const NODE_BOUND: usize = 0x20;

    /// A container's list beside the container (`container_menu.xml`), three strips of its box: the
    /// list on its parchment (the column heads, the scroll bar, and under it the gold and the weight,
    /// or the merchant's name and gold), the category tabs, and the buttons (take all or haggle, the
    /// switch between the container's things and the player's, exit).
    const CONTAINER: &[Piece] = &[
        Piece {
            from: &[
                From::Box("cont_window"),
                From::Drawn("cont_headers_top"),
                From::Drawn("cont_scroll_bar"),
                From::Drawn("cont_player_money_icon"),
                From::Drawn("cont_npc_money_icon"),
                From::Drawn("cont_encumberance_icon"),
            ],
            point: true,
        },
        Piece { from: &[From::Drawn("cont_tab_layout")], point: true },
        Piece { from: &[From::Drawn("cont_button_take_all"), From::Drawn("cont_button_haggle"), From::Drawn("cont_button_exit"), From::Drawn("cont_select_frame")], point: true },
    ];

    /// The box of a piece on the screen now (the game's units), if any of it shows.
    fn piece_box(tree: &tiles::Tree, p: &Piece) -> Option<[f32; 4]> {
        p.from
            .iter()
            .filter_map(|f| match f {
                From::Box(name) => tree.rect(name),
                From::Drawn(name) => tree.drawn(name),
            })
            .fold(None, tiles::grow)
    }

    /// A menu's pieces as a group of `layout`.
    fn add_group(layout: &mut HudLayout, group: HudGroup, tree: &tiles::Tree, pieces: &[Piece], screen: (f32, f32)) {
        let found: Vec<(&Piece, [f32; 4])> = pieces.iter().filter_map(|p| piece_box(tree, p).map(|b| (p, b))).collect();
        if found.is_empty() {
            return;
        }
        let boxes: Vec<[f32; 4]> = found.iter().map(|(_, b)| *b).collect();
        let offsets = arrange(&boxes, screen);
        let n = layout.add_group(group);
        for ((p, b), offset) in found.iter().zip(offsets) {
            if let Some(uv) = tiles::uv_of(*b, screen, MARGIN) {
                let flags = HUDE_SOFT | if p.point { HUDE_POINT } else { 0 };
                layout.push(HudElement::piece(n, uv, offset, flags));
            }
        }
    }

    /// A reference's 3D bound in the protocol's world (metres): its centre and radius.
    fn bound(refr: usize) -> Option<(Vec3, f32)> {
        let node = mem::deref(refr + REFR_NODE);
        let b: [f32; 4] = mem::at(node + NODE_BOUND)?;
        (node != 0 && b.iter().all(|v| v.is_finite()) && b[3] > 0.0).then(|| (camera::basis().point_from_game(Vec3::new(b[0], b[1], b[2])), b[3] / camera::UNITS_PER_METER))
    }

    /// The group beside a container (or a merchant).
    fn beside(refr: usize, kind: u32, side: u32) -> HudGroup {
        let (at, radius) = bound(refr).unwrap_or((Vec3::ZERO, 0.0));
        HudGroup { anchor: if radius > 0.0 { ANCHOR_WORLD } else { ANCHOR_PLAYER }, side, at: at.to_array(), radius: radius.min(1.0), id: refr as u32 ^ kind, height: 0.0 }
    }

    /// The HUD and the menus this frame.
    pub fn layout() -> HudLayout {
        let mut l = HudLayout::new(HUDL_ALPHA | HUDL_REST_HIDDEN, &[]);
        let Some(screen) = tiles::screen() else { return l };
        // A line of the game's ordinary text (a list row's name: 38 units with its leading).
        l.text_height = 36.0 / screen.1;
        if crate::menus::menu_mode() {
            let top = tiles::top_menu();
            tiles::dump(top);
            if top == MENU_CONTAINER {
                let menu = tiles::menu(MENU_CONTAINER);
                let refr = mem::deref(menu + CONTAINER_REF);
                let tree = tiles::Tree::read(tiles::menu_tile(MENU_CONTAINER));
                add_group(&mut l, beside(refr, MENU_CONTAINER, SIDE_RIGHT), &tree, CONTAINER, screen);
            }
            // A menu not described yet: whole, floating (the runtime's fallback).
            return l;
        }
        hud(&mut l);
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
            tracing::info!("hud: {} pieces in {} groups", l.count, l.group_count);
        }
        *last = Some((l, now));
        Some(l)
    }
}
