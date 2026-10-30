//! The world's loose Havok bodies, for the hands: the parts of what's near the player that the
//! game's own physics swings and the player can't take (hanging chains and the chain dolls on them,
//! roots, ropes, a bucket or skulls on a rope, a trap's chain once it's sprung), each offered to the
//! runtime as one of the game's own props ([`OBJF_GAME_SIMULATED`]): the hands, the fingers and
//! what they hold go into it and push it, and the game's Havok moves it, so it swings off the hand
//! and back on its own constraints, as the game would swing it. Each push is given to its body as
//! the impulse its point lacks of the push (Havok's `applyPointImpulse`), as the game's own grab
//! pushes a body.
//!
//! Bodies the game moves itself aren't offered: fixed ones are the world's collision
//! ([`crate::collision`]), keyframed ones are moved by their animations (the doors and levers,
//! [`crate::world`]); nor what the player can take ([`crate::items`]), nor the creatures'.
//!
//! Engine facts (`Oblivion.exe` 1.2.0.416; Havok's own reflection, `docs/research/oblivion-havok.md`):
//! `hkEntity` world +8, filter info +0x30 (Bethesda's layer its low 7 bits), motion +0x50, `fixed`
//! +0x91, `isFixedOrKeyframed` +0x92. Its `hkMotion`: centre of mass (Havok's world) +0x60, inverse
//! mass +0xC0, linear and angular velocity +0xD0, +0xE0; virtual `getInertiaInvWorld` +0x3C and
//! `applyPointImpulse` +0x60 (thiscall: impulse, point); `hkEntity::activate` 0x8A6410 wakes it. A
//! node's collision object +0xA8, its `bhkRigidBody` +0x10, that one's `hkRigidBody` +8. Havok's
//! unit is 7 game units.

use glam::{Mat3, Quat, Vec3};
use rustvr_protocol::*;

/// Metres from the player's chest the loose bodies are offered within (a step and an arm), and at
/// most this many, nearest first.
pub const REACH: f32 = 3.0;
pub const MAX_BODIES: usize = 48;
/// Seconds between looks round.
pub const SCAN_EVERY: f64 = 0.5;

