//! Oblivion's side of the framework's combat (`rustvr_protocol::combat`).
//!
//! * **Its creatures** ([`actors`]): every actor the game is fully processing near the player
//!   (its high-process list), as capsules on its own skeleton's bones (`Bip01 ...`): the head, the
//!   spine, the arms, the hands, the legs and the feet, each as thick as what it wears
//!   ([`crate::shapes`]).
//! * **The player's spells** ([`spells`]): the spells, powers and lesser powers on the player's
//!   own list, with how each is delivered (its effects' range), what it costs the player now
//!   (the game's own cost), the colour of its first effect's light, which one is selected, and
//!   the player's magicka.
//! * **What the hands did** ([`apply`], on the game's thread before its frame). The runtime judges
//!   each blow (`docs/design/oblivion-melee.md`; each part's material is what
//!   the actor wears there, [`materials`]); the game does the rest:
//!   * A melee hit is the game's own hit: the player's hit routine (`Actor` vtable 0x3AC,
//!     0x5FEBF0) on that creature, the way the attack animation would land it. That brings the
//!     game's damage formula, skills, sneak attacks, enchantments, armour, blocking, sounds, blood,
//!     the OnHit scripts and the stagger. It's as hard as the runtime judged the blow ([`how_hard`]: the
//!     swing's strength, the runtime's make of it, times the body part's), through the damage
//!     formula's own multiplier ([`game_hit`]); one that shoves the body by the push tables (its push
//!     level 1 or more) is the game's power attack, in the way the blow went ([`power_group`]). A
//!     fist (or a shield's bash) is its hand-to-hand.
//!   * A blade gone in deep is one more hit for its wound past the trunk's ([`wound_beyond`]: a
//!     cut throat, a point deep in the head). A blunt blow to the head past the knockout speed
//!     knocks the creature out (its Fatigue below nothing, the game's own knockout); a hand struck
//!     at the disarming push level, or an arm stabbed through, drops its weapon (the game's own
//!     disarm, [`disarm`]).
//!   * A block (the hand's weapon meeting a creature's swing) is the game's own block: its Block
//!     control held through the blow, so the game's block rules, skill and sounds decide it.
//!   * A cast is the game's own cast, at once: the caster's usability check (enough magicka, not
//!     silenced), its cost, and an immediate cast (0x69BE90: no casting animation, the hand
//!     already did that). The projectile leaves the palm along the hand's aim: the projectile
//!     spawn (0x69F490) is given the hand's position and the aim's angles while the cast runs.

use crate::camera;
use crate::engine::{self, refr};
use crate::mem;
use glam::Vec3;
use rustvr_protocol::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// `ActorProcessManager` (a global struct) and its list of high-process actors (`tList` with an
/// inline first node, then its tail: every actor the game is fully processing around the player).
/// In this build it's at +0x68, not where xOBSE's header has `highActors` (+0x5C holds no list:
/// walking it reads a creature's vtable as an actor).
const PROCESS_MANAGER: usize = 0x00B3_BD00;
const ACTORS_LIST: usize = 0x68;
/// Creatures further than this aren't sent (metres).
const ACTORS_RANGE: f32 = 30.0;
/// `TESForm::refID`.
const FORM_ID: usize = 0x0C;
/// The vtables of the game's actors (`Character`, `Creature`). Anything else in the list (an object
/// being made or taken away, as a save loads or a creature is placed) is passed over: calling its
/// virtuals jumps to garbage.
const ACTOR_VTABLES: [usize; 2] = [0x00A6_FC9C, 0x00A7_10F4];

/// Whether `actor` is one of the game's actors as it stands (not the player).
pub(crate) fn is_actor(actor: usize) -> bool {
    actor != 0 && ACTOR_VTABLES.contains(&mem::deref(actor))
}
/// `TESObjectREFR`: its base form, its 3D root.
const BASE_FORM: usize = 0x1C;
const NI_NODE: usize = 0x3C;
/// `Actor`: its magic caster; the vtable slot of `GetActorValue` and of the hit routine.
const MAGIC_CASTER: usize = 0x5C;
const VT_GET_AV: usize = 0x284;
const VT_HIT: usize = 0x3AC;
const GET_BASE_AV: usize = 0x005F_1910;
const AV_HEALTH: u32 = 8;
const AV_MAGICKA: u32 = 9;
const AV_FATIGUE: u32 = 10;
/// `Actor` vtable: `DamageActorValue` (thiscall: the value, minus the amount, who did it).
const VT_DAMAGE_AV: usize = 0x2A4;
/// The game's settings its damage formulas read (and nothing else does): the weapon's damage
/// multiplier (`fDamageWeaponMult`), a fist's least and most (`fHandHealthMin`, `fHandHealthMax`).
const DAMAGE_WEAPON_MULT: usize = 0x00B3_6DD8;
const HAND_HEALTH_MIN: usize = 0x00B3_6E70;
const HAND_HEALTH_MAX: usize = 0x00B3_6E78;
/// `Actor` vtable +0x164: its `ActorAnimData`, whose +0x42's low byte is the attack's anim group
/// (22..26 its power attacks), which the hit routine reads for the power attack's bonus and perks.
const VT_ANIM_DATA: usize = 0x164;
const ANIM_GROUP: usize = 0x42;
/// The disarm's steps (thiscall unless said): the actor's drawn weapon's entry, its drop (vtable),
/// the dropped one owned (cdecl), its current action and setting it, its combat controller
/// (vtable) and resetting it.
const DRAWN_WEAPON: usize = 0x005E_4A80;
const VT_DROP: usize = 0x2C8;
const OWN_DROPPED: usize = 0x004D_C000;
const CURRENT_ACTION: usize = 0x005E_0EE0;
const SET_ACTION: usize = 0x005E_FFD0;
const VT_COMBAT_CONTROLLER: usize = 0x330;
const RESET_COMBAT: usize = 0x0061_DD10;
/// `PlayerCharacter::GetActiveMagicItem` (thiscall): the selected spell's `MagicItem*` (inside its
/// spell at +0x18).
const ACTIVE_MAGIC_ITEM: usize = 0x0065_D4A0;
/// `TESActorBase::spellList` entries (`TESSpellList` at +0x54, its list at +4).
const SPELL_LIST: usize = 0x58;
/// `SpellItem`: its `MagicItem` (name at +4, effects at +0x0C of it), then its data (`SPIT`:
/// type, cost, level, flags) at +0x34 in this build (xOBSE's header has +0x38).
const MAGIC_ITEM: usize = 0x18;
const SPELL_TYPE: usize = 0x34;
const SPELL_COST: usize = 0x38;
const SPELL_EFFECTS: usize = 0x28;
/// `EffectItem`: range, its `EffectSetting`; `EffectSetting`: its light; a light's colour.
const EFFECT_RANGE: usize = 0x10;
const EFFECT_SETTING: usize = 0x1C;
const SETTING_LIGHT: usize = 0x70;
const LIGHT_COLOUR: usize = 0x78;
/// `MagicCaster` vtable slots: apply the cost, is it usable.
const VT_APPLY_COST: usize = 0x18;
const VT_USABLE: usize = 0x1C;
/// `MagicCaster` immediate cast (`this`, item, target, unknown).
const CAST_IMMEDIATE: usize = 0x0069_BE90;
/// The game's staff cast (thiscall on the actor: its equipped weapon's inventory entry).
const USE_STAFF: usize = 0x005E_4920;
/// The projectile spawn (cdecl, 12 arguments: ... x, y, z, angle x, y, z).
pub const SPAWN_PROJECTILE: usize = 0x0069_F490;
/// `NiNode` children (`NiTArray`: data at +0xB0, first free entry at +0xB6).
const NODE_CHILDREN: usize = 0xB0;
const NODE_COUNT: usize = 0xB6;
/// `NiObjectNET` name.
const NODE_NAME: usize = 0x08;

