//! Riding Oblivion's horses: the framework's riding (`rustvr_protocol::ride`, the runtime's
//! `riding`) on the game's own horse. The game keeps its horse: its model and animations, its
//! speeds and turning, whose it is and what stealing it costs, getting on and off and the sounds
//! of it. The adapter tells the runtime where things are, and gives the game the rider's controls
//! as its own. Engine facts are in `docs/research/riding.md`.
//!
//! * **Getting on and off** ([`before_update`]): the runtime's hand on a saddle, and its getting
//!   off, are the player's own activation of the horse (`TESObjectREFR::Activate`, 0x4DD260,
//!   thiscall on the horse with the player activating: the call the game's Activate control
//!   makes). On foot it gets on (the game's check of whose horse it is, "Do you want to steal this
//!   horse?" and the crime, the mount animation and its sounds); on the horse it gets off.
//! * **The controls** ([`before_update`]): a mounted player's input reaches the horse through the
//!   player's update (0x671620). The first joystick's Y axis is its pace forward or back (|axis| ·
//!   0.02, at most 1, as the horse's movement flags 1 and 2 with the analog speed), its X axis
//!   turns it (the horse's heading, ramped up at `iHorseTurnDegreesRampUpPerSecond`, 80, to
//!   `iHorseTurnDegreesPerSecond`, 45, times |X| · 0.02 up to 1.5: as the keys do), Run gallops
//!   (0x200) and Jump jumps. The rider's gait is the Y axis (and Run for the gallop), the steering
//!   the X. The game reads the stick as a direction (0x67212C): only while |Y| is at least |X| is
//!   it forward or back; past that it's a slide alone, which on a horse is a turn on the spot. So
//!   while the horse goes, Y is kept at least |X| (its pace is capped at 1, so a full pace stays as
//!   it is; a slow one turning hard goes a little faster), and it turns as hard as the game turns
//!   a horse without stopping. Either axis past 98 is Run too.
//! * **The rider** ([`publish`]): the horse's own animations carry the rider's `Bip01` tracks, so
//!   the game animates the player's own skeleton in the saddle with every gait: the pelvis between
//!   the hip joints, the knees and feet astride, the hands on the reins. The runtime's body sits
//!   in that seat with its legs to those knees and feet, and its hands find the reins where those
//!   hands are.
//! * **The saddles near the player** ([`publish`]): every horse the game processes near the
//!   player, its saddle (measured on the game's own model, `oblivion_assets::horse`) on its live
//!   spine, for a hand to take hold of.
//! * **The view** ([`carry`]): the horse's turn carries the view round with it, and while riding
//!   the view's anchor is the rider's eye held steady on the horse: over the saddle's seat, at the
//!   height the game's own first-person camera seats a rider (not the game's camera itself: after
//!   getting off and on again it stays at a standing player's height on the horse's back, and
//!   behind the horse as it moves). The gait's bob is the seat's, which the runtime's head bob
//!   setting brings back.
//!
//! State: the player's process says sitting (`GetSitSleepState`, vtable +0x36C: 3 getting on, 4
//! seated, 5 getting off), and the player's `horseOrRider` (+0xD4) is the horse.

