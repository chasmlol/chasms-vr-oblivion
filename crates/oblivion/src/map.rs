//! The map unfolded from the belt (`docs/design/books.md`): a folded map the player keeps in a
//! body slot (the right hip by default, `map_slot` in `rustvr-oblivion.toml`), an object of the
//! adapter's own the runtime keeps there ([`OBJF_KEPT`]: let go anywhere, it goes back). Drawn from
//! the belt, it opens the game's own map menu on the local map in an interior and the world map
//! outside, and the map's page lies in the hand as a sheet of paper ([`crate::hud`]:
//! `ANCHOR_HELD`, `HELD_SHEET`): the other hand spreads it (bigger), grips its face to drag the
//! map (the game's own pan), and presses it with a fingertip (a place's name, fast travel with the
//! game's own question and rules; its tabs, the journal's pages too, stay in the hand). Put back (or
//! let go), it folds away: the game's menu shut, and nothing of it shown while it shuts.
//!
//! The map menu (`menus\main\map_menu.xml`, `MapMenu`, menu type 0x3FF) is opened the way the
//! game's Menu Mode opens the last of the big four: the interface's menu root keeps which that was
//! (its value 6001, 0x1771: the map is 1023), so that's set to the map and Menu Mode pressed. Its
//! pages are `map_page_1` (the local map), `map_page_2` (the world map) and three of the journal,
//! the one that shows the one open; its tabs' clicks (`HandleClick(1..5)`) turn to them.

use crate::{mem, tiles};
use rustvr_protocol::*;

/// The map's object id.
pub const MAP_ID: u32 = 0x0B1E_0005;
pub const MENU_MAP: u32 = 0x3FF;
pub const PAGE_LOCAL: f32 = 1.0;
pub const PAGE_WORLD: f32 = 2.0;
/// `InterfaceManager::GetSingleton` (cdecl `(0, 1)`) and its menu root; the root's value for the
/// last of the big four the player had open.
const INTERFACE: usize = 0x0058_2160;
const MENU_ROOT: usize = 0x68;
const LAST_BIG_FOUR: u32 = 0x1771;
const UPDATE_FLOAT: usize = 0x0058_CEB0;
const VT_HANDLE_CLICK: usize = 3 * 4;
/// `TES` and its interior cell (0 outside).
const TES: usize = 0x00B3_33A0;
const TES_INTERIOR: usize = 0x34;

/// The folded map: small enough for a pouch, as light as paper. Nothing draws it: the game's
/// own map art is the map, when it's open.
pub fn desc(slot: u32) -> ObjectDesc {
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = MAP_ID;
    d.kind = OBJ_PROP;
    d.flags = OBJF_GRABBABLE | OBJF_HOLSTERABLE | OBJF_KEPT;
    d.spawn_seq = 1;
    d.spawn_pose = Pose::IDENTITY;
    d.shape = SHAPE_BOX;
    d.half_extents = [0.05, 0.015, 0.07];
    d.mass = 0.15;
    d.start_slot = 1 + slot;
    d.name[..3].copy_from_slice(b"map");
    d
}