/// `BaseProcess::GetCurrentAction` (vtable +0x2D0, thiscall, a signed 16-bit action): attacking
/// (attack, its follow-through) and blocking.
const VT_CURRENT_ACTION: usize = 0x2D0;
const ACTION_ATTACK: i16 = 2;
const ACTION_ATTACK_FOLLOW: i16 = 3;
const ACTION_BLOCK: i16 = 6;
/// The process's equipped weapon (`MiddleHighProcess` +0xE4: an inventory entry, its form at +8).
const EQUIPPED_WEAPON: usize = 0xE4;
/// A hand's reach past its wrist when it has no weapon (metres).
const FIST_REACH: f32 = 0.15;
/// How long a block holds the Block control (seconds): through the blow it meets.
const BLOCK_HOLD: f64 = 0.5;

/// Spell types the hand casts: a spell, a power, a lesser power.
const CASTABLE_TYPES: [u32; 3] = [0, 2, 3];

/// The game's magic effects by their codes (`NiTPointerMap<EffectSetting>`: buckets at +8, as many
/// as +4 says, a code's bucket the code modulo that; each entry {next, code, effect}).
const EFFECT_SETTINGS: usize = 0x00B3_3508;
/// `EffectItem`: its code; a script effect's info (`ScriptEffectInfo`: its visual effect's code at
/// +0x10).
const EFFECT_CODE: usize = 0x00;
const EFFECT_SCRIPT_INFO: usize = 0x18;
const SCRIPT_VISUAL: usize = 0x10;
pub const SCRIPT_EFFECT: u32 = u32::from_le_bytes(*b"SEFF");

/// The game's magic effect (`EffectSetting`) with this code, or 0.
pub fn effect_setting(code: u32) -> usize {
    let buckets = mem::at::<u32>(EFFECT_SETTINGS + 4).unwrap_or(0) as usize;
    let table = mem::deref(EFFECT_SETTINGS + 8);
    if buckets == 0 || table == 0 {
        return 0;
    }
    let mut entry = mem::deref(table + (code as usize % buckets) * 4);
    let mut guard = 0;
    while entry != 0 && guard < 256 {
        guard += 1;
        if mem::at::<u32>(entry + 4) == Some(code) {
            return mem::deref(entry + 8);
        }
        entry = mem::deref(entry);
    }
    0
}

/// The magic effect an effect looks like: its own, or for a script effect the one it was given the
/// look of (its visual effect), as the game shows it.
fn looks_like(effect: usize) -> usize {
    let info = mem::deref(effect + EFFECT_SCRIPT_INFO);
    let visual = if mem::at::<u32>(effect + EFFECT_CODE) == Some(SCRIPT_EFFECT) && info != 0 { mem::at::<u32>(info + SCRIPT_VISUAL).unwrap_or(0) } else { 0 };
    let shown = if visual != 0 { effect_setting(visual) } else { 0 };
    if shown != 0 { shown } else { mem::deref(effect + EFFECT_SETTING) }
}

pub(crate) fn read_str(p: usize) -> String {
    let mut out = Vec::new();
    for i in 0..128 {
        match mem::at::<u8>(p + i) {
            Some(0) | None => break,
            Some(b) => out.push(b),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `NiObject` vtable +0x8: the object as an `NiNode` (itself for one, else 0; +0xC is the same for a
/// shape). Only nodes have children: what's at a node's children's place in a shape (or a light,
/// a camera) is something else, and walked as children it's garbage that used up the walk before
/// the spine and legs.
const VT_AS_NODE: usize = 0x08;

#[cfg(target_arch = "x86")]
fn is_node(obj: usize) -> bool {
    let f = mem::deref(mem::deref(obj) + VT_AS_NODE);
    (0x0040_1000..0x00A3_0000).contains(&f) && unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(f)(obj) } == obj
}

#[cfg(not(target_arch = "x86"))]
fn is_node(_obj: usize) -> bool {
    false
}

/// The named nodes under `root` that are bones (named `Bip01...`), and a few named attachments.
pub(crate) fn bones(root: usize) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    let mut stack = vec![(root, 0u32)];
    let mut seen = 0;
    while let Some((node, depth)) = stack.pop() {
        seen += 1;
        if seen > 600 || depth > 48 {
            break;
        }
        let name = read_str(mem::deref(node + NODE_NAME));
        // (Only into a node: a mesh near the root, as the game's blood decals are, has no
        // children, and reading its fields as children followed garbage into freed memory.)
        let descend = (depth <= 2 || name.starts_with("Bip01")) && engine::is_node(node);
        if name.starts_with("Bip01") || name == "Weapon" || name == "Shield" {
            out.insert(name, node);
        }
        if !descend || !is_node(node) {
            continue;
        }
        let data = mem::deref(node + NODE_CHILDREN);
        let n = mem::at::<u16>(node + NODE_COUNT).unwrap_or(0).min(64) as usize;
        if data == 0 {
            continue;
        }
        for i in 0..n {
            let child = mem::deref(data + i * 4);
            if child != 0 {
                stack.push((child, depth + 1));
            }
        }
    }
    out
}

/// What an actor is doing (`GetCurrentAction`), or -1.
#[cfg(target_arch = "x86")]
fn current_action(actor: usize) -> i16 {
    let process = mem::deref(actor + 0x58);
    let f = mem::deref(mem::deref(process) + VT_CURRENT_ACTION);
    if process == 0 || !(0x0040_1000..0x00A3_0000).contains(&f) {
        return -1;
    }
    type F = unsafe extern "thiscall" fn(usize) -> i16;
    unsafe { std::mem::transmute::<usize, F>(f)(process) }
}

#[cfg(not(target_arch = "x86"))]
fn current_action(_actor: usize) -> i16 {
    -1
}

/// The length of an actor's equipped weapon (its model's, from the asset cache; metres), cached by
/// model path.
fn weapon_length(actor: usize) -> Option<f32> {
    static LENGTHS: Mutex<Option<HashMap<String, Option<f32>>>> = Mutex::new(None);
    let form = mem::deref(mem::deref(mem::deref(actor + 0x58) + EQUIPPED_WEAPON) + 8);
    if form == 0 || mem::at::<u8>(form + 4) != Some(0x21) {
        return None;
    }
    let path = read_str(mem::deref(form + 0x34));
    let mut cache = LENGTHS.lock().unwrap();
    *cache.get_or_insert_with(HashMap::new).entry(path.clone()).or_insert_with(|| {
        let key = oblivion_assets::export::model_key(&path);
        rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key).ok().map(|m| m.bounds_max[2])
    })
}

/// Where an attacker's blow is: its weapon (the `Weapon` node, the blade along its +Y as long as
/// the weapon's model), else its right fist (forearm to past the hand).
fn attack_segment(actor: usize, b: &HashMap<String, usize>) -> Option<(Vec3, Vec3, f32)> {
    if let (Some(&node), Some(len)) = (b.get("Weapon"), weapon_length(actor)) {
        let rot: [f32; 9] = mem::at(node + engine::av::WORLD_ROTATE)?;
        let at = node_pos(node)?;
        let along = camera::basis().dir_from_game(Vec3::new(rot[1], rot[4], rot[7])).normalize_or_zero();
        return Some((at, at + along * len, 0.04));
    }
    let (Some(fore), Some(hand)) = (b.get("Bip01 R Forearm").and_then(|&n| node_pos(n)), b.get("Bip01 R Hand").and_then(|&n| node_pos(n))) else { return None };
    Some((fore, hand + (hand - fore).normalize_or_zero() * FIST_REACH, 0.06))
}

/// A node's world position, in the protocol's space.
pub(crate) fn node_pos(node: usize) -> Option<Vec3> {
    let p: [f32; 3] = mem::at(node + engine::av::WORLD_TRANSLATE)?;
    Some(camera::basis().point_from_game(Vec3::from(p)))
}

/// Calls `Actor::GetActorValue`.
#[cfg(target_arch = "x86")]
pub(crate) fn actor_value(actor: usize, av: u32) -> i32 {
    type F = unsafe extern "thiscall" fn(usize, u32) -> i32;
    let vt = mem::deref(actor);
    let f = mem::deref(vt + VT_GET_AV);
    if f == 0 { 0 } else { unsafe { std::mem::transmute::<usize, F>(f)(actor, av) } }
}

#[cfg(target_arch = "x86")]
pub(crate) fn base_actor_value(actor: usize, av: u32) -> i32 {
    type F = unsafe extern "thiscall" fn(usize, u32) -> u32;
    unsafe { std::mem::transmute::<usize, F>(GET_BASE_AV)(actor, av) as i32 }
}

#[cfg(not(target_arch = "x86"))]
pub(crate) fn actor_value(_actor: usize, _av: u32) -> i32 {
    0
}

#[cfg(not(target_arch = "x86"))]
pub(crate) fn base_actor_value(_actor: usize, _av: u32) -> i32 {
    0
}

/// The capsules of a humanoid skeleton: (from bone, to bone, radius, part, material bare). Each is
/// fitted to what the actor wears ([`crate::shapes`]); the radius here holds until it is, and where
/// nothing's drawn on it; its material is what's worn over it ([`materials`]). (A part's index here
/// is its `game_part`.)
pub(crate) const HUMANOID: &[(&str, &str, f32, u32, u32)] = &[
    // (The torso to the top of the chest: its round end stops short of the neck, which a blade can
    // cut through.)
    ("Bip01 Pelvis", "Bip01 Spine2", 0.17, PART_TORSO, MAT_FLESH),
    ("Bip01 Neck", "Bip01 Head", 0.1, PART_HEAD, MAT_FLESH),
    ("Bip01 L UpperArm", "Bip01 L Forearm", 0.06, PART_ARM, MAT_FLESH),
    ("Bip01 L Forearm", "Bip01 L Hand", 0.05, PART_ARM, MAT_FLESH),
    ("Bip01 R UpperArm", "Bip01 R Forearm", 0.06, PART_ARM, MAT_FLESH),
    ("Bip01 R Forearm", "Bip01 R Hand", 0.05, PART_ARM, MAT_FLESH),
    ("Bip01 L Thigh", "Bip01 L Calf", 0.08, PART_LEG, MAT_FLESH),
    ("Bip01 L Calf", "Bip01 L Foot", 0.06, PART_LEG, MAT_FLESH),
    ("Bip01 R Thigh", "Bip01 R Calf", 0.08, PART_LEG, MAT_FLESH),
    ("Bip01 R Calf", "Bip01 R Foot", 0.06, PART_LEG, MAT_FLESH),
    ("Bip01 L Hand", "Bip01 L Finger2", 0.045, PART_HAND, MAT_FLESH),
    ("Bip01 R Hand", "Bip01 R Finger2", 0.045, PART_HAND, MAT_FLESH),
    ("Bip01 L Foot", "Bip01 L Toe0", 0.05, PART_FOOT, MAT_FLESH),
    ("Bip01 R Foot", "Bip01 R Toe0", 0.05, PART_FOOT, MAT_FLESH),
];

/// The bone a part of a humanoid starts at (its `game_part`: its index in [`HUMANOID`]).
pub(crate) fn part_bone(game_part: u32) -> Option<&'static str> {
    HUMANOID.get(game_part as usize).map(|e| e.0)
}