use crate::{camera, engine, mem};
use glam::{Quat, Vec3};
use rustvr_protocol::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// `Actor::horseOrRider`: a rider's horse, a horse's rider.
const HORSE_OR_RIDER: usize = 0xD4;
/// `Actor::process` and `BaseProcess::GetSitSleepState` (vtable +0x36C), its states on a horse.
const PROCESS: usize = 0x58;
const VT_SIT_SLEEP: usize = 0x36C;
const SEATED: u32 = 4;
const GETTING_OFF: u32 = 5;
/// `BaseProcess::GetMovementFlags` (vtable +0x2C0): forward, back, running.
const VT_MOVEMENT_FLAGS: usize = 0x2C0;
const MOVE_FORWARD: u32 = 0x1;
const MOVE_BACK: u32 = 0x2;
const MOVE_RUN: u32 = 0x200;
/// `TESObjectREFR::Activate(this, activator, 0, 0, 1)`.
const ACTIVATE: usize = 0x004D_D260;
/// `TESObjectREFR`: its base form, its 3D, its form id; `TESCreature::type` (4: a horse).
const BASE_FORM: usize = 0x1C;
const NI_NODE: usize = 0x3C;
const FORM_ID: usize = 0x0C;
const CREATURE_TYPE: usize = 0x104;
const HORSE: u8 = 4;
/// The game's `Creature` vtable (horses are creatures).
const CREATURE_VTABLE: usize = 0x00A7_10F4;
/// `ExtraOwnership` (extra data 0x27): its owner at +0x0C; the player's base form.
const EXTRA_OWNERSHIP: u8 = 0x27;
const PLAYER_BASE: u32 = 0x7;
/// `NiNode` children (`NiTArray`: data at +0xB0, count at +0xB6), `NiObjectNET` name.
const NODE_CHILDREN: usize = 0xB0;
const NODE_COUNT: usize = 0xB6;
const NODE_NAME: usize = 0x08;
/// The horse's neck bone at its base (the withers), and where along from the pommel to it the reins
/// are held, how far apart (metres), and a foot's length forward of the ankle (metres).
const NECK: &str = "Bip02 Neck1";
const REINS_ALONG: f32 = 0.5;
const REINS_APART: f32 = 0.09;
const FOOT: f32 = 0.15;
/// Without the saddle's measure, the seat is this far over the horse's feet (metres).
const SEAT_FALLBACK: f32 = 1.6;
/// Horses further than this aren't offered (metres).
const NEAR: f32 = 6.0;
/// A hand takes hold of the saddle this far round its line past the seat (metres): a hand's reach.
const SADDLE_REACH: f32 = 0.07;
/// How the rider's eye is held steady on the horse: its place on it eased over this (seconds), so
/// the gait's bob goes and the seat's place stays.
const STEADY: f32 = 0.35;
/// The rider's eye over the saddle's seat (metres): where the game's own first-person camera is on
/// a horse when it seats the rider right (the first time in a session it does; measured on both
/// rides on the headset: 0.85 m over the seat, upright above it). In first person nothing in the
/// game poses its rider (`docs/research/riding.md`), and once off and on again its camera stays at
/// a standing player's eye on the horse's back (1.9 m over the seat).
const EYE_OVER_SEAT: f32 = 0.85;
/// The horse's speed is eased over this (seconds).
const SPEED_EASE: f32 = 0.25;
/// The game's gaits forward: walk, gallop.
const GAITS: u32 = 2;
/// The joystick's axis at a pace of 1 (the game's |axis| · 0.02), and at the game's full turn
/// (|axis| · 0.02 = 1.5, as fast as the turn keys turn a horse). Both under 98, where the game
/// takes the stick as Run.
const PACE_AXIS: f32 = 50.0;
const TURN_AXIS: f32 = 75.0;

/// Where the player is with a horse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Afoot,
    GettingOn,
    Riding,
    GettingOff,
}

/// The horse the player rides, gets on or gets off (or 0), and where they are with it.
pub fn horse() -> (usize, State) {
    let p = engine::player();
    let horse = if p != 0 { mem::deref(p + HORSE_OR_RIDER) } else { 0 };
    if horse == 0 || mem::deref(horse) != CREATURE_VTABLE {
        return (0, State::Afoot);
    }
    // (The horse is set as the mount begins, before the process says so (3): getting on.)
    let state = match sit_state(p) {
        SEATED => State::Riding,
        GETTING_OFF => State::GettingOff,
        _ => State::GettingOn,
    };
    (horse, state)
}

/// The player is on a horse (or getting on or off it): the game moves them, not the stick.
pub fn mounted() -> bool {
    horse().0 != 0
}

#[cfg(target_arch = "x86")]
fn sit_state(actor: usize) -> u32 {
    let process = mem::deref(actor + PROCESS);
    let f = mem::deref(mem::deref(process) + VT_SIT_SLEEP);
    if process == 0 || !(0x0040_1000..0x00A3_0000).contains(&f) {
        return 0;
    }
    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> u32>(f)(process) }
}

#[cfg(not(target_arch = "x86"))]
fn sit_state(_actor: usize) -> u32 {
    0
}

