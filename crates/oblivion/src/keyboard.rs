//! A keyboard in VR wherever the game wants typed text: the character's name in the race menu, a
//! custom class's name (the game's text box, `TextEditMenu`), and the name of a spell, an
//! enchantment, a potion or a sigil stone's enchantment. The player types on it with the laser, the
//! trigger clicking its keys as on any menu.
//!
//! * **When.** The menu on top has its text field taking keys: the game's own text field (the one
//!   its menus share) is taking keys while its first byte is set. The race menu's is at +0x8EC,
//!   spellmaking's +0x70, enchanting's +0x98, alchemy's +0xA0, the sigil stone's +0x74 (pointers);
//!   the text box has its own at +0x34, taking keys while it's open. A field takes keys once it's
//!   clicked (the race menu's name, a spell's name), and stops at Enter or a click elsewhere: the
//!   game's own rules.
//! * **The keys are the game's own UI.** They're tiles read by the game's own `Tile::ReadXML`
//!   (0x590420) from a file of ours (written beside the runtime's settings at start) into the
//!   menu's own tiles, so the game draws them, lights the one under the cursor and clicks them as
//!   its own. Each is the game's short button (`shared_button_short_off.dds`, `_on` under the
//!   cursor, as `button_short.xml` has them) at a key's size, its label in the game's font and
//!   brown ink: what the game types for that key (its own scan code to character, 0x403CF0, for
//!   the game's language, shifted or not), or the game's own name of the key (the controls menu's
//!   names by scan code, 0xB39578: Backspace, L-Shift, Space, Enter).
//! * **A key typed is a key pressed.** The menu's click handler sees a key's id first (hooked:
//!   the race menu's through [`crate::facesculpt`]'s hook, the others here) and the key is pressed
//!   and let go on the game's keyboard ([`crate::input::type_key`]), the left Shift held round it
//!   for a capital. The game's interface reads its keyboard as ever (0x5821F0: a key down a frame),
//!   turns the key into a character by its own table (0x57F7A0: Shift, Backspace, Enter) and gives
//!   it to the menu on top (`Menu::HandleKeyboardInput`, its vtable's 12th), whose field takes it:
//!   the game's own typing, its own limits (what fits the field), its own Enter.
//! * **Shift**: once for the next character, again to lock capitals (letters), again off; lit
//!   while it's on. The keyboard comes up shifted when the field is empty (a name's capital).
//! * **Where.** In the text box the keys lie on its own parchment, under its text and over its
//!   Back and OK (the parchment made taller, its `user1`), so the box is one piece as ever. Over a
//!   menu's own field the keys are a block of their own beside the menu on the game's screen (where
//!   there's room: right of it, under it, over it, left of it), left out of the menu's tiles as
//!   the adapter reads them ([`crate::tiles`]), and they float in VR as a pop-up where the eyes
//!   look, the menu's field over them, so what's typed shows over the keys ([`pieces`], from
//!   [`crate::hud`]).

/// The keys' block over a menu's own field (it floats apart from the menu), and the keys inside the
/// text box (part of it): their top tiles' names.
pub const APART: &str = "rustvr_keyboard";
pub const INSIDE: &str = "rustvr_keys";

/// A field is taking keys in VR, the keyboard up for it (updated once a game frame).
static TYPING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A field is taking keys in VR: the game's keyboard is the VR keyboard's alone then.
pub fn typing() -> bool {
    TYPING.load(std::sync::atomic::Ordering::Relaxed)
}

/// A key's id is this plus its scan code (DirectInput's): clear of every id of the menus it's in
/// (theirs are under 300, their lists' from 1000).
pub const KEY_ID: i32 = 700;

/// A key's size on the game's screen (the menus' units; a wider key is more columns) and the room
/// between keys.
const KEY_W: f32 = 48.0;
const KEY_H: f32 = 44.0;
const KEY_GAP: f32 = 4.0;
const ROW_GAP: f32 = 6.0;
const COLUMNS: f32 = 10.0;
/// The keys' block: ten columns, five rows.
pub const SIZE: (f32, f32) = (COLUMNS * (KEY_W + KEY_GAP) - KEY_GAP, 5.0 * (KEY_H + ROW_GAP) - ROW_GAP);

