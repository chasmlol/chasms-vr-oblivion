//! Oblivion's models in the runtime's cache format ([`rustvr_asset::Model`]).
//!
//! Space: Oblivion's model space is right-handed, Z up, Y forward, in game units (70 to the metre,
//! as the adapter maps the world); the cache's is left-handed, Y up, Z forward, in metres. A point
//! goes over as (x, z, y) / 70: the swap of Y and Z is the change of hand, so a triangle's corners
//! keep their order.
//!
//! Keys: a model's key is `m` and a hash of its path under `meshes\` ([`model_key`]), a texture's
//! `t` and a hash of its path under `textures\` ([`texture_key`]). The adapter names the model of a
//! thing the game has by the same hash of the path the game's own form gives (an object's name in
//! the protocol is short: `oblivion/m1a2b3c4d` fits where a path wouldn't).
//!
//! Skinned models (bodies, hands, armour, clothes) are put in the bind pose of the game's own
//! skeleton ([`Skeleton`], `characters\_male\skeleton.nif`) and carry all its bones in its order,
//! so the parts of a body (the race's skin, what it wears) go together by putting their meshes
//! side by side.

use std::collections::HashMap;

use glam::{Mat3, Mat4, Quat, Vec3};
use rustvr_asset::{Bone, FORMAT_VERSION, Material, Mesh, Model, Shader, Vertex};

use crate::nif::{Block, Nif, Transform};

/// Game units to the metre (the adapter's world scale).
pub const UNITS_PER_METRE: f32 = 70.0;
/// The specular power of a shape without a material (the game's own, 0x85BEA0).
pub const GLOSSINESS: f32 = 30.0;

/// The swap of Y and Z (its own inverse).
const SWAP: Mat3 = Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::Y);

/// A point in the game's model space, in the cache's.
pub fn point(p: Vec3) -> Vec3 {
    Vec3::new(p.x, p.z, p.y) / UNITS_PER_METRE
}

/// A direction in the game's model space, in the cache's.
pub fn direction(d: Vec3) -> Vec3 {
    Vec3::new(d.x, d.z, d.y).normalize_or(Vec3::Y)
}

/// A game transform as a cache matrix (column-major, metres).
pub fn matrix(t: &Transform) -> Mat4 {
    let r = SWAP * t.rotation * SWAP;
    Mat4::from_cols((r.x_axis * t.scale).extend(0.0), (r.y_axis * t.scale).extend(0.0), (r.z_axis * t.scale).extend(0.0), point(t.translation).extend(1.0))
}

fn fnv1a(s: &str) -> u32 {
    s.bytes().fold(0x811c_9dc5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193))
}

/// A game path as its key is made from: lowercase, backslashes, without the leading folder
/// (`meshes\` or `textures\`, and `data\` before it).
pub fn normalise(path: &str, folder: &str) -> String {
    let mut p = path.trim().replace('/', "\\").to_ascii_lowercase();
    while p.starts_with('\\') {
        p.remove(0);
    }
    for prefix in ["data\\", folder] {
        if let Some(rest) = p.strip_prefix(prefix) {
            p = rest.to_owned();
        }
    }
    p
}

/// The cache key of the model at `path` (under `meshes\`, as the game's forms give it).
pub fn model_key(path: &str) -> String {
    format!("m{:08x}", fnv1a(&normalise(path, "meshes\\")))
}

/// The key of a variant of the model at `path` (e.g. a weapon's scabbard alone).
pub fn variant_key(path: &str, variant: &str) -> String {
    format!("m{:08x}", fnv1a(&format!("{}#{variant}", normalise(path, "meshes\\"))))
}

/// The cache key of the texture at `path` (under `textures\`).
pub fn texture_key(path: &str) -> String {
    format!("t{:08x}", fnv1a(&normalise(path, "textures\\")))
}

/// The game's skeleton: every node of the skeleton model, parents first.
pub struct Skeleton {
    pub bones: Vec<Bone>,
    /// Each bone's bind pose in the game's model space.
    pub world: Vec<Transform>,
    by_name: HashMap<String, usize>,
}