#[cfg(target_arch = "x86")]
fn movement_flags(actor: usize) -> u32 {
    let process = mem::deref(actor + PROCESS);
    let f = mem::deref(mem::deref(process) + VT_MOVEMENT_FLAGS);
    if process == 0 || !(0x0040_1000..0x00A3_0000).contains(&f) {
        return 0;
    }
    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> u32>(f)(process) }
}

#[cfg(not(target_arch = "x86"))]
fn movement_flags(_actor: usize) -> u32 {
    0
}

/// The player activates `target` as the game's Activate control does.
#[cfg(target_arch = "x86")]
fn activate(target: usize) -> bool {
    let p = engine::player();
    if p == 0 || target == 0 {
        return false;
    }
    type F = unsafe extern "thiscall" fn(usize, usize, u32, u32, u32) -> u8;
    unsafe { std::mem::transmute::<usize, F>(ACTIVATE)(target, p, 0, 0, 1) != 0 }
}

#[cfg(not(target_arch = "x86"))]
fn activate(_target: usize) -> bool {
    false
}

/// The nodes named `names` under `root` (0 where there's none): the skeleton's bones (`Bip0...`).
fn nodes<const N: usize>(root: usize, names: [&str; N]) -> [usize; N] {
    let found = nodes_named(root, &names);
    std::array::from_fn(|i| found[i])
}

/// As [`nodes`], for names known only as the game runs (the neck's bones from `horse.toml`).
fn nodes_named(root: usize, names: &[&str]) -> Vec<usize> {
    let mut out = vec![0usize; names.len()];
    let mut stack = vec![(root, 0u32)];
    let mut seen = 0;
    while let Some((node, depth)) = stack.pop() {
        seen += 1;
        if seen > 800 || depth > 48 || node == 0 {
            continue;
        }
        let name = crate::combat::read_str(mem::deref(node + NODE_NAME));
        if let Some(i) = names.iter().position(|n| *n == name) {
            out[i] = node;
        }
        // (Only nodes are gone into: the skeleton's, and the few above it.)
        if depth > 2 && !name.starts_with("Bip0") {
            continue;
        }
        let data = mem::deref(node + NODE_CHILDREN);
        let n = mem::at::<u16>(node + NODE_COUNT).unwrap_or(0).min(64) as usize;
        for i in 0..n {
            let child = if data != 0 { mem::deref(data + i * 4) } else { 0 };
            if child != 0 {
                stack.push((child, depth + 1));
            }
        }
    }
    out
}

/// A node's world place, in the protocol's world (metres).
fn node_pos(node: usize) -> Option<Vec3> {
    let p: [f32; 3] = mem::at(node + engine::av::WORLD_TRANSLATE)?;
    let v = camera::basis().point_from_game(Vec3::from(p));
    v.is_finite().then_some(v)
}

/// A point in a node's frame (game units), in the protocol's world (metres).
fn node_point(node: usize, local: [f32; 3]) -> Option<Vec3> {
    let r: [f32; 9] = mem::at(node + engine::av::WORLD_ROTATE)?;
    let t: [f32; 3] = mem::at(node + engine::av::WORLD_TRANSLATE)?;
    let s: f32 = mem::at(node + engine::av::WORLD_SCALE)?;
    let l = Vec3::from(local) * s;
    // (Row-major.)
    let g = Vec3::new(r[0] * l.x + r[1] * l.y + r[2] * l.z, r[3] * l.x + r[4] * l.y + r[5] * l.z, r[6] * l.x + r[7] * l.y + r[8] * l.z) + Vec3::from(t);
    let v = camera::basis().point_from_game(g);
    v.is_finite().then_some(v)
}

/// A reference's heading (radians, 0 north turning east: the protocol's yaw from +Z towards +X).
fn heading(r: usize) -> Option<f32> {
    mem::at::<f32>(r + engine::refr::ROT + 8).filter(|h| h.is_finite())
}

fn position(r: usize) -> Option<Vec3> {
    mem::at::<[f32; 3]>(r + engine::refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)))
}

