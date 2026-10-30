//! A horse's saddle, measured on the game's own model so the hands find it where it is: its pommel
//! (the horn at its front), its seat and its cantle (the back), in the frame of the horse's bone it
//! rides on. Oblivion's saddle (`meshes\creatures\horse\saddle.nif`) is skinned to the horse's
//! spine and made in its skeleton's bind pose, so its vertices are already where they sit on that
//! skeleton; measured there and put in its spine bone's frame, they go wherever the live horse's
//! spine is. `oblivion-extract export meshes/creatures/horse` writes them into
//! `local_assets/oblivion/horse.toml` ([`FILE`]); the adapter reads them from there.

use crate::nif::{Block, Nif};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SADDLE: &str = "meshes\\creatures\\horse\\saddle.nif";
pub const BODY: &str = "meshes\\creatures\\horse\\horse.nif";
pub const SKELETON: &str = "meshes\\creatures\\horse\\skeleton.nif";
/// The bone the saddle sits on: the middle of the three spine bones it's skinned to.
pub const BONE: &str = "Bip02 Spine2";
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
}

/// Measures the saddle model (`saddle`) as it sits on the horse's skeleton (`skeleton`). Along its
/// middle line (forward is +Y, up +Z, left -X): the pommel is the highest point, the cantle the
/// highest behind it, the seat the lowest point of the top between them. And the horse's body
/// (`body`) beside the seat: how wide it is at a rider's knee, and at their ankle (below its belly,
/// as wide as its belly's lowest).
pub fn measure(saddle: &Nif, skeleton: &Nif, body: &Nif) -> Option<Saddle> {
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
    let bone = skeleton.find(BONE)?;
    let to_bone = skeleton.world(bone, &skeleton.parents()).inverse();
    let local = |p: Vec3| to_bone.apply(p).to_array();
    Some(Saddle { bone: BONE.into(), pommel: local(pommel), seat: local(seat), cantle: local(cantle), knee: local(knee), ankle: local(ankle) })
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
         # and ankle go round the horse's body ({BODY}), in the frame of the bone it rides on (game units).\n\
         # Written by `oblivion-extract export meshes/creatures/horse`.\n{}",
        toml::to_string(s)?
    );
    std::fs::write(root.join(FILE), text)?;
    Ok(())
}

pub fn load(root: &Path) -> Option<Saddle> {
    toml::from_str(&std::fs::read_to_string(root.join(FILE)).ok()?).ok()
}
