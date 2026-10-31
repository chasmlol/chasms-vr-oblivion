//! The player's spells, as the game has them, for the runtime's hands ([`Spells`]), and what the
//! hands do with them made the game's own way (`docs/research/spells.md`; the engine's facts in
//! `docs/research/spells-engine-notes.md`).
//!
//! * **The table** ([`table`]): the spells, powers and lesser powers on the player's list, in the
//!   magic menu's order (powers, lesser powers, then spells, each by name), each as the game has it:
//!   how it's delivered (its farthest range), what it costs the player now (the game's cost for
//!   them, their skill and luck in it), whether the game would cast it now (its own check: magicka,
//!   silence, mastery, a power used today, a bound item already out, a ranged spell under water),
//!   its mastery level, its school, its icon (the magic menu's: its costliest effect's), its glow
//!   (its main effect's light), whether it harms (a hostile effect), whether it's on a quick key, and
//!   which is selected. How long a hand charges it is the game's cast animation's time to its release
//!   (Hit: self 0.37 s, touch 0.27 s, target 0.4 s), and how long it waits after it the rest of that
//!   animation. With the player's magicka as the HUD's bar has it, the interface's font for the
//!   wheel, and each pick the player makes in the game's own interface (its magic menu, a quick key).
//! * **A cast** ([`cast`]): the game's own checked cast (0x69BEC0, as its cast animation's release
//!   makes it: the check with its message and its school's failure sound, the effects and the skill
//!   use as they land, the projectile, the cost, the fatigue, a power's day, invisibility dispelled),
//!   with the spell made the caster's item for it and, for a spell that throws, its main effect's
//!   casting light for the bolt to carry (as the casting art hands it over). The projectile leaves
//!   the palm along the hand's aim (the spawn hooked) with the player faced that way while it's
//!   made; a touch spell goes into the creature the hand touched (the player's touch pick hooked),
//!   else the game's own touch takes what's in its reach along the aim (a creature, else what the
//!   eyes are on: a door). The game's rule against summoning in the water is kept, as its release
//!   keeps it.
//! * **Bound weapons and shields** ([`conjured_into`]): a spell that makes one, cast from a hand, has
//!   it made in that hand: the game equips it as it does, and `crate::weapons` gives it to the
//!   runtime in the hand that cast it rather than at the hip or over the shoulder.
//! * **A pick** ([`select`]): the game's own selection (0x664700, as the magic menu makes it, a scroll
//!   put down first): the HUD's icon, the menu's sound.
//! * **A failure** ([`failed`]): the trigger pulled on a spell the game won't cast now: its message
//!   and its school's failure sound, as the start of its casting animation gives them.
//! * **Telekinesis** ([`telekinesis_rules`]): while the game's Telekinesis is on the player (its
//!   actor value, its magnitude in feet), the hands' telekinesis takes the gravity gloves' place
//!   (the runtime's own): point an open hand at a thing within the spell's
//!   reach, grip to catch it, and it floats where the hand points; the trigger pulls it in, a flick
//!   throws it. Things only, not creatures, as the game's own moves no one. When the spell ends,
//!   what's held falls and the gloves are back.
//! * **A staff** ([`cast_staff`]): the game's staff cast (0x5E4920: the enchantment's cost from the
//!   staff's charge, cast, or "not enough charge"), from its tip.