/// Whose a horse is: the player's own, someone else's (getting on is stealing), nobody's.
fn owner_flags(horse: usize) -> u32 {
    let mut e = mem::deref(horse + 0x44 + 4);
    for _ in 0..64 {
        if e == 0 {
            break;
        }
        if mem::at::<u8>(e + 4) == Some(EXTRA_OWNERSHIP) {
            let owner = mem::deref(e + 0x0C);
            return if owner != 0 && mem::at::<u32>(owner + FORM_ID) == Some(PLAYER_BASE) { MOUNT_OWNED } else if owner != 0 { MOUNT_CRIME } else { 0 };
        }
        e = mem::deref(e + 8);
    }
    0
}

/// A horse the game processes near the player, free to get on (alive, nobody on it).
fn is_free_horse(actor: usize) -> bool {
    if actor == 0 || mem::deref(actor) != CREATURE_VTABLE {
        return false;
    }
    let base = mem::deref(actor + BASE_FORM);
    base != 0 && mem::at::<u8>(base + CREATURE_TYPE) == Some(HORSE) && mem::deref(actor + HORSE_OR_RIDER) == 0
}

/// The saddle's measure (`local_assets/oblivion/horse.toml`), read once.
fn saddle() -> Option<&'static oblivion_assets::horse::Saddle> {
    static SADDLE: std::sync::OnceLock<Option<oblivion_assets::horse::Saddle>> = std::sync::OnceLock::new();
    SADDLE
        .get_or_init(|| {
            let s = oblivion_assets::horse::load(&rustvr_asset::default_root().join("oblivion"));
            match &s {
                Some(s) => tracing::info!("riding: the horse's saddle on {}: pommel {:?}, seat {:?}, cantle {:?}", s.bone, s.pommel, s.seat, s.cantle),
                None => tracing::warn!("riding: no measure of the horse's saddle (oblivion-extract export meshes/creatures/horse): getting on by the saddle is off, Activate still gets on"),
            }
            s
        })
        .as_ref()
}

/// A horse's saddle (pommel, seat, cantle) in the protocol's world.
fn saddle_points(horse: usize) -> Option<[Vec3; 3]> {
    let s = saddle()?;
    let [bone] = nodes(mem::deref(horse + NI_NODE), [s.bone.as_str()]);
    if bone == 0 {
        return None;
    }
    Some([node_point(bone, s.pommel)?, node_point(bone, s.seat)?, node_point(bone, s.cantle)?])
}

/// The view, carried with the horse.
pub struct Carry {
    /// The rider's eye: seated, over the saddle's seat as the horse's gait moves it (getting on or
    /// off, the game's camera); and held steady on the horse (the anchor without the gait's bob).
    pub eye: Vec3,
    pub base_eye: Vec3,
    /// The horse's heading: what the view faces when it's recentred.
    pub look_yaw: f32,
    /// How far the horse turned since the last game frame (radians).
    pub turn: f32,
    /// Just seated: the view is recentred on the horse (facing its way, the head on its eye).
    pub recenter: bool,
}

struct Steady {
    horse: usize,
    heading: f32,
    /// The rider's eye in the horse's frame, eased.
    eye: Vec3,
    state: State,
}

static STEADY_STATE: Mutex<Option<Steady>> = Mutex::new(None);