/// The body slot the map's kept in (`map_slot`: "belt", "hip_right", "hip_left", "chest_left",
/// "chest_right"), or none: it isn't on the body.
pub fn slot_named(name: &str) -> Option<u32> {
    match name {
        "hip_right" => Some(SLOT_HIP_RIGHT),
        "hip_left" => Some(SLOT_HIP_LEFT),
        "chest_left" => Some(SLOT_CHEST_LEFT),
        "chest_right" => Some(SLOT_CHEST_RIGHT),
        "belt" => Some(SLOT_BELT),
        _ => None,
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn publish(_a: &rustvr_adapter::Adapter) {}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use std::sync::Mutex;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Fold {
        /// Folded on the belt (or in a hand with the map shut by B: it stays shut until it's back).
        Away,
        /// Drawn: Menu Mode pressed for the map, since then.
        Opening(f64),
        Open,
        /// Put back: Menu Mode pressed to shut it, since then.
        Closing(f64),
        Shut,
    }

    struct State {
        published: bool,
        fold: Fold,
        /// In a hand now (which).
        hand: Option<usize>,
        states: Box<ObjectStates>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State { published: false, fold: Fold::Away, hand: None, states: bytemuck::zeroed_box() }))
    }

    fn update_float(tile: usize, id: u32, v: f32) {
        type Update = unsafe extern "thiscall" fn(usize, u32, f32);
        crate::hooks::guard_void("map value", || unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(tile, id, v) });
    }

    /// The map menu's page now (`PAGE_*`, or the journal's 3 to 5): the one of its pages
    /// (`map_page_1` to `map_page_5`) that shows.
    pub fn page() -> Option<f32> {
        let t = tiles::menu_tile(MENU_MAP);
        if t == 0 {
            return None;
        }
        let tree = tiles::Tree::read(t);
        (1..=5).find(|k| tree.find(&format!("map_page_{k}")).is_some_and(|p| p.shown)).map(|k| k as f32)
    }

    /// The map menu shows, the big four on top.
    fn shows() -> bool {
        tiles::shows(MENU_MAP) && tiles::top_menu() == tiles::BIG_FOUR
    }

    /// The map held open in a hand: its page lies there, whichever it is (a journal tab turned to
    /// from the map in the hand is read there too: nothing leaves the hand but by letting go).
    pub fn held_open() -> bool {
        with_state(|s| s.hand.is_some() && s.fold == Fold::Open) && shows() && page().is_some()
    }

    /// The page in the hand is a map (the local or the world map): what a grip on it drags.
    pub fn held_map_page() -> bool {
        page().is_some_and(|p| p == PAGE_LOCAL || p == PAGE_WORLD)
    }

    /// Let go of (or put back), the game's map is shutting: nothing of it is shown meanwhile (it would
    /// float ahead for a frame or two as a menu nothing holds).
    pub fn put_away() -> bool {
        with_state(|s| matches!(s.fold, Fold::Closing(_)))
    }

    /// Once (in the world): the folded map to the runtime, in its slot, when it has one.
    pub fn publish(a: &rustvr_adapter::Adapter) {
        with_state(|s| {
            if s.published {
                return;
            }
            // (None by default for now: the body's slots are the holsters' to hand things out
            // from, and the map isn't one of the game's things.)
            let Some(slot) = slot_named(&crate::settings::get().map_slot) else { return };
            if crate::objects::publish(crate::objects::Owner::Map, &[desc(slot)], &[], a) {
                s.published = true;
                tracing::info!("map: kept in body slot {slot}");
            }
        });
    }

    /// Before the eyes are drawn: drawn from the belt, the game's map opened; put back, shut.
    pub fn place(now: f64) {
        with_state(|s| {
            if !s.published || !crate::link::with_adapter(|a| a.read_object_states(&mut s.states)).unwrap_or(false) {
                return;
            }
            let n = (s.states.count as usize).min(MAX_OBJECTS);
            let Some(st) = s.states.states[..n].iter().find(|st| st.id == MAP_ID) else { return };
            let hand = match st.flags {
                f if f & OS_HELD_LEFT != 0 => Some(0),
                f if f & OS_HELD_RIGHT != 0 => Some(1),
                _ => None,
            };
            if hand != s.hand {
                tracing::info!("map: {}", match (s.hand, hand) {
                    (None, Some(_)) => "drawn from the belt",
                    (Some(_), Some(_)) => "handed to the other hand",
                    _ => "put away",
                });
            }
            s.hand = hand;
            let menu = crate::menus::menu_mode();
            s.fold = match (s.fold, hand.is_some()) {
                (Fold::Away, true) if !menu => {
                    // As Menu Mode opens the last of the big four: the map, then.
                    type Interface = unsafe extern "C" fn(u8, u8) -> usize;
                    let ui = unsafe { std::mem::transmute::<usize, Interface>(INTERFACE)(0, 1) };
                    let root = if ui != 0 { mem::deref(ui + MENU_ROOT) } else { 0 };
                    if root == 0 {
                        return;
                    }
                    update_float(root, LAST_BIG_FOUR, MENU_MAP as f32);
                    crate::player::hold_control("Menu Mode", 0.12);
                    Fold::Opening(now)
                }
                (Fold::Opening(t), true) => {
                    if shows() {
                        // The local map in an interior, the world's outside: the game's own tabs.
                        let interior = mem::deref(mem::deref(TES) + TES_INTERIOR) != 0;
                        let want = if interior { PAGE_LOCAL } else { PAGE_WORLD };
                        if page() != Some(want) {
                            let m = tiles::menu(MENU_MAP);
                            let f = mem::deref(mem::deref(m) + VT_HANDLE_CLICK);
                            if m != 0 && (0x0040_1000..0x00A0_0000).contains(&f) {
                                type Click = unsafe extern "thiscall" fn(usize, u32, usize);
                                crate::hooks::guard_void("map page", || unsafe { std::mem::transmute::<usize, Click>(f)(m, want as u32, 0) });
                            }
                        }
                        tracing::info!("map: open on the {} map", if interior { "local" } else { "world" });
                        Fold::Open
                    } else if now - t > 2.0 {
                        tracing::info!("map: the game's map didn't open");
                        Fold::Shut
                    } else {
                        Fold::Opening(t)
                    }
                }
                (Fold::Open, true) if !tiles::shows(MENU_MAP) => Fold::Shut,
                (Fold::Open | Fold::Opening(_), false) => {
                    if shows() {
                        crate::player::hold_control("Menu Mode", 0.12);
                        Fold::Closing(now)
                    } else {
                        Fold::Away
                    }
                }
                (Fold::Closing(t), _) if menu && now - t < 1.0 => Fold::Closing(t),
                (Fold::Closing(_), _) => Fold::Away,
                (Fold::Shut, false) => Fold::Away,
                (f, _) => f,
            };
        });
    }
}
