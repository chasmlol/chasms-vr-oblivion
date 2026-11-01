//! The face shaped by hand in the race menu (`rustvr_protocol::FaceSculpt`, the runtime's
//! `sculpt`): the runtime's hands pull the face into shape within its own FaceGen geometry
//! coordinates, and this puts each face they make into the player's character as the race menu's
//! own sliders do (docs/design/facesculpt.md):
//!
//! * **The coordinates.** The face the runtime sends is the whole face (the race's coordinates and
//!   the character's added, as the game adds them, 0x552990); the character's own set (the
//!   `TESNPC`'s, which a save keeps) becomes that less the race's, in place, as 0x5C6390 does for a
//!   slider.
//! * **The game's own head** (its third-person and first-person FaceGen heads, which the headset
//!   never shows: the runtime draws the player) is morphed by the change, as 0x5C6390 morphs it for
//!   a slider: each FaceGen part's model (`BSFaceGenModel`, from its extra data) applies the change
//!   to its geometry (0x558840), the hair's length morph put back after.
//! * **The sliders follow.** The page of sliders on show is refreshed from the face (0x5C6AF0, what
//!   the menu does to a page's other sliders when one moves); a page shown later is refreshed as it
//!   comes up. The sliders' range is the race's: `limits` sends it to the runtime, which keeps the
//!   face inside it (the general page's sliders go as far as the race's face clamp, `TESRace`
//!   +0xA4, 3 by default; the others' its main clamp, +0xA0, 5).
//! * **Undo and reset**, in the race menu itself: buttons made from its own button template
//!   (`race_template_button`, the one its Race, Face and Hair buttons are made from), on its main
//!   page (Undo, Reset Face: the race's own face, its age and complexion kept) and on each of the
//!   face's shape pages (Undo, Reset: that page's features back to the race's own). Their clicks are
//!   taken from the menu's `HandleClick` (0x5CC200, hooked: the game's own ids go on to it; the VR
//!   keyboard's keys over the name are [`crate::keyboard`]'s). An undo goes back to the face before
//!   the last pull or reset; the menu closing forgets them.
//!
//! The race menu (`RaceSexMenu`, its `Menu*`): its pages' tiles from +0x40 (main) and +0x58 (the
//! shape pages: general, brow, cheeks, chin, eyes, forehead, jaw, mouth, nose); its tile's `user0`
//! the id of the page on show. A coordinate set is four `{count, columns, allocator, first, last,
//! end}` vectors of 0x18 bytes (symmetric and asymmetric geometry, symmetric and asymmetric texture).

use crate::mem;
use rustvr_protocol::FACE_CONTROLS;

/// The general page's controls, which go as far as the race's face clamp (the rest its main one).
const GENERAL: std::ops::RangeInclusive<usize> = 19..=23;
/// `TESRace`'s FaceGen clamps, main and face, and the game's own when a race has none.
const RACE_MAIN_CLAMP: usize = 0xA0;
const RACE_FACE_CLAMP: usize = 0xA4;
const MAIN_CLAMP: f32 = 5.0;
const FACE_CLAMP: f32 = 3.0;

/// How far each of FaceGen's named controls may go for this race, as its sliders let them.
pub fn limits(race: usize) -> [f32; FACE_CONTROLS] {
    let clamp = |at: usize, own: f32| mem::at::<f32>(race + at).filter(|c| *c > 0.0 && c.is_finite()).unwrap_or(own);
    let (main, face) = (clamp(RACE_MAIN_CLAMP, MAIN_CLAMP), clamp(RACE_FACE_CLAMP, FACE_CLAMP));
    let mut out = [0.0; FACE_CONTROLS];
    for (i, l) in out.iter_mut().enumerate().take(56) {
        *l = if GENERAL.contains(&i) { face } else { main };
    }
    out
}

/// The race menu itself: its buttons, its clicks, its sliders (the game's tiles are 32-bit Windows').
#[cfg(all(windows, target_arch = "x86"))]
mod imp {
    use super::*;
    use crate::{body, tiles};
    use rustvr_protocol::FaceSculpt;
    use std::sync::Mutex;

    /// The race menu's page tiles: main, and the shape pages with the face controls each has (their
    /// indices in FaceGen's named controls, `si.ctl`: 5CA010 makes their sliders).
    const MAIN_PAGE: usize = 0x40;
    const SHAPE_PAGES: [(usize, &str, std::ops::RangeInclusive<usize>); 9] = [
        (0x58, "General", 19..=23),
        (0x5C, "Brow", 0..=2),
        (0x60, "Cheeks", 3..=7),
        (0x64, "Chin", 8..=14),
        (0x68, "Eyes", 15..=18),
        (0x6C, "Forehead", 24..=26),
        (0x70, "Jaw", 27..=30),
        (0x74, "Mouth", 31..=40),
        (0x78, "Nose", 41..=54),
    ];

