//! The holsters' editor: a popup made of Oblivion's own UI that sets up the body's slots
//! (`rustvr_protocol::inventory`): its parchment frame (`generic_background.xml`), list rows with
//! the inventory's own equipped marker for what's chosen, the game's own icons of the categories
//! (a thing the player has in each, else the game's own icon of its kind), its buttons
//! (`button_long.xml`, `button_xtralong.xml`), its fonts and its click sounds. It floats in front
//! of the player as the other menus do (`crate::hud`), and the laser drives it as any menu.
//!
//! On the left the slots; for the one picked, the categories it draws from (picked rows marked, with
//! their place in its order), and buttons: move the picked category up or down the order, the
//! order inside (most healing first, most valuable...), what reaching into it empty does, its size
//! and reach, the equipped first or not, Move (reach where you want it on your body and squeeze:
//! the runtime places it), New, Remove and Done. Each change goes to the runtime at once
//! (`SlotEdits`), which keeps it in its settings. It opens when the runtime asks (a grip held in a
//! slot, or Holsters in its menu beside the pause menu); B or Done closes it.
//!
//! It's a menu of its own: type 1050, which the game's menu factory (0x587D20) has no case for, so
//! the factory is hooked to make it a `Menu` whose clicks come here (a copy of the base `Menu`'s
//! vtable with its destructor, `HandleClick` and `GetID` ours). Its XML (written beside the
//! runtime's settings at start) is read by the game's own `Tile::ReadXML`, which takes an absolute
//! path; its `<include>`s are the game's own prefabs. Opening and closing it are the game's own
//! steps for its message box (0x5BC8B0) and generic menus. Facts: `docs/research/holsters.md`.

use rustvr_protocol::*;

/// Our menu's type.
pub const MENU_HOLSTERS: u32 = 1050;
/// Rows of slots shown.
pub const SLOT_ROWS: usize = 8;

/// The game's own icons of the categories (`crate::inventory::CATEGORIES`), when the player has
/// none of a kind to show its own.
pub const ICONS: [&str; 13] = [
    "Icons\\Clutter\\IconCheeseWheel.dds",
    "Icons\\Clutter\\Potions\\IconPotion01.dds",
    "Icons\\Clutter\\Potions\\IconPotionPoison01.dds",
    "Icons\\Clutter\\IconNirnroot.dds",
    "Icons\\Clutter\\IconScroll1.dds",
    "Icons\\Weapons\\IronArrow.dds",
    "Menus\\Inventory\\inv_icon_tab_weapons.dds",
    "Icons\\Clutter\\SoulGems\\SoulGem01.dds",
    "Icons\\Clutter\\Key\\Key.dds",
    "Icons\\Clutter\\IconBook1.dds",
    "Icons\\Clutter\\IconSigilStone.dds",
    "Icons\\Clutter\\IconMortarPestle.dds",
    "Menus\\Inventory\\inv_icon_tab_misc.dds",
];

/// The orders' names, by `RULE_*`.
pub const RULES: [&str; RULE_COUNT as usize] =
    ["Healing first", "Magicka first", "Fatigue first", "Most valuable", "Heaviest first", "Lightest first", "Newest first", "By name", "Most of first", "Strongest first", "My own order"];
/// What an empty slot does, by `EMPTY_*`.
pub const EMPTIES: [&str; EMPTY_COUNT as usize] = ["Empty: a tick", "Empty: nothing", "Empty: anything"];
/// Sizes (a slot's radius, metres) and reaches (how much further out it's taken from).
pub const SIZES: [(f32, &str); 3] = [(0.06, "Size: small"), (0.09, "Size: medium"), (0.13, "Size: large")];
pub const REACHES: [(f32, &str); 3] = [(0.02, "Reach: close"), (0.04, "Reach: normal"), (0.07, "Reach: far")];

/// The buttons, by id.
pub const UP: i32 = 300;
pub const DOWN: i32 = 301;
pub const ORDER: i32 = 302;
pub const EMPTY: i32 = 303;
pub const SIZE: i32 = 304;
pub const REACH: i32 = 305;
pub const EQUIPPED: i32 = 306;
pub const MOVE: i32 = 307;
pub const NEW: i32 = 308;
pub const REMOVE: i32 = 309;
pub const DONE: i32 = 310;
/// A slot row's id is this plus its index; a category row's the other.
pub const SLOT_ROW: i32 = 100;
pub const CATEGORY_ROW: i32 = 200;

/// The buttons: id, tile name, its prefab (long or extra long), its label until it's set.
const BUTTONS: [(i32, &str, bool, &str); 11] = [
    (UP, "hol_up", false, "Earlier"),
    (DOWN, "hol_down", false, "Later"),
    (ORDER, "hol_order", true, ""),
    (EMPTY, "hol_empty", true, ""),
    (SIZE, "hol_size", true, ""),
    (REACH, "hol_reach", true, ""),
    (EQUIPPED, "hol_equipped", true, ""),
    (MOVE, "hol_move", false, "Move"),
    (NEW, "hol_new", false, "New slot"),
    (REMOVE, "hol_remove", false, "Remove"),
    (DONE, "hol_done", false, ""),
];