/// Which of the game's biped slots covers each part of [`HUMANOID`] (`TESBipedModelForm`'s mask):
/// the cuirass the trunk and the upper arms, the gauntlets the forearms and hands, the greaves the
/// legs, the boots the feet, the helmet the head.
const PART_SLOTS: [u16; 14] = [SLOT_UPPER, SLOT_HEAD, SLOT_UPPER, SLOT_HAND, SLOT_UPPER, SLOT_HAND, SLOT_LOWER, SLOT_LOWER, SLOT_LOWER, SLOT_LOWER, SLOT_HAND, SLOT_HAND, SLOT_FOOT, SLOT_FOOT];
const SLOT_HEAD: u16 = 1 << 0;
const SLOT_UPPER: u16 = 1 << 2;
const SLOT_LOWER: u16 = 1 << 3;
const SLOT_HAND: u16 = 1 << 4;
const SLOT_FOOT: u16 = 1 << 5;

/// What each part of an actor is made of to a blade, as the runtime's armour has it, from what
/// the actor wears over it: the game's heavy armour is plate, its light armour leather
/// (the game's own two kinds), nothing (or clothes) flesh. Read again each second.
fn materials(id: u32, actor: usize) -> [u32; 14] {
    static WORN: Mutex<Option<HashMap<u32, (f64, [u32; 14])>>> = Mutex::new(None);
    let now = crate::link::now();
    let mut worn = WORN.lock().unwrap();
    let worn = worn.get_or_insert_with(HashMap::new);
    if let Some(&(at, m)) = worn.get(&id)
        && now - at < 1.0
    {
        return m;
    }
    let pieces = crate::body::worn_armour(actor);
    let m = PART_SLOTS.map(|slot| match pieces.iter().filter(|(slots, _)| slots & slot != 0).map(|&(_, heavy)| heavy).max() {
        Some(true) => MAT_PLATE,
        Some(false) => MAT_LEATHER,
        None => MAT_FLESH,
    });
    if worn.get(&id).is_none_or(|&(_, old)| old != m) {
        tracing::info!("combat: {id:#x} wears {m:?} (the melee's materials, by part)");
    }
    worn.retain(|_, (at, _)| now - *at < 10.0);
    worn.insert(id, (now, m));
    m
}

/// How hard a blow lands, for the game's own hit (1: a full one, the game's own attack): how hard it
/// was swung (the runtime's strength, by its speed past the least for its kind up to a full swing:
/// a tap next to nothing, a cut at 7 m/s a whole blow), times what the runtime makes of this
/// blow against the same blade's clean one at that speed (`rustvr_melee::judge`: a bad angle's blunt
/// blow, the flat, a fraction; thrown, its thrown multiplier, a dagger's x3), times the body part's
/// (a head x3, the throat and the trunk x1, a limb x0.5, a hand or a foot x0.25). The armour's
/// multiplier is taken back out: what the creature wears is the game's to count (its armour rating),
/// the runtime's armour only says whether it lands at all. `blade` is the damager that struck (a hand's own:
/// the runtime's punch).
pub fn how_hard(ev: &CombatEvent, blade: &MeleeBlade, tier: u32, part: rustvr_melee::body::Part) -> f32 {
    use rustvr_melee::*;
    let glancing = ev.flags & CEF_GLANCING != 0;
    let modifier = if glancing { bad_angle(blade).0 } else { modifier_of(blade) };
    let armour = row(modifier, ev.material, tier, false).map_or(1.0, |r| r.damage);
    let (c, slopes) = curve_of(blade);
    let clean = curve(c, slopes, ev.speed) * if blade.tier > 0.0 { blade.tier } else { 1.0 };
    if armour <= 0.0 || clean <= 0.0 {
        return 0.0;
    }
    (ev.strength * ev.damage / armour / clean * part.damage()).clamp(0.0, MOST_HARD)
}

/// A blade gone in deep ([`CE_WOUND`]), as the runtime wounds: its penetration damage (or, out
/// the far side, its skewer damage) times the part's (a point's deep multiplier, a cut's, a
/// skewer's). The blow that put it in was already the game's hit; what the runtime makes of the wound past
/// what it makes of the same wound in the body's trunk is one more (a throat cut x32, a point deep in
/// a head x16: those kill), the trunk's own nothing more.
pub fn wound_beyond(ev: &CombatEvent, part: rustvr_melee::body::Part) -> f32 {
    use rustvr_melee::body::Part;
    let (here, trunk) = if ev.flags & CEF_SKEWER != 0 {
        (part.skewer(), Part::Torso.skewer())
    } else if ev.what == DMG_PIERCE {
        (part.deep().1, Part::Torso.deep().1)
    } else {
        (part.cut(), Part::Torso.cut())
    };
    (here / trunk - 1.0).clamp(0.0, MOST_HARD)
}

