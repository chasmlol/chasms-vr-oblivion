//! The shape of a model the player's hands meet: what the game draws of it, not its collision.
//!
//! Oblivion's collision is a stand-in for most of its models: a box round a chair, a hull round a
//! table, a slab for a wall and its mouldings. That's right for walking and for things falling,
//! and wrong for a hand reaching between a chair's rungs, into a shelf or along a wall. A model's
//! hitbox here is its drawn triangles, in the model's own frame (game units, its root node's
//! space: the reference's 3D puts it in the world), less what isn't solid:
//!
//! * what's never drawn: hidden nodes, the editor's markers, collision-only geometry
//!   (`RootCollisionNode`), particles;
//! * what's drawn but isn't a thing: sprites turned to the camera (`NiBillboardNode`), glows,
//!   flames and light shafts (added onto the picture, not blended over it);
//! * what's soft or not there to touch, by its texture ([`soft`]): water and lava, the sky,
//!   clouds, mist and the shadows and fades painted on the air, blood and stains painted on a
//!   surface, hanging cloth (banners, tapestries, curtains, flags, sails), cobwebs, vines and
//!   ivy, and (cut out or see-through) leaves, grass, flowers and moss;
//! * what the model's own animations move (a waterwheel's wheel, a bridge that falls): drawn
//!   where it is now, not where the model has it;
//! * the parts of a cut-out (alpha-tested) shape clear of the model's own collision: a cut-out is
//!   a flat card whose holes are in its texture, so it's solid only where the game made it so (a
//!   grate's slab, a railing's), never round a shrub's stem.
//!
//! Everything else drawn is solid, whether or not the game gave that part collision of its own:
//! the game's collision leaves out much that's drawn (iron struts, bars, posts, statues' arms)
//! that a hand should still stop on. A model the game gave no collision at all is never the
//! hands' (the adapter only stands a drawn shape in for the game's collision). [`collision`] is
//! the model's own collision, in the same frame, for the cut-outs and the checks.

use crate::nif::{Block, Nif, Transform};
use glam::{Mat3, Quat, Vec3};
use std::collections::HashMap;

/// Game units per Havok unit.
pub use crate::nif::HAVOK_SCALE;

/// How far (game units) a drawn triangle may be from the model's own collision and still be on
/// it: a wall's drawn face and its collision's are a hair apart, a cut-out grate's bars inside
/// its slab.
pub const NEAR: f32 = 8.0;

/// Bethesda's collision layers (the low 7 bits of a body's filter info) that are solid to what
/// the hands hold: the world's statics, the land, trees, furniture, stairs. Water, triggers,
/// portals, the characters' own capsules and the pick-only layers are not.
pub fn solid_layer(layer: u32) -> bool {
    matches!(layer, 0 | 1 | 2 | 3 | 4 | 9 | 10 | 13 | 14 | 17 | 19)
}

/// A model's hitbox: triangles in its root node's frame, game units.
#[derive(Clone, Debug, Default)]
pub struct Hitbox {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    /// The root node's own transform in the file (the model's space as the file has it is this
    /// after the hitbox's).
    pub root: Transform,
}

impl Hitbox {
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        bounds(self.vertices.iter().copied())
    }

    /// A triangle, its corners shared with the triangles already in that have them (`seen`: each
    /// corner's index by its bits).
    fn push(&mut self, t: [Vec3; 3], seen: &mut HashMap<[u32; 3], u32>) {
        let tri = t.map(|v| {
            *seen.entry(v.to_array().map(f32::to_bits)).or_insert_with(|| {
                self.vertices.push(v);
                self.vertices.len() as u32 - 1
            })
        });
        self.triangles.push(tri);
    }
}

fn bounds(points: impl Iterator<Item = Vec3>) -> Option<(Vec3, Vec3)> {
    points.fold(None, |b, p| Some(b.map_or((p, p), |(lo, hi): (Vec3, Vec3)| (lo.min(p), hi.max(p)))))
}

/// What became of one drawn shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Fate {
    /// Solid, all of it.
    Kept,
    /// A cut-out shape: its parts on the model's collision solid, the rest not.
    KeptOnCollision,
    Hidden,
    Marker,
    Billboard,
    /// Added onto the picture (a glow, a light shaft, a flame), or an effect's.
    Glow,
    /// Water, lava.
    Water,
    /// The sky, clouds, mist, smoke, and shadows and fades painted on the air.
    Air,
    /// Blood or a stain painted on a surface.
    Decal,
    /// Hanging cloth: a banner, a tapestry, a curtain, a flag, a sail.
    Cloth,
    /// A plant's soft parts (leaves, grass, flowers, vines, ivy, moss), or a cobweb.
    Plant,
    /// A cut-out shape clear of the model's own collision.
    OffCollision,
    /// A skinned shape whose bones the model doesn't have.
    Unplaced,
    /// A triangle or two far from the rest of the model: a stand-in that makes its bounds bigger
    /// (a castle's piece has one far underground).
    Stray,
    /// Moved by the model's own animations.
    Moving,
}

