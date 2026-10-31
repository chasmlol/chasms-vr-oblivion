//! Oblivion's menus as the game lays them out: trees of tiles (xOBSE's `GameTiles.h`), each menu's
//! read from its XML (`menus\*.xml`). The adapter reads where the game has put each tile, by name,
//! to cut its menus and its HUD into pieces ([`crate::hud`]); it never draws anything of its own.
//!
//! A tile (0x40 bytes) has its name at +8 (`BSStringT`: the characters' pointer and count), its
//! values at +0x14 (a list: +4 its first node; a node is next, previous, the data) and its children
//! at +0x30 (the same list). A value has its number at +4 and its id at +0x18 (u16): `visible`
//! 0xFA1 (1 no, 2 yes), `locus` 0xFA6, `alpha` 0xFA7, `y` 0xFAC, `x` 0xFAD, `height` 0xFCA, `width`
//! 0xFCB. A tile's `x` and `y` are from its nearest ancestor with `locus` (the menu at the top), in
//! the screen's units, the y axis down. What kind of tile it is is its vtable's fourth function:
//! 0x385 a rect (a box for others), 0x386 an image, 0x387 text, 0x388 a model.
//!
//! A text's `x` is where its `justify` (0xFD1: 1 left, 2 centre, 4 right) puts it: its left edge,
//! its middle or its right edge. A tile with `clipwindow` (0xFA4) shows only what's inside it of
//! what's under it (a list's rows, the compass's strip).
//!
//! A menu is read once a frame into a [`Tree`] (a container's hundreds of rows aren't: a piece is
//! never one row, and reading them all would cost the game its frame).

/// A box `[x0, y0, x1, y1]` grown to take in another.
pub fn grow(b: Option<[f32; 4]>, r: [f32; 4]) -> Option<[f32; 4]> {
    Some(match b {
        None => r,
        Some(b) => [b[0].min(r[0]), b[1].min(r[1]), b[2].max(r[2]), b[3].max(r[3])],
    })
}

/// A box in the screen's units as the part of the screen it is (0..1), a little margin round it.
pub fn uv_of(b: [f32; 4], screen: (f32, f32), margin: f32) -> Option<[f32; 4]> {
    let (w, h) = screen;
    if !(w > 0.0 && h > 0.0) || b[2] <= b[0] || b[3] <= b[1] {
        return None;
    }
    let uv = [(b[0] - margin) / w, (b[1] - margin) / h, (b[2] + margin) / w, (b[3] + margin) / h];
    Some(uv.map(|v| v.clamp(0.0, 1.0)))
}

pub const VISIBLE: u16 = 0xFA1;
pub const LOCUS: u16 = 0xFA6;
pub const ALPHA: u16 = 0xFA7;
pub const Y: u16 = 0xFAC;
pub const X: u16 = 0xFAD;
pub const USER0: u16 = 0xFAE;
/// A text tile's text (the `string` trait).
pub const STRING: u16 = 0xFDE;
pub const CLIP_WINDOW: u16 = 0xFA4;
pub const JUSTIFY: u16 = 0xFD1;
const CENTRE: f32 = 2.0;
const RIGHT: f32 = 4.0;
pub const HEIGHT: u16 = 0xFCA;
pub const WIDTH: u16 = 0xFCB;
/// Tile values' true (1 is false).
pub const TRUE: f32 = 2.0;

/// The menu on top while the big four (inventory, magic, map, character) are.
pub const BIG_FOUR: u32 = 1;

pub const KIND_RECT: u32 = 0x385;
pub const KIND_IMAGE: u32 = 0x386;
pub const KIND_TEXT: u32 = 0x387;
pub const KIND_MODEL: u32 = 0x388;

/// A tile with more children than this is a list (a container's hundreds of rows): it's read, its
/// rows aren't.
const LIST: usize = 24;

