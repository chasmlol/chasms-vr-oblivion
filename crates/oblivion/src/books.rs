//! Books and notes read in the hands (`docs/design/books.md`): a book lying about, taken in a
//! hand ([`crate::items`]), is read as soon as it's there, and Oblivion reads it exactly as
//! activating it would (the reference's own `Activate` by the player): its book menu with the
//! book's text laid out in pages by the game, its "Take" (with its stealing), a skill book's skill
//! rise and message, a note's script, "you can't read in combat". The book menu then floats over the
//! book in the hand, turned to the eyes ([`crate::hud`]: `ANCHOR_HELD`, `HELD_FACING`), its pages
//! turned by the other hand's fingertip swiped across them (`PageNext`, `PagePrev`), its buttons
//! (Take, Exit, the page turns) pressed by that fingertip.
//!
//! A book is held as anything else is: the game plays on (its world, the player's walking and
//! fighting, the HUD), and the book's pages just float on it. Oblivion goes into its menus (its world
//! stopped, the player's controls the menu's, the HUD gone) when a menu goes on the interface's menu
//! stack (`InterfaceManager` +0xE0; 0x57D640 pushes one, and the first sets the interface's mode,
//! +8, from play, 1, to 3). A book opened with no menu up isn't pushed ([`install`]), so the game
//! never leaves play; and the game's cursor and mouse are then its play (the left button swings), so
//! its pages are turned and its buttons pressed on the menu itself: the step its own wheel takes
//! (0x595DA0), its own `HandleClick`.
//!
//! It's read once while it's in the hands: shut by the game's own Exit (`HandleClick(31)`), or
//! refused by the game, it stays shut until it's been let go of and taken again. Let go of (or taken
//! from the hand) while it's open, it's shut too. A magic scroll isn't read: the game takes one
//! that's activated rather than reading it, so it would go straight to the pack. One read from the
//! pack is over the inventory, a menu, and is the game's menu as ever (B shuts it).
//!
//! The book menu (xOBSE's `GameMenus.h`, `BookMenu`, menu type 0x402): the reference read at +0x30
//! (0 from the inventory), the book at +0x34 (`TESObjectBOOK`: its flags +0x88, 1 a scroll).
//! `Menu::HandleClick(id, tile)` is the menu's vtable slot 3.