/// The short button's picture is its texture's top left 101 x 37 of 128 x 64 (the rest its shadow
/// and nothing): stretched by this much more, the button is the key's size.
const ART: (f32, f32) = (128.0 / 101.0, 64.0 / 37.0);

/// Scan codes (DirectInput's) of the keys that aren't characters.
pub const BACKSPACE: u8 = 0x0E;
pub const ENTER: u8 = 0x1C;
pub const SHIFT: u8 = crate::input::DIK_LSHIFT;
pub const SPACE: u8 = 0x39;

/// The rows: each key's scan code and how many columns wide it is. The digits; Q to P; A to L and
/// the apostrophe; Shift, Z to M and the hyphen (what names have); then Backspace, Space and Enter.
/// In the text box its own OK confirms, so its last row has no Enter.
const ROWS: [&[(u8, u8)]; 4] = [
    &[(0x02, 1), (0x03, 1), (0x04, 1), (0x05, 1), (0x06, 1), (0x07, 1), (0x08, 1), (0x09, 1), (0x0A, 1), (0x0B, 1)],
    &[(0x10, 1), (0x11, 1), (0x12, 1), (0x13, 1), (0x14, 1), (0x15, 1), (0x16, 1), (0x17, 1), (0x18, 1), (0x19, 1)],
    &[(0x1E, 1), (0x1F, 1), (0x20, 1), (0x21, 1), (0x22, 1), (0x23, 1), (0x24, 1), (0x25, 1), (0x26, 1), (0x28, 1)],
    &[(SHIFT, 2), (0x2C, 1), (0x2D, 1), (0x2E, 1), (0x2F, 1), (0x30, 1), (0x31, 1), (0x32, 1), (0x0C, 1)],
];
const LAST_ROW_APART: &[(u8, u8)] = &[(BACKSPACE, 3), (SPACE, 4), (ENTER, 3)];
const LAST_ROW_INSIDE: &[(u8, u8)] = &[(BACKSPACE, 4), (SPACE, 6)];

/// The game's brown ink.
const INK: &str = "<red> 117 </red> <green> 59 </green> <blue> 33 </blue> <alpha> 255 </alpha>";

/// The keys' file: `apart` the block over a menu's field (above the menu's own tiles, hidden until
/// it's placed), else the keys inside the text box.
pub fn xml(apart: bool) -> String {
    let (w, h) = SIZE;
    let (name, depth) = if apart { (APART, 20) } else { (INSIDE, 3) };
    let mut x = format!("<rect name=\"{name}\">\n<locus> &true; </locus> <depth> {depth} </depth> <visible> &false; </visible>\n<x> 0 </x> <y> 0 </y> <width> {w} </width> <height> {h} </height>\n");
    let last = if apart { LAST_ROW_APART } else { LAST_ROW_INSIDE };
    for (row, keys) in ROWS.iter().copied().chain(std::iter::once(last)).enumerate() {
        let mut column = 0u8;
        for &(code, wide) in keys {
            let kx = column as f32 * (KEY_W + KEY_GAP);
            let ky = row as f32 * (KEY_H + ROW_GAP);
            x.push_str(&key(code, kx, ky, wide as f32 * (KEY_W + KEY_GAP) - KEY_GAP, KEY_H));
            column += wide;
        }
    }
    x.push_str("</rect>\n");
    x
}