/// No blow is worth more than this many of the game's own (its overflow, and a sane end to the
/// deadliest wounds, which kill well before it).
const MOST_HARD: f32 = 64.0;

/// The part a blow met, as the runtime tells them apart: the game's part (its index in [`HUMANOID`], or
/// `u32::MAX` for a creature's one capsule), and the runtime's (the throat the lower part of the head's
/// capsule and the top of the trunk's).
#[derive(Clone, Copy, Debug)]
pub struct Struck {
    pub game_part: u32,
    pub kind: u32,
    pub body: rustvr_melee::body::Part,
}

/// The part of `ev`'s creature it met, from the creatures as published: by its game part (a hit's),
/// or by its index in the actor's parts (`by_index`: a blade in it, [`CE_WOUND`]).
pub fn struck_part(table: &Actors, ev: &CombatEvent, by_index: bool) -> Option<Struck> {
    let a = table.actors[..(table.count as usize).min(MAX_ACTORS)].iter().find(|a| a.id == ev.actor)?;
    let parts = &a.parts[..(a.count as usize).min(MAX_ACTOR_PARTS)];
    let p = if by_index { parts.get(ev.part as usize)? } else { parts.iter().find(|p| p.game_part == ev.part)? };
    let throat = rustvr_melee::body::on_throat(p.kind, Vec3::from_array(p.a), Vec3::from_array(p.b), Vec3::from_array(ev.point));
    Some(Struck { game_part: p.game_part, kind: p.kind, body: rustvr_melee::body::Part::of(p.kind, throat) })
}

/// The game part of `ev`'s creature whose capsule is nearest where the blow met (the runtime names the part
/// a cut takes off, which may be the one beside it; the gore cuts at the joint the blade crossed).
fn nearest_part(table: &Actors, ev: &CombatEvent) -> Option<u32> {
    let a = table.actors[..(table.count as usize).min(MAX_ACTORS)].iter().find(|a| a.id == ev.actor)?;
    let p = Vec3::from_array(ev.point);
    let off = |q: &ActorPart| {
        let (a, b) = (Vec3::from_array(q.a), Vec3::from_array(q.b));
        let t = ((p - a).dot(b - a) / (b - a).length_squared().max(1e-8)).clamp(0.0, 1.0);
        p.distance(a.lerp(b, t)) - q.radius
    };
    a.parts[..(a.count as usize).min(MAX_ACTOR_PARTS)].iter().min_by(|x, y| off(x).total_cmp(&off(y))).map(|q| q.game_part)
}

/// The push level of a blow (0..4: how hard it shoves what it met, by the speed against its
/// table's row for the material).
pub fn push_level(ev: &CombatEvent, blade: &MeleeBlade, tier: u32) -> u32 {
    let modifier = if ev.flags & CEF_GLANCING != 0 { rustvr_melee::bad_angle(blade).0 } else { rustvr_melee::modifier_of(blade) };
    rustvr_melee::row(modifier, ev.material, tier, false).map_or(0, |r| r.push_level(ev.speed, ev.flags & CEF_THROWN != 0))
}

/// The damager that struck (the weapon's own blade, a hand's punch) and its weapon's tier.
fn blade_of(ev: &CombatEvent) -> (MeleeBlade, u32) {
    match crate::weapons::melee_weapon(ev.object).and_then(|w| w.blades().get(ev.blade as usize).map(|b| (*b, w.tier))) {
        Some(b) => b,
        None => (rustvr_melee::weapons::punch(0.045), 1),
    }
}

/// Every actor the game is fully processing near the player (not the player).
pub(crate) fn high_actors() -> Vec<usize> {
    let player = engine::player();
    let mut out = Vec::new();
    let mut node = PROCESS_MANAGER + ACTORS_LIST;
    let mut guard = 0;
    while node != 0 && guard < 256 {
        guard += 1;
        let actor = mem::deref(node);
        node = mem::deref(node + 4);
        if actor != player && is_actor(actor) {
            out.push(actor);
        }
    }
    out
}

/// The actors near the player and, by id, where each is in the game's memory.
pub fn actors() -> (Box<Actors>, HashMap<u32, usize>) {
    let mut table: Box<Actors> = bytemuck::zeroed_box();
    let mut by_id = HashMap::new();
    let player = engine::player();
    let me = mem::at::<[f32; 3]>(player + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)));
    let mut node = PROCESS_MANAGER + ACTORS_LIST;
    let mut guard = 0;
    while node != 0 && guard < 256 && (table.count as usize) < MAX_ACTORS {
        guard += 1;
        let actor = mem::deref(node);
        node = mem::deref(node + 4);
        // (The horse the player rides is under them, not in the hands' way.)
        if actor == player || !is_actor(actor) || actor == crate::riding::horse().0 {
            continue;
        }
        let Some(id) = mem::at::<u32>(actor + FORM_ID) else { continue };
        let at = mem::at::<[f32; 3]>(actor + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)));
        if let (Some(me), Some(at)) = (me, at)
            && me.distance(at) > ACTORS_RANGE
        {
            continue;
        }
        let root = mem::deref(actor + NI_NODE);
        if root == 0 {
            continue;
        }
        let b = bones(root);
        let mut a: Actor = bytemuck::Zeroable::zeroed();
        a.id = id;
        let fitted = crate::shapes::fitted(id, root, &b);
        let worn = materials(id, actor);
        for (part, &(from, to, r, kind, _)) in HUMANOID.iter().enumerate() {
            let material = worn[part];
            // (Not a limb the gore has cut off: a head goes with its neck.)
            if crate::gore::cut_off(id, from) || (kind == PART_HEAD && crate::gore::cut_off(id, to)) {
                continue;
            }
            let (Some(p), Some(q)) = (b.get(from).and_then(|&n| node_pos(n)), b.get(to).and_then(|&n| node_pos(n))) else { continue };
            let along = (q - p).normalize_or(Vec3::Y);
            // (A part cut at its end bone ends at the stump: nothing of it reaches past.)
            let stump = crate::gore::cut_at(id, to);
            let (p, q, r) = match fitted.get(part).copied().flatten() {
                // As thick as what it wears, and on past its free ends to what's drawn there.
                Some(f) => (p - along * f.start, if stump { q } else { q + along * f.end }, f.radius),
                // The head: from the neck up through the crown (while it's there).
                None if kind == PART_HEAD && !stump => (p, q + along * 0.12, r),
                None => (p, q, r),
            };
            let i = a.count as usize;
            a.parts[i] = ActorPart { a: p.to_array(), radius: r, b: q.to_array(), kind, material, game_part: part as u32, _pad: [0; 2] };
            a.count += 1;
        }
        if a.count == 0 {
            // Not a humanoid (a creature): one capsule from its feet to its height (no bone of
            // [`HUMANOID`]'s: its frame is along it).
            let Some(feet) = mem::at::<[f32; 3]>(actor + refr::POS) else { continue };
            let f = camera::basis().point_from_game(Vec3::from(feet));
            a.parts[0] = ActorPart { a: (f + Vec3::Y * 0.3).to_array(), radius: 0.35, b: (f + Vec3::Y * 1.3).to_array(), kind: PART_TORSO, material: MAT_FLESH, game_part: u32::MAX, _pad: [0; 2] };
            a.count = 1;
        }
        match current_action(actor) {
            ACTION_ATTACK | ACTION_ATTACK_FOLLOW => {
                if let Some((wa, wb, r)) = attack_segment(actor, &b) {
                    a.flags |= ACT_ATTACKING;
                    a.weapon_a = wa.to_array();
                    a.weapon_b = wb.to_array();
                    a.weapon_radius = r;
                }
            }
            ACTION_BLOCK => a.flags |= ACT_BLOCKING,
            _ => {}
        }
        let health = actor_value(actor, AV_HEALTH);
        let most = base_actor_value(actor, AV_HEALTH).max(1);
        a.flags |= if health > 0 { ACT_ALIVE } else { 0 };
        a.health = (health as f32 / most as f32).clamp(0.0, 1.0);
        let i = table.count as usize;
        table.actors[i] = a;
        table.count += 1;
        by_id.insert(id, actor);
    }
    crate::shapes::forget_others(&by_id);
    (table, by_id)
}