impl Skeleton {
    pub fn from_nif(nif: &Nif) -> Skeleton {
        let mut s = Skeleton { bones: Vec::new(), world: Vec::new(), by_name: HashMap::new() };
        let mut stack: Vec<(i32, i32, Transform)> = nif.roots.iter().rev().map(|&r| (r, -1, Transform::IDENTITY)).collect();
        while let Some((i, parent, parent_world)) = stack.pop() {
            let Some(Block::Node { av, children }) = nif.get(i) else { continue };
            if nif.kinds.get(i as usize).is_some_and(|k| k == "RootCollisionNode") {
                continue;
            }
            let world = av.transform.then(&parent_world);
            let index = s.bones.len();
            let local = matrix(&av.transform);
            let (_, rot, pos) = local.to_scale_rotation_translation();
            s.bones.push(Bone {
                name: av.name.clone(),
                parent,
                local_pos: pos.to_array(),
                local_rot: rot.normalize().to_array(),
                model_matrix: matrix(&world).to_cols_array(),
            });
            s.by_name.entry(av.name.to_ascii_lowercase()).or_insert(index);
            s.world.push(world.clone());
            for &c in children.iter().rev() {
                stack.push((c, index as i32, world.clone()));
            }
        }
        s
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.by_name.get(&name.to_ascii_lowercase()).copied()
    }
}

/// What a model converts to: the model and the game paths of the textures it names (each under
/// its [`texture_key`]).
pub struct Converted {
    pub model: Model,
    pub textures: Vec<String>,
    /// Each mesh's vertices' linear map from the game's model space into the cache's (what a move
    /// of the vertex, a FaceGen morph's, becomes).
    pub linear: Vec<Vec<Mat3>>,
}