use crate::camera;
use crate::engine;
use crate::mem;
use glam::Vec3;
use rustvr_protocol::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// `TESForm::refID`; `TESObjectREFR`'s base form.
const FORM_ID: usize = 0x0C;
const BASE_FORM: usize = 0x1C;
/// `TESActorBase::spellList` entries (`TESSpellList` at +0x54, its list at +4).
const SPELL_LIST: usize = 0x58;
/// `SpellItem`: its `MagicItem` (the name at +4 of it, its `EffectItemList` at +0x0C of it, the
/// list's first node at +0x10), then its type at +0x34 (0 spell, 2 power, 3 lesser power).
const MAGIC_ITEM: usize = 0x18;
const ITEM_EFFECTS: usize = 0x0C;
const SPELL_TYPE: usize = 0x34;
/// `EffectItem`: its range (0 self, 1 touch, 2 target), its `EffectSetting`.
const EFFECT_RANGE: usize = 0x10;
const EFFECT_SETTING: usize = 0x1C;
/// `EffectSetting`: its icon (under `textures\menus\icons\`), its flags (hostile; disabled), its
/// school, its light (a LIGH: its colour at +0x78), its casting sound.
const SETTING_ICON: usize = 0x48;
const SETTING_FLAGS: usize = 0x58;
const SETTING_SCHOOL: usize = 0x64;
const SETTING_LIGHT: usize = 0x70;
const LIGHT_COLOUR: usize = 0x78;
const HOSTILE: u32 = 0x1;
const DISABLED: u32 = 0x40_0000;
/// An effect that makes a bound weapon, and one that makes bound armour (a bound shield is the one
/// whose code is `BASH`).
const BOUND_WEAPON: u32 = 0x1_0000;
const BOUND_ARMOUR: u32 = 0x2_0000;
const BOUND_SHIELD: u32 = u32::from_le_bytes(*b"BASH");
/// A bound thing the hand conjured is put in it if the game equips it this soon after (seconds).
const CONJURE_WITHIN: f64 = 3.0;
/// `EffectSetting`'s data: a bound effect's item (the weapon or armour it equips).
const SETTING_ITEM: usize = 0x60;
/// The types a hand casts.
const SPELL: u32 = 0;
const POWER: u32 = 2;
const LESSER_POWER: u32 = 3;
/// `EffectItemList` (thiscall on it): its costliest effect of a range (3: any) (0x4152C0); its
/// mastery level (vtable +8); its cost for an actor (vtable +0, on the FPU); whether it summons
/// (fastcall, 0x414CD0); its school's skill (0x415360).
const COSTLIEST: usize = 0x0041_52C0;
const VT_LIST_COST: usize = 0x00;
const VT_LIST_LEVEL: usize = 0x08;
const SUMMONS: usize = 0x0041_4CD0;
const SCHOOL_SKILL: usize = 0x0041_5360;
/// `MagicItem` main effect (thiscall, its farthest range's costliest), as the casting art picks it.
const MAIN_EFFECT: usize = 0x0041_9B30;
/// The player's `MagicCaster` (`Actor` +0x5C) and its slots: apply the cost, is it usable (item,
/// &skill, &failure, use the base magicka), its magic node, set the item being cast; its casting
/// light (+4) and state (+8: 7 when it wouldn't cast).
const MAGIC_CASTER: usize = 0x5C;
const VT_USABLE: usize = 0x1C;
const VT_MAGIC_NODE: usize = 0x24;
const VT_SET_ITEM: usize = 0x34;
const CASTER_LIGHT: usize = 0x04;
/// `NiAVObject`'s world translation.
const NODE_WORLD_POS: usize = 0x88;
const CASTER_STATE: usize = 0x08;
const CANT_CAST: u32 = 7;
/// The checked cast (thiscall on the caster, the source object: 0), the projectile spawn (cdecl, 12
/// arguments: the caster, 0, the casting light, the item, two effects, x, y, z, angles x, y, z), the
/// player's touch pick (`MagicCaster` +0x0C, thiscall: a `MagicTarget`), the game's staff cast
/// (thiscall on the actor: its equipped weapon's inventory entry).
const CHECKED_CAST: usize = 0x0069_BEC0;
pub const SPAWN_PROJECTILE: usize = 0x0069_F490;
const TOUCH_PICK: usize = 0x0065_D4E0;
const USE_STAFF: usize = 0x005E_4920;
/// An actor's `MagicTarget` (vtable +0x124).
const VT_MAGIC_TARGET: usize = 0x124;
/// The process's movement flags (vtable +0x2C0): swimming or under the water.
const VT_MOVEMENT: usize = 0x2C0;
const IN_WATER: u32 = 0x800;
/// The process's equipped weapon (`MiddleHighProcess` +0xE4: an inventory entry).
const EQUIPPED_WEAPON: usize = 0xE4;
/// `PlayerCharacter`: the selected `MagicItem`, the selected scroll; select a spell (thiscall: its
/// `MagicItem`), a scroll (0: none); `GetActiveMagicItem` (the selected, or the default spell's).
const SELECTED: usize = 0x624;
const SELECTED_SCROLL: usize = 0x628;
const SET_ACTIVE_SPELL: usize = 0x0066_4700;
const SET_ACTIVE_SCROLL: usize = 0x0066_4850;
/// The interface's sounds (cdecl: which); the magic menu's equip ("SPLEquip").
const MENU_SOUND: usize = 0x0057_DE50;
const SPL_EQUIP: u32 = 23;
/// The HUD's message (cdecl: text, 0, 1, -1.0); the player's failure sound for a school (thiscall on
/// the player: its index 0..5, from its skill by 0x41B7C0, cdecl).
const QUEUE_MESSAGE: usize = 0x0057_ACC0;
const FAILURE_SOUND: usize = 0x0066_35E0;
const SCHOOL_INDEX: usize = 0x0041_B7C0;
/// The failure messages (game settings: their text at +0), by the check's failure code: none, not
/// enough magicka, silenced, too little skill, a power used today, a bound item, ranged under water;
/// and the release's own, summoning in the water.
const FAILURE_TEXT: [usize; 7] = [0x00B3_351C, 0x00B3_3524, 0x00B3_3534, 0x00B3_352C, 0x00B3_353C, 0x00B3_3544, 0x00B3_354C];
const SUMMON_IN_WATER: usize = 0x00B3_8DE8;
/// The quick keys: 8 lists (16 bytes each: the first node at +4; a node {next, prev, the form}).
const QUICK_KEYS: usize = 0x00B3_B440;
/// `Actor`'s actor values (thiscall: which), the player's max modifiers (+0x204, a float each), the
/// base value (0x5F1910); magicka's.
const AV_MAGICKA: u32 = 9;
const MAX_MODIFIERS: usize = 0x204;
const GET_BASE_AV: usize = 0x005F_1910;
/// The player's Telekinesis (an actor value: the spell's magnitude in feet while it's on), and a
/// foot in metres.
const AV_TELEKINESIS: u32 = 60;
const FOOT: f32 = 0.3048;
/// The game's cast animations to their release (Hit) and on to their end, by range: self
/// (`castself.kf`), touch, target (seconds).
const CAST_TIME: [f32; 3] = [0.367, 0.267, 0.400];
const RECOVER: [f32; 3] = [0.933, 0.600, 0.500];