#[cfg(target_arch = "x86")]
fn active_magic_item(player: usize) -> usize {
    type F = unsafe extern "thiscall" fn(usize) -> usize;
    unsafe { std::mem::transmute::<usize, F>(ACTIVE_MAGIC_ITEM)(player) }
}

#[cfg(not(target_arch = "x86"))]
fn active_magic_item(_player: usize) -> usize {
    0
}

/// The player's spells now, and each one's `SpellItem` by id.
pub fn spells() -> (Box<Spells>, HashMap<u32, usize>) {
    let mut table: Box<Spells> = bytemuck::zeroed_box();
    let mut by_id = HashMap::new();
    let player = engine::player();
    if player == 0 {
        return (table, by_id);
    }
    let magicka = actor_value(player, AV_MAGICKA) as f32;
    table.magicka = magicka;
    table.magicka_max = base_actor_value(player, AV_MAGICKA) as f32;
    table.cast_time = 0.5;
    // The game's own casting art goes where the runtime's casting is (`crate::magic`).
    table.flags = SPELLS_GAME_ART;
    let active = active_magic_item(player);
    static ACTIVE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);
    if ACTIVE.swap(active, std::sync::atomic::Ordering::Relaxed) != active {
        tracing::info!("combat: the player's active magic item {active:#x}");
    }
    let base = mem::deref(player + BASE_FORM);
    let mut entry = base + SPELL_LIST;
    let mut guard = 0;
    while entry != 0 && guard < 512 && (table.count as usize) < MAX_SPELLS {
        guard += 1;
        let spell = mem::deref(entry);
        entry = mem::deref(entry + 4);
        if spell == 0 {
            continue;
        }
        let kind = mem::at::<u32>(spell + SPELL_TYPE).unwrap_or(u32::MAX);
        if !CASTABLE_TYPES.contains(&kind) {
            continue;
        }
        let Some(id) = mem::at::<u32>(spell + FORM_ID) else { continue };
        let mut s: Spell = bytemuck::Zeroable::zeroed();
        s.id = id;
        s.set_name(&read_str(mem::deref(spell + MAGIC_ITEM + 4)));
        // Delivery: target if any effect is, else touch if any is, else on the player.
        let (mut range, mut colour) = (DELIVER_SELF, None);
        let mut e = spell + SPELL_EFFECTS;
        let mut n = 0;
        while e != 0 && n < 16 {
            n += 1;
            let effect = mem::deref(e);
            e = mem::deref(e + 4);
            if effect == 0 {
                continue;
            }
            let r = mem::at::<u32>(effect + EFFECT_RANGE).unwrap_or(0);
            range = range.max(r.min(2));
            if colour.is_none() {
                let light = mem::deref(looks_like(effect) + SETTING_LIGHT);
                colour = mem::at::<[u8; 4]>(light + LIGHT_COLOUR).filter(|_| light != 0);
            }
        }
        s.delivery = range;
        let c = colour.unwrap_or([255, 220, 160, 0]);
        s.color = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
        s.cost = mem::at::<u32>(spell + SPELL_COST).unwrap_or(0) as f32;
        s.flags = if kind != 0 { SPELL_POWER } else { 0 };
        if spell + MAGIC_ITEM == active {
            s.flags |= SPELL_SELECTED;
        }
        if kind != 0 || s.cost <= magicka {
            s.flags |= SPELL_CASTABLE;
        }
        let i = table.count as usize;
        table.spells[i] = s;
        table.count += 1;
        by_id.insert(id, spell);
    }
    (table, by_id)
}

/// The player's spell with form id `id` (its `SpellItem`), as last published.
pub fn spell_item(id: u32) -> Option<usize> {
    KNOWN.lock().unwrap().as_ref()?.spells.get(&id).copied()
}

/// A cast the adapter is making: the projectile goes from here this way (game units, radians).
#[derive(Clone, Copy, Debug)]
pub struct Launch {
    pub pos: [f32; 3],
    pub pitch: f32,
    pub heading: f32,
}

static LAUNCH: Mutex<Option<Launch>> = Mutex::new(None);

/// The launch of the cast being made now, if any (for the projectile spawn hook).
pub fn launch() -> Option<Launch> {
    *LAUNCH.lock().unwrap()
}

/// Angles (pitch looking down positive, heading 0 north turning east) of a direction in the
/// protocol's space.
pub fn angles(dir: Vec3) -> (f32, f32) {
    let d = camera::basis().dir_to_game(dir).normalize_or(Vec3::Y);
    (-d.z.clamp(-1.0, 1.0).asin(), d.x.atan2(d.y))
}

/// What the hands did, done by the game. `actors` and `spells` map ids to the game's objects.
#[cfg(target_arch = "x86")]
pub fn apply(events: &[CombatEvent], actors: &HashMap<u32, usize>, table: &Actors, spells: &HashMap<u32, usize>) {
    let player = engine::player();
    if player == 0 {
        return;
    }
    // The game's rule: no fighting from the saddle (nor casting). The hands still swing, as a
    // rider's would, but nothing lands.
    if crate::riding::mounted() {
        if !events.is_empty() {
            tracing::info!("combat: {} blows or casts from the saddle: the game doesn't fight mounted", events.len());
        }
        return;
    }
    for ev in events {
        match ev.kind {
            CE_HIT => {
                // (Gone since it was published: the game's frame ran in between.)
                let Some(&target) = actors.get(&ev.actor).filter(|&&a| is_actor(a)) else { continue };
                let Some(part) = struck_part(table, ev, false) else { continue };
                let (blade, tier) = blade_of(ev);
                // (In at its going-in speed but under the material's least for a hit, it goes
                // in without hurting.)
                if ev.damage <= 0.0 {
                    tracing::info!("combat: {:#x}'s part {} met at {:.1} m/s: in, without hurting", ev.actor, ev.part, ev.speed);
                    continue;
                }
                let scale = how_hard(ev, &blade, tier, part.body);
                // (A blow that pushes the body at all by the push tables is a power attack: a cut at
                // 8 m/s on flesh, a mace at 7.5.)
                let power = push_level(ev, &blade, tier) >= 1;
                let before = crate::gore::health(target);
                tracing::info!(
                    "combat: the player hits {:#x}'s {:?} ({} on {}) at {:.1} m/s: judged x{scale:.2}{}{}",
                    ev.actor,
                    part.body,
                    match ev.what {
                        DMG_SLASH => "an edge",
                        DMG_PIERCE => "a point",
                        DMG_PUNCH => "a fist",
                        _ if ev.flags & CEF_GLANCING != 0 => "a flat",
                        _ => "a blunt blow",
                    },
                    ev.material,
                    ev.speed,
                    if power { ", a power attack" } else { "" },
                    if ev.flags & CEF_THROWN != 0 { ", thrown" } else { "" }
                );
                let heading = mem::at::<[f32; 3]>(player + refr::ROT).map_or(0.0, |r| r[2]);
                let group = power.then(|| power_group(Vec3::from_array(ev.dir), heading, ev.what == DMG_PIERCE));
                // (A shield's bash is a blow of the arm: the game has none of its own.)
                game_hit(player, target, group, scale, ev.object == 0 || ev.object == crate::weapons::SHIELD_ID);
                // (A blow that kills, or one into a corpse, may take a limb: the gore's, at the joint
                // nearest where the blade met.)
                let mut cut = *ev;
                cut.part = nearest_part(table, ev).unwrap_or(part.game_part);
                crate::gore::after_hit(target, &cut, before);
                // What the melee's tables make of it besides: a knockout, a disarm.
                after_blow(target, ev, &blade, tier, &part);
            }
            CE_WOUND => {
                let Some(&target) = actors.get(&ev.actor).filter(|&&a| is_actor(a)) else { continue };
                let Some(part) = struck_part(table, ev, true) else { continue };
                let more = wound_beyond(ev, part.body);
                tracing::info!("combat: a blade {} {:#x}'s {:?} ({:.3} m): x{more:.2} more", if ev.flags & CEF_SKEWER != 0 { "out through" } else { "deep in" }, ev.actor, part.body, ev.depth);
                if more > 0.0 {
                    game_hit(player, target, None, more, false);
                }
            }
            CE_CAST if ev.object == crate::weapons::WEAPON_ID => cast_staff(player, ev, actors),
            CE_CAST => {
                let Some(&spell) = spells.get(&ev.what) else { continue };
                cast(player, spell, ev, actors);
            }
            CE_BLOCK => {
                tracing::info!("combat: the player blocks {:#x}{}", ev.actor, if ev.flags & CEF_PARRY != 0 { " (a parry)" } else { "" });
                crate::player::hold_control("Block", BLOCK_HOLD);
            }
            CE_LOOSE => crate::archery::loose(ev),
            _ => {}
        }
    }
}