/// The game's brown ink.
const INK: &str = "<red> 117 </red> <green> 59 </green> <blue> 33 </blue> <alpha> 255 </alpha>";

/// The menu's XML: the game's frame, rows and buttons (its prefabs), filled in by code.
pub fn xml(categories: usize) -> String {
    let mut x = String::new();
    x.push_str(&format!(
        "<menu name=\"RustVRHolstersMenu\">\n<class> {MENU_HOLSTERS} </class>\n<stackingtype> &no_click_past; </stackingtype>\n<locus> &true; </locus>\n<explorefade> 0.25 </explorefade>\n"
    ));
    // The frame, in the middle of the screen.
    x.push_str(
        "<rect name=\"hol_background\">\n<include src=\"generic_background.xml\"/>\n<id> &generic; </id>\n<target> &true; </target>\n<depth> 1 </depth>\n\
         <user0> 1000 </user0>\n<user1> 616 </user1>\n\
         <x> <copy src=\"screen()\" trait=\"width\"/> <div> 2 </div> <sub> <copy src=\"me()\" trait=\"width\"/> <div> 2 </div> </sub> </x>\n\
         <y> <copy src=\"screen()\" trait=\"height\"/> <div> 2 </div> <sub> <copy src=\"me()\" trait=\"height\"/> <div> 2 </div> </sub> </y>\n\
         <locus> &true; </locus>\n",
    );
    let text = |name: &str, s: &str, font: u32, x_: i32, y: i32, justify: &str, wrap: i32| {
        format!("<text name=\"{name}\">\n<depth> 3 </depth> <justify> &{justify}; </justify> <font> {font} </font> {INK}\n<wrapwidth> {wrap} </wrapwidth> <string> {s} </string> <x> {x_} </x> <y> {y} </y>\n</text>\n")
    };
    // (Each column in a rect of its own: a tile of more than a few dozen children is read as a
    // list by `crate::tiles`, its rows unread.)
    let column = |name: &str, x_: i32, w: i32| format!("<rect name=\"{name}\">\n<locus> &true; </locus> <depth> 2 </depth> <x> {x_} </x> <y> 44 </y> <width> {w} </width> <height> 572 </height>\n");
    x.push_str(&text("hol_title", "Holsters", 2, 500, -2, "center", 900));
    // The rows: a slot's (its name, what it gives now) and a category's (its icon, its name, its
    // place in the order); the chosen ones marked as the inventory marks what's worn; the one
    // pointed at lit as the inventory lights a worn one.
    let row = |name: &str, id: i32, x_: i32, y: i32, w: i32, h: i32, icon: bool| {
        let mut r = format!(
            "<rect name=\"{name}\">\n<id> {id} </id> <target> <copy src=\"me()\" trait=\"user8\"/> </target> <visible> <copy src=\"me()\" trait=\"user8\"/> </visible>\n\
             <clicksound> 1 </clicksound> <locus> &true; </locus> <depth> 3 </depth>\n\
             <x> {x_} </x> <y> {y} </y> <width> {w} </width> <height> {h} </height>\n\
             <user1> </user1> <user2> </user2> <user6> </user6> <user7> </user7> <user8> &false; </user8> <user10> &false; </user10>\n\
             <image name=\"{name}_on\"> <visible> <copy src=\"parent()\" trait=\"user10\"/> </visible> <depth> 1 </depth> <filename> Menus\\Inventory\\inv_equiped_marker_1.dds </filename> <alpha> 160 </alpha> <width> {w} </width> <height> {h} </height> <x> 0 </x> <y> 0 </y> </image>\n\
             <image name=\"{name}_over\"> <visible> <copy src=\"parent()\" trait=\"mouseover\"/> <eq> 1 </eq> </visible> <depth> 1 </depth> <filename> Menus\\Inventory\\inv_hilite_equipped.dds </filename> <alpha> 110 </alpha> <width> {w} </width> <height> {h} </height> <x> 0 </x> <y> 0 </y> </image>\n"
        );
        if icon {
            r.push_str(&format!(
                "<image name=\"{name}_icon\"> <depth> 2 </depth> <filename> <copy src=\"parent()\" trait=\"user6\"/> </filename> <width> 32 </width> <height> 32 </height> <x> 6 </x> <y> 3 </y> </image>\n\
                 <text name=\"{name}_name\"> <depth> 2 </depth> <justify> &left; </justify> <font> 1 </font> {INK} <string> <copy src=\"parent()\" trait=\"user1\"/> </string> <x> 46 </x> <y> 4 </y> </text>\n\
                 <text name=\"{name}_place\"> <depth> 2 </depth> <justify> &right; </justify> <font> 3 </font> {INK} <string> <copy src=\"parent()\" trait=\"user7\"/> </string> <x> {} </x> <y> 9 </y> </text>\n",
                w - 12
            ));
        } else {
            r.push_str(&format!(
                "<text name=\"{name}_name\"> <depth> 2 </depth> <justify> &left; </justify> <font> 1 </font> {INK} <string> <copy src=\"parent()\" trait=\"user1\"/> </string> <x> 12 </x> <y> 2 </y> </text>\n\
                 <text name=\"{name}_now\"> <depth> 2 </depth> <justify> &left; </justify> <font> 3 </font> {INK} <wrapwidth> {} </wrapwidth> <string> <copy src=\"parent()\" trait=\"user2\"/> </string> <x> 12 </x> <y> 29 </y> </text>\n",
                w - 20
            ));
        }
        r.push_str("</rect>\n");
        r
    };
    x.push_str(&column("hol_slots", 16, 300));
    x.push_str(&text("hol_slots_head", "Slots", 1, 0, 0, "left", 300));
    for k in 0..SLOT_ROWS {
        x.push_str(&row(&format!("hol_slot_{k}"), SLOT_ROW + k as i32, 0, 36 + 57 * k as i32, 300, 53, false));
    }
    x.push_str(&text("hol_now", "", 3, 0, 496, "left", 300));
    // (A rule under it, as the game's quest pop-up has: the strip ends there however many slots
    // and whatever the hint, so the strips don't shift as they change.)
    x.push_str(
        "<image name=\"hol_slots_rule\"> <filename> Menus\\Stats\\stat_border_horizontal_1.dds </filename> <depth> 3 </depth> <width> 300 </width> <height> 3 </height> <x> 0 </x> <y> 544 </y> </image>\n",
    );
    x.push_str("</rect>\n");
    x.push_str(&column("hol_categories", 340, 360));
    x.push_str(&text("hol_categories_head", "Draws from, in order", 1, 0, 0, "left", 360));
    for j in 0..categories {
        x.push_str(&row(&format!("hol_category_{j}"), CATEGORY_ROW + j as i32, 0, 36 + 40 * j as i32, 360, 38, true));
    }
    x.push_str("</rect>\n");
    x.push_str(&column("hol_buttons", 724, 255));
    for (i, (id, name, long, label)) in BUTTONS.iter().enumerate() {
        let (prefab, w) = if *long { ("button_xtralong.xml", 255) } else { ("button_long.xml", 177) };
        x.push_str(&format!(
            "<image name=\"{name}\">\n<include src=\"{prefab}\"/>\n<id> {id} </id> <target> &true; </target> <depth> 3 </depth> <clicksound> {} </clicksound>\n<user0> {label} </user0>\n<x> {} </x> <y> {} </y>\n</image>\n",
            if *id == DONE { 2 } else { 3 },
            (255 - w) / 2,
            52 * i as i32
        ));
    }
    x.push_str("</rect>\n</rect>\n</menu>\n");
    x
}