    /// Our buttons' ids (the menu's own are under 300): undo, reset the whole face, reset a page's.
    const UNDO: i32 = 9100;
    const RESET_FACE: i32 = 9101;
    const RESET_PAGE: i32 = 9110;

    /// The game's functions: the template made into a tile (thiscall `(menu, parent, name, 0)`), a
    /// page's sliders read from the face again (thiscall `(menu, page, a slider to leave)`), the race
    /// menu's `HandleClick` (thiscall `(menu, id, tile)`), a tile's value set (thiscall `(tile, id,
    /// value)`, number and text).
    const TEMPLATE: usize = 0x0058_5410;
    const REFRESH_PAGE: usize = 0x005C_6AF0;
    const HANDLE_CLICK: usize = 0x005C_C200;
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const UPDATE_STRING: usize = 0x0058_CED0;
    /// Tile values: id, list index, its number of children, user0 (the page on show, on the menu's
    /// tile), user1, user2 (a button's label), user3 (a manual set), user6 (its text).
    const ID: u32 = 0xFA8;
    const LIST_INDEX: u32 = 0xFAA;
    const CHILD_COUNT: u16 = 0xFD0;
    const USER0: u16 = 0xFAE;
    const USER1: u32 = 0xFAF;
    const USER2: u32 = 0xFB0;
    const USER3: u32 = 0xFB1;
    const USER6: u32 = 0xFB4;
    const MENU_TILE: usize = 4;

    /// The game's own head: the player's FaceGen heads (its vtable, thiscall `(player, 0)`: third and
    /// first person), a node's children (+0xB0, their count +0xB6), a child's geometry (vtable +0x10),
    /// its FaceGen model's extra data (cdecl), that's model (vtable +0x54), the model's morph applied
    /// (thiscall `(model, coords, geometry, 1.0, 0)`), the hair's length morph (cdecl `(geometry,
    /// length)`, the NPC's +0x1CC).
    const PLAYER: usize = 0x00B3_33C4;
    const HEADS: [usize; 2] = [0x130, 0x134];
    const CHILDREN: usize = 0xB0;
    const CHILD_COUNT_NODE: usize = 0xB6;
    const AS_GEOMETRY: usize = 0x10;
    const MODEL_DATA: usize = 0x0055_0790;
    const MODEL_OF: usize = 0x54;
    const APPLY_MORPH: usize = 0x0055_8840;
    const HAIR_LENGTH: usize = 0x0055_0980;
    const NPC_HAIR_LENGTH: usize = 0x1CC;

    #[derive(Default)]
    struct State {
        /// The race menu it's for (its `Menu*`), and the runtime's face last taken (its seq, its pull).
        menu: usize,
        seq: Option<u32>,
        stroke: u32,
        /// The faces before each pull or reset (symmetric, asymmetric), the last on top.
        undo: Vec<(Vec<f32>, Vec<f32>)>,
        /// The pages whose sliders show an older face, and the page on show last frame.
        stale: [bool; 9],
        shown: Option<f32>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);
    static CLICKS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