/// Each spell's `SpellItem` by id, as last published; the selected item the adapter itself made,
/// and the player's own picks counted.
struct Known {
    by_id: HashMap<u32, usize>,
    selected: usize,
    picks: u32,
    picked: u32,
}

static KNOWN: Mutex<Option<Known>> = Mutex::new(None);

/// The game's say on the hands' telekinesis, into `rules` (the game's melee rules for the runtime):
/// on while its Telekinesis is on the player, as far as the spell reaches, things only.
pub fn telekinesis_rules(rules: &mut MeleeRules) {
    let player = engine::player();
    let feet = if player != 0 { crate::combat::actor_value(player, AV_TELEKINESIS) } else { 0 };
    rules.flags |= RULES_SAYS_TELEKINESIS | RULES_TK_THINGS | if feet > 0 { RULES_TELEKINESIS } else { 0 };
    rules.tk_reach = if feet > 0 { feet as f32 * FOOT } else { 0.0 };
}

/// Once a game frame: the game's melee rules for the runtime, written when they change (only
/// telekinesis's so far: whoever writes the rest folds [`telekinesis_rules`] in).
pub fn publish_rules(a: &rustvr_adapter::Adapter) {
    static SENT: Mutex<Option<(u32, u32)>> = Mutex::new(None);
    let mut rules: MeleeRules = bytemuck::Zeroable::zeroed();
    telekinesis_rules(&mut rules);
    let key = (rules.flags, rules.tk_reach.to_bits());
    let mut sent = SENT.lock().unwrap();
    if *sent == Some(key) {
        return;
    }
    if a.write_melee_rules(&rules) {
        if rules.flags & RULES_TELEKINESIS != 0 {
            tracing::info!("spells: the game's Telekinesis is on the player: the hands' telekinesis, {:.1} m", rules.tk_reach);
        } else if sent.is_some() {
            tracing::info!("spells: the game's Telekinesis is off: the gravity gloves again");
        }
        *sent = Some(key);
    }
}

/// The player's spell with form id `id` (its `SpellItem`), as last published.
pub fn spell_item(id: u32) -> Option<usize> {
    KNOWN.lock().unwrap().as_ref()?.by_id.get(&id).copied()
}

/// The interface's font for the wheel: its first (the menus' own).
fn font() -> String {
    #[cfg(all(windows, target_arch = "x86"))]
    return crate::alchemy::font(1);
    #[cfg(not(all(windows, target_arch = "x86")))]
    String::new()
}

#[cfg(target_arch = "x86")]
mod game {
    use super::*;

    pub fn list(spell: usize) -> usize {
        spell + MAGIC_ITEM + ITEM_EFFECTS
    }

    /// The costliest effect of the list (its `EffectItem`), any range.
    pub fn costliest(list: usize) -> usize {
        type F = unsafe extern "thiscall" fn(usize, u32, u8) -> usize;
        unsafe { std::mem::transmute::<usize, F>(COSTLIEST)(list, 3, 0) }
    }

    pub fn main_effect(item: usize) -> usize {
        type F = unsafe extern "thiscall" fn(usize, i32) -> usize;
        unsafe { std::mem::transmute::<usize, F>(MAIN_EFFECT)(item, 0) }
    }

    pub fn level(list: usize) -> u32 {
        type F = unsafe extern "thiscall" fn(usize) -> u32;
        let f = mem::deref(mem::deref(list) + VT_LIST_LEVEL);
        if f == 0 { 0 } else { unsafe { std::mem::transmute::<usize, F>(f)(list) } }
    }

    pub fn cost(list: usize, actor: usize) -> f32 {
        type F = unsafe extern "thiscall" fn(usize, usize) -> f32;
        let f = mem::deref(mem::deref(list) + VT_LIST_COST);
        if f == 0 { 0.0 } else { unsafe { std::mem::transmute::<usize, F>(f)(list, actor) } }
    }

