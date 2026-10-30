//! A horse's saddle, measured on the game's own model so the hands find it where it is: its pommel
//! (the horn at its front), its seat and its cantle (the back), in the frame of the horse's bone it
//! rides on. Oblivion's saddle (`meshes\creatures\horse\saddle.nif`) is skinned to the horse's
//! spine and made in its skeleton's bind pose, so its vertices are already where they sit on that
//! skeleton; measured there and put in its spine bone's frame, they go wherever the live horse's
//! spine is. `oblivion-extract export meshes/creatures/horse` writes them into
//! `local_assets/oblivion/horse.toml` ([`FILE`]); the adapter reads them from there.
//!
//! Oblivion's horses have a bridle and bit but no reins, so the runtime draws them, and these say
//! how: the bit's rings on the head, the neck and head as cross-sections (each an ellipse round the
//! top of the neck, on its bone, for the reins to lie on rather than pass through), and the leather
//! (the saddle's material: `Tack.dds`, which the bridle wears too, and the stretch of it that is
//! the bridle's plain strap).

use crate::nif::{Block, Nif};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SADDLE: &str = "meshes\\creatures\\horse\\saddle.nif";
pub const BODY: &str = "meshes\\creatures\\horse\\horse.nif";
pub const HEAD: &str = "meshes\\creatures\\horse\\horsehead.nif";
pub const TACK: &str = "textures\\creatures\\horse\\tack.dds";
pub const SKELETON: &str = "meshes\\creatures\\horse\\skeleton.nif";
/// The bone the saddle sits on: the middle of the three spine bones it's skinned to.
pub const BONE: &str = "Bip02 Spine2";
/// The bone the head (and its bridle) rides on.
pub const HEAD_BONE: &str = "Bip02 Head";
/// The neck's bones from the withers up (the head's bone comes after the last).
pub const NECK_BONES: [&str; 5] = ["Bip02 Neck", "Bip02 Neck1", "Bip02 Neck2", "Bip02 Neck3", "Bip02 Neck4"];
/// The head's cross-sections: this far along it from its bone towards the muzzle (game units).
const HEAD_ALONG: [f32; 4] = [1.0, 10.0, 19.0, 28.0];
/// A cross-section is the model's vertices this near its plane (either side, game units), its
/// ellipse fitted to the top of it this deep (the neck's, the head's: as deep as reins go on them).
const SLAB: f32 = 2.5;
const NECK_DEPTH: f32 = 16.0;
const HEAD_DEPTH: f32 = 24.0;
/// The bridle's plain strap in `Tack.dds`: its cheek piece, a straight stretch of the leather
/// between its keeper and its buckle (texels 79 to 103 across, 236 to 466 along, of 512; read off
/// the texture). The reins are cut from it.
const STRAP_UV: [f32; 4] = [79.0 / 512.0, 236.0 / 512.0, 103.0 / 512.0, 466.0 / 512.0];
pub const FILE: &str = "horse.toml";

/// Vertices this near the saddle's middle (game units either side) are its middle line.
const MIDDLE: f32 = 2.0;
/// A vertex is on the top of the middle line when nothing within this (along it) is higher.
const TOP_SPAN: f32 = 1.5;
/// The cantle is at least this far behind the pommel (game units).
const CANTLE_BEHIND: f32 = 8.0;
/// The horse's body beside the seat: this far along it either way (game units).
const BESIDE: f32 = 12.0;
/// A rider's knee and ankle below the seat (game units: a thigh dropped forward and a whole leg,
/// at the game's 70 to the metre), and a leg's half thickness off the horse's side. The game poses
/// its rider only in third person (the rider's tracks in the horse's own animations), so the legs
/// go round the horse's own body at a rider's proportions.
const KNEE_BELOW: f32 = 30.0;
const KNEE_AHEAD: f32 = 8.0;
const ANKLE_BELOW: f32 = 62.0;
const LEG: f32 = 5.0;
/// The bit sits in the corners of the mouth: this far behind the muzzle's tip (game units), at this
/// share of the muzzle's depth down from the top of the nose, its rings this far out from the face.
/// Oblivion's horses wear a bridle and bit but no reins; the reins drawn in the rider's hands run
/// from here.
const MOUTH_BEHIND: f32 = 6.0;
const MOUTH_DOWN: f32 = 0.7;
const BIT_RING: f32 = 1.5;