    /// Once a game frame (before the face is published): while the race menu is open, its buttons, the
    /// clicks on them, and the face the runtime's hands make.
    pub fn update(a: &rustvr_adapter::Adapter) {
        let menu = tiles::menu(crate::menus::MENU_RACE_SEX);
        let mut state = STATE.lock().unwrap();
        if menu == 0 {
            if state.as_ref().is_some_and(|s| s.menu != 0) {
                tracing::info!("facesculpt: the race menu closed: its undos forgotten");
            }
            *state = None;
            CLICKS.lock().unwrap().clear();
            return;
        }
        install();
        let s = state.get_or_insert_with(State::default);
        if s.menu != menu {
            *s = State { menu, ..Default::default() };
            tracing::info!("facesculpt: the race menu is open ({menu:#x}): the face may be shaped by hand");
        }
        buttons(menu);
        let clicks: Vec<i32> = std::mem::take(&mut *CLICKS.lock().unwrap());
        for id in clicks {
            match id {
                UNDO => match s.undo.pop() {
                    Some(face) => {
                        tracing::info!("facesculpt: undo ({} more to go back)", s.undo.len());
                        apply(s, &face.0, &face.1);
                    }
                    None => tracing::info!("facesculpt: nothing to undo"),
                },
                RESET_FACE => reset(s, None),
                id if (RESET_PAGE..RESET_PAGE + SHAPE_PAGES.len() as i32).contains(&id) => reset(s, Some((id - RESET_PAGE) as usize)),
                _ => {}
            }
        }
        // The face the hands make (not one written before the menu opened).
        let mut f: FaceSculpt = bytemuck::Zeroable::zeroed();
        if a.read_face_sculpt(&mut f) {
            match s.seq {
                None => {
                    s.seq = Some(f.seq);
                    s.stroke = f.stroke;
                }
                Some(seq) if seq != f.seq => {
                    s.seq = Some(f.seq);
                    if f.stroke != s.stroke {
                        s.stroke = f.stroke;
                        if let Some(now) = face_now() {
                            s.undo.push(now);
                        }
                    }
                    apply(s, &f.geometry_symmetric, &f.geometry_asymmetric);
                }
                _ => {}
            }
        }
        refresh(s, menu);
    }

    /// The player's face now, whole (the race's and the character's): symmetric, asymmetric geometry.
    fn face_now() -> Option<(Vec<f32>, Vec<f32>)> {
        let (npc_set, race_set, ..) = body::face_sets()?;
        let add = |a: Vec<f32>, b: Vec<f32>| (0..a.len().max(b.len())).map(|i| a.get(i).copied().unwrap_or(0.0) + b.get(i).copied().unwrap_or(0.0)).collect::<Vec<f32>>();
        Some((add(body::coords(race_set, 0), body::coords(npc_set, 0)), add(body::coords(race_set, 1), body::coords(npc_set, 1))))
    }

    /// Puts a whole face (symmetric, asymmetric geometry) into the character: its own coordinates
    /// the face less the race's, its head morphed by the change, the sliders to follow.
    fn apply(s: &mut State, symmetric: &[f32], asymmetric: &[f32]) {
        let Some((npc_set, race_set, npc, _)) = body::face_sets() else { return };
        let Some(before) = face_now() else { return };
        let mut change = (vec![0.0f32; before.0.len()], vec![0.0f32; before.1.len()]);
        for (which, (want, was, delta)) in [(symmetric, &before.0, &mut change.0), (asymmetric, &before.1, &mut change.1)].into_iter().enumerate() {
            let race = body::coords(race_set, which);
            let v = npc_set + which * 0x18;
            let first = mem::deref(v + 0x0C);
            let room = (mem::deref(v + 0x10).saturating_sub(first)) / 4;
            let n = room.min(want.len()).min(was.len());
            if first == 0 || n == 0 {
                tracing::warn!("facesculpt: the character's {} coordinates have no room ({room}): left as they were", ["symmetric", "asymmetric"][which]);
                continue;
            }
            for i in 0..n {
                let own = want[i] - race.get(i).copied().unwrap_or(0.0);
                if own.is_finite() {
                    unsafe { ((first + 4 * i) as *mut f32).write(own) };
                    delta[i] = want[i] - was[i];
                }
            }
        }
        morph_heads(npc, &change.0, &change.1);
        s.stale = [true; 9];
        s.shown = None;
        static SAID: Mutex<Option<std::time::Instant>> = Mutex::new(None);
        let mut said = SAID.lock().unwrap();
        if said.is_none_or(|t| t.elapsed().as_secs_f32() > 1.0) {
            *said = Some(std::time::Instant::now());
            let size = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
            tracing::info!("facesculpt: the face shaped by hand put into the character (changed by {:.3} symmetric, {:.3} asymmetric)", size(&change.0), size(&change.1));
        }
    }