    pub fn summons(list: usize) -> bool {
        type F = unsafe extern "fastcall" fn(usize) -> u8;
        unsafe { std::mem::transmute::<usize, F>(SUMMONS)(list) != 0 }
    }

    pub fn school_skill(list: usize) -> i32 {
        type F = unsafe extern "thiscall" fn(usize) -> i32;
        unsafe { std::mem::transmute::<usize, F>(SCHOOL_SKILL)(list) }
    }

    /// Whether the player's caster would cast `item` now, and if not the failure code.
    pub fn usable(caster: usize, item: usize) -> (bool, u32) {
        type F = unsafe extern "thiscall" fn(usize, usize, *mut f32, *mut u32, u8) -> u8;
        let (mut skill, mut failure) = (0f32, 0u32);
        let f = mem::deref(mem::deref(caster) + VT_USABLE);
        let ok = f != 0 && unsafe { std::mem::transmute::<usize, F>(f)(caster, item, &mut skill, &mut failure, 0) } != 0;
        (ok, failure)
    }

    pub fn base_av(actor: usize, av: u32) -> f32 {
        type F = unsafe extern "thiscall" fn(usize, u32) -> i32;
        unsafe { std::mem::transmute::<usize, F>(GET_BASE_AV)(actor, av) as f32 }
    }

    /// The forms on the game's quick keys.
    pub fn quick_keys() -> Vec<usize> {
        let mut out = Vec::new();
        for k in 0..8 {
            let mut node = mem::deref(QUICK_KEYS + k * 16 + 4);
            let mut guard = 0;
            while node != 0 && guard < 16 {
                guard += 1;
                let form = mem::deref(node + 8);
                if form != 0 {
                    out.push(form);
                }
                node = mem::deref(node);
            }
        }
        out
    }

    pub fn message(text: usize) {
        type F = unsafe extern "C" fn(usize, u32, u32, f32) -> u8;
        if text != 0 {
            unsafe { std::mem::transmute::<usize, F>(QUEUE_MESSAGE)(text, 0, 1, -1.0) };
        }
    }

    pub fn failure_sound(player: usize, list: usize) {
        type Index = unsafe extern "C" fn(i32) -> i32;
        type Play = unsafe extern "thiscall" fn(usize, i32) -> usize;
        let school = unsafe { std::mem::transmute::<usize, Index>(SCHOOL_INDEX)(school_skill(list)) };
        if (0..6).contains(&school) {
            unsafe { std::mem::transmute::<usize, Play>(FAILURE_SOUND)(player, school) };
        }
    }

    pub fn in_water(actor: usize) -> bool {
        type F = unsafe extern "thiscall" fn(usize) -> u32;
        let process = mem::deref(actor + 0x58);
        let f = if process != 0 { mem::deref(mem::deref(process) + VT_MOVEMENT) } else { 0 };
        f != 0 && unsafe { std::mem::transmute::<usize, F>(f)(process) } & IN_WATER != 0
    }

    pub fn magic_target(actor: usize) -> usize {
        type F = unsafe extern "thiscall" fn(usize) -> usize;
        let f = mem::deref(mem::deref(actor) + VT_MAGIC_TARGET);
        if f == 0 { 0 } else { unsafe { std::mem::transmute::<usize, F>(f)(actor) } }
    }
}