/// The book menu (`kMenuType_Book`).
pub const MENU_BOOK: u32 = 0x402;

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn reading() -> bool {
    false
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{mem, tiles};
    use rustvr_protocol::{Action, INT_CURSOR, INT_CURSOR_DOWN};
    use std::sync::atomic::{AtomicUsize, Ordering};

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
    /// The book's buttons' ids (`book_menu.xml`, a book's and a scroll's alike).
    const EXIT: u32 = 31;
    const TAKE: u32 = 32;
    /// A menu's top tile (`Menu` +4), and the type it put on the interface's menu stack, kept for its
    /// close to take off it (+0x14, set as it opens).
    const MENU_TILE: usize = 0x4;
    const MENU_STACKED: usize = 0x14;
    /// The book menu's values (xOBSE's `BookMenu`: user6, user8, user9): its page step (two a turn, a
    /// page each side), its page-turn sound, and its page (a scroll: where it's scrolled to by).
    const PAGE_STEP: u16 = 4020;
    const TURN_SOUND: u16 = 4022;
    const PAGE: u16 = 4023;
    /// `Tile::UpdateFloat` (thiscall: tile, value, number), and a menu sound by its number (cdecl; 6,
    /// "ITMBookPageTurn").
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const MENU_SOUND: usize = 0x0057_DE50;
    /// `InterfaceManager::GetSingleton` (cdecl `(0, 1)`), its menu stack (the open menus' types, up to
    /// ten, a 0 after the last) and its push (thiscall: the interface, the type).
    const INTERFACE: usize = 0x0058_2160;
    const MENU_STACK: usize = 0xE0;
    const PUSH_MENU: usize = 0x0057_D640;
    /// `TESForm` form id.
    const FORM_ID: usize = 0xC;

    /// The book menu, if it's open: (`BookMenu*`, the reference being read or 0).
    pub fn open() -> Option<(usize, usize)> {
        let m = tiles::menu(MENU_BOOK);
        (m != 0).then(|| (m, mem::deref(m + BOOK_REF)))
    }

    /// The book menu is the menu on top.
    pub fn reading() -> bool {
        tiles::menu_tile(MENU_BOOK) != 0 && tiles::top_menu() == MENU_BOOK
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
        (refr != 0 && HELD_OPEN.load(Ordering::Relaxed) != refr).then(|| crate::items::offered_object(refr)).flatten()
    }

    /// The reference of the book being read, once a hand has had it open (until its menu shuts).
    static HELD_OPEN: AtomicUsize = AtomicUsize::new(0);

    /// The book being read was in a hand and isn't now: let go of (or taken away), its menu is shutting
    /// (it fades out), and there's nothing of it to hold.
    pub fn let_go() -> bool {
        open().is_some_and(|(_, refr)| refr != 0 && HELD_OPEN.load(Ordering::Relaxed) == refr && held_book().is_none())
    }

    /// Shuts the book with the game's own Exit.
    fn shut(menu: usize) {
        click(menu, EXIT);
    }

    /// The book menu's own handling of one of its buttons (`Menu::HandleClick`): Exit, Take.
    fn click(menu: usize, id: u32) {
        let f = mem::deref(mem::deref(menu) + VT_HANDLE_CLICK);
        if !(0x0040_1000..0x00A0_0000).contains(&f) {
            return;
        }
        type Click = unsafe extern "thiscall" fn(usize, u32, usize);
        crate::hooks::guard_void("book button", || unsafe { std::mem::transmute::<usize, Click>(f)(menu, id, 0) });
    }

    fn update_float(tile: usize, id: u16, v: f32) {
        type Update = unsafe extern "thiscall" fn(usize, u32, f32);
        crate::hooks::guard_void("book value", || unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(tile, id as u32, v) });
    }

    /// Turns the book's page, onwards or back, as its own wheel does (0x595DA0): a book's page step by
    /// two, both its pages, with its page-turn sound if the page moved; a scroll scrolled a notch.
    fn turn(menu: usize, onwards: bool) {
        let tile = mem::deref(menu + MENU_TILE);
        if tile == 0 {
            return;
        }
        if is_scroll(menu) {
            update_float(tile, PAGE, if onwards { 1.0 } else { -1.0 });
            update_float(tile, PAGE, 0.0);
            return;
        }
        let before = tiles::value(tile, PAGE);
        update_float(tile, PAGE_STEP, if onwards { 2.0 } else { -2.0 });
        update_float(tile, PAGE_STEP, 0.0);
        if tiles::value(tile, PAGE) != before
            && let Some(sound) = tiles::value(tile, TURN_SOUND).filter(|s| *s >= 1.0)
        {
            crate::hooks::guard_void("book page sound", || unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(i32) -> i32>(MENU_SOUND)(sound as i32); });
        }
    }

    /// A button of the book's, by what it does.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Button {
        Take,
        Exit,
        Prev,
        Next,
    }

    /// The book's button under a point of the HUD texture (0..1 across and down), as the game laid it
    /// out (a scroll's arrows turn it as a book's Prev and Next do).
    fn button_at(uv: [f32; 2]) -> Option<Button> {
        let (w, h) = tiles::screen()?;
        let (x, y) = (uv[0] * w, uv[1] * h);
        let tree = tiles::Tree::read(tiles::menu_tile(MENU_BOOK));
        [
            ("book_take", Button::Take),
            ("book_scroll_take", Button::Take),
            ("book_exit", Button::Exit),
            ("book_scroll_exit", Button::Exit),
            ("book_prev", Button::Prev),
            ("book_scroll_up", Button::Prev),
            ("book_next", Button::Next),
            ("book_scroll_down", Button::Next),
        ]
        .into_iter()
        .find(|(name, _)| tree.find(name).is_some_and(|t| t.shown && x >= t.rect[0] && x <= t.rect[2] && y >= t.rect[1] && y <= t.rect[3]))
        .map(|(_, b)| b)
    }

    /// The interface's menu stack has a menu of this type.
    fn stacked(kind: u32) -> bool {
        type Interface = unsafe extern "C" fn(u8, u8) -> usize;
        let ui = unsafe { std::mem::transmute::<usize, Interface>(INTERFACE)(0, 1) };
        ui != 0 && (0..10).map(|k| mem::deref(ui + MENU_STACK + 4 * k) as u32).take_while(|t| *t != 0).any(|t| t == kind)
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
        /// The page turns' presses as last seen (onwards, back).
        pages: Option<(u16, u16)>,
        /// A fingertip pressing the book in play: the button it went down on.
        pressing: Option<Option<Button>>,
    }

    static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);

    /// Once a game frame, where the game's scripts run: a book just taken in a hand read, one let go of
    /// while it's open shut, one shut with B; its pages turned and its buttons pressed.
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
        if let Some((m, _)) = book {
            let play = !stacked(MENU_BOOK);
            if play {
                // (Off the stack: its close mustn't take another menu off it.)
                if mem::is_readable((m + MENU_STACKED) as *const u8, 4) {
                    unsafe { ((m + MENU_STACKED) as *mut u32).write(0) };
                }
            }
            let it = crate::link::last().map(|l| l.frame.intents);
            // Its pages turned by the hand (the runtime's page presses): the game's own step.
            let pages = it.map(|i| (i.presses[Action::PageNext as usize], i.presses[Action::PagePrev as usize]));
            if let (Some((n0, p0)), Some((n, p))) = (s.pages, pages) {
                if n != n0 {
                    turn(m, true);
                }
                if p != p0 {
                    turn(m, false);
                }
            }
            s.pages = pages;
            // A fingertip on its buttons, in play (in the menus the game's own cursor and click are
            // them): what it went down on, pressed when it comes off still on it.
            if let (true, Some(i)) = (play, it) {
                let on = i.flags & INT_CURSOR != 0;
                let down = on && i.flags & INT_CURSOR_DOWN != 0;
                match (s.pressing, down) {
                    (None, true) => s.pressing = Some(button_at(i.cursor_uv)),
                    (Some(was), false) => {
                        s.pressing = None;
                        let now = if on { button_at(i.cursor_uv) } else { None };
                        if let Some(b) = was.filter(|b| Some(*b) == now) {
                            tracing::info!("books: {b:?} pressed by a finger");
                            match b {
                                Button::Prev => turn(m, false),
                                Button::Next => turn(m, true),
                                // (Shut by it: what's left of this frame is for an open book.)
                                Button::Take => return click(m, TAKE),
                                Button::Exit => return click(m, EXIT),
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            s.pages = None;
            s.pressing = None;
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
                let ok = crate::world::activate(h.refr);
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

    static REAL_PUSH: AtomicUsize = AtomicUsize::new(0);

    /// The interface's push of a menu onto its stack: a book opened with no menu up (taken in the hand,
    /// or activated where it lies) isn't pushed, so the game stays in play under it. In VR only: on
    /// the flat screen a book is the game's menu as ever. Nor is the container menu a script opens
    /// for a look at what's in it at the hand ([`crate::quickloot::opening`]).
    unsafe extern "thiscall" fn push_menu(ui: usize, kind: u32) -> i32 {
        let real: unsafe extern "thiscall" fn(usize, u32) -> i32 = unsafe { std::mem::transmute(REAL_PUSH.load(Ordering::Acquire)) };
        if kind == crate::quickloot::MENU_CONTAINER && crate::quickloot::opening() {
            crate::quickloot::hide_opened();
            return 0;
        }
        if kind == MENU_BOOK && mem::deref(ui + MENU_STACK) == 0 && crate::link::last().is_some_and(|s| s.view.is_some()) {
            static SAID: std::sync::Once = std::sync::Once::new();
            SAID.call_once(|| tracing::info!("books: a book opened in play stays off the menu stack: the game plays on under it"));
            return 0;
        }
        unsafe { real(ui, kind) }
    }

    /// Once: books opened in play stay off the menu stack ([`push_menu`]).
    pub fn install() {
        if crate::hooks::install_into(PUSH_MENU as *const (), push_menu as *const (), "InterfaceManager's menu push", &REAL_PUSH) {
            tracing::info!("books: a book is read as the game plays on");
        }
    }
}