/// The saddle's points in its bone's frame (game units).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Saddle {
    /// The bone (of the horse's skeleton) the points are in the frame of.
    pub bone: String,
    /// The top of the pommel, the lowest point of the seat along the middle, the top of the cantle.
    pub pommel: [f32; 3],
    pub seat: [f32; 3],
    pub cantle: [f32; 3],
    /// Where a rider's left knee and ankle go, round the horse's body (the right's mirror them).
    pub knee: [f32; 3],
    pub ankle: [f32; 3],
    /// The bit's left ring (the right's mirrors it) in the frame of the head's bone (`head_bone`),
    /// where the reins run from.
    pub head_bone: String,
    pub bit: [f32; 3],
    /// The tack's leather (its texture's own average, linear RGB): the reins' colour.
    pub tack: [f32; 3],
    /// The model whose (first mesh's) material is the tack's leather (the saddle's: its texture,
    /// normal map, glossiness), and the stretch of its texture the reins are cut from (u0, v0, u1,
    /// v1).
    #[serde(default)]
    pub tack_model: String,
    #[serde(default)]
    pub strap_uv: [f32; 4],
    /// The neck and head from the withers to the muzzle, as cross-sections the reins lie on.
    #[serde(default)]
    pub neck: Vec<Section>,
}

/// A cross-section of the neck or head on one of its bones (in that bone's frame, game units): an
/// ellipse round its top, at `centre`, as wide either way as `side` is long and as high either way
/// as `up` (its top on the crest).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub bone: String,
    pub centre: [f32; 3],
    pub side: [f32; 3],
    pub up: [f32; 3],
}