/// The player's spells now, and each one's `SpellItem` by id.
#[cfg(target_arch = "x86")]
pub fn table() -> (Box<Spells>, HashMap<u32, usize>) {
    let mut table: Box<Spells> = bytemuck::zeroed_box();
    let mut by_id = HashMap::new();
    let player = engine::player();
    if player == 0 {
        return (table, by_id);
    }
    let caster = player + MAGIC_CASTER;
    table.magicka = crate::combat::actor_value(player, AV_MAGICKA) as f32;
    // (The HUD's bar: the base and the max modifier, a Fortify Magicka's.)
    table.magicka_max = game::base_av(player, AV_MAGICKA) + mem::at::<f32>(player + MAX_MODIFIERS + 4 * AV_MAGICKA as usize).unwrap_or(0.0);
    table.cast_time = CAST_TIME[2];
    table.recover = RECOVER[2];
    // The game's own casting art goes where the runtime's casting is (`crate::magic`).
    table.flags = SPELLS_GAME_ART;
    set_name(&mut table.font, &font());
    let selected = mem::deref(player + SELECTED);
    let quick = game::quick_keys();
    let base = mem::deref(player + BASE_FORM);
    let mut entry = base + SPELL_LIST;
    let mut rows: Vec<(u32, String, Spell)> = Vec::new();
    let mut guard = 0;
    while entry != 0 && guard < 512 && rows.len() < MAX_SPELLS {
        guard += 1;
        let spell = mem::deref(entry);
        entry = mem::deref(entry + 4);
        if spell == 0 {
            continue;
        }
        let kind = mem::at::<u32>(spell + SPELL_TYPE).unwrap_or(u32::MAX);
        if ![SPELL, POWER, LESSER_POWER].contains(&kind) {
            continue;
        }
        let Some(id) = mem::at::<u32>(spell + FORM_ID) else { continue };
        let list = game::list(spell);
        let mut s: Spell = bytemuck::Zeroable::zeroed();
        s.id = id;
        let name = crate::combat::read_str(mem::deref(spell + MAGIC_ITEM + 4));
        s.set_name(&name);
        // Its farthest range (target over touch over self), whether any effect harms.
        let mut node = list + 4;
        let (mut range, mut harmful, mut n) = (0u32, false, 0);
        while node != 0 && n < 16 {
            n += 1;
            let effect = mem::deref(node);
            node = mem::deref(node + 4);
            if effect == 0 {
                continue;
            }
            let setting = mem::deref(effect + EFFECT_SETTING);
            let flags = mem::at::<u32>(setting + SETTING_FLAGS).unwrap_or(0);
            if flags & DISABLED != 0 {
                continue;
            }
            range = range.max(mem::at::<u32>(effect + EFFECT_RANGE).unwrap_or(0).min(2));
            harmful |= flags & HOSTILE != 0;
        }
        s.delivery = range;
        s.cast_time = CAST_TIME[range as usize];
        s.recover = RECOVER[range as usize];
        // The magic menu's icon and school: its costliest effect's. Its glow: its main effect's light.
        let costliest = game::costliest(list);
        let setting = if costliest != 0 { mem::deref(costliest + EFFECT_SETTING) } else { 0 };
        let icon = crate::combat::read_str(mem::deref(setting + SETTING_ICON));
        if setting != 0 && !icon.is_empty() {
            s.set_icon(&crate::alchemy::texture_name(&format!("textures\\menus\\icons\\{icon}")));
        }
        s.school = mem::at::<u32>(setting + SETTING_SCHOOL).unwrap_or(0);
        let main = game::main_effect(spell + MAGIC_ITEM);
        let light = mem::deref(main + SETTING_LIGHT);
        let c = mem::at::<[u8; 4]>(light + LIGHT_COLOUR).filter(|_| main != 0 && light != 0).unwrap_or([255, 220, 160, 0]);
        s.color = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
        s.level = game::level(list).min(4);
        s.cost = game::cost(list, player).max(0.0);
        s.flags = if kind != SPELL { SPELL_POWER } else { 0 } | if harmful { SPELL_HARMFUL } else { 0 } | if quick.contains(&spell) { SPELL_FAVOURITE } else { 0 } | if selected == spell + MAGIC_ITEM { SPELL_SELECTED } else { 0 };
        if game::usable(caster, spell + MAGIC_ITEM).0 {
            s.flags |= SPELL_CASTABLE;
        }
        let group = match kind {
            POWER => 0,
            LESSER_POWER => 1,
            _ => 2,
        };
        rows.push((group, name.to_lowercase(), s));
        by_id.insert(id, spell);
    }
    rows.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    for (i, (_, _, s)) in rows.iter().enumerate() {
        table.spells[i] = *s;
    }
    table.count = rows.len() as u32;
    // The player's own picks: the selected item changed, and not by the adapter.
    let mut k = KNOWN.lock().unwrap();
    let k = k.get_or_insert_with(|| Known { by_id: HashMap::new(), selected, picks: 0, picked: 0 });
    if selected != k.selected {
        k.selected = selected;
        if let Some((&id, _)) = by_id.iter().find(|(_, s)| **s + MAGIC_ITEM == selected) {
            k.picks = k.picks.wrapping_add(1);
            k.picked = id;
            tracing::info!("spells: the player picked {id:#x} in the game's interface");
        }
    }
    k.by_id = by_id.clone();
    table.picks = k.picks;
    table.picked = k.picked;
    (table, by_id)
}

#[cfg(not(target_arch = "x86"))]
pub fn table() -> (Box<Spells>, HashMap<u32, usize>) {
    (bytemuck::zeroed_box(), HashMap::new())
}

/// A cast the adapter is making: the projectile goes from here this way (game units, radians),
/// carrying this casting light.
#[derive(Clone, Copy, Debug)]
pub struct Launch {
    pub pos: [f32; 3],
    pub pitch: f32,
    pub heading: f32,
}

static LAUNCH: Mutex<Option<Launch>> = Mutex::new(None);
/// While the adapter casts a touch spell into a creature the hand touched: that creature.
static TOUCHED: AtomicUsize = AtomicUsize::new(0);
/// Casts made, for the log.
static CASTS: AtomicU32 = AtomicU32::new(0);
/// The bound weapon and the bound shield a hand just conjured: which hand, until when.
static CONJURED: Mutex<[Option<(u32, f64)>; 2]> = Mutex::new([None, None]);