/// One tile as the game has it now: its box on the screen and whether it shows.
#[derive(Clone, Debug)]
pub struct Placed {
    pub tile: usize,
    pub name: String,
    /// How deep it is in its menu's tree (the menu 0).
    pub depth: u32,
    /// `[x0, y0, x1, y1]`, screen units (the y axis down).
    pub rect: [f32; 4],
    /// It and every tile above it are visible.
    pub shown: bool,
    /// Its alpha, if it has one (0 is clear).
    pub alpha: Option<f32>,
    /// What kind of tile it is (`KIND_*`).
    pub kind: u32,
}

impl Placed {
    fn has_size(&self) -> bool {
        self.rect[2] > self.rect[0] && self.rect[3] > self.rect[1]
    }
}

/// A menu's tiles as the game has placed them now, read once (depth first, the menu at its top from
/// (0, 0)).
#[derive(Default)]
pub struct Tree(pub Vec<Placed>);

impl Tree {
    /// The tile named `name` (the first), as placed now.
    pub fn find(&self, name: &str) -> Option<&Placed> {
        self.0.iter().find(|p| p.name.eq_ignore_ascii_case(name))
    }

    /// The box the tile named `name` has, if it shows.
    pub fn rect(&self, name: &str) -> Option<[f32; 4]> {
        self.find(name).filter(|p| p.shown && p.has_size()).map(|p| p.rect)
    }

    /// The box of what the game draws under the tile named `name` (itself included: its images,
    /// text and models that show and aren't clear), or None if nothing of it shows.
    pub fn drawn(&self, name: &str) -> Option<[f32; 4]> {
        let i = self.0.iter().position(|p| p.name.eq_ignore_ascii_case(name))?;
        let depth = self.0[i].depth;
        let under = std::iter::once(&self.0[i]).chain(self.0[i + 1..].iter().take_while(|p| p.depth > depth));
        under
            .filter(|p| p.shown && p.has_size() && p.alpha.is_none_or(|a| a > 0.0) && matches!(p.kind, KIND_IMAGE | KIND_TEXT | KIND_MODEL))
            .fold(None, |b, p| grow(b, p.rect))
    }