/// What a click did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clicked {
    Nothing,
    /// The slots changed: send them.
    Changed,
    /// Place the picked slot by hand.
    Place,
    Close,
}

/// The editor's own state: the slots as edited, which is picked, and the category last picked
/// (what Earlier and Later move).
#[derive(Clone, Debug, Default)]
pub struct Editor {
    pub slots: Vec<SlotDesc>,
    pub picked: usize,
    pub category: Option<u8>,
}

fn nearest<T>(table: &[(f32, T)], v: f32) -> usize {
    (0..table.len()).min_by(|&a, &b| (table[a].0 - v).abs().total_cmp(&(table[b].0 - v).abs())).unwrap_or(0)
}

impl Editor {
    pub fn slot(&mut self) -> Option<&mut SlotDesc> {
        self.slots.get_mut(self.picked)
    }

    /// The categories slot `k` draws from, in order.
    pub fn categories(&self, k: usize) -> Vec<u8> {
        self.slots.get(k).map(|s| s.categories().collect()).unwrap_or_default()
    }

    fn set_categories(&mut self, list: &[u8]) {
        if let Some(s) = self.slot() {
            s.categories = [SLOT_NO_CATEGORY; MAX_SLOT_CATEGORIES];
            for (i, c) in list.iter().take(MAX_SLOT_CATEGORIES).enumerate() {
                s.categories[i] = *c;
            }
        }
    }