/// What a bound spell makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Weapon = 0,
    Shield = 1,
}

/// The hand a newly equipped bound weapon or shield goes into (`ObjectDesc::into_hand`: 1 the left,
/// 2 the right), if a hand just cast the spell that made it; 0 otherwise. Taken once.
pub fn conjured_into(what: Bound) -> u32 {
    let mut c = CONJURED.lock().unwrap();
    match c[what as usize].take() {
        Some((hand, until)) if crate::link::now() <= until => hand.min(1) + 1,
        _ => 0,
    }
}

/// Whether `form` (a weapon or an armour) is a bound one: the item a bound effect of the game's
/// equips (its `EffectSetting` data, as a form or its id).
pub fn is_bound_item(form: usize) -> bool {
    static ITEMS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
    if form == 0 {
        return false;
    }
    let mut items = ITEMS.lock().unwrap();
    if items.is_empty() {
        for e in crate::combat::effect_settings() {
            if mem::at::<u32>(e + SETTING_FLAGS).unwrap_or(0) & (BOUND_WEAPON | BOUND_ARMOUR) != 0
                && let Some(item) = mem::at::<u32>(e + SETTING_ITEM).filter(|&i| i != 0)
            {
                items.push(item);
            }
        }
        if !items.is_empty() {
            tracing::info!("spells: {} bound weapons and armour among the game's effects", items.len());
        }
    }
    let id = mem::at::<u32>(form + FORM_ID).unwrap_or(0);
    items.iter().any(|&i| i == form as u32 || (id != 0 && i == id))
}

/// The bound things a spell's list makes on the caster: a weapon, a shield.
#[cfg(target_arch = "x86")]
fn bound(list: usize) -> (bool, bool) {
    let (mut weapon, mut shield) = (false, false);
    let mut node = list + 4;
    let mut n = 0;
    while node != 0 && n < 16 {
        n += 1;
        let effect = mem::deref(node);
        node = mem::deref(node + 4);
        if effect == 0 || mem::at::<u32>(effect + EFFECT_RANGE) != Some(0) {
            continue;
        }
        let setting = mem::deref(effect + EFFECT_SETTING);
        let flags = mem::at::<u32>(setting + SETTING_FLAGS).unwrap_or(0);
        weapon |= flags & BOUND_WEAPON != 0;
        shield |= flags & BOUND_ARMOUR != 0 && mem::at::<u32>(effect) == Some(BOUND_SHIELD);
    }
    (weapon, shield)
}

/// The launch of the cast being made now, if any (for the projectile spawn hook).
pub fn launch() -> Option<Launch> {
    *LAUNCH.lock().unwrap()
}

/// While the game casts for the hand: the projectile leaves `ev.point` along `ev.dir` (the launch
/// the projectile spawn takes), the player faces that way (the game aims some things by where the
/// caster faces: its touch pick, a projectile's), and the caster's magic node is at the palm (where
/// the game puts what goes off in the caster's hand: a touch spell's burst with nothing touched);
/// `f` casts.
#[cfg(target_arch = "x86")]
fn launching(player: usize, ev: &CombatEvent, f: impl FnOnce()) {
    let dir = Vec3::from_array(ev.dir);
    let (pitch, heading) = crate::combat::angles(dir);
    let pos = camera::basis().point_to_game(Vec3::from_array(ev.point)).to_array();
    *LAUNCH.lock().unwrap() = Some(Launch { pos, pitch, heading });
    let rot = (player + engine::refr::ROT) as *mut f32;
    let (old_pitch, old_heading) = unsafe { (rot.read(), rot.add(2).read()) };
    unsafe {
        rot.write(pitch);
        rot.add(2).write(heading.rem_euclid(std::f32::consts::TAU));
    }
    // (The node's world place, for the moment of the cast: the game's next update puts it back.)
    let node = magic_node(player);
    let at = (node + NODE_WORLD_POS) as *mut [f32; 3];
    let old_at = (node != 0 && mem::is_readable(at as *const u8, 12)).then(|| unsafe { at.read() });
    if old_at.is_some() {
        unsafe { at.write(pos) };
    }
    f();
    if let Some(old) = old_at {
        unsafe { at.write(old) };
    }
    unsafe {
        rot.write(old_pitch);
        rot.add(2).write(old_heading);
    }
    *LAUNCH.lock().unwrap() = None;
}

/// The player's caster's magic node (`MagicCaster` +0x24: the skeleton's `magicNode`), or 0.
#[cfg(target_arch = "x86")]
fn magic_node(player: usize) -> usize {
    type Node = unsafe extern "thiscall" fn(usize) -> usize;
    let caster = player + MAGIC_CASTER;
    let f = mem::deref(mem::deref(caster) + VT_MAGIC_NODE);
    if f == 0 { 0 } else { unsafe { std::mem::transmute::<usize, Node>(f)(caster) } }
}

