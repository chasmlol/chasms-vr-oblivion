//! Books and notes read in the hands (`docs/research/books.md`): a book lying about, taken in a
//! hand ([`crate::items`]), is read as soon as it's there, and Oblivion reads it exactly as
//! activating it would (the reference's own `Activate` by the player): its book menu with the
//! book's text laid out in pages by the game, its "Take" (with its stealing), a skill book's skill
//! rise and message, a note's script, "you can't read in combat". The book menu then floats over the
//! book in the hand, turned to the eyes ([`crate::hud`]: `ANCHOR_HELD`, `HELD_FACING`), its pages
//! turned by a flick of the other hand (the game's own page turn, [`crate::input`]: `PageNext`,
//! `PagePrev`).
//!
//! It's read once while it's in the hands: shut by the game's own Exit (`HandleClick(31)`), with B,
//! or refused by the game, it stays shut until it's been let go of and taken again. Let go of (or
//! taken from the hand) while it's open, it's shut too. A magic scroll isn't read: the game takes
//! one that's activated rather than reading it, so it would go straight to the pack.
//!
//! The game goes on while a book is read in the hands ([`install`]): Oblivion stops its world while
//! any menu is open, and a book held open is a menu.
//!
//! The book menu (xOBSE's `GameMenus.h`, `BookMenu`, menu type 0x402): the reference read at +0x30
//! (0 from the inventory), the book at +0x34 (`TESObjectBOOK`: its flags +0x88, 1 a scroll).
//! `Menu::HandleClick(id, tile)` is the menu's vtable slot 3.

use crate::{mem, tiles};
use rustvr_protocol::Action;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const MENU_BOOK: u32 = 0x402;
const BOOK_REF: usize = 0x30;
const BOOK_FORM: usize = 0x34;
const BOOK_FLAGS: usize = 0x88;
/// The skill a book teaches (0xFF none: read, a skill book teaches nothing more).
const BOOK_SKILL: usize = 0x89;
const SCROLL: u8 = 1;
/// The book's enchantment (`TESEnchantableForm` +0x60, its `EnchantmentItem` +4): a magic scroll's.
const BOOK_ENCHANTMENT: usize = 0x64;
/// A reference's base form.
const REFR_BASE: usize = 0x1C;
const VT_HANDLE_CLICK: usize = 3 * 4;
const EXIT: u32 = 31;
/// `TESForm` form id.
const FORM_ID: usize = 0xC;

/// The book menu, if it's open: (`BookMenu*`, the reference being read or 0).
pub fn open() -> Option<(usize, usize)> {
    let m = tiles::menu(MENU_BOOK);
    (m != 0).then(|| (m, mem::deref(m + BOOK_REF)))
}

/// The book menu is the menu on top.
#[cfg(target_arch = "x86")]
pub fn reading() -> bool {
    tiles::menu_tile(MENU_BOOK) != 0 && tiles::top_menu() == MENU_BOOK
}

#[cfg(not(target_arch = "x86"))]
pub fn reading() -> bool {
    false
}

/// What's being read is a scroll (a note: one long page) rather than a book.
pub fn is_scroll(menu: usize) -> bool {
    let book = mem::deref(menu + BOOK_FORM);
    book != 0 && mem::at::<u8>(book + BOOK_FLAGS).is_some_and(|f| f & SCROLL != 0)
}

/// The held thing (the runtime's object) the book menu is reading, if a hand holds it.
pub fn held_book() -> Option<u32> {
    let (_, refr) = open()?;
    (refr != 0).then(|| crate::items::held().into_iter().find(|h| h.refr == refr).map(|h| h.id)).flatten()
}

/// The thing (the runtime's object) the book menu is reading where it lies (on a shelf, activated
/// by looking at it), if it's one the hands can take: never one a hand had while it was open (let
/// go of, it's being shut; brought back, it would leap into the other hand and stay there).
pub fn lying_book() -> Option<u32> {
    let (_, refr) = open()?;
    (refr != 0 && HELD_OPEN.load(Ordering::Relaxed) != refr).then(|| crate::items::offered(refr)).flatten()
}

/// The reference of the book being read, once a hand has had it open (until its menu shuts).
static HELD_OPEN: AtomicUsize = AtomicUsize::new(0);

/// Shuts the book with the game's own Exit.
fn shut(menu: usize) {
    let f = mem::deref(mem::deref(menu) + VT_HANDLE_CLICK);
    if !(0x0040_1000..0x00A0_0000).contains(&f) {
        return;
    }
    type Click = unsafe extern "thiscall" fn(usize, u32, usize);
    crate::hooks::guard_void("book exit", || unsafe { std::mem::transmute::<usize, Click>(f)(menu, EXIT, 0) });
}

#[derive(Default)]
struct State {
    /// The held books read since they came to the hands (their references): each is read once while
    /// it's held, so its Exit, B or the game's refusal isn't undone the next frame.
    read_held: Vec<usize>,
    /// The held book being read (its reference), while its menu's open.
    reading: Option<usize>,
    /// B's presses as last seen.
    back: Option<u16>,
    /// A book just read: its form and the skill it taught before (for the log: the game's own skill
    /// rise is that it teaches nothing now).
    read: Option<(usize, u8)>,
}

static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);

/// Once a game frame, where the game's scripts run: a book just taken in a hand read, one let go of
/// while it's open shut, or one shut with B; then whether the world goes on under it this frame.
pub fn frame() {
    step();
    READING_IN_HAND.store(reading() && held_book().is_some(), Ordering::Relaxed);
}

