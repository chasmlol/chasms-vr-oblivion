//! Oblivion's side of the framework's combat (`rustvr_protocol::combat`).
//!
//! * **Its creatures** ([`actors`]): every actor the game is fully processing near the player
//!   (its high-process list), as capsules on its own skeleton's bones (`Bip01 ...`): the head, the
//!   spine, the arms and the legs.
//! * **The player's spells** ([`spells`]): the spells, powers and lesser powers on the player's
//!   own list, with how each is delivered (its effects' range), what it costs the player now
//!   (the game's own cost), the colour of its first effect's light, which one is selected, and
//!   the player's magicka.
//!   Its weapon (always) and its shield go as its guard ([`PART_SHIELD`] parts: what blades stop
//!   on), with its weapon's Havok mass, whether its attack is a power attack (its weapon sequence's
//!   group, 22..26) and how many of its blows have landed on the player (`Actor::Hit` finding the
//!   player in its hit cone, 0x6156C0, as it lands a blow).
//! * **What the hands did** ([`apply`], on the game's thread before its frame;
//!   `docs/research/melee.md`):
//!   * A melee hit is the game's own hit: the player's hit routine (`Actor` vtable 0x3AC,
//!     0x5FEBF0) on that creature, the way the attack animation would land it. That brings the
//!     game's damage formula, skills, sneak attacks, enchantments, blocking, sounds, blood, the
//!     OnHit scripts and the stagger. The swing scales it: while it runs the weapon's damage
//!     (0x547070) is that share of the game's, from the game's own weakest hit (an exhausted
//!     one's, `fFatigueBase - fFatigueMult`) at the weakest swing to the game's own at a full one.
//!     A power hit is its power attack: the flag, and the player's weapon sequence set to the
//!     power attack the swing's direction asks for while the hit runs (the side ones once the
//!     weapon's skill is Journeyman, the back one Expert, the forward one Master, standing while
//!     sneaking: what the game itself lets the player do), so its bonus, knockback, knockdown,
//!     disarm and paralysis are the game's. A glancing blow is a weak hit; a blow on a blocking
//!     creature's guard is the game's hit, which the game's own block takes.
//!   * A swing as hard as a full hit costs the game's own attack fatigue (0x5E4010, five times
//!     for a power attack).
//!   * A block (the hand's weapon meeting a creature's swing) is the game's own block: its Block
//!     control held through the blow, so the game's block rules, skill and sounds decide it. A
//!     parry knocks the blow aside at once: the attacker's own recoil (0x5F4F00), and the block's
//!     Master disarm (0x5FC2B0).
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
fn is_actor(actor: usize) -> bool {
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

/// The process's equipped shield (`MiddleHighProcess` +0xF0: an inventory entry, its form at +8;
/// an armour's male model path at +0x70).
const EQUIPPED_SHIELD: usize = 0xF0;
const ARMOR_MODEL: usize = 0x70;
/// An actor's animation data (vtable +0x164, thiscall) and the group its weapon sequence plays (the
/// low byte of the word at +0x42): 22 a standing power attack, 23 forward, 24 back, 25 left, 26 right.
const VT_ANIM_DATA: usize = 0x164;
const WEAPON_GROUP: usize = 0x42;
const GROUP_POWER: u8 = 22;
const GROUP_POWER_FORWARD: u8 = 23;
const GROUP_POWER_BACK: u8 = 24;
const GROUP_POWER_LEFT: u8 = 25;
const GROUP_POWER_RIGHT: u8 = 26;
/// The game's weapon damage (cdecl: skill, luck, strength, fatigue, damage, condition, multiplier,
/// no fatigue), its target in a blow's hit cone (cdecl: attacker, reach), `Actor::Hit` itself, an
/// attack's fatigue (thiscall: actor, power), a recoil (thiscall: actor), the block's Master disarm
/// (thiscall: blocker, attacker).
const WEAPON_DAMAGE: usize = 0x0054_7070;
const HIT_TARGET: usize = 0x0061_56C0;
const HIT: usize = 0x005F_EBF0;
const ATTACK_FATIGUE: usize = 0x005E_4010;
const RECOIL: usize = 0x005F_4F00;
const BLOCK_DISARM: usize = 0x005F_C2B0;
/// A blocked blow's sound (cdecl: the attacker's weapon type, the blocker's (-1: a hand), the
/// attacker's place, the blocker's place, enchanted): the game's `WPNBlockBlade`, `WPNBlockBlunt`...
const BLOCK_SOUND: usize = 0x006A_FFF0;
/// A blow's sounds (cdecl: attacker, damage, block, target, ...): with no target, the swish of the
/// attacker's weapon (`WPNSwishSmall`, `Medium`, `Large` by its weight; `WPNSwishHand` without one),
/// as the game sounds a blow that finds nothing.
const BLOW_SOUND: usize = 0x006A_F880;
/// The skill each weapon type uses (actor values, by `TESObjectWEAP::type`).
const WEAPON_SKILLS: usize = 0x00B0_86A0;
/// Game settings: `fFatigueBase`, `fFatigueMult` (the fatigue term of a blow's damage), and the
/// skill levels (`iSkillJourneymanMin`, `iSkillExpertMin`, `iSkillMasterMin`).
const F_FATIGUE_BASE: usize = 0x00B3_7530;
const F_FATIGUE_MULT: usize = 0x00B3_7538;
const I_SKILL_JOURNEYMAN: usize = 0x00B3_A500;
const I_SKILL_EXPERT: usize = 0x00B3_A508;
const I_SKILL_MASTER: usize = 0x00B3_A510;
/// A creature's guard: its weapon's thickness; a shield's disc as two capsules side by side, each
/// this share of its radius thick.
const GUARD_RADIUS: f32 = 0.03;
const SHIELD_HALF: f32 = 0.5;

/// The share of the game's damage the adapter's own hit asks for while it runs (f32 bits; 0: no hit
/// of ours is running), and the creatures whose blows have landed on the player (form id: how many).
static OUR_SHARE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static LANDED: Mutex<Option<HashMap<u32, u32>>> = Mutex::new(None);
/// The creature whose `Actor::Hit` is running now (0: none).
static HITTING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

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

/// The named nodes under `root` that are bones (named `Bip01...`), and a few named attachments: the
/// `Weapon` node, and the equipped shield's own (as `Shield`).
fn bones(root: usize) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    let mut stack = vec![(root, 0u32)];
    let mut seen = 0;
    while let Some((node, depth)) = stack.pop() {
        seen += 1;
        if seen > 600 || depth > 48 {
            break;
        }
        let name = read_str(mem::deref(node + NODE_NAME));
        let descend = depth <= 2 || name.starts_with("Bip01");
        if name.starts_with("Bip01") || name == "Weapon" {
            out.insert(name, node);
        } else if name.starts_with("Shield") {
            // An equipped shield's model, on the left forearm's twist: "Shield (<its form>)".
            out.insert("Shield".into(), node);
        }
        if !descend {
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
fn node_pos(node: usize) -> Option<Vec3> {
    let p: [f32; 3] = mem::at(node + engine::av::WORLD_TRANSLATE)?;
    Some(camera::basis().point_from_game(Vec3::from(p)))
}

/// Calls `Actor::GetActorValue`.
#[cfg(target_arch = "x86")]
fn actor_value(actor: usize, av: u32) -> i32 {
    type F = unsafe extern "thiscall" fn(usize, u32) -> i32;
    let vt = mem::deref(actor);
    let f = mem::deref(vt + VT_GET_AV);
    if f == 0 { 0 } else { unsafe { std::mem::transmute::<usize, F>(f)(actor, av) } }
}

#[cfg(target_arch = "x86")]
fn base_actor_value(actor: usize, av: u32) -> i32 {
    type F = unsafe extern "thiscall" fn(usize, u32) -> u32;
    unsafe { std::mem::transmute::<usize, F>(GET_BASE_AV)(actor, av) as i32 }
}

#[cfg(not(target_arch = "x86"))]
fn actor_value(_actor: usize, _av: u32) -> i32 {
    0
}

#[cfg(not(target_arch = "x86"))]
fn base_actor_value(_actor: usize, _av: u32) -> i32 {
    0
}

/// One of the player's actor values (0 without a player).
pub fn player_value(av: u32) -> i32 {
    let p = engine::player();
    if p == 0 { 0 } else { actor_value(p, av) }
}

/// An actor's animation data, if any (vtable +0x164).
#[cfg(target_arch = "x86")]
fn anim_data(actor: usize) -> usize {
    let f = mem::deref(mem::deref(actor) + VT_ANIM_DATA);
    if actor == 0 || !(0x0040_1000..0x00A3_0000).contains(&f) {
        return 0;
    }
    type F = unsafe extern "thiscall" fn(usize) -> usize;
    unsafe { std::mem::transmute::<usize, F>(f)(actor) }
}

#[cfg(not(target_arch = "x86"))]
fn anim_data(_actor: usize) -> usize {
    0
}

/// The group an actor's weapon sequence plays (its attack), if it has one.
fn weapon_group(actor: usize) -> Option<u8> {
    let data = anim_data(actor);
    (data != 0).then(|| mem::at::<u8>(data + WEAPON_GROUP)).flatten()
}

/// An actor's equipped weapon's form, if it has one.
fn equipped_weapon(actor: usize) -> usize {
    let form = mem::deref(mem::deref(mem::deref(actor + 0x58) + EQUIPPED_WEAPON) + 8);
    if form != 0 && mem::at::<u8>(form + 4) == Some(0x21) { form } else { 0 }
}

/// What an actor's equipped weapon weighs (its Havok mass, kg).
fn weapon_mass(actor: usize) -> f32 {
    let form = equipped_weapon(actor);
    if form == 0 {
        return 0.0;
    }
    crate::havok::body(&read_str(mem::deref(form + 0x34))).map_or(0.0, |b| b.mass)
}

/// An actor's equipped shield's disc (its model's, from the asset cache): its centre and its radius
/// in the model's space (metres), cached by model path.
fn shield_disc(actor: usize) -> Option<(Vec3, f32)> {
    static DISCS: Mutex<Option<HashMap<String, Option<(Vec3, f32)>>>> = Mutex::new(None);
    let form = mem::deref(mem::deref(mem::deref(actor + 0x58) + EQUIPPED_SHIELD) + 8);
    if form == 0 || mem::at::<u8>(form + 4) != Some(0x14) {
        return None;
    }
    let path = read_str(mem::deref(form + ARMOR_MODEL));
    let mut cache = DISCS.lock().unwrap();
    *cache.get_or_insert_with(HashMap::new).entry(path.clone()).or_insert_with(|| {
        let key = oblivion_assets::export::model_key(&path);
        let m = rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key).ok()?;
        let (lo, hi) = (Vec3::from_array(m.bounds_min), Vec3::from_array(m.bounds_max));
        Some(((lo + hi) * 0.5, ((hi.x - lo.x).min(hi.y - lo.y) * 0.5).max(0.1)))
    })
}

/// A creature's guard as capsules (protocol space): its weapon along its `Weapon` node, and its
/// shield's disc on its `Shield` node (the disc across the model's X and Y, the game's X and Z).
fn guard_parts(actor: usize, b: &HashMap<String, usize>) -> Vec<(Vec3, Vec3, f32)> {
    let mut out = Vec::new();
    let axes = |node: usize| -> Option<[Vec3; 3]> {
        let rot: [f32; 9] = mem::at(node + engine::av::WORLD_ROTATE)?;
        Some([0, 1, 2].map(|k| camera::basis().dir_from_game(Vec3::new(rot[k], rot[3 + k], rot[6 + k])).normalize_or_zero()))
    };
    if let (Some(&node), Some(len)) = (b.get("Weapon"), weapon_length(actor))
        && let (Some(at), Some(ax)) = (node_pos(node), axes(node))
    {
        out.push((at, at + ax[1] * len, GUARD_RADIUS));
    }
    if let (Some(&node), Some((centre, r))) = (b.get("Shield"), shield_disc(actor))
        && let (Some(at), Some(ax)) = (node_pos(node), axes(node))
    {
        // The model's (x, y, z) is the game's (x, z, y).
        let c = at + (ax[0] * centre.x + ax[2] * centre.y + ax[1] * centre.z);
        let up = ax[2] * r * (1.0 - SHIELD_HALF);
        for side in [-1.0, 1.0] {
            let off = ax[0] * (side * r * SHIELD_HALF);
            out.push((c + off - up, c + off + up, r * SHIELD_HALF));
        }
    }
    out
}

/// Once a creature: what its guard was made of (its `Weapon` and `Shield` nodes, its equipped
/// weapon's and shield's forms).
fn log_guard_once(id: u32, actor: usize, b: &HashMap<String, usize>, parts: usize) {
    static SEEN: Mutex<Option<std::collections::HashSet<u32>>> = Mutex::new(None);
    if !SEEN.lock().unwrap().get_or_insert_with(Default::default).insert(id) {
        return;
    }
    let process = mem::deref(actor + 0x58);
    let form = |off: usize| mem::deref(mem::deref(process + off) + 8);
    tracing::info!(
        "combat: {id:#x}'s guard: {parts} parts (its Weapon node {}, its Shield node {}, its weapon form {:#x}, its shield form {:#x}, the weapon's length {:?}, the shield's disc {:?})",
        b.contains_key("Weapon"),
        b.contains_key("Shield"),
        form(EQUIPPED_WEAPON),
        form(EQUIPPED_SHIELD),
        weapon_length(actor),
        shield_disc(actor)
    );
}

/// The capsules of a humanoid skeleton: (from bone, to bone, radius, part, material).
const HUMANOID: &[(&str, &str, f32, u32, u32)] = &[
    ("Bip01 Pelvis", "Bip01 Neck", 0.17, PART_TORSO, MAT_LEATHER),
    ("Bip01 Neck", "Bip01 Head", 0.1, PART_HEAD, MAT_FLESH),
    ("Bip01 L UpperArm", "Bip01 L Forearm", 0.06, PART_ARM, MAT_FLESH),
    ("Bip01 L Forearm", "Bip01 L Hand", 0.05, PART_ARM, MAT_FLESH),
    ("Bip01 R UpperArm", "Bip01 R Forearm", 0.06, PART_ARM, MAT_FLESH),
    ("Bip01 R Forearm", "Bip01 R Hand", 0.05, PART_ARM, MAT_FLESH),
    ("Bip01 L Thigh", "Bip01 L Calf", 0.08, PART_LEG, MAT_FLESH),
    ("Bip01 L Calf", "Bip01 L Foot", 0.06, PART_LEG, MAT_FLESH),
    ("Bip01 R Thigh", "Bip01 R Calf", 0.08, PART_LEG, MAT_FLESH),
    ("Bip01 R Calf", "Bip01 R Foot", 0.06, PART_LEG, MAT_FLESH),
];

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
        if actor == player || !is_actor(actor) {
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
        for &(from, to, r, kind, material) in HUMANOID {
            let (Some(p), Some(q)) = (b.get(from).and_then(|&n| node_pos(n)), b.get(to).and_then(|&n| node_pos(n))) else { continue };
            // The head: from the neck up through the crown.
            let q = if kind == PART_HEAD { q + (q - p).normalize_or(Vec3::Y) * 0.12 } else { q };
            let i = a.count as usize;
            a.parts[i] = ActorPart { a: p.to_array(), radius: r, b: q.to_array(), kind, material, game_part: i as u32, _pad: [0; 2] };
            a.count += 1;
        }
        if a.count == 0 {
            // Not a humanoid (a creature): one capsule from its feet to its height.
            let Some(feet) = mem::at::<[f32; 3]>(actor + refr::POS) else { continue };
            let f = camera::basis().point_from_game(Vec3::from(feet));
            a.parts[0] = ActorPart { a: (f + Vec3::Y * 0.3).to_array(), radius: 0.35, b: (f + Vec3::Y * 1.3).to_array(), kind: PART_TORSO, material: MAT_FLESH, game_part: 0, _pad: [0; 2] };
            a.count = 1;
        }
        // Its guard: what blades stop on (and, while it holds it up, a hit on it is a blocked one).
        let guard = guard_parts(actor, &b);
        log_guard_once(id, actor, &b, guard.len());
        for (p, q, r) in guard {
            let i = a.count as usize;
            if i >= MAX_ACTOR_PARTS {
                break;
            }
            a.parts[i] = ActorPart { a: p.to_array(), radius: r, b: q.to_array(), kind: PART_SHIELD, material: MAT_PLATE, game_part: i as u32, _pad: [0; 2] };
            a.count += 1;
        }
        match current_action(actor) {
            ACTION_ATTACK | ACTION_ATTACK_FOLLOW => {
                if let Some((wa, wb, r)) = attack_segment(actor, &b) {
                    a.flags |= ACT_ATTACKING;
                    a.weapon_a = wa.to_array();
                    a.weapon_b = wb.to_array();
                    a.weapon_radius = r;
                }
                if weapon_group(actor).is_some_and(|g| (GROUP_POWER..=GROUP_POWER_RIGHT).contains(&g)) {
                    a.flags |= ACT_POWER_ATTACK;
                }
            }
            ACTION_BLOCK => a.flags |= ACT_BLOCKING,
            _ => {}
        }
        a.weapon_mass = weapon_mass(actor);
        a.blows = LANDED.lock().unwrap().as_ref().and_then(|l| l.get(&id)).copied().unwrap_or(0);
        let health = actor_value(actor, AV_HEALTH);
        let most = base_actor_value(actor, AV_HEALTH).max(1);
        a.flags |= if health > 0 { ACT_ALIVE } else { 0 };
        a.health = (health as f32 / most as f32).clamp(0.0, 1.0);
        let i = table.count as usize;
        table.actors[i] = a;
        table.count += 1;
        by_id.insert(id, actor);
    }
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
                let light = mem::deref(mem::deref(effect + EFFECT_SETTING) + SETTING_LIGHT);
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
pub fn apply(events: &[CombatEvent], actors: &HashMap<u32, usize>, spells: &HashMap<u32, usize>) {
    let player = engine::player();
    if player == 0 {
        return;
    }
    for ev in events {
        match ev.kind {
            CE_HIT => {
                // (Gone since it was published: the game's frame ran in between.)
                let Some(&target) = actors.get(&ev.actor).filter(|&&a| is_actor(a)) else { continue };
                // The game's own hit, as strong as the swing: a weak one (a tap, a glancing blow
                // with the flat) its weakest, a full one its own, a power one its power attack
                // (the one the swing's direction is, as far as the skill has learned).
                let power = ev.flags & CEF_POWER != 0;
                let share = damage_share(ev.strength);
                let group = power.then(|| power_group(player, ev.swing));
                type Hit = unsafe extern "thiscall" fn(usize, u32, usize, usize);
                let f = mem::deref(mem::deref(player) + VT_HIT);
                tracing::info!(
                    "combat: the player hits {:#x} (strength {:.2}: {:.0}% of the game's damage{}{}{}, the swing {})",
                    ev.actor,
                    ev.strength,
                    share * 100.0,
                    group.map(|g| format!(", a power attack (group {g})")).unwrap_or_default(),
                    if ev.flags & CEF_GLANCING != 0 { ", glancing" } else { "" },
                    if ev.flags & CEF_GUARD != 0 { ", on its guard" } else { "" },
                    ev.swing
                );
                if f != 0 {
                    as_our_hit(player, share, group, || unsafe { std::mem::transmute::<usize, Hit>(f)(player, power as u32, 0, target) });
                }
            }
            CE_SWING => {
                // The game's own attack: its fatigue (a power attack's five times).
                let power = ev.flags & CEF_POWER != 0;
                let before = actor_value(player, AV_FATIGUE);
                type Fatigue = unsafe extern "thiscall" fn(usize, u32);
                unsafe { std::mem::transmute::<usize, Fatigue>(ATTACK_FATIGUE)(player, power as u32) };
                // Missed: the game's own swish of the weapon through the air.
                if ev.actor == 0 {
                    type Sound = unsafe extern "C" fn(usize, f32, u32, usize, i32, i32, i32, u32, u32);
                    unsafe { std::mem::transmute::<usize, Sound>(BLOW_SOUND)(player, 0.0, 0, 0, -1, -1, -1, 0, 0) };
                }
                tracing::info!(
                    "combat: a swing ({:.2} of a full hit{}, the swing {}, {}): fatigue {before} -> {}",
                    ev.strength,
                    if power { ", a power attack" } else { "" },
                    ev.swing,
                    if ev.actor != 0 { "it hit" } else { "it missed" },
                    actor_value(player, AV_FATIGUE)
                );
            }
            CE_CAST => {
                let Some(&spell) = spells.get(&ev.what) else { continue };
                cast(player, spell, ev, actors);
            }
            CE_BLOCK => {
                let parry = ev.flags & CEF_PARRY != 0;
                tracing::info!("combat: the player blocks {:#x}{} (the guard {:.2} of the blow)", ev.actor, if parry { ", a parry" } else { "" }, ev.strength);
                crate::player::hold_control("Block", BLOCK_HOLD);
                // A parry knocks the blow aside before it lands: the attacker's own recoil, and the
                // block's Master disarm.
                if parry && let Some(&attacker) = actors.get(&ev.actor).filter(|&&a| is_actor(a)) {
                    type Recoil = unsafe extern "thiscall" fn(usize) -> u32;
                    type Disarm = unsafe extern "thiscall" fn(usize, usize) -> u8;
                    unsafe { std::mem::transmute::<usize, Recoil>(RECOIL)(attacker) };
                    let disarmed = unsafe { std::mem::transmute::<usize, Disarm>(BLOCK_DISARM)(player, attacker) } != 0;
                    // The clash: the game's own sound of the two weapons meeting (a shield's is its
                    // own blocked hit's, which a parried blow never comes to: the runtime's knock).
                    let kind = |actor: usize| {
                        let form = equipped_weapon(actor);
                        if form == 0 { -1 } else { mem::at::<u32>(form + 0x90).map_or(-1, |t| t as i32) }
                    };
                    if ev.object != crate::weapons::SHIELD_ID
                        && let (Some(a), Some(b)) = (mem::at::<[f32; 3]>(attacker + refr::POS), mem::at::<[f32; 3]>(player + refr::POS))
                    {
                        type Sound = unsafe extern "C" fn(i32, i32, f32, f32, f32, f32, f32, f32, u32);
                        unsafe { std::mem::transmute::<usize, Sound>(BLOCK_SOUND)(kind(attacker), kind(player), a[0], a[1], a[2], b[0], b[1], b[2], 0) };
                    }
                    tracing::info!("combat: {:#x} recoils from the parry{}", ev.actor, if disarmed { " and drops its weapon" } else { "" });
                }
            }
            _ => {}
        }
    }
}

/// The share of the game's damage a hit of `strength` (1: a full hit) does: from the game's own
/// weakest hit (an exhausted fighter's, `fFatigueBase - fFatigueMult` of a rested one's) at the
/// weakest to its own at a full one.
fn damage_share(strength: f32) -> f32 {
    let (base, mult) = (mem::at::<f32>(F_FATIGUE_BASE).unwrap_or(1.0), mem::at::<f32>(F_FATIGUE_MULT).unwrap_or(0.5));
    let weakest = if base > 0.05 && base.is_finite() && mult.is_finite() { ((base - mult) / base).clamp(0.05, 1.0) } else { 0.5 };
    weakest + (1.0 - weakest) * strength.clamp(0.0, 1.0)
}

/// The power attack (its animation group) a swing going `swing` is for the player, as the game lets
/// the player do them: the side ones from Journeyman in the weapon's skill, the back one (a rising
/// swing) from Expert, the forward one (a thrust) at Master; else, and always while sneaking, the
/// standing one (a chop).
fn power_group(player: usize, swing: u32) -> u8 {
    let (group, level) = match swing {
        SWING_LEFT => (GROUP_POWER_LEFT, I_SKILL_JOURNEYMAN),
        SWING_RIGHT => (GROUP_POWER_RIGHT, I_SKILL_JOURNEYMAN),
        SWING_UP => (GROUP_POWER_BACK, I_SKILL_EXPERT),
        SWING_THRUST => (GROUP_POWER_FORWARD, I_SKILL_MASTER),
        _ => return GROUP_POWER,
    };
    let form = equipped_weapon(player);
    let skill = if form != 0 { mem::at::<u32>(WEAPON_SKILLS + 4 * mem::at::<u32>(form + 0x90).unwrap_or(0).min(5) as usize) } else { None };
    let (Some(skill), Some(needed)) = (skill, mem::at::<i32>(level)) else { return GROUP_POWER };
    if crate::player::sneaking() || actor_value(player, skill) < needed { GROUP_POWER } else { group }
}

/// Runs the player's hit `f` as the adapter's: the weapon's damage that share of the game's, and the
/// player's weapon sequence that power attack's group, while it runs.
fn as_our_hit(player: usize, share: f32, group: Option<u8>, f: impl FnOnce()) {
    use std::sync::atomic::Ordering;
    let data = anim_data(player);
    let old = (data != 0).then(|| mem::at::<u8>(data + WEAPON_GROUP)).flatten();
    if let (Some(_), Some(g)) = (old, group) {
        unsafe { ((data + WEAPON_GROUP) as *mut u8).write(g) };
    }
    OUR_SHARE.store(share.to_bits(), Ordering::Relaxed);
    f();
    OUR_SHARE.store(0, Ordering::Relaxed);
    if let (Some(o), Some(_)) = (old, group) {
        unsafe { ((data + WEAPON_GROUP) as *mut u8).write(o) };
    }
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
    let dir = Vec3::from_array(ev.dir);
    let (pitch, heading) = angles(dir);
    let pos = camera::basis().point_to_game(Vec3::from_array(ev.point)).to_array();
    *LAUNCH.lock().unwrap() = Some(Launch { pos, pitch, heading });
    // The player faces the aim for the cast (the game aims some things by where the caster faces).
    let rot = (player + refr::ROT) as *mut f32;
    let (old_pitch, old_heading) = unsafe { (rot.read(), rot.add(2).read()) };
    unsafe {
        rot.write(pitch);
        rot.add(2).write(heading.rem_euclid(std::f32::consts::TAU));
    }
    let target = if ev.actor != 0 { actors.get(&ev.actor).copied().filter(|&a| is_actor(a)).unwrap_or(0) } else { 0 };
    tracing::info!("combat: the player casts {:#x} from {pos:?}, pitch {pitch:.2}, heading {heading:.2}{}", ev.what, if target != 0 { " on its target" } else { "" });
    unsafe { std::mem::transmute::<usize, Cast>(CAST_IMMEDIATE)(caster, item, target, 0) };
    unsafe {
        rot.write(old_pitch);
        rot.add(2).write(old_heading);
    }
    *LAUNCH.lock().unwrap() = None;
}

#[cfg(not(target_arch = "x86"))]
pub fn apply(_events: &[CombatEvent], _actors: &HashMap<u32, usize>, _spells: &HashMap<u32, usize>) {}

#[cfg(test)]
mod tests {
    use super::*;

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
    let k = known.get_or_insert_with(|| Known { actors: HashMap::new(), spells: HashMap::new(), sent: None, frame: 0 });
    k.actors = actors;
    k.frame += 1;
    // (The game frame they're from: the runtime times their motion by it. Their blows are taken at
    // the share of the game's speed the player's own swings are.)
    let mut table = table;
    table.frame = k.frame;
    table.blow_scale = crate::weapons::to_hand();
    if k.frame % 60 == 2 {
        log_actors(&table);
    }
    let spells_now = (k.frame % 10 == 1).then(spells);
    crate::link::with_adapter(|a| {
        crate::weapons::publish(a);
        if k.frame % 30 == 3 {
            crate::body::publish(a);
        }
        a.write_actors(&table);
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

/// A creature's blow just landed on the player: counted (the runtime feels it in the hands).
fn landed(attacker: usize) {
    let Some(id) = mem::at::<u32>(attacker + FORM_ID) else { return };
    let mut l = LANDED.lock().unwrap();
    let n = l.get_or_insert_with(HashMap::new).entry(id).or_insert(0);
    *n = n.wrapping_add(1);
    tracing::info!("combat: {id:#x}'s blow lands on the player{}", if weapon_group(attacker).is_some_and(|g| (GROUP_POWER..=GROUP_POWER_RIGHT).contains(&g)) { " (a power attack)" } else { "" });
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

/// Before the game's frame: what the hands did since the last one, done by the game.
pub fn apply_pending() {
    let Some(events) = crate::link::with_adapter(|a| a.combat_events()) else { return };
    if events.is_empty() {
        return;
    }
    let known = KNOWN.lock().unwrap();
    let Some(k) = known.as_ref() else { return };
    apply(&events, &k.actors, &k.spells);
}

#[cfg(target_arch = "x86")]
mod hook {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static REAL: AtomicUsize = AtomicUsize::new(0);
    type Spawn = unsafe extern "C" fn(u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) -> u32;

    static REAL_DAMAGE: AtomicUsize = AtomicUsize::new(0);
    static REAL_HIT: AtomicUsize = AtomicUsize::new(0);
    static REAL_TARGET: AtomicUsize = AtomicUsize::new(0);
    type Damage = unsafe extern "C" fn(i32, i32, i32, f32, i32, f32, f32, u32) -> f32;
    type Hit = unsafe extern "thiscall" fn(usize, u32, usize, usize);
    type Target = unsafe extern "C" fn(usize, f32) -> usize;

    pub fn install() {
        crate::hooks::install_into(super::SPAWN_PROJECTILE as *const (), spawn as *const (), "MagicProjectile spawn", &REAL);
        crate::hooks::install_into(super::WEAPON_DAMAGE as *const (), damage as *const (), "weapon damage", &REAL_DAMAGE);
        crate::hooks::install_into(super::HIT as *const (), hit as *const (), "Actor::Hit", &REAL_HIT);
        crate::hooks::install_into(super::HIT_TARGET as *const (), target as *const (), "the hit cone's target", &REAL_TARGET);
    }

    /// The weapon's damage: while the adapter's own hit runs, the share of it the swing asks for
    /// (on its multiplier, the power attack's bonus, so the game's formula is the rest).
    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn damage(skill: i32, luck: i32, strength: i32, fatigue: f32, base: i32, condition: f32, mult: f32, no_fatigue: u32) -> f32 {
        let real: Damage = unsafe { std::mem::transmute(REAL_DAMAGE.load(Ordering::Acquire)) };
        // (Only the hit's own weapon damage, the first it works out: what the creature hit does in
        // answer, weighing its own weapon, is its own.)
        let share = f32::from_bits(super::OUR_SHARE.swap(0, Ordering::Relaxed));
        let mult = if share > 0.0 { mult * share } else { mult };
        let out = unsafe { real(skill, luck, strength, fatigue, base, condition, mult, no_fatigue) };
        if share > 0.0 {
            tracing::info!("combat: the weapon's damage {base} (skill {skill}, strength {strength}, fatigue {fatigue:.2}, condition {condition:.2}) times {mult:.2}: {out:.1}");
        }
        out
    }

    /// Every actor's blow: one by a creature that finds the player in its hit cone (or is aimed at
    /// the player) has landed on the player.
    unsafe extern "thiscall" fn hit(this: usize, power: u32, projectile: usize, target: usize) {
        let real: Hit = unsafe { std::mem::transmute(REAL_HIT.load(Ordering::Acquire)) };
        let player = crate::engine::player();
        if this == player || this == 0 {
            return unsafe { real(this, power, projectile, target) };
        }
        if target != 0 && target == player && projectile == 0 {
            super::landed(this);
        }
        let outer = super::HITTING.swap(if target == 0 && projectile == 0 { this } else { 0 }, Ordering::Relaxed);
        let before = (super::actor_value(player, super::AV_HEALTH), super::actor_value(player, super::AV_FATIGUE), super::current_action(player));
        unsafe { real(this, power, projectile, target) };
        super::HITTING.store(outer, Ordering::Relaxed);
        let after = (super::actor_value(player, super::AV_HEALTH), super::actor_value(player, super::AV_FATIGUE));
        if after != (before.0, before.1) {
            tracing::info!("combat: a blow on the player{}: health {} -> {}, fatigue {} -> {} (the player's action {})", if power != 0 { " (a power attack)" } else { "" }, before.0, after.0, before.1, after.1, before.2);
        }
    }

    /// The target in a blow's hit cone: the player, inside a creature's blow, is a blow landed.
    unsafe extern "C" fn target(attacker: usize, reach: f32) -> usize {
        let real: Target = unsafe { std::mem::transmute(REAL_TARGET.load(Ordering::Acquire)) };
        let t = unsafe { real(attacker, reach) };
        if t != 0 && t == crate::engine::player() && attacker == super::HITTING.load(Ordering::Relaxed) {
            super::landed(attacker);
        }
        t
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