/// A hand's cast of `spell` (its `SpellItem`): the game's own checked cast, from the palm along the
/// aim, into what the hand touched.
#[cfg(target_arch = "x86")]
pub fn cast(player: usize, spell: usize, ev: &CombatEvent, actors: &HashMap<u32, usize>) {
    let caster = player + MAGIC_CASTER;
    let item = spell + MAGIC_ITEM;
    let list = game::list(spell);
    // (The game's release keeps this one: no summoning while swimming or under the water.)
    if game::summons(list) && game::in_water(player) {
        tracing::info!("spells: {:#x} summons, and the player is in the water", ev.what);
        game::message(mem::deref(SUMMON_IN_WATER));
        return;
    }
    let touched = if ev.actor != 0 && ev.flags & CEF_TOUCHED != 0 { actors.get(&ev.actor).copied().filter(|&a| crate::combat::is_actor(a)).unwrap_or(0) } else { 0 };
    // A spell that throws carries its main effect's casting light (the casting art's, handed to the
    // bolt), made at the caster's magic node as the art makes it.
    let mut light = 0;
    if ev.dir != [0.0; 3] && has_target_effect(list) && mem::deref(caster + CASTER_LIGHT) == 0 {
        let main = game::main_effect(item);
        let form = mem::deref(main + SETTING_LIGHT);
        let node = magic_node(player);
        if let Some(l) = crate::scene::CastingLight::new(mem::at::<u32>(form + FORM_ID).filter(|_| form != 0).unwrap_or(0), node, 0.01) {
            light = l.raw();
            unsafe { ((caster + CASTER_LIGHT) as *mut usize).write(light) };
        }
    }
    let n = CASTS.fetch_add(1, Ordering::Relaxed) + 1;
    tracing::info!("spells: cast {n}: {:#x} from {:?} towards {:?}{}", ev.what, ev.point, ev.dir, if touched != 0 { " into the creature the hand touched" } else { "" });
    TOUCHED.store(touched, Ordering::Release);
    type SetItem = unsafe extern "thiscall" fn(usize, usize);
    type Cast = unsafe extern "thiscall" fn(usize, usize) -> usize;
    let set = mem::deref(mem::deref(caster) + VT_SET_ITEM);
    launching(player, ev, || unsafe {
        if set != 0 {
            std::mem::transmute::<usize, SetItem>(set)(caster, item);
            std::mem::transmute::<usize, Cast>(CHECKED_CAST)(caster, 0);
        }
    });
    TOUCHED.store(0, Ordering::Release);
    // (A light the bolt didn't take: the cast failed, or nothing was thrown.)
    if light != 0 && mem::deref(caster + CASTER_LIGHT) == light {
        unsafe { ((caster + CASTER_LIGHT) as *mut usize).write(0) };
        crate::scene::CastingLight::from_raw(light).remove();
    }
    let refused = mem::at::<u32>(caster + CASTER_STATE) == Some(CANT_CAST);
    if refused {
        tracing::info!("spells: the game wouldn't cast {:#x}", ev.what);
    } else {
        crate::mirror_spell::cast(spell);
        // A bound weapon or shield: the game equips it, and it's made in the hand that cast it
        // (`crate::weapons`).
        let (weapon, shield) = bound(list);
        let until = crate::link::now() + CONJURE_WITHIN;
        let mut c = CONJURED.lock().unwrap();
        if weapon {
            c[Bound::Weapon as usize] = Some((ev.hand, until));
        }
        if shield {
            c[Bound::Shield as usize] = Some((ev.hand, until));
        }
        if weapon || shield {
            tracing::info!("spells: {:#x} conjures a bound {} into the {} hand", ev.what, if weapon { "weapon" } else { "shield" }, if ev.hand == 0 { "left" } else { "right" });
        }
    }
}

/// Whether a spell's list has a target effect.
#[cfg(target_arch = "x86")]
fn has_target_effect(list: usize) -> bool {
    let mut node = list + 4;
    let mut n = 0;
    while node != 0 && n < 16 {
        n += 1;
        let effect = mem::deref(node);
        node = mem::deref(node + 4);
        if effect != 0 && mem::at::<u32>(effect + EFFECT_RANGE) == Some(2) {
            return true;
        }
    }
    false
}

