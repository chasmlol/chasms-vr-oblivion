//! Oblivion's side of the framework's combat (`rustvr_protocol::combat`).
//!
//! * **Its creatures** ([`actors`]): every actor the game is fully processing near the player
//!   (its high-process list), as capsules on its own skeleton's bones (`Bip01 ...`): the head, the
//!   spine, the arms, the hands, the legs and the feet, each as thick as what it wears
//!   ([`crate::shapes`]).
//! * **The player's spells** and the hands' casts are `crate::spells`'s.
//! * **What the hands did** ([`apply`], on the game's thread before its frame):
//!   * A melee hit is the game's own hit: the player's hit routine (`Actor` vtable 0x3AC,
//!     0x5FEBF0) on that creature, the way the attack animation would land it. That brings the
//!     game's damage formula, skills, sneak attacks, enchantments, blocking, sounds, blood, the
//!     OnHit scripts and the stagger. A power hit is its power attack. A glancing blow lands
//!     nothing.
//!   * A block (the hand's weapon meeting a creature's swing) is the game's own block: its Block
//!     control held through the blow, so the game's block rules, skill and sounds decide it.
//!   * A cast or a spell picked from the wheel goes to `crate::spells`.

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
/// `TESObjectREFR`: its 3D root.
const NI_NODE: usize = 0x3C;
/// `Actor`: the vtable slot of `GetActorValue` and of the hit routine.
const VT_GET_AV: usize = 0x284;
const VT_HIT: usize = 0x3AC;
const GET_BASE_AV: usize = 0x005F_1910;
const AV_HEALTH: u32 = 8;
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

/// The game's magic effects by their codes (`NiTPointerMap<EffectSetting>`: buckets at +8, as many
/// as +4 says, a code's bucket the code modulo that; each entry {next, code, effect}).
const EFFECT_SETTINGS: usize = 0x00B3_3508;
/// A script effect's code.
pub const SCRIPT_EFFECT: u32 = u32::from_le_bytes(*b"SEFF");

/// Every one of the game's magic effects (`EffectSetting`s).
pub fn effect_settings() -> Vec<usize> {
    let buckets = mem::at::<u32>(EFFECT_SETTINGS + 4).unwrap_or(0) as usize;
    let table = mem::deref(EFFECT_SETTINGS + 8);
    let mut out = Vec::new();
    if table == 0 {
        return out;
    }
    for b in 0..buckets.min(4096) {
        let mut entry = mem::deref(table + b * 4);
        let mut guard = 0;
        while entry != 0 && guard < 256 {
            guard += 1;
            let e = mem::deref(entry + 8);
            if e != 0 {
                out.push(e);
            }
            entry = mem::deref(entry);
        }
    }
    out
}

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

/// The capsules of a humanoid skeleton: (from bone, to bone, radius, part, material). Each is
/// fitted to what the actor wears ([`crate::shapes`]); the radius here holds until it is, and where
/// nothing's drawn on it. (A part's index here is its `game_part`.)
pub(crate) const HUMANOID: &[(&str, &str, f32, u32, u32)] = &[
    // (The torso to the top of the chest: its round end stops short of the neck, which a blade can
    // cut through.)
    ("Bip01 Pelvis", "Bip01 Spine2", 0.17, PART_TORSO, MAT_LEATHER),
    ("Bip01 Neck", "Bip01 Head", 0.1, PART_HEAD, MAT_FLESH),
    ("Bip01 L UpperArm", "Bip01 L Forearm", 0.06, PART_ARM, MAT_FLESH),
    ("Bip01 L Forearm", "Bip01 L Hand", 0.05, PART_ARM, MAT_FLESH),
    ("Bip01 R UpperArm", "Bip01 R Forearm", 0.06, PART_ARM, MAT_FLESH),
    ("Bip01 R Forearm", "Bip01 R Hand", 0.05, PART_ARM, MAT_FLESH),
    ("Bip01 L Thigh", "Bip01 L Calf", 0.08, PART_LEG, MAT_FLESH),
    ("Bip01 L Calf", "Bip01 L Foot", 0.06, PART_LEG, MAT_FLESH),
    ("Bip01 R Thigh", "Bip01 R Calf", 0.08, PART_LEG, MAT_FLESH),
    ("Bip01 R Calf", "Bip01 R Foot", 0.06, PART_LEG, MAT_FLESH),
    ("Bip01 L Hand", "Bip01 L Finger2", 0.045, PART_ARM, MAT_FLESH),
    ("Bip01 R Hand", "Bip01 R Finger2", 0.045, PART_ARM, MAT_FLESH),
    ("Bip01 L Foot", "Bip01 L Toe0", 0.05, PART_LEG, MAT_FLESH),
    ("Bip01 R Foot", "Bip01 R Toe0", 0.05, PART_LEG, MAT_FLESH),
];

