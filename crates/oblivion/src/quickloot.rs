//! Looting at a glance: what's in a chest or a body a free hand points at ([`crate::pointing`]) is
//! listed over it, the game playing on, and taken from there: the turning stick's up and down move
//! through it, the trigger takes the thing picked, that hand's A takes everything
//! ([`rustvr_protocol::GS_QUICK_LOOT`]). The list stays on its thing while the aim slips off it a
//! moment (past a body's arm), and goes over to another only once the hand's on that.
//!
//! * **What's looted:** a container the game would open (not locked, the lock is the game's to
//!   work), and a dead person or creature. One whose script has its own activation (an
//!   `OnActivate` block, which the game runs instead of opening it: Captain Renault's body moves
//!   the main quest on as it's searched) is activated by the player as ever, its script run: if the
//!   script opens it, the container menu it opens stays off the menu stack (the game plays on,
//!   [`opening`]) and is shut with its own Exit, and what's in it is listed; if the script doesn't,
//!   nothing is. Once a look: again only when the hand comes back to it.
//! * **What's listed** is the container menu's own list: its entries of the thing's things
//!   (`GetInventoryItem`, 0x4D88F0) and its own filter (0x4854F0, as it lists a container's or a
//!   body's things, worn ones too), each by the game's name, value, weight and inventory icon.
//! * **Taking** is the container menu's take: the stealing it decides as it opens (the thing has an
//!   owner the player isn't, and isn't a body), the whole stack by its `RemoveItem` into the
//!   player's things (marked stolen while stealing), its own item sound; what was stolen added up,
//!   and when the hand moves on, its own check for whoever saw the theft (the player's vtable
//!   +0x238 with the value and the owner), as closing its menu does.
//! * **The list is the game's UI, as its HUD has it**: over the game's black, see-through, the
//!   thing's name under a gilt rule (the magic menu's), the rows (an item's inventory icon, its name
//!   and count as the game's lists write them, its value and weight under its small icons, in the
//!   HUD's shadowed font and colour), the picked one lit by the HUD's own parchment between two
//!   rules, and under them the button art and words for Take All and Take (`strings()`); as tall as
//!   its rows. Read by the game's own reader into its HUD (at the top right of its screen, clear of
//!   its own pieces: the notices are at the top left), drawn by the game, and the runtime floats
//!   that piece of the HUD at the thing ([`rustvr_protocol::HUD_LOOT`]), a little smaller than the
//!   menus' text.

/// The list's top tile's name (in the HUD).
pub const LOOT: &str = "rustvr_loot";

/// The container menu (`kMenuType_Container`).
pub const MENU_CONTAINER: u32 = 0x3F0;

/// Rows shown at once, and the list's sizes (the menus' units): its width, a row's height, its
/// edge; the thing's name over the rows, the rule under it and the small icons over the columns;
/// the buttons under the rows (the Xbox's button art, 64 square).
const ROWS: usize = 6;
const WIDTH: f32 = 500.0;
const ROW: f32 = 44.0;
const PAD: f32 = 14.0;
const TITLE: f32 = 44.0;
const RULE: f32 = 8.0;
const HEADER: f32 = 36.0;
const FOOTER: f32 = 72.0;
/// Where it is on the game's screen: in from its right edge, down from its top.
const AT: (f32, f32) = (16.0, 40.0);
/// Its rows' ids, and the focus box's.
const ROW_ID: i32 = 980;
const FOCUS_ID: i32 = 979;
/// The middles of the value's and the weight's columns; where a row's name starts (past its icon).
const VALUE_X: f32 = 376.0;
const WEIGHT_X: f32 = 452.0;
const NAME_X: f32 = 50.0;
const ICON: f32 = 36.0;

/// The HUD's own text: its shadowed font, in its colour (its card's of what the crosshair's on).
const FONT: u32 = 2;
const INK: &str = "<red> 254 </red> <green> 244 </green> <blue> 183 </blue> <alpha> 255 </alpha>";
/// How dark the game's black is under it (of 255: the world shows through), how bright the HUD's
/// parchment is under the picked row.
const SHADE: u32 = 150;
const LIT: u32 = 60;
/// The game's thin gilt rule (`shared_border_horizontal_1`, 636 long as drawn), as big across the
/// list (and across a row), and as tall then.
const RULE_ZOOM: u32 = 76;
const RULE_ROW_ZOOM: u32 = 70;

/// The rows' top, on the list.
fn rows_top() -> f32 {
    TITLE + RULE + HEADER
}