/// A body's object: the runtime's prop that is the game's body, pushed but not taken.
pub fn object_desc(id: u32, seq: u32, at: (Vec3, Quat), centre: Vec3, half: Vec3, mass: f32, hull: bool) -> ObjectDesc {
    let mut d = crate::items::object_desc(id, seq, at, centre, half, mass, hull);
    d.flags = OBJF_GAME_SIMULATED;
    d
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, av, refr};
    use crate::{camera, mem};
    use std::collections::HashMap;
    use std::sync::Mutex;

    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DELETED: u32 = 0x20;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const NPC: u8 = 0x23;
    const CREATURE: u8 = 0x24;
    const CELL_OBJECTS: usize = 0x48;

    const NODE_COLLISION: usize = 0xA8;
    const COLLISION_BODY: usize = 0x10;
    const HK_OBJ: usize = 0x8;
    const ENTITY_WORLD: usize = 0x8;
    const ENTITY_FILTER: usize = 0x30;
    const ENTITY_MOTION: usize = 0x50;
    const ENTITY_FIXED: usize = 0x91;
    const ENTITY_FIXED_OR_KEYFRAMED: usize = 0x92;
    const LAYER_MASK: u32 = 0x7F;
    const LAYER_NONCOLLIDABLE: u32 = 15;
    const MOTION_COM: usize = 0x60;
    const MOTION_MASS_INV: usize = 0xC0;
    const MOTION_LIN_VEL: usize = 0xD0;
    const MOTION_ANG_VEL: usize = 0xE0;
    const VT_INERTIA_INV_WORLD: usize = 0x3C;
    const VT_POINT_IMPULSE: usize = 0x60;
    const ACTIVATE: usize = 0x008A_6410;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A0_0000;
    /// Metres per Havok unit.
    const HK_METRES: f32 = crate::collision::HAVOK_SCALE / camera::UNITS_PER_METER;
    /// The first id of the loose bodies in the object table (the world's parts are at 0x7F00_0000).
    const ID_BASE: u32 = 0x7E00_0000;

    struct Body {
        refr: usize,
        node: usize,
        hk: usize,
        desc: ObjectDesc,
        hull: Vec<Vec3>,
    }

    #[derive(Default)]
    struct State {
        bodies: HashMap<u32, Body>,
        /// Each body's id by its Havok body (offered again with the same id while it's there).
        ids: HashMap<usize, u32>,
        next_id: u32,
        sent: Vec<u32>,
        next_scan: f64,
        poses: Option<Box<GamePoses>>,
        pushes: Option<Box<GamePushes>>,
        logged: HashMap<u32, f64>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn virt(obj: usize, slot: usize) -> Option<usize> {
        let f = mem::deref(mem::deref(obj) + slot);
        (obj != 0 && CODE.contains(&f)).then_some(f)
    }

    /// A node's Havok body, if the game's physics moves it: in a world, neither fixed nor keyframed,
    /// meeting something.
    fn loose_body(node: usize) -> Option<usize> {
        let co = mem::deref(node + NODE_COLLISION);
        let hk = if co != 0 { mem::deref(mem::deref(co + COLLISION_BODY) + HK_OBJ) } else { 0 };
        if hk == 0 || !mem::is_readable(hk as *const u8, 0xC4) || mem::deref(hk + ENTITY_WORLD) == 0 {
            return None;
        }
        let fixed = mem::at::<u8>(hk + ENTITY_FIXED)? != 0 || mem::at::<u8>(hk + ENTITY_FIXED_OR_KEYFRAMED)? != 0;
        let layer = mem::at::<u32>(hk + ENTITY_FILTER)? & LAYER_MASK;
        (!fixed && layer != LAYER_NONCOLLIDABLE && mem::deref(hk + ENTITY_MOTION) != 0).then_some(hk)
    }

    /// A reference whose loose bodies the hands may push: in the world, not something the player
    /// takes (an item's), not a creature.
    fn candidate(r: usize) -> bool {
        let Some(flags) = mem::at::<u32>(r + FORM_FLAGS) else { return false };
        let base = mem::deref(r + REFR_BASE);
        let kind = mem::at::<u8>(base + FORM_TYPE).unwrap_or(0);
        flags & (DELETED | DISABLED) == 0 && mem::deref(r + REFR_NODE) != 0 && !matches!(kind, NPC | CREATURE) && crate::items::takeable_ref(r).is_none()
    }

    fn world_of(node: usize) -> Option<(Mat3, Vec3)> {
        let r: [f32; 9] = mem::at(node + av::WORLD_ROTATE)?;
        let t: [f32; 3] = mem::at(node + av::WORLD_TRANSLATE)?;
        let m = crate::items::ni_matrix(r);
        (m.is_finite() && Vec3::from(t).is_finite()).then_some((m, Vec3::from(t)))
    }

    fn pose_of(node: usize) -> Option<(Vec3, Quat)> {
        let (m, t) = world_of(node)?;
        let b = camera::basis();
        Some((b.point_from_game(t), b.rot_from_game(m).normalize()))
    }

    /// The loose bodies under the references near `at` (the runtime's world): (reference, node,
    /// body, distance), nearest first.
    fn nearby(at: Vec3) -> Vec<(usize, usize, usize, f32)> {
        let mut out = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + CELL_OBJECTS;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                if r == 0 || !candidate(r) {
                    continue;
                }
                // (A reference is where it hangs from: its bodies may hang a few metres below.)
                let Some(p) = mem::at::<[f32; 3]>(r + refr::POS) else { continue };
                if camera::basis().point_from_game(Vec3::from(p)).distance(at) > REACH + 6.0 {
                    continue;
                }
                let mut stack = vec![(mem::deref(r + REFR_NODE), 0u32)];
                let mut seen = 0;
                while let Some((n, depth)) = stack.pop() {
                    seen += 1;
                    if seen > 256 || depth > 16 {
                        continue;
                    }
                    if let Some(hk) = loose_body(n)
                        && let Some((p, _)) = pose_of(n)
                        && p.distance(at) <= REACH
                    {
                        out.push((r, n, hk, p.distance(at)));
                    }
                    stack.extend(crate::world::child_nodes(n).into_iter().map(|c| (c, depth + 1)));
                }
            }
        }
        out.sort_by(|a, b| a.3.total_cmp(&b.3));
        out
    }

    /// A body as an object: where its node is, its Havok shape in the node's space, its mass.
    fn make(refr: usize, node: usize, hk: usize, id: u32) -> Option<Body> {
        let (m, t) = world_of(node)?;
        let at = pose_of(node)?;
        let b = camera::basis();
        let tris = crate::collision::body_triangles(hk);
        let local: Vec<Vec3> = tris.iter().flatten().map(|w| b.point_from_game(m.transpose() * (*w - t))).filter(|p| p.is_finite() && p.length() < 5.0).collect();
        let hull = crate::items::hull_points(&local, MAX_HULL_POINTS);
        if hull.len() < 4 {
            return None;
        }
        let (centre, half) = crate::items::bounds(&hull)?;
        let mass = mem::at::<f32>(mem::deref(hk + ENTITY_MOTION) + MOTION_MASS_INV).filter(|m| *m > 1e-4 && m.is_finite()).map_or(5.0, |m| 1.0 / m).clamp(0.1, 200.0);
        Some(Body { refr, node, hk, desc: object_desc(id, 1, at, centre, half, mass, true), hull })
    }

    /// Once a game frame: a look round, now and then, for the loose bodies near the player.
    pub fn frame(now: f64) {
        let player = engine::player();
        if player == 0 || mem::deref(player + REFR_NODE) == 0 {
            return;
        }
        with_state(|s| {
            if now < s.next_scan {
                return;
            }
            s.next_scan = now + SCAN_EVERY;
            let Some(feet) = mem::at::<[f32; 3]>(player + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p))) else { return };
            let chest = feet + Vec3::Y * 1.2;
            let near = if crate::menus::in_world() { nearby(chest) } else { Vec::new() };
            let mut keep = Vec::new();
            for &(refr, node, hk, _) in near.iter().take(MAX_BODIES) {
                let id = match s.ids.get(&hk) {
                    Some(&id) => id,
                    None => {
                        s.next_id += 1;
                        let id = ID_BASE + s.next_id;
                        s.ids.insert(hk, id);
                        id
                    }
                };
                keep.push(id);
                if s.bodies.get(&id).is_some_and(|b| b.node == node && b.hk == hk) {
                    continue;
                }
                if let Some(body) = make(refr, node, hk, id) {
                    tracing::info!("props: {:08X}'s {:?} offered ({id:#x}, {:.1} kg, {} points)", mem::at::<u32>(refr + FORM_ID).unwrap_or(0), crate::combat::read_str(mem::deref(node + 0x8)), body.desc.mass, body.hull.len());
                    s.bodies.insert(id, body);
                }
            }
            s.bodies.retain(|id, _| keep.contains(id));
            s.ids.retain(|_, id| keep.contains(id));
            let mut ids: Vec<u32> = s.bodies.keys().copied().collect();
            ids.sort();
            if ids != s.sent {
                publish(s);
                s.sent = ids;
            }
        });
    }

    fn publish(s: &State) {
        let mut bodies: Vec<&Body> = s.bodies.values().collect();
        bodies.sort_by_key(|b| b.desc.id);
        let mut hulls: Vec<GameHull> = Vec::new();
        for b in &bodies {
            let mut h: GameHull = bytemuck::Zeroable::zeroed();
            h.id = b.desc.id;
            h.count = b.hull.len().min(MAX_HULL_POINTS) as u32;
            for (k, q) in b.hull.iter().take(MAX_HULL_POINTS).enumerate() {
                h.points[k] = [q.x, q.y, q.z, 0.0];
            }
            hulls.push(h);
        }
        let descs: Vec<ObjectDesc> = bodies.iter().map(|b| b.desc).collect();
        let ok = crate::link::with_adapter(|a| crate::objects::publish_hulls(crate::objects::Owner::Props, &hulls, a) && crate::objects::publish(crate::objects::Owner::Props, &descs, a)).unwrap_or(false);
        if ok {
            tracing::info!("props: {} loose bodies offered", descs.len());
        }
    }

    /// Before the eyes are drawn: the hands' pushes given to the bodies, and where each body is now,
    /// for the runtime.
    pub fn place(now: f64) {
        with_state(|s| {
            // (A body whose reference or 3D has gone since the last look round, or everything while
            // the game loads: never touched again.)
            if !crate::menus::in_world() {
                s.bodies.clear();
            }
            let gone = s.bodies.len();
            s.bodies.retain(|id, b| {
                mem::is_readable(b.refr as *const u8, 0x48) && candidate(b.refr) && mem::is_readable(b.node as *const u8, 0xB8) && loose_body(b.node) == Some(b.hk) && s.ids.get(&b.hk) == Some(id)
            });
            if s.bodies.len() != gone {
                publish(s);
                s.sent = {
                    let mut ids: Vec<u32> = s.bodies.keys().copied().collect();
                    ids.sort();
                    ids
                };
            }
            if s.bodies.is_empty() {
                return;
            }
            let pushes = s.pushes.get_or_insert_with(bytemuck::zeroed_box);
            if crate::link::with_adapter(|a| a.read_game_pushes(pushes)).unwrap_or(false) {
                let n = (pushes.count as usize).min(MAX_GAME_PUSHES);
                for p in pushes.pushes[..n].iter().filter(|p| p.part == 0) {
                    let Some(b) = s.bodies.get(&p.id) else { continue };
                    let (point, dir, speed) = (Vec3::new(p.point[0], p.point[1], p.point[2]), Vec3::new(p.normal[0], p.normal[1], p.normal[2]), p.normal[3]);
                    if give(b.hk, point, dir, speed) && s.logged.get(&p.id).is_none_or(|t| now - t > 1.0) {
                        s.logged.insert(p.id, now);
                        tracing::info!("props: {:#x} pushed at {point:.2?}, {speed:.2} m/s", p.id);
                    }
                }
            }
            let poses = s.poses.get_or_insert_with(bytemuck::zeroed_box);
            poses.count = 0;
            for b in s.bodies.values() {
                let i = poses.count as usize;
                if i >= MAX_OBJECTS {
                    break;
                }
                if let Some((p, q)) = pose_of(b.node) {
                    poses.poses[i] = GamePose { id: b.desc.id, pose: Pose { pos: p.to_array(), valid: 1, rot: q.to_array() }, ..Default::default() };
                    poses.count += 1;
                }
            }
            let poses = &poses.poses[..poses.count as usize];
            crate::link::with_adapter(|a| crate::objects::publish_poses(crate::objects::Owner::Props, poses, a));
        });
    }

    /// The push a hand gives a body: what its point `point` (the runtime's world) lacks of going
    /// `speed` m/s along `dir`, as an impulse there. False if it needed none.
    fn give(hk: usize, point: Vec3, dir: Vec3, speed: f32) -> bool {
        if !dir.is_finite() || dir.length_squared() < 0.5 || !speed.is_finite() || speed <= 0.0 || !point.is_finite() {
            return false;
        }
        let m = mem::deref(hk + ENTITY_MOTION);
        if m == 0 || !mem::is_readable(m as *const u8, 0xF0) {
            return false;
        }
        let (Some(com), Some(inv_mass)) = (mem::at::<[f32; 4]>(m + MOTION_COM), mem::at::<f32>(m + MOTION_MASS_INV)) else { return false };
        let (Some(inertia), Some(impulse_at)) = (virt(m, VT_INERTIA_INV_WORLD), virt(m, VT_POINT_IMPULSE)) else { return false };
        #[repr(C, align(16))]
        #[derive(Clone, Copy, Default)]
        struct V4([f32; 4]);
        let mut i = [V4::default(); 3];
        type Inertia = unsafe extern "thiscall" fn(usize, *mut V4);
        unsafe { std::mem::transmute::<usize, Inertia>(inertia)(m, i.as_mut_ptr()) };
        let i_inv = Mat3::from_cols(Vec3::from_slice(&i[0].0[..3]), Vec3::from_slice(&i[1].0[..3]), Vec3::from_slice(&i[2].0[..3]));
        // In Havok's world: the lever from its centre of mass, and how fast the point moves along
        // the push now.
        let b = camera::basis();
        let at = b.point_to_game(point) / crate::collision::HAVOK_SCALE;
        let r = at - Vec3::new(com[0], com[1], com[2]);
        let n = b.dir_to_game(dir).normalize_or_zero();
        let v: [f32; 4] = mem::at(m + MOTION_LIN_VEL).unwrap_or_default();
        let w: [f32; 4] = mem::at(m + MOTION_ANG_VEL).unwrap_or_default();
        let moving = (Vec3::new(v[0], v[1], v[2]) + Vec3::new(w[0], w[1], w[2]).cross(r)).dot(n);
        let lack = speed / HK_METRES - moving;
        // (How much the point's speed along the push changes for each unit of impulse there.)
        let rn = r.cross(n);
        let k = inv_mass + rn.dot(i_inv * rn);
        if !lack.is_finite() || lack <= 0.0 || !k.is_finite() || k <= 1e-6 || !at.is_finite() || n == Vec3::ZERO {
            return false;
        }
        let j = n * (lack / k);
        let (impulse, at) = (V4([j.x, j.y, j.z, 0.0]), V4([at.x, at.y, at.z, 0.0]));
        type Activate = unsafe extern "thiscall" fn(usize) -> u32;
        type Impulse = unsafe extern "thiscall" fn(usize, *const V4, *const V4);
        unsafe {
            std::mem::transmute::<usize, Activate>(ACTIVATE)(hk);
            std::mem::transmute::<usize, Impulse>(impulse_at)(m, &impulse, &at);
        }
        true
    }
}