    /// A click on the menu's tile `id`.
    pub fn click(&mut self, id: i32) -> Clicked {
        let k = (id - SLOT_ROW) as usize;
        if (SLOT_ROW..SLOT_ROW + SLOT_ROWS as i32).contains(&id) {
            if k < self.slots.len() {
                self.picked = k;
                self.category = None;
            }
            return Clicked::Nothing;
        }
        if (CATEGORY_ROW..CATEGORY_ROW + MAX_INV_CATEGORIES as i32).contains(&id) {
            let c = (id - CATEGORY_ROW) as u8;
            let mut list = self.categories(self.picked);
            match list.iter().position(|&x| x == c) {
                // (Picked again once it's the one picked: out of the slot.)
                Some(i) if self.category == Some(c) => {
                    list.remove(i);
                    self.category = None;
                }
                Some(_) => {
                    self.category = Some(c);
                    return Clicked::Nothing;
                }
                None if list.len() < MAX_SLOT_CATEGORIES => {
                    list.push(c);
                    self.category = Some(c);
                }
                None => return Clicked::Nothing,
            }
            self.set_categories(&list);
            return Clicked::Changed;
        }
        let Some(slot) = self.slots.get_mut(self.picked) else {
            return match id {
                NEW => self.new_slot(),
                DONE => Clicked::Close,
                _ => Clicked::Nothing,
            };
        };
        match id {
            UP | DOWN => {
                let Some(c) = self.category else { return Clicked::Nothing };
                let mut list = self.categories(self.picked);
                let Some(i) = list.iter().position(|&x| x == c) else { return Clicked::Nothing };
                let j = if id == UP { i.checked_sub(1) } else { (i + 1 < list.len()).then_some(i + 1) };
                let Some(j) = j else { return Clicked::Nothing };
                list.swap(i, j);
                self.set_categories(&list);
                Clicked::Changed
            }
            ORDER => {
                slot.rule = (slot.rule + 1) % RULE_COUNT;
                Clicked::Changed
            }
            EMPTY => {
                slot.empty = (slot.empty + 1) % EMPTY_COUNT;
                Clicked::Changed
            }
            SIZE => {
                slot.radius = SIZES[(nearest(&SIZES, slot.radius) + 1) % SIZES.len()].0;
                Clicked::Changed
            }
            REACH => {
                slot.reach = REACHES[(nearest(&REACHES, slot.reach) + 1) % REACHES.len()].0;
                Clicked::Changed
            }
            EQUIPPED => {
                slot.flags ^= SLOTF_EQUIPPED_FIRST;
                Clicked::Changed
            }
            MOVE => Clicked::Place,
            NEW => self.new_slot(),
            REMOVE => {
                self.slots.remove(self.picked);
                self.picked = self.picked.saturating_sub(1);
                self.category = None;
                Clicked::Changed
            }
            DONE => Clicked::Close,
            _ => Clicked::Nothing,
        }
    }