/// The list's file: hidden until there's something to show; the thing's name its `user0`, how many
/// rows show its `user1` (the list as tall as them), its place across the screen set once it's made
/// (the screen's width is the HUD's). The game's black, see-through, under the thing's name and its
/// gilt rule, the small icons over the value and weight columns, the rows, a rule, and the buttons.
pub fn xml() -> String {
    let w = WIDTH;
    let inner = w - 2.0 * PAD;
    let top = rows_top();
    // (Down past the rows shown.)
    let below = |add: f32| format!("<copy src=\"parent()\" trait=\"user1\"/> <mul> {ROW} </mul> <add> {} </add>", top + add);
    let rule = |name: &str, zoom: u32, x: f32, y: &str, width: f32| {
        format!(
            "<image name=\"{name}\"> <filename> Menus\\Shared\\shared_border_horizontal_1.dds </filename> <zoom> {zoom} </zoom> <depth> 2 </depth> <width> {width} </width> <height> {} </height> <x> {x} </x> <y> {y} </y> </image>\n",
            (8 * zoom).div_ceil(100)
        )
    };
    let mut x = format!(
        "<rect name=\"{LOOT}\">\n<locus> &true; </locus> <depth> 30 </depth> <visible> &false; </visible> <user0> </user0> <user1> {ROWS} </user1>\n\
         <x> {} </x> <y> {} </y> <width> {w} </width> <height> <copy src=\"me()\" trait=\"user1\"/> <mul> {ROW} </mul> <add> {} </add> </height>\n\
         <image name=\"rvloot_background\"> <filename> Menus\\Faders\\black.dds </filename> <zoom> -1 </zoom> <alpha> {SHADE} </alpha> <depth> 1 </depth> <x> 0 </x> <y> 0 </y> <width> {w} </width> <height> <copy src=\"parent()\" trait=\"height\"/> </height> </image>\n\
         <text name=\"rvloot_name\"> <depth> 2 </depth> <justify> &center; </justify> <font> {FONT} </font> {INK} <wrapwidth> {inner} </wrapwidth> <wraplines> 1 </wraplines> <string> <copy src=\"parent()\" trait=\"user0\"/> </string> <x> {} </x>\n\
         <y> {TITLE} <sub src=\"me()\" trait=\"height\"/> <div> 2 </div> </y> </text>\n",
        AT.0,
        AT.1,
        top + RULE + FOOTER,
        w * 0.5
    );
    x.push_str(&rule("rvloot_rule_top", RULE_ZOOM, 0.0, &TITLE.to_string(), w));
    // The small icons over the value and weight columns (the game's, as its card has them).
    for (name, crop, cx) in [("rvloot_value_icon", 224, VALUE_X), ("rvloot_weight_icon", 32, WEIGHT_X)] {
        x.push_str(&format!(
            "<image name=\"{name}\"> <filename> Menus\\Shared\\all_small_icons.dds </filename> <cropx> {crop} </cropx> <depth> 2 </depth> <width> 32 </width> <height> 32 </height> <x> {} </x> <y> {} </y> </image>\n",
            cx - 16.0,
            TITLE + RULE + (HEADER - 32.0) * 0.5
        ));
    }
    // The picked row: the HUD's parchment, faint, between two gilt rules.
    x.push_str(&format!(
        "<rect name=\"rvloot_focus\"> <id> {FOCUS_ID} </id> <locus> &true; </locus> <depth> 2 </depth> <x> {PAD} </x> <y> {top} </y> <width> {inner} </width> <height> {ROW} </height>\n\
         <image name=\"rvloot_focus_light\"> <filename> Menus\\HUD\\hud_back.dds </filename> <cropx> 60 </cropx> <cropy> 12 </cropy> <alpha> {LIT} </alpha> <depth> 1 </depth> <width> {inner} </width> <height> {ROW} </height> <x> 0 </x> <y> 0 </y> </image>\n",
    ));
    x.push_str(&rule("rvloot_focus_top", RULE_ROW_ZOOM, 0.0, "-2", inner));
    x.push_str(&rule("rvloot_focus_bottom", RULE_ROW_ZOOM, 0.0, &(ROW - 3.0).to_string(), inner));
    x.push_str("</rect>\n");
    for i in 0..ROWS {
        x.push_str(&row(i, inner));
    }
    x.push_str(&rule("rvloot_rule_bottom", RULE_ZOOM, 0.0, &below(1.0), w));
    // The buttons: A, Take All; the trigger, Take (the Xbox's button art, the game's words).
    x.push_str(&format!(
        "<rect name=\"rvloot_buttons\"> <locus> &true; </locus> <depth> 3 </depth> <x> 0 </x> <y> {} </y> <width> {w} </width> <height> 64 </height>\n",
        below(RULE + 2.0)
    ));
    for (n, crop, word, bx) in [(0, 128, "_takeall", PAD), (1, 256, "_take", w * 0.5)] {
        x.push_str(&format!(
            "<image name=\"rvloot_button_{n}\"> <filename> Menus\\Shared\\all_xbox_buttons.dds </filename> <cropx> {crop} </cropx> <depth> 1 </depth> <width> 64 </width> <height> 64 </height> <x> {bx} </x> <y> 0 </y> </image>\n\
             <text name=\"rvloot_button_{n}_text\"> <depth> 1 </depth> <font> {FONT} </font> {INK} <string> <copy src=\"strings()\" trait=\"{word}\"/> </string> <x> {} </x>\n\
             <y> 64 <sub src=\"me()\" trait=\"height\"/> <div> 2 </div> </y> </text>\n",
            bx + 70.0
        ));
    }
    x.push_str("</rect>\n</rect>\n");
    x
}