/// The game's own hit on `target` (the player's hit routine, as the attack animation lands it:
/// the game's damage, skill, sneak attack, enchantment, armour, block, sounds, blood, OnHit and
/// stagger), as hard as the runtime judged the blow: its damage formula's own multiplier
/// (`fDamageWeaponMult`; a fist's `fHandHealthMin` and `Max`) times `scale` while it runs, so all
/// that follows from the damage follows (the stagger's chance, the knockback, wear, the death
/// push). A power attack (`power`: the attack's anim group, [`power_group`]) is the game's own in
/// that direction (its bonus by the skill's mastery and its perks). A fist (`unarmed`) is the
/// game's hand-to-hand, the weapon out of the process's hand while it runs.
#[cfg(target_arch = "x86")]
fn game_hit(player: usize, target: usize, power: Option<u8>, scale: f32, unarmed: bool) {
    type Hit = unsafe extern "thiscall" fn(usize, u8, usize, usize);
    type AnimData = unsafe extern "thiscall" fn(usize) -> usize;
    let f = mem::deref(mem::deref(player) + VT_HIT);
    let anim_f = mem::deref(mem::deref(player) + VT_ANIM_DATA);
    if !engine::CODE.contains(&f) || !engine::CODE.contains(&anim_f) {
        return;
    }
    let before = actor_value(target, AV_HEALTH);
    let settings = [DAMAGE_WEAPON_MULT, HAND_HEALTH_MIN, HAND_HEALTH_MAX];
    let saved = settings.map(|a| mem::at::<f32>(a).unwrap_or(1.0));
    let anim = unsafe { std::mem::transmute::<usize, AnimData>(anim_f)(player) };
    let group = mem::at::<u16>(anim + ANIM_GROUP).filter(|_| anim != 0);
    let hand = mem::deref(player + 0x58) + EQUIPPED_WEAPON;
    let weapon = mem::deref(hand);
    unsafe {
        for (a, v) in settings.iter().zip(saved) {
            (*a as *mut f32).write(v * scale);
        }
        if let (Some(g), Some(p)) = (group, power) {
            ((anim + ANIM_GROUP) as *mut u16).write(g & 0xFF00 | p as u16);
        }
        if unarmed && hand != EQUIPPED_WEAPON {
            (hand as *mut usize).write(0);
        }
        std::mem::transmute::<usize, Hit>(f)(player, power.is_some() as u8, 0, target);
        if unarmed && hand != EQUIPPED_WEAPON {
            (hand as *mut usize).write(weapon);
        }
        if let Some(g) = group {
            ((anim + ANIM_GROUP) as *mut u16).write(g);
        }
        for (a, v) in settings.iter().zip(saved) {
            (*a as *mut f32).write(v);
        }
    }
    tracing::info!("combat: the game's hit (x{scale:.2}{}{}) took {} of {before} health", if power.is_some() { ", a power attack" } else { "" }, if unarmed { ", hand to hand" } else { "" }, before - actor_value(target, AV_HEALTH));
}

#[cfg(not(target_arch = "x86"))]
fn game_hit(_player: usize, _target: usize, _power: Option<u8>, _scale: f32, _unarmed: bool) {}

/// The game's power attack a blow is, by the way it went from where the player faces (its anim
/// groups: a thrust or a blow driven ahead `AttackForwardPower`, one drawn back `AttackBackPower`, one
/// across `AttackLeftPower` or `AttackRightPower`, one down or up `AttackPower`): `dir` the blow's
/// way (the protocol's axes), `heading` the player's (the game's: 0 north, turning east).
pub fn power_group(dir: Vec3, heading: f32, stab: bool) -> u8 {
    const STAND: u8 = 22;
    const FORWARD: u8 = 23;
    const BACK: u8 = 24;
    const LEFT: u8 = 25;
    const RIGHT: u8 = 26;
    let d = camera::basis().dir_to_game(dir);
    let (s, c) = heading.sin_cos();
    let (ahead, right, up) = (d.x * s + d.y * c, d.x * c - d.y * s, d.z);
    if stab || (ahead > right.abs() && ahead > up.abs()) {
        FORWARD
    } else if -ahead > right.abs() && -ahead > up.abs() {
        BACK
    } else if right.abs() > up.abs() {
        if right > 0.0 { RIGHT } else { LEFT }
    } else {
        STAND
    }
}

/// Takes `amount` off an actor value (less than nothing gives it back), the player its cause
/// (`Actor` vtable +0x2A4, `DamageActorValue`: the game's own, which kills at no health).
#[cfg(target_arch = "x86")]
fn damage_av(actor: usize, av: u32, amount: f32) {
    type F = unsafe extern "thiscall" fn(usize, u32, f32, usize);
    let f = mem::deref(mem::deref(actor) + VT_DAMAGE_AV);
    if engine::CODE.contains(&f) {
        unsafe { std::mem::transmute::<usize, F>(f)(actor, av, -amount, engine::player()) };
    }
}

/// What the melee's tables make of a blow besides its damage, done the game's way (as the range's
/// creatures take them: the framework's `docs/MELEE.md`):
/// * a clean blunt blow to the head faster than its table's knockout speed knocks the creature out
///   cold: the game's own knockout, its Fatigue taken below nothing;
/// * a blow on a hand at the disarming push level (2; 3 in heavy gauntlets), or a point through
///   the arm, makes it drop its weapon: the game's own disarm.
#[cfg(target_arch = "x86")]
fn after_blow(target: usize, ev: &CombatEvent, blade: &MeleeBlade, tier: u32, part: &Struck) {
    let glancing = ev.flags & CEF_GLANCING != 0;
    let modifier = if glancing { rustvr_melee::bad_angle(blade).0 } else { rustvr_melee::modifier_of(blade) };
    let Some(row) = rustvr_melee::row(modifier, ev.material, tier, false) else { return };
    let thrown = ev.flags & CEF_THROWN != 0;
    if actor_value(target, AV_HEALTH) <= 0 {
        return;
    }
    if ev.what == DMG_BLUNT && !glancing && part.body == rustvr_melee::body::Part::Head && ev.speed >= if thrown { row.knockout_thrown } else { row.knockout } {
        let fatigue = actor_value(target, AV_FATIGUE).max(0) as f32;
        let most = base_actor_value(target, AV_FATIGUE).max(1) as f32;
        damage_av(target, AV_FATIGUE, fatigue + most);
        tracing::info!("combat: {:#x} knocked out ({:.1} m/s to the head)", ev.actor, ev.speed);
    }
    let level = push_level(ev, blade, tier);
    let disarm_level = if ev.material == MAT_PLATE { 3 } else { 2 };
    let on_hand = part.kind == PART_HAND && level >= disarm_level;
    let through_arm = matches!(part.kind, PART_ARM | PART_HAND) && ev.what == DMG_PIERCE && ev.flags & CEF_PENETRATED != 0;
    if on_hand || through_arm {
        tracing::info!("combat: {:#x} disarmed ({}, push level {level})", ev.actor, if on_hand { "its hand struck" } else { "stabbed through the arm" });
        disarm(target, Some(Vec3::from_array(ev.point)));
    }
}