    /// A reset: the whole face back to the race's own (None: its age and complexion kept), or a shape
    /// page's features (theirs set to the race's, the rest of the face as little changed as it can be).
    fn reset(s: &mut State, page: Option<usize>) {
        let (Some((_, race_set, ..)), Some(now)) = (body::face_sets(), face_now()) else { return };
        let Some(controls) = controls() else { return };
        let race = (body::coords(race_set, 0), body::coords(race_set, 1));
        let (mut sym, mut asym) = now.clone();
        match page {
            None => {
                // The race's face, at the age and complexion it has now.
                let kept = controls.kept();
                let want: Vec<f64> = kept.iter().map(|k| k.iter().zip(&now.0).map(|(a, b)| a * b).sum::<f32>() as f64 - k.iter().zip(&race.0).map(|(a, b)| a * b).sum::<f32>() as f64).collect();
                let d = rustvr_asset::facegen::least_change(&kept, &want, race.0.len());
                sym = race.0.iter().zip(d.iter().chain(std::iter::repeat(&0.0))).map(|(r, d)| r + d).collect();
                asym = race.1.clone();
                tracing::info!("facesculpt: the face reset to the race's own");
            }
            Some(p) => {
                let (_, name, range) = &SHAPE_PAGES[p];
                let targets: Vec<(usize, f32)> = range.clone().map(|i| (i, controls.value(i, &race.0))).collect();
                controls.set(&mut sym, &targets);
                tracing::info!("facesculpt: the {name} page's features reset to the race's own");
            }
        }
        if (sym.clone(), asym.clone()) != now {
            s.undo.push(now);
            apply(s, &sym, &asym);
        }
    }

