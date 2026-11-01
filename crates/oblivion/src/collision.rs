//! Oblivion's world, for the runtime's physics: what the hands, the arms and what they hold meet,
//! and what what they drop and throw lands on, in view or not.
//!
//! **The shapes the hands meet are what's drawn.** Oblivion's own collision is a stand-in for most
//! of its models (a box round a chair, a hull round a table, a slab for a wall and its mouldings),
//! right for walking and things falling and wrong for a hand reaching between a chair's rungs,
//! into a shelf or along a wall. So each reference near the player whose model has collision of
//! its own (the fixed bodies under its 3D, below) is given the runtime as its model's drawn shape
//! instead ([`oblivion_assets::hitbox`]: the drawn triangles, less what the game doesn't make
//! solid), read from the game's archives on a thread of their own and placed where the game draws
//! it (its 3D's world transform), one collision mesh per reference. What has no model (the land),
//! a model with nothing drawn solid, and a model not read yet keep the game's own shape. The game's
//! own physics (the player's walking, the NPCs, things falling) keeps the game's collision.
//!
//! **The game's collision.** Oblivion simulates with Havok, a 2005 version. Everything that never
//! moves (the land, buildings, rocks, trees, furniture) is a fixed rigid body in the current Havok
//! world's fixed island: the exterior's world, or the interior cell's own. The adapter reads each
//! body's shape where Havok's own reflection (its `hkClass` tables, in the game) puts the fields:
//! boxes, spheres, capsules, cylinders, convex hulls, lists and transforms; every shape collection
//! (the packed triangle strips most of the architecture is, the land's height field) through
//! Havok's own interface for its children, one triangle at a time. A body's triangles are worked
//! out once, when the player first comes near it; the ones within reach that no drawn shape stands
//! in for go to the runtime as one collision mesh, sent again when the player has moved on or the
//! world changed, with each triangle's material (what a blade bites into): the game's own,
//! Bethesda's per shape (`bhkShape` +0x10, the wrapper a shape's user data points to) and per part
//! of a packed triangle mesh ([`material`]). A drawn shape's triangles take the material of the
//! game's nearest.
//!
//! The triangle maths is pure and tested; the Havok reads are guarded.

use glam::{Mat3, Vec3};

/// Game units to Havok's unit (Oblivion's figure: a tenth of a metre).
pub const HAVOK_SCALE: f32 = 7.0;
/// Metres around the player the runtime gets collision for, and how far the player moves before
/// it's sent again.
pub const RADIUS: f32 = 12.0;
pub const RESEND: f32 = 3.0;
/// At most this many triangles in the mesh of the game's shapes sent (nearest first).
pub const MAX_TRIANGLES: usize = 40_000;
/// At most this many drawn triangles in the runtime at once (the nearest references' first), and
/// sent in one frame (a frame's share of the collision ring: the rest go next frame).
pub const MAX_DRAWN_TRIANGLES: usize = 800_000;
pub const SEND_PER_FRAME: usize = 40_000;
/// The first id of the references' drawn meshes in the runtime (each its own, counting up).
pub const DRAWN_ID_BASE: u32 = 0x4F43_0000;
/// Drawn triangles kept read at most (models not near the player forgotten first).
pub const HITBOX_CACHE: usize = 1_000_000;
/// References whose 3D is looked through for fixed bodies in one scan (the rest the next).
pub const WALKS_PER_SCAN: usize = 600;
/// At most this many triangles from one body (a whole castle is far less).
pub const MAX_BODY_TRIANGLES: usize = 250_000;
/// The collision mesh's id in the runtime.
pub const MESH_ID: u32 = 0x4F42_0001;
/// Sides of the prism a round shape (sphere, capsule, cylinder: a tree's trunk) becomes.
pub const ROUND_SIDES: usize = 16;

pub use oblivion_assets::hitbox::solid_layer;

/// A rigid transform as Havok keeps one: rotation (columns), then translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Xf {
    pub r: Mat3,
    pub t: Vec3,
}

impl Xf {
    pub const IDENTITY: Xf = Xf { r: Mat3::IDENTITY, t: Vec3::ZERO };

    pub fn apply(&self, v: Vec3) -> Vec3 {
        self.r * v + self.t
    }

    /// `inner` (a child's, in this one's frame) then this one.
    pub fn of(&self, inner: &Xf) -> Xf {
        Xf { r: self.r * inner.r, t: self.apply(inner.t) }
    }
}

/// The 12 triangles of a box with half extents `h` about `c`.
pub fn box_triangles(c: Vec3, h: Vec3) -> Vec<[Vec3; 3]> {
    let p = |x: f32, y: f32, z: f32| c + Vec3::new(x * h.x, y * h.y, z * h.z);
    let faces = [
        [p(1., -1., -1.), p(1., 1., -1.), p(1., 1., 1.), p(1., -1., 1.)],
        [p(-1., -1., 1.), p(-1., 1., 1.), p(-1., 1., -1.), p(-1., -1., -1.)],
        [p(-1., 1., -1.), p(-1., 1., 1.), p(1., 1., 1.), p(1., 1., -1.)],
        [p(-1., -1., 1.), p(-1., -1., -1.), p(1., -1., -1.), p(1., -1., 1.)],
        [p(-1., -1., 1.), p(1., -1., 1.), p(1., 1., 1.), p(-1., 1., 1.)],
        [p(1., -1., -1.), p(-1., -1., -1.), p(-1., 1., -1.), p(1., 1., -1.)],
    ];
    faces.iter().flat_map(|f| [[f[0], f[1], f[2]], [f[0], f[2], f[3]]]).collect()
}

/// A round shape around the segment `a`-`b` of radius `r` (a capsule's ends extended by `r`, a
/// cylinder's not) as a `sides`-sided prism with its caps.
pub fn prism_triangles(a: Vec3, b: Vec3, r: f32, extend: bool, sides: usize) -> Vec<[Vec3; 3]> {
    let along = b - a;
    let axis = along.try_normalize().unwrap_or(Vec3::Z);
    let (a, b) = if extend { (a - axis * r, b + axis * r) } else { (a, b) };
    let u = axis.any_orthonormal_vector();
    let v = axis.cross(u);
    // (The prism's sides touch the circle: its corners lie a little outside it.)
    let rc = r / (std::f32::consts::PI / sides as f32).cos();
    let ring = |c: Vec3| -> Vec<Vec3> {
        (0..sides)
            .map(|i| {
                let t = i as f32 / sides as f32 * std::f32::consts::TAU;
                c + (u * t.cos() + v * t.sin()) * rc
            })
            .collect()
    };
    let (ra, rb) = (ring(a), ring(b));
    let mut out = Vec::with_capacity(sides * 4);
    for i in 0..sides {
        let j = (i + 1) % sides;
        out.push([ra[i], ra[j], rb[j]]);
        out.push([ra[i], rb[j], rb[i]]);
        out.push([a, ra[j], ra[i]]);
        out.push([b, rb[i], rb[j]]);
    }
    out
}