    /// A new slot (at the middle of the belt in front, empty), picked and to be placed by hand.
    fn new_slot(&mut self) -> Clicked {
        if self.slots.len() >= SLOT_ROWS.min(MAX_BODY_SLOTS) {
            return Clicked::Nothing;
        }
        let mut s: SlotDesc = bytemuck::Zeroable::zeroed();
        set_name(&mut s.name, &format!("Slot {}", self.slots.len() + 1));
        s.offset = [0.0, -0.47, 0.16];
        s.radius = SIZES[1].0;
        s.reach = REACHES[1].0;
        s.rule = RULE_VALUE;
        s.empty = EMPTY_TICK;
        s.categories = [SLOT_NO_CATEGORY; MAX_SLOT_CATEGORIES];
        self.slots.push(s);
        self.picked = self.slots.len() - 1;
        self.category = None;
        Clicked::Place
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn on_top() -> bool {
    false
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn back() {}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{hooks, mem, tiles};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// The game's menu factory (stdcall: the type) and its heap (cdecl: size; ptr).
    const FACTORY: usize = 0x0058_7D20;
    const HEAP_ALLOC: usize = 0x0040_1F00;
    /// `Menu`: its constructor (thiscall), the base vtable (15 slots: 0 the deleting destructor, 3
    /// HandleClick(id, tile), 13 GetID).
    const MENU_CTOR: usize = 0x0058_4640;
    const MENU_VTABLE: usize = 0x00A6_96DC;
    const MENU_SIZE: usize = 0x28;
    /// Opening, as the message box does (0x5BC8B0): an open one of a type (cdecl), the
    /// `InterfaceManager` (cdecl `(0, 1)`, its menu root +0x68), the depth (cdecl, ST0), the root
    /// tidied (cdecl), `Tile::ReadXML(root, path)`, the menu of a tile, `Menu::RegisterTile(menu,
    /// tile)`, a tile's value (thiscall `(tile, id)`, ST0), `Tile::Update(tile, 0.0)`,
    /// `Menu::EnableMenu(menu, 0)`; and `Menu::Close`.
    const OPEN_OF_TYPE: usize = 0x0058_9B70;
    const INTERFACE: usize = 0x0058_2160;
    const MENU_ROOT: usize = 0x68;
    const DEPTH: usize = 0x0058_8890;
    const TIDY_ROOT: usize = 0x0059_03E0;
    const READ_XML: usize = 0x0059_0420;
    const MENU_OF: usize = 0x0058_98F0;
    const REGISTER: usize = 0x0058_4880;
    const GET_FLOAT: usize = 0x0058_8BD0;
    const UPDATE: usize = 0x0058_FBA0;
    const ENABLE: usize = 0x0058_5190;
    const CLOSE: usize = 0x0058_4740;
    /// `Tile::UpdateFloat`, `Tile::UpdateString` (thiscall: tile, id, value).
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const UPDATE_STRING: usize = 0x0058_CED0;
    /// The game's menu sounds (cdecl: which): the inventory's opening and closing.
    const SOUND: usize = 0x0057_DE50;
    const SOUND_OPEN: u32 = 16;
    const SOUND_CLOSE: u32 = 17;
    /// Values: visible, depth, stacking type (no_click_past 102 or 6006 take the depth), string,
    /// user0.., and the internal one that has the game delete the menu once it's faded out.
    const VISIBLE: u32 = 0xFA1;
    const DEPTH_ID: u32 = 0xFAB;
    const STACKING: u32 = 0xFA5;
    const STRING: u32 = 0xFDE;
    const DELETE_AFTER: u32 = 6002;
    const fn user(n: u32) -> u32 {
        0xFAE + n
    }
    const TRUE: f32 = 2.0;
    const FALSE: f32 = 1.0;

    static REAL_FACTORY: AtomicUsize = AtomicUsize::new(0);
    static VTABLE: std::sync::OnceLock<[usize; 15]> = std::sync::OnceLock::new();
    static PATH: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    /// Our menu while it's there (its `Menu*`), and the clicks on it waiting for the game frame.
    static OPEN: AtomicUsize = AtomicUsize::new(0);
    static CLICKS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

    /// The deleting destructor: ours is gone (a loaded game closes every menu too).
    unsafe extern "thiscall" fn destroy(this: usize, flags: u8) -> usize {
        let _ = OPEN.compare_exchange(this, 0, Ordering::AcqRel, Ordering::Acquire);
        type Dtor = unsafe extern "thiscall" fn(usize, u8) -> usize;
        let base = mem::deref(MENU_VTABLE);
        unsafe { std::mem::transmute::<usize, Dtor>(base)(this, flags) }
    }

    unsafe extern "thiscall" fn click(_this: usize, id: i32, _tile: usize) {
        hooks::guard_void("holsters click", || CLICKS.lock().unwrap().push(id));
    }

    unsafe extern "thiscall" fn get_id(_this: usize) -> u32 {
        MENU_HOLSTERS
    }

    /// The game's factory, with ours for 1050.
    unsafe extern "stdcall" fn factory(kind: u32) -> usize {
        if kind == MENU_HOLSTERS {
            return hooks::guard("holsters menu", 0, || {
                type Alloc = unsafe extern "C" fn(usize) -> usize;
                type Ctor = unsafe extern "thiscall" fn(usize) -> usize;
                let m = unsafe { std::mem::transmute::<usize, Alloc>(HEAP_ALLOC)(MENU_SIZE) };
                if m == 0 {
                    return 0;
                }
                unsafe {
                    std::mem::transmute::<usize, Ctor>(MENU_CTOR)(m);
                    (m as *mut usize).write(VTABLE.get().unwrap().as_ptr() as usize);
                }
                m
            });
        }
        type Factory = unsafe extern "stdcall" fn(u32) -> usize;
        unsafe { std::mem::transmute::<usize, Factory>(REAL_FACTORY.load(Ordering::Acquire))(kind) }
    }

    /// The factory hooked and the menu's file written (beside the runtime's settings).
    pub fn install() {
        let mut vt = [0usize; 15];
        for (i, v) in vt.iter_mut().enumerate() {
            *v = mem::deref(MENU_VTABLE + 4 * i);
        }
        vt[0] = destroy as *const () as usize;
        vt[3] = click as *const () as usize;
        vt[13] = get_id as *const () as usize;
        let _ = VTABLE.set(vt);
        let dir = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("RustVR");
        let path = dir.join("oblivion_holsters_menu.xml");
        let written = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, xml(crate::inventory::CATEGORIES.len())));
        match written {
            Ok(()) => {
                let _ = PATH.set(std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default());
                hooks::install_into(FACTORY as *const (), factory as *const () as *const (), "the menu factory (holsters)", &REAL_FACTORY);
            }
            Err(e) => tracing::warn!("holsters: can't write the editor's menu {}: {e}", path.display()),
        }
    }

    fn update_float(tile: usize, id: u32, v: f32) {
        type F = unsafe extern "thiscall" fn(usize, u32, f32);
        if tile != 0 {
            unsafe { std::mem::transmute::<usize, F>(UPDATE_FLOAT)(tile, id, v) };
        }
    }