/// While on a horse: its turn since the last game frame, and the rider's eye on it and held steady
/// on it (`eye` is the game's camera, in the protocol's world). None on foot.
pub fn carry(eye: Vec3, dt: f32) -> Option<Carry> {
    let (horse, state) = horse();
    let mut s = STEADY_STATE.lock().unwrap();
    let (Some(h), Some(at)) = (heading(horse).filter(|_| horse != 0), position(horse)) else {
        *s = None;
        return None;
    };
    // Seated, the eye is over the saddle's seat; getting on or off, the game's camera (the view is
    // faded out meanwhile).
    let seated = (state == State::Riding).then(|| seat(horse)).flatten();
    let camera = eye;
    let eye = seated.map_or(camera, |seat| seat + Vec3::Y * EYE_OVER_SEAT);
    let rel = Quat::from_rotation_y(-h) * (eye - at);
    let was = s.take().filter(|w| w.horse == horse);
    let turn = was.as_ref().map_or(0.0, |w| wrap(h - w.heading));
    // Seated, the eye's place on the horse eases (the gait's bob goes); getting on or off, it's the
    // game's as it is (the view is faded out meanwhile).
    let steady = match &was {
        Some(w) if state == State::Riding && w.state == State::Riding => w.eye + (rel - w.eye) * (1.0 - (-dt.max(0.0) / STEADY).exp()),
        _ => rel,
    };
    let recenter = state == State::Riding && was.as_ref().is_some_and(|w| w.state == State::GettingOn);
    if was.as_ref().is_none_or(|w| w.state != state) {
        tracing::info!(
            "riding: {:?} (horse {:#x} at heading {:.1} deg){}",
            state,
            mem::at::<u32>(horse + FORM_ID).unwrap_or(0),
            h.to_degrees(),
            match seated {
                Some(seat) => format!("; the eye over the seat, the game's camera {:.2} m off it", (seat + Vec3::Y * EYE_OVER_SEAT).distance(camera)),
                None => String::new(),
            }
        );
    }
    *s = Some(Steady { horse, heading: h, eye: steady, state });
    Some(Carry { eye, base_eye: at + Quat::from_rotation_y(h) * steady, look_yaw: h, turn, recenter })
}

/// The saddle's seat on a horse's live spine (the protocol's world), when it's measured.
fn seat(horse: usize) -> Option<Vec3> {
    let s = saddle()?;
    let [spine] = nodes(mem::deref(horse + NI_NODE), [s.bone.as_str()]);
    (spine != 0).then(|| node_point(spine, s.seat)).flatten()
}

/// The horses offered last (by id), to get on when the runtime asks.
static OFFERED: Mutex<Option<HashMap<u32, usize>>> = Mutex::new(None);
/// How the horse moved since the last game frame, for the runtime's comfort.
static LAST_PLACE: Mutex<Option<(usize, Vec3, f64, f32)>> = Mutex::new(None);