/// A key: a rect the cursor clicks (its id the scan code's), the button's picture over it (lit
/// under the cursor, or while `user10`: Shift on), and its label (`user0`).
fn key(code: u8, x: f32, y: f32, w: f32, h: f32) -> String {
    let id = KEY_ID + code as i32;
    let (aw, ah) = (w * ART.0, h * ART.1);
    let name = format!("rvkb_key_{code:02x}");
    format!(
        "<rect name=\"{name}\">\n<id> {id} </id> <target> &true; </target> <clicksound> 1 </clicksound> <locus> &true; </locus> <depth> 1 </depth>\n\
         <x> {x} </x> <y> {y} </y> <width> {w} </width> <height> {h} </height>\n\
         <user0> </user0> <user10> &false; </user10>\n\
         <image name=\"{name}_off\"> <filename> Menus\\Shared\\shared_button_short_off.dds </filename> <zoom> -1 </zoom> <depth> 1 </depth> <width> {aw} </width> <height> {ah} </height> <x> 0 </x> <y> 0 </y> </image>\n\
         <image name=\"{name}_on\"> <visible> <copy src=\"parent()\" trait=\"mouseover\"/> <eq> 1 </eq> <or src=\"parent()\" trait=\"user10\"/> </visible> <filename> Menus\\Shared\\shared_button_short_on.dds </filename> <zoom> -1 </zoom> <depth> 2 </depth> <width> {aw} </width> <height> {ah} </height> <x> 0 </x> <y> 0 </y> </image>\n\
         <text name=\"{name}_text\"> <depth> 3 </depth> <font> 1 </font> <justify> &center; </justify> {INK}\n\
         <string> <copy src=\"parent()\" trait=\"user0\"/> </string> <x> {} </x>\n\
         <y> <copy src=\"parent()\" trait=\"height\"/> <sub src=\"me()\" trait=\"height\"/> <div> 2 </div> </y> </text>\n\
         </rect>\n",
        w * 0.5
    )
}

/// What Shift does to the next keys: nothing, the next character, or every letter (capitals).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shift {
    #[default]
    Off,
    Once,
    Locked,
}

impl Shift {
    /// Shift clicked: once, then locked, then off.
    pub fn next(self) -> Shift {
        match self {
            Shift::Off => Shift::Once,
            Shift::Once => Shift::Locked,
            Shift::Locked => Shift::Off,
        }
    }