/// Row `i`: an item's icon (`user0`), its name and count (`user1`), value (`user2`) and weight
/// (`user3`); shown while `user8`.
fn row(i: usize, w: f32) -> String {
    let id = ROW_ID + i as i32;
    let y = rows_top() + i as f32 * ROW;
    let mid = "<y> <copy src=\"parent()\" trait=\"height\"/> <sub src=\"me()\" trait=\"height\"/> <div> 2 </div> </y>";
    format!(
        "<rect name=\"rvloot_row{i}\"> <id> {id} </id> <locus> &true; </locus> <depth> 3 </depth> <visible> <copy src=\"me()\" trait=\"user8\"/> </visible>\n\
         <x> {PAD} </x> <y> {y} </y> <width> {w} </width> <height> {ROW} </height>\n\
         <user0> </user0> <user1> </user1> <user2> </user2> <user3> </user3> <user8> &false; </user8>\n\
         <image name=\"rvloot_row{i}_icon\"> <depth> 1 </depth> <filename> <copy src=\"parent()\" trait=\"user0\"/> </filename> <zoom> -1 </zoom> <width> {ICON} </width> <height> {ICON} </height> <x> 4 </x> <y> {} </y> </image>\n\
         <text name=\"rvloot_row{i}_name\"> <depth> 1 </depth> <justify> &left; </justify> <font> {FONT} </font> {INK} <wrapwidth> {} </wrapwidth> <wraplines> 1 </wraplines> <string> <copy src=\"parent()\" trait=\"user1\"/> </string> <x> {NAME_X} </x> {mid} </text>\n\
         <text name=\"rvloot_row{i}_value\"> <depth> 1 </depth> <justify> &center; </justify> <font> {FONT} </font> {INK} <wrapwidth> 70 </wrapwidth> <wraplines> 1 </wraplines> <string> <copy src=\"parent()\" trait=\"user2\"/> </string> <x> {} </x> {mid} </text>\n\
         <text name=\"rvloot_row{i}_weight\"> <depth> 1 </depth> <justify> &center; </justify> <font> {FONT} </font> {INK} <wrapwidth> 70 </wrapwidth> <wraplines> 1 </wraplines> <string> <copy src=\"parent()\" trait=\"user3\"/> </string> <x> {} </x> {mid} </text>\n\
         </rect>\n",
        (ROW - ICON) * 0.5,
        VALUE_X - 40.0 - NAME_X - PAD,
        VALUE_X - PAD,
        WEIGHT_X - PAD
    )
}