impl Fate {
    /// Drawn, and not solid: soft, or not a thing (not hidden, nor a helper never drawn).
    pub fn drawn_soft(self) -> bool {
        matches!(self, Fate::Billboard | Fate::Glow | Fate::Water | Fate::Air | Fate::Decal | Fate::Cloth | Fate::Plant | Fate::OffCollision)
    }
}

/// Whether a model draws something but nothing solid (a banner, a sail, water): the hands meet
/// nothing there. A model that draws nothing at all (a collision stand-in for something drawn
/// elsewhere: a unique tree's trunk) keeps the game's shape.
pub fn nothing_solid(hitbox: &Hitbox, notes: &[ShapeNote]) -> bool {
    hitbox.is_empty() && notes.iter().any(|n| n.fate.drawn_soft())
}

/// One drawn shape and what became of it (for the checks).
#[derive(Clone, Debug)]
pub struct ShapeNote {
    pub name: String,
    pub texture: String,
    pub triangles: usize,
    pub kept: usize,
    pub fate: Fate,
    /// The share of its area on the model's own collision (1 when the model has none).
    pub on_collision: f32,
    pub alpha: u16,
}

// ---- the model's own collision --------------------------------------------------------------------

/// A convex solid: its faces' planes (outward `n`, `n·p + w = 0` on the face), game units.
#[derive(Clone, Debug)]
struct Convex {
    planes: Vec<(Vec3, f32)>,
    lo: Vec3,
    hi: Vec3,
}

/// A round solid: the points within `r` of the segment `a`-`b`.
#[derive(Clone, Copy, Debug)]
struct Round {
    a: Vec3,
    b: Vec3,
    r: f32,
}

/// The model's own collision in its root node's frame (game units): the solid layers' shapes.
#[derive(Clone, Debug, Default)]
pub struct Collision {
    convex: Vec<Convex>,
    round: Vec<Round>,
    /// The triangle meshes' triangles, and a grid over them.
    pub triangles: Vec<[Vec3; 3]>,
    grid: HashMap<[i32; 3], Vec<u32>>,
    /// The triangles too big for the grid (a floor, a wall), with their boxes.
    big: Vec<(u32, Vec3, Vec3)>,
    /// Every shape as triangles (round ones as prisms), for drawing and the checks.
    pub surface: Vec<[Vec3; 3]>,
    /// Kinds of shape it had that aren't read (block kind, count).
    pub unread: HashMap<String, u32>,
}

/// The grid's cell (game units): a few times [`NEAR`].
const CELL: f32 = 32.0;

fn cell(p: Vec3) -> [i32; 3] {
    let c = (p / CELL).floor();
    [c.x as i32, c.y as i32, c.z as i32]
}

impl Collision {
    /// Triangles alone as a collision (a hitbox, to measure the game's against).
    pub fn mesh(tris: &[[Vec3; 3]]) -> Collision {
        let mut c = Collision::default();
        for t in tris {
            c.add_triangle(*t);
        }
        c
    }

    pub fn is_empty(&self) -> bool {
        self.convex.is_empty() && self.round.is_empty() && self.triangles.is_empty()
    }