/// Every game frame in VR (with this frame's mapping): the horse ridden and the horses near the
/// player, for the runtime.
pub fn publish(view: &rustvr_adapter::View, now: f64) {
    let mut ride = Ride::default();
    let p = engine::player();
    if p == 0 {
        return;
    }
    let me = position(p);
    let t = |v: Vec3| view.to_tracking(v);
    let (horse, state) = horse();
    let mut offered = HashMap::new();
    if horse == 0 {
        for actor in crate::combat::high_actors() {
            if !is_free_horse(actor) {
                continue;
            }
            let (Some(at), Some(me)) = (position(actor), me) else { continue };
            if at.distance(me) > NEAR {
                continue;
            }
            let Some([pommel, seat, cantle]) = saddle_points(actor) else { continue };
            let id = mem::at::<u32>(actor + FORM_ID).unwrap_or(0);
            let (a, b) = (t(pommel), t(cantle));
            let radius = segment_distance(t(seat), a, b) + SADDLE_REACH;
            ride.push_mount(Mount { id, flags: owner_flags(actor), _pad: [0; 2], saddle_a: a.to_array(), radius, saddle_b: b.to_array(), _p0: 0.0 });
            offered.insert(id, actor);
        }
    } else {
        let r = &mut ride.riding;
        r.id = mem::at::<u32>(horse + FORM_ID).unwrap_or(0);
        r.flags = match state {
            State::GettingOn => RIDE_GETTING_ON,
            State::Riding => RIDE_MOUNTED,
            State::GettingOff => RIDE_GETTING_OFF,
            State::Afoot => 0,
        };
        r.gaits = GAITS;
        let flags = movement_flags(horse);
        r.gait = if flags & MOVE_BACK != 0 {
            -1
        } else if flags & MOVE_FORWARD != 0 {
            if flags & MOVE_RUN != 0 { 2 } else { 1 }
        } else {
            0
        };
        // Its speed over the ground, eased (the game moves it a little unevenly from frame to
        // frame: sometimes not at all, then twice as far).
        let at = position(horse);
        let mut last = LAST_PLACE.lock().unwrap();
        let mut speed = 0.0;
        if let (Some(at), Some((h, was, then, eased))) = (at, *last)
            && h == horse
        {
            speed = eased;
            if now > then {
                let step = Vec3::new(at.x - was.x, 0.0, at.z - was.z).length() / (now - then) as f32;
                speed += (step - speed) * (1.0 - (-((now - then) as f32) / SPEED_EASE).exp());
            }
        }
        r.speed = speed;
        *last = at.map(|a| (horse, a, now, speed));
        drop(last);
        let h = heading(horse).unwrap_or(0.0);
        // The mount's frame in tracking space: its heading, with the view's turn taken off.
        let turn = Quat::from_rotation_y(h - view.body_yaw);
        let four = |v: Vec3| [v.x, v.y, v.z, 0.0];
        // The rider in the saddle: the game poses its rider only in third person (the rider's tracks
        // in the horse's own animations; in first person its skeleton is left standing), so the seat
        // and the legs are the saddle's and the horse's own body's (`oblivion_assets::horse`), on
        // its spine as its animation moves it with every gait.
        let horse_root = mem::deref(horse + NI_NODE);
        if let (Some(s), [spine, neck]) = (saddle(), nodes(horse_root, [saddle().map_or("", |s| s.bone.as_str()), NECK])) {
            let mirror = |v: [f32; 3]| [v[0], v[1], -v[2]];
            let fwd = Quat::from_rotation_y(h) * Vec3::Z;
            let at = |local: [f32; 3]| if spine != 0 { node_point(spine, local) } else { None };
            if let (Some(seat), Some(pommel)) = (at(s.seat), at(s.pommel)) {
                r.seat = Pose { pos: t(seat).to_array(), valid: 1, rot: turn.to_array() };
                r.pommel = four(t(pommel));
                if let (Some(kl), Some(kr), Some(al), Some(ar)) = (at(s.knee), at(mirror(s.knee)), at(s.ankle), at(mirror(s.ankle))) {
                    r.knees = [four(t(kl)), four(t(kr))];
                    r.ankles = [four(t(al)), four(t(ar))];
                    r.toes = [four(t(al + fwd * FOOT)), four(t(ar + fwd * FOOT))];
                    r.flags |= RIDE_LEGS;
                }
                // The reins run from the bit along the neck to the rider's hands: held over the
                // withers, between the pommel and the neck's base, either side of the mane.
                let side = Quat::from_rotation_y(h) * Vec3::X;
                if let Some(neck) = (neck != 0).then(|| node_pos(neck)).flatten() {
                    let hold = pommel.lerp(neck, REINS_ALONG);
                    r.reins = [four(t(hold - side * REINS_APART)), four(t(hold + side * REINS_APART))];
                    r.flags |= RIDE_REINS;
                }
                // The bit's rings, on the head as it moves (Oblivion's horses have no reins of their
                // own: the runtime draws them from here, in the tack's own leather).
                let [head] = nodes(horse_root, [s.head_bone.as_str()]);
                if let (Some(a), Some(b)) = ((head != 0).then(|| node_point(head, s.bit)).flatten(), (head != 0).then(|| node_point(head, mirror(s.bit))).flatten()) {
                    let (left, right) = if a.dot(side) < b.dot(side) { (a, b) } else { (b, a) };
                    r.bits = [four(t(left)), four(t(right))];
                    r.reins_color = [s.tack[0], s.tack[1], s.tack[2], 1.0];
                    r.flags |= RIDE_BITS;
                    if !s.tack_model.is_empty() {
                        r.set_reins_leather(&format!("oblivion/{}", s.tack_model));
                        r.reins_uv = s.strap_uv;
                    }
                }
                // The neck and head, which the reins lie on: each cross-section on its bone as the
                // horse's animation has it.
                let mut bones: Vec<&str> = s.neck.iter().map(|n| n.bone.as_str()).collect();
                bones.dedup();
                let found: HashMap<&str, usize> = bones.iter().copied().zip(nodes_named(horse_root, &bones)).collect();
                let mut count = 0;
                for n in s.neck.iter().take(NECK_SECTIONS) {
                    let node = found.get(n.bone.as_str()).copied().unwrap_or(0);
                    let add = |a: [f32; 3], b: [f32; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
                    if let (true, Some(c), Some(sd), Some(up)) = (node != 0, node_point(node, n.centre), node_point(node, add(n.centre, n.side)), node_point(node, add(n.centre, n.up))) {
                        let (c, sd, up) = (t(c), t(sd), t(up));
                        r.neck[count] = NeckSection { centre: four(c), side: four(sd - c), up: four(up - c) };
                        count += 1;
                    }
                }
                if count >= 2 {
                    r.neck_count = count as u32;
                    r.flags |= RIDE_NECK;
                }
            }
        }
        if r.flags & RIDE_LEGS == 0
            && let Some(at) = at
        {
            // (No measure of the saddle: the seat over the horse.)
            r.seat = Pose { pos: t(at + Vec3::Y * SEAT_FALLBACK).to_array(), valid: 1, rot: turn.to_array() };
        }
        log_ride(r);
    }
    if !offered.is_empty() {
        static SAID: Mutex<f64> = Mutex::new(f64::MIN);
        let mut said = SAID.lock().unwrap();
        if now - *said > 5.0 {
            *said = now;
            for m in ride.mounts() {
                tracing::info!("riding: a horse near ({:#x}, flags {:#x}): its saddle from {:.2?} to {:.2?} (tracking), {:.2} m round", m.id, m.flags, m.saddle_a, m.saddle_b, m.radius);
            }
        }
    }
    *OFFERED.lock().unwrap() = Some(offered);
    static FRAME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    ride.frame = FRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    crate::link::with_adapter(|a| a.write_ride(&ride));
}

fn log_ride(r: &Riding) {
    static LAST: Mutex<(u32, i32, f64)> = Mutex::new((0, i32::MIN, 0.0));
    let mut last = LAST.lock().unwrap();
    let now = crate::link::now();
    if (r.flags, r.gait) != (last.0, last.1) || now - last.2 > 5.0 {
        tracing::info!(
            "riding: horse {:#x} flags {:#x} gait {} at {:.2} m/s; seat {:.2?}, knees {:.2?}, ankles {:.2?}, reins {:.2?}, pommel {:.2?}, bits {:.2?}; the neck in {} sections, the reins of {}",
            r.id,
            r.flags,
            r.gait,
            r.speed,
            r.seat.pos,
            r.knees.map(|k| [k[0], k[1], k[2]]),
            r.ankles.map(|k| [k[0], k[1], k[2]]),
            r.reins.map(|k| [k[0], k[1], k[2]]),
            [r.pommel[0], r.pommel[1], r.pommel[2]],
            r.bits.map(|k| [k[0], k[1], k[2]]),
            r.neck().len(),
            r.reins_leather().unwrap_or("no leather (ribbons)")
        );
        *last = (r.flags, r.gait, now);
    }
}

/// The rider's controls for the game this frame: the joystick's axes and whether Run is held.
pub struct Controls {
    pub stick: [i32; 2],
    pub run: bool,
}

/// What the runtime asked for last (the requests already acted on).
static ASKED: Mutex<Option<(u32, u32)>> = Mutex::new(None);

/// Before the game's frame: getting on or off as the runtime asks, and while on a horse its
/// controls (None on foot).
pub fn before_update() -> Option<Controls> {
    let rt = crate::link::with_adapter(|a| a.read_rt_ride()).flatten();
    let (horse, state) = horse();
    if let Some(rt) = rt {
        let mut asked = ASKED.lock().unwrap();
        // (The first word from a runtime is where its counts start, not a request.)
        let (mount_seq, dismount_seq) = asked.unwrap_or((rt.mount_seq, rt.dismount_seq));
        if rt.mount_seq != mount_seq && horse == 0 {
            let target = OFFERED.lock().unwrap().as_ref().and_then(|o| o.get(&rt.mount).copied());
            match target {
                Some(h) => {
                    let ok = activate(h);
                    tracing::info!("riding: the hand on the saddle of horse {:#x}: the game's own activation ({})", rt.mount, if ok { "getting on" } else { "it said no" });
                }
                None => tracing::warn!("riding: asked to get on horse {:#x}, which isn't near", rt.mount),
            }
        }
        if rt.dismount_seq != dismount_seq && horse != 0 && state == State::Riding {
            let ok = activate(horse);
            tracing::info!("riding: getting off: the game's own activation of the horse ({})", if ok { "done" } else { "it said no" });
        }
        *asked = Some((rt.mount_seq, rt.dismount_seq));
        if horse != 0 && state == State::Riding && rt.flags & RR_RIDING != 0 {
            return Some(controls(&rt));
        }
    }
    if horse != 0 {
        // On a horse with nothing from the rider (getting on or off): it stands.
        return Some(Controls { stick: [0, 0], run: false });
    }
    None
}

/// The rider's gait, pace and steering as the game's joystick and Run.
pub fn controls(rt: &RtRide) -> Controls {
    let pace = rt.pace.clamp(0.0, 1.0);
    let x = (rt.steer.clamp(-1.0, 1.0) * TURN_AXIS).round() as i32;
    // (Going, the forward axis is at least the turn's: the game stops a horse whose stick points
    // more to the side than ahead, and turns it on the spot.)
    let ahead = ((pace * PACE_AXIS).round() as i32).max(x.abs());
    let y = match rt.gait {
        g if g > 0 => -ahead,
        g if g < 0 => ahead,
        _ => 0,
    };
    let c = Controls { stick: [x, y], run: rt.gait >= GAITS as i32 };
    // What the game is given, when it changes (the turn by a fifth of the game's at least): with
    // the runtime's log of the hands, what the rider did and what the horse was told.
    static LAST: Mutex<Option<([i32; 2], bool)>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap();
    if last.is_none_or(|(s, r)| s[1] != c.stick[1] || r != c.run || (s[0] - c.stick[0]).abs() >= (TURN_AXIS / 5.0) as i32 || (s[0] == 0) != (c.stick[0] == 0)) {
        tracing::info!("riding: the game's controls: forward axis {}, turn axis {}{} (the rider's gait {}, pace {:.2}, steer {:+.2})", -c.stick[1], c.stick[0], if c.run { ", Run" } else { "" }, rt.gait, rt.pace, rt.steer);
        *last = Some((c.stick, c.run));
    }
    c
}

/// Tests (`PlayerHorse <base> [metres]`): a horse of the player's own beside them. The game places
/// it (`placeatme`); the next frames find it (the horse near the player that wasn't there) and make
/// it the player's (`SetOwnership`).
pub fn give_horse(base: &str, metres: f32) -> bool {
    let known: Vec<u32> = crate::combat::high_actors().into_iter().filter_map(|a| mem::at::<u32>(a + FORM_ID)).collect();
    let line = format!("player.placeatme {base} 1 {:.0} 0", metres * camera::UNITS_PER_METER);
    if !crate::console::run_script(&line) {
        return false;
    }
    *GIVING.lock().unwrap() = Some((known, crate::link::now() + 10.0));
    true
}

static GIVING: Mutex<Option<(Vec<u32>, f64)>> = Mutex::new(None);

/// Every game frame: a horse being given is made the player's once it's found.
pub fn frame(now: f64) {
    let mut giving = GIVING.lock().unwrap();
    let Some((known, until)) = giving.as_ref() else { return };
    if now > *until {
        tracing::warn!("riding: the horse placed for the player wasn't found");
        *giving = None;
        return;
    }
    let me = position(engine::player());
    let new = crate::combat::high_actors().into_iter().find(|&a| {
        let id = mem::at::<u32>(a + FORM_ID).unwrap_or(0);
        is_free_horse(a) && !known.contains(&id) && me.zip(position(a)).is_some_and(|(m, p)| m.distance(p) < 20.0)
    });
    if let Some(h) = new {
        let id = mem::at::<u32>(h + FORM_ID).unwrap_or(0);
        let ok = crate::console::run_script(&format!("{id:08x}.SetOwnership"));
        tracing::info!("riding: horse {id:08x} placed beside the player, now theirs ({})", if ok { "set" } else { "SetOwnership didn't compile" });
        *giving = None;
    }
}

/// An angle wrapped to (-pi, pi].
fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let r = a.rem_euclid(t);
    if r > std::f32::consts::PI { r - t } else { r }
}

/// The distance from `p` to the segment `a`..`b`.
fn segment_distance(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 1e-8 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(a + ab * t)
}