    /// Whether a key is typed with Shift held (`letter`: it types a letter).
    pub fn held(self, letter: bool) -> bool {
        match self {
            Shift::Off => false,
            Shift::Once => true,
            Shift::Locked => letter,
        }
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{hooks, mem, tiles};
    use rustvr_protocol::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// `Tile::ReadXML(parent, path)` (thiscall): the file's tiles made under `parent`.
    const READ_XML: usize = 0x0059_0420;
    /// The game's scan code to character (stdcall `(code, shifted)`), in its language's layout.
    const TO_CHARACTER: usize = 0x0040_3CF0;
    /// The controls menu's names of the keyboard's keys: a setting for each scan code (its value,
    /// the name, at +0).
    const KEY_NAMES: usize = 0x00B3_9578;
    /// `Tile::UpdateFloat`, `Tile::UpdateString` (thiscall: tile, id, value).
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const UPDATE_STRING: usize = 0x0058_CED0;
    /// Tile values: id, user0 (a key's label), user1 (the text box's parchment's height), user10
    /// (a key lit).
    const ID: u16 = 0xFA8;
    const USER0: u32 = 0xFAE;
    const USER1: u16 = 0xFAF;
    const USER10: u32 = 0xFB8;
    const FALSE: f32 = 1.0;

    /// Where on a menu its field is shown, as a strip of it floating over the keys.
    #[derive(Clone, Copy)]
    enum Strip {
        /// The tile of that name (the field's own box).
        Box(&'static str),
        /// The race menu's name: its parchment's top, down to the line under the name.
        RaceName,
        /// None: the keys are inside the menu (the text box).
        Inside,
    }

    /// A menu with a text field: its type, where its field is (`inline`: in the menu itself, else a
    /// pointer to it there), its strip, and what it's for (for the log).
    struct Field {
        menu: u32,
        at: usize,
        inline: bool,
        strip: Strip,
        what: &'static str,
    }

    const FIELDS: [Field; 6] = [
        Field { menu: crate::menus::MENU_RACE_SEX, at: 0x8EC, inline: false, strip: Strip::RaceName, what: "the character's name (the race menu)" },
        Field { menu: 0x41B, at: 0x34, inline: true, strip: Strip::Inside, what: "the game's text box" },
        Field { menu: 0x411, at: 0x70, inline: false, strip: Strip::Box("spell_name_background"), what: "the spell's name (spellmaking)" },
        Field { menu: 0x412, at: 0x98, inline: false, strip: Strip::Box("ench_name_background"), what: "the enchantment's name (enchanting)" },
        Field { menu: 0x410, at: 0xA0, inline: false, strip: Strip::Box("name_background"), what: "the potion's name (alchemy)" },
        Field { menu: 0x418, at: 0x74, inline: false, strip: Strip::Box("sigil_name_background"), what: "the sigil stone's name" },
    ];

    /// The text box's parchment and its line of text (`TextEditMenu.xml`).
    const TEXT_BOX: &str = "textedit_background";
    const TEXT_LINE: &str = "textedit_text";
    /// A line of the game's text, and the room under the keys for the text box's Back and OK
    /// (their 44, and 16 over and under them).
    const LINE: f32 = 36.0;
    const UNDER_KEYS: f32 = 76.0;
    /// The keys' block over a menu's field: this far from the menu on the game's screen, and from
    /// the screen's edges.
    const ROOM: f32 = 24.0;
    const EDGE: f32 = 8.0;

    /// The keyboard in VR over a menu's field: how much bigger it floats than the game's screen has
    /// it (its keys about 4 cm across where the eyes look), a margin round its pieces, the room
    /// between the field and the keys (the group's units), and its group's name.
    const SCALE: f32 = 1.4;
    const MARGIN: f32 = 3.0;
    const FIELD_GAP: f32 = 0.012;
    const GROUP: u32 = 0x4B42_0000;

    /// The keys' ids clicked, waiting for the game's frame.
    static CLICKS: Mutex<Vec<i32>> = Mutex::new(Vec::new());
    /// The keys' files.
    static PATHS: std::sync::OnceLock<[std::ffi::CString; 2]> = std::sync::OnceLock::new();

    #[derive(Default)]
    struct State {
        /// The menu the keys are in (its type, its `Menu*`), the keys' top tile and the tile it's in.
        kind: u32,
        menu: usize,
        tile: usize,
        parent: usize,
        /// The keys float apart (over a menu's field), not inside the menu.
        apart: bool,
        /// Each key's tile by its scan code.
        keys: Vec<(u8, usize)>,
        /// The field took keys last frame; the keys show.
        typing: bool,
        shown: bool,
        shift: Shift,
        /// What each key's tile was last given: its label, lit; and the Shift they're labelled for.
        labels: HashMap<usize, Vec<u8>>,
        lit: Option<bool>,
        labelled: Option<Shift>,
        /// Where the keys were put last (in the tile they're in).
        at: Option<(f32, f32)>,
        /// The game made no keys in this menu (tried once).
        failed: bool,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    /// A key's id clicked in a menu (its click handler, before the menu's own): taken, the menu
    /// never sees it (a click would end its field's typing). False for the menu's own ids.
    pub fn click(id: i32) -> bool {
        if !(KEY_ID..KEY_ID + 256).contains(&id) {
            return false;
        }
        hooks::guard_void("keyboard click", || CLICKS.lock().unwrap().push(id));
        true
    }

    type Click = unsafe extern "thiscall" fn(usize, i32, usize);

    /// A menu's click handler, the keys' ids taken off first.
    macro_rules! keys_first {
        ($hook:ident, $real:ident) => {
            static $real: AtomicUsize = AtomicUsize::new(0);
            unsafe extern "thiscall" fn $hook(menu: usize, id: i32, tile: usize) {
                if click(id) {
                    return;
                }
                let real: Click = unsafe { std::mem::transmute($real.load(Ordering::Acquire)) };
                unsafe { real(menu, id, tile) }
            }
        };
    }
    keys_first!(text_box_click, REAL_TEXT_BOX);
    keys_first!(alchemy_click, REAL_ALCHEMY);
    keys_first!(enchanting_click, REAL_ENCHANTING);
    keys_first!(sigil_click, REAL_SIGIL);
    keys_first!(spellmaking_click, REAL_SPELLMAKING);

    /// The keys' files written (beside the runtime's settings) and the menus' click handlers hooked
    /// (the race menu's is [`crate::facesculpt`]'s).
    pub fn install() {
        let dir = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("RustVR");
        let files = [("oblivion_keyboard.xml", xml(true)), ("oblivion_keys.xml", xml(false))];
        let mut paths = Vec::new();
        for (file, text) in files {
            let path = dir.join(file);
            if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, text)) {
                tracing::warn!("keyboard: can't write its keys {}: {e}", path.display());
                return;
            }
            paths.push(std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default());
        }
        let [a, b]: [std::ffi::CString; 2] = paths.try_into().unwrap_or_default();
        let _ = PATHS.set([a, b]);
        let handlers: [(usize, *const (), &AtomicUsize, &str); 5] = [
            (0x005D_D1B0, text_box_click as *const (), &REAL_TEXT_BOX, "TextEditMenu::HandleClick"),
            (0x0059_5080, alchemy_click as *const (), &REAL_ALCHEMY, "AlchemyMenu::HandleClick"),
            (0x005A_3160, enchanting_click as *const (), &REAL_ENCHANTING, "EnchantmentMenu::HandleClick"),
            (0x005D_5470, sigil_click as *const (), &REAL_SIGIL, "SigilStoneMenu::HandleClick"),
            (0x005D_7CA0, spellmaking_click as *const (), &REAL_SPELLMAKING, "SpellMakingMenu::HandleClick"),
        ];
        for (at, hook, real, name) in handlers {
            hooks::install_into(at as *const (), hook, name, real);
        }
    }

    fn update_float(tile: usize, id: u32, v: f32) {
        type F = unsafe extern "thiscall" fn(usize, u32, f32);
        if tile != 0 {
            hooks::guard_void("keyboard tile", || unsafe { std::mem::transmute::<usize, F>(UPDATE_FLOAT)(tile, id, v) });
        }
    }

    fn update_bytes(tile: usize, id: u32, text: &[u8]) {
        type F = unsafe extern "thiscall" fn(usize, u32, *const u8);
        let mut c: Vec<u8> = text.iter().copied().filter(|&b| b != 0).collect();
        c.push(0);
        if tile != 0 {
            hooks::guard_void("keyboard label", || unsafe { std::mem::transmute::<usize, F>(UPDATE_STRING)(tile, id, c.as_ptr()) });
        }
    }

    /// The menu's text field.
    fn field_of(menu: usize, f: &Field) -> usize {
        if menu == 0 {
            0
        } else if f.inline {
            menu + f.at
        } else {
            mem::deref(menu + f.at)
        }
    }

    /// The field is taking keys.
    fn taking(field: usize) -> bool {
        field != 0 && mem::at::<u8>(field).is_some_and(|b| b != 0)
    }

    /// The field has nothing in it (its text, a `BSStringT` at +0x18).
    fn empty(field: usize) -> bool {
        let text = mem::deref(field + 0x18);
        text == 0 || mem::at::<u8>(text).is_none_or(|b| b == 0)
    }

    /// What the game types for a key (0: nothing), shifted or not.
    fn character(code: u8, shifted: bool) -> u8 {
        type F = unsafe extern "stdcall" fn(u32, u32) -> i32;
        let c = hooks::guard("keyboard character", -1, || unsafe { std::mem::transmute::<usize, F>(TO_CHARACTER)(code as u32, shifted as u32) });
        // (The printable ones: not Backspace, Enter, Escape, nor the bar the game types none of.)
        if (32..=255).contains(&c) && c != 127 && c != b'|' as i32 { c as u8 } else { 0 }
    }

    /// It types a letter (Shift's lock makes it a capital).
    fn letter(code: u8) -> bool {
        (char::from(character(code, false))).is_alphabetic()
    }

    /// The game's own name of a key (the controls menu's).
    fn key_name(code: u8) -> Vec<u8> {
        let name = mem::deref(mem::deref(KEY_NAMES + 4 * code as usize));
        let mut out = Vec::new();
        for i in 0..24 {
            match (name != 0).then(|| mem::at::<u8>(name + i)).flatten() {
                Some(0) | None => break,
                Some(b) => out.push(b),
            }
        }
        if out.is_empty() {
            out = match code {
                BACKSPACE => b"Backspace".to_vec(),
                ENTER => b"Enter".to_vec(),
                SHIFT => b"Shift".to_vec(),
                _ => b"Space".to_vec(),
            };
        }
        out
    }

    /// A key's label now: its name, or the character it types with Shift as it is.
    fn label(code: u8, shift: Shift) -> Vec<u8> {
        match code {
            BACKSPACE | ENTER | SHIFT | SPACE => key_name(code),
            _ => match character(code, shift.held(letter(code))) {
                0 => Vec::new(),
                c => vec![c],
            },
        }
    }

    /// The game's text (Windows' Western code page) for the log.
    fn shown(bytes: &[u8]) -> String {
        bytes.iter().map(|&b| char::from(b)).collect()
    }

    /// The keys' tiles are still where they were made (the menu hasn't closed, nor made its tiles
    /// again).
    fn alive(s: &State) -> bool {
        s.tile != 0 && s.menu != 0 && tiles::menu(s.kind) == s.menu && tiles::children(s.parent).contains(&s.tile)
    }

    fn show(s: &mut State, on: bool) {
        if s.shown != on && alive(s) {
            update_float(s.tile, tiles::VISIBLE as u32, if on { tiles::TRUE } else { FALSE });
        }
        s.shown = on && s.tile != 0;
    }

    /// The keys made in the menu: inside the text box's parchment, else under the menu's top tile.
    fn make(s: &mut State, menu_tile: usize) -> bool {
        let Some(paths) = PATHS.get() else { return false };
        let parent = if s.apart {
            menu_tile
        } else {
            match tiles::Tree::read(menu_tile).find(TEXT_BOX) {
                Some(p) => p.tile,
                None => return false,
            }
        };
        let path = &paths[if s.apart { 0 } else { 1 }];
        type Read = unsafe extern "thiscall" fn(usize, *const u8) -> usize;
        let tile = hooks::guard("keyboard keys", 0, || unsafe { std::mem::transmute::<usize, Read>(READ_XML)(parent, path.as_ptr() as *const u8) });
        if tile == 0 {
            tracing::warn!("keyboard: the game made no keys from {path:?}");
            s.failed = true;
            return false;
        }
        s.tile = tile;
        s.parent = parent;
        s.keys = tiles::children(tile).into_iter().filter_map(|t| tiles::value(t, ID).map(|id| ((id as i32 - KEY_ID) as u8, t))).collect();
        s.labels.clear();
        s.lit = None;
        s.labelled = None;
        s.at = None;
        s.shown = false;
        tracing::info!("keyboard: {} keys made of the game's buttons ({})", s.keys.len(), if s.apart { "over the menu, apart" } else { "on the text box's parchment" });
        true
    }

    /// Where the keys' block goes on the game's screen, clear of the menu (`menu`, the box it draws)
    /// and near its field: right of it, under it, over it or left of it, whichever has room.
    fn spot(menu: [f32; 4], field: [f32; 4], screen: (f32, f32)) -> Option<((f32, f32), &'static str)> {
        let ((w, h), (kw, kh)) = (screen, SIZE);
        let along_y = field[1].clamp(EDGE, (h - kh - EDGE).max(EDGE));
        let along_x = ((field[0] + field[2] - kw) * 0.5).clamp(EDGE, (w - kw - EDGE).max(EDGE));
        let right = menu[2] + ROOM;
        let under = menu[3] + ROOM;
        let over = menu[1] - ROOM - kh;
        let left = menu[0] - ROOM - kw;
        if right + kw <= w - EDGE {
            Some(((right, along_y), "right of the menu"))
        } else if under + kh <= h - EDGE {
            Some(((along_x, under), "under the menu"))
        } else if over >= EDGE {
            Some(((along_x, over), "over the menu"))
        } else if left >= EDGE {
            Some(((left, along_y), "left of the menu"))
        } else {
            None
        }
    }

    /// The keys put where they go (only when that's changed), and the text box made tall enough.
    fn place(s: &mut State, f: &Field, menu_tile: usize) {
        let tree = tiles::Tree::read(menu_tile);
        let at = if s.apart {
            let Some(screen) = tiles::screen() else { return };
            let Some(top) = tree.0.first() else { return };
            let Some(menu) = tree.drawn(&top.name) else { return };
            let field = strip(&tree, f.strip).unwrap_or(menu);
            let ((x, y), how) = spot(menu, field, screen).unwrap_or(((EDGE, screen.1 - SIZE.1 - EDGE), "at the screen's foot (no room clear of the menu)"));
            if s.at.is_none() {
                tracing::info!("keyboard: on the game's screen at ({x:.0}, {y:.0}), {how} (the menu takes {menu:.0?} of {screen:.0?})");
            }
            // (Its x and y are from the menu's top tile.)
            (x - top.rect[0], y - top.rect[1])
        } else {
            let (Some(bg), Some(text)) = (tree.find(TEXT_BOX), tree.find(TEXT_LINE)) else { return };
            let (x, y) = (((bg.rect[2] - bg.rect[0]) - SIZE.0) * 0.5, text.rect[1] - bg.rect[1] + LINE + 12.0);
            let tall = y + SIZE.1 + UNDER_KEYS;
            if tiles::value(bg.tile, USER1).is_some_and(|now| now < tall - 0.5) {
                update_float(bg.tile, USER1 as u32, tall);
                tracing::info!("keyboard: the text box's parchment made {tall:.0} tall for its keys ({:.0} wide as ever)", bg.rect[2] - bg.rect[0]);
            }
            (x.max(0.0), y)
        };
        if s.at.is_none_or(|(x, y)| (x - at.0).abs() > 0.5 || (y - at.1).abs() > 0.5) {
            update_float(s.tile, tiles::X as u32, at.0);
            update_float(s.tile, tiles::Y as u32, at.1);
            s.at = Some(at);
        }
    }

    /// The menu's field as it shows on the game's screen (the strip that floats over the keys).
    fn strip(tree: &tiles::Tree, strip: Strip) -> Option<[f32; 4]> {
        match strip {
            Strip::Box(name) => tree.rect(name),
            Strip::RaceName => {
                let (bg, line) = (tree.rect("race_background")?, tree.rect("race_name_separator")?);
                Some([bg[0], bg[1], bg[2], line[3] + 6.0])
            }
            Strip::Inside => None,
        }
    }

    /// The keys' labels as Shift has them, and Shift lit while it's on.
    fn label_keys(s: &mut State) {
        if s.labelled != Some(s.shift) {
            for &(code, tile) in &s.keys {
                let want = label(code, s.shift);
                if s.labels.get(&tile) != Some(&want) {
                    update_bytes(tile, USER0, &want);
                    s.labels.insert(tile, want);
                }
            }
            s.labelled = Some(s.shift);
        }
        let lit = s.shift != Shift::Off;
        if s.lit != Some(lit)
            && let Some(&(_, tile)) = s.keys.iter().find(|(c, _)| *c == SHIFT)
        {
            update_float(tile, USER10, if lit { tiles::TRUE } else { FALSE });
            s.lit = Some(lit);
        }
    }

    /// A key clicked: pressed on the game's keyboard (Shift: its state).
    fn press(s: &mut State, code: u8) {
        if code == SHIFT {
            s.shift = s.shift.next();
            tracing::info!("keyboard: Shift {}", match s.shift {
                Shift::Off => "off",
                Shift::Once => "on for the next character",
                Shift::Locked => "locked (capitals)",
            });
            return;
        }
        let character = !matches!(code, BACKSPACE | ENTER);
        let shifted = character && s.shift.held(letter(code));
        crate::input::type_key(code, shifted);
        match code {
            BACKSPACE | ENTER => tracing::info!("keyboard: {} pressed", shown(&key_name(code))),
            _ => tracing::info!("keyboard: typed '{}' (key {code:#04x}{})", shown(&label(code, s.shift)), if shifted { ", shifted" } else { "" }),
        }
        if character && s.shift == Shift::Once {
            s.shift = Shift::Off;
        }
    }

    /// Once a game frame: the keyboard up while the menu on top has its field taking keys, its keys
    /// clicked typed, put away when the field's done.
    pub fn frame() {
        let clicks: Vec<i32> = std::mem::take(&mut *CLICKS.lock().unwrap());
        let mut guard = STATE.lock().unwrap();
        let s = guard.get_or_insert_with(State::default);
        if s.tile != 0 && !alive(s) {
            // (Gone with its menu, or with the tiles the menu made again.)
            s.tile = 0;
            s.keys.clear();
            s.shown = false;
        }
        // (In VR, or a test's emulated headset: a flat game has its own keyboard.)
        let vr = crate::link::last().is_some_and(|snap| snap.view.is_some()) || crate::settings::get().background;
        let top = if vr && crate::menus::menu_mode() { tiles::top_menu() } else { 0 };
        let field = FIELDS.iter().find(|f| f.menu == top);
        let menu = field.map_or(0, |f| tiles::menu(f.menu));
        let typing = field.is_some_and(|f| taking(field_of(menu, f)));
        super::TYPING.store(typing, Ordering::Relaxed);
        let Some(f) = field.filter(|_| typing) else {
            // Not typing: the keys hidden in whichever menu they're in.
            if s.typing {
                let what = FIELDS.iter().find(|f| f.menu == s.kind).map_or("", |f| f.what);
                tracing::info!("keyboard: put away ({what} is done, or another menu is over it)");
            }
            show(s, false);
            s.typing = false;
            return;
        };
        if (s.kind, s.menu) != (f.menu, menu) {
            show(s, false);
            *s = State { kind: f.menu, menu, apart: !matches!(f.strip, Strip::Inside), ..Default::default() };
        }
        let menu_tile = tiles::menu_tile(f.menu);
        if !s.typing {
            s.shift = if empty(field_of(menu, f)) { Shift::Once } else { Shift::Off };
            s.at = None;
            tracing::info!("keyboard: up for {} ({})", f.what, if s.shift == Shift::Once { "shifted for a capital" } else { "the field has text" });
        }
        s.typing = true;
        if s.tile == 0 && (s.failed || !make(s, menu_tile)) {
            return;
        }
        place(s, f, menu_tile);
        show(s, true);
        for id in clicks {
            press(s, (id - KEY_ID) as u8);
        }
        label_keys(s);
    }

    /// The keyboard in VR over a menu's field: a pop-up where the eyes look, its keys with the
    /// field over them (the keys inside the text box float with it as it does).
    pub fn pieces(l: &mut HudLayout, screen: (f32, f32)) {
        let (kind, tile, field) = {
            let guard = STATE.lock().unwrap();
            let Some(s) = guard.as_ref().filter(|s| s.typing && s.apart && s.shown) else { return };
            let Some(f) = FIELDS.iter().find(|f| f.menu == s.kind) else { return };
            (s.kind, s.tile, f.strip)
        };
        let keys = tiles::Tree::read(tile);
        let Some(block) = keys.0.first().filter(|p| p.shown).map(|p| p.rect) else { return };
        let menu = tiles::Tree::read(tiles::menu_tile(kind));
        let field = strip(&menu, field);
        let h = screen.1.max(1.0);
        let n = l.add_group(HudGroup { anchor: ANCHOR_POPUP, id: GROUP ^ kind, ..Default::default() });
        if n == 0 {
            return;
        }
        // (Pages, as the menus' pieces are: solid where the game drew them. The keys are pointed
        // at; the field over them shows what's typed.)
        let flags = HUDE_SOFT | HUDE_SOLID;
        if let Some(uv) = tiles::uv_of(block, screen, MARGIN) {
            l.push(HudElement { scale: SCALE, ..HudElement::piece(n, uv, [0.0; 3], flags | HUDE_POINT) });
        }
        if let Some(b) = field
            && let Some(uv) = tiles::uv_of(b, screen, MARGIN)
        {
            let up = ((block[3] - block[1]) * 0.5 + (b[3] - b[1]) * 0.5 + 2.0 * MARGIN) / h * SCALE + FIELD_GAP;
            l.push(HudElement { scale: SCALE, ..HudElement::piece(n, uv, [0.0, up, 0.0], flags) });
        }
    }
}