    /// Whether `p` is inside the collision or within `d` of it.
    pub fn near(&self, p: Vec3, d: f32) -> bool {
        let inside = |c: &Convex| p.cmpge(c.lo - d).all() && p.cmple(c.hi + d).all() && c.planes.iter().all(|(n, w)| n.dot(p) + w <= d);
        if self.convex.iter().any(inside) {
            return true;
        }
        if self.round.iter().any(|r| segment_distance(p, r.a, r.b) <= r.r + d) {
            return true;
        }
        if self.big.iter().any(|&(i, lo, hi)| p.cmpge(lo - d).all() && p.cmple(hi + d).all() && triangle_distance(p, &self.triangles[i as usize]) <= d) {
            return true;
        }
        let (lo, hi) = (cell(p - d), cell(p + d));
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    if let Some(ts) = self.grid.get(&[x, y, z])
                        && ts.iter().any(|&i| triangle_distance(p, &self.triangles[i as usize]) <= d)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn add_triangle(&mut self, t: [Vec3; 3]) {
        if !(t[0].is_finite() && t[1].is_finite() && t[2].is_finite()) {
            return;
        }
        let i = self.triangles.len() as u32;
        let (min, max) = (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]));
        let (lo, hi) = (cell(min), cell(max));
        let cells = (0..3).map(|k| (hi[k] as i64 - lo[k] as i64 + 1).max(1)).product::<i64>();
        if cells > 64 {
            self.big.push((i, min, max));
            self.triangles.push(t);
            self.surface.push(t);
            return;
        }
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    self.grid.entry([x, y, z]).or_default().push(i);
                }
            }
        }
        self.triangles.push(t);
        self.surface.push(t);
    }

    fn add_convex(&mut self, corners: &[Vec3], planes: Vec<(Vec3, f32)>) {
        let Some((lo, hi)) = bounds(corners.iter().copied()) else { return };
        self.surface.extend(hull_faces(corners, &planes));
        self.convex.push(Convex { planes, lo, hi });
    }

    fn add_round(&mut self, a: Vec3, b: Vec3, r: f32) {
        self.surface.extend(prism(a, b, r, 12));
        self.round.push(Round { a, b, r });
    }
}

/// Distance from `p` to the segment `a`-`b`.
pub fn segment_distance(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 0.0 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(a + ab * t)
}

/// Distance from `p` to the triangle `t`.
pub fn triangle_distance(p: Vec3, t: &[Vec3; 3]) -> f32 {
    let (a, b, c) = (t[0], t[1], t[2]);
    let n = (b - a).cross(c - a);
    if n.length_squared() > 1e-12 {
        let n = n.normalize();
        let q = p - n * n.dot(p - a);
        // (Inside all three edges: the plane's distance.)
        let inside = [(a, b), (b, c), (c, a)].iter().all(|&(u, v)| (v - u).cross(q - u).dot(n) >= 0.0);
        if inside {
            return n.dot(p - a).abs();
        }
    }
    segment_distance(p, a, b).min(segment_distance(p, b, c)).min(segment_distance(p, c, a))
}

