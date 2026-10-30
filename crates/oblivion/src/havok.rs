//! The game's own Havok bodies of its models, read from its archives: what a weapon or a shield
//! weighs and its convex shape, as the game simulates them when they lie in the world.
//!
//! The masses are Havok's own (each mesh's `bhkRigidBody`), in the game's own proportions: its
//! people's ragdoll (`characters\_male\skeleton.nif`, its 18 bodies) weighs 605 of them. An adult
//! weighs about 75 kg, and that is the conversion: an iron longsword's 10 is 1.24 kg, a claymore's
//! 14 is 1.74 kg, a war axe's 16 is 2 kg and a warhammer's 30 is 3.7 kg, as real ones weigh. No
//! weapon is weighed by hand.

use crate::items::hull_points;
use glam::{Mat3, Quat, Vec3};
use oblivion_assets::bsa::Archives;
use oblivion_assets::export;
use oblivion_assets::nif::{Block, HAVOK_SCALE, Nif};
use rustvr_protocol::MAX_HULL_POINTS;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// An adult's weight (kg): what the game's people's ragdoll weighs.
const PERSON_KG: f32 = 75.0;
/// The skeleton whose ragdoll is the game's people.
const SKELETON: &str = "meshes\\characters\\_male\\skeleton.nif";

/// A model's Havok body: its mass (kg) and its convex parts (the model's space, metres: the
/// extracted model's).
#[derive(Clone, Debug, Default)]
pub struct Body {
    pub mass: f32,
    pub parts: Vec<Vec<Vec3>>,
}

/// The game's archives (its `Data` folder, beside the exe).
fn archives() -> Option<&'static Archives> {
    static A: OnceLock<Option<Archives>> = OnceLock::new();
    A.get_or_init(|| {
        let game = std::env::current_exe().ok()?.parent()?.to_path_buf();
        match Archives::open(&game) {
            Ok(a) => Some(a),
            Err(e) => {
                tracing::warn!("havok: the game's archives in {}: {e:#}", game.display());
                None
            }
        }
    })
    .as_ref()
}

fn read(path: &str) -> Option<Nif> {
    let data = archives()?.read(path).ok()?;
    Nif::parse(&data).ok()
}

/// The game's Havok masses to the kilogram (its people's ragdoll against an adult's weight).
fn per_kg() -> f32 {
    static K: OnceLock<f32> = OnceLock::new();
    *K.get_or_init(|| {
        let total: f32 = read(SKELETON).map(|n| n.blocks.iter().filter_map(|b| if let Block::RigidBody(r) = b { Some(r.mass) } else { None }).sum()).unwrap_or(0.0);
        let k = if total > 1.0 { total / PERSON_KG } else { 605.0 / PERSON_KG };
        tracing::info!("havok: the game's people weigh {total:.0} in its Havok masses: {k:.2} to the kilogram");
        k
    })
}

/// The Havok body of the model at `path` (as a form gives it, under `meshes\`), cached.
pub fn body(path: &str) -> Option<Body> {
    static CACHE: Mutex<Option<HashMap<String, Option<Body>>>> = Mutex::new(None);
    let key = path.to_ascii_lowercase();
    if let Some(b) = CACHE.lock().unwrap().get_or_insert_with(HashMap::new).get(&key) {
        return b.clone();
    }
    let full = if key.starts_with("meshes\\") { key.clone() } else { format!("meshes\\{key}") };
    let found = read(&full).and_then(|n| of(&n));
    match &found {
        Some(b) => tracing::info!("havok: {path}: {:.2} kg, {} convex parts", b.mass, b.parts.len()),
        None => tracing::info!("havok: {path}: no Havok body"),
    }
    CACHE.lock().unwrap().get_or_insert_with(HashMap::new).insert(key, found.clone());
    found
}

