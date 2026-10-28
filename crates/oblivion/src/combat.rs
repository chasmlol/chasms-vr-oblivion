//! Oblivion's side of the framework's combat (`rustvr_protocol::combat`).
//!
//! * **Its creatures** ([`actors`]): every actor the game is fully processing near the player
//!   (its high-process list), as capsules on its own skeleton's bones (`Bip01 ...`): the head, the
//!   spine, the arms and the legs.
//! * **The player's spells** ([`spells`]): the spells, powers and lesser powers on the player's
//!   own list, with how each is delivered (its effects' range), what it costs the player now
//!   (the game's own cost), the colour of its first effect's light, which one is selected, and
//!   the player's magicka.
//! * **What the hands did** ([`apply`], on the game's thread before its frame):
//!   * A melee hit is the game's own hit: the player's hit routine (`Actor` vtable 0x3AC,
//!     0x5FEBF0) on that creature, the way the attack animation would land it. That brings the
//!     game's damage formula, skills, sneak attacks, enchantments, blocking, sounds, blood, the
//!     OnHit scripts and the stagger. A power hit is its power attack. A glancing blow lands
//!     nothing.
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

/// `ActorProcessManager` (a global struct) and its list of high-process actors (`tList`, inline
/// first node).
const PROCESS_MANAGER: usize = 0x00B3_BD00;
const HIGH_ACTORS: usize = 0x5C;
/// `TESForm::refID`.
const FORM_ID: usize = 0x0C;
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

/// Spell types the hand casts: a spell, a power, a lesser power.
const CASTABLE_TYPES: [u32; 3] = [0, 2, 3];

fn read_str(p: usize) -> String {
    let mut out = Vec::new();
    for i in 0..128 {
        match mem::at::<u8>(p + i) {
            Some(0) | None => break,
            Some(b) => out.push(b),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The named nodes under `root` that are bones (named `Bip01...`), and a few named attachments.
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
        if name.starts_with("Bip01") || name == "Weapon" || name == "Shield" {
            out.insert(name, node);
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
    let mut node = PROCESS_MANAGER + HIGH_ACTORS;
    let mut guard = 0;
    while node != 0 && guard < 256 && (table.count as usize) < MAX_ACTORS {
        guard += 1;
        let actor = mem::deref(node);
        node = mem::deref(node + 4);
        if actor == 0 || actor == player {
            continue;
        }
        let Some(id) = mem::at::<u32>(actor + FORM_ID) else { continue };
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
        let health = actor_value(actor, AV_HEALTH);
        let most = base_actor_value(actor, AV_HEALTH).max(1);
        a.flags = if health > 0 { ACT_ALIVE } else { 0 };
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
                let Some(&target) = actors.get(&ev.actor) else { continue };
                if ev.flags & CEF_GLANCING != 0 || ev.strength < 0.3 {
                    tracing::info!("combat: a glancing blow on {:#x}: nothing lands", ev.actor);
                    continue;
                }
                let power = ev.flags & CEF_POWER != 0;
                type Hit = unsafe extern "thiscall" fn(usize, u8, usize, usize);
                let f = mem::deref(mem::deref(player) + VT_HIT);
                tracing::info!("combat: the player hits {:#x} (strength {:.2}{})", ev.actor, ev.strength, if power { ", a power attack" } else { "" });
                if f != 0 {
                    unsafe { std::mem::transmute::<usize, Hit>(f)(player, power as u8, 0, target) };
                }
            }
            CE_CAST => {
                let Some(&spell) = spells.get(&ev.what) else { continue };
                cast(player, spell, ev, actors);
            }
            _ => {}
        }
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
    let target = if ev.actor != 0 { actors.get(&ev.actor).copied().unwrap_or(0) } else { 0 };
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
    let spells_now = (k.frame % 10 == 1).then(spells);
    crate::link::with_adapter(|a| {
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