/// The bone a part of a humanoid starts at (its `game_part`: its index in [`HUMANOID`]).
pub(crate) fn part_bone(game_part: u32) -> Option<&'static str> {
    HUMANOID.get(game_part as usize).map(|e| e.0)
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
        for (part, &(from, to, r, kind, material)) in HUMANOID.iter().enumerate() {
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
    // The game's rule: no fighting from the saddle (nor casting). The hands still swing, as a
    // rider's would, but nothing lands. (A spell can still be picked.)
    let mounted = crate::riding::mounted();
    for ev in events {
        if mounted && ev.kind != CE_SELECT {
            tracing::info!("combat: a blow or cast from the saddle: the game doesn't fight mounted");
            continue;
        }
        match ev.kind {
            CE_HIT => {
                // (Gone since it was published: the game's frame ran in between.)
                let Some(&target) = actors.get(&ev.actor).filter(|&&a| is_actor(a)) else { continue };
                // (The game's damage is its own: any clean hit is a hit, a fast one its power
                // attack; a blow at a bad angle glances off.)
                if ev.flags & CEF_GLANCING != 0 {
                    tracing::info!("combat: a glancing blow on {:#x}: nothing lands", ev.actor);
                    continue;
                }
                let power = ev.flags & CEF_POWER != 0;
                type Hit = unsafe extern "thiscall" fn(usize, u8, usize, usize);
                let f = mem::deref(mem::deref(player) + VT_HIT);
                tracing::info!("combat: the player hits {:#x} (strength {:.2}{})", ev.actor, ev.strength, if power { ", a power attack" } else { "" });
                let before = crate::gore::health(target);
                if f != 0 {
                    unsafe { std::mem::transmute::<usize, Hit>(f)(player, power as u8, 0, target) };
                }
                // (A blow that kills, or one into a corpse, may take a limb: the gore's.)
                crate::gore::after_hit(target, ev, before);
            }
            CE_CAST if ev.object == crate::weapons::WEAPON_ID => crate::spells::cast_staff(player, ev),
            CE_CAST => {
                let Some(&spell) = spells.get(&ev.what) else { continue };
                crate::spells::cast(player, spell, ev, actors);
            }
            CE_SELECT => {
                if let Some(&spell) = spells.get(&ev.what) {
                    crate::spells::select(spell);
                }
            }
            CE_CAST_REFUSED => {
                if let Some(&spell) = spells.get(&ev.what) {
                    crate::spells::failed(spell);
                }
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
    /// The spells as last sent (sent again when they change): each one's id, flags and cost, the
    /// magicka, the picks.
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
    if k.frame % 60 == 2 {
        log_actors(&table);
    }
    let spells_now = (k.frame % 10 == 1).then(crate::spells::table);
    crate::link::with_adapter(|a| {
        crate::weapons::publish(a);
        crate::torch::publish(a);
        crate::map::publish(a);
        // The face shaped by hand in the race menu, into the character (before it's published).
        crate::facesculpt::update(a);
        // (Every frame while the race menu changes the body and the face as the player watches.)
        if k.frame % 30 == 3 || crate::menus::menu_open(crate::menus::MENU_RACE_SEX) {
            crate::body::publish(a);
        }
        a.write_actors(&table);
        crate::grabs::publish(a, &table, &k.actors);
        if let Some((s, by_id)) = spells_now {
            let mut key: Vec<(u32, u32, u32)> = s.spells[..s.count as usize].iter().map(|x| (x.id, x.flags, x.cost as u32)).collect();
            key.push((s.magicka as u32, s.magicka_max as u32, s.picks));
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
    apply(&events, &k.actors, &k.spells);
}