#[cfg(not(target_arch = "x86"))]
fn after_blow(_target: usize, _ev: &CombatEvent, _blade: &MeleeBlade, _tier: u32, _part: &Struck) {}

/// The creature drops the weapon it holds, as the game's own disarm does it (its power attack's
/// perk, 0x5FC090): its drawn weapon (0x5E4A80) dropped where its hand holds it (the actor's drop,
/// vtable +0x2C8: the next weapon equipped), the dropped one still its own (0x4DC000), its attack
/// cancelled (0x5EFFD0) and its combat set to go for a weapon (0x61DD10).
#[cfg(target_arch = "x86")]
fn disarm(actor: usize, hand_at: Option<Vec3>) {
    type Drawn = unsafe extern "thiscall" fn(usize) -> usize;
    type Drop = unsafe extern "thiscall" fn(usize, usize, usize, u32, *const [f32; 3], *const [f32; 3]) -> usize;
    type Own = unsafe extern "C" fn(usize, usize);
    type Action = unsafe extern "thiscall" fn(usize) -> i32;
    type SetAction = unsafe extern "thiscall" fn(usize, i32, u32);
    type Controller = unsafe extern "thiscall" fn(usize) -> usize;
    type Reset = unsafe extern "thiscall" fn(usize);
    let drop_f = mem::deref(mem::deref(actor) + VT_DROP);
    let controller_f = mem::deref(mem::deref(actor) + VT_COMBAT_CONTROLLER);
    if !engine::CODE.contains(&drop_f) || !engine::CODE.contains(&controller_f) {
        return;
    }
    let entry = unsafe { std::mem::transmute::<usize, Drawn>(DRAWN_WEAPON)(actor) };
    if entry == 0 {
        return;
    }
    let form = mem::deref(entry + 8);
    let extra = mem::deref(mem::deref(entry));
    let at = hand_at.map(|p| camera::basis().point_to_game(p).to_array());
    let dropped = unsafe { std::mem::transmute::<usize, Drop>(drop_f)(actor, form, extra, 1, at.as_ref().map_or(std::ptr::null(), |p| p as *const _), std::ptr::null()) };
    if dropped == 0 {
        return;
    }
    unsafe {
        std::mem::transmute::<usize, Own>(OWN_DROPPED)(actor, dropped);
        if (0..=5).contains(&std::mem::transmute::<usize, Action>(CURRENT_ACTION)(actor)) {
            std::mem::transmute::<usize, SetAction>(SET_ACTION)(actor, -1, 0);
        }
        let controller = std::mem::transmute::<usize, Controller>(controller_f)(actor);
        if controller != 0 {
            std::mem::transmute::<usize, Reset>(RESET_COMBAT)(controller);
        }
    }
}

/// While the game casts for the hand: the projectile leaves `ev.point` along `ev.dir` (the launch
/// the projectile spawn takes) and the player faces that way (the game aims some things by where
/// the caster faces); `f` casts.
#[cfg(target_arch = "x86")]
fn launching(player: usize, ev: &CombatEvent, f: impl FnOnce()) {
    let dir = Vec3::from_array(ev.dir);
    let (pitch, heading) = angles(dir);
    let pos = camera::basis().point_to_game(Vec3::from_array(ev.point)).to_array();
    *LAUNCH.lock().unwrap() = Some(Launch { pos, pitch, heading });
    let rot = (player + refr::ROT) as *mut f32;
    let (old_pitch, old_heading) = unsafe { (rot.read(), rot.add(2).read()) };
    unsafe {
        rot.write(pitch);
        rot.add(2).write(heading.rem_euclid(std::f32::consts::TAU));
    }
    f();
    unsafe {
        rot.write(old_pitch);
        rot.add(2).write(old_heading);
    }
    *LAUNCH.lock().unwrap() = None;
}

/// A staff's cast: the game's own (0x5E4920, as its staff attack does it: the enchantment's cost
/// taken from the staff's charge, cast, or the game's "not enough charge"), from its tip.
#[cfg(target_arch = "x86")]
fn cast_staff(player: usize, ev: &CombatEvent, _actors: &HashMap<u32, usize>) {
    let entry = mem::deref(mem::deref(player + 0x58) + EQUIPPED_WEAPON);
    if entry == 0 {
        return;
    }
    tracing::info!("combat: the player's staff casts {:#x} from {:?} towards {:?}", ev.what, ev.point, ev.dir);
    type UseStaff = unsafe extern "thiscall" fn(usize, usize);
    launching(player, ev, || unsafe { std::mem::transmute::<usize, UseStaff>(USE_STAFF)(player, entry) });
}

#[cfg(target_arch = "x86")]
fn cast(player: usize, spell: usize, ev: &CombatEvent, actors: &HashMap<u32, usize>) {
    let caster = player + MAGIC_CASTER;
    let item = spell + MAGIC_ITEM;
    let vt = mem::deref(caster);
    type Usable = unsafe extern "thiscall" fn(usize, usize, *mut f32, *mut u32, u8) -> u8;
    type Cost = unsafe extern "thiscall" fn(usize, usize, u8);
    type Cast = unsafe extern "thiscall" fn(usize, usize, usize, usize) -> usize;
    let (mut skill, mut failure) = (0f32, 0u32);
    let usable = unsafe { std::mem::transmute::<usize, Usable>(mem::deref(vt + VT_USABLE))(caster, item, &mut skill, &mut failure, 0) } != 0;
    if !usable {
        tracing::info!("combat: the game won't cast {:#x} now (failure {failure})", ev.what);
        return;
    }
    unsafe { std::mem::transmute::<usize, Cost>(mem::deref(vt + VT_APPLY_COST))(caster, item, 1) };
    let target = if ev.actor != 0 { actors.get(&ev.actor).copied().filter(|&a| is_actor(a)).unwrap_or(0) } else { 0 };
    tracing::info!("combat: the player casts {:#x} from {:?} towards {:?}{}", ev.what, ev.point, ev.dir, if target != 0 { " on its target" } else { "" });
    launching(player, ev, || unsafe {
        std::mem::transmute::<usize, Cast>(CAST_IMMEDIATE)(caster, item, target, 0);
    });
    crate::mirror_spell::cast(spell);
}