    fn update_string(tile: usize, id: u32, s: &str) {
        type F = unsafe extern "thiscall" fn(usize, u32, *const u8);
        if tile != 0
            && let Ok(c) = std::ffi::CString::new(s)
        {
            unsafe { std::mem::transmute::<usize, F>(UPDATE_STRING)(tile, id, c.as_ptr() as *const u8) };
        }
    }

    struct State {
        editor: Editor,
        /// The menu's tile and its tiles by name.
        tile: usize,
        named: HashMap<String, usize>,
        body: Box<BodySlots>,
        edits: Box<SlotEdits>,
        /// The runtime's asks seen (for the editor; a slot placed by hand).
        edit_seq: Option<u32>,
        placed_seq: u32,
        placing: bool,
        /// What was last written into each tile's values (only changes are written).
        shown: HashMap<(usize, u32), String>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    /// Our menu is there (and on top: it's a popup).
    pub fn on_top() -> bool {
        OPEN.load(Ordering::Acquire) != 0 && tiles::top_menu() == MENU_HOLSTERS
    }

    /// B on the editor: it closes, as Done does.
    pub fn back() {
        if OPEN.load(Ordering::Acquire) != 0 {
            CLICKS.lock().unwrap().push(DONE);
        }
    }

    fn walk(tile: usize, depth: u32, out: &mut HashMap<String, usize>) {
        if depth > 12 || out.len() > 512 {
            return;
        }
        let name = tiles::name(tile);
        if name.starts_with("hol_") {
            out.insert(name, tile);
        }
        for c in tiles::children(tile) {
            walk(c, depth + 1, out);
        }
    }

    /// Opens the editor on slot `slot`, as the game opens its message box.
    fn open(s: &mut State, slot: usize) -> bool {
        type Of = unsafe extern "C" fn(u32) -> usize;
        type Im = unsafe extern "C" fn(u32, u32) -> usize;
        type Depth = unsafe extern "C" fn() -> f32;
        type Tidy = unsafe extern "C" fn();
        type Read = unsafe extern "thiscall" fn(usize, *const u8) -> usize;
        type MenuOf = unsafe extern "thiscall" fn(usize) -> usize;
        type Id = unsafe extern "thiscall" fn(usize) -> u32;
        type Register = unsafe extern "thiscall" fn(usize, usize);
        type GetF = unsafe extern "thiscall" fn(usize, u32) -> f32;
        type Update = unsafe extern "thiscall" fn(usize, f32);
        type Enable = unsafe extern "thiscall" fn(usize, u8);
        type Sound = unsafe extern "C" fn(u32);
        type Del = unsafe extern "thiscall" fn(usize, u8) -> usize;
        let Some(path) = PATH.get() else { return false };
        unsafe {
            let old = std::mem::transmute::<usize, Of>(OPEN_OF_TYPE)(MENU_HOLSTERS);
            if old != 0 {
                std::mem::transmute::<usize, Del>(mem::deref(mem::deref(old)))(old, 1);
            }
            let im = std::mem::transmute::<usize, Im>(INTERFACE)(0, 1);
            let root = mem::deref(im + MENU_ROOT);
            if im == 0 || root == 0 {
                return false;
            }
            let depth = std::mem::transmute::<usize, Depth>(DEPTH)();
            std::mem::transmute::<usize, Tidy>(TIDY_ROOT)();
            let t = std::mem::transmute::<usize, Read>(READ_XML)(root, path.as_ptr() as *const u8);
            let m = if t != 0 { std::mem::transmute::<usize, MenuOf>(MENU_OF)(t) } else { 0 };
            if m == 0 || std::mem::transmute::<usize, Id>(mem::deref(mem::deref(m) + 13 * 4))(m) != MENU_HOLSTERS {
                tracing::warn!("holsters: the editor's menu didn't come up (tile {t:#x}, menu {m:#x}): is the factory hooked?");
                if m != 0 && mem::deref(m + 4) != 0 {
                    std::mem::transmute::<usize, Del>(mem::deref(mem::deref(m)))(m, 1);
                }
                return false;
            }
            std::mem::transmute::<usize, Register>(REGISTER)(m, t);
            let stacking = std::mem::transmute::<usize, GetF>(GET_FLOAT)(t, STACKING);
            if stacking == 6006.0 || stacking == 102.0 {
                update_float(t, DEPTH_ID, depth);
            }
            OPEN.store(m, Ordering::Release);
            s.tile = t;
            s.named.clear();
            walk(t, 0, &mut s.named);
            s.shown.clear();
            s.editor.picked = slot.min(s.editor.slots.len().saturating_sub(1));
            s.editor.category = None;
            fill(s);
            std::mem::transmute::<usize, Sound>(SOUND)(SOUND_OPEN);
            std::mem::transmute::<usize, Update>(UPDATE)(t, 0.0);
            std::mem::transmute::<usize, Update>(UPDATE)(t, 0.0);
            std::mem::transmute::<usize, Enable>(ENABLE)(m, 0);
            update_float(t, VISIBLE, TRUE);
            // (The runtime shows the picked slot's first thing at its place while it's open.)
            s.edits.editing = s.editor.picked as u32 + 1;
            crate::link::with_adapter(|a| a.write_slot_edits(&s.edits));
            tracing::info!("holsters: the editor is open on slot {slot} ({} tiles named)", s.named.len());
        }
        true
    }

