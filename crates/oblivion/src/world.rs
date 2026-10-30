//! The world's moving parts, for the hands: the doors that swing in place (inside buildings, ruins,
//! cells, gates, cupboards) and the activators worked by moving a part of them (levers, switches,
//! buttons, pull chains, wheels), each offered to the runtime as a hinge or a slide
//! ([`OBJ_VALVE`]), worked by hand, with the game's own activation where the game would have it.
//!
//! **How each moves is the game's own model's.** The door's "Open" animation (an activator's
//! "Forward") turns its leaf about the leaf node's own hinge, or slides it; the adapter reads it
//! from the game's archives (the model the form names) and samples it ([`Animation`]): each part
//! that moves and has its own Havok body (a door's leaf, both leaves of a double door, a lever's
//! handle) is a joint, its axis and pivot from the rotation between the animation's start and its
//! furthest pose, its travel how far; the other nodes it moves (a door's handles, a lever's gears)
//! go with it, posed by the animation at the matching moment. Its sounds are the animation's
//! ("Sound: DRSMetalOpen01" where it plays).
//!
//! **Doors.** Load doors stay the game's Activate. A door that swings in place is latched while the
//! game has it shut: it moves only a hair from its frame ([`LATCH_PLAY`]), and a hand pulling it
//! (or a palm pushing it) to the latch's end is the game's Activate, as the player's own would be:
//! its script, its lock and key (picks: the game's lock; none: its "locked" and its rattle), its
//! open state. If the game opens it, the latch lets go and the hand swings it on: the game's own
//! animation is stopped there (the hand is the animation now), its open sound played, and its leaf,
//! handles and keyframed Havok body put where the runtime has it every frame (so the player and the
//! NPCs meet it where it's drawn). Swung back into its frame, the game's Activate again: closed,
//! its close sound, latched. The game opening or closing it itself (the trigger's Activate at gaze,
//! an NPC, a script) plays its animation, and the runtime's leaf follows. A door that slides (a
//! ruin's stone door, a portcullis) is too heavy for a hand: a tug on it is the game's Activate,
//! and the game's animation moves it.
//!
//! **Activators.** The lever, switch, button or chain moves by hand along its own "Forward"
//! travel; at its end ([`FIRE_AT`]), the game's Activate (its script does what it does). One whose
//! animation comes back to where it began (a pull chain, a button) springs back when let go, as
//! fast as its animation comes back; a two-way lever stays where it's put, and back at its start
//! it's the game's Activate again.
//!
//! **Their weight.** Oblivion gives these parts no mass (their bodies are keyframed), so a part's
//! mass is its Havok shape's volume times its Havok material's density, and its hinge's friction
//! is its pins' (its weight on them, [`PIN_FRICTION`] on a [`PIN_RADIUS`] pin). A hinge more than
//! halfway to level (a trap door) turns under its own weight.
//!
//! The engine facts are in `docs/research/world.md`.

use glam::{Quat, Vec3};
use oblivion_assets::nif::Nif;
use rustvr_protocol::*;

/// Samples taken along an animation.
pub const SAMPLES: usize = 33;
/// A node the animation turns less than this (radians) and slides less than this (game units) is
/// still.
pub const STILL_TURN: f32 = 0.09;
pub const STILL_SLIDE: f32 = 2.0;
/// How far a latched door gives (radians at its hinge; a slide's, metres): its rattle in its frame.
pub const LATCH_PLAY: f32 = 0.035;
/// A door pulled this far into its latch's play tries the game's Activate.
pub const TUG: f32 = 0.8;
/// A door within this of its frame (radians) is shut: the game's Activate closes it.
pub const SHUT: f32 = 0.012;
/// An activator worked this far along its travel (of the whole) fires; it's ready again once it's
/// back within [`REARM`] of where it started (or, a two-way lever, of the other end).
pub const FIRE_AT: f32 = 0.88;
pub const REARM: f32 = 0.35;
/// How hard a lever's two positions hold it (newtons at the end of its handle): more than a hand
/// resting on it, less than a deliberate push.
pub const DETENT_FORCE: f32 = 25.0;
/// How much more than its own weight's turn a lid's stay holds at the end of its swing.
pub const STAY: f32 = 1.3;
/// How long (seconds) a door's swing takes to die away once let go (its damping's time).
pub const SETTLE: f32 = 2.0;
/// Its pins: the friction of iron on iron, greased or not, and their radius (metres).
pub const PIN_FRICTION: f32 = 0.3;
pub const PIN_RADIUS: f32 = 0.01;

/// How a part moves, in its node's parent's space (game units).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    /// Turned about the line through `pivot` along `axis` (positive: the way the animation goes).
    Turn { axis: Vec3, pivot: Vec3 },
    /// Slid along `dir`.
    Slide { dir: Vec3 },
}

/// A node the animation moves: its local transform at each sample.
#[derive(Clone, Debug)]
pub struct Track {
    pub node: String,
    pub poses: Vec<(Quat, Vec3)>,
}

/// A part that moves: its node, how, and how far it's gone at each sample (radians or game units).
#[derive(Clone, Debug)]
pub struct Joint {
    pub track: usize,
    pub motion: Motion,
    pub travel: Vec<f32>,
    /// Its furthest (the animation's end for a door; the bottom of a pull chain's stroke), and the
    /// sample it's at.
    pub far: f32,
    pub far_at: usize,
    /// Its Havok body's mass in the model (kg; Oblivion leaves most doors' at 0), and the box round
    /// its drawn shapes in its own frame (half extents, game units).
    pub mass: f32,
    pub drawn: Option<(Vec3, f32)>,
}

/// A model's animation, sampled.
#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub times: Vec<f32>,
    pub tracks: Vec<Track>,
    /// The tracks that move far enough to be worked by hand, as joints.
    pub joints: Vec<Joint>,
    /// Its sounds: (time, the sound's editor id).
    pub sounds: Vec<(f32, String)>,
    /// It comes back to where it began (a pull chain, a button).
    pub returns: bool,
}

/// A rotation's angle about `axis` (radians, the short way).
pub fn angle_about(q: Quat, axis: Vec3) -> f32 {
    let q = if q.w < 0.0 { -q } else { q };
    2.0 * Vec3::new(q.x, q.y, q.z).dot(axis).atan2(q.w)
}

/// The fixed line of the rigid motion `p -> r p + d` (a rotation `r` about `axis`): its point
/// nearest the origin.
pub fn pivot_of(r: Quat, axis: Vec3, d: Vec3) -> Vec3 {
    let angle = angle_about(r, axis);
    let flat = d - axis * axis.dot(d);
    if angle.abs() < 1e-4 {
        return Vec3::ZERO;
    }
    (flat + axis.cross(flat) / (angle * 0.5).tan()) * 0.5
}

impl Animation {
    /// `name`'s samples from the model, and the joints it makes.
    pub fn of(nif: &Nif, name: &str) -> Option<Animation> {
        let s = nif.sequence(name)?;
        let (t0, t1) = nif.sequence_span(s);
        let times: Vec<f32> = (0..SAMPLES).map(|k| t0 + (t1 - t0) * k as f32 / (SAMPLES - 1) as f32).collect();
        let mut tracks: Vec<Track> = Vec::new();
        for &t in &times {
            for (k, (node, x)) in nif.sequence_pose(s, t).into_iter().enumerate() {
                let pose = (Quat::from_mat3(&x.rotation).normalize(), x.translation);
                match tracks.get_mut(k) {
                    Some(tr) => tr.poses.push(pose),
                    None => tracks.push(Track { node, poses: vec![pose] }),
                }
            }
        }
        let mut joints = Vec::new();
        let parents = nif.parents();
        for (k, tr) in tracks.iter().enumerate() {
            if let Some(mut j) = joint(k, tr) {
                if let Some(node) = nif.find(&tr.node) {
                    (j.mass, j.drawn) = (body_mass(nif, node), drawn_box(nif, node, &parents));
                }
                joints.push(j);
            }
        }
        // ("Sound: X", or "sound: X" in some.)
        let sound = |k: &str| k.get(..6).filter(|p| p.eq_ignore_ascii_case("sound:")).map(|_| k[6..].trim().to_owned());
        let sounds = nif.text_keys(s).iter().filter_map(|(t, k)| Some((*t, sound(k)?))).filter(|(_, k)| !k.is_empty()).collect();
        let returns = joints.first().is_some_and(|j| j.travel.last().is_some_and(|l| l.abs() < j.far.abs() * 0.3));
        Some(Animation { name: s.name.clone(), times, tracks, joints, sounds, returns })
    }