/// Measures the saddle model (`saddle`) as it sits on the horse's skeleton (`skeleton`). Along its
/// middle line (forward is +Y, up +Z, left -X): the pommel is the highest point, the cantle the
/// highest behind it, the seat the lowest point of the top between them. And the horse's body
/// (`body`) beside the seat: how wide it is at a rider's knee, and at their ankle (below its belly,
/// as wide as its belly's lowest). The head (`head`, in its bone's frame: +X to the muzzle, +Y down
/// to the jaw, Z across): the corners of the mouth, where the bit is. The tack's texture (`tack`):
/// its smallest mip, its own average colour.
pub fn measure(saddle: &Nif, skeleton: &Nif, body: &Nif, head: &Nif, tack: Option<&rustvr_asset::Texture>) -> Option<Saddle> {
    let middle: Vec<Vec3> = vertices(saddle).into_iter().filter(|p| p.x.abs() < MIDDLE).collect();
    let top: Vec<Vec3> = middle.iter().copied().filter(|v| !middle.iter().any(|o| (o.y - v.y).abs() < TOP_SPAN && o.z > v.z + 1e-3)).collect();
    let pommel = top.iter().copied().max_by(|a, b| a.z.total_cmp(&b.z))?;
    let cantle = top.iter().copied().filter(|v| v.y < pommel.y - CANTLE_BEHIND).max_by(|a, b| a.z.total_cmp(&b.z))?;
    let seat = top.iter().copied().filter(|v| v.y > cantle.y && v.y < pommel.y).min_by(|a, b| a.z.total_cmp(&b.z))?;
    let beside: Vec<Vec3> = vertices(body).into_iter().filter(|v| (v.y - seat.y).abs() < BESIDE).collect();
    let belly = beside.iter().map(|v| v.z).min_by(f32::total_cmp)?;
    let flank = |z: f32| {
        let at = |z: f32| beside.iter().filter(|v| (v.z - z).abs() < 4.0).map(|v| v.x.abs()).max_by(f32::total_cmp);
        at(z).or_else(|| at(belly + 4.0)).unwrap_or(0.0) + LEG
    };
    let knee = Vec3::new(-flank(seat.z - KNEE_BELOW), seat.y + KNEE_AHEAD, seat.z - KNEE_BELOW);
    let ankle = Vec3::new(-flank(seat.z - ANKLE_BELOW), seat.y, seat.z - ANKLE_BELOW);
    let face = vertices(head);
    let tip = face.iter().map(|v| v.x).max_by(f32::total_cmp)?;
    let corner: Vec<Vec3> = face.iter().copied().filter(|v| (v.x - (tip - MOUTH_BEHIND)).abs() < 1.5).collect();
    let (nose, jaw) = (corner.iter().map(|v| v.y).min_by(f32::total_cmp)?, corner.iter().map(|v| v.y).max_by(f32::total_cmp)?);
    let mouth = nose + (jaw - nose) * MOUTH_DOWN;
    let cheek = corner.iter().filter(|v| (v.y - mouth).abs() < 2.5).map(|v| v.z.abs()).max_by(f32::total_cmp)?;
    let bit = [tip - MOUTH_BEHIND, mouth, cheek + BIT_RING];
    let bone = skeleton.find(BONE)?;
    let to_bone = skeleton.world(bone, &skeleton.parents()).inverse();
    let local = |p: Vec3| to_bone.apply(p).to_array();
    // The neck: a cross-section across each of its bones (its axis from the bone before to the
    // one after), the body's vertices there in it (+X the horse's right, up the model's up off the
    // axis); and the head's along its bone (+X to the muzzle, +Y down, Z across).
    let parents = skeleton.parents();
    let chain: Vec<i32> = NECK_BONES.iter().chain([&HEAD_BONE]).map(|n| skeleton.find(n)).collect::<Option<_>>()?;
    let at: Vec<Vec3> = chain.iter().map(|&b| skeleton.world(b, &parents).translation).collect();
    let mut neck = Vec::new();
    let body_vertices = vertices(body);
    for (i, name) in NECK_BONES.iter().enumerate() {
        let axis = (at[i + 1] - at[i.saturating_sub(1)]).normalize();
        let up = (Vec3::Z - axis * axis.dot(Vec3::Z)).normalize();
        let slab: Vec<(f32, f32)> = body_vertices.iter().filter(|v| (**v - at[i]).dot(axis).abs() < SLAB).map(|v| ((*v - at[i]).x, (*v - at[i]).dot(up))).collect();
        let Some((crest, half_w, half_h)) = fit_top(&slab, NECK_DEPTH) else { continue };
        let to_bone = skeleton.world(chain[i], &parents).inverse();
        let centre = at[i] + up * (crest - half_h);
        let vector = |v: Vec3| (to_bone.apply(centre + v) - to_bone.apply(centre)).to_array();
        neck.push(Section { bone: (*name).into(), centre: to_bone.apply(centre).to_array(), side: vector(Vec3::X * half_w), up: vector(up * half_h) });
    }
    for x in HEAD_ALONG {
        let slab: Vec<(f32, f32)> = face.iter().filter(|v| (v.x - x).abs() < SLAB).map(|v| (v.z, -v.y)).collect();
        let Some((crest, half_w, half_h)) = fit_top(&slab, HEAD_DEPTH) else { continue };
        neck.push(Section { bone: HEAD_BONE.into(), centre: [x, -(crest - half_h), 0.0], side: [0.0, 0.0, half_w], up: [0.0, -half_h, 0.0] });
    }
    Some(Saddle {
        bone: BONE.into(),
        pommel: local(pommel),
        seat: local(seat),
        cantle: local(cantle),
        knee: local(knee),
        ankle: local(ankle),
        head_bone: HEAD_BONE.into(),
        bit,
        tack: tack.and_then(average).unwrap_or([0.1, 0.06, 0.03]),
        tack_model: crate::export::model_key(SADDLE),
        strap_uv: STRAP_UV,
        neck,
    })
}