    fn close(s: &mut State) {
        type Close = unsafe extern "thiscall" fn(usize);
        type Sound = unsafe extern "C" fn(u32);
        let m = OPEN.swap(0, Ordering::AcqRel);
        if m == 0 {
            return;
        }
        update_float(s.tile, DELETE_AFTER, TRUE);
        unsafe {
            std::mem::transmute::<usize, Close>(CLOSE)(m);
            std::mem::transmute::<usize, Sound>(SOUND)(SOUND_CLOSE);
        }
        s.edits.editing = 0;
        if s.placing {
            place(s, None);
        } else {
            crate::link::with_adapter(|a| a.write_slot_edits(&s.edits));
        }
        s.tile = 0;
        s.named.clear();
        tracing::info!("holsters: the editor closed");
    }

    /// A value of a tile by name, written only when it changes.
    fn set(s: &mut State, name: &str, id: u32, v: Value) {
        let Some(&tile) = s.named.get(name) else { return };
        let text = match &v {
            Value::F(f) => format!("{f}"),
            Value::S(t) => t.clone(),
        };
        if s.shown.get(&(tile, id)) == Some(&text) {
            return;
        }
        match v {
            Value::F(f) => update_float(tile, id, f),
            Value::S(t) => update_string(tile, id, &t),
        }
        s.shown.insert((tile, id), text);
    }

    enum Value {
        F(f32),
        S(String),
    }

    fn flag(b: bool) -> Value {
        Value::F(if b { TRUE } else { FALSE })
    }

    /// The menu's tiles from the editor's state.
    fn fill(s: &mut State) {
        let slots = s.editor.slots.clone();
        let picked = s.editor.picked;
        for k in 0..SLOT_ROWS {
            let row = format!("hol_slot_{k}");
            let Some(slot) = slots.get(k) else {
                set(s, &row, user(8), flag(false));
                continue;
            };
            let now = s.body.now.get(k).copied().unwrap_or_default();
            let gives = if now.item != 0 {
                crate::inventory::describe(now.item).map_or_else(String::new, |(name, count)| format!("{name} ({count})"))
            } else {
                "nothing".to_owned()
            };
            set(s, &row, user(8), flag(true));
            set(s, &row, user(10), flag(k == picked));
            set(s, &row, user(1), Value::S(slot.name().to_owned()));
            set(s, &row, user(2), Value::S(gives));
        }
        let order = s.editor.categories(picked);
        for (j, name) in crate::inventory::CATEGORIES.iter().enumerate() {
            let row = format!("hol_category_{j}");
            let place = order.iter().position(|&c| c as usize == j);
            set(s, &row, user(8), flag(!slots.is_empty()));
            set(s, &row, user(10), flag(place.is_some()));
            set(s, &row, user(1), Value::S((*name).to_owned()));
            set(s, &row, user(6), Value::S(crate::inventory::category_icon(j as u32).unwrap_or_else(|| ICONS[j].to_owned())));
            let mark = match place {
                Some(p) if s.editor.category == Some(j as u8) => format!("< {} >", p + 1),
                Some(p) => format!("{}", p + 1),
                None => String::new(),
            };
            set(s, &row, user(7), Value::S(mark));
        }
        let slot = slots.get(picked).copied();
        let label = |s: &mut State, name: &str, text: String, enabled: bool| {
            set(s, name, user(0), Value::S(text));
            set(s, name, user(1), flag(enabled));
        };
        let has = slot.is_some();
        let picked_in = s.editor.category.is_some();
        label(s, "hol_up", "Earlier".into(), has && picked_in);
        label(s, "hol_down", "Later".into(), has && picked_in);
        label(s, "hol_order", slot.map_or(String::new(), |d| RULES.get(d.rule as usize).copied().unwrap_or("").to_owned()), has);
        label(s, "hol_empty", slot.map_or(String::new(), |d| EMPTIES.get(d.empty as usize).copied().unwrap_or("").to_owned()), has);
        label(s, "hol_size", slot.map_or(String::new(), |d| SIZES[super::nearest(&SIZES, d.radius)].1.to_owned()), has);
        label(s, "hol_reach", slot.map_or(String::new(), |d| REACHES[super::nearest(&REACHES, d.reach)].1.to_owned()), has);
        label(s, "hol_equipped", slot.map_or(String::new(), |d| if d.flags & SLOTF_EQUIPPED_FIRST != 0 { "Worn first" } else { "Worn: as any" }.to_owned()), has);
        label(s, "hol_move", "Move".into(), has);
        label(s, "hol_new", "New slot".into(), slots.len() < SLOT_ROWS);
        label(s, "hol_remove", "Remove".into(), has);
        // (The game's own word for it, its message boxes' one button.)
        label(s, "hol_done", words(OK_WORD).unwrap_or_else(|| "Done".into()), true);
        let now = if s.placing {
            "Reach where you want it, and squeeze.".to_owned()
        } else if picked_in {
            "Pick it again to take it out.".to_owned()
        } else {
            String::new()
        };
        set(s, "hol_now", STRING, Value::S(now));
    }