    /// FaceGen's controls (the extractor's `facegen.toml`), read once.
    fn controls() -> Option<&'static rustvr_asset::FaceControls> {
        static READ: std::sync::OnceLock<Option<rustvr_asset::FaceControls>> = std::sync::OnceLock::new();
        READ.get_or_init(|| rustvr_asset::load_face_controls(&rustvr_asset::default_root().join("oblivion")).map_err(|e| tracing::warn!("facesculpt: FaceGen's controls: {e:#}")).ok()).as_ref()
    }

    /// The page on show refreshed when its sliders show an older face (or it's just come up).
    fn refresh(s: &mut State, menu: usize) {
        let shown = tiles::value(mem::deref(menu + MENU_TILE), USER0);
        if shown == s.shown {
            return;
        }
        s.shown = shown;
        for (i, (at, ..)) in SHAPE_PAGES.iter().enumerate() {
            let page = mem::deref(menu + at);
            if s.stale[i] && page != 0 && shown.is_some() && tiles::value(page, ID as u16) == shown {
                s.stale[i] = false;
                unsafe {
                    std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, usize, usize)>(REFRESH_PAGE)(menu, page, 0);
                }
            }
        }
    }

    fn update_float(tile: usize, id: u32, v: f32) {
        unsafe {
            std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32, f32)>(UPDATE_FLOAT)(tile, id, v)
        };
    }

    fn update_string(tile: usize, id: u32, s: &str) {
        let Ok(c) = std::ffi::CString::new(s) else { return };
        unsafe {
            std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32, *const u8)>(UPDATE_STRING)(tile, id, c.as_ptr() as *const u8)
        };
    }

    /// Our buttons on the menu's pages, made from its own template wherever one isn't there yet (the
    /// menu makes its pages again as the race changes).
    fn buttons(menu: usize) {
        let pages = std::iter::once((MAIN_PAGE, vec![(UNDO, "Undo"), (RESET_FACE, "Reset Face")]))
            .chain(SHAPE_PAGES.iter().enumerate().map(|(i, (at, ..))| (*at, vec![(UNDO, "Undo"), (RESET_PAGE + i as i32, "Reset")])));
        for (at, wanted) in pages {
            let page = mem::deref(menu + at);
            if page == 0 {
                continue;
            }
            let have: Vec<i32> = tiles::children(page).into_iter().filter_map(|t| tiles::value(t, ID as u16)).map(|v| v as i32).collect();
            for (id, text) in wanted {
                if have.contains(&id) {
                    continue;
                }
                let tile = unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, usize, *const u8, usize) -> usize>(TEMPLATE)(menu, page, c"race_template_button".as_ptr() as *const u8, 0) };
                if tile == 0 {
                    tracing::warn!("facesculpt: the race menu's button template made nothing");
                    return;
                }
                // As the menu sets up its own buttons (0x5C4480): its id, its place in the list, its
                // starting value, no label, its text.
                let count = tiles::value(page, CHILD_COUNT).unwrap_or(tiles::children(page).len() as f32);
                update_float(tile, ID, id as f32);
                update_float(tile, LIST_INDEX, count - 1.0);
                update_float(tile, USER1, 0.0);
                update_string(tile, USER2, "   ");
                update_float(tile, USER3, -2147483600.0);
                update_float(tile, USER3, 0.0);
                update_string(tile, USER6, text);
                tracing::info!("facesculpt: the race menu's \"{text}\" button ({id}) made on its page {at:#x}, row {}", count - 1.0);
            }
        }
    }

    /// Once: the race menu's clicks, ours taken off before it.
    fn install() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            if crate::hooks::install_into(HANDLE_CLICK as *const (), click as *const (), "RaceSexMenu::HandleClick", &REAL_CLICK) {
                tracing::info!("facesculpt: the race menu's clicks are seen first (Undo and Reset)");
            }
        });
    }

    static REAL_CLICK: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    unsafe extern "thiscall" fn click(menu: usize, id: i32, tile: usize) {
        // (The VR keyboard's keys over the name: `crate::keyboard`.)
        if crate::keyboard::click(id) {
            return;
        }
        if (UNDO..RESET_PAGE + SHAPE_PAGES.len() as i32).contains(&id) {
            crate::hooks::guard_void("race menu click", || {
                tracing::info!("facesculpt: the race menu's button {id} clicked");
                CLICKS.lock().unwrap().push(id);
            });
            return;
        }
        let real: unsafe extern "thiscall" fn(usize, i32, usize) = unsafe { std::mem::transmute(REAL_CLICK.load(std::sync::atomic::Ordering::Acquire)) };
        unsafe { real(menu, id, tile) }
    }

    /// A coordinate set as the game lays one out: four vectors (count, columns, allocator, first,
    /// last, end).
    #[repr(C)]
    struct GameVector {
        count: u32,
        columns: u32,
        allocator: u32,
        first: *const f32,
        last: *const f32,
        end: *const f32,
    }

    /// The game's own heads morphed by a change of the face (symmetric, asymmetric geometry), as its
    /// sliders morph them (0x5C6390). The change is lent to the game as a coordinate set of our own:
    /// the morph only reads it.
    fn morph_heads(npc: usize, symmetric: &[f32], asymmetric: &[f32]) {
        if symmetric.iter().chain(asymmetric).all(|x| *x == 0.0) {
            return;
        }
        let vector = |v: &[f32]| GameVector { count: v.len() as u32, columns: 1, allocator: 0, first: v.as_ptr(), last: v.as_ptr_range().end, end: v.as_ptr_range().end };
        let none = GameVector { count: 0, columns: 0, allocator: 0, first: std::ptr::null(), last: std::ptr::null(), end: std::ptr::null() };
        let none2 = GameVector { count: 0, columns: 0, allocator: 0, first: std::ptr::null(), last: std::ptr::null(), end: std::ptr::null() };
        let set = [vector(symmetric), vector(asymmetric), none, none2];
        let player = mem::deref(PLAYER);
        let hair = mem::at::<f32>(npc + NPC_HAIR_LENGTH).unwrap_or(0.0);
        let mut morphed = 0;
        for slot in HEADS {
            let f = mem::deref(mem::deref(player) + slot);
            if player == 0 || !(0x0040_1000..0x00A3_0000).contains(&f) {
                continue;
            }
            let head = unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32) -> usize>(f)(player, 0) };
            if head == 0 {
                continue;
            }
            let children = mem::deref(head + CHILDREN);
            let n = mem::at::<u16>(head + CHILD_COUNT_NODE).unwrap_or(0) as usize;
            for i in 0..n.min(64) {
                let child = mem::deref(children + 4 * i);
                if child == 0 {
                    continue;
                }
                unsafe {
                    let geometry = std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(mem::deref(mem::deref(child) + AS_GEOMETRY))(child);
                    if geometry == 0 {
                        continue;
                    }
                    let is_hair = crate::combat::read_str(mem::deref(geometry + 8)) == "FaceGenHair";
                    let hair_length = std::mem::transmute::<usize, unsafe extern "C" fn(usize, f32)>(HAIR_LENGTH);
                    if is_hair {
                        hair_length(geometry, hair);
                    }
                    let data = std::mem::transmute::<usize, unsafe extern "C" fn(usize) -> usize>(MODEL_DATA)(geometry);
                    if data == 0 {
                        continue;
                    }
                    let model_of = std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(mem::deref(mem::deref(data) + MODEL_OF));
                    let model = model_of(data);
                    if model == 0 {
                        continue;
                    }
                    std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, *const GameVector, usize, f32, u32) -> u8>(APPLY_MORPH)(model, set.as_ptr(), geometry, 1.0, 0);
                    if is_hair {
                        hair_length(geometry, hair);
                    }
                    morphed += 1;
                }
            }
        }
        static SAID: std::sync::Once = std::sync::Once::new();
        SAID.call_once(|| tracing::info!("facesculpt: the game's own heads morphed with the face ({morphed} parts)"));
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use imp::update;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn update(_a: &rustvr_adapter::Adapter) {}