/// A model's rigid bodies, together: their masses summed, their shapes' convex parts.
fn of(nif: &Nif) -> Option<Body> {
    let parents = nif.parents();
    let mut body = Body::default();
    for b in &nif.blocks {
        let Block::CollisionObject { target, body: rb } = b else { continue };
        let Some(Block::RigidBody(r)) = nif.get(*rb) else { continue };
        body.mass += r.mass;
        // The node it's on, into the model's space (game units); the body's own place on it
        // (`bhkRigidBodyT`) before that, in Havok's units.
        let node = nif.world(*target, &parents);
        let own = |p: Vec3| {
            if r.transformed {
                let q = Quat::from_xyzw(r.rotation[0], r.rotation[1], r.rotation[2], r.rotation[3]).normalize();
                q * p + Vec3::new(r.translation[0], r.translation[1], r.translation[2])
            } else {
                p
            }
        };
        let to_model = |p: Vec3| export::point(node.apply(own(p) * HAVOK_SCALE));
        let mut parts = Vec::new();
        shape(nif, r.shape, Mat3::IDENTITY, Vec3::ZERO, 0, &mut parts);
        for part in parts {
            let points: Vec<Vec3> = part.into_iter().map(to_model).collect();
            let hull = hull_points(&points, MAX_HULL_POINTS);
            if hull.len() >= 4 {
                body.parts.push(hull);
            }
        }
    }
    (body.mass > 0.0).then(|| Body { mass: body.mass / per_kg(), ..body })
}

/// A shape's convex parts' points (Havok units, in its parent's space through `rot`, `at`).
fn shape(nif: &Nif, i: i32, rot: Mat3, at: Vec3, depth: u32, out: &mut Vec<Vec<Vec3>>) {
    if depth > 8 {
        return;
    }
    let put = |p: Vec3| at + rot * p;
    match nif.get(i) {
        Some(Block::ListShape { shapes }) => {
            for &s in shapes {
                shape(nif, s, rot, at, depth + 1, out);
            }
        }
        Some(Block::ConvexTransform { shape: s, transform: m }) => {
            // A Havok transform as stored: the rotation's columns, then the translation.
            let r = Mat3::from_cols(Vec3::new(m[0], m[1], m[2]), Vec3::new(m[4], m[5], m[6]), Vec3::new(m[8], m[9], m[10]));
            let t = Vec3::new(m[12], m[13], m[14]);
            shape(nif, *s, rot * r, at + rot * t, depth + 1, out);
        }
        Some(Block::BoxShape { dims, .. }) => {
            let d = *dims;
            out.push((0..8).map(|k| put(Vec3::new(if k & 1 == 0 { -d.x } else { d.x }, if k & 2 == 0 { -d.y } else { d.y }, if k & 4 == 0 { -d.z } else { d.z }))).collect());
        }
        Some(Block::CapsuleShape { a, b, radius }) => out.push(capsule(*a, *b, *radius).into_iter().map(put).collect()),
        Some(Block::SphereShape { radius }) => out.push(capsule(Vec3::ZERO, Vec3::ZERO, *radius).into_iter().map(put).collect()),
        Some(Block::ConvexVertices { vertices, .. }) => out.push(vertices.iter().map(|&p| put(p)).collect()),
        _ => {}
    }
}

/// A capsule's points for its hull: each end's sphere at its poles and round its middle.
fn capsule(a: Vec3, b: Vec3, r: f32) -> Vec<Vec3> {
    let axis = (b - a).normalize_or(Vec3::Y);
    let u = axis.any_orthonormal_vector();
    let v = axis.cross(u);
    let mut out = Vec::with_capacity(26);
    for (end, out_dir) in [(a, -axis), (b, axis)] {
        out.push(end + out_dir * r);
        for k in 0..8 {
            let t = k as f32 * std::f32::consts::TAU / 8.0;
            let ring = u * t.cos() + v * t.sin();
            out.push(end + ring * r);
            if k % 2 == 0 {
                out.push(end + (ring + out_dir).normalize() * r);
            }
        }
    }
    out
}