    /// Where joint `j` is `q` along (radians or game units), as a fractional sample: the first
    /// place on the way out to its furthest it gets there.
    pub fn sample_at(&self, j: usize, q: f32) -> f32 {
        let Some(joint) = self.joints.get(j) else { return 0.0 };
        let t = &joint.travel;
        for k in 1..=joint.far_at {
            if t[k] >= q {
                let span = (t[k] - t[k - 1]).max(1e-6);
                return (k - 1) as f32 + ((q - t[k - 1]) / span).clamp(0.0, 1.0);
            }
        }
        joint.far_at as f32
    }

    /// Track `k`'s pose at fractional sample `f`.
    pub fn pose(&self, k: usize, f: f32) -> (Quat, Vec3) {
        let p = &self.tracks[k].poses;
        let i = (f.floor().max(0.0) as usize).min(p.len() - 1);
        let n = (i + 1).min(p.len() - 1);
        let s = f - i as f32;
        (p[i].0.slerp(p[n].0, s).normalize(), p[i].1.lerp(p[n].1, s))
    }
}

/// The mass the model gives node `node`'s Havok body (0: none).
fn body_mass(nif: &Nif, node: i32) -> f32 {
    use oblivion_assets::nif::Block;
    let Some(av) = nif.av(node) else { return 0.0 };
    match nif.get(av.collision) {
        Some(Block::CollisionObject { body, .. }) => match nif.get(*body) {
            Some(Block::RigidBody(r)) if r.mass.is_finite() => r.mass.max(0.0),
            _ => 0.0,
        },
        _ => 0.0,
    }
}

/// What's drawn under node `node`, in its own frame: the box round it (half extents) and its volume
/// (its triangles' closed surface, by the divergence theorem; never more than the box).
fn drawn_box(nif: &Nif, node: i32, parents: &[i32]) -> Option<(Vec3, f32)> {
    use oblivion_assets::nif::Block;
    let to_node = nif.world(node, parents).inverse();
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let mut volume = 0.0f32;
    for (i, b) in nif.blocks.iter().enumerate() {
        let Block::Shape { data, .. } = b else { continue };
        let mut p = parents.get(i).copied().unwrap_or(-1);
        while p >= 0 && p != node {
            p = parents[p as usize];
        }
        let Some(Block::ShapeData(d)) = (p == node).then(|| nif.get(*data)).flatten() else { continue };
        let x = nif.world(i as i32, parents).then(&to_node);
        let v: Vec<Vec3> = d.vertices.iter().map(|v| x.apply(*v)).collect();
        for q in &v {
            lo = lo.min(*q);
            hi = hi.max(*q);
        }
        let shape: f32 = d.triangles.iter().filter_map(|t| Some(v.get(t[0] as usize)?.dot(v.get(t[1] as usize)?.cross(*v.get(t[2] as usize)?)) / 6.0)).sum();
        volume += shape.abs();
    }
    (lo.x <= hi.x).then(|| {
        let half = (hi - lo) * 0.5;
        (half, volume.min(8.0 * half.x * half.y * half.z))
    })
}

/// A track as a joint, if it moves enough: turned (by the rotation between its start and its
/// furthest) or slid.
fn joint(k: usize, tr: &Track) -> Option<Joint> {
    let (r0, t0) = *tr.poses.first()?;
    let turned: Vec<f32> = tr.poses.iter().map(|(r, _)| (*r * r0.inverse()).normalize().angle_between(Quat::IDENTITY)).collect();
    let slid: Vec<f32> = tr.poses.iter().map(|(_, t)| t.distance(t0)).collect();
    let (far_at, turn) = turned.iter().copied().enumerate().max_by(|a, b| a.1.total_cmp(&b.1))?;
    if turn > STILL_TURN {
        let r = (tr.poses[far_at].0 * r0.inverse()).normalize();
        let (axis, angle) = r.to_axis_angle();
        let axis = if angle > std::f32::consts::PI { -axis } else { axis }.normalize();
        let pivot = pivot_of(r, axis, tr.poses[far_at].1 - r * t0);
        let travel: Vec<f32> = tr.poses.iter().map(|(q, _)| angle_about((*q * r0.inverse()).normalize(), axis)).collect();
        let far = travel[far_at];
        return Some(Joint { track: k, motion: Motion::Turn { axis, pivot }, travel, far, far_at, mass: 0.0, drawn: None });
    }
    let (far_at, slide) = slid.iter().copied().enumerate().max_by(|a, b| a.1.total_cmp(&b.1))?;
    if slide > STILL_SLIDE {
        let dir = (tr.poses[far_at].1 - t0).normalize();
        let travel: Vec<f32> = tr.poses.iter().map(|(_, t)| (*t - t0).dot(dir)).collect();
        return Some(Joint { track: k, motion: Motion::Slide { dir }, travel, far: slide, far_at, mass: 0.0, drawn: None });
    }
    None
}

/// Bethesda's Havok material (Oblivion's 0..31) as a density, kg a cubic metre: stone, cloth, dirt,
/// glass, grass, metal, organic, skin, water, wood, heavy stone, heavy metal, heavy wood, chain,
/// snow; then each again as stairs.
pub fn density(material: u32) -> f32 {
    match if (15..30).contains(&material) { material - 15 } else { material } {
        0 | 10 => 2500.0,
        1 => 400.0,
        2 => 1600.0,
        3 => 2500.0,
        4 | 14 => 300.0,
        5 | 11 | 13 => 7800.0,
        6 | 7 | 8 => 1000.0,
        12 => 800.0,
        _ => 650.0,
    }
}

/// The mass of the game's own iron gates per square metre of their leaf (the cell gates' bodies
/// have one: 100 kg on 0.68 by 2.28 m), and a solid plank door's thickness (metres).
pub const GATE_MASS_PER_AREA: f32 = 64.5;
pub const PLANK: f32 = 0.045;

/// What a part weighs that the game gives no mass (most of its doors and levers): `half` its
/// collision's half size (metres), `drawn` its drawn shapes' box (half size, game units) and volume
/// (game units cubed), `material` its Havok material. A leaf (thin one way: a door, a lid) is as
/// heavy as its area of plank of its material, or, of metal, as the game's own gates (a drawn leaf's
/// volume counts its hinges, handles and bands as solid); anything else (a lever, a stone) is its
/// drawn shape's volume of its material.
pub fn part_mass(half: Vec3, drawn: Option<(Vec3, f32)>, material: u32) -> f32 {
    let rho = density(material);
    let units = crate::camera::UNITS_PER_METER;
    let size = drawn.map_or(half, |(h, _)| h / units);
    let mut e = [size.x, size.y, size.z];
    e.sort_by(|a, b| a.total_cmp(b));
    let mass = if e[1] > 4.0 * e[0] {
        let area = 4.0 * e[1] * e[2];
        if rho > 3000.0 { area * GATE_MASS_PER_AREA } else { area * PLANK * rho }
    } else {
        match drawn {
            Some((_, v)) if v > 0.0 => v / units.powi(3) * rho,
            _ => 8.0 * e[0] * e[1] * e[2] * rho,
        }
    };
    mass.clamp(0.2, 2000.0)
}

/// A rotation that takes +Z to `axis`.
pub fn frame_of(axis: Vec3) -> Quat {
    Quat::from_rotation_arc(Vec3::Z, axis.normalize()).normalize()
}