/// The player put `spell` in a hand from the wheel: the game's own selection, as its magic menu
/// makes it (a scroll put down first; its sound).
#[cfg(target_arch = "x86")]
pub fn select(spell: usize) {
    let player = engine::player();
    if player == 0 || spell == 0 {
        return;
    }
    type Select = unsafe extern "thiscall" fn(usize, usize);
    type Sound = unsafe extern "C" fn(u32) -> u32;
    unsafe {
        if mem::deref(player + SELECTED_SCROLL) != 0 {
            std::mem::transmute::<usize, Select>(SET_ACTIVE_SCROLL)(player, 0);
        }
        std::mem::transmute::<usize, Select>(SET_ACTIVE_SPELL)(player, spell + MAGIC_ITEM);
        std::mem::transmute::<usize, Sound>(MENU_SOUND)(SPL_EQUIP);
    }
    // (The adapter's own pick, not one of the player's in the game's interface.)
    if let Some(k) = KNOWN.lock().unwrap().as_mut() {
        k.selected = mem::deref(player + SELECTED);
    }
    tracing::info!("spells: selected {:#x} from the wheel", mem::at::<u32>(spell + FORM_ID).unwrap_or(0));
}

/// The trigger pulled on `spell` (its `SpellItem`), which the game won't cast now: why, its own way
/// (its message, its school's failure sound), as the start of its casting animation says it.
#[cfg(target_arch = "x86")]
pub fn failed(spell: usize) {
    let player = engine::player();
    if player == 0 || spell == 0 {
        return;
    }
    let (ok, failure) = game::usable(player + MAGIC_CASTER, spell + MAGIC_ITEM);
    if ok {
        return;
    }
    tracing::info!("spells: the game won't cast {:#x}: failure {failure}", mem::at::<u32>(spell + FORM_ID).unwrap_or(0));
    game::failure_sound(player, game::list(spell));
    if let Some(&at) = FAILURE_TEXT.get(failure as usize) {
        game::message(mem::deref(at));
    }
}

/// A staff's cast: the game's own (0x5E4920, as its staff attack does it: the enchantment's cost
/// taken from the staff's charge, cast, or the game's "not enough charge"), from its tip.
#[cfg(target_arch = "x86")]
pub fn cast_staff(player: usize, ev: &CombatEvent) {
    let entry = mem::deref(mem::deref(player + 0x58) + EQUIPPED_WEAPON);
    if entry == 0 {
        return;
    }
    tracing::info!("spells: the player's staff casts {:#x} from {:?} towards {:?}", ev.what, ev.point, ev.dir);
    type UseStaff = unsafe extern "thiscall" fn(usize, usize);
    launching(player, ev, || unsafe { std::mem::transmute::<usize, UseStaff>(USE_STAFF)(player, entry) });
}

#[cfg(target_arch = "x86")]
mod hook {
    use super::*;

    static SPAWN: AtomicUsize = AtomicUsize::new(0);
    static PICK: AtomicUsize = AtomicUsize::new(0);
    type Spawn = unsafe extern "C" fn(u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) -> u32;
    type Pick = unsafe extern "thiscall" fn(usize) -> usize;

    pub fn install() {
        crate::hooks::install_into(SPAWN_PROJECTILE as *const (), spawn as *const (), "MagicProjectile spawn", &SPAWN);
        crate::hooks::install_into(TOUCH_PICK as *const (), pick as *const (), "the player's touch pick", &PICK);
    }

    /// While the adapter casts: the projectile leaves the palm along the hand's aim.
    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn spawn(a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: u32, y: u32, z: u32, ax: u32, ay: u32, az: u32) -> u32 {
        let real: Spawn = unsafe { std::mem::transmute(SPAWN.load(Ordering::Acquire)) };
        let (x, y, z, ax, ay, az) = match super::launch() {
            Some(l) => {
                let f = f32::from_bits;
                tracing::info!("spells: the projectile leaves the hand at {:?} (the game's {:?}), angles {:.2}, {:.2} (the game's {:.2}, {:.2}), light {a3:#x}", l.pos, [f(x), f(y), f(z)], l.pitch, l.heading, f(ax), f(az));
                (l.pos[0].to_bits(), l.pos[1].to_bits(), l.pos[2].to_bits(), l.pitch.to_bits(), 0f32.to_bits(), l.heading.to_bits())
            }
            None => (x, y, z, ax, ay, az),
        };
        unsafe { real(a1, a2, a3, a4, a5, a6, x, y, z, ax, ay, az) }
    }

    /// While the adapter casts a touch spell into the creature the hand touched: that creature's
    /// `MagicTarget` (else the game's own pick).
    unsafe extern "thiscall" fn pick(caster: usize) -> usize {
        let real: Pick = unsafe { std::mem::transmute(PICK.load(Ordering::Acquire)) };
        let touched = TOUCHED.load(Ordering::Acquire);
        if touched != 0 && crate::combat::is_actor(touched) {
            let t = game::magic_target(touched);
            if t != 0 {
                return t;
            }
        }
        unsafe { real(caster) }
    }
}

#[cfg(target_arch = "x86")]
pub use hook::install;

#[cfg(not(target_arch = "x86"))]
pub fn install() {}
