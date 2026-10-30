//! Oblivion's alchemy apparatus for the hands: the mortar and pestle, which the game models as one
//! shape (the pestle lying in the bowl), split into its two pieces, and both measured, so the
//! runtime can hold the pestle in a hand and grind in the bowl
//! (`rustvr_protocol::stations`).
//!
//! The pieces are the model's connected parts (triangles sharing corners, the corners welded where
//! the texture's seams split them): the mortar is the biggest, the pestle the longest of the rest.
//! Both stay in the model's own space (the runtime's axes, metres: [`crate::export`]), so the
//! mortar is drawn where the game's object is and the pestle where it lies in it until a hand takes
//! it. Measured on the mortar: its inside floor's middle (the highest point facing up near its
//! axis), its rim (its top), the radius inside its mouth (the nearest its wall comes to the axis
//! near the rim), its wall and its bottom. On the pestle: its axis (its longest reach), its two
//! ends, which is the tip (the end that lies lower, in the bowl), how thick it is there and at its
//! handle.
//!
//! `oblivion-extract bench <prefix>...` writes each model's pieces into the cache (`#mortar`,
//! `#pestle`) and the measurements into `bench.toml` beside the index, which the adapter reads.

use std::collections::HashMap;

use glam::{Quat, Vec3};
use rustvr_asset::{Mesh, Model};
use serde::{Deserialize, Serialize};

/// A mortar and pestle as the hands use it (the model's space: the runtime's axes, metres).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Bench {
    /// The model's path under `meshes\` (lowercase, backslashes), and its pieces' cache keys.
    pub model: String,
    pub mortar: String,
    pub pestle: String,
    /// The bowl: the middle of its inside floor, the radius inside its mouth, how deep it is from
    /// the floor to the rim, its wall's thickness, and its outside's lowest point (along +Y).
    pub floor: [f32; 3],
    pub radius: f32,
    pub depth: f32,
    pub wall: f32,
    pub bottom: f32,
    /// The pestle's frame where it lies in the bowl (its middle, and the rotation whose +Y runs from
    /// its tip to its handle), its half length, how thick it is (its radius) and its handle's.
    pub pestle_pos: [f32; 3],
    pub pestle_rot: [f32; 4],
    pub pestle_half: f32,
    pub pestle_radius: f32,
    pub handle_radius: f32,
    /// Its tip's middle (the pestle's frame) and how thick it is there.
    pub tip: [f32; 3],
    pub tip_radius: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Benches {
    pub benches: Vec<Bench>,
}

pub const FILE: &str = "bench.toml";

impl Benches {
    pub fn load(dir: &std::path::Path) -> Option<Benches> {
        toml::from_str(&std::fs::read_to_string(dir.join(FILE)).ok()?).ok()
    }

    /// The bench made from a model (its path as a form gives it).
    pub fn of(&self, model: &str) -> Option<&Bench> {
        let path = crate::export::normalise(model, "meshes\\");
        self.benches.iter().find(|b| b.model == path)
    }
}