/// A convex hull's faces from its corners and its planes (outward normal `xyz`, `n·p + w = 0` on
/// the plane): each plane's corners in order around it, as a fan.
pub fn hull_triangles(corners: &[Vec3], planes: &[[f32; 4]]) -> Vec<[Vec3; 3]> {
    let size = corners.iter().fold(0.0f32, |m, c| m.max(c.abs().max_element()));
    let eps = (size * 1e-3).max(1e-4);
    let mut out = Vec::new();
    for p in planes {
        let n = Vec3::new(p[0], p[1], p[2]);
        let Some(n) = n.try_normalize() else { continue };
        let w = p[3] / Vec3::new(p[0], p[1], p[2]).length();
        let mut on: Vec<Vec3> = Vec::new();
        for c in corners {
            if (n.dot(*c) + w).abs() <= eps && !on.iter().any(|q| q.distance_squared(*c) < eps * eps) {
                on.push(*c);
            }
        }
        if on.len() < 3 {
            continue;
        }
        let mid = on.iter().copied().sum::<Vec3>() / on.len() as f32;
        let u = n.any_orthonormal_vector();
        let v = n.cross(u);
        on.sort_by(|a, b| {
            let (da, db) = (*a - mid, *b - mid);
            da.dot(v).atan2(da.dot(u)).total_cmp(&db.dot(v).atan2(db.dot(u)))
        });
        for i in 1..on.len() - 1 {
            out.push([on[0], on[i], on[i + 1]]);
        }
    }
    out
}

fn sphere(t: &[Vec3; 3]) -> (Vec3, f32) {
    let c = (t[0] + t[1] + t[2]) / 3.0;
    (c, t.iter().map(|v| v.distance(c)).fold(0.0, f32::max))
}

/// Distance from `at` to a triangle's bounding sphere, near enough for choosing.
pub fn sphere_distance(t: &[Vec3; 3], at: Vec3) -> f32 {
    let (c, r) = sphere(t);
    (c.distance(at) - r).max(0.0)
}

/// The triangles of `bodies` (with their materials) within `radius` of `at`, nearest first, at most
/// `max`.
pub fn nearby<'a>(bodies: impl IntoIterator<Item = (&'a [[Vec3; 3]], &'a [u8])>, at: Vec3, radius: f32, max: usize) -> Vec<([Vec3; 3], u8)> {
    let mut near: Vec<(f32, [Vec3; 3], u8)> = Vec::new();
    for (tris, mats) in bodies {
        near.extend(tris.iter().zip(mats).map(|(t, &m)| (sphere_distance(t, at), *t, m)).filter(|(d, ..)| *d <= radius));
    }
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    near.into_iter().take(max).map(|(_, t, m)| (t, m)).collect()
}

/// Each of `tris`' material: the material of the nearest of `from` (by their middles), or
/// `fallback` when there are none. One material for all of `from` is all of `tris`' (most models).
pub fn nearest_materials(tris: &[[Vec3; 3]], from: &[[Vec3; 3]], mats: &[u8], fallback: u8) -> Vec<u8> {
    let first = mats.first().copied().unwrap_or(fallback);
    if from.is_empty() || mats.iter().all(|&m| m == first) {
        return vec![first; tris.len()];
    }
    // A grid of `from`'s middles, half a metre a cell, looked through ring by ring.
    const CELL: f32 = 0.5;
    let key = |p: Vec3| ((p.x / CELL).floor() as i32, (p.y / CELL).floor() as i32, (p.z / CELL).floor() as i32);
    let mid = |t: &[Vec3; 3]| (t[0] + t[1] + t[2]) / 3.0;
    let mut grid: std::collections::HashMap<(i32, i32, i32), Vec<usize>> = std::collections::HashMap::new();
    for (i, t) in from.iter().enumerate() {
        grid.entry(key(mid(t))).or_default().push(i);
    }
    tris.iter()
        .map(|t| {
            let p = mid(t);
            let (x, y, z) = key(p);
            for ring in 0i32..6 {
                let mut best: Option<(f32, usize)> = None;
                for dx in -ring..=ring {
                    for dy in -ring..=ring {
                        for dz in -ring..=ring {
                            if dx.abs().max(dy.abs()).max(dz.abs()) != ring {
                                continue;
                            }
                            for &i in grid.get(&(x + dx, y + dy, z + dz)).into_iter().flatten() {
                                let d = mid(&from[i]).distance_squared(p);
                                if best.is_none_or(|b| d < b.0) {
                                    best = Some((d, i));
                                }
                            }
                        }
                    }
                }
                if let Some((_, i)) = best {
                    return mats.get(i).copied().unwrap_or(fallback);
                }
            }
            first
        })
        .collect()
}

/// Bethesda's collision material (Oblivion's 0..31: stone, cloth, dirt, glass, grass, metal,
/// organic, skin, water, wood, heavy stone, heavy metal, heavy wood, chain, snow; then each again
/// as stairs; the elevator) as the framework's `MAT_*`.
pub fn material(m: u32) -> u32 {
    use rustvr_protocol::{MAT_CHAINMAIL, MAT_EARTH, MAT_FLESH, MAT_LEATHER, MAT_PLATE, MAT_STONE, MAT_WOOD};
    match if (15..30).contains(&m) { m - 15 } else { m } {
        1 => MAT_LEATHER,
        2 | 4 | 14 => MAT_EARTH,
        5 | 11 => MAT_PLATE,
        6 | 9 | 12 => MAT_WOOD,
        7 => MAT_FLESH,
        13 => MAT_CHAINMAIL,
        _ => MAT_STONE,
    }
}

/// The height of the highest triangle under `at` (straight down along `up`, no higher than `at`
/// + 0.5 m): for the log, to see the world lines up with the player.
pub fn floor_under(tris: &[[Vec3; 3]], at: Vec3) -> Option<f32> {
    let mut best: Option<f32> = None;
    for t in tris {
        let (a, b, c) = (t[0], t[1], t[2]);
        let d = (b.z - c.z) * (a.x - c.x) + (c.x - b.x) * (a.z - c.z);
        if d.abs() < 1e-9 {
            continue;
        }
        let l1 = ((b.z - c.z) * (at.x - c.x) + (c.x - b.x) * (at.z - c.z)) / d;
        let l2 = ((c.z - a.z) * (at.x - c.x) + (a.x - c.x) * (at.z - c.z)) / d;
        let l3 = 1.0 - l1 - l2;
        if l1 < 0.0 || l2 < 0.0 || l3 < 0.0 {
            continue;
        }
        let y = l1 * a.y + l2 * b.y + l3 * c.y;
        if y <= at.y + 0.5 && best.is_none_or(|h| y > h) {
            best = Some(y);
        }
    }
    best
}

