//! The player's things, for the body's slots (`rustvr_protocol::inventory`): Oblivion's inventory
//! read with the game's own rows (as its inventory menu builds its list), each in a category the
//! slots draw from, with the values their orders need from the game's own data (what a potion's
//! or a food's effects restore, the value and weight the inventory shows, an arrow's damage) and
//! its model to draw in the hand (`oblivion-extract export meshes/clutter`, with the weapons').
//!
//! What a hand draws from a slot is the runtime's copy of one. When the runtime says one was used
//! (held at the mouth, up in front of the eyes, on the blade in the other hand), this does the
//! game's own use of it: food, drinks, potions and ingredients with `Actor::EquipItem`, as the
//! inventory's click does (their effects, the swallow or the chew, the potion limit and the quest
//! items it refuses); a poison put on the weapon in hand, the game's own steps without its yes/no
//! box (the touch is the yes); a scroll read and cast, the player's own scroll use. One let go
//! anywhere but a slot is dropped by the game where it came to rest (its own drop: a thing in the
//! world, to be picked up again). Either way it tells the runtime it has taken it back.
//!
//! A weapon or a torch drawn from a slot, and arrows while a bow is on, the game equips (as the
//! inventory's click does), and its own takes the copy's place in the hand: the weapon, the
//! burning torch, an arrow to nock. Arrows of the kind on the bow come out as the bow's own at once
//! (the runtime makes them from the bow's arrows, `INV_ARROW`), from whichever slot they're kept
//! in: there's no quiver of its own.
//!
//! The game's own slots: what heals at the belt in front of the off-hand hip (the sword hand
//! fights), the arrows over the right shoulder where Oblivion wears its quiver, scrolls and
//! poisons in the right chest pouch, food at the right hip.

use rustvr_protocol::*;

/// The categories the slots draw from, as the runtime knows them (their names are what the
/// player's slots remember).
pub const CATEGORIES: [&str; 14] = ["Food", "Potions", "Poisons", "Ingredients", "Scrolls", "Arrows", "Weapons", "Soul Gems", "Keys", "Books", "Sigil Stones", "Apparatus", "Misc", "Torches"];
pub const FOOD: u32 = 0;
pub const POTIONS: u32 = 1;
pub const POISONS: u32 = 2;
pub const INGREDIENTS: u32 = 3;
pub const SCROLLS: u32 = 4;
pub const ARROWS: u32 = 5;
pub const WEAPONS: u32 = 6;
pub const SOUL_GEMS: u32 = 7;
pub const KEYS: u32 = 8;
pub const BOOKS: u32 = 9;
pub const SIGIL_STONES: u32 = 10;
pub const APPARATUS: u32 = 11;
pub const MISC: u32 = 12;
pub const TORCHES: u32 = 13;

use crate::items::{AMMO, APPARATUS as APPA, BOOK, INGREDIENT, KEY, LIGHT, MISC as MISC_FORM, POTION, SIGIL_STONE, SOUL_GEM, WEAPON};

/// The category of a thing of form type `kind` (food, a poison and a scroll as the game tells
/// them); None: not one the slots hand out (apparel). A light in the inventory is one that's
/// carried: a torch.
pub fn category(kind: u8, food: bool, poison: bool, scroll: bool) -> Option<u32> {
    Some(match kind {
        POTION if poison => POISONS,
        POTION if food => FOOD,
        POTION => POTIONS,
        INGREDIENT if food => FOOD,
        INGREDIENT => INGREDIENTS,
        BOOK if scroll => SCROLLS,
        BOOK => BOOKS,
        AMMO => ARROWS,
        WEAPON => WEAPONS,
        SOUL_GEM => SOUL_GEMS,
        KEY => KEYS,
        SIGIL_STONE => SIGIL_STONES,
        APPA => APPARATUS,
        MISC_FORM => MISC,
        LIGHT => TORCHES,
        _ => return None,
    })
}

/// How using one is done, for a thing of this category.
pub fn use_flags(category: u32, kind: u8) -> u32 {
    match category {
        POISONS => INV_COAT,
        SCROLLS => INV_READ,
        ARROWS => INV_ARROW,
        _ if kind == POTION => INV_DRINK,
        _ if kind == INGREDIENT => INV_EAT,
        _ => 0,
    }
}