/// Which rows show, picking `sel` of `n` things with `top` the first shown: the first shown kept so
/// the picked one's in view.
pub fn scrolled(sel: usize, top: usize, n: usize) -> usize {
    if n <= ROWS {
        0
    } else if sel < top {
        sel
    } else if sel >= top + ROWS {
        sel + 1 - ROWS
    } else {
        top.min(n - ROWS)
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn shown() -> bool {
    false
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn opening() -> bool {
    false
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{combat, engine, items, mem, tiles};
    use rustvr_protocol::Action;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// `TESObjectREFR`: its base, flags, 3D; `TESForm`: its type.
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const FORM_FLAGS: usize = 0x8;
    const FORM_TYPE: usize = 0x4;
    const DELETED: u32 = 0x20;
    const DISABLED: u32 = 0x800;
    /// Form types: a container, a person, a creature.
    const CONTAINER: u8 = 0x17;
    const NPC: u8 = 0x23;
    const CREATURE: u8 = 0x24;
    /// A thing's things, as the container menu walks them: how many entries (thiscall: the
    /// reference, wares), each (thiscall: index, wares; a heap copy: its extra lists +0, count +4,
    /// form +8, freed by 0x484470 then `FormHeap_Free`); whether the menu lists one (thiscall: the
    /// person or none, barter, hour, all, 0), its owner and its value (thiscall).
    const INV_COUNT: usize = 0x004D_8950;
    const INV_ITEM: usize = 0x004D_88F0;
    const ENTRY_FREE: usize = 0x0048_4470;
    const HEAP_FREE: usize = 0x0040_1F20;
    const ENTRY_SHOWN: usize = 0x0048_54F0;
    const ENTRY_OWNER: usize = 0x0048_4B70;
    const ENTRY_VALUE: usize = 0x0048_42E0;
    /// `TESObjectREFR`: its owner (thiscall); whether the player may use it as theirs (thiscall:
    /// the player, 1); its lock (thiscall) and whether that's locked (thiscall on the lock), as the
    /// game's `GetLocked` asks.
    const GET_OWNER: usize = 0x004D_B6B0;
    const MAY_TAKE: usize = 0x004D_E770;
    const GET_LOCK: usize = 0x004D_7740;
    const IS_LOCKED: usize = 0x0042_8E70;
    /// `TESObjectREFR::RemoveItem` (vtable +0x100: form, extra list, count, the stealing flag,
    /// drop, whose it goes to, where it's dropped, its angles, 1, 0); `Actor`'s dead (+0x198, 0).
    const VT_REMOVE_ITEM: usize = 0x100;
    const VT_IS_DEAD: usize = 0x198;
    /// The player: a theft seen (vtable +0x238: the victim, 0, 0, the value, its owner), the
    /// things it's stolen (a statistic).
    const VT_THEFT: usize = 0x238;
    const PC_STOLEN: usize = 0x6C8;
    /// `Actor::PlayItemSound(actor, form, up, use)` (thiscall), as the container menu sounds a take.
    const ITEM_SOUND: usize = 0x005E_99C0;
    /// The inventory's icon of a thing (cdecl: its form; under `Icons\`).
    const ICON: usize = 0x005C_0C50;
    /// `TESScriptableForm`'s RTTI: its script at +4. A `Script`'s compiled code (+0x30) and its
    /// length (+0x20): its blocks, each a `Begin` line (0x10: an opcode, its length, then the
    /// block's type, `OnActivate`'s 2, and how long the block is past it), outside them lines of
    /// an opcode, a length and that much more (its name's).
    const RTTI_SCRIPTABLE: usize = 0x00B0_331C;
    const SCRIPT_CODE: usize = 0x30;
    const SCRIPT_LENGTH: usize = 0x20;
    const BEGIN: u16 = 0x10;
    const ON_ACTIVATE: u16 = 2;
    /// `Tile::ReadXML(parent, path)`, `Tile::UpdateFloat`, `Tile::UpdateString` (thiscall).
    const READ_XML: usize = 0x0059_0420;
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const UPDATE_STRING: usize = 0x0058_CED0;
    /// Tile values: id, user0..3, user8.
    const ID: u16 = 0xFA8;
    const USER0: u32 = 0xFAE;
    const USER1: u32 = 0xFAF;
    const USER2: u32 = 0xFB0;
    const USER3: u32 = 0xFB1;
    const USER8: u32 = 0xFB6;
    const FALSE: f32 = 1.0;
    /// The HUD (`kMenuType_HUDMain`), whose tiles the list goes in; the container menu
    /// (`kMenuType_Container`): its container (+0x44), the type it put on the menu stack (+0x14),
    /// its click (vtable slot 3: the id and the clicked tile) and its Exit (id and tile).
    const MENU_HUD: u32 = 0x3EC;
    const MENU_OF: usize = 0x44;
    const MENU_STACKED: usize = 0x14;
    const VT_HANDLE_CLICK: usize = 3 * 4;
    const EXIT: u32 = 33;
    const EXIT_BUTTON: &str = "cont_button_exit";
    /// How long a script is given to open what it was activated for (seconds).
    const SCRIPT_WAIT: f64 = 1.5;
    /// Seconds between looks at what's in it.
    const REFRESH: f64 = 0.25;
    /// How long the list stays on its thing with the aim off it (slipped past a body's arm), and
    /// how long the aim's on another before the list goes over to that (seconds).
    const HOLD: f64 = 0.6;
    const SWITCH: f64 = 0.25;

    static PATH: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    /// The thing being activated for a look (its script run): the container menu it opens isn't
    /// pushed on the menu stack ([`crate::books`]'s push).
    static OPENING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    /// A thing is being activated for a look: the container menu its script opens stays off the
    /// menu stack, so the game plays on.
    pub fn opening() -> bool {
        OPENING.load(Ordering::Relaxed) != 0
    }
    /// The list is up (its things beside a hand): the trigger is its.
    static SHOWN: AtomicBool = AtomicBool::new(false);

    /// The list is up at what a hand points at: its trigger takes, its A takes everything.
    pub fn shown() -> bool {
        SHOWN.load(Ordering::Relaxed)
    }
    /// What the list is of (0: none).
    static LISTED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    /// What the list is of, while it's up (0: none): the HUD floats it there.
    pub fn listed() -> usize {
        if shown() { LISTED.load(Ordering::Relaxed) } else { 0 }
    }

    /// One of the game's entries of what's in the thing, as it was read.
    #[derive(Clone, Debug, PartialEq)]
    struct Thing {
        form: usize,
        extra: usize,
        count: i32,
        name: String,
        value: Option<i32>,
        weight: Option<f32>,
        icon: String,
    }

    /// What's being looted.
    struct Loot {
        refr: usize,
        name: String,
        things: Vec<Thing>,
        sel: usize,
        top: usize,
        /// The container menu's stealing, as it decides it opening; and what's been stolen.
        stealing: bool,
        stolen: i32,
        read_at: f64,
    }

    #[derive(Default)]
    struct State {
        loot: Option<Loot>,
        /// The list's tiles: its top, the HUD's tile it's in, its rows, its focus box.
        tile: usize,
        parent: usize,
        rows: Vec<usize>,
        focus: usize,
        shown: bool,
        /// What each row's values were last given.
        given: HashMap<(usize, u32), String>,
        focus_y: Option<f32>,
        /// The runtime's presses seen last (menu up, menu down, use, loot all).
        presses: [u16; 4],
        /// What a hand pointed at last (for the log).
        pointed: usize,
        /// What the list is of (held through the aim's slips, [`HOLD`]), when the aim was last on
        /// it; another the aim's on, and since when ([`SWITCH`]).
        on: usize,
        on_seen: f64,
        next: usize,
        next_since: f64,
        /// The thing's name the list was given.
        named: String,
        /// A thing whose script has its own activation: the one its script has opened for this
        /// look, and the one it didn't; when the one activated was.
        scripted: usize,
        refused: usize,
        activated_at: f64,
        failed: bool,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn vcall(obj: usize, slot: usize) -> Option<usize> {
        let f = mem::deref(mem::deref(obj) + slot);
        (f != 0).then_some(f)
    }

    fn update_float(tile: usize, id: u32, v: f32) {
        type F = unsafe extern "thiscall" fn(usize, u32, f32);
        if tile != 0 {
            crate::hooks::guard_void("loot tile", || unsafe { std::mem::transmute::<usize, F>(UPDATE_FLOAT)(tile, id, v) });
        }
    }

    fn update_text(tile: usize, id: u32, text: &str) {
        type F = unsafe extern "thiscall" fn(usize, u32, *const u8);
        let mut c: Vec<u8> = text.bytes().filter(|&b| b != 0).collect();
        c.push(0);
        if tile != 0 {
            crate::hooks::guard_void("loot label", || unsafe { std::mem::transmute::<usize, F>(UPDATE_STRING)(tile, id, c.as_ptr()) });
        }
    }

    /// The list's file written (beside the runtime's settings).
    pub fn install() {
        let dir = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("RustVR");
        let path = dir.join("oblivion_loot.xml");
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, xml())) {
            tracing::warn!("quickloot: can't write its list {}: {e}", path.display());
            return;
        }
        let _ = PATH.set(std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default());
    }

    /// Whether the thing can be looted at a glance (else why not): a container the game would open,
    /// or a body; one whose script has its own activation, once that's opened it (`SCRIPTED`).
    const SCRIPTED: &str = "its script has its own activation";
    fn lootable(refr: usize) -> Result<(), &'static str> {
        if refr == 0 || mem::at::<u32>(refr + FORM_FLAGS).is_none_or(|f| f & (DELETED | DISABLED) != 0) || mem::deref(refr + REFR_NODE) == 0 {
            return Err("it isn't there");
        }
        let base = mem::deref(refr + REFR_BASE);
        match mem::at::<u8>(base + FORM_TYPE) {
            Some(CONTAINER) if locked(refr) => return Err("it's locked"),
            Some(CONTAINER) => {}
            Some(NPC | CREATURE) if combat::is_actor(refr) && dead(refr) => {}
            Some(NPC | CREATURE) => return Err("they're alive"),
            _ => return Err("it holds nothing"),
        }
        let scriptable = items::cast(base, RTTI_SCRIPTABLE);
        let script = if scriptable != 0 { mem::deref(scriptable + 4) } else { 0 };
        if script != 0 && activates(script) {
            return Err(SCRIPTED);
        }
        Ok(())
    }

    /// A script has an `OnActivate` block (or its code can't be read: as if it had).
    fn activates(script: usize) -> bool {
        let (code, n) = (mem::deref(script + SCRIPT_CODE), mem::at::<u32>(script + SCRIPT_LENGTH).unwrap_or(0) as usize);
        let Some(bytes) = (code != 0 && n > 0 && n < 1 << 20).then(|| mem::bytes(code as *const u8, n)).flatten() else { return true };
        let word = |at: usize| bytes.get(at..at + 2).map(|w| u16::from_le_bytes([w[0], w[1]]));
        let long = |at: usize| bytes.get(at..at + 4).map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]));
        let mut at = 0;
        while at + 4 <= n {
            let (Some(op), Some(len)) = (word(at), word(at + 2)) else { return true };
            at += 4 + len as usize;
            if op == BEGIN {
                let (Some(kind), Some(block)) = (word(at - len as usize), long(at - len as usize + 2)) else { return true };
                if kind == ON_ACTIVATE {
                    return true;
                }
                at += block as usize;
            }
        }
        at != n
    }

    fn locked(refr: usize) -> bool {
        type Get = unsafe extern "thiscall" fn(usize) -> usize;
        type Is = unsafe extern "thiscall" fn(usize) -> u8;
        let lock = unsafe { std::mem::transmute::<usize, Get>(GET_LOCK)(refr) };
        lock != 0 && unsafe { std::mem::transmute::<usize, Is>(IS_LOCKED)(lock) } != 0
    }

    fn dead(actor: usize) -> bool {
        vcall(actor, VT_IS_DEAD).is_some_and(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32) -> u8>(f)(actor, 0) } != 0)
    }

    fn owner_of(refr: usize) -> usize {
        unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(GET_OWNER)(refr) }
    }

    /// The container menu's stealing as it opens on a container or a body: the thing has an owner,
    /// the player may not use it as theirs, and it isn't a body.
    fn stealing(refr: usize, player: usize) -> bool {
        type May = unsafe extern "thiscall" fn(usize, usize, u32) -> u8;
        owner_of(refr) != 0 && unsafe { std::mem::transmute::<usize, May>(MAY_TAKE)(refr, player, 1) } == 0 && !combat::is_actor(refr)
    }

    /// One of the game's entries as it is now: its form, extra list, count, owner and value.
    struct Entry {
        form: usize,
        extra: usize,
        count: i32,
        owner: usize,
        value: i32,
    }

    /// What's in the thing, as its container menu lists it.
    fn entries(refr: usize) -> Vec<Entry> {
        type Count = unsafe extern "thiscall" fn(usize, u32) -> i32;
        type Item = unsafe extern "thiscall" fn(usize, i32, u32) -> usize;
        type Shown = unsafe extern "thiscall" fn(usize, usize, u32, i32, u32, u32) -> u8;
        type Of = unsafe extern "thiscall" fn(usize) -> usize;
        type Value = unsafe extern "thiscall" fn(usize) -> i32;
        type Free = unsafe extern "thiscall" fn(usize);
        type HeapFree = unsafe extern "C" fn(usize);
        let person = if combat::is_actor(refr) { refr } else { 0 };
        let n = unsafe { std::mem::transmute::<usize, Count>(INV_COUNT)(refr, 0) };
        let mut out = Vec::new();
        for i in 0..n.clamp(0, 256) {
            let e = unsafe { std::mem::transmute::<usize, Item>(INV_ITEM)(refr, i, 0) };
            if e == 0 {
                continue;
            }
            let form = mem::deref(e + 8);
            let count = mem::at::<i32>(e + 4).unwrap_or(0);
            if form != 0 && count > 0 && unsafe { std::mem::transmute::<usize, Shown>(ENTRY_SHOWN)(e, person, 0, 1, 1, 0) } != 0 {
                let lists = mem::deref(e);
                out.push(Entry {
                    form,
                    extra: if lists != 0 { mem::deref(lists) } else { 0 },
                    count,
                    owner: unsafe { std::mem::transmute::<usize, Of>(ENTRY_OWNER)(e) },
                    value: unsafe { std::mem::transmute::<usize, Value>(ENTRY_VALUE)(e) },
                });
            }
            unsafe {
                std::mem::transmute::<usize, Free>(ENTRY_FREE)(e);
                std::mem::transmute::<usize, HeapFree>(HEAP_FREE)(e);
            }
        }
        out
    }

    /// What's in the thing, to show: the game's name, value, weight and icon of each.
    fn things(refr: usize) -> Vec<Thing> {
        type Icon = unsafe extern "C" fn(usize) -> usize;
        entries(refr)
            .into_iter()
            .map(|e| {
                let facts = items::form_facts(e.form);
                let icon = combat::read_str(unsafe { std::mem::transmute::<usize, Icon>(ICON)(e.form) });
                Thing {
                    form: e.form,
                    extra: e.extra,
                    count: e.count,
                    name: facts.name,
                    value: facts.value,
                    weight: facts.weight,
                    icon: if icon.is_empty() { String::new() } else { format!("Icons\\{icon}") },
                }
            })
            .collect()
    }

    impl Loot {
        fn open(refr: usize, player: usize, now: f64) -> Loot {
            let name = items::form_facts(mem::deref(refr + REFR_BASE)).name;
            let stealing = stealing(refr, player);
            let things = things(refr);
            tracing::info!("quickloot: {name} ({refr:#x}) at the hand: {} things{}", things.len(), if stealing { ", taking them is stealing" } else { "" });
            Loot { refr, name, things, sel: 0, top: 0, stealing, stolen: 0, read_at: now }
        }

        /// A stack taken, as the container menu takes it: the stealing's value added up (none
        /// once a thing of the player's own is taken: the game's rule), its `RemoveItem` into the
        /// player's things, its item sound.
        fn take(&mut self, i: usize, player: usize) -> bool {
            let Some(t) = self.things.get(i).cloned() else { return false };
            let Some(e) = entries(self.refr).into_iter().find(|e| e.form == t.form && e.extra == t.extra) else { return false };
            if self.stealing {
                if e.owner != 0 && e.owner == mem::deref(player + REFR_BASE) {
                    self.stealing = false;
                } else {
                    self.stolen += e.value * e.count;
                    if let Some(c) = mem::at::<i32>(player + PC_STOLEN) {
                        unsafe { ((player + PC_STOLEN) as *mut i32).write(c + e.count) };
                    }
                }
            }
            type Remove = unsafe extern "thiscall" fn(usize, usize, usize, i32, u32, u32, usize, *const f32, *const f32, u32, u32);
            type Sound = unsafe extern "thiscall" fn(usize, usize, u8, u8);
            let Some(f) = vcall(self.refr, VT_REMOVE_ITEM) else { return false };
            unsafe { std::mem::transmute::<usize, Remove>(f)(self.refr, e.form, e.extra, e.count, self.stealing as u32, 0, player, std::ptr::null(), std::ptr::null(), 1, 0) };
            unsafe { std::mem::transmute::<usize, Sound>(ITEM_SOUND)(player, e.form, 1, 0) };
            tracing::info!("quickloot: {} x{} taken from {}{}", t.name, e.count, self.name, if self.stealing { " (stolen)" } else { "" });
            true
        }

        /// Done with it: whoever sees the player now saw what was stolen, as closing the container
        /// menu has it.
        fn close(&mut self, player: usize) {
            if self.stealing && self.stolen > 0 {
                tracing::info!("quickloot: done with {}, {} gold's worth stolen: the game's check for witnesses", self.name, self.stolen);
                if let Some(f) = vcall(player, VT_THEFT) {
                    let owner = owner_of(self.refr);
                    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, usize, usize, usize, i32, usize) -> i32>(f)(player, self.refr, 0, 0, self.stolen, owner) };
                }
            }
            self.stolen = 0;
        }

        fn read(&mut self, now: f64) {
            self.things = things(self.refr);
            self.read_at = now;
            self.sel = self.sel.min(self.things.len().saturating_sub(1));
            self.top = scrolled(self.sel, self.top, self.things.len());
        }
    }

    impl State {
        /// What the list is of now, from what the aim's on (`candidate`, 0: nothing to loot): kept
        /// through a moment off it, over to another once the aim's stayed there, none with no hand
        /// up (but not while a script's opening it: its menu is up a moment).
        fn follow(&mut self, candidate: usize, hand_up: bool, now: f64) {
            let was = self.on;
            if !hand_up && OPENING.load(Ordering::Relaxed) == 0 {
                (self.on, self.next) = (0, 0);
            } else if candidate != 0 && candidate == self.on {
                (self.on_seen, self.next) = (now, 0);
            } else if candidate != 0 {
                if self.next != candidate {
                    (self.next, self.next_since) = (candidate, now);
                }
                if self.on == 0 || now - self.next_since >= SWITCH {
                    (self.on, self.on_seen, self.next) = (candidate, now, 0);
                }
            }
            if self.on != 0 && now - self.on_seen > HOLD && OPENING.load(Ordering::Relaxed) == 0 {
                self.on = 0;
            }
            // (Another look: a script of its own runs again.)
            if self.on != was {
                (self.scripted, self.refused) = (0, 0);
            }
        }

        /// A thing whose script has its own activation, pointed at (`scripted`): activated once for
        /// this look; the container menu its script opens, shut; or given up on.
        fn scripts(&mut self, target: usize, scripted: bool, now: f64) {
            if OPENING.load(Ordering::Relaxed) == 0 && scripted && target != self.scripted && target != self.refused {
                let name = items::form_facts(mem::deref(target + REFR_BASE)).name;
                tracing::info!("quickloot: {name} ({target:#x}) has a script of its own activation: activated as the player would, its script run");
                OPENING.store(target, Ordering::Relaxed);
                self.activated_at = now;
                crate::world::activate(target);
            }
            let waiting = OPENING.load(Ordering::Relaxed);
            if waiting == 0 {
                return;
            }
            let menu = tiles::menu(super::MENU_CONTAINER);
            if menu != 0 && mem::deref(menu + MENU_OF) == waiting {
                // (Off the stack: its close mustn't take another menu off it.)
                if mem::is_readable((menu + MENU_STACKED) as *const u8, 4) {
                    unsafe { ((menu + MENU_STACKED) as *mut u32).write(0) };
                }
                // (Its Exit's own tile with it: the click fades it.)
                let f = mem::deref(mem::deref(menu) + VT_HANDLE_CLICK);
                let exit = tiles::Tree::read(tiles::menu_tile(super::MENU_CONTAINER)).find(EXIT_BUTTON).map_or(0, |p| p.tile);
                if (0x0040_1000..0x00A0_0000).contains(&f) && exit != 0 {
                    type Click = unsafe extern "thiscall" fn(usize, u32, usize);
                    unsafe { std::mem::transmute::<usize, Click>(f)(menu, EXIT, exit) };
                } else {
                    tracing::warn!("quickloot: the container menu {waiting:#x}'s script opened has no Exit to shut it with");
                }
                tracing::info!("quickloot: its script opened {waiting:#x}: its menu shut, what's in it listed at the hand");
                self.scripted = waiting;
                OPENING.store(0, Ordering::Relaxed);
            } else if now - self.activated_at > SCRIPT_WAIT {
                tracing::info!("quickloot: its script didn't open {waiting:#x}: nothing to list");
                self.refused = waiting;
                OPENING.store(0, Ordering::Relaxed);
            }
        }

        /// The list's tiles, as they are (gone with the HUD's, made again).
        fn alive(&self) -> bool {
            self.tile != 0 && tiles::menu_tile(MENU_HUD) == self.parent && tiles::children(self.parent).contains(&self.tile)
        }

        fn make(&mut self) -> bool {
            let Some(path) = PATH.get() else { return false };
            let parent = tiles::menu_tile(MENU_HUD);
            if parent == 0 {
                return false;
            }
            type Read = unsafe extern "thiscall" fn(usize, *const u8) -> usize;
            let tile = crate::hooks::guard("loot list", 0, || unsafe { std::mem::transmute::<usize, Read>(READ_XML)(parent, path.as_ptr() as *const u8) });
            if tile == 0 {
                tracing::warn!("quickloot: the game made no list from {path:?}");
                self.failed = true;
                return false;
            }
            let ids: Vec<(i32, usize)> = tiles::children(tile).into_iter().filter_map(|t| tiles::value(t, ID).map(|id| (id as i32, t))).collect();
            self.rows = (0..ROWS).filter_map(|i| ids.iter().find(|(id, _)| *id == ROW_ID + i as i32).map(|&(_, t)| t)).collect();
            self.focus = ids.iter().find(|(id, _)| *id == FOCUS_ID).map_or(0, |&(_, t)| t);
            (self.tile, self.parent, self.shown, self.focus_y) = (tile, parent, false, None);
            self.given.clear();
            self.named.clear();
            // (In from the screen's right edge.)
            let screen = tiles::value(parent, tiles::WIDTH).filter(|w| *w > WIDTH).unwrap_or(1280.0);
            update_float(tile, tiles::X as u32, screen - WIDTH - AT.0);
            tracing::info!(
                "quickloot: its list made in the HUD of the game's own art ({} rows{}), at {} across the screen's {screen}",
                self.rows.len(),
                if self.focus == 0 { ", no focus box" } else { "" },
                screen - WIDTH - AT.0
            );
            true
        }

        fn show(&mut self, on: bool) {
            if self.shown != on && self.alive() {
                update_float(self.tile, tiles::VISIBLE as u32, if on { tiles::TRUE } else { FALSE });
            }
            self.shown = on && self.tile != 0;
        }

        fn give(&mut self, row: usize, id: u32, text: &str) {
            let Some(&tile) = self.rows.get(row) else { return };
            if self.given.get(&(row, id)).is_none_or(|t| t != text) {
                update_text(tile, id, text);
                self.given.insert((row, id), text.to_string());
            }
        }

        fn visible(&mut self, row: usize, on: bool) {
            let Some(&tile) = self.rows.get(row) else { return };
            let text = if on { "1" } else { "0" };
            if self.given.get(&(row, USER8)).is_none_or(|t| t != text) {
                update_float(tile, USER8, if on { tiles::TRUE } else { FALSE });
                self.given.insert((row, USER8), text.to_string());
            }
        }

        /// The rows as the list is now: the things shown, the picked one in the focus box.
        fn lay_out(&mut self) {
            let Some(l) = &self.loot else { return };
            let (things, top, sel) = (l.things.clone(), l.top, l.sel);
            if self.named != l.name {
                update_text(self.tile, USER0, &l.name);
                self.named = l.name.clone();
            }
            // (As tall as the rows shown.)
            let shown = things.len().saturating_sub(top).min(ROWS);
            if self.given.get(&(ROWS, USER1)).is_none_or(|n| *n != shown.to_string()) {
                update_float(self.tile, USER1, shown as f32);
                self.given.insert((ROWS, USER1), shown.to_string());
            }
            for row in 0..ROWS {
                match things.get(top + row) {
                    Some(t) => {
                        // (A stack's count after its name, as the game's lists have it; a space for
                        // nothing, where nothing given reads as a 0.)
                        self.give(row, USER0, &t.icon);
                        self.give(row, USER1, &if t.count > 1 { format!("{} ({})", t.name, t.count) } else { t.name.clone() });
                        self.give(row, USER2, &t.value.map_or(" ".to_string(), |v| v.to_string()));
                        self.give(row, USER3, &t.weight.map_or(" ".to_string(), items::weight_text));
                        self.visible(row, true);
                    }
                    None => self.visible(row, false),
                }
            }
            let y = rows_top() + sel.saturating_sub(top) as f32 * ROW;
            if self.focus != 0 && self.focus_y != Some(y) {
                update_float(self.focus, tiles::Y as u32, y);
                self.focus_y = Some(y);
            }
        }
    }

    /// Once a game frame (where its scripts run): what a free hand points at looted, its list
    /// shown, what the runtime pressed for it done.
    pub fn frame(now: f64) {
        let player = engine::player();
        let snap = crate::link::last();
        let presses = snap.map(|s| {
            let p = &s.frame.intents.presses;
            [p[Action::MenuUp as usize], p[Action::MenuDown as usize], p[Action::Use as usize], p[Action::LootAll as usize]]
        });
        let mut guard = STATE.lock().unwrap();
        let s = guard.get_or_insert_with(State::default);
        let seen = std::mem::replace(&mut s.presses, presses.unwrap_or([0; 4]));
        let pressed = |i: usize| presses.is_some_and(|p| p[i] != seen[i]);
        if s.tile != 0 && !s.alive() {
            (s.tile, s.rows, s.focus, s.shown) = (0, Vec::new(), 0, false);
        }
        // What a free hand points at (none in a menu, or with no hand put up).
        let vr = snap.is_some_and(|s| s.view.is_some());
        let hand_up = vr && player != 0 && !crate::menus::menu_mode() && crate::pointing::hand().is_some();
        let target = if hand_up { crate::hud_info::looked_at() } else { 0 };
        let test = |r: usize| crate::hooks::guard("loot test", Err("it couldn't be read"), || lootable(r));
        let why = (target != 0).then(|| test(target));
        if target != s.pointed {
            s.pointed = target;
            if let Some(Err(why)) = why.filter(|w| *w != Err(SCRIPTED)) {
                tracing::info!("quickloot: {} ({target:#x}) isn't looted at the hand: {why}", items::form_facts(mem::deref(target + REFR_BASE)).name);
            }
        }
        // What the list is of: held through the aim's slips; dropped once it can't be looted (gone,
        // or locked).
        s.follow(if matches!(why, Some(Ok(()) | Err(SCRIPTED))) { target } else { 0 }, hand_up, now);
        let on_why = (s.on != 0).then(|| if s.on == target { why.unwrap_or(Ok(())) } else { test(s.on) });
        if matches!(on_why, Some(Err(w)) if w != SCRIPTED) {
            s.on = 0;
        }
        // One whose script has its own activation: activated as the player would, its script run,
        // once a look; the container menu that opens (off the stack) shut with its own Exit, and
        // what's in it listed. (Or the script didn't open it: nothing to list.)
        let on = s.on;
        crate::hooks::guard_void("loot script", || s.scripts(on, on_why == Some(Err(SCRIPTED)), now));
        let target = if on != 0 && (on_why == Some(Ok(())) || s.scripted == on) { on } else { 0 };
        LISTED.store(target, Ordering::Relaxed);
        if s.loot.as_ref().is_some_and(|l| l.refr != target) {
            let mut l = s.loot.take().unwrap();
            crate::hooks::guard_void("loot close", || l.close(player));
        }
        if target != 0 && s.loot.is_none() {
            s.loot = crate::hooks::guard("loot open", None, || Some(Loot::open(target, player, now)));
        }
        let Some(l) = s.loot.as_mut() else {
            SHOWN.store(false, Ordering::Relaxed);
            s.show(false);
            return;
        };
        if now - l.read_at >= REFRESH {
            crate::hooks::guard_void("loot read", || l.read(now));
        }
        let up = SHOWN.load(Ordering::Relaxed);
        if up && !l.things.is_empty() {
            let n = l.things.len();
            if pressed(0) {
                l.sel = l.sel.saturating_sub(1);
            }
            if pressed(1) {
                l.sel = (l.sel + 1).min(n - 1);
            }
            let mut took = false;
            if pressed(3) {
                tracing::info!("quickloot: everything in {} taken", l.name);
                for i in (0..n).rev() {
                    took |= crate::hooks::guard("loot take", false, || l.take(i, player));
                }
            } else if pressed(2) {
                let sel = l.sel;
                took = crate::hooks::guard("loot take", false, || l.take(sel, player));
            }
            if took {
                crate::hooks::guard_void("loot read", || l.read(now));
            }
            l.top = scrolled(l.sel, l.top, l.things.len());
        }
        let any = !l.things.is_empty();
        if any && s.tile == 0 && !s.failed && !s.make() {
            SHOWN.store(false, Ordering::Relaxed);
            return;
        }
        SHOWN.store(any && s.tile != 0, Ordering::Relaxed);
        s.lay_out();
        s.show(any);
    }
}