/// How far along `dir` (a unit length) from `from` the ray meets the triangle `t`, either side.
pub fn ray_triangle(from: Vec3, dir: Vec3, t: &[Vec3; 3]) -> Option<f32> {
    let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let s = from - t[0];
    let u = s.dot(p) / det;
    let q = s.cross(e1);
    let v = dir.dot(q) / det;
    if u < 0.0 || v < 0.0 || u + v > 1.0 {
        return None;
    }
    Some(e2.dot(q) / det).filter(|d| *d >= 0.0)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::camera;
    use crate::engine::{self, av, refr};
    use crate::mem;
    use oblivion_assets::hitbox::Hitbox;
    use oblivion_assets::nif::Nif;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// `TES*` (xOBSE's `g_TES`) and its current interior cell (null outside).
    const TES: usize = 0x00B3_33A0;
    const TES_INTERIOR: usize = 0x34;
    /// The exterior's `bhkWorldM*`.
    const EXTERIOR_WORLD: usize = 0x00B3_5C24;
    /// A cell's first extra data (its `ExtraDataList` at +0x28, after the list's vtable); an
    /// extra's type and next, and `ExtraHavok`'s world.
    const CELL_EXTRA: usize = 0x2C;
    const EXTRA_TYPE: usize = 0x4;
    const EXTRA_NEXT: usize = 0x8;
    const EXTRA_HAVOK: u8 = 0x02;
    const EXTRA_HAVOK_WORLD: usize = 0xC;
    /// `bhkRefObject::hkObj`; the `ahkWorld` vtable (the game's `hkWorld`).
    const HK_OBJ: usize = 0x8;
    const AHKWORLD_VTABLE: usize = 0x00A9_5C54;
    /// `hkWorld::m_fixedIsland`; `hkSimulationIsland::m_entities` (pointer, size).
    const WORLD_FIXED_ISLAND: usize = 0x30;
    const ISLAND_ENTITIES: usize = 0x34;
    /// `hkWorldObject`: its world; its collidable's shape and transform (`hkCdBody`: shape +0,
    /// motion +8, at +0x14); its broad phase handle's filter info (+0x14 + 0x14 + 8).
    const ENTITY_WORLD: usize = 0x8;
    const ENTITY_SHAPE: usize = 0x14;
    const ENTITY_TRANSFORM: usize = 0x1C;
    const ENTITY_FILTER: usize = 0x30;

    // hkShape: getType (slot 2), getAabb (3); hkShapeCollection: getFirstKey (8), getNextKey (9),
    // getChildShape (10). `hkShapeType` from the game's own names for them.
    const SLOT_TYPE: usize = 2;
    const SLOT_AABB: usize = 3;
    const SLOT_FIRST_KEY: usize = 8;
    const SLOT_NEXT_KEY: usize = 9;
    const SLOT_CHILD: usize = 10;
    const COLLECTION: i32 = 2;
    const BV_TREE: i32 = 3;
    const SPHERE: i32 = 4;
    const CYLINDER: i32 = 5;
    const TRIANGLE: i32 = 6;
    const BOX: i32 = 7;
    const CAPSULE: i32 = 8;
    const CONVEX_VERTICES: i32 = 9;
    const MULTI_SPHERE: i32 = 11;
    const LIST: i32 = 12;
    const CONVEX_LIST: i32 = 13;
    const CONVEX_TRANSLATE: i32 = 14;
    const CONVEX_TRANSFORM: i32 = 15;
    const TRIANGLE_COLLECTION: i32 = 16;
    const MOPP: i32 = 24;
    const TRANSFORM: i32 = 25;
    const END_KEY: u32 = 0xFFFF_FFFF;
    /// The game's code: a vtable entry outside it isn't one.
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A0_0000;

    #[repr(C, align(16))]
    #[derive(Clone, Copy, Default)]
    struct HkTransform {
        cols: [[f32; 4]; 4],
    }

    #[repr(C, align(16))]
    #[derive(Clone, Copy, Default)]
    struct HkAabb {
        min: [f32; 4],
        max: [f32; 4],
    }

    /// What `getChildShape` builds a child in (Havok's `ShapeBuffer` is 512 bytes).
    #[repr(C, align(16))]
    struct ShapeBuffer([u8; 1024]);

    fn v3(a: [f32; 4]) -> Vec3 {
        Vec3::new(a[0], a[1], a[2])
    }

    fn vec4(at: usize) -> Option<Vec3> {
        mem::at::<[f32; 4]>(at).map(v3).filter(|v| v.is_finite())
    }

    fn transform(at: usize) -> Option<Xf> {
        let t: HkTransform = mem::at::<[[f32; 4]; 4]>(at).map(|cols| HkTransform { cols })?;
        let xf = Xf { r: Mat3::from_cols(v3(t.cols[0]), v3(t.cols[1]), v3(t.cols[2])), t: v3(t.cols[3]) };
        (xf.r.is_finite() && xf.t.is_finite()).then_some(xf)
    }

    fn quat_transform(rot_at: usize, t: Vec3) -> Option<Xf> {
        let q = mem::at::<[f32; 4]>(rot_at)?;
        let q = glam::Quat::from_xyzw(q[0], q[1], q[2], q[3]);
        q.is_finite().then(|| Xf { r: Mat3::from_quat(q.normalize()), t })
    }

    /// A Havok array's (pointer, size).
    fn array(at: usize) -> Option<(usize, usize)> {
        let [p, n] = mem::at::<[u32; 2]>(at)?;
        Some((p as usize, n as usize))
    }

    /// Slot `slot` of `obj`'s vtable, if it's the game's code.
    fn slot(obj: usize, slot: usize) -> Option<usize> {
        let vt = mem::deref(obj);
        let f = mem::deref(vt + slot * 4);
        CODE.contains(&f).then_some(f)
    }

    fn shape_type(shape: usize) -> Option<i32> {
        let f = slot(shape, SLOT_TYPE)?;
        type F = unsafe extern "thiscall" fn(usize) -> i32;
        Some(unsafe { std::mem::transmute::<usize, F>(f)(shape) })
    }

    /// The shape's box placed by `xf`, in Havok's world.
    fn aabb(shape: usize, xf: &Xf) -> Option<(Vec3, Vec3)> {
        let f = slot(shape, SLOT_AABB)?;
        let mut t = HkTransform::default();
        for (i, c) in [xf.r.x_axis, xf.r.y_axis, xf.r.z_axis, xf.t].iter().enumerate() {
            t.cols[i] = [c.x, c.y, c.z, 0.0];
        }
        let mut out = HkAabb::default();
        type F = unsafe extern "thiscall" fn(usize, *const HkTransform, f32, *mut HkAabb);
        unsafe { std::mem::transmute::<usize, F>(f)(shape, &t, 0.0, &mut out) };
        let (lo, hi) = (v3(out.min), v3(out.max));
        (lo.is_finite() && hi.is_finite() && lo.cmple(hi).all()).then_some((lo, hi))
    }

    /// Every child of a shape collection, through its own interface: `each(child, key)` with the
    /// child built in a buffer that's reused (read it before returning).
    fn children(collection: usize, mut each: impl FnMut(usize, u32) -> bool) {
        let (Some(first), Some(next), Some(child)) = (slot(collection, SLOT_FIRST_KEY), slot(collection, SLOT_NEXT_KEY), slot(collection, SLOT_CHILD)) else { return };
        type First = unsafe extern "thiscall" fn(usize) -> u32;
        type Next = unsafe extern "thiscall" fn(usize, u32) -> u32;
        type Child = unsafe extern "thiscall" fn(usize, u32, *mut ShapeBuffer) -> usize;
        let (first, next, child) = unsafe { (std::mem::transmute::<usize, First>(first), std::mem::transmute::<usize, Next>(next), std::mem::transmute::<usize, Child>(child)) };
        let mut buf = Box::new(ShapeBuffer([0; 1024]));
        let mut key = unsafe { first(collection) };
        let mut n = 0usize;
        while key != END_KEY && n < MAX_BODY_TRIANGLES {
            let c = unsafe { child(collection, key, &mut *buf) };
            if c != 0 && !each(c, key) {
                return;
            }
            key = unsafe { next(collection, key) };
            n += 1;
        }
    }

    /// `hkPackedNiTriStripsShape`'s vtable; its data (+0x10) has its parts (+0x1C, 12 bytes each:
    /// filter info, vertex count, material), a child's part the key's top byte.
    const PACKED_STRIPS_VTABLE: usize = 0x00A9_940C;
    const STRIPS_DATA: usize = 0x10;
    const STRIPS_PARTS: usize = 0x1C;
    const PART_SIZE: usize = 12;
    const PART_MATERIAL: usize = 8;
    /// `bhkShape::material`, on the wrapper a shape's user data (+8) points to.
    const WRAPPER_MATERIAL: usize = 0x10;

    /// The shape's own material (its Bethesda wrapper's), if it has one.
    fn own_material(shape: usize) -> Option<u32> {
        let wrapper = mem::deref(shape + 8);
        (wrapper != 0).then(|| mem::at::<u32>(wrapper + WRAPPER_MATERIAL)).flatten().filter(|&m| m < 32).map(material)
    }

    /// A body's Bethesda collision material (0..31, as the game's collision sounds take it): its
    /// shape's own (its Bethesda wrapper's).
    pub fn body_material(entity: usize) -> Option<u32> {
        let wrapper = mem::deref(mem::deref(entity + ENTITY_SHAPE) + 8);
        (wrapper != 0).then(|| mem::at::<u32>(wrapper + WRAPPER_MATERIAL)).flatten().filter(|&m| m < 32)
    }

    /// A body's triangles in the game's world (game units): its own shape where Havok has it now
    /// (one of the things the hands take, [`crate::items`]).
    pub fn body_triangles(entity: usize) -> Vec<[Vec3; 3]> {
        let shape = mem::deref(entity + ENTITY_SHAPE);
        let Some(xf) = transform(mem::deref(entity + ENTITY_TRANSFORM)) else { return Vec::new() };
        let (mut tris, mut mats, mut unknown) = (Vec::new(), Vec::new(), HashMap::new());
        shape_triangles(shape, &xf, rustvr_protocol::MAT_STONE, &mut tris, &mut mats, &mut unknown, 0);
        tris.into_iter().map(|t| t.map(|v| v * HAVOK_SCALE)).collect()
    }

    /// The triangles of `shape` placed by `xf` (Havok's world), added to `out` with their materials
    /// to `mats` (`material`: the one it inherits); the kinds it doesn't know are counted in
    /// `unknown`.
    fn shape_triangles(shape: usize, xf: &Xf, material: u32, out: &mut Vec<[Vec3; 3]>, mats: &mut Vec<u8>, unknown: &mut HashMap<i32, u32>, depth: u32) {
        let material = own_material(shape).unwrap_or(material);
        shape_triangles_of(shape, xf, material, out, mats, unknown, depth);
        // (The triangles a leaf shape added are its material; a collection's children set theirs.)
        if mats.len() < out.len() {
            mats.resize(out.len(), material as u8);
        }
    }

    fn shape_triangles_of(shape: usize, xf: &Xf, material: u32, out: &mut Vec<[Vec3; 3]>, mats: &mut Vec<u8>, unknown: &mut HashMap<i32, u32>, depth: u32) {
        if depth > 12 || out.len() >= MAX_BODY_TRIANGLES || !mem::is_readable(shape as *const u8, 0x10) {
            return;
        }
        let Some(ty) = shape_type(shape) else { return };
        let radius = mem::at::<f32>(shape + 0xC).filter(|r| r.is_finite() && *r >= 0.0).unwrap_or(0.0);
        let place = |tris: Vec<[Vec3; 3]>, out: &mut Vec<[Vec3; 3]>| out.extend(tris.into_iter().map(|t| t.map(|v| xf.apply(v))));
        match ty {
            TRIANGLE => {
                if let (Some(a), Some(b), Some(c)) = (vec4(shape + 0x10), vec4(shape + 0x20), vec4(shape + 0x30)) {
                    out.push([xf.apply(a), xf.apply(b), xf.apply(c)]);
                }
            }
            BOX => {
                if let Some(h) = vec4(shape + 0x10) {
                    place(box_triangles(Vec3::ZERO, h + Vec3::splat(radius)), out);
                }
            }
            SPHERE => place(prism_triangles(Vec3::new(0.0, 0.0, -radius), Vec3::new(0.0, 0.0, radius), radius, false, ROUND_SIDES), out),
            CAPSULE => {
                if let (Some(a), Some(b)) = (vec4(shape + 0x10), vec4(shape + 0x20)) {
                    place(prism_triangles(a, b, radius, true, ROUND_SIDES), out);
                }
            }
            CYLINDER => {
                if let (Some(r), Some(a), Some(b)) = (mem::at::<f32>(shape + 0x10), vec4(shape + 0x20), vec4(shape + 0x30)) {
                    place(prism_triangles(a, b, r + radius, false, ROUND_SIDES), out);
                }
            }
            MULTI_SPHERE => {
                let n = mem::at::<u32>(shape + 0xC).unwrap_or(0).min(8) as usize;
                for i in 0..n {
                    if let Some(s) = mem::at::<[f32; 4]>(shape + 0x10 + i * 16) {
                        let c = v3(s);
                        place(prism_triangles(c - Vec3::Z * s[3], c + Vec3::Z * s[3], s[3], false, ROUND_SIDES), out);
                    }
                }
            }
            CONVEX_VERTICES => {
                // The corners four at a time (x, y, z of four), and the faces' planes.
                let corners: Vec<Vec3> = (|| {
                    let (p, _) = array(shape + 0x30)?;
                    let n = mem::at::<u32>(shape + 0x3C)? as usize;
                    let blocks: Vec<[[f32; 4]; 3]> = (0..n.div_ceil(4)).map(|i| mem::at(p + i * 48)).collect::<Option<_>>()?;
                    Some((0..n).map(|i| { let b = &blocks[i / 4]; Vec3::new(b[0][i % 4], b[1][i % 4], b[2][i % 4]) }).collect())
                })()
                .unwrap_or_default();
                let planes: Vec<[f32; 4]> = array(shape + 0x40).map(|(p, n)| (0..n.min(1024)).filter_map(|i| mem::at(p + i * 16)).collect()).unwrap_or_default();
                let tris = hull_triangles(&corners, &planes);
                if tris.is_empty() {
                    // (No faces: the box around it.)
                    if let (Some(h), Some(c)) = (vec4(shape + 0x10), vec4(shape + 0x20)) {
                        place(box_triangles(c, h), out);
                    }
                } else {
                    place(tris, out);
                }
            }
            CONVEX_TRANSLATE => {
                if let Some(t) = vec4(shape + 0x20) {
                    let child = mem::deref(shape + 0x10);
                    shape_triangles(child, &xf.of(&Xf { r: Mat3::IDENTITY, t }), material, out, mats, unknown, depth + 1);
                }
            }
            CONVEX_TRANSFORM => {
                if let Some(t) = transform(shape + 0x20) {
                    shape_triangles(mem::deref(shape + 0x10), &xf.of(&t), material, out, mats, unknown, depth + 1);
                }
            }
            TRANSFORM => {
                if let Some(t) = transform(shape + 0x20).or_else(|| quat_transform(shape + 0x10, Vec3::ZERO)) {
                    shape_triangles(mem::deref(shape + 0xC), &xf.of(&t), material, out, mats, unknown, depth + 1);
                }
            }
            BV_TREE | MOPP => shape_triangles(mem::deref(shape + 0xC), xf, material, out, mats, unknown, depth + 1),
            COLLECTION | LIST | CONVEX_LIST | TRIANGLE_COLLECTION => {
                // A packed triangle mesh's parts each have a material of their own.
                let parts = (mem::deref(shape) == PACKED_STRIPS_VTABLE).then(|| mem::deref(mem::deref(shape + STRIPS_DATA) + STRIPS_PARTS)).filter(|&p| p != 0);
                children(shape, |child, key| {
                    let m = parts
                        .and_then(|p| mem::at::<u32>(p + (key >> 24) as usize * PART_SIZE + PART_MATERIAL))
                        .filter(|&m| m < 32)
                        .map(super::material)
                        .unwrap_or(material);
                    shape_triangles(child, xf, m, out, mats, unknown, depth + 1);
                    out.len() < MAX_BODY_TRIANGLES
                })
            }
            other => *unknown.entry(other).or_default() += 1,
        }
    }

    /// The Havok world the player is in (`hkWorld*`): the interior cell's, or the exterior's.
    fn world() -> Option<usize> {
        let tes = mem::deref(TES);
        let interior = if tes != 0 { mem::deref(tes + TES_INTERIOR) } else { 0 };
        let bhk = if interior != 0 {
            let mut extra = mem::deref(interior + CELL_EXTRA);
            let mut found = 0;
            for _ in 0..256 {
                if extra == 0 {
                    break;
                }
                if mem::at::<u8>(extra + EXTRA_TYPE) == Some(EXTRA_HAVOK) {
                    found = mem::deref(extra + EXTRA_HAVOK_WORLD);
                    break;
                }
                extra = mem::deref(extra + EXTRA_NEXT);
            }
            found
        } else {
            mem::deref(EXTERIOR_WORLD)
        };
        let hk = if bhk != 0 { mem::deref(bhk + HK_OBJ) } else { 0 };
        (hk != 0 && mem::deref(hk) == AHKWORLD_VTABLE).then_some(hk)
    }

    /// A fixed body: its shape and where it is (Havok's world), its layer.
    #[derive(Clone, Copy)]
    struct Fixed {
        shape: usize,
        xf: Xf,
        layer: u32,
    }

    fn fixed_bodies(world: usize) -> HashMap<usize, Fixed> {
        let mut out = HashMap::new();
        let island = mem::deref(world + WORLD_FIXED_ISLAND);
        let Some((p, n)) = (island != 0).then(|| array(island + ISLAND_ENTITIES)).flatten() else { return out };
        for i in 0..n.min(200_000) {
            let e = mem::deref(p + i * 4);
            if e == 0 || mem::deref(e + ENTITY_WORLD) != world {
                continue;
            }
            let shape = mem::deref(e + ENTITY_SHAPE);
            let motion = mem::deref(e + ENTITY_TRANSFORM);
            let (Some(xf), Some(filter)) = ((motion != 0).then(|| transform(motion)).flatten(), mem::at::<u32>(e + ENTITY_FILTER)) else { continue };
            if shape != 0 {
                out.insert(e, Fixed { shape, xf, layer: filter & 0x7F });
            }
        }
        out
    }

    fn to_runtime(p: Vec3) -> Vec3 {
        camera::basis().point_from_game(p * HAVOK_SCALE)
    }

    struct Body {
        fixed: Fixed,
        /// Its box in the runtime's world.
        lo: Vec3,
        hi: Vec3,
        /// Its triangles in the runtime's world, once it's been near, and their materials.
        tris: Option<Vec<[Vec3; 3]>>,
        mats: Vec<u8>,
    }

    impl Body {
        fn gap(&self, at: Vec3) -> f32 {
            (self.lo - at).max(at - self.hi).max(Vec3::ZERO).length()
        }

        /// Its triangles worked out, if they aren't yet (how many).
        fn read(&mut self, unknown: &mut HashMap<i32, u32>) -> usize {
            if self.tris.is_some() {
                return 0;
            }
            let (mut tris, mut mats) = (Vec::new(), Vec::new());
            shape_triangles(self.fixed.shape, &self.fixed.xf, rustvr_protocol::MAT_STONE, &mut tris, &mut mats, unknown, 0);
            mats.resize(tris.len(), rustvr_protocol::MAT_STONE as u8);
            let n = tris.len();
            self.tris = Some(tris.into_iter().map(|t| t.map(to_runtime)).collect());
            self.mats = mats;
            n
        }
    }

    /// A reference whose 3D holds fixed bodies: its model's drawn shape stands in for them.
    struct Owner {
        root: usize,
        model: String,
        bodies: Vec<usize>,
        /// Its bodies' box in the runtime's world.
        lo: Vec3,
        hi: Vec3,
        /// Its model's hitbox once read (None in it: nothing drawn is solid, its bodies stay the
        /// game's shape).
        hitbox: Option<Option<Arc<Hitbox>>>,
        /// Its mesh's id in the runtime and the 3D's world transform it was placed by, while sent.
        id: u32,
        sent: Option<[f32; 13]>,
        triangles: usize,
        /// Seen in the latest look round.
        seen: bool,
    }

    impl Owner {
        fn gap(&self, at: Vec3) -> f32 {
            (self.lo - at).max(at - self.hi).max(Vec3::ZERO).length()
        }

        fn drawn(&self) -> Option<&Arc<Hitbox>> {
            self.hitbox.as_ref().and_then(|h| h.as_ref())
        }
    }

    struct State {
        world: usize,
        bodies: HashMap<usize, Body>,
        last_scan: f64,
        sent_at: Option<Vec3>,
        connections: u32,
        logged_world: bool,
        /// The references holding fixed bodies (by reference), which reference holds each body,
        /// and the references looked through that hold none (with the 3D they had, and how many
        /// times bodies had come into the world then: they're looked through again after).
        owners: HashMap<usize, Owner>,
        claimed: HashMap<usize, usize>,
        plain: HashMap<usize, (usize, u32)>,
        bodies_added: u32,
        next_id: u32,
        /// The bodies left out of the game's mesh as last sent (their drawn shapes stand in).
        covered: Vec<usize>,
        /// The drawn meshes to take out of the runtime.
        to_remove: Vec<u32>,
        logged_owners: u32,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    /// How far the world's fixed things are along `dir` (unit length) from the nearest of the
    /// points `from` (the runtime's world, metres), within `look`: of the bodies whose triangles
    /// have been worked out (all of them near the player). The game's own shapes: where the game's
    /// body can go.
    pub fn nearest_along(from: &[Vec3], dir: Vec3, look: f32) -> Option<f32> {
        let st = STATE.lock().unwrap();
        let s = st.as_ref()?;
        let (lo, hi) = from.iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        let (lo, hi) = (lo.min(lo + dir * look), hi.max(hi + dir * look));
        let apart = |a: Vec3, b: Vec3| b.cmplt(lo).any() || a.cmpgt(hi).any();
        let mut best: Option<f32> = None;
        for b in s.bodies.values().filter(|b| !apart(b.lo, b.hi)) {
            for t in b.tris.iter().flatten().filter(|t| !apart(t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]))) {
                for p in from {
                    if let Some(d) = ray_triangle(*p, dir, t).filter(|d| *d <= look && best.is_none_or(|b| *d < b)) {
                        best = Some(d);
                    }
                }
            }
        }
        best
    }

    // ---- the references and their models --------------------------------------------------------
    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const DELETED: u32 = 0x20;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    /// A cell's references (`{refr, next}`); a node's collision object, its `bhkRigidBody` (+0x10),
    /// that one's `hkRigidBody` (+8).
    const CELL_OBJECTS: usize = 0x48;
    const NODE_COLLISION: usize = 0xA8;
    const COLLISION_BODY: usize = 0x10;
    /// Form types whose references are the characters (`NPC_`, `CREA`): never the world's.
    const NPC: u8 = 0x23;
    const CREATURE: u8 = 0x24;

    /// The fixed bodies (of `bodies`) under a reference's 3D.
    fn bodies_under(root: usize, bodies: &HashMap<usize, Body>) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = vec![(root, 0u32)];
        let mut n = 0;
        while let Some((node, depth)) = stack.pop() {
            n += 1;
            if n > 4096 || depth > 32 {
                break;
            }
            let co = mem::deref(node + NODE_COLLISION);
            if co != 0 {
                let rb = mem::deref(co + COLLISION_BODY);
                let hk = if rb != 0 { mem::deref(rb + HK_OBJ) } else { 0 };
                if hk != 0 && bodies.contains_key(&hk) && !out.contains(&hk) {
                    out.push(hk);
                }
            }
            stack.extend(crate::world::child_nodes(node).into_iter().map(|c| (c, depth + 1)));
        }
        out
    }

    /// The 3D's world transform: rotation (rows), translation, scale.
    fn placement(root: usize) -> Option<[f32; 13]> {
        let r: [f32; 9] = mem::at(root + av::WORLD_ROTATE)?;
        let t: [f32; 3] = mem::at(root + av::WORLD_TRANSLATE)?;
        let s: f32 = mem::at(root + av::WORLD_SCALE)?;
        let mut out = [0.0; 13];
        out[..9].copy_from_slice(&r);
        out[9..12].copy_from_slice(&t);
        out[12] = s;
        (out.iter().all(|v| v.is_finite()) && s > 0.0).then_some(out)
    }

    /// A hitbox placed by a 3D's world transform, in the runtime's world.
    fn place(h: &Hitbox, at: &[f32; 13]) -> Vec<[Vec3; 3]> {
        let rows: [f32; 9] = at[..9].try_into().unwrap();
        let r = crate::items::ni_matrix(rows);
        let (t, s) = (Vec3::new(at[9], at[10], at[11]), at[12]);
        let b = camera::basis();
        let points: Vec<Vec3> = h.vertices.iter().map(|v| b.point_from_game(t + r * (*v * s))).collect();
        h.triangles.iter().map(|t| t.map(|i| points[i as usize])).collect()
    }

    /// Looks through the references in the loaded cells near the player for the ones holding fixed
    /// bodies: those not looked through yet, or not since bodies came into the world, nearest
    /// first, [`WALKS_PER_SCAN`] of them.
    fn look_round(s: &mut State, feet: Vec3) {
        for o in s.owners.values_mut() {
            o.seen = false;
        }
        let b = camera::basis();
        let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut todo: Vec<(f32, usize, usize, usize)> = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + CELL_OBJECTS;
            let mut guard = 0;
            while entry != 0 && guard < 16384 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                if r == 0 {
                    continue;
                }
                let root = mem::deref(r + REFR_NODE);
                let flags = mem::at::<u32>(r + FORM_FLAGS).unwrap_or(DELETED);
                if root == 0 || flags & (DELETED | DISABLED) != 0 {
                    continue;
                }
                seen.insert(r);
                if let Some(o) = s.owners.get_mut(&r).filter(|o| o.root == root) {
                    o.seen = true;
                    continue;
                }
                if s.plain.get(&r) == Some(&(root, s.bodies_added)) {
                    continue;
                }
                let base = mem::deref(r + REFR_BASE);
                let kind = mem::at::<u8>(base + FORM_TYPE).unwrap_or(0);
                if base == 0 || kind == NPC || kind == CREATURE {
                    continue;
                }
                let at = mem::at::<[f32; 3]>(r + refr::POS).map_or(f32::MAX, |p| b.point_from_game(Vec3::from(p)).distance(feet));
                todo.push((at, r, root, base));
            }
        }
        todo.sort_by(|a, b| a.0.total_cmp(&b.0));
        for &(_, r, root, base) in todo.iter().take(WALKS_PER_SCAN) {
            let bodies = bodies_under(root, &s.bodies);
            let model = crate::items::model_of(base).unwrap_or_default();
            if bodies.is_empty() || model.is_empty() {
                s.plain.insert(r, (root, s.bodies_added));
                continue;
            }
            if let Some(old) = s.owners.remove(&r) {
                forget(s, old);
            }
            let (lo, hi) = bodies.iter().filter_map(|e| s.bodies.get(e)).fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), b| (lo.min(b.lo), hi.max(b.hi)));
            for &e in &bodies {
                s.claimed.insert(e, r);
            }
            s.plain.remove(&r);
            s.owners.insert(r, Owner { root, model, bodies, lo, hi, hitbox: None, id: 0, sent: None, triangles: 0, seen: true });
        }
        let gone: Vec<usize> = s.owners.iter().filter(|(_, o)| !o.seen).map(|(r, _)| *r).collect();
        for r in gone {
            if let Some(o) = s.owners.remove(&r) {
                forget(s, o);
            }
        }
        // (A cell gone, its references are forgotten with it.)
        s.plain.retain(|r, _| seen.contains(r));
    }

    /// A reference no longer the hands': its bodies are the game's shape again, its mesh taken out.
    fn forget(s: &mut State, o: Owner) {
        for e in &o.bodies {
            if s.claimed.get(e).is_some_and(|r| !s.owners.contains_key(r)) {
                s.claimed.remove(e);
            }
        }
        if o.sent.is_some() {
            s.to_remove.push(o.id);
        }
    }

    // ---- the models' hitboxes, read from the game's archives on a thread of their own ------------
    /// The game's archives, opened once (by the first thread that asks: it takes a moment).
    pub(crate) fn archives() -> Option<Arc<oblivion_assets::bsa::Archives>> {
        static ARCHIVES: std::sync::OnceLock<Option<Arc<oblivion_assets::bsa::Archives>>> = std::sync::OnceLock::new();
        ARCHIVES
            .get_or_init(|| {
                let game = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default();
                match oblivion_assets::bsa::Archives::open(&game) {
                    Ok(a) => {
                        tracing::info!("the game's archives opened ({} of them)", a.bsas.len());
                        Some(Arc::new(a))
                    }
                    Err(e) => {
                        tracing::warn!("the game's archives: {e:#}");
                        None
                    }
                }
            })
            .clone()
    }

    #[derive(Default)]
    struct Hitboxes {
        /// Model path (lower case) -> its hitbox (None: nothing drawn is solid, or it couldn't be
        /// read), and when it was last asked for.
        done: HashMap<String, (Option<Arc<Hitbox>>, f64)>,
        wanted: Vec<String>,
        busy: bool,
        triangles: usize,
    }
    static HITBOXES: Mutex<Option<Hitboxes>> = Mutex::new(None);

    fn with_hitboxes<R>(f: impl FnOnce(&mut Hitboxes) -> R) -> R {
        f(HITBOXES.lock().unwrap().get_or_insert_with(Hitboxes::default))
    }

    /// A model's hitbox, if it's read; None while it's being read.
    fn hitbox(model: &str, now: f64) -> Option<Option<Arc<Hitbox>>> {
        let key = model.to_ascii_lowercase().replace('/', "\\");
        // (A tree is grown from its SpeedTree file, not drawn from a model: it keeps the game's.)
        if !key.ends_with(".nif") {
            return Some(None);
        }
        let start = with_hitboxes(|h| {
            if let Some(d) = h.done.get_mut(&key) {
                d.1 = now;
                return Err(d.0.clone());
            }
            if !h.wanted.contains(&key) {
                h.wanted.push(key.clone());
            }
            Ok(!std::mem::replace(&mut h.busy, true))
        });
        match start {
            Err(d) => Some(d),
            Ok(true) => {
                std::thread::spawn(move || read_hitboxes(now));
                None
            }
            Ok(false) => None,
        }
    }

    fn read_hitboxes(now: f64) {
        let Some(archives) = archives() else {
            with_hitboxes(|h| {
                for k in std::mem::take(&mut h.wanted) {
                    h.done.insert(k, (None, now));
                }
                h.busy = false;
            });
            return;
        };
        loop {
            let Some(path) = with_hitboxes(|h| h.wanted.pop()) else { break };
            let started = std::time::Instant::now();
            // (A model that trips the reader keeps the game's shape, and reading goes on.)
            let made = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let nif = archives.read(&format!("meshes\\{path}")).and_then(|d| Nif::parse(&d))?;
                let (h, notes) = oblivion_assets::hitbox::of(&nif);
                anyhow::Ok((h, notes, nif.stopped_at))
            }));
            let hitbox = match made {
                Ok(Ok((h, notes, stopped))) => {
                    let dropped: usize = notes.iter().map(|n| n.triangles - n.kept).sum();
                    tracing::debug!(
                        "collision: {path}: {} drawn triangles solid, {dropped} not ({} shapes{}) in {:.1} ms",
                        h.triangles.len(),
                        notes.len(),
                        stopped.map(|k| format!(", read up to {k}")).unwrap_or_default(),
                        started.elapsed().as_secs_f64() * 1e3
                    );
                    // (Drawn, and nothing of it solid, a banner or water: an empty hitbox, which
                    // stands in for its bodies all the same. Nothing drawn: the game's shape.)
                    (!h.is_empty() || oblivion_assets::hitbox::nothing_solid(&h, &notes)).then(|| Arc::new(h))
                }
                Ok(Err(e)) => {
                    tracing::info!("collision: {path}: {e:#}");
                    None
                }
                Err(_) => {
                    tracing::warn!("collision: {path}: working out its hitbox failed");
                    None
                }
            };
            with_hitboxes(|h| {
                h.triangles += hitbox.as_ref().map_or(0, |b| b.triangles.len());
                h.done.insert(path, (hitbox, now));
                // (Over the budget: the models asked for longest ago are forgotten.)
                if h.triangles > HITBOX_CACHE {
                    let mut by_age: Vec<(f64, String)> = h.done.iter().map(|(k, v)| (v.1, k.clone())).collect();
                    by_age.sort_by(|a, b| a.0.total_cmp(&b.0));
                    for (_, k) in by_age {
                        if h.triangles <= HITBOX_CACHE * 3 / 4 {
                            break;
                        }
                        if let Some((Some(b), _)) = h.done.remove(&k) {
                            h.triangles -= b.triangles.len();
                        }
                    }
                }
            });
        }
        with_hitboxes(|h| h.busy = false);
    }

    /// Once per game frame in the world: keeps the runtime's collision around the player current.
    pub fn update(now: f64) {
        let player = engine::player();
        let Some(feet) = (player != 0).then(|| mem::at::<[f32; 3]>(player + refr::POS)).flatten() else { return };
        let feet = camera::basis().point_from_game(Vec3::from(feet));
        let mut st = STATE.lock().unwrap();
        let s = st.get_or_insert_with(|| State {
            world: 0,
            bodies: HashMap::new(),
            last_scan: f64::MIN,
            sent_at: None,
            connections: u32::MAX,
            logged_world: false,
            owners: HashMap::new(),
            claimed: HashMap::new(),
            plain: HashMap::new(),
            bodies_added: 0,
            next_id: DRAWN_ID_BASE,
            covered: Vec::new(),
            to_remove: Vec::new(),
            logged_owners: 0,
        });
        let mut changed = false;
        let conns = crate::link::with_adapter(|a| a.connections()).unwrap_or(0);
        if conns != s.connections {
            // (A restarted runtime knows nothing yet.)
            s.connections = conns;
            s.sent_at = None;
            for o in s.owners.values_mut() {
                o.sent = None;
            }
            s.to_remove.clear();
        }
        // The world's fixed bodies and the references holding them: looked at every second (cells
        // load and unload as the player goes).
        if now - s.last_scan > 1.0 {
            s.last_scan = now;
            let world = world().unwrap_or(0);
            if world != s.world {
                tracing::info!("collision: the player's Havok world is now {world:#x}");
                s.world = world;
                s.bodies.clear();
                s.logged_world = false;
                let owners: Vec<Owner> = s.owners.drain().map(|(_, o)| o).collect();
                for o in owners {
                    forget(s, o);
                }
                s.claimed.clear();
                s.plain.clear();
                changed = true;
            }
            if world != 0 {
                let fixed = fixed_bodies(world);
                let before = s.bodies.len();
                s.bodies.retain(|e, b| fixed.get(e).is_some_and(|f| f.shape == b.fixed.shape && f.xf == b.fixed.xf));
                changed |= s.bodies.len() != before;
                let mut layers: HashMap<u32, u32> = HashMap::new();
                let mut added = false;
                for (e, f) in fixed {
                    *layers.entry(f.layer).or_default() += 1;
                    if s.bodies.contains_key(&e) || !solid_layer(f.layer) {
                        continue;
                    }
                    let Some((lo, hi)) = aabb(f.shape, &f.xf) else { continue };
                    let (a, b) = (to_runtime(lo), to_runtime(hi));
                    s.bodies.insert(e, Body { fixed: f, lo: a.min(b), hi: a.max(b), tris: None, mats: Vec::new() });
                    added = true;
                }
                changed |= added;
                // (New bodies may be under references looked through before they had them.)
                if added {
                    s.bodies_added = s.bodies_added.wrapping_add(1);
                }
                // A body gone, its reference is looked through again.
                let gone: Vec<usize> = s.owners.iter().filter(|(_, o)| o.bodies.iter().any(|e| !s.bodies.contains_key(e))).map(|(r, _)| *r).collect();
                for r in gone {
                    if let Some(o) = s.owners.remove(&r) {
                        forget(s, o);
                    }
                }
                look_round(s, feet);
                if !s.logged_world {
                    s.logged_world = true;
                    let mut l: Vec<_> = layers.into_iter().collect();
                    l.sort();
                    tracing::info!("collision: {} solid fixed bodies in the world; layers (layer, bodies) {l:?}; {} references near hold {} of them", s.bodies.len(), s.owners.len(), s.claimed.len());
                }
            }
        }
        if s.world == 0 {
            return;
        }
        let mut unknown: HashMap<i32, u32> = HashMap::new();
        let mut made = 0usize;
        // The drawn shapes: the references near, their hitboxes asked for, placed and sent, the
        // nearest first.
        let mut near: Vec<(f32, usize)> = s.owners.iter().map(|(r, o)| (o.gap(feet), *r)).filter(|(g, _)| *g <= RADIUS + RESEND).collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut budget = SEND_PER_FRAME;
        let mut total = 0usize;
        let mut pending = false;
        let mut sent_now = 0usize;
        for &(gap, r) in &near {
            let Some(o) = s.owners.get_mut(&r) else { continue };
            if o.hitbox.is_none() {
                o.hitbox = hitbox(&o.model, now);
                pending |= o.hitbox.is_none();
            }
            let Some(h) = o.drawn().cloned() else { continue };
            let at = placement(o.root);
            let wanted = total + h.triangles.len() <= MAX_DRAWN_TRIANGLES && (gap <= RADIUS || o.sent.is_some());
            if o.sent.is_some() && (o.sent != at || !wanted) {
                // (Moved, or no longer wanted: out, and in again where it is now.)
                s.to_remove.push(o.id);
                o.sent = None;
            }
            if o.sent.is_some() {
                total += o.triangles;
                continue;
            }
            let Some(at) = at.filter(|_| wanted) else { continue };
            if h.is_empty() {
                // (Nothing solid: no mesh, its bodies out of the game's all the same.)
                o.id = 0;
                o.sent = Some(at);
                o.triangles = 0;
                continue;
            }
            if h.triangles.len() > budget && budget < SEND_PER_FRAME {
                pending = true;
                continue;
            }
            // Its bodies' triangles, for the materials.
            let (mut from, mut mats) = (Vec::new(), Vec::new());
            for e in &o.bodies {
                if let Some(b) = s.bodies.get_mut(e) {
                    made += b.read(&mut unknown);
                    from.extend_from_slice(b.tris.as_deref().unwrap_or_default());
                    mats.extend_from_slice(&b.mats);
                }
            }
            let tris = place(&h, &at);
            let materials = nearest_materials(&tris, &from, &mats, rustvr_protocol::MAT_STONE as u8);
            let id = s.next_id;
            if !send_mesh(id, &tris, &materials) {
                pending = true;
                break;
            }
            s.next_id = s.next_id.wrapping_add(1).max(DRAWN_ID_BASE);
            o.id = id;
            o.sent = Some(at);
            o.triangles = tris.len();
            total += tris.len();
            budget = budget.saturating_sub(tris.len());
            sent_now += 1;
            if s.logged_owners < 12 {
                s.logged_owners += 1;
                // The drawn shape against the game's where they're placed: they should overlap.
                let (dlo, dhi) = tris.iter().flatten().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
                tracing::info!(
                    "collision: {} ({} bodies, {} of the game's triangles) drawn as {} triangles; the drawn box {dlo:.2?}..{dhi:.2?}, the game's {:.2?}..{:.2?}",
                    o.model,
                    o.bodies.len(),
                    from.len(),
                    tris.len(),
                    o.lo,
                    o.hi
                );
            }
        }
        // The far ones out.
        for o in s.owners.values_mut() {
            if o.sent.is_some() && o.gap(feet) > RADIUS + RESEND {
                s.to_remove.push(o.id);
                o.sent = None;
            }
        }
        while let Some(&id) = s.to_remove.last() {
            if id != 0 && !send_mesh(id, &[], &[]) {
                pending = true;
                break;
            }
            s.to_remove.pop();
        }
        // The game's own shapes: the bodies near that no drawn shape stands in for, sent again once
        // the drawn ones are all in (not every frame they come in).
        let mut covered: Vec<usize> = s.owners.values().filter(|o| o.sent.is_some()).flat_map(|o| o.bodies.iter().copied()).collect();
        covered.sort_unstable();
        if !pending && covered != s.covered {
            changed = true;
        }
        if !changed && s.sent_at.is_some_and(|p| p.distance(feet) < RESEND) {
            return;
        }
        let reach = RADIUS + RESEND;
        for b in s.bodies.values_mut() {
            if b.gap(feet) <= reach {
                made += b.read(&mut unknown);
            }
        }
        if !unknown.is_empty() {
            tracing::info!("collision: shapes of kinds it doesn't read (kind, count): {unknown:?}");
        }
        let game = nearby(
            s.bodies
                .iter()
                .filter(|(e, b)| b.gap(feet) <= RADIUS && covered.binary_search(e).is_err())
                .filter_map(|(_, b)| Some((b.tris.as_deref()?, b.mats.as_slice()))),
            feet,
            RADIUS,
            MAX_TRIANGLES,
        );
        let tris: Vec<[Vec3; 3]> = game.iter().map(|(t, _)| *t).collect();
        let mats: Vec<u8> = game.iter().map(|(_, m)| *m).collect();
        if send_mesh(MESH_ID, &tris, &mats) {
            s.sent_at = Some(feet);
            s.covered = covered;
            let drawn: Vec<&Owner> = s.owners.values().filter(|o| o.sent.is_some()).collect();
            let keeps_game = s.owners.values().filter(|o| o.gap(feet) <= RADIUS && matches!(o.hitbox, Some(None))).count();
            let mut kinds: HashMap<u8, usize> = HashMap::new();
            for m in &mats {
                *kinds.entry(*m).or_default() += 1;
            }
            tracing::info!(
                "collision: around {feet:.2?}: {} references drawn ({} triangles, {sent_now} sent now), {keeps_game} near keep the game's shape; the game's shapes {} triangles ({made} newly read; materials {kinds:?}); the floor under the feet is at y {:?}",
                drawn.len(),
                drawn.iter().map(|o| o.triangles).sum::<usize>(),
                tris.len(),
                floor_under(&tris, feet)
            );
        }
    }

    /// One collision mesh and its materials to the runtime (no triangles: taken out).
    fn send_mesh(id: u32, tris: &[[Vec3; 3]], mats: &[u8]) -> bool {
        let head = rustvr_protocol::ColMesh { id, count: tris.len() as u32 };
        let mut payload = bytemuck::bytes_of(&head).to_vec();
        for t in tris {
            let v: [f32; 9] = [t[0].x, t[0].y, t[0].z, t[1].x, t[1].y, t[1].z, t[2].x, t[2].y, t[2].z];
            payload.extend_from_slice(bytemuck::cast_slice(&v));
        }
        crate::link::with_adapter(|a| {
            if !a.push_collision(rustvr_protocol::COL_MESH, &payload) {
                return false;
            }
            if !tris.is_empty() {
                let mut materials = bytemuck::bytes_of(&head).to_vec();
                materials.extend_from_slice(mats);
                // (Its triangles are in; without their materials they're stone.)
                let _ = a.push_collision(rustvr_protocol::COL_MESH_MATERIALS, &materials);
            }
            true
        })
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cube_from_its_planes() {
        let corners: Vec<Vec3> = (0..8).map(|i| Vec3::new(if i & 1 == 0 { -1.0 } else { 1.0 }, if i & 2 == 0 { -1.0 } else { 1.0 }, if i & 4 == 0 { -1.0 } else { 1.0 })).collect();
        let planes = [[1., 0., 0., -1.], [-1., 0., 0., -1.], [0., 1., 0., -1.], [0., -1., 0., -1.], [0., 0., 1., -1.], [0., 0., -1., -1.]];
        let tris = hull_triangles(&corners, &planes);
        assert_eq!(tris.len(), 12);
        // Every face's triangles lie on its plane.
        for t in &tris {
            assert!(t.iter().all(|v| v.abs().max_element() > 0.999));
        }
    }

    #[test]
    fn a_capsule_reaches_past_its_ends() {
        let tris = prism_triangles(Vec3::ZERO, Vec3::Z, 0.5, true, 8);
        assert_eq!(tris.len(), 32);
        let top = tris.iter().flatten().fold(f32::MIN, |m, v| m.max(v.z));
        assert!((top - 1.5).abs() < 1e-5);
    }

    #[test]
    fn transforms_compose() {
        let a = Xf { r: Mat3::from_rotation_z(std::f32::consts::FRAC_PI_2), t: Vec3::new(1.0, 0.0, 0.0) };
        let b = Xf { r: Mat3::IDENTITY, t: Vec3::new(0.0, 2.0, 0.0) };
        let p = a.of(&b).apply(Vec3::X);
        assert!(p.distance(a.apply(b.apply(Vec3::X))) < 1e-5);
    }
}