/// Each triangle's connected part (the same number for triangles sharing a corner, corners welded
/// at a tenth of a millimetre), over all the model's meshes, numbered from 0; and how many there are.
pub fn parts(model: &Model) -> (Vec<Vec<usize>>, usize) {
    let weld = |p: [f32; 3]| p.map(|v| (v * 10_000.0).round() as i64);
    let mut ids: HashMap<[i64; 3], usize> = HashMap::new();
    let mut tris: Vec<[usize; 3]> = Vec::new();
    for m in &model.meshes {
        for t in m.indices.chunks_exact(3) {
            let mut c = [0; 3];
            for (k, &i) in t.iter().enumerate() {
                let n = ids.len();
                c[k] = *ids.entry(weld(m.vertices[i as usize].pos)).or_insert(n);
            }
            tris.push(c);
        }
    }
    let mut root: Vec<usize> = (0..ids.len()).collect();
    fn find(root: &mut [usize], mut i: usize) -> usize {
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    for t in &tris {
        let (a, b, c) = (find(&mut root, t[0]), find(&mut root, t[1]), find(&mut root, t[2]));
        root[b] = a;
        let c2 = find(&mut root, c);
        root[c2] = a;
    }
    let mut number: HashMap<usize, usize> = HashMap::new();
    let mut out = Vec::new();
    let mut k = 0;
    for m in &model.meshes {
        let mut per = Vec::new();
        for _ in m.indices.chunks_exact(3) {
            let r = find(&mut root, tris[k][0]);
            let n = number.len();
            per.push(*number.entry(r).or_insert(n));
            k += 1;
        }
        out.push(per);
    }
    (out, number.len())
}

/// The model with only the triangles of part `keep` (`parts` as [`parts`] numbers them).
pub fn piece(model: &Model, parts: &[Vec<usize>], keep: usize) -> Model {
    let meshes: Vec<Mesh> = model
        .meshes
        .iter()
        .zip(parts)
        .filter_map(|(m, p)| {
            let indices: Vec<u32> = m.indices.chunks_exact(3).zip(p).filter(|(_, part)| **part == keep).flat_map(|(t, _)| t.iter().copied()).collect();
            (!indices.is_empty()).then(|| Mesh { indices, ..m.clone() })
        })
        .collect();
    let used: Vec<Vec3> = meshes.iter().flat_map(|m| m.indices.iter().map(|&i| Vec3::from_array(m.vertices[i as usize].pos))).collect();
    let lo = used.iter().copied().reduce(Vec3::min).unwrap_or_default();
    let hi = used.iter().copied().reduce(Vec3::max).unwrap_or_default();
    Model { meshes, bounds_min: lo.to_array(), bounds_max: hi.to_array(), ..model.clone() }
}

/// A piece's corners (each with its normal).
fn corners(m: &Model) -> Vec<(Vec3, Vec3)> {
    m.meshes.iter().flat_map(|mesh| mesh.indices.iter().map(|&i| (Vec3::from_array(mesh.vertices[i as usize].pos), Vec3::from_array(mesh.vertices[i as usize].normal)))).collect()
}

/// Splits a mortar and pestle's model and measures both: None if it has no second piece long
/// enough to be a pestle (a plain bowl).
pub fn measure(model: &Model, path: &str) -> Option<(Bench, Model, Model)> {
    let (p, n) = parts(model);
    if n < 2 {
        return None;
    }
    let pieces: Vec<Model> = (0..n).map(|k| piece(model, &p, k)).collect();
    let size = |m: &Model| Vec3::from_array(m.bounds_max) - Vec3::from_array(m.bounds_min);
    let volume = |m: &Model| {
        let s = size(m);
        s.x * s.y * s.z
    };
    let mortar = (0..n).max_by(|&a, &b| volume(&pieces[a]).total_cmp(&volume(&pieces[b])))?;
    // The pestle: the longest of the rest for its thickness.
    let long = |m: &Model| {
        let pts: Vec<Vec3> = corners(m).into_iter().map(|c| c.0).collect();
        let (a, b) = farthest(&pts);
        a.distance(b)
    };
    let pestle = (0..n).filter(|&k| k != mortar).max_by(|&a, &b| long(&pieces[a]).total_cmp(&long(&pieces[b])))?;
    // (The rest, if any, go with the mortar: its decoration.)
    let rest: Vec<usize> = (0..n).filter(|&k| k != pestle).collect();
    let mortar_model = {
        let keep: Vec<Vec<usize>> = p.iter().map(|m| m.iter().map(|&k| if rest.contains(&k) { usize::MAX } else { k }).collect()).collect();
        piece(model, &keep, usize::MAX)
    };
    let pestle_model = pieces[pestle].clone();

    // The bowl, about the mortar's own axis (+Y through the middle of its box).
    let mc = corners(&mortar_model);
    let (lo, hi) = (Vec3::from_array(mortar_model.bounds_min), Vec3::from_array(mortar_model.bounds_max));
    let mid = (lo + hi) * 0.5;
    let across = |p: Vec3| Vec3::new(p.x - mid.x, 0.0, p.z - mid.z).length();
    let outer = mc.iter().map(|c| across(c.0)).fold(0.0f32, f32::max);
    let rim = hi.y;
    let floor_y = mc.iter().filter(|(q, nrm)| nrm.y > 0.6 && across(*q) < outer * 0.5 && q.y < rim - 0.004).map(|c| c.0.y).fold(f32::MIN, f32::max);
    if floor_y == f32::MIN {
        return None;
    }
    let depth = rim - floor_y;
    let radius = mc.iter().filter(|c| c.0.y > rim - depth * 0.3 && c.1.y < 0.9).map(|c| across(c.0)).fold(outer, f32::min);
    let floor = Vec3::new(mid.x, floor_y, mid.z);

    // The pestle: its longest reach runs along it, the lower end its tip. (Its two farthest
    // corners are on its surface, across from each other: its middle line runs between the
    // middles of its two ends.)
    let pts: Vec<Vec3> = corners(&pestle_model).into_iter().map(|c| c.0).collect();
    let (a, b) = farthest(&pts);
    let (a, b) = if a.y <= b.y { (a, b) } else { (b, a) };
    let rough = (b - a).normalize_or(Vec3::Y);
    let reach = a.distance(b).max(1e-4);
    let end = |from: f32, to: f32| {
        let near: Vec<Vec3> = pts.iter().copied().filter(|q| (from..=to).contains(&((*q - a).dot(rough) / reach))).collect();
        near.iter().copied().sum::<Vec3>() / near.len().max(1) as f32
    };
    let (tip_mid, handle_mid) = (end(-0.1, 0.06), end(0.94, 1.1));
    let axis = (handle_mid - tip_mid).normalize_or(rough);
    let lo_along = pts.iter().map(|q| (*q - tip_mid).dot(axis)).fold(f32::MAX, f32::min);
    let hi_along = pts.iter().map(|q| (*q - tip_mid).dot(axis)).fold(f32::MIN, f32::max);
    let (tip_end, handle_end) = (tip_mid + axis * lo_along, tip_mid + axis * hi_along);
    let length = tip_end.distance(handle_end);
    let off = |q: Vec3| {
        let d = q - tip_end;
        (d - axis * d.dot(axis)).length()
    };
    let along = |q: Vec3| (q - tip_end).dot(axis) / length.max(1e-4);
    let thick = |from: f32, to: f32| pts.iter().filter(|q| (from..=to).contains(&along(**q))).map(|q| off(*q)).fold(0.0f32, f32::max);
    let tip_radius = thick(0.0, 0.15).max(0.003);
    let handle_radius = thick(0.6, 1.0).max(0.003);
    let radius_all = thick(0.0, 1.0).max(0.004);
    let centre = (tip_end + handle_end) * 0.5;
    let rot = Quat::from_rotation_arc(Vec3::Y, axis);
    let half = length * 0.5;
    let tip = Vec3::new(0.0, -half + tip_radius, 0.0);
    let model_name = crate::export::normalise(path, "meshes\\");
    let bench = Bench {
        model: model_name,
        mortar: crate::export::variant_key(path, "mortar"),
        pestle: crate::export::variant_key(path, "pestle"),
        floor: floor.to_array(),
        radius,
        depth,
        wall: (outer - radius).max(0.004),
        bottom: lo.y,
        pestle_pos: centre.to_array(),
        pestle_rot: rot.to_array(),
        pestle_half: half,
        pestle_radius: radius_all,
        handle_radius,
        tip: tip.to_array(),
        tip_radius,
    };
    Some((bench, mortar_model, pestle_model))
}

/// The two of `points` farthest apart (near enough: from the farthest from any, its farthest).
fn farthest(points: &[Vec3]) -> (Vec3, Vec3) {
    let Some(&first) = points.first() else { return (Vec3::ZERO, Vec3::ZERO) };
    let far = |from: Vec3| points.iter().copied().max_by(|p, q| p.distance_squared(from).total_cmp(&q.distance_squared(from))).unwrap_or(from);
    let a = far(first);
    (a, far(a))
}