fn step() {
    let mut s = STATE.lock().unwrap();
    let s = s.get_or_insert_with(State::default);
    let held = crate::items::held();
    let book = open();
    if let Some((base, before)) = s.read.take()
        && before != 0xFF
    {
        let now = mem::at::<u8>(base + BOOK_SKILL).unwrap_or(0);
        tracing::info!("books: a skill book (skill {before}) read: {}", if now == 0xFF { "the game raised the skill" } else { "it still teaches (the skill was at its most)" });
    }
    // B's presses (the runtime's "back"): on a book, the book's own Exit.
    let back = crate::link::last().map(|l| l.frame.intents.presses[Action::MenuBack as usize]);
    let pressed_back = matches!((s.back, back), (Some(a), Some(b)) if a != b);
    s.back = back;
    if let (true, Some((m, _))) = (pressed_back && reading(), book) {
        tracing::info!("books: shut with B");
        shut(m);
        return;
    }
    // Read once while it's in the hands (one brought to the hand by looking at it was read where it
    // lay).
    s.read_held.retain(|r| held.iter().any(|h| h.refr == *r));
    if let Some((_, refr)) = book
        && refr != 0
        && held.iter().any(|h| h.refr == refr)
        && !s.read_held.contains(&refr)
    {
        s.read_held.push(refr);
    }
    if book.is_none()
        && !crate::menus::menu_mode()
        && let Some(h) = held.iter().find(|h| h.kind == crate::items::BOOK && !s.read_held.contains(&h.refr))
    {
        s.read_held.push(h.refr);
        let id = mem::at::<u32>(h.refr + FORM_ID).unwrap_or(h.id);
        let base = mem::deref(h.refr + REFR_BASE);
        if base != 0 && mem::deref(base + BOOK_ENCHANTMENT) != 0 {
            tracing::info!("books: a magic scroll in the hand ({id:08X}): not read (the game would take it)");
        } else {
            // Read as activating it does: the game's own book menu, skill rise and script.
            s.read = Some((base, mem::at::<u8>(base + BOOK_SKILL).unwrap_or(0xFF)));
            let ok = crate::items::activate(h.refr);
            tracing::info!("books: taken in the hand ({id:08X}): {}", if ok { "read" } else { "not read" });
        }
    }
    // Being read and let go of (or taken from the hand): shut.
    match book {
        Some((m, refr)) if refr != 0 => {
            let in_hand = held.iter().any(|h| h.refr == refr);
            if in_hand {
                HELD_OPEN.store(refr, Ordering::Relaxed);
            }
            if s.reading == Some(refr) && !in_hand {
                tracing::info!("books: let go of: shut");
                shut(m);
                s.reading = None;
            } else if in_hand {
                s.reading = Some(refr);
            }
        }
        _ => {
            s.reading = None;
            HELD_OPEN.store(0, Ordering::Relaxed);
        }
    }
}

/// The book being read is in a hand and its menu is on top (as [`frame`] last found, before the
/// game's frame runs: the checks below are asked in the middle of it).
static READING_IN_HAND: AtomicBool = AtomicBool::new(false);

/// `IsMenuMode` (cdecl: the interface's cursor is out, `InterfaceManager` +0x1C, and its mode, +8,
/// isn't 1).
const IS_MENU_MODE: usize = 0x0057_8F60;
/// Where the world's update asks it, each a 5-byte call: the game's frame (0x40D800) before each
/// part of the world's update (the sound's pause, the actors' AI and processing, the player's own
/// update and the cell grid round them, the sky and the time of day, Havok's step), and the world's
/// seconds for the frame (0x405290, 0 while a menu is open), which all of it goes by.
const WORLD_MENU_CHECKS: [usize; 15] = [
    0x0040_D809,
    0x0040_D817,
    0x0040_D9C6,
    0x0040_D9D4,
    0x0040_DA78,
    0x0040_DB5B,
    0x0040_DBAB,
    0x0040_DBFB,
    0x0040_DC61,
    0x0040_DD5F,
    0x0040_DD6E,
    0x0040_DE3F,
    0x0040_DE76,
    0x0040_DEA6,
    0x0040_5297,
];

/// The world's own question, "is a menu open?": not while a book is read in the hands, so the game
/// goes on as it does with no menu (time, the actors, the weather, the physics, the sounds). Every
/// other asker still sees the menu: its cursor and clicks are the book's, the player's Activate stays
/// off (the hand on the pages isn't activating), scripts run their menu-mode blocks.
#[cfg(target_arch = "x86")]
extern "C" fn world_menu_mode() -> u8 {
    let menu = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn() -> u8>(IS_MENU_MODE)() };
    if menu != 0 && READING_IN_HAND.load(Ordering::Relaxed) { 0 } else { menu }
}

/// Once: the world's checks ask [`world_menu_mode`] (each only if it's the call the build had).
#[cfg(target_arch = "x86")]
pub fn install() {
    let ours = world_menu_mode as *const () as usize;
    let mut done = 0;
    for at in WORLD_MENU_CHECKS {
        let real = (IS_MENU_MODE as i32).wrapping_sub(at as i32 + 5).to_le_bytes();
        let rel = (ours as i32).wrapping_sub(at as i32 + 5).to_le_bytes();
        done += mem::patch_code(at, &[0xE8, real[0], real[1], real[2], real[3]], &[0xE8, rel[0], rel[1], rel[2], rel[3]]) as u32;
    }
    tracing::info!("books: the world goes on while a book is read in the hands ({done} of {} of its menu checks)", WORLD_MENU_CHECKS.len());
}

#[cfg(not(target_arch = "x86"))]
pub fn install() {}