/// A part's object for the runtime: the hinge or slide at `centre` along `base`'s +Z, its shape the
/// game's (`hull`, in its frame), its weight and friction, how far it moves now (`min`..`max`), its
/// spring and damping, its kind (`JOINT_*`).
#[allow(clippy::too_many_arguments)]
pub fn part_desc(id: u32, seq: u32, centre: Vec3, base: Quat, hull: &[Vec3], mass: f32, friction: f32, range: (f32, f32), spring: (f32, f32), flags: u32) -> ObjectDesc {
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_VALVE;
    d.flags = OBJF_GAME_MOVED;
    d.spawn_seq = seq;
    d.spawn_pose = Pose { pos: centre.to_array(), valid: 1, rot: base.to_array() };
    d.shape = SHAPE_HULL;
    let (c, half) = crate::items::bounds(hull).unwrap_or((Vec3::ZERO, Vec3::splat(0.05)));
    d.center = c.to_array();
    d.half_extents = half.max(Vec3::splat(0.01)).to_array();
    // (Its rim: as far as it reaches from the axle.)
    d.radius = hull.iter().map(|p| p.truncate().length()).fold(0.02, f32::max);
    d.mass = mass;
    d.friction = friction;
    (d.turn_min, d.turn_max) = range;
    (d.joint_spring, d.joint_damping) = spring;
    d.joint_flags = flags;
    d
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, av};
    use glam::Mat3;
    use crate::{camera, mem};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DELETED: u32 = 0x20;
    const DISABLED: u32 = 0x800;
    const ACTIVATOR: u8 = 0x12;
    const DOOR: u8 = 0x18;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const REFR_EXTRA: usize = 0x44;
    /// `TESModel` (+0x30 in doors and activators): its path (`BSStringT`) at +4.
    const MODEL_PATH: usize = 0x34;
    /// `TESObjectDOOR` flags (+0x64): an Oblivion gate; its sounds (+0x58: opening, closing).
    const DOOR_FLAGS: usize = 0x64;
    const DOOR_SOUNDS: usize = 0x58;
    const DOOR_OBLIVION_GATE: u8 = 1;
    /// `TESObjectACTI`'s script (`TESScriptableForm` +0x48, its script at +4).
    const ACTI_SCRIPT: usize = 0x4C;
    /// `BaseExtraList::GetExtra(type)` (thiscall); `ExtraTeleport` (a load door) and `ExtraLock`
    /// (its data +0xC: level +0, key +4, flags +8, 1 locked).
    const GET_EXTRA: usize = 0x0041_E210;
    const EXTRA_TELEPORT: u32 = 0x32;
    const EXTRA_LOCK: u32 = 0x31;
    /// `GetOpenState` (thiscall on the reference): 1 open, 2 opening, 3 closed, 4 closing.
    const GET_OPEN_STATE: usize = 0x004D_E660;
    /// `NiObjectNET`: name, controller; `NiAVObject`: parent, collision object; `NiNode`: children.
    const NODE_NAME: usize = 0x08;
    const NODE_CONTROLLER: usize = 0x0C;
    const NODE_PARENT: usize = 0x1C;
    const NODE_COLLISION: usize = 0xA8;
    const NODE_CHILDREN: usize = 0xB0;
    const NODE_COUNT: usize = 0xB6;
    /// `NiRTTI` cast (cdecl: rtti, object) and the controller manager's RTTI; its sequence by name
    /// (thiscall, one argument); a sequence's state (+0x44: 1 animating) and its `Deactivate(ease
    /// out, transition)` (thiscall).
    const NI_CAST: usize = 0x0056_0920;
    const RTTI_CONTROLLER_MANAGER: usize = 0x00B3_CAC0;
    const RTTI_NODE: usize = 0x00B3_FAB0;
    const SEQUENCE_BY_NAME: usize = 0x0047_15A0;
    const SEQUENCE_STATE: usize = 0x44;
    const SEQUENCE_DEACTIVATE: usize = 0x006C_9CB0;
    /// The sound form by its editor id (stdcall; a `TESSound`, form type 0x0A). The sounds, as the
    /// `PlaySound3D` command plays one: the sound manager (`OSGlobals` +0x24); a sound made from a
    /// form (thiscall on the manager: form id, flags 0x102, 0), put at a place, tied to its
    /// reference, played, given up (0x6AE0A0, 0x6B7360, 0x6AC3E0, 0x6B7190, 0x6B73E0,
    /// 0x401F20).
    const SOUND_BY_NAME: usize = 0x0044_7490;
    const OS_SOUND_MANAGER: usize = 0x24;
    const SOUND_MAKE: usize = 0x006A_E0A0;
    const SOUND_FLAGS_3D: u32 = 0x102;
    const SOUND_PLACE: usize = 0x006B_7360;
    const SOUND_TIE: usize = 0x006A_C3E0;
    const SOUND_PLAY: usize = 0x006B_7190;
    const SOUND_STARTED: usize = 0x006B_73E0;
    const SOUND_RELEASE: usize = 0x0040_1F20;
    /// `TESObjectREFR::Activate` (thiscall: who activates it, 0, 0, 1), as the player's Activate
    /// control and the lock's opening call it.
    const REFR_ACTIVATE: usize = 0x004D_D260;
    /// `bhkNiCollisionObject` -> `bhkRigidBody` (+0x10) -> `hkRigidBody` (+8); its shape (+0x14) and
    /// the shape's Bethesda wrapper (user data +8), whose material is at +0x10.
    const COLLISION_BODY: usize = 0x10;
    const HK_OBJ: usize = 0x8;
    const ENTITY_SHAPE: usize = 0x14;
    const WRAPPER_MATERIAL: usize = 0x10;
    /// The collision under a node told to follow it (cdecl: node, all below); `NiAVObject::Update`.
    const RESET_COLLISION: usize = 0x0089_7A20;
    const UPDATE_NODE: usize = 0x0070_7370;

    /// Metres round the player doors and activators are offered within, and at most how many.
    const REACH: f32 = 6.0;
    const MAX_THINGS: usize = 24;
    const SCAN_EVERY: f64 = 0.25;
    /// Seconds between the game's Activates of one thing (a refused tug waits this long, and for the
    /// hand to ease off, before it tries again).
    const RETRY: f64 = 1.0;
    /// The first id of the world's parts in the object table.
    const ID_BASE: u32 = 0x7F00_0000;

    fn deref(p: usize) -> usize {
        mem::deref(p)
    }

    fn extra(refr: usize, kind: u32) -> usize {
        type Get = unsafe extern "thiscall" fn(usize, u32) -> usize;
        unsafe { std::mem::transmute::<usize, Get>(GET_EXTRA)(refr + REFR_EXTRA, kind) }
    }

    fn locked(refr: usize) -> bool {
        let x = extra(refr, EXTRA_LOCK);
        let data = if x != 0 { deref(x + 0xC) } else { 0 };
        data != 0 && mem::at::<u8>(data + 8).is_some_and(|f| f & 1 != 0)
    }

    fn open_state(refr: usize) -> i32 {
        type F = unsafe extern "thiscall" fn(usize) -> i32;
        unsafe { std::mem::transmute::<usize, F>(GET_OPEN_STATE)(refr) }
    }

    fn name_of(node: usize) -> String {
        crate::combat::read_str(deref(node + NODE_NAME))
    }

    /// Whether a scene graph object is a node (it has children; a shape, say, doesn't).
    fn is_node(obj: usize) -> bool {
        type Cast = unsafe extern "C" fn(usize, usize) -> usize;
        obj != 0 && mem::is_readable(obj as *const u8, 0xB8) && unsafe { std::mem::transmute::<usize, Cast>(NI_CAST)(RTTI_NODE, obj) } != 0
    }

    /// A node's children that are nodes.
    fn child_nodes(node: usize) -> Vec<usize> {
        let data = deref(node + NODE_CHILDREN);
        let n = mem::at::<u16>(node + NODE_COUNT).unwrap_or(0).min(256) as usize;
        if data == 0 || !mem::is_readable(data as *const u8, n * 4) {
            return Vec::new();
        }
        (0..n).map(|i| deref(data + i * 4)).filter(|&c| is_node(c)).collect()
    }

    /// The nodes under `root` (itself too), by name, in the model's order: a name the model has
    /// more than once is `name@#0` the second time, `name@#1` the third (as a model's animations
    /// name them).
    fn nodes(root: usize) -> HashMap<String, usize> {
        let mut out = HashMap::new();
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut stack = vec![(root, 0u32)];
        let mut count = 0;
        while let Some((node, depth)) = stack.pop() {
            count += 1;
            if count > 2000 || depth > 32 {
                continue;
            }
            let name = name_of(node);
            let k = seen.entry(name.clone()).or_insert(0);
            let key = if *k == 0 { name } else { format!("{name}@#{}", *k - 1) };
            *k += 1;
            out.entry(key).or_insert(node);
            // (Its children in order: the first is looked at next.)
            stack.extend(child_nodes(node).into_iter().rev().map(|c| (c, depth + 1)));
        }
        out
    }

    /// The Havok body of the first node at or under `node` that has one (`hkRigidBody*`), and that
    /// node; not under any of `others` (another part's).
    fn body_under(node: usize, others: &[usize]) -> Option<(usize, usize)> {
        let mut stack = vec![(node, 0u32)];
        while let Some((n, depth)) = stack.pop() {
            if depth > 16 || (n != node && others.contains(&n)) {
                continue;
            }
            let co = deref(n + NODE_COLLISION);
            let hk = if co != 0 { deref(deref(co + COLLISION_BODY) + HK_OBJ) } else { 0 };
            if hk != 0 && mem::is_readable(hk as *const u8, 0xC4) {
                return Some((n, hk));
            }
            stack.extend(child_nodes(n).into_iter().map(|c| (c, depth + 1)));
        }
        None
    }

    fn world_of(node: usize) -> Option<(Mat3, Vec3, f32)> {
        let r: [f32; 9] = mem::at(node + av::WORLD_ROTATE)?;
        let t: [f32; 3] = mem::at(node + av::WORLD_TRANSLATE)?;
        let s: f32 = mem::at(node + av::WORLD_SCALE)?;
        let m = crate::items::ni_matrix(r);
        (m.is_finite() && Vec3::from(t).is_finite()).then_some((m, Vec3::from(t), s))
    }

    fn local_of(node: usize) -> Option<(Quat, Vec3)> {
        let r: [f32; 9] = mem::at(node + av::LOCAL_ROTATE)?;
        let t: [f32; 3] = mem::at(node + av::LOCAL_TRANSLATE)?;
        let m = crate::items::ni_matrix(r);
        m.is_finite().then(|| (Quat::from_mat3(&m).normalize(), Vec3::from(t)))
    }

    fn set_local(node: usize, rot: Quat, pos: Vec3) {
        if !mem::is_readable(node as *const u8, 0xB8) || !rot.is_finite() || !pos.is_finite() {
            return;
        }
        unsafe {
            ((node + av::LOCAL_ROTATE) as *mut [f32; 9]).write_unaligned(crate::items::ni_rows(Mat3::from_quat(rot)));
            ((node + av::LOCAL_TRANSLATE) as *mut [f32; 3]).write_unaligned(pos.to_array());
        }
    }

    fn update(node: usize) {
        type Update = unsafe extern "thiscall" fn(usize, f32, u32) -> u32;
        unsafe { std::mem::transmute::<usize, Update>(UPDATE_NODE)(node, 0.0, 0) };
    }

    fn reset_collision(node: usize) {
        type Reset = unsafe extern "C" fn(usize, u8);
        unsafe { std::mem::transmute::<usize, Reset>(RESET_COLLISION)(node, 1) };
    }

    /// The reference's animations' controller manager (on its 3D's root, or the model's root under
    /// it), or 0.
    fn manager(root: usize) -> usize {
        type Cast = unsafe extern "C" fn(usize, usize) -> usize;
        let first = child_nodes(root).first().copied().unwrap_or(0);
        for n in [root, first] {
            let c = if n != 0 { deref(n + NODE_CONTROLLER) } else { 0 };
            if c != 0 && mem::is_readable(c as *const u8, 0x60) {
                let m = unsafe { std::mem::transmute::<usize, Cast>(NI_CAST)(RTTI_CONTROLLER_MANAGER, c) };
                if m != 0 {
                    return m;
                }
            }
        }
        0
    }

    fn sequence(manager: usize, name: &str) -> usize {
        type ByName = unsafe extern "thiscall" fn(usize, *const u8) -> usize;
        let Ok(c) = std::ffi::CString::new(name) else { return 0 };
        if manager == 0 {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, ByName>(SEQUENCE_BY_NAME)(manager, c.as_ptr() as *const u8) }
    }

    fn animating(seq: usize) -> bool {
        seq != 0 && mem::at::<u32>(seq + SEQUENCE_STATE) == Some(1)
    }

    /// Stops the game's animation where it is (the hand is the animation now).
    fn stop(seq: usize) {
        type Deactivate = unsafe extern "thiscall" fn(usize, f32, u8) -> u8;
        if seq != 0 && mem::at::<u32>(seq + SEQUENCE_STATE).is_some_and(|s| s != 0) {
            unsafe { std::mem::transmute::<usize, Deactivate>(SEQUENCE_DEACTIVATE)(seq, 0.0, 0) };
        }
    }

    /// Plays a door's own sound (its form's: 0 opening, 1 closing) at it.
    fn play_door_sound(refr: usize, which: usize) {
        let base = deref(refr + REFR_BASE);
        let sound = deref(base + DOOR_SOUNDS + which * 4);
        if let Some(id) = (sound != 0).then(|| mem::at::<u32>(sound + FORM_ID)).flatten() {
            play(refr, id);
        }
    }

    /// Plays one of the game's sounds (by its editor id) at a reference.
    fn play_sound(refr: usize, sound: &str) {
        type ByName = unsafe extern "system" fn(*const u8) -> usize;
        let Ok(c) = std::ffi::CString::new(sound) else { return };
        let s = unsafe { std::mem::transmute::<usize, ByName>(SOUND_BY_NAME)(c.as_ptr() as *const u8) };
        match (s != 0).then(|| mem::at::<u32>(s + FORM_ID)).flatten() {
            Some(id) => play(refr, id),
            None => tracing::info!("world: no sound {sound}"),
        }
    }

    /// Plays sound form `id` at a reference, as the game's `PlaySound3D` does.
    fn play(refr: usize, id: u32) {
        type Make = unsafe extern "thiscall" fn(usize, u32, u32, u8) -> usize;
        type Place = unsafe extern "thiscall" fn(usize, f32, f32, f32) -> i32;
        type Tie = unsafe extern "thiscall" fn(usize, u32, usize) -> i32;
        type Play = unsafe extern "thiscall" fn(usize, u8) -> i32;
        type Started = unsafe extern "thiscall" fn(usize) -> i32;
        type Release = unsafe extern "C" fn(usize) -> i32;
        let os = deref(engine::OS_GLOBALS);
        let mgr = if os != 0 { deref(os + OS_SOUND_MANAGER) } else { 0 };
        let Some(at) = mem::at::<[f32; 3]>(refr + engine::refr::POS) else { return };
        if mgr == 0 {
            return;
        }
        unsafe {
            let h = std::mem::transmute::<usize, Make>(SOUND_MAKE)(mgr, id, SOUND_FLAGS_3D, 0);
            tracing::info!("world: the game's sound {id:08X} at {:08X}{}", mem::at::<u32>(refr + FORM_ID).unwrap_or(0), if h == 0 { ": none made" } else { "" });
            if h == 0 {
                return;
            }
            std::mem::transmute::<usize, Place>(SOUND_PLACE)(h, at[0], at[1], at[2]);
            std::mem::transmute::<usize, Tie>(SOUND_TIE)(mgr, deref(h) as u32, refr);
            std::mem::transmute::<usize, Play>(SOUND_PLAY)(h, 0);
            std::mem::transmute::<usize, Started>(SOUND_STARTED)(h);
            std::mem::transmute::<usize, Release>(SOUND_RELEASE)(h);
        }
    }

    /// The player activates it, as the Activate control does (its script, its lock, its own).
    pub(crate) fn activate(refr: usize) -> bool {
        type Activate = unsafe extern "thiscall" fn(usize, usize, u32, u32, u32) -> u8;
        let player = engine::player();
        if player == 0 || refr == 0 {
            return false;
        }
        unsafe { std::mem::transmute::<usize, Activate>(REFR_ACTIVATE)(refr, player, 0, 0, 1) != 0 }
    }

    // ---- the models, read from the game's archives on a thread of their own ------------------------
    #[derive(Default)]
    struct Models {
        archives: Option<Arc<oblivion_assets::bsa::Archives>>,
        opening: bool,
        /// Model path -> its two animations (out and back), or None if it has none to work.
        done: HashMap<String, Option<Arc<(Animation, Option<Animation>)>>>,
        /// Models to read, each with the animations to read from it (out, and back).
        wanted: Vec<(String, &'static str, &'static str)>,
        busy: bool,
    }
    static MODELS: Mutex<Option<Models>> = Mutex::new(None);

    fn with_models<R>(f: impl FnOnce(&mut Models) -> R) -> R {
        f(MODELS.lock().unwrap().get_or_insert_with(Models::default))
    }

    /// The model's animations (`out`, then `back`), if they're read; None while they're being read.
    fn model(path: &str, out: &'static str, back: &'static str) -> Option<Option<Arc<(Animation, Option<Animation>)>>> {
        let key = path.to_ascii_lowercase();
        let start = with_models(|m| {
            if let Some(d) = m.done.get(&key) {
                return Err(d.clone());
            }
            if !m.wanted.iter().any(|w| w.0 == key) {
                m.wanted.push((key.clone(), out, back));
            }
            let start = !m.busy;
            m.busy = true;
            Ok(start)
        });
        match start {
            Err(d) => Some(d),
            Ok(true) => {
                std::thread::spawn(read_models);
                None
            }
            Ok(false) => None,
        }
    }

    fn read_models() {
        let archives = with_models(|m| m.archives.clone());
        let archives = match archives {
            Some(a) => a,
            None => {
                let game = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default();
                match oblivion_assets::bsa::Archives::open(&game) {
                    Ok(a) => {
                        let a = Arc::new(a);
                        with_models(|m| m.archives = Some(a.clone()));
                        tracing::info!("world: the game's archives opened ({} of them)", a.bsas.len());
                        a
                    }
                    Err(e) => {
                        tracing::warn!("world: the game's archives: {e:#}");
                        with_models(|m| {
                            m.busy = false;
                            m.opening = false;
                        });
                        return;
                    }
                }
            }
        };
        loop {
            let Some((path, out, back)) = with_models(|m| m.wanted.pop()) else { break };
            let read = archives.read(&format!("meshes\\{path}")).and_then(|d| Nif::parse(&d));
            // (A model that trips the reader is one with nothing to work, not the end of reading.)
            let anims = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match read {
                Ok(nif) => {
                    let a = Animation::of(&nif, out);
                    if a.as_ref().is_some_and(|a| a.joints.is_empty()) {
                        tracing::info!("world: {path}: its '{out}' moves nothing far enough");
                    }
                    a.filter(|a| !a.joints.is_empty()).map(|a| Arc::new((a, Animation::of(&nif, back))))
                }
                Err(e) => {
                    tracing::info!("world: {path}: {e:#}");
                    None
                }
            }))
            .unwrap_or_else(|_| {
                tracing::warn!("world: {path}: reading its animation failed");
                None
            });
            if let Some(a) = &anims {
                let j = &a.0.joints[0];
                tracing::info!("world: {path}: '{}' moves {} part(s) (the first '{}' {:?} {:.2} over {} samples, mass {} drawn {:?}; sounds {:?}; back again: {})", a.0.name, a.0.joints.len(), a.0.tracks[j.track].node, j.motion, j.far, a.0.times.len(), j.mass, j.drawn, a.0.sounds, a.0.returns);
            }
            with_models(|m| m.done.insert(path, anims));
        }
        with_models(|m| m.busy = false);
    }

    // ---- the doors and activators near the player -----------------------------------------------
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        /// A door that swings in place, worked by hand.
        Door,
        /// A door too heavy for a hand (it slides): a tug is the game's Activate.
        HeavyDoor,
        Activator,
    }

    /// One part of a thing that moves, as the runtime has it.
    struct Part {
        id: u32,
        joint: usize,
        node: usize,
        body_node: usize,
        /// The hinge or slide in the runtime's world: its centre, its frame (+Z the axis, at the
        /// start of its travel), slide or turn, and the game units in one of its runtime units.
        centre: Vec3,
        base: Quat,
        slide: bool,
        units: f32,
        /// How far it moves (runtime units), and the tracks that go with it (and their nodes).
        far: f32,
        tracks: Vec<(usize, usize)>,
        desc: ObjectDesc,
        hull: Vec<Vec3>,
        /// Where the runtime has it, and whether a hand has it (which: bit 0 left, bit 1 right);
        /// where it was last logged.
        at: f32,
        vel: f32,
        held: bool,
        hands: u32,
        logged: f32,
    }

    struct Thing {
        refr: usize,
        form_id: u32,
        kind: Kind,
        root: usize,
        anims: Arc<(Animation, Option<Animation>)>,
        parts: Vec<Part>,
        /// The game has it open (a door), or a two-way lever is at its far end.
        open: bool,
        /// The runtime's parts are where the game has them (else the runtime follows the game's).
        synced: bool,
        /// The runtime has moved it from the game's pose: the adapter poses its nodes.
        owned: bool,
        /// The game's Activate to make in the next game frame, and when one was last made.
        pending: bool,
        tried: f64,
        armed: bool,
        locked: bool,
        /// The game's own animation is playing on it.
        game_moving: bool,
    }

    #[derive(Default)]
    struct State {
        things: HashMap<usize, Thing>,
        ids: HashMap<(u32, usize), u32>,
        next_id: u32,
        sent: Vec<ObjectDesc>,
        next_scan: f64,
        states: Option<Box<ObjectStates>>,
        republish: bool,
        /// References with nothing to work by hand (with the 3D they had then).
        none: HashMap<usize, usize>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    /// A reference that's a door or an activator to work by hand: its kind and its model's path.
    fn candidate(refr: usize) -> Option<(u8, String)> {
        let flags = mem::at::<u32>(refr + FORM_FLAGS)?;
        if flags & (DELETED | DISABLED) != 0 || deref(refr + REFR_NODE) == 0 {
            return None;
        }
        let base = deref(refr + REFR_BASE);
        let kind = mem::at::<u8>(base + FORM_TYPE)?;
        match kind {
            DOOR if extra(refr, EXTRA_TELEPORT) == 0 && mem::at::<u8>(base + DOOR_FLAGS).is_some_and(|f| f & DOOR_OBLIVION_GATE == 0) => {}
            ACTIVATOR if deref(base + ACTI_SCRIPT) != 0 => {}
            _ => return None,
        }
        let path = crate::combat::read_str(deref(base + MODEL_PATH));
        // (Traps fire by themselves, by the game's own triggers: not a hand's.)
        let file = path.rsplit(['\\', '/']).next().unwrap_or("").to_ascii_lowercase();
        if path.is_empty() || (kind == ACTIVATOR && (file.contains("trap") || path.to_ascii_lowercase().contains("\\traps\\"))) {
            return None;
        }
        Some((kind, path))
    }

    /// The doors and activators within `reach` of `at` (runtime world), nearest first.
    fn nearby(at: Vec3, reach: f32) -> Vec<(usize, u8, String, f32)> {
        let b = camera::basis();
        let mut out = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = deref(entry);
                entry = deref(entry + 4);
                if r == 0 {
                    continue;
                }
                let Some(p) = mem::at::<[f32; 3]>(r + engine::refr::POS) else { continue };
                let d = b.point_from_game(Vec3::from(p)).distance(at);
                if d > reach {
                    continue;
                }
                if let Some((kind, path)) = candidate(r) {
                    out.push((r, kind, path, d));
                }
            }
        }
        out.sort_by(|a, b| a.3.total_cmp(&b.3));
        out
    }

    /// A body's shape's material (its Bethesda wrapper's), or wood.
    fn material(body: usize) -> u32 {
        let shape = deref(body + ENTITY_SHAPE);
        let wrapper = if shape != 0 { deref(shape + 8) } else { 0 };
        (wrapper != 0).then(|| mem::at::<u32>(wrapper + WRAPPER_MATERIAL)).flatten().filter(|&m| m < 32).unwrap_or(9)
    }

    /// The thing a reference is, its parts from its model's animation and its live 3D.
    fn make(s: &mut State, refr: usize, kind: u8, anims: Arc<(Animation, Option<Animation>)>) -> Option<Thing> {
        let root = deref(refr + REFR_NODE);
        let form_id = mem::at::<u32>(refr + FORM_ID)?;
        let a = &anims.0;
        let live = nodes(root);
        let b = camera::basis();
        let joint_nodes: Vec<usize> = a.joints.iter().filter_map(|j| live.get(&a.tracks[j.track].node).copied()).collect();
        let mut parts = Vec::new();
        let mut kind = if kind == DOOR { Kind::Door } else { Kind::Activator };
        let mut why = Vec::new();
        for (k, j) in a.joints.iter().enumerate() {
            let tr = &a.tracks[j.track];
            let Some(&node) = live.get(&tr.node) else {
                why.push(format!("no node '{}'", tr.node));
                continue;
            };
            let others: Vec<usize> = joint_nodes.iter().copied().filter(|&n| n != node).collect();
            let Some((body_node, body)) = body_under(node, &others) else {
                why.push(format!("'{}' has no Havok body", tr.node));
                continue;
            };
            let parent = deref(node + NODE_PARENT);
            let Some((pr, pt, ps)) = (parent != 0).then(|| world_of(parent)).flatten() else { continue };
            let Some((now_r, now_t)) = local_of(node) else { continue };
            let (r0, t0) = tr.poses[0];
            // Where it is now along its travel (the game may have it open).
            let (slide, units, centre, axis, now) = match j.motion {
                Motion::Turn { axis, pivot } => {
                    let pivot_w = pt + pr * (pivot * ps);
                    let axis_w = (pr * axis).normalize();
                    // (The runtime's axis for the same turn: its handedness is the other.)
                    let turn = b.rot_from_game(Mat3::from_axis_angle(axis_w, 0.5));
                    let (axis_r, angle) = turn.to_axis_angle();
                    let axis_r = if angle > std::f32::consts::PI { -axis_r } else { axis_r };
                    (false, 1.0, b.point_from_game(pivot_w), axis_r.normalize(), angle_about((now_r * r0.inverse()).normalize(), axis))
                }
                Motion::Slide { dir } => {
                    let dir_w = (pr * dir).normalize();
                    let origin_w = pt + pr * (t0 * ps);
                    (true, camera::UNITS_PER_METER, b.point_from_game(origin_w), b.dir_from_game(dir_w).normalize(), (now_t - t0).dot(dir) / camera::UNITS_PER_METER)
                }
            };
            let base = frame_of(axis);
            // Its Havok shape, in its frame at the start of its travel.
            let undo = if slide { (Quat::IDENTITY, -axis * now) } else { (Quat::from_axis_angle(axis, -now), Vec3::ZERO) };
            let tris = crate::collision::body_triangles(body);
            let mut points: Vec<Vec3> = tris
                .iter()
                .flatten()
                .map(|w| {
                    let p = b.point_from_game(*w);
                    let p = if slide { p + undo.1 } else { centre + undo.0 * (p - centre) };
                    base.inverse() * (p - centre)
                })
                .filter(|p| p.is_finite() && p.length() < 12.0)
                .collect();
            points = crate::items::hull_points(&points, MAX_HULL_POINTS);
            if points.len() < 4 {
                why.push(format!("'{}' has no shape ({} triangles)", tr.node, tris.len()));
                continue;
            }
            let (_, half) = crate::items::bounds(&points).unwrap_or((Vec3::ZERO, Vec3::splat(0.05)));
            let mass = if j.mass > 0.0 { j.mass } else { part_mass(half, j.drawn, material(body)) };
            let far = j.far / units;
            let size = half.max_element() * 2.0;
            // A door that slides is too heavy for a hand (a ruin's stone, a portcullis); an
            // activator's big moving part is a gate or a bridge some lever works, not a hand.
            if kind == Kind::Door && slide {
                kind = Kind::HeavyDoor;
            }
            if kind == Kind::Activator && (size > 1.6 || (slide && far > 0.9)) {
                why.push(format!("'{}' is too big for a hand ({size:.1} m, {far:.2})", tr.node));
                continue;
            }
            let key = (form_id, k);
            let id = *s.ids.entry(key).or_insert_with(|| {
                s.next_id += 1;
                ID_BASE + s.next_id
            });
            // The nodes it moves: its own, and the others the animation moves that are under it
            // (a door's handles) or, the first part's, under no part at all (a lever's gears).
            let tracks: Vec<(usize, usize)> = a
                .tracks
                .iter()
                .enumerate()
                .filter(|(ti, t)| *ti == j.track || (a.joints.iter().all(|o| o.track != *ti) && t.poses.windows(2).any(|w| w[0] != w[1])))
                .filter_map(|(ti, t)| Some((ti, *live.get(&t.node)?)))
                .filter(|&(ti, n)| ti == j.track || (n != root && (under(n, node) || (k == 0 && !joint_nodes.iter().any(|&o| under(n, o))))))
                .collect();
            parts.push(Part { id, joint: k, node, body_node, centre, base, slide, units, far, tracks, desc: bytemuck::Zeroable::zeroed(), hull: points, at: now, vel: 0.0, held: false, hands: 0, logged: now });
            let p = parts.last_mut().unwrap();
            p.desc = part_desc(id, 1, centre, base, &p.hull, mass, 0.0, (0.0, far), (0.0, 0.0), 0);
            p.desc.friction = PIN_FRICTION * mass * 9.81 * PIN_RADIUS;
        }
        if parts.is_empty() {
            let mut names: Vec<&String> = live.keys().collect();
            names.sort();
            tracing::info!("world: {form_id:08X} ({}) has nothing to work by hand: {}; its nodes {names:?}", name_of(root), why.join("; "));
            return None;
        }
        let open = if matches!(kind, Kind::Door | Kind::HeavyDoor) { matches!(open_state(refr), 1 | 2) } else { parts[0].at > parts[0].far * 0.5 };
        let mut t = Thing { refr, form_id, kind, root, anims, parts, open, synced: false, owned: false, pending: false, tried: f64::MIN, armed: true, locked: locked(refr), game_moving: false };
        shape_descs(&mut t);
        tracing::info!(
            "world: {:?} {form_id:08X} ({}): {} part(s) [{}], open {}, locked {}",
            t.kind,
            name_of(root),
            t.parts.len(),
            t.parts
                .iter()
                .map(|p| {
                    let (c, h) = crate::items::bounds(&p.hull).unwrap_or_default();
                    format!("{:.0} kg {} at {:.2?} axis {:.2?}, {:.2} far, its middle at {:.2?} (half {:.2?})", p.desc.mass, if p.slide { "slides" } else { "turns" }, p.centre, p.base * Vec3::Z, p.far, p.centre + p.base * c, h)
                })
                .collect::<Vec<_>>()
                .join("; "),
            t.open,
            t.locked
        );
        Some(t)
    }

    /// Whether a thing's reference is still the one it was made from, with the same 3D (a cell left
    /// or reloaded, the reference deleted or disabled: its nodes are the game's to free).
    fn alive(refr: usize, t: &Thing) -> bool {
        mem::is_readable(refr as *const u8, 0x48)
            && mem::at::<u32>(refr + FORM_ID) == Some(t.form_id)
            && deref(refr + REFR_NODE) == t.root
            && mem::at::<u32>(refr + FORM_FLAGS).is_some_and(|f| f & (DELETED | DISABLED) == 0)
            && t.parts.iter().all(|p| mem::is_readable(p.node as *const u8, 0xB8))
    }

    /// Whether `node` is under `ancestor` in the live tree.
    fn under(node: usize, ancestor: usize) -> bool {
        let mut n = node;
        for _ in 0..32 {
            if n == ancestor {
                return true;
            }
            n = deref(n + NODE_PARENT);
            if n == 0 {
                break;
            }
        }
        false
    }

    /// Each part's description as the thing is now: a door latched or free to swing, a lever's
    /// spring, its weight moving it.
    fn shape_descs(t: &mut Thing) {
        let (anim, back_time) = (&t.anims.0, t.anims.0.times.last().copied().unwrap_or(0.0));
        for p in &mut t.parts {
            let j = &anim.joints[p.joint];
            let range = match t.kind {
                Kind::Door | Kind::HeavyDoor if !t.open => (0.0, LATCH_PLAY.min(p.far)),
                _ => (0.0, p.far),
            };
            // (What it is to a hand: a leaf turned about its edge, or its mass along a slide.)
            let inertia = if p.slide { p.desc.mass } else { p.desc.mass * (p.desc.radius * p.desc.radius / 3.0).max(1e-4) };
            // Springing back as fast as its animation comes back (critically damped, settled in
            // the time it takes the game's).
            let spring = if t.kind == Kind::Activator && anim.returns {
                let back = (back_time - anim.times[j.far_at]).max(0.2);
                let w = 4.0 / back;
                (inertia * w * w, 2.0 * inertia * w)
            } else {
                // A door (or a lever) let go mid-swing slows to a stop over a couple of seconds, as a
                // real one on its hinges does, rather than drifting on.
                (0.0, inertia / SETTLE)
            };
            let axis = p.base * Vec3::Z;
            let level = !p.slide && axis.y.abs() < std::f32::consts::FRAC_1_SQRT_2;
            let flags = JOINT_PANEL | if p.slide { JOINT_SLIDE } else { 0 } | if level && t.kind == Kind::Door { JOINT_GRAVITY } else { 0 };
            p.desc.turn_min = range.0;
            p.desc.turn_max = range.1;
            // A lever that stays where it's put holds at its ends (its two positions) against a
            // hand merely resting on it; a lid or a trap door props open at the end of its swing
            // (as a real one's stay holds it), against its own weight, and falls shut short of it.
            p.desc.detent = if t.kind == Kind::Activator && !anim.returns {
                DETENT_FORCE * if p.slide { 1.0 } else { p.desc.radius.max(0.1) }
            } else if flags & JOINT_GRAVITY != 0 {
                let (c, _) = crate::items::bounds(&p.hull).unwrap_or_default();
                STAY * p.desc.mass * 9.81 * c.truncate().length()
            } else {
                0.0
            };
            (p.desc.joint_spring, p.desc.joint_damping) = spring;
            p.desc.joint_flags = flags;
        }
    }

    /// Once a game frame, where its scripts run: the Activates asked for, and a look round for
    /// what's near.
    pub fn frame(now: f64) {
        let player = engine::player();
        if player == 0 || deref(player + REFR_NODE) == 0 {
            return;
        }
        with_state(|s| {
            for (r, t) in s.things.iter_mut().filter(|(_, t)| t.pending) {
                t.pending = false;
                t.tried = now;
                if alive(*r, t) {
                    fire(t);
                }
                s.republish = true;
            }
            if now >= s.next_scan {
                s.next_scan = now + SCAN_EVERY;
                scan(s);
            }
            if std::mem::take(&mut s.republish) {
                publish(s);
            }
        });
    }

    /// The game's Activate on a thing a hand worked, and what it did: a door opened (its animation
    /// stopped where the hand has it, its sound) or closed (latched, its sound); an activator's
    /// script run (its animation stopped too: the hand put it there).
    fn fire(t: &mut Thing) {
        let before = (t.kind != Kind::Activator).then(|| open_state(t.refr));
        let ok = activate(t.refr);
        let mgr = manager(t.root);
        let (out, back) = (&t.anims.0, t.anims.1.as_ref());
        let (seq_out, seq_back) = (sequence(mgr, &out.name), back.map_or(0, |b| sequence(mgr, &b.name)));
        match t.kind {
            Kind::Door => {
                let state = open_state(t.refr);
                let opened = matches!(state, 1 | 2);
                tracing::info!("world: door {:08X} activated ({ok}): open state {before:?} -> {state}", t.form_id);
                if opened != t.open {
                    // The hand is the animation: the game's is stopped where it began.
                    let states = |q: usize| mem::at::<u32>(q + SEQUENCE_STATE);
                    let before = (states(seq_out), states(seq_back));
                    stop(seq_out);
                    stop(seq_back);
                    tracing::info!("world: its animations ({mgr:#x}: {seq_out:#x}, {seq_back:#x}) were {before:?}, now {:?}", (states(seq_out), states(seq_back)));
                    // Its animation's sound where it has one, else the door's own.
                    let sounds = if opened { Some(&out.sounds) } else { back.map(|b| &b.sounds) };
                    match sounds.and_then(|s| s.first()) {
                        Some((_, sound)) => play_sound(t.refr, sound),
                        None => play_door_sound(t.refr, if opened { 0 } else { 1 }),
                    }
                    t.open = opened;
                    t.owned = true;
                    // The latch let go, or caught: felt in the hands on it.
                    latch_felt(t);
                }
                t.locked = locked(t.refr);
            }
            // (Its own animation moves it: the runtime follows.)
            Kind::HeavyDoor => {
                tracing::info!("world: heavy door {:08X} activated ({ok}): open state {before:?} -> {}", t.form_id, open_state(t.refr));
                t.open = matches!(open_state(t.refr), 1 | 2);
                t.owned = false;
                t.synced = false;
            }
            Kind::Activator => {
                let ran = [seq_out, seq_back].iter().filter(|&&q| animating(q)).count();
                tracing::info!("world: activator {:08X} activated ({ok}), {ran} animation(s) started", t.form_id);
                let seq = if animating(seq_back) && t.open { seq_back } else { seq_out };
                let sounds = if seq == seq_back { back.map(|b| &b.sounds) } else { Some(&out.sounds) };
                if animating(seq) {
                    if let Some((_, sound)) = sounds.and_then(|s| s.first()) {
                        play_sound(t.refr, sound);
                    }
                }
                stop(seq_out);
                stop(seq_back);
                t.owned = true;
                // A two-way lever stays at the end it's at: its far end is "on".
                if !out.returns {
                    t.open = !t.open;
                }
            }
        }
        shape_descs(t);
    }

    /// A door's latch letting go or catching, in the hands that have it: a click.
    fn latch_felt(t: &Thing) {
        let hands = t.parts.iter().fold(0, |h, p| h | p.hands);
        for hand in 0..2u32 {
            if hands & (1 << hand) != 0 {
                crate::link::with_adapter(|a| a.haptic(hand, 0.03, 140.0, 0.8));
            }
        }
    }

    fn scan(s: &mut State) {
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p))) else { return };
        let near = nearby(me, REACH);
        let mut keep = Vec::new();
        for (refr, kind, path, _) in near.into_iter().take(MAX_THINGS) {
            keep.push(refr);
            let root = deref(refr + REFR_NODE);
            if s.things.get(&refr).is_some_and(|t| t.root == root) || s.none.get(&refr) == Some(&root) {
                continue;
            }
            let (out, back) = if kind == DOOR { ("Open", "Close") } else { ("Forward", "Backward") };
            let Some(anims) = model(&path, out, back) else { continue };
            let Some(anims) = anims else {
                s.none.insert(refr, root);
                continue;
            };
            match make(s, refr, kind, anims) {
                Some(t) => {
                    s.things.insert(refr, t);
                    s.republish = true;
                }
                None => {
                    s.none.insert(refr, root);
                }
            }
        }
        let before = s.things.len();
        s.things.retain(|r, t| keep.contains(r) || t.parts.iter().any(|p| p.held) || t.owned && t.parts.iter().any(|p| p.vel.abs() > 0.01));
        s.things.retain(|r, t| alive(*r, t));
        if s.things.len() != before {
            s.republish = true;
        }
    }

    fn publish(s: &mut State) {
        let mut things: Vec<&Thing> = s.things.values().collect();
        things.sort_by_key(|t| t.form_id);
        let descs: Vec<ObjectDesc> = things.iter().flat_map(|t| t.parts.iter().map(|p| p.desc)).collect();
        if descs.len() == s.sent.len() && descs.iter().zip(&s.sent).all(|(a, b)| bytemuck::bytes_of(a) == bytemuck::bytes_of(b)) {
            return;
        }
        let mut hulls: Vec<GameHull> = Vec::new();
        for p in things.iter().flat_map(|t| &t.parts) {
            let mut h: GameHull = bytemuck::Zeroable::zeroed();
            h.id = p.id;
            h.count = p.hull.len().min(MAX_HULL_POINTS) as u32;
            for (k, q) in p.hull.iter().take(MAX_HULL_POINTS).enumerate() {
                h.points[k] = [q.x, q.y, q.z, 0.0];
            }
            hulls.push(h);
        }
        let ok = crate::link::with_adapter(|a| crate::objects::publish_hulls(crate::objects::Owner::World, &hulls, a) && crate::objects::publish(crate::objects::Owner::World, &descs, a)).unwrap_or(false);
        if ok {
            tracing::info!("world: {} part(s) of {} thing(s) offered", descs.len(), things.len());
            s.sent = descs;
        }
    }

    /// Before the eyes are drawn: where the runtime has each part (its nodes posed there, its
    /// Havok body with them), or, while the game moves it, the game's pose for the runtime to follow;
    /// and the game's Activate asked for where a hand has worked one far enough.
    pub fn place(now: f64) {
        with_state(|s| {
            // (A thing whose reference or 3D has gone since the last look round, or everything
            // while the game loads: never touched again.)
            let before = s.things.len();
            if !crate::menus::in_world() {
                s.things.clear();
            }
            s.things.retain(|r, t| alive(*r, t));
            if s.things.len() != before {
                s.republish = true;
            }
            if s.things.is_empty() {
                return;
            }
            let states = s.states.get_or_insert_with(bytemuck::zeroed_box);
            if !crate::link::with_adapter(|a| a.read_object_states(states)).unwrap_or(false) {
                return;
            }
            let n = (states.count as usize).min(MAX_OBJECTS);
            let by_id: HashMap<u32, &ObjectState> = states.states[..n].iter().map(|st| (st.id, st)).collect();
            let mut poses: Vec<GamePose> = Vec::new();
            for t in s.things.values_mut() {
                for p in &mut t.parts {
                    if let Some(st) = by_id.get(&p.id) {
                        p.at = st.slide;
                        p.vel = if p.slide { Vec3::from(st.lin_vel).length() * Vec3::from(st.lin_vel).dot(p.base * Vec3::Z).signum() } else { Vec3::from(st.ang_vel).dot(p.base * Vec3::Z) };
                        p.held = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0;
                        p.hands = u32::from(st.flags & OS_HELD_LEFT != 0) | u32::from(st.flags & OS_HELD_RIGHT != 0) << 1;
                    }
                }
                for p in t.parts.iter_mut().filter(|p| (p.at - p.logged).abs() > 0.1 * p.far.max(0.05)) {
                    p.logged = p.at;
                    tracing::info!("world: {:08X} part {} at {:.3} of {:.3} ({:.2}/s), held {}, the runtime's {}", t.form_id, p.id, p.at, p.far, p.vel, p.held, t.owned);
                }
                let held = t.parts.iter().any(|p| p.held);
                // The game moving it itself (its own animation playing): the runtime follows,
                // unless a hand takes it, which stops the game's animation where it is.
                let mgr = manager(t.root);
                let seqs = [sequence(mgr, &t.anims.0.name), t.anims.1.as_ref().map_or(0, |a| sequence(mgr, &a.name))];
                let game_moving = seqs.iter().any(|&q| animating(q));
                if game_moving != t.game_moving {
                    t.game_moving = game_moving;
                    tracing::info!("world: {:08X}: the game {} it{}", t.form_id, if game_moving { "moves" } else { "has stopped moving" }, if held { " (a hand has it)" } else { "" });
                }
                if game_moving && held && t.kind != Kind::HeavyDoor {
                    seqs.iter().for_each(|&q| stop(q));
                    t.owned = true;
                    if t.kind == Kind::Door {
                        t.open = matches!(open_state(t.refr), 1 | 2);
                        shape_descs(t);
                        s.republish = true;
                    }
                } else if game_moving {
                    t.owned = false;
                    t.synced = false;
                    if t.kind != Kind::Activator {
                        let open = matches!(open_state(t.refr), 1 | 2);
                        if open != t.open {
                            t.open = open;
                            shape_descs(t);
                            s.republish = true;
                        }
                    }
                }
                // Following the game: where its node is now along its travel.
                if !t.owned {
                    let mut all_there = true;
                    for p in &t.parts {
                        let tr = &t.anims.0.tracks[t.anims.0.joints[p.joint].track];
                        let Some((r, pos)) = local_of(p.node) else { continue };
                        let (r0, t0) = tr.poses[0];
                        let game_at = match t.anims.0.joints[p.joint].motion {
                            Motion::Turn { axis, .. } => angle_about((r * r0.inverse()).normalize(), axis),
                            Motion::Slide { dir } => (pos - t0).dot(dir) / p.units,
                        };
                        let axis = p.base * Vec3::Z;
                        let (gp, gq) = if p.slide { (p.centre + axis * game_at, p.base) } else { (p.centre, p.base * Quat::from_rotation_z(game_at)) };
                        if (game_at - p.at).abs() > 0.005 || game_moving {
                            all_there = false;
                            poses.push(GamePose { id: p.id, pose: Pose { pos: gp.to_array(), valid: 1, rot: gq.to_array() }, ..Default::default() });
                        }
                    }
                    t.synced = all_there && !game_moving;
                    // A hand on it (or pressing it away from where the game has it, once it's
                    // caught up): the runtime's from now on.
                    if t.synced && t.parts.iter().any(|p| p.held || p.vel.abs() > 0.02) {
                        t.owned = true;
                    }
                }
                // The runtime's: its nodes where the runtime has it, its Havok body with them.
                if t.owned {
                    for p in &t.parts {
                        let a = &t.anims.0;
                        let f = a.sample_at(p.joint, p.at * p.units);
                        for &(k, node) in &p.tracks {
                            let (r, pos) = a.pose(k, f);
                            set_local(node, r, pos);
                        }
                    }
                    update(t.root);
                    for p in &t.parts {
                        reset_collision(p.body_node);
                    }
                }
                // Worked far enough: the game's Activate (in its next frame, where scripts run).
                let ready = !t.pending && now - t.tried > RETRY;
                let most = t.parts.iter().map(|p| p.at / p.far.max(1e-4)).fold(0.0f32, f32::max);
                match t.kind {
                    Kind::Door | Kind::HeavyDoor if !t.open => {
                        let tug = t.parts.iter().any(|p| p.at >= LATCH_PLAY.min(p.far) * TUG);
                        if tug && ready && t.armed {
                            t.pending = true;
                            t.armed = false;
                        }
                        if !tug {
                            t.armed = true;
                        }
                    }
                    Kind::Door if t.owned && t.parts.iter().all(|p| p.at <= SHUT && p.vel <= 0.05) && ready => t.pending = true,
                    Kind::Activator if t.owned || t.synced => {
                        let back = t.anims.0.returns || !t.open;
                        // (Out to its far end, or a two-way lever that's on, back to its start.)
                        let fire = if back { most >= FIRE_AT } else { most <= 1.0 - FIRE_AT };
                        if fire && ready && t.armed {
                            t.pending = true;
                            t.armed = false;
                        }
                        if if back { most <= REARM } else { most >= 1.0 - REARM } {
                            t.armed = true;
                        }
                    }
                    _ => {}
                }
            }
            crate::link::with_adapter(|a| crate::objects::publish_poses(crate::objects::Owner::World, &poses, a));
        });
    }
}