/// What effects restore (health, magicka, fatigue), each its magnitude over its time (an instant
/// one once): `(code, magnitude, duration)`.
pub fn restores(effects: &[([u8; 4], u32, u32)]) -> [f32; 3] {
    let mut r = [0.0; 3];
    for (code, magnitude, duration) in effects {
        let k = match code {
            b"REHE" => 0,
            b"RESP" => 1,
            b"REFA" => 2,
            _ => continue,
        };
        r[k] += *magnitude as f32 * (*duration).max(1) as f32;
    }
    r
}

/// A slot of the game's own.
fn slot(name: &str, at: [f32; 3], radius: f32, reach: f32, categories: &[u32], rule: u32, flags: u32) -> SlotDesc {
    let mut d: SlotDesc = bytemuck::Zeroable::zeroed();
    set_name(&mut d.name, name);
    d.offset = at;
    d.radius = radius;
    d.reach = reach;
    d.rule = rule;
    d.empty = EMPTY_TICK;
    d.flags = flags;
    d.categories = [SLOT_NO_CATEGORY; MAX_SLOT_CATEGORIES];
    for (k, c) in categories.iter().enumerate() {
        d.categories[k] = *c as u8;
    }
    d
}

/// Oblivion's own slots, for a right-handed player (the runtime mirrors them for a left hand).
/// Clear of the holsters the equipped weapon and shield hang in (the left hip, the back).
pub fn defaults() -> Vec<SlotDesc> {
    vec![
        // What heals, at the belt in front of the left hip: the off hand takes it while the sword
        // hand fights.
        slot("Healing", [-0.13, -0.47, 0.14], 0.075, 0.035, &[POTIONS, FOOD], RULE_HEALING, 0),
        // The arrows, the equipped ones first, over the right shoulder where Oblivion wears its
        // quiver (its arrows at the right shoulder). The biggest, from above the shoulder down
        // behind it: a hand reaching behind the head can't see where it is.
        slot("Arrows", [0.14, -0.12, -0.12], 0.13, 0.13, &[ARROWS], RULE_STRONGEST, SLOTF_EQUIPPED_FIRST | SLOTF_BEHIND),
        // Scrolls and poisons high on the chest, on the right.
        slot("Scrolls", [0.13, -0.28, 0.12], 0.07, 0.03, &[SCROLLS, POISONS], RULE_VALUE, 0),
        // Food at the right hip, what restores the most fatigue first (Oblivion's food mostly does).
        slot("Food", [0.19, -0.55, 0.02], 0.09, 0.04, &[FOOD], RULE_FATIGUE, 0),
    ]
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem};
    use glam::Vec3;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// The inventory's rows (thiscall): how many there may be (`refr`, 0), and the `n`th as a
    /// fresh `EntryData` (`refr`, n, 0) or none, freed with `EntryData::Cleanup` then the game's
    /// heap. An entry: its extra lists +0 (a node: the list +0, next +4), its count +4, its form +8.
    const ROW_COUNT: usize = 0x004D_8950;
    const ROW: usize = 0x004D_88F0;
    const ENTRY_CLEANUP: usize = 0x0048_4470;
    const HEAP_FREE: usize = 0x0040_1F20;
    const ENTRY_LISTS: usize = 0x0;
    const ENTRY_COUNT: usize = 0x4;
    const ENTRY_FORM: usize = 0x8;
    /// `EntryData::IsEquipped(entry, 0)`, `GetStatusFlags(entry)` (1 stolen, 2 enchanted), the
    /// inventory menu's own test of what it shows `(entry, player, 0, 1, 1, 0)`, and the numbers it
    /// shows: value `(entry, 0, 0, 0)` and weight `(entry, 0, 0, 0)` (in ST0).
    const IS_EQUIPPED: usize = 0x0048_4E80;
    const STATUS: usize = 0x0048_5C00;
    const SHOWN: usize = 0x0048_54F0;
    const VALUE: usize = 0x0048_8E50;
    const WEIGHT: usize = 0x0048_5260;
    const STOLEN: u32 = 1;
    const ENCHANTED: u32 = 2;
    /// Gold isn't a thing in the hand.
    const GOLD: u32 = 0xF;
    /// A form's model (cdecl, relative to `meshes\`).
    const MODEL_PATH: usize = 0x0046_D3F0;
    /// Potions and ingredients: their effects (an `EffectItemList`: its first node inline at +4,
    /// `{EffectItem*, next}`; an effect's code +0, magnitude +4, duration +0xC), their flags (bit 1
    /// food), and the game's poison test `EffectItemList::IsPoison` (thiscall, the list).
    const EFFECTS: usize = 0x30;
    const ALCH_FLAGS: usize = 0x7C;
    const FOOD_FLAG: u8 = 2;
    const IS_POISON: usize = 0x0041_4EB0;
    /// What eaten food restores, whatever its record says (`fWortcraftFatigueMag`), for a second.
    const FOOD_MAGNITUDE: usize = 0x00B3_7F90;
    /// Books: flags (bit 0 a scroll) and the enchantment a scroll casts.
    const BOOK_FLAGS: usize = 0x88;
    const BOOK_ENCHANTMENT: usize = 0x64;
    /// An arrow's and a weapon's damage, a soul gem's capacity, a weapon's type (4 a staff, 5 a bow).
    const AMMO_DAMAGE: usize = 0x78;
    const WEAP_DAMAGE: usize = 0x8C;
    const SOUL_CAPACITY: usize = 0x71;
    const WEAP_TYPE: usize = 0x90;
    const STAFF: u8 = 4;
    /// The player's things' changes (`GetOrCreateExtraContainerChanges`, cdecl), and how many of a
    /// thing there are (`InventoryChanges::GetItemCount`, thiscall: the changes, the form).
    const CHANGES: usize = 0x0048_5E00;
    const ITEM_COUNT: usize = 0x0048_69C0;
    /// `Actor::EquipItem(actor, form, count, extra list, 0, 0)`: what the inventory's click does.
    const EQUIP: usize = 0x005F_AEA0;
    /// Poisoning (the game's own yes, 0x666890): the weapon's entry (the process's virtual +0xEC,
    /// `(process, 1)`), already poisoned (`EntryData`'s 0x484DF0), poisoned (0x484E20, thiscall
    /// on the weapon's entry with the poison), the poison taken out (the reference's `RemoveItem`,
    /// virtual +0x100), "<poison> used." (0x57ADD0, cdecl `(text, seconds, 0, 0)`).
    const PROCESS: usize = 0x58;
    const VT_WEAPON_ENTRY: usize = 0xEC;
    const POISONED: usize = 0x0048_4DF0;
    const SET_POISON: usize = 0x0048_4E20;
    const VT_REMOVE_ITEM: usize = 0x100;
    const MESSAGE: usize = 0x0057_ADD0;
    /// The game's own words: "used." and the poison's refusals (no weapon, already poisoned, a
    /// staff); `char*`s.
    const USED: usize = 0x00B3_8BA8;
    const NO_WEAPON: usize = 0x00B3_88B0;
    const ALREADY: usize = 0x00B3_88B8;
    const NOT_STAFFS: usize = 0x00B3_88D0;
    /// The player's scroll use (its virtual +0x2D0, `(book, 0)`): one removed, its enchantment cast.
    const VT_USE_SCROLL: usize = 0x2D0;
    /// `Actor::PlayItemSound(actor, form, up, use)` (thiscall): a thing of its kind taken out, put
    /// away or used, as the inventory sounds it.
    const ITEM_SOUND: usize = 0x005E_99C0;
    /// The inventory's icon of a thing (cdecl: its form; under `Icons\`).
    const ICON: usize = 0x005C_0C50;
    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const QUEST_ITEM: u32 = 0x400;
    const RTTI_FULL_NAME: usize = 0x00B0_3298;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A0_0000;

    /// Seconds between looks at the inventory; and how often it's published anyway (a runtime that
    /// has come back).
    const SCAN_EVERY: f64 = 0.5;
    const REPUBLISH: f64 = 2.0;
    /// A thing let go and moving slower than this (m/s) for this long (s) has come to rest; it's
    /// dropped after this long whatever.
    const REST_SPEED: f32 = 0.06;
    const REST_TIME: f64 = 0.3;
    const MAX_FALL: f64 = 4.0;

    /// One of the inventory's rows, while it's held (freed after).
    struct Row {
        form: usize,
        /// The row's own extra list (0: the plain stack).
        list: usize,
        count: i32,
        entry: usize,
    }

    /// Each of the player's rows the inventory menu would show (not gold), each freed after `f`.
    fn each_row(player: usize, mut f: impl FnMut(&Row)) {
        type Count = unsafe extern "thiscall" fn(usize, u8) -> i32;
        type Get = unsafe extern "thiscall" fn(usize, i32, u8) -> usize;
        type Cleanup = unsafe extern "thiscall" fn(usize);
        type Free = unsafe extern "C" fn(usize);
        type Shown = unsafe extern "thiscall" fn(usize, usize, u8, u8, u8, u8) -> u8;
        let n = unsafe { std::mem::transmute::<usize, Count>(ROW_COUNT)(player, 0) };
        for i in 0..n.clamp(0, 1024) {
            let entry = unsafe { std::mem::transmute::<usize, Get>(ROW)(player, i, 0) };
            if entry == 0 {
                continue;
            }
            let form = mem::deref(entry + ENTRY_FORM);
            let node = mem::deref(entry + ENTRY_LISTS);
            let row = Row { form, list: if node != 0 { mem::deref(node) } else { 0 }, count: mem::at::<i32>(entry + ENTRY_COUNT).unwrap_or(0), entry };
            let shown = form != 0 && mem::at::<u32>(form + FORM_ID) != Some(GOLD) && unsafe { std::mem::transmute::<usize, Shown>(SHOWN)(entry, player, 0, 1, 1, 0) } != 0;
            if shown && row.count > 0 {
                f(&row);
            }
            unsafe {
                std::mem::transmute::<usize, Cleanup>(ENTRY_CLEANUP)(entry);
                std::mem::transmute::<usize, Free>(HEAP_FREE)(entry);
            }
        }
    }

    /// How many of a thing the player has.
    fn count_of(player: usize, form: usize) -> i32 {
        type Changes = unsafe extern "C" fn(usize) -> usize;
        type Count = unsafe extern "thiscall" fn(usize, usize) -> i32;
        let changes = unsafe { std::mem::transmute::<usize, Changes>(CHANGES)(player) };
        if changes == 0 {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, Count>(ITEM_COUNT)(changes, form) }
    }

    fn name_of(form: usize) -> String {
        let n = crate::items::cast(form, RTTI_FULL_NAME);
        if n != 0 { crate::combat::read_str(mem::deref(n + 4)) } else { String::new() }
    }

    /// A game string (a `char*`'s address).
    fn words(at: usize) -> String {
        crate::combat::read_str(mem::deref(at))
    }

    fn message(text: &str) {
        type Message = unsafe extern "C" fn(*const u8, f32, u32, u32) -> u8;
        if let Ok(t) = std::ffi::CString::new(text) {
            unsafe { std::mem::transmute::<usize, Message>(MESSAGE)(t.as_ptr() as *const u8, 2.0, 0, 0) };
        }
    }

    /// A potion's or an ingredient's effects: (code, magnitude, duration); an ingredient's first
    /// only (eaten, that's the one the game applies).
    fn effects(form: usize, first: bool) -> Vec<([u8; 4], u32, u32)> {
        let mut out = Vec::new();
        let mut node = form + EFFECTS + 4;
        let mut guard = 0;
        while node != 0 && guard < 16 {
            guard += 1;
            let e = mem::deref(node);
            node = mem::deref(node + 4);
            if e == 0 {
                continue;
            }
            if let (Some(code), Some(magnitude), Some(duration)) = (mem::at::<[u8; 4]>(e), mem::at::<u32>(e + 4), mem::at::<u32>(e + 0xC)) {
                out.push((code, magnitude, duration));
            }
            if first {
                break;
            }
        }
        out
    }

    /// Its model's name and its box (model space), if it has been extracted to draw.
    fn model_of(form: usize, models: &mut HashMap<String, Option<([f32; 3], [f32; 3])>>) -> Option<(String, [f32; 3], [f32; 3])> {
        type Path = unsafe extern "C" fn(usize) -> usize;
        let path = crate::combat::read_str(unsafe { std::mem::transmute::<usize, Path>(MODEL_PATH)(form) });
        if path.is_empty() {
            return None;
        }
        let key = oblivion_assets::export::model_key(&path);
        let bounds = *models.entry(key.clone()).or_insert_with(|| match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
            Ok(m) if !m.meshes.is_empty() => {
                let (lo, hi) = (Vec3::from_array(m.bounds_min), Vec3::from_array(m.bounds_max));
                Some((((lo + hi) * 0.5).to_array(), ((hi - lo) * 0.5).to_array()))
            }
            Ok(_) => None,
            Err(e) => {
                tracing::info!("inventory: {path} ({key}) can't be drawn: {e:#}");
                None
            }
        });
        bounds.map(|(c, h)| (format!("oblivion/{key}"), c, h))
    }

    /// A copy drawn from a slot: its thing's form, its uses done, since when it has been let go (and
    /// still), and whether the game has taken it back (used, or dropped).
    #[derive(Clone, Copy)]
    struct Out {
        form: usize,
        uses: u32,
        let_go: Option<(f64, Option<f64>)>,
        taken: bool,
    }

    /// The game's own sound of taking a thing of its kind out (`up`) or putting it away
    /// (`Actor::PlayItemSound`, the inventory's own).
    fn item_sound(player: usize, form: usize, up: bool) {
        type Sound = unsafe extern "thiscall" fn(usize, usize, u8, u8);
        unsafe { std::mem::transmute::<usize, Sound>(ITEM_SOUND)(player, form, up as u8, 0) };
    }

    struct State {
        inv: Box<Inventory>,
        /// Each stack's number: (form, its own extra list).
        ids: HashMap<(usize, usize), u32>,
        next_id: u32,
        /// When each thing was first seen (bigger: newer).
        seen: HashMap<u32, u32>,
        next_seen: u32,
        models: HashMap<String, Option<([f32; 3], [f32; 3])>>,
        next_scan: f64,
        published: f64,
        /// Each copy drawn from a slot, by its object.
        drawn: HashMap<u32, Out>,
        states: Box<ObjectStates>,
        logged: usize,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State {
            inv: bytemuck::zeroed_box(),
            ids: HashMap::new(),
            next_id: 1,
            seen: HashMap::new(),
            next_seen: 1,
            models: HashMap::new(),
            next_scan: 0.0,
            published: -100.0,
            drawn: HashMap::new(),
            states: bytemuck::zeroed_box(),
            logged: usize::MAX,
        }))
    }

    /// One row as the runtime has it.
    fn item(s: &mut State, r: &Row) -> Option<InvItem> {
        type Flag = unsafe extern "thiscall" fn(usize, u8) -> u8;
        type Status = unsafe extern "thiscall" fn(usize) -> u32;
        type Value = unsafe extern "thiscall" fn(usize, u8, usize, u8) -> f64;
        type Weight = unsafe extern "thiscall" fn(usize, u32, u32, u32) -> f64;
        type Poison = unsafe extern "thiscall" fn(usize) -> u8;
        let form = r.form;
        let kind = mem::at::<u8>(form + FORM_TYPE)?;
        let food = matches!(kind, POTION | INGREDIENT) && mem::at::<u8>(form + ALCH_FLAGS).is_some_and(|f| f & FOOD_FLAG != 0);
        let poison = kind == POTION && unsafe { std::mem::transmute::<usize, Poison>(IS_POISON)(form + EFFECTS) } != 0;
        let scroll = kind == BOOK && mem::at::<u8>(form + BOOK_FLAGS).is_some_and(|f| f & 1 != 0) && mem::deref(form + BOOK_ENCHANTMENT) != 0;
        let category = category(kind, food && !poison, poison, scroll)?;
        let (model, center, half) = model_of(form, &mut s.models)?;
        let key = mem::at::<u32>(form + FORM_ID)?;
        let next_id = &mut s.next_id;
        let id = *s.ids.entry((form, r.list)).or_insert_with(|| {
            let id = *next_id;
            *next_id += 1;
            id
        });
        let next_seen = &mut s.next_seen;
        let newest = *s.seen.entry(key).or_insert_with(|| {
            *next_seen += 1;
            *next_seen
        });
        let status = unsafe { std::mem::transmute::<usize, Status>(STATUS)(r.entry) };
        let equipped = unsafe { std::mem::transmute::<usize, Flag>(IS_EQUIPPED)(r.entry, 0) } != 0;
        // (The weapon and the torch on are the hands' own already, in their holsters: not the
        // slots' things.)
        if equipped && matches!(kind, WEAPON | LIGHT) {
            return None;
        }
        let quest = mem::at::<u32>(form + FORM_FLAGS).is_some_and(|f| f & QUEST_ITEM != 0);
        let mut it: InvItem = bytemuck::Zeroable::zeroed();
        it.id = id;
        it.category = category;
        it.count = r.count as u32;
        it.flags = use_flags(category, kind)
            | if equipped { INV_EQUIPPED } else { 0 }
            | if status & STOLEN != 0 { INV_STOLEN } else { 0 }
            | if status & ENCHANTED != 0 { INV_ENCHANTED } else { 0 }
            | if quest { INV_QUEST } else { 0 };
        it.key = key;
        it.newest = newest;
        it.value = unsafe { std::mem::transmute::<usize, Value>(VALUE)(r.entry, 0, 0, 0) } as f32;
        it.weight = unsafe { std::mem::transmute::<usize, Weight>(WEIGHT)(r.entry, 0, 0, 0) } as f32;
        it.restores = match kind {
            POTION if !poison => restores(&effects(form, false)),
            // Eaten, an ingredient applies its first effect; food at the game's own magnitude for
            // a second, whatever its record says.
            INGREDIENT => {
                let mut first = effects(form, true);
                if food {
                    let m = mem::at::<f32>(FOOD_MAGNITUDE).filter(|m| m.is_finite() && (0.0..1000.0).contains(m)).unwrap_or(5.0);
                    for e in &mut first {
                        (e.1, e.2) = (m.round() as u32, 1);
                    }
                }
                restores(&first)
            }
            _ => [0.0; 3],
        };
        it.power = match kind {
            AMMO => mem::at::<u16>(form + AMMO_DAMAGE).unwrap_or(0) as f32,
            WEAPON => mem::at::<u16>(form + WEAP_DAMAGE).unwrap_or(0) as f32,
            SOUL_GEM => mem::at::<u8>(form + SOUL_CAPACITY).unwrap_or(0) as f32,
            _ => 0.0,
        };
        // (The game's weights are about pounds: a kilogram is two.)
        it.mass = (it.weight * 0.45).clamp(0.05, 5.0);
        it.center = center;
        it.half_extents = half;
        set_name(&mut it.name, &name_of(form));
        set_name(&mut it.model, &model);
        // What the one drawn from a slot sounds like: the inventory's sound of a thing of its kind.
        set_name(&mut it.sound, &crate::sounds::kind(form).unwrap_or_default());
        Some(it)
    }

    /// The inventory now, as the runtime has it (the taken-back list kept).
    fn scan(s: &mut State, player: usize) {
        let mut out = Vec::new();
        each_row(player, |r| {
            if out.len() < MAX_INV_ITEMS
                && let Some(i) = item(s, r)
            {
                out.push(i);
            }
        });
        s.inv.count = out.len() as u32;
        s.inv.items.fill(bytemuck::Zeroable::zeroed());
        s.inv.items[..out.len()].copy_from_slice(&out);
        s.inv.category_count = CATEGORIES.len() as u32;
        for (k, c) in CATEGORIES.iter().enumerate() {
            set_name(&mut s.inv.categories[k].name, c);
        }
        let d = defaults();
        s.inv.default_count = d.len() as u32;
        s.inv.defaults[..d.len()].copy_from_slice(&d);
    }

    fn publish(s: &mut State, now: f64) {
        if crate::link::with_adapter(|a| a.write_inventory(&s.inv)).unwrap_or(false) {
            s.published = now;
            if s.logged != s.inv.count as usize {
                s.logged = s.inv.count as usize;
                let names: Vec<String> = s.inv.items().iter().take(12).map(|i| format!("{} x{} ({})", i.name(), i.count, CATEGORIES.get(i.category as usize).unwrap_or(&"?"))).collect();
                tracing::info!("inventory: {} things for the slots: {}", s.inv.count, names.join(", "));
            }
        }
    }

    /// The row of stack `id` now: (form, its extra list).
    fn stack(s: &State, id: u32) -> Option<(usize, usize)> {
        s.ids.iter().find(|(_, v)| **v == id).map(|(k, _)| *k)
    }

    /// The game's own use of one of stack `id` (`flags`: how). True when it took one.
    fn use_one(player: usize, form: usize, list: usize, flags: u32) -> bool {
        let before = count_of(player, form);
        let name = name_of(form);
        if flags & INV_COAT != 0 {
            poison(player, form, list, &name);
        } else if flags & INV_READ != 0 {
            type UseScroll = unsafe extern "thiscall" fn(usize, usize, usize) -> u32;
            let f = mem::deref(mem::deref(player) + VT_USE_SCROLL);
            if CODE.contains(&f) {
                unsafe { std::mem::transmute::<usize, UseScroll>(f)(player, form, 0) };
            }
        } else {
            type Equip = unsafe extern "thiscall" fn(usize, usize, i32, usize, u8, u8);
            unsafe { std::mem::transmute::<usize, Equip>(EQUIP)(player, form, 1, list, 0, 0) };
        }
        let after = count_of(player, form);
        tracing::info!("inventory: {name} used ({:#x}): {before} -> {after}", flags);
        after < before
    }

    /// A poison onto the weapon in hand, as the game's own yes does (its refusals said as notices).
    fn poison(player: usize, form: usize, list: usize, name: &str) {
        type Entry = unsafe extern "thiscall" fn(usize, u8) -> usize;
        type Poisoned = unsafe extern "thiscall" fn(usize) -> usize;
        type SetPoison = unsafe extern "thiscall" fn(usize, usize);
        type Remove = unsafe extern "thiscall" fn(usize, usize, usize, u32, u32, u32, usize, usize, usize, u32, u32) -> usize;
        let process = mem::deref(player + PROCESS);
        let get = mem::deref(mem::deref(process) + VT_WEAPON_ENTRY);
        let entry = if process != 0 && CODE.contains(&get) { unsafe { std::mem::transmute::<usize, Entry>(get)(process, 1) } } else { 0 };
        let weapon = mem::deref(entry + ENTRY_FORM);
        if entry == 0 || mem::at::<u8>(weapon + FORM_TYPE) != Some(WEAPON) {
            return message(&words(NO_WEAPON));
        }
        if unsafe { std::mem::transmute::<usize, Poisoned>(POISONED)(entry) } != 0 {
            return message(&words(ALREADY));
        }
        if mem::at::<u8>(weapon + WEAP_TYPE) == Some(STAFF) {
            return message(&words(NOT_STAFFS));
        }
        message(&format!("{name} {}", words(USED)));
        let remove = mem::deref(mem::deref(player) + VT_REMOVE_ITEM);
        if !CODE.contains(&remove) {
            return;
        }
        unsafe {
            std::mem::transmute::<usize, SetPoison>(SET_POISON)(entry, form);
            std::mem::transmute::<usize, Remove>(remove)(player, form, list, 1, 0, 0, 0, 0, 0, 1, 0);
        }
    }

    /// A weapon, arrows or a torch drawn from a slot equipped, as the inventory's click equips it
    /// (its own sound; arrows the whole stack, as the click does); its copy `copy` in the hand is to
    /// be its place, the game's own taking it (`crate::weapons`, `crate::archery`, `crate::torch`).
    /// True once the game has it on.
    fn equip(player: usize, form: usize, list: usize, copy: u32, kind: u8) -> bool {
        type Equip = unsafe extern "thiscall" fn(usize, usize, i32, usize, u8, u8);
        let count = if kind == AMMO { crate::archery::inventory_count(form).unwrap_or(1).max(1) as i32 } else { 1 };
        unsafe { std::mem::transmute::<usize, Equip>(EQUIP)(player, form, count, list, 0, 0) };
        let on = match kind {
            WEAPON => crate::weapons::equipped().is_some_and(|e| e.form == form),
            AMMO => crate::archery::equipped_arrows().is_some_and(|a| a.form == form),
            _ => crate::torch::equipped().is_some_and(|(f, _)| f == form),
        };
        if on {
            match kind {
                WEAPON => crate::weapons::replacing(form, copy),
                AMMO => crate::archery::replacing(form, copy),
                _ => crate::torch::replacing(form, copy),
            }
        }
        tracing::info!("inventory: {} drawn from a slot and equipped: {on}", name_of(form));
        on
    }

    /// One of a stack dropped by the game where its copy came to rest (`at`: the runtime's world).
    fn drop_one(player: usize, form: usize, list: usize, at: Vec3) -> bool {
        type Remove = unsafe extern "thiscall" fn(usize, usize, usize, u32, u32, u32, usize, *const f32, usize, u32, u32) -> usize;
        let remove = mem::deref(mem::deref(player) + VT_REMOVE_ITEM);
        if !CODE.contains(&remove) {
            return false;
        }
        let before = count_of(player, form);
        let g = camera::basis().point_to_game(at).to_array();
        // (Facing as the player does: the game's own drop's default.)
        unsafe { std::mem::transmute::<usize, Remove>(remove)(player, form, list, 1, 0, 1, 0, g.as_ptr(), 0, 1, 0) };
        let after = count_of(player, form);
        tracing::info!("inventory: {} dropped at {at:.2?}: {before} -> {after}", name_of(form));
        after < before
    }

    /// A stack's name and count as the inventory last had it.
    pub fn describe(id: u32) -> Option<(String, u32)> {
        with_state(|s| s.inv.item(id).map(|i| (i.name().to_owned(), i.count)))
    }

    /// The game's own icon (`Icons\...`) of a thing the player has in a category, for the
    /// holsters' editor.
    pub fn category_icon(category: u32) -> Option<String> {
        type Icon = unsafe extern "C" fn(usize) -> usize;
        with_state(|s| {
            let id = s.inv.items().iter().find(|i| i.category == category)?.id;
            let (form, _) = stack(s, id)?;
            let path = crate::combat::read_str(unsafe { std::mem::transmute::<usize, Icon>(ICON)(form) });
            (!path.is_empty()).then(|| format!("Icons\\{path}"))
        })
    }

    /// Once a game frame (where its scripts run): the copies drawn from the slots used or dropped
    /// as the runtime says; the inventory looked at, and published when it changes.
    pub fn frame(now: f64) {
        let player = engine::player();
        if player == 0 || !crate::menus::in_world() {
            return;
        }
        with_state(|s| {
            let mut changed = false;
            if crate::link::with_adapter(|a| a.read_object_states(&mut s.states)).unwrap_or(false) {
                let n = (s.states.count as usize).min(MAX_OBJECTS);
                let mine: Vec<ObjectState> = s.states.states[..n].iter().filter(|st| st.flags & OS_INVENTORY != 0).copied().collect();
                // Gone without the game taking it: put back in a slot (the game's own sound of
                // putting a thing of its kind away).
                let gone: Vec<(u32, Out)> = s.drawn.iter().filter(|(id, _)| !mine.iter().any(|st| st.id == **id)).map(|(id, d)| (*id, *d)).collect();
                for (id, d) in gone {
                    s.drawn.remove(&id);
                    if !d.taken {
                        item_sound(player, d.form, false);
                    }
                }
                for st in mine {
                    let Some((form, list)) = stack(s, st.parent) else { continue };
                    // Just drawn: the game's own sound of taking a thing of its kind out.
                    let d = *s.drawn.entry(st.id).or_insert_with(|| {
                        item_sound(player, form, true);
                        Out { form, uses: 0, let_go: None, taken: false }
                    });
                    if d.taken {
                        continue;
                    }
                    let (uses, let_go) = (d.uses, d.let_go);
                    let flags = s.inv.item(st.parent).map_or(0, |i| i.flags);
                    let mut take = false;
                    if st.shots > uses {
                        take = use_one(player, form, list, flags);
                    }
                    // A weapon, a torch, or arrows while a bow is on, in a hand: the game equips it,
                    // and its own takes the copy's place in that hand (`crate::weapons`,
                    // `crate::torch`, `crate::archery`: an arrow to nock; the runtime lets the copy
                    // go then). Arrows with no bow on are things like any other.
                    let mut equipped = false;
                    let kind = mem::at::<u8>(form + FORM_TYPE).unwrap_or(0);
                    let bow_on = crate::weapons::equipped().is_some_and(|e| e.kind == crate::archery::BOW);
                    if !take && st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0 && (matches!(kind, WEAPON | LIGHT) || (kind == AMMO && bow_on)) {
                        equipped = equip(player, form, list, st.id, kind);
                        take = equipped;
                    }
                    let free = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT | OS_PULLED | OS_TRACKING) == 0;
                    let mut let_go = if free { Some(let_go.unwrap_or((now, None))) } else { None };
                    if let (false, Some((since, still))) = (take, let_go) {
                        let slow = Vec3::from_array(st.lin_vel).length() < REST_SPEED;
                        let still = if slow { Some(still.unwrap_or(now)) } else { None };
                        let rest = still.is_some_and(|t| now - t > REST_TIME) || now - since > MAX_FALL;
                        let_go = Some((since, still));
                        if rest {
                            take = drop_one(player, form, list, Vec3::from_array(st.pose.pos));
                            if !take {
                                let_go = Some((now, None));
                            }
                        }
                    }
                    s.drawn.insert(st.id, Out { form, uses: st.shots, let_go, taken: take });
                    if take {
                        if !equipped {
                            s.inv.push_taken(st.id);
                        }
                        changed = true;
                    }
                }
            }
            if changed || now >= s.next_scan {
                s.next_scan = now + SCAN_EVERY;
                let before: Vec<u8> = bytemuck::bytes_of(&*s.inv).to_vec();
                scan(s, player);
                if changed || bytemuck::bytes_of(&*s.inv) != before.as_slice() || now - s.published > REPUBLISH {
                    publish(s, now);
                }
            }
        });
    }
}