    /// The box of everything that shows (any tile with a size).
    pub fn shown(&self) -> Option<[f32; 4]> {
        self.0.iter().filter(|p| p.shown && p.has_size()).fold(None, |b, p| grow(b, p.rect))
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::mem;

    const NAME: usize = 0x8;
    const VALUES: usize = 0x14;
    const CHILDREN: usize = 0x30;
    const LIST_FIRST: usize = 0x4;
    /// `g_TileMenuArray`: the open menus' `TileMenu*` by type (its data at +4, from 0x3E9).
    const TILE_MENUS: usize = 0x00B1_3970;
    const MENU_FIRST: u32 = 0x3E9;
    /// A `TileMenu`'s `Menu*`.
    const TILE_MENU_MENU: usize = 0x44;
    /// `InterfaceManager::GetSingleton` (cdecl `(0, 1)`), its menu root tile (the screen's size).
    const INTERFACE: usize = 0x0058_2160;
    const MENU_ROOT: usize = 0x68;
    /// The menu on top (cdecl): 0 with none.
    const TOP_MENU: usize = 0x0057_8FE0;

    fn list(at: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = mem::deref(at + LIST_FIRST);
        while node != 0 && out.len() < 1024 {
            // (A node is next, previous, the data: read at once.)
            let Some([next, _, data]) = mem::at::<[u32; 3]>(node) else { break };
            if data != 0 {
                out.push(data as usize);
            }
            node = next as usize;
        }
        out
    }

    pub fn children(tile: usize) -> Vec<usize> {
        list(tile + CHILDREN)
    }

    /// A tile's values that place it, read at once.
    #[derive(Clone, Copy, Debug, Default)]
    struct Values {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        visible: Option<f32>,
        locus: bool,
        alpha: Option<f32>,
        justify: f32,
        clips: bool,
    }

    fn values(tile: usize) -> Values {
        let mut v = Values::default();
        for value in list(tile + VALUES) {
            // (Its number at +4, its id at +0x18.)
            let Some(b) = mem::at::<[u8; 0x1A]>(value) else { continue };
            let num = f32::from_le_bytes([b[4], b[5], b[6], b[7]]);
            if !num.is_finite() {
                continue;
            }
            match u16::from_le_bytes([b[0x18], b[0x19]]) {
                X => v.x = num,
                Y => v.y = num,
                WIDTH => v.w = num,
                HEIGHT => v.h = num,
                VISIBLE => v.visible = Some(num),
                LOCUS => v.locus = num == TRUE,
                ALPHA => v.alpha = Some(num),
                JUSTIFY => v.justify = num,
                CLIP_WINDOW => v.clips = num == TRUE,
                _ => {}
            }
        }
        v
    }

    /// The value with this id (`USER0`...), as the game has it.
    pub fn value(tile: usize, id: u16) -> Option<f32> {
        list(tile + VALUES).into_iter().find_map(|v| {
            let b = mem::at::<[u8; 0x1A]>(v)?;
            let num = f32::from_le_bytes([b[4], b[5], b[6], b[7]]);
            (u16::from_le_bytes([b[0x18], b[0x19]]) == id && num.is_finite()).then_some(num)
        })
    }

    /// The text a tile shows (its `string` value: a `BSStringT` at the value's +8), if it has any.
    pub fn text(tile: usize) -> Option<String> {
        list(tile + VALUES)
            .into_iter()
            .find_map(|v| {
                if mem::at::<u16>(v + 0x18)? != STRING {
                    return None;
                }
                let (p, n) = (mem::deref(v + 8), mem::at::<u16>(v + 0xC).unwrap_or(0) as usize);
                let bytes = (p != 0 && n != 0).then(|| mem::bytes(p as *const u8, n.min(512))).flatten()?;
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                Some(String::from_utf8_lossy(&bytes[..end]).into_owned())
            })
            .filter(|t| !t.trim().is_empty())
    }

    pub fn name(tile: usize) -> String {
        // (`BSStringT`: the characters, their count.)
        let (p, n) = (mem::deref(tile + NAME), mem::at::<u16>(tile + NAME + 4).unwrap_or(0) as usize);
        let n = n.min(256);
        // (Copied through the guarded read: the game may free a menu's tiles under it.)
        let Some(bytes) = (p != 0 && n != 0).then(|| mem::bytes(p as *const u8, n)).flatten() else { return String::new() };
        String::from_utf8_lossy(bytes.split(|&b| b == 0).next().unwrap_or(&[])).into_owned()
    }

    /// What kind of tile it is (`KIND_*`).
    pub fn kind(tile: usize) -> u32 {
        let f = mem::deref(mem::deref(tile) + 3 * 4);
        if !(0x0040_1000..0x00A3_0000).contains(&f) {
            return 0;
        }
        type F = unsafe extern "thiscall" fn(usize) -> u32;
        unsafe { std::mem::transmute::<usize, F>(f)(tile) }
    }

    impl Tree {
        /// The tiles under `root` (a menu's top tile) as placed now.
        pub fn read(root: usize) -> Tree {
            fn go(tile: usize, origin: (f32, f32), clip: [f32; 4], shown: bool, depth: u32, out: &mut Vec<Placed>) {
                if depth > 24 || out.len() > 2048 {
                    return;
                }
                let v = values(tile);
                let (x, y) = (origin.0 + v.x, origin.1 + v.y);
                let shown = shown && v.visible.is_none_or(|v| v == TRUE);
                let k = kind(tile);
                // (A text's x is its left edge, its middle or its right edge, as it's justified.)
                let left = if k != KIND_TEXT {
                    x
                } else if v.justify == CENTRE {
                    x - v.w * 0.5
                } else if v.justify == RIGHT {
                    x - v.w
                } else {
                    x
                };
                let own = [left, y, left + v.w, y + v.h];
                let r = [own[0].max(clip[0]), own[1].max(clip[1]), own[2].min(clip[2]), own[3].min(clip[3])];
                let r = if r[2] > r[0] && r[3] > r[1] { r } else { [own[0], own[1], own[0], own[1]] };
                out.push(Placed { tile, name: name(tile), depth, rect: r, shown, alpha: v.alpha, kind: k });
                let inner = if v.locus { (x, y) } else { origin };
                let clip = if v.clips { [clip[0].max(own[0]), clip[1].max(own[1]), clip[2].min(own[2]), clip[3].min(own[3])] } else { clip };
                let kids = children(tile);
                if kids.len() <= LIST {
                    for c in kids {
                        go(c, inner, clip, shown, depth + 1, out);
                    }
                }
            }
            let mut out = Vec::new();
            if root != 0 {
                let all = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
                go(root, (0.0, 0.0), all, true, 0, &mut out);
            }
            Tree(out)
        }
    }

    /// The open menu of this type's top tile (`kMenuType_*`), or 0.
    pub fn menu_tile(kind: u32) -> usize {
        let data = mem::deref(TILE_MENUS + 4);
        if kind < MENU_FIRST || data == 0 {
            return 0;
        }
        mem::deref(data + 4 * (kind - MENU_FIRST) as usize)
    }

    /// The open menu of this type (`Menu*`), or 0.
    pub fn menu(kind: u32) -> usize {
        let t = menu_tile(kind);
        if t == 0 { 0 } else { mem::deref(t + TILE_MENU_MENU) }
    }

    /// The menu on top (`kMenuType_*`), or 0.
    pub fn top_menu() -> u32 {
        unsafe { std::mem::transmute::<usize, unsafe extern "C" fn() -> u32>(TOP_MENU)() }
    }

    /// The HUD's reticle (`hud_reticle.xml`: the crosshair, the sneak eye, a foe's health), a tile
    /// of the interface's own (`InterfaceManager` +0x80), not a menu.
    pub fn reticle_tile() -> usize {
        type Interface = unsafe extern "C" fn(u8, u8) -> usize;
        let ui = unsafe { std::mem::transmute::<usize, Interface>(INTERFACE)(0, 1) };
        if ui == 0 { 0 } else { mem::deref(ui + 0x80) }
    }

    /// The screen's size in the menus' units.
    pub fn screen() -> Option<(f32, f32)> {
        type Interface = unsafe extern "C" fn(u8, u8) -> usize;
        let ui = unsafe { std::mem::transmute::<usize, Interface>(INTERFACE)(0, 1) };
        let root = mem::deref(ui + MENU_ROOT);
        if ui == 0 || root == 0 {
            return None;
        }
        let v = values(root);
        (v.w > 0.0 && v.h > 0.0).then_some((v.w, v.h))
    }

    /// The open menu of this type shows (its top tile is visible).
    pub fn shows(kind: u32) -> bool {
        let t = menu_tile(kind);
        t != 0 && value(t, VISIBLE).is_none_or(|v| v == TRUE)
    }

    /// Testing: every tile of the menu of this type (but a list's rows), its kind, box and whether it
    /// shows, into the log (once a type, hidden and showing): what a menu is cut into pieces by.
    pub fn dump(kind_of_menu: u32) {
        static DONE: std::sync::Mutex<Vec<(u32, bool)>> = std::sync::Mutex::new(Vec::new());
        {
            let key = (kind_of_menu, shows(kind_of_menu));
            let mut done = DONE.lock().unwrap();
            if done.contains(&key) {
                return;
            }
            done.push(key);
        }
        tracing::info!("tiles: menu {kind_of_menu:#x} on a screen of {:?}:", screen());
        for p in Tree::read(menu_tile(kind_of_menu)).0 {
            let a = p.alpha.map_or(String::new(), |a| format!(" alpha {a}"));
            let r = p.rect;
            tracing::info!("tiles: {}{} {:#x} [{:.0}, {:.0}, {:.0}, {:.0}]{}{a}", "  ".repeat(p.depth as usize), p.name, p.kind, r[0], r[1], r[2], r[3], if p.shown { "" } else { " hidden" });
        }
    }
}
