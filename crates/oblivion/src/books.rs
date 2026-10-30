//! Books and notes read in the hands (`docs/research/books.md`): a book lying about, taken in one
//! hand ([`crate::items`]), is opened by the other hand's grip on it ([`OBJF_OPENS`]), and Oblivion
//! reads it exactly as activating it would (the reference's own `Activate` by the player): its
//! book menu with the book's text laid out in pages by the game, its "Take" (with its stealing), a
//! skill book's skill rise and message, a note's script, "you can't read in combat". The book menu then lies on the
//! book in the hand ([`crate::hud`]: `ANCHOR_HELD`), its pages turned by the other hand (the game's
//! own page turn, [`crate::input`]: `PageNext`, `PagePrev`).
//!
//! It's shut by the game's own Exit (`HandleClick(31)`): when the other hand grips it again, when
//! it's let go of (or taken from the hand), or with B.
//!
//! The book menu (xOBSE's `GameMenus.h`, `BookMenu`, menu type 0x402): the reference read at +0x30
//! (0 from the inventory), the book at +0x34 (`TESObjectBOOK`: its flags +0x88, 1 a scroll).
//! `Menu::HandleClick(id, tile)` is the menu's vtable slot 3.

use crate::{mem, tiles};
use rustvr_protocol::Action;

pub const MENU_BOOK: u32 = 0x402;
const BOOK_REF: usize = 0x30;
const BOOK_FORM: usize = 0x34;
const BOOK_FLAGS: usize = 0x88;
/// The skill a book teaches (0xFF none: read, a skill book teaches nothing more).
const BOOK_SKILL: usize = 0x89;
const SCROLL: u8 = 1;
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
/// by looking at it), if it's one the hands can take.
pub fn lying_book() -> Option<u32> {
    let (_, refr) = open()?;
    (refr != 0).then(|| crate::items::offered(refr)).flatten()
}

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
    /// Each held thing's opens as last seen.
    opens: Vec<(u32, u32)>,
    /// The held book being read (its reference), while its menu's open.
    reading: Option<usize>,
    /// B's presses as last seen.
    back: Option<u16>,
    /// A book just read: its form and the skill it taught before (for the log: the game's own skill
    /// rise is that it teaches nothing now).
    read: Option<(usize, u8)>,
}

static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);

/// Once a game frame, where the game's scripts run: a held book opened or shut by the other hand,
/// let go of while it's open, or shut with B.
pub fn frame() {
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
    for h in held.iter().filter(|h| h.kind == crate::items::BOOK) {
        let last = s.opens.iter().find(|(id, _)| *id == h.id).map(|&(_, n)| n);
        if let Some(n) = last
            && h.opens > n
        {
            match book {
                Some((m, refr)) if refr == h.refr => {
                    tracing::info!("books: shut by the other hand");
                    shut(m);
                }
                None if !crate::menus::menu_mode() => {
                    // Read as activating it does: the game's own book menu, skill rise and script.
                    let id = mem::at::<u32>(h.refr + FORM_ID).unwrap_or(h.id);
                    let base = mem::deref(h.refr + REFR_BASE);
                    s.read = Some((base, mem::at::<u8>(base + BOOK_SKILL).unwrap_or(0xFF)));
                    let ok = crate::items::activate(h.refr);
                    tracing::info!("books: opened by the other hand ({id:08X}): {}", if ok { "read" } else { "not read" });
                }
                _ => {}
            }
        }
    }
    s.opens = held.iter().map(|h| (h.id, h.opens)).collect();
    // Being read and let go of (or taken from the hand): shut.
    match book {
        Some((m, refr)) if refr != 0 => {
            let in_hand = held.iter().any(|h| h.refr == refr);
            if s.reading == Some(refr) && !in_hand {
                tracing::info!("books: let go of: shut");
                shut(m);
                s.reading = None;
            } else if in_hand {
                s.reading = Some(refr);
            }
        }
        _ => s.reading = None,
    }
}