/// A convex solid's faces from its corners and planes: each plane's corners in order round it.
fn hull_faces(corners: &[Vec3], planes: &[(Vec3, f32)]) -> Vec<[Vec3; 3]> {
    let size = corners.iter().fold(0.0f32, |m, c| m.max(c.abs().max_element()));
    let eps = (size * 1e-3).max(1e-3);
    let mut out = Vec::new();
    for &(n, w) in planes {
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

/// A round shape round `a`-`b` of radius `r`, as a `sides`-sided prism with flat ends (for drawing).
fn prism(a: Vec3, b: Vec3, r: f32, sides: usize) -> Vec<[Vec3; 3]> {
    let axis = (b - a).try_normalize().unwrap_or(Vec3::Z);
    let (a, b) = (a - axis * r, b + axis * r);
    let u = axis.any_orthonormal_vector();
    let v = axis.cross(u);
    let ring = |c: Vec3| -> Vec<Vec3> { (0..sides).map(|i| (i as f32 / sides as f32 * std::f32::consts::TAU, c)).map(|(t, c)| c + (u * t.cos() + v * t.sin()) * r).collect() };
    let (ra, rb) = (ring(a), ring(b));
    let mut out = Vec::new();
    for i in 0..sides {
        let j = (i + 1) % sides;
        out.extend([[ra[i], ra[j], rb[j]], [ra[i], rb[j], rb[i]], [a, ra[j], ra[i]], [b, rb[i], rb[j]]]);
    }
    out
}

/// A node's transform into the model's root frame (through its parents, not the root's own).
fn in_root(nif: &Nif, i: i32, parents: &[i32]) -> Transform {
    let mut t = Transform::IDENTITY;
    let mut at = i;
    while at >= 0 {
        let p = parents.get(at as usize).copied().unwrap_or(-1);
        if p < 0 {
            break; // (the root: the reference places it)
        }
        if let Some(a) = nif.av(at) {
            t = t.then(&a.transform);
        }
        at = p;
    }
    t
}

/// `t` applied to a plane `(n, w)` (`n·p + w = 0`) of a shape in its own frame.
fn plane_to(t: &Transform, n: Vec3, w: f32) -> (Vec3, f32) {
    let n2 = (t.rotation * n).normalize_or_zero();
    (n2, t.scale * w - n2.dot(t.translation))
}

/// The model's own collision, in its root node's frame.
pub fn collision(nif: &Nif) -> Collision {
    let parents = nif.parents();
    let mut out = Collision::default();
    for b in &nif.blocks {
        let Block::CollisionObject { target, body } = b else { continue };
        let Some(Block::RigidBody(rb)) = nif.get(*body) else { continue };
        if !solid_layer(rb.layer as u32 & 0x7F) {
            continue;
        }
        let node = in_root(nif, *target, &parents);
        // A transformed body (`bhkRigidBodyT`) is placed in its node; Havok's unit is 7 game units.
        let mut at = Transform { rotation: Mat3::IDENTITY, translation: Vec3::ZERO, scale: HAVOK_SCALE };
        if rb.transformed {
            let q = Quat::from_xyzw(rb.rotation[0], rb.rotation[1], rb.rotation[2], rb.rotation[3]);
            let r = if q.is_finite() && q.length() > 0.5 { Mat3::from_quat(q.normalize()) } else { Mat3::IDENTITY };
            at = Transform { rotation: r, translation: Vec3::new(rb.translation[0], rb.translation[1], rb.translation[2]) * HAVOK_SCALE, scale: HAVOK_SCALE };
        }
        shape(nif, rb.shape, &at.then(&node), &mut out, 0);
    }
    out
}

/// The shape `i` placed by `t` (its own frame, Havok units, to the root's, game units).
fn shape(nif: &Nif, i: i32, t: &Transform, out: &mut Collision, depth: u32) {
    if depth > 16 {
        return;
    }
    let s = t.scale;
    match nif.get(i) {
        Some(Block::BoxShape { dims, radius }) => {
            let h = *dims + Vec3::splat(*radius);
            let corners: Vec<Vec3> = (0..8).map(|k| t.apply(Vec3::new(if k & 1 == 0 { -h.x } else { h.x }, if k & 2 == 0 { -h.y } else { h.y }, if k & 4 == 0 { -h.z } else { h.z }))).collect();
            let planes = [(Vec3::X, h.x), (Vec3::NEG_X, h.x), (Vec3::Y, h.y), (Vec3::NEG_Y, h.y), (Vec3::Z, h.z), (Vec3::NEG_Z, h.z)].iter().map(|&(n, e)| plane_to(t, n, -e)).collect();
            out.add_convex(&corners, planes);
        }
        Some(Block::SphereShape { radius }) => out.add_round(t.apply(Vec3::ZERO), t.apply(Vec3::ZERO), radius * s),
        Some(Block::CapsuleShape { a, b, radius }) => out.add_round(t.apply(*a), t.apply(*b), radius * s),
        Some(Block::MultiSphere { spheres }) => {
            for &(c, r) in spheres {
                out.add_round(t.apply(c), t.apply(c), r * s);
            }
        }
        Some(Block::ConvexVertices { vertices, radius, planes }) => {
            let corners: Vec<Vec3> = vertices.iter().map(|v| t.apply(*v)).collect();
            // (A plane cutting through the hull's own corners is left out: a few of the game's flat
            // hulls have them.)
            let size = corners.iter().fold(0.0f32, |m, c| m.max(c.abs().max_element()));
            let planes: Vec<(Vec3, f32)> = planes
                .iter()
                .map(|p| plane_to(t, Vec3::new(p[0], p[1], p[2]), p[3] - radius))
                .filter(|(n, w)| *n != Vec3::ZERO && corners.iter().all(|c| n.dot(*c) + w <= 0.01 * size.max(1.0)))
                .collect();
            if planes.is_empty() {
                // (No faces: the box round its corners.)
                if let Some((lo, hi)) = bounds(corners.iter().copied()) {
                    let box_planes = vec![(Vec3::X, -hi.x), (Vec3::NEG_X, lo.x), (Vec3::Y, -hi.y), (Vec3::NEG_Y, lo.y), (Vec3::Z, -hi.z), (Vec3::NEG_Z, lo.z)];
                    let box_corners: Vec<Vec3> = (0..8).map(|k| Vec3::new(if k & 1 == 0 { lo.x } else { hi.x }, if k & 2 == 0 { lo.y } else { hi.y }, if k & 4 == 0 { lo.z } else { hi.z })).collect();
                    out.add_convex(&box_corners, box_planes);
                }
            } else {
                out.add_convex(&corners, planes);
            }
        }
        Some(Block::ListShape { shapes }) | Some(Block::ConvexList { shapes }) => {
            for &c in shapes {
                shape(nif, c, t, out, depth + 1);
            }
        }
        Some(Block::ConvexTransform { shape: c, transform }) => {
            // (Havok's transform: the rotation's columns, then the translation, four floats each.)
            let m = transform;
            let r = Mat3::from_cols(Vec3::new(m[0], m[1], m[2]), Vec3::new(m[4], m[5], m[6]), Vec3::new(m[8], m[9], m[10]));
            let inner = Transform { rotation: r, translation: Vec3::new(m[12], m[13], m[14]), scale: 1.0 };
            shape(nif, *c, &inner.then(t), out, depth + 1);
        }
        Some(Block::Mopp { shape: c }) => shape(nif, *c, t, out, depth + 1),
        Some(Block::PackedStrips { parts, data }) => {
            let Some(Block::PackedStripsData { triangles, vertices }) = nif.get(*data) else { return };
            // Each part is the next run of vertices; a triangle is its first vertex's part's.
            let mut part_of = Vec::with_capacity(vertices.len());
            for (k, p) in parts.iter().enumerate() {
                part_of.extend(std::iter::repeat_n(k, p.vertices as usize));
            }
            for tri in triangles {
                let solid = part_of.get(tri[0] as usize).and_then(|&k| parts.get(k)).is_none_or(|p| solid_layer(p.layer as u32 & 0x7F));
                if let (true, Some(a), Some(b), Some(c)) = (solid, vertices.get(tri[0] as usize), vertices.get(tri[1] as usize), vertices.get(tri[2] as usize)) {
                    out.add_triangle([t.apply(*a), t.apply(*b), t.apply(*c)]);
                }
            }
        }
        Some(Block::TriStripsShape { strips }) => {
            // (Its triangles are in game units, not Havok's.)
            let at = |v: &Vec3| t.apply(*v / HAVOK_SCALE);
            for &d in strips {
                if let Some(Block::ShapeData(sd)) = nif.get(d) {
                    for tri in &sd.triangles {
                        if let (Some(a), Some(b), Some(c)) = (sd.vertices.get(tri[0] as usize), sd.vertices.get(tri[1] as usize), sd.vertices.get(tri[2] as usize)) {
                            out.add_triangle([at(a), at(b), at(c)]);
                        }
                    }
                }
            }
        }
        Some(_) => *out.unread.entry(nif.kinds.get(i as usize).cloned().unwrap_or_default()).or_default() += 1,
        None => {}
    }
}

// ---- what's drawn -----------------------------------------------------------------------------------

/// `NiAlphaProperty`'s flags: blending on, its destination factor (bits 5..8; 0 is "one": added
/// onto what's behind), testing on.
const ALPHA_BLEND: u16 = 1;
const ALPHA_TEST: u16 = 1 << 9;
const DEST_SHIFT: u16 = 5;
const DEST_ONE: u16 = 0;

/// What the drawing state is at a node: its alpha flags, its base texture and its material's
/// opacity, inherited down.
#[derive(Clone, Default)]
struct Look {
    alpha: Option<u16>,
    texture: String,
    opacity: Option<f32>,
}

fn look_of(nif: &Nif, props: &[i32], mut look: Look) -> Look {
    for &p in props {
        match nif.get(p) {
            Some(Block::Alpha { flags, .. }) => look.alpha = Some(*flags),
            Some(Block::Material(m)) => look.opacity = Some(m.alpha),
            Some(Block::Texturing(t)) => {
                if let Some(Block::SourceTexture { file }) = nif.get(t.base) {
                    look.texture = file.to_ascii_lowercase().replace('/', "\\");
                }
            }
            _ => {}
        }
    }
    look
}

/// What a texture says isn't solid: always (`see_through` false), or also when the shape is cut out
/// or see-through (a leaf, a blade of grass and a bloodstain are cards; a mossy stone or a grassy
/// bank drawn solid is a thing).
pub fn soft(texture: &str, see_through: bool) -> Option<Fate> {
    let t = texture.to_ascii_lowercase().replace('/', "\\");
    let file = t.rsplit('\\').next().unwrap_or(&t);
    let any = |words: &[&str]| words.iter().any(|w| file.contains(w));
    if t.contains("\\water\\") || file.starts_with("water") || file.starts_with("lava0") || any(&["landscapewater", "cisternwater", "maniwater", "palacewater", "dunxwater", "waterfall", "sewage"]) {
        return Some(Fate::Water);
    }
    if any(&["skydome", "skyfalse", "cellsky", "oblivion_sky", "skystar", "skynebula", "cloudlayer", "atmospherecloud", "cloudsclear", "cloudscloudy", "redcloud", "gascloud", "mist", "fog", "smoke", "shadefade", "shadow"]) {
        return Some(Fate::Air);
    }
    if t.contains("\\fx\\") || t.contains("\\effects\\") || any(&["lightray", "lightbeam", "godray"]) {
        return Some(Fate::Glow);
    }
    if any(&["tapestry", "banner", "flag0", "flags0", "pirateflag", "shipflag", "curtain", "sail0", "sailripped", "tatteredrobe"]) {
        return Some(Fate::Cloth);
    }
    if any(&["web0", "cobweb"]) {
        return Some(Fate::Plant);
    }
    if !see_through {
        return None;
    }
    // (Not a sign's "Divine Elegance", nor a vineyard's.)
    if any(&["ivy"]) || (file.contains("vine") && !any(&["divine", "vineyard"])) {
        return Some(Fate::Plant);
    }
    if any(&["blood", "stain", "decal", "splat"]) {
        return Some(Fate::Decal);
    }
    if any(&["fire", "flame", "ember", "spark", "glow"]) {
        return Some(Fate::Glow);
    }
    let plants = [
        "leaf", "leaves", "grass", "fern", "flora", "crop", "groundcover", "weed", "moss", "hang", "seaweed", "kelp", "lily", "daisy", "poppy", "heather", "gorse", "sage", "cattail", "thistle", "fungus", "roots", "hair",
        "butterfly", "moth", "plant", "flower", "nirnroot", "strawberrytop", "arrowroot",
    ];
    any(&plants).then_some(Fate::Plant)
}

/// A triangle or two at least this far (game units) from the rest of a model is a stray.
const STRAY: f32 = 256.0;

/// The nodes the model's own animations move (by name, lower case): sampled along each, those that
/// turn or slide at all.
fn moving_nodes(nif: &Nif) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for s in nif.sequences() {
        let (t0, t1) = nif.sequence_span(s);
        let first = nif.sequence_pose(s, t0);
        for k in 1..=8 {
            let at = nif.sequence_pose(s, t0 + (t1 - t0) * k as f32 / 8.0);
            for ((name, a), (_, b)) in first.iter().zip(&at) {
                let turned = (a.rotation - b.rotation).to_cols_array().iter().any(|v| v.abs() > 0.01);
                if turned || a.translation.distance(b.translation) > 0.5 || (a.scale - b.scale).abs() > 0.01 {
                    out.insert(name.to_ascii_lowercase());
                }
            }
        }
    }
    out
}

/// The model's hitbox, and what became of each drawn shape.
pub fn of(nif: &Nif) -> (Hitbox, Vec<ShapeNote>) {
    let own = collision(nif);
    drawn(nif, &own, false, None, &[])
}

/// The model's hitbox without what's under the nodes named in `swung` (lower case): the parts the
/// game's physics swings on what holds still (a chain's links under its ring on the wall), which
/// are shapes of their own where the game has them (`of_part`).
pub fn of_without(nif: &Nif, swung: &[&str]) -> (Hitbox, Vec<ShapeNote>) {
    let own = collision(nif);
    drawn(nif, &own, false, None, swung)
}

/// The model's hitbox against its collision `own` (see [`collision`]); `measure`: each shape's
/// share on that collision worked out (for the checks: only a cut-out's is needed for the hitbox).
pub fn of_with(nif: &Nif, own: &Collision, measure: bool) -> (Hitbox, Vec<ShapeNote>) {
    drawn(nif, own, measure, None, &[])
}

/// The hitbox of one part of a model, in that part's own frame: what's drawn under the node named
/// `node` (case-insensitive), not under a node below it with collision of its own (another part:
/// the next link of a chain). The part moves as one (it's what the game swings), so nothing in it
/// is left out for moving.
pub fn of_part(nif: &Nif, node: &str) -> Option<(Hitbox, Vec<ShapeNote>)> {
    let i = nif.find(node)?;
    let own = collision(nif);
    Some(drawn(nif, &own, false, Some(i), &[]))
}

fn drawn(nif: &Nif, own: &Collision, measure: bool, part: Option<i32>, swung: &[&str]) -> (Hitbox, Vec<ShapeNote>) {
    let parents = nif.parents();
    let mut notes = Vec::new();
    // Each solid shape's triangles (by its note), joined once the strays are out.
    let mut pieces: Vec<(usize, Vec<[Vec3; 3]>)> = Vec::new();
    let moving: std::collections::HashSet<String> = match part {
        Some(_) => Default::default(),
        None => moving_nodes(nif).into_iter().chain(swung.iter().map(|n| n.to_ascii_lowercase())).collect(),
    };
    // (A part's triangles in its own frame.)
    let to_part = part.map(|i| in_root(nif, i, &parents).inverse());
    let mut stack: Vec<(i32, Look, Option<Fate>)> = match part {
        Some(i) => vec![(i, Look::default(), None)],
        None => nif.roots.iter().map(|&r| (r, Look::default(), None)).collect(),
    };
    if stack.is_empty() && !nif.blocks.is_empty() {
        stack.push((0, Look::default(), None));
    }
    let mut seen = vec![false; nif.blocks.len()];
    while let Some((i, look, dropped)) = stack.pop() {
        if i < 0 || (i as usize) >= nif.blocks.len() || std::mem::replace(&mut seen[i as usize], true) {
            continue;
        }
        let kind = nif.kinds.get(i as usize).map(String::as_str).unwrap_or("");
        match &nif.blocks[i as usize] {
            Block::Node { av, children } => {
                if part.is_some_and(|p| p != i) && av.collision >= 0 {
                    continue;
                }
                let name = av.name.to_ascii_lowercase();
                let dropped = dropped
                    .or((av.flags & 1 != 0).then_some(Fate::Hidden))
                    .or((kind == "RootCollisionNode" || name.starts_with("editormarker") || name == "marker" || name.starts_with("marker:")).then_some(Fate::Marker))
                    .or((kind == "NiBillboardNode" || kind == "NiBSParticleNode").then_some(Fate::Billboard))
                    .or(moving.contains(&name).then_some(Fate::Moving));
                let look = look_of(nif, &av.properties, look);
                stack.extend(children.iter().map(|&c| (c, look.clone(), dropped)));
            }
            Block::Shape { av, data, skin } => {
                let Some(Block::ShapeData(d)) = nif.get(*data) else { continue };
                let look = look_of(nif, &av.properties, look);
                let alpha = look.alpha.unwrap_or(0);
                let blended = alpha & ALPHA_BLEND != 0;
                let cut_out = alpha & ALPHA_TEST != 0;
                // A shape that's only a helper: the bounds' stand-in far below a castle's piece
                // (`SecretBigger`), a particle emitter's shape, a collision stand-in.
                let name = av.name.to_ascii_lowercase();
                let helper = name.starts_with("editormarker") || name.starts_with("secretbigger") || (look.texture.is_empty() && ["collision", "emitter", "emmiter", "marker"].iter().any(|w| name.contains(w)));
                let fate = dropped
                    .or((av.flags & 1 != 0 || look.opacity.is_some_and(|a| a < 0.01)).then_some(Fate::Hidden))
                    .or(moving.contains(&name).then_some(Fate::Moving))
                    .or(helper.then_some(Fate::Marker))
                    .or((blended && !cut_out && (alpha >> DEST_SHIFT) & 0xF == DEST_ONE).then_some(Fate::Glow))
                    .or(soft(&look.texture, blended || cut_out));
                let mut note = ShapeNote { name: av.name.clone(), texture: look.texture.clone(), triangles: d.triangles.len(), kept: 0, fate: Fate::Kept, on_collision: 1.0, alpha };
                if let Some(f) = fate {
                    note.fate = f;
                    notes.push(note);
                    continue;
                }
                // The shape's triangles in the root's frame: skinned, by its bones' bind poses.
                let Some(mut points) = placed(nif, i, d, *skin, &parents) else {
                    note.fate = Fate::Unplaced;
                    notes.push(note);
                    continue;
                };
                if let Some(t) = &to_part {
                    points.iter_mut().for_each(|p| *p = t.apply(*p));
                }
                let tris: Vec<[Vec3; 3]> = d
                    .triangles
                    .iter()
                    .filter_map(|t| Some([*points.get(t[0] as usize)?, *points.get(t[1] as usize)?, *points.get(t[2] as usize)?]))
                    .filter(|t| (t[1] - t[0]).cross(t[2] - t[0]).length_squared() > 1e-8)
                    .collect();
                // Which of them lie on the model's own collision (centre, or two of its corners): a
                // cut-out's solid parts, and for the checks.
                let on: Vec<bool> = if own.is_empty() || !(cut_out || measure) {
                    vec![true; tris.len()]
                } else {
                    tris.iter().map(|t| own.near((t[0] + t[1] + t[2]) / 3.0, NEAR) || t.iter().filter(|v| own.near(**v, NEAR)).count() >= 2).collect()
                };
                let area = |t: &[Vec3; 3]| (t[1] - t[0]).cross(t[2] - t[0]).length() * 0.5;
                let total: f32 = tris.iter().map(area).sum();
                let on_area: f32 = tris.iter().zip(&on).filter(|(_, o)| **o).map(|(t, _)| area(t)).sum();
                note.on_collision = if total > 0.0 { on_area / total } else { 0.0 };
                let solid: Vec<[Vec3; 3]> = if cut_out {
                    note.fate = if on.iter().any(|o| *o) { Fate::KeptOnCollision } else { Fate::OffCollision };
                    tris.iter().zip(&on).filter(|(_, o)| **o).map(|(t, _)| *t).collect()
                } else {
                    tris
                };
                note.kept = solid.len();
                if !solid.is_empty() {
                    pieces.push((notes.len(), solid));
                }
                notes.push(note);
            }
            _ => {}
        }
    }
    // The strays: a shape of a triangle or two far from all the others' bounds.
    let span = |ts: &[[Vec3; 3]]| bounds(ts.iter().flatten().copied());
    let big: Vec<(Vec3, Vec3)> = pieces.iter().filter(|(_, t)| t.len() > 2).filter_map(|(_, t)| span(t)).collect();
    let root = nif.roots.first().and_then(|&r| nif.av(r)).map_or(Transform::IDENTITY, |a| a.transform.clone());
    let mut hitbox = Hitbox { root, ..Default::default() };
    let mut seen = HashMap::new();
    for (note, tris) in pieces {
        let far = |(lo, hi): (Vec3, Vec3)| big.iter().all(|&(blo, bhi)| (blo - hi).max(lo - bhi).max(Vec3::ZERO).length() > STRAY);
        if tris.len() <= 2 && !big.is_empty() && span(&tris).is_some_and(far) {
            notes[note].fate = Fate::Stray;
            notes[note].kept = 0;
            continue;
        }
        for t in tris {
            hitbox.push(t, &mut seen);
        }
    }
    (hitbox, notes)
}

/// A drawn shape's vertices in the root's frame: rigid, by its place; skinned, each by its
/// weighted bones' bind poses (`bone · skin to bone`).
fn placed(nif: &Nif, i: i32, d: &crate::nif::ShapeData, skin: i32, parents: &[i32]) -> Option<Vec<Vec3>> {
    if skin < 0 {
        let t = in_root(nif, i, parents);
        return Some(d.vertices.iter().map(|v| t.apply(*v)).collect());
    }
    let Some(Block::SkinInstance { data, bones, .. }) = nif.get(skin) else { return None };
    let Some(Block::SkinData { bones: bone_data, .. }) = nif.get(*data) else { return None };
    let mut sum = vec![(Vec3::ZERO, 0.0f32); d.vertices.len()];
    for (&node, bd) in bones.iter().zip(bone_data) {
        nif.av(node)?;
        let to = bd.transform.then(&in_root(nif, node, parents));
        for &(v, w) in &bd.weights {
            if let (Some(p), Some(s)) = (d.vertices.get(v as usize), sum.get_mut(v as usize)) {
                s.0 += to.apply(*p) * w;
                s.1 += w;
            }
        }
    }
    sum.iter().map(|&(p, w)| (w > 0.0).then(|| p / w)).collect()
}

/// What a model is, by where the game keeps it: for the checks' tallies.
pub fn category(path: &str) -> &'static str {
    let p = path.to_ascii_lowercase().replace('/', "\\");
    let p = p.strip_prefix("meshes\\").unwrap_or(&p);
    let file = p.rsplit('\\').next().unwrap_or(p);
    let first = p.split('\\').next().unwrap_or("");
    if file.starts_with("marker") || file.starts_with("furnituremarker") || p.starts_with("landscape\\lod") {
        return "markers and distant land";
    }
    match first {
        "architecture" | "dungeons" | "oblivion" | "obelisk" | "battlehorncastle" | "vilelair" => {
            if p.contains("\\plants\\") || p.starts_with("oblivion\\plants") {
                "plants"
            } else if p.contains("\\clutter\\") || p.starts_with("oblivion\\clutter") {
                "clutter"
            } else {
                "architecture"
            }
        }
        "furniture" => "furniture",
        "clutter" if p.starts_with("clutter\\sefurniture") => "furniture",
        "clutter" => "clutter",
        "plants" => "plants",
        "trees" => "trees",
        "rocks" => "rocks",
        "landscape" => "landscape",
        "lights" => "lights",
        "effects" | "magiceffects" | "fire" | "sky" => "effects",
        "characters" | "creatures" | "armor" | "clothes" | "weapons" | "staffs" | "menus" | "idleobjects" => "not placed in the world",
        _ => "other",
    }
}