    /// The game's own button word for a message box with one button (its "Ok").
    const OK_WORD: usize = 0x00B3_8CF0;

    fn words(at: usize) -> Option<String> {
        let s = crate::combat::read_str(mem::deref(at));
        (!s.is_empty() && s.len() < 24).then_some(s)
    }

    /// The editor's slots to the runtime.
    fn send(s: &mut State) {
        s.edits.revision = s.edits.revision.wrapping_add(1).max(1);
        let n = s.editor.slots.len().min(MAX_BODY_SLOTS);
        s.edits.count = n as u32;
        for k in 0..MAX_BODY_SLOTS {
            s.edits.slots[k] = s.editor.slots.get(k).copied().unwrap_or_else(bytemuck::Zeroable::zeroed);
        }
        crate::link::with_adapter(|a| a.write_slot_edits(&s.edits));
    }

    /// Asks the runtime to place slot `slot` by hand (None: not any more).
    fn place(s: &mut State, slot: Option<usize>) {
        s.edits.place_seq = s.edits.place_seq.wrapping_add(1);
        s.edits.place_slot = slot.map_or(u32::MAX, |k| k as u32);
        s.placing = slot.is_some();
        crate::link::with_adapter(|a| a.write_slot_edits(&s.edits));
    }

    /// Once a game frame: the runtime's asks (open the editor), the clicks on it, its tiles.
    pub fn frame() {
        let mut guard = STATE.lock().unwrap();
        let s = guard.get_or_insert_with(|| State {
            editor: Editor::default(),
            tile: 0,
            named: HashMap::new(),
            body: bytemuck::zeroed_box(),
            edits: bytemuck::zeroed_box(),
            edit_seq: None,
            placed_seq: 0,
            placing: false,
            shown: HashMap::new(),
        });
        if !crate::link::with_adapter(|a| a.read_body_slots(&mut s.body)).unwrap_or(false) {
            return;
        }
        let ask = s.body.edit_seq;
        let asked = s.edit_seq.is_some_and(|seen| seen != ask);
        s.edit_seq = Some(ask);
        let open_now = OPEN.load(Ordering::Acquire) != 0;
        if asked && crate::menus::in_world() {
            s.editor.slots = s.body.slots().to_vec();
            if !open_now || s.tile == 0 {
                let slot = s.body.edit_slot as usize;
                open(s, slot);
            } else {
                s.editor.picked = (s.body.edit_slot as usize).min(s.editor.slots.len().saturating_sub(1));
            }
        }
        if OPEN.load(Ordering::Acquire) == 0 {
            s.tile = 0;
            s.placing = false;
            CLICKS.lock().unwrap().clear();
            // (Gone without being closed here: the game closed every menu, loading a game.)
            if s.edits.editing != 0 {
                s.edits.editing = 0;
                crate::link::with_adapter(|a| a.write_slot_edits(&s.edits));
            }
            return;
        }
        // A slot placed by hand: where the runtime has it now.
        if s.body.placed_seq != s.placed_seq {
            s.placed_seq = s.body.placed_seq;
            if s.placing {
                s.placing = false;
                for (mine, theirs) in s.editor.slots.iter_mut().zip(s.body.slots()) {
                    mine.offset = theirs.offset;
                    mine.flags = (mine.flags & !SLOTF_BEHIND) | (theirs.flags & SLOTF_BEHIND);
                }
            }
        }
        for id in std::mem::take(&mut *CLICKS.lock().unwrap()) {
            let what = s.editor.click(id);
            tracing::info!("holsters: the editor's {id} clicked: {what:?} (slot {})", s.editor.picked);
            match what {
                Clicked::Changed => send(s),
                Clicked::Place => {
                    send(s);
                    let k = s.editor.picked;
                    place(s, Some(k));
                }
                Clicked::Close => {
                    close(s);
                    return;
                }
                Clicked::Nothing => {}
            }
        }
        // The slot picked shown at its place on the body.
        let picked = if s.editor.slots.is_empty() { 0 } else { s.editor.picked as u32 + 1 };
        if s.edits.editing != picked {
            s.edits.editing = picked;
            crate::link::with_adapter(|a| a.write_slot_edits(&s.edits));
        }
        fill(s);
    }
}