/// Which of a model's nodes to keep.
#[derive(Clone, Copy)]
pub enum Part<'a> {
    All,
    /// Everything but the node of this name and what hangs from it (a weapon without its
    /// scabbard).
    Without(&'a str),
    /// Only the node of this name and what hangs from it.
    Only(&'a str),
}

/// Converts a model. `skeleton`: put a skinned model in its bind pose with its bones (else the
/// model's own nodes are its bones). `exists`: whether the game has a file (a texture's normal
/// map, `<texture>_n.dds`).
pub fn convert(nif: &Nif, source: &str, skeleton: Option<&Skeleton>, part: Part, exists: &dyn Fn(&str) -> bool) -> Converted {
    convert_on(nif, source, skeleton, part, exists, None)
}

/// Converts a rigid part of the head (the hair, ears, eyes, mouth, teeth, tongue) as the game puts
/// it on a body: on the skeleton's bone `bone` (`Bip01 Head`), its vertices in the model's own axes
/// about that bone's place (docs/research/mirror.md), carrying the skeleton's bones.
pub fn convert_attached(nif: &Nif, source: &str, skeleton: &Skeleton, bone: &str, exists: &dyn Fn(&str) -> bool) -> Converted {
    convert_on(nif, source, Some(skeleton), Part::All, exists, skeleton.find(bone))
}

/// The hair shader's highlight table, by (N·L, N·H).
const HAIR_ANISO: &str = "textures\\effects\\aniso.dds";

/// A texture the skin shader draws: the races' skins and heads (not the eyes, nor the hair).
fn skin_texture(path: &str) -> bool {
    path.starts_with("textures\\characters\\") && !path.contains("\\hair\\") && !path.contains("\\eyes\\")
}

fn convert_on(nif: &Nif, source: &str, skeleton: Option<&Skeleton>, part: Part, exists: &dyn Fn(&str) -> bool, attach: Option<usize>) -> Converted {
    let parents = nif.parents();
    let own;
    let skinned = nif.blocks.iter().any(|b| matches!(b, Block::Shape { skin, .. } if *skin >= 0));
    let skeleton = match skeleton {
        Some(s) if skinned || attach.is_some() => Some(s),
        _ if skinned => {
            own = Skeleton::from_nif(nif);
            Some(&own)
        }
        _ => None,
    };
    let named = |i: i32, name: &str| nif.av(i).is_some_and(|a| a.name.eq_ignore_ascii_case(name));
    let under = |mut i: i32, name: &str| {
        while i >= 0 {
            if named(i, name) {
                return true;
            }
            i = parents[i as usize];
        }
        false
    };
    let hidden = |mut i: i32| {
        while i >= 0 {
            let kind = nif.kinds.get(i as usize).map(String::as_str);
            if kind == Some("RootCollisionNode") || nif.av(i).is_some_and(|a| a.flags & 1 != 0 || a.name.to_ascii_lowercase().starts_with("editormarker")) {
                return true;
            }
            i = parents[i as usize];
        }
        false
    };
    let mut meshes = Vec::new();
    let mut linear = Vec::new();
    let mut textures: Vec<String> = Vec::new();
    for (i, b) in nif.blocks.iter().enumerate() {
        let i = i as i32;
        let Block::Shape { av, data, skin } = b else { continue };
        if hidden(i) {
            continue;
        }
        match part {
            Part::All => {}
            Part::Without(n) if under(i, n) => continue,
            Part::Only(n) if !under(i, n) => continue,
            _ => {}
        }
        let Some(Block::ShapeData(d)) = nif.get(*data) else { continue };
        if d.vertices.is_empty() || d.triangles.is_empty() {
            continue;
        }
        // Each vertex's transform(s) into the model's space: skinned, the weighted bones' bind
        // poses (`bone world · skin to bone`); rigid, the shape's own place, on the bone it hangs
        // from when there's a skeleton.
        let mut weights: Vec<Vec<(usize, f32)>> = vec![Vec::new(); d.vertices.len()];
        let mut skin_to: Vec<Transform> = Vec::new();
        let mut skin_bone: Vec<usize> = Vec::new();
        if let (Some(Block::SkinInstance { data: sd, bones, .. }), Some(sk)) = (nif.get(*skin), skeleton)
            && let Some(Block::SkinData { bones: bd, .. }) = nif.get(*sd)
        {
            for (j, (&node, data)) in bones.iter().zip(bd).enumerate() {
                let name = nif.av(node).map(|a| a.name.as_str()).unwrap_or("");
                let k = sk.find(name).or_else(|| nearest_bone(nif, &parents, node, sk)).unwrap_or(0);
                skin_to.push(data.transform.then(&sk.world[k]));
                skin_bone.push(k);
                for &(v, w) in &data.weights {
                    if let Some(list) = weights.get_mut(v as usize)
                        && w > 0.0
                    {
                        list.push((j, w));
                    }
                }
            }
        }
        let mut rigid = nif.world(i, &parents);
        let mut rigid_bone = skeleton.and_then(|sk| nearest_bone(nif, &parents, i, sk));
        // A head's part: its vertices as they are, in the model's axes about its bone's place. (The
        // game's FaceGen takes a part's one shape alone: the mouth, teeth and tongue hang under a
        // node turned as the head bone is, which it doesn't apply.)
        if let (Some(k), Some(sk)) = (attach, skeleton) {
            rigid = Transform { translation: sk.world[k].translation, ..Transform::IDENTITY };
            rigid_bone = Some(k);
        }
        // The hair (its texture under `characters\hair\`): its vertex colour is its tint's mask,
        // in the green (the hair shader's `v1.y`).
        let hair = av.properties.iter().any(|&p| match nif.get(p) {
            Some(Block::Texturing(t)) => matches!(nif.get(t.base), Some(Block::SourceTexture { file }) if normalise(file, "textures\\").starts_with("characters\\hair\\")),
            _ => false,
        });
        let mut vertices = Vec::with_capacity(d.vertices.len());
        let mut maps = Vec::with_capacity(d.vertices.len());
        for (v, &p) in d.vertices.iter().enumerate() {
            let n = d.normals.get(v).copied().unwrap_or(Vec3::Z);
            // The vertex colour, which the game multiplies the base texture by: grey in every model
            // the player wears or holds, so its brightness alone.
            let ao = match d.colors.get(v) {
                Some(c) if hair => (c[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                Some(c) => (((c[0] + c[1] + c[2]) / 3.0).clamp(0.0, 1.0) * 255.0).round() as u8,
                None => 255,
            };
            let mut vert = Vertex { uv: d.uvs.get(v).copied().unwrap_or_default(), ao, ..Default::default() };
            let w = &mut weights[v];
            if w.is_empty() || skin_to.is_empty() {
                vert.pos = point(rigid.apply(p)).to_array();
                vert.normal = direction(rigid.rotation * n).to_array();
                maps.push(SWAP * rigid.rotation * (rigid.scale / UNITS_PER_METRE));
                if let Some(k) = rigid_bone {
                    vert.bones[0] = k as u8;
                    vert.weights[0] = 255;
                }
            } else {
                w.sort_by(|a, b| b.1.total_cmp(&a.1));
                w.truncate(4);
                let total: f32 = w.iter().map(|x| x.1).sum::<f32>().max(1e-6);
                let (mut pos, mut nrm, mut map) = (Vec3::ZERO, Vec3::ZERO, Mat3::ZERO);
                for &(j, wt) in w.iter() {
                    pos += skin_to[j].apply(p) * (wt / total);
                    nrm += skin_to[j].rotation * n * (wt / total);
                    map += skin_to[j].rotation * (skin_to[j].scale * wt / total);
                }
                maps.push(SWAP * map / UNITS_PER_METRE);
                vert.pos = point(pos).to_array();
                vert.normal = direction(nrm).to_array();
                // Weights in 255ths, summing to 255 exactly.
                let mut left = 255u32;
                for (s, &(j, wt)) in w.iter().enumerate() {
                    let q = if s + 1 == w.len() { left } else { ((wt / total) * 255.0).round().min(left as f32) as u32 };
                    vert.bones[s] = skin_bone[j] as u8;
                    vert.weights[s] = q as u8;
                    left -= q;
                }
            }
            vertices.push(vert);
        }
        // Gamebryo's lighting (rustvr_asset::Shader::GamebryoLit): the skin's (the parts made with
        // the races' skins, under `textures\characters\`) with its rim; the normal map
        // (`<texture>_n.dds`), the glow map, and the material's glossiness, specular, emittance and
        // alpha.
        let mut material = Material { shader: Shader::GamebryoLit, params: [GLOSSINESS, 0.0, 0.0, 0.0], color: [0.0, 0.0, 0.0, 1.0], ..Default::default() };
        let mut texture = String::new();
        let add = |path: String, textures: &mut Vec<String>| {
            let key = texture_key(&path);
            if !textures.contains(&path) {
                textures.push(path);
            }
            key
        };
        for &p in &av.properties {
            match nif.get(p) {
                Some(Block::Texturing(t)) => {
                    if let Some(Block::SourceTexture { file }) = nif.get(t.base) {
                        let path = format!("textures\\{}", normalise(file, "textures\\"));
                        if skin_texture(&path) {
                            material.shader = Shader::GamebryoSkin;
                        } else if hair {
                            material.shader = Shader::GamebryoHair;
                        }
                        let normal = path.strip_suffix(".dds").map(|s| format!("{s}_n.dds"));
                        // The hair's layer map (`<texture>_hl.dds`: its strands over the base, by
                        // its alpha), where the others have a glow map.
                        let layer = path.strip_suffix(".dds").map(|s| format!("{s}_hl.dds")).filter(|_| hair);
                        texture = add(path, &mut textures);
                        if let Some(n) = normal.filter(|n| exists(n)) {
                            material.bump = add(n, &mut textures);
                        }
                        if let Some(l) = layer.filter(|l| exists(l)) {
                            material.detail = add(l, &mut textures);
                        }
                        // The hair's highlight, from the game's table (its hair shader's AnisoMap).
                        if hair && exists(HAIR_ANISO) {
                            material.env = add(HAIR_ANISO.to_owned(), &mut textures);
                        }
                    }
                    if let Some(Block::SourceTexture { file }) = nif.get(t.glow) {
                        let path = format!("textures\\{}", normalise(file, "textures\\"));
                        if exists(&path) {
                            material.detail = add(path, &mut textures);
                        }
                    }
                }
                Some(Block::Material(m)) => {
                    material.params[0] = m.glossiness;
                    material.color = [m.emissive[0], m.emissive[1], m.emissive[2], m.alpha];
                }
                Some(Block::Specular { on }) => material.params[1] = *on as u32 as f32,
                Some(Block::Alpha { flags, .. }) => material.alpha_test = flags & 0x201 != 0,
                _ => {}
            }
        }
        let indices = d.triangles.iter().flat_map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect();
        meshes.push(Mesh { texture, vertices, indices, name: av.name.clone(), material });
        linear.push(maps);
    }
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for v in meshes.iter().flat_map(|m| &m.vertices) {
        lo = lo.min(Vec3::from_array(v.pos));
        hi = hi.max(Vec3::from_array(v.pos));
    }
    if meshes.is_empty() {
        (lo, hi) = (Vec3::ZERO, Vec3::ZERO);
    }
    let bones = skeleton.map(|s| s.bones.clone()).unwrap_or_default();
    Converted {
        model: Model { version: FORMAT_VERSION, source: source.to_owned(), meshes, bones, bounds_min: lo.to_array(), bounds_max: hi.to_array() },
        textures,
        linear,
    }
}

/// The skeleton's bone for a node: its own name's, else its nearest ancestor's.
fn nearest_bone(nif: &Nif, parents: &[i32], mut i: i32, sk: &Skeleton) -> Option<usize> {
    while i >= 0 {
        if let Some(k) = nif.av(i).and_then(|a| sk.find(&a.name)) {
            return Some(k);
        }
        i = parents[i as usize];
    }
    None
}

/// A model's node, as a transform in the cache's space (e.g. where a weapon hangs from: its `Prn`
/// extra data names the body's node).
pub fn node_matrix(nif: &Nif, name: &str) -> Option<Mat4> {
    let i = nif.find(name)?;
    Some(matrix(&nif.world(i, &nif.parents())))
}

/// The rotation part of a cache matrix, as the asset format keeps it.
pub fn rotation(m: &Mat4) -> Quat {
    let (_, r, _) = m.to_scale_rotation_translation();
    r.normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_ignore_case_slashes_and_the_folder() {
        assert_eq!(model_key("Weapons\\Iron\\LongSword.NIF"), model_key("meshes/weapons/iron/longsword.nif"));
        assert_eq!(texture_key("textures\\weapons\\IronLongsword.dds"), texture_key("Weapons\\IronLongsword.dds"));
        assert_ne!(model_key("weapons\\iron\\longsword.nif"), variant_key("weapons\\iron\\longsword.nif", "scb"));
        assert_eq!(model_key("x").len(), 9);
    }

    #[test]
    fn game_space_goes_over_y_up_z_forward_in_metres() {
        // Game forward (+Y) is the cache's +Z; game up (+Z) is +Y.
        assert!(point(Vec3::new(0.0, 70.0, 0.0)).abs_diff_eq(Vec3::Z, 1e-6));
        assert!(point(Vec3::new(0.0, 0.0, 70.0)).abs_diff_eq(Vec3::Y, 1e-6));
        // A rotation about game up (heading) is one about the cache's up.
        let t = Transform { rotation: Mat3::from_rotation_z(0.4), translation: Vec3::new(7.0, 14.0, 21.0), scale: 1.0 };
        let m = matrix(&t);
        let p = Vec3::new(1.0, 2.0, 3.0);
        assert!(m.transform_point3(point(p)).abs_diff_eq(point(t.apply(p)), 1e-5));
        assert!((m.determinant() - 1.0).abs() < 1e-5);
    }
}