/// The ellipse round the top of a cross-section (its points: across, up), fitted to its outline
/// down to `depth` below its crest: the crest, and the ellipse's half width and half height (its
/// middle that far below the crest). Least squares over a grid, each point's distance across from
/// the ellipse at its height (none below the ellipse).
fn fit_top(points: &[(f32, f32)], depth: f32) -> Option<(f32, f32, f32)> {
    let crest = points.iter().map(|p| p.1).max_by(f32::total_cmp)?;
    let near: Vec<(f32, f32)> = points.iter().filter(|p| p.1 >= crest - depth).map(|p| (p.0.abs(), crest - p.1)).collect();
    if near.len() < 4 {
        return None;
    }
    let mut best = (f32::MAX, 0.0, 0.0);
    for hh in (6..=48).map(|k| k as f32 * 0.5) {
        for hw in (4..=100).map(|k| k as f32 * 0.25) {
            let err: f32 = near
                .iter()
                .map(|&(across, down)| {
                    let y = (down - hh) / hh;
                    let w = hw * (1.0 - y * y).max(0.0).sqrt();
                    (w - across).powi(2)
                })
                .sum();
            if err < best.0 {
                best = (err, hw, hh);
            }
        }
    }
    Some((crest, best.1, best.2))
}

/// A texture's own average colour (linear RGB): its smallest mip, one block-compressed colour block.
fn average(t: &rustvr_asset::Texture) -> Option<[f32; 3]> {
    use rustvr_asset::TexFormat;
    let last = t.mips.last()?;
    // (BC2 and BC3 keep their alpha block first.)
    let block = match t.format {
        TexFormat::Bc1 => last.get(0..8)?,
        TexFormat::Bc2 | TexFormat::Bc3 => last.get(8..16)?,
        TexFormat::Rgba8 => return Some([0, 1, 2].map(|i| srgb_to_linear(*last.get(i).unwrap_or(&0) as f32 / 255.0))),
        _ => return None,
    };
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);
    let rgb = |c: u16| [((c >> 11) & 31) as f32 / 31.0, ((c >> 5) & 63) as f32 / 63.0, (c & 31) as f32 / 31.0];
    // The first texel's colour of the four the block's endpoints give.
    let (a, b) = (rgb(c0), rgb(c1));
    let colour = match block[4] & 3 {
        0 => a,
        1 => b,
        2 if c0 > c1 => [0, 1, 2].map(|i| (2.0 * a[i] + b[i]) / 3.0),
        3 if c0 > c1 => [0, 1, 2].map(|i| (a[i] + 2.0 * b[i]) / 3.0),
        _ => [0, 1, 2].map(|i| (a[i] + b[i]) / 2.0),
    };
    Some(colour.map(srgb_to_linear))
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// A model's vertices where it's drawn (skinned ones in their bind pose).
fn vertices(nif: &Nif) -> Vec<Vec3> {
    let parents = nif.parents();
    let mut out = Vec::new();
    for (i, b) in nif.blocks.iter().enumerate() {
        let Block::Shape { data, .. } = b else { continue };
        let Some(Block::ShapeData(d)) = nif.get(*data) else { continue };
        let w = nif.world(i as i32, &parents);
        out.extend(d.vertices.iter().map(|v| w.apply(*v)));
    }
    out
}

pub fn save(root: &Path, s: &Saddle) -> anyhow::Result<()> {
    let text = format!(
        "# A horse's saddle, measured on {SADDLE} where it sits on {SKELETON}: the top of its pommel, the\n\
         # lowest point of its seat along its middle and the top of its cantle, and where a rider's left knee\n\
         # and ankle go round the horse's body ({BODY}), in the frame of the bone it rides on (game units);\n\
         # the bit's left ring on its head ({HEAD}) in its head bone's frame, and its tack's colour ({TACK});\n\
         # the tack's leather (the saddle's material) and the bridle's strap in its texture, which the reins are cut\n\
         # from; and the neck and head as cross-sections round their top, on their bones, for the reins to lie on.\n\
         # Written by `oblivion-extract export meshes/creatures/horse`.\n{}",
        toml::to_string(s)?
    );
    std::fs::write(root.join(FILE), text)?;
    Ok(())
}

pub fn load(root: &Path) -> Option<Saddle> {
    toml::from_str(&std::fs::read_to_string(root.join(FILE)).ok()?).ok()
}