#[cfg(not(target_arch = "x86"))]
pub fn apply(_events: &[CombatEvent], _actors: &HashMap<u32, usize>, _table: &Actors, _spells: &HashMap<u32, usize>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blow_is_as_hard_as_bas_judges_it_against_a_full_one() {
        use rustvr_melee::body::Part;
        use rustvr_melee::weapons::*;
        let blade = blade(SWORD_SLASH, Vec3::ZERO, Vec3::Z, 0.02, Vec3::X, 0.01);
        let judged = |speed: f32, material: u32, flags: u32| {
            let c = rustvr_melee::Contact { along: Vec3::Z, edge: Vec3::X, vel: if flags & CEF_GLANCING != 0 { Vec3::Y * speed } else { Vec3::X * speed }, normal: if flags & CEF_GLANCING != 0 { -Vec3::Y } else { -Vec3::X }, material, tier: 2, thrown: false, fire: false };
            let j = rustvr_melee::judge(&blade, &c).unwrap();
            // (The runtime's strength: the speed past the kind's least, 3 m/s for a cut and 1 for a
            // blunt blow, up to its full one.)
            let least = if j.kind == DMG_SLASH { 3.0 } else { 1.0 };
            let strength = ((speed - least) / (rustvr_melee::full_speed(j.kind) - least)).clamp(0.05, 1.5);
            CombatEvent { kind: CE_HIT, flags: j.flags, what: j.kind, material, speed, strength, damage: j.damage, ..Default::default() }
        };
        // A full cut on the trunk is the game's own blow; on the head three of them.
        let full = how_hard(&judged(7.0, MAT_FLESH, 0), &blade, 2, Part::Torso);
        assert!((full - 1.0).abs() < 0.02, "{full}");
        assert!((how_hard(&judged(7.0, MAT_FLESH, 0), &blade, 2, Part::Head) - 3.0).abs() < 0.05);
        // The armour's own share is the game's: plate that lets a fast cut land doesn't weaken it.
        let plate = how_hard(&judged(9.0, MAT_PLATE, 0), &blade, 2, Part::Torso);
        assert!(plate > 1.0, "{plate}");
        // The flat: a light slap; a tap next to nothing.
        let flat = how_hard(&judged(7.0, MAT_FLESH, CEF_GLANCING), &blade, 2, Part::Torso);
        assert!(flat > 0.0 && flat < 0.6, "{flat}");
        assert!(how_hard(&judged(3.5, MAT_FLESH, 0), &blade, 2, Part::Torso) < 0.2);
        // A throat cut gone in kills; one in the trunk is the blow's own.
        let wound = CombatEvent { kind: CE_WOUND, what: DMG_SLASH, ..Default::default() };
        assert!(wound_beyond(&wound, Part::Neck) > 10.0 && wound_beyond(&wound, Part::Torso) == 0.0);
    }

    #[test]
    fn aim_angles_match_the_games_rotation() {
        // North (canonical +Z), level: heading 0, pitch 0.
        let (p, h) = angles(Vec3::Z);
        assert!(p.abs() < 1e-5 && h.abs() < 1e-5);
        // East (canonical +X): a quarter turn.
        let (_, h) = angles(Vec3::X);
        assert!((h - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        // Up: pitch negative (positive looks down).
        let (p, _) = angles(Vec3::new(0.0, 1.0, 1.0));
        assert!(p < -0.7);
    }
}

struct Known {
    actors: HashMap<u32, usize>,
    /// The creatures as last published (their parts: what a hit's part was).
    table: Box<Actors>,
    spells: HashMap<u32, usize>,
    /// The spells as last sent (sent again when they change).
    sent: Option<Vec<(u32, u32, u32)>>,
    frame: u64,
}

static KNOWN: Mutex<Option<Known>> = Mutex::new(None);

/// Once a game frame (in the world): the creatures, and the spells when they changed.
pub fn publish() {
    let (table, actors) = actors();
    let mut known = KNOWN.lock().unwrap();
    let k = known.get_or_insert_with(|| Known { actors: HashMap::new(), table: bytemuck::zeroed_box(), spells: HashMap::new(), sent: None, frame: 0 });
    k.actors = actors;
    k.frame += 1;
    if k.frame % 60 == 2 {
        log_actors(&table);
    }
    let spells_now = (k.frame % 10 == 1).then(spells);
    crate::link::with_adapter(|a| {
        crate::weapons::publish(a);
        crate::melee::publish(a);
        crate::torch::publish(a);
        crate::map::publish(a);
        // The face shaped by hand in the race menu, into the character (before it's published).
        crate::facesculpt::update(a);
        // (Every frame while the race menu changes the body and the face as the player watches.)
        if k.frame % 30 == 3 || crate::menus::menu_open(crate::menus::MENU_RACE_SEX) {
            crate::body::publish(a);
        }
        a.write_actors(&table);
        k.table.clone_from(&table);
        crate::grabs::publish(a, &table, &k.actors);
        if let Some((s, by_id)) = spells_now {
            let key: Vec<(u32, u32, u32)> = s.spells[..s.count as usize].iter().map(|x| (x.id, x.flags, x.cost as u32)).collect();
            if k.sent.as_ref() != Some(&key) {
                tracing::info!(
                    "combat: spells {:?}, selected {:?}, magicka {}",
                    s.spells[..s.count as usize].iter().map(|x| format!("{} ({:#x}, delivery {}, cost {})", x.name(), x.id, x.delivery, x.cost)).collect::<Vec<_>>(),
                    s.spells[..s.count as usize].iter().find(|x| x.flags & SPELL_SELECTED != 0).map(|x| x.name().to_owned()),
                    s.magicka
                );
                a.write_spells(&s);
                k.sent = Some(key);
            }
            k.spells = by_id;
        }
    });
}

/// The creatures published, where each is from the player (metres, the protocol's axes).
fn log_actors(table: &Actors) {
    let player = engine::player();
    let Some(feet) = mem::at::<[f32; 3]>(player + refr::POS) else { return };
    let me = camera::basis().point_from_game(Vec3::from(feet));
    let list: Vec<String> = table.actors[..table.count as usize]
        .iter()
        .map(|a| {
            let c = (Vec3::from_array(a.parts[0].a) + Vec3::from_array(a.parts[0].b)) * 0.5 - me;
            format!("{:#x} at ({:.1}, {:.1}, {:.1}) health {:.2} parts {} flags {:#x}", a.id, c.x, c.y, c.z, a.health, a.count, a.flags)
        })
        .collect();
    tracing::info!("combat: {} creatures: {}", list.len(), list.join("; "));
}

/// The creatures published last (by id: where each is in the game's memory; check it's still an
/// actor, [`is_actor`], before using it).
pub fn known_actors() -> HashMap<u32, usize> {
    KNOWN.lock().unwrap().as_ref().map(|k| k.actors.clone()).unwrap_or_default()
}

/// The bones a body part's capsule runs between (its `game_part`), for a humanoid.
pub fn part_bones(part: u32) -> Option<(&'static str, &'static str)> {
    HUMANOID.get(part as usize).map(|&(from, to, ..)| (from, to))
}

/// Before the game's frame: what the hands did since the last one, done by the game.
pub fn apply_pending() {
    let Some(events) = crate::link::with_adapter(|a| a.combat_events()) else { return };
    if events.is_empty() {
        return;
    }
    let known = KNOWN.lock().unwrap();
    let Some(k) = known.as_ref() else { return };
    apply(&events, &k.actors, &k.table, &k.spells);
}

#[cfg(target_arch = "x86")]
mod hook {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static REAL: AtomicUsize = AtomicUsize::new(0);
    type Spawn = unsafe extern "C" fn(u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) -> u32;

    pub fn install() {
        crate::hooks::install_into(super::SPAWN_PROJECTILE as *const (), spawn as *const (), "MagicProjectile spawn", &REAL);
    }

    /// While the adapter casts: the projectile leaves the palm along the hand's aim.
    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn spawn(a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, x: u32, y: u32, z: u32, ax: u32, ay: u32, az: u32) -> u32 {
        let real: Spawn = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        let (x, y, z, ax, ay, az) = match super::launch() {
            Some(l) => {
                let f = f32::from_bits;
                tracing::info!("combat: the spell's projectile leaves the hand at {:?} (the game's {:?}), angles {:.2}, {:.2} (the game's {:.2}, {:.2})", l.pos, [f(x), f(y), f(z)], l.pitch, l.heading, f(ax), f(az));
                (l.pos[0].to_bits(), l.pos[1].to_bits(), l.pos[2].to_bits(), l.pitch.to_bits(), 0f32.to_bits(), l.heading.to_bits())
            }
            None => (x, y, z, ax, ay, az),
        };
        unsafe { real(a1, a2, a3, a4, a5, a6, x, y, z, ax, ay, az) }
    }
}

#[cfg(target_arch = "x86")]
pub use hook::install;
