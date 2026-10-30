//! Oblivion's own swings, from its attack animations (`meshes\characters\_male\*attack*.kf`): how
//! fast an attack moves the weapon at the moment it lands (its `Hit` text key), so a VR swing can
//! be judged against the game's own.
//!
//! An attack moves the Biped chain (`Bip01` down through the spine, the right clavicle, arm and hand
//! to the `Weapon` node, the grip): each node's place at a moment is its animation's (its keys,
//! plain or B-spline, or its constant transform) composed down the chain. Around the `Hit` key the
//! `Weapon` node's velocity and spin give every point of a weapon in it: one `d` up its blade (its
//! +Y) moves at `|v + w x (d blade)|` ([`Swing::speed_at`]).

use crate::nif::{Block, Keyframes, Nif};
use glam::{Mat3, Quat, Vec3};

/// The Biped chain from an attack animation's root to the weapon.
const CHAIN: [&str; 12] = ["Bip01", "Bip01 NonAccum", "Bip01 Pelvis", "Bip01 Spine", "Bip01 Spine1", "Bip01 Spine2", "Bip01 Neck", "Bip01 R Clavicle", "Bip01 R UpperArm", "Bip01 R Forearm", "Bip01 R Hand", "Weapon"];
/// Game units per metre.
const UNITS_PER_METRE: f32 = 70.0;
/// Half the span (s) the velocity and spin are measured over, either side of the `Hit` key.
const AROUND: f32 = 1.0 / 120.0;

/// An attack's swing at its `Hit` key, in the animation's root space: the grip's velocity (m/s),
/// the weapon's spin (rad/s) and its blade's way (its +Y).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swing {
    pub grip: Vec3,
    pub spin: Vec3,
    pub blade: Vec3,
}

impl Swing {
    /// How fast a point `d` metres up the blade from the grip moves (m/s).
    pub fn speed_at(&self, d: f32) -> f32 {
        (self.grip + self.spin.cross(self.blade * d)).length()
    }
}

/// An animation's sequence, ready to sample: its nodes by name (their interpolators) and its
/// `Hit` key.
struct Sequence<'a> {
    nif: &'a Nif,
    nodes: Vec<(String, i32)>,
}

/// The swing of the attack animation `kf` at its `Hit` key; None if it has none, or the chain isn't
/// in it.
pub fn swing_at_hit(kf: &Nif) -> Option<Swing> {
    let (controlled, text_keys, palette) = kf.blocks.iter().find_map(|b| match b {
        Block::Sequence { controlled, text_keys, palette, .. } => Some((controlled, *text_keys, *palette)),
        _ => None,
    })?;
    let Some(Block::StringPalette(pal)) = kf.get(palette) else { return None };
    let name = |offset: u32| {
        let rest = pal.get(offset as usize..)?;
        Some(rest[..rest.find('\0').unwrap_or(rest.len())].to_owned())
    };
    let nodes: Vec<(String, i32)> = controlled.iter().filter_map(|&(interp, off)| Some((name(off)?, interp))).collect();
    let Some(Block::TextKeys(keys)) = kf.get(text_keys) else { return None };
    let hit = keys.iter().find(|(_, k)| k.to_ascii_lowercase().starts_with("hit"))?.0;
    let seq = Sequence { nif: kf, nodes };
    if !CHAIN.iter().all(|c| seq.interpolator(c).is_some()) {
        return None;
    }
    let ((r0, t0), (r1, t1)) = (seq.weapon(hit - AROUND), seq.weapon(hit + AROUND));
    let grip = (t1 - t0) / (2.0 * AROUND) / UNITS_PER_METRE;
    let (axis, angle) = Quat::from_mat3(&(r1 * r0.transpose())).normalize().to_axis_angle();
    let spin = if angle.is_finite() && axis.is_finite() { axis * angle / (2.0 * AROUND) } else { Vec3::ZERO };
    let (r, _) = seq.weapon(hit);
    let s = Swing { grip, spin, blade: (r * Vec3::Y).normalize_or(Vec3::Y) };
    (s.grip.is_finite() && s.spin.is_finite()).then_some(s)
}

impl Sequence<'_> {
    fn interpolator(&self, node: &str) -> Option<i32> {
        self.nodes.iter().find(|(n, _)| n.eq_ignore_ascii_case(node)).map(|(_, i)| *i)
    }

    /// The `Weapon` node's rotation and place (game units) in the root's space at time `t`.
    fn weapon(&self, t: f32) -> (Mat3, Vec3) {
        let (mut r, mut at) = (Mat3::IDENTITY, Vec3::ZERO);
        for node in CHAIN {
            let (rot, tr) = self.interpolator(node).map_or((None, None), |i| self.local(i, t));
            at += r * tr.unwrap_or(Vec3::ZERO);
            r *= rot.unwrap_or(Mat3::IDENTITY);
        }
        (r, at)
    }

    /// A node's local rotation and translation at time `t` (None: the animation doesn't set it).
    fn local(&self, interp: i32, t: f32) -> (Option<Mat3>, Option<Vec3>) {
        let set = |v: f32| v > -1e30;
        match self.nif.get(interp) {
            Some(Block::TransformInterpolator { translation, rotation, data, .. }) => {
                let mut rot = set(rotation[0]).then(|| quat(*rotation));
                let mut tr = set(translation.x).then_some(*translation);
                if let Some(Block::TransformData(k)) = self.nif.get(*data) {
                    if let Some(r) = keyed_rotation(k, t) {
                        rot = Some(r);
                    }
                    if !k.translations.is_empty() {
                        tr = Some(lerp_keys(&k.translations, t));
                    }
                }
                (rot, tr)
            }
            Some(Block::BSplineInterpolator { start, stop, data, basis, translation, rotation, handles, bias_mult }) => {
                let (Some(Block::BSplineData { shorts }), Some(Block::BSplineBasis(n))) = (self.nif.get(*data), self.nif.get(*basis)) else {
                    return (None, None);
                };
                let n = *n as usize;
                let u = (t - start) / (stop - start).max(1e-6);
                let points = |handle: u32, width: usize, bias: f32, mult: f32| -> Option<Vec<Vec<f32>>> {
                    if handle >= 0xFFFF {
                        return None;
                    }
                    (0..n).map(|i| (0..width).map(|c| shorts.get(handle as usize + width * i + c).map(|&v| bias + mult * v as f32 / 32767.0)).collect()).collect()
                };
                let tr = points(handles[0], 3, bias_mult[0], bias_mult[1]).map(|p| {
                    let v = bspline(&p, u);
                    Vec3::new(v[0], v[1], v[2])
                });
                let rot = points(handles[1], 4, bias_mult[2], bias_mult[3]).map(|p| {
                    let v = bspline(&p, u);
                    quat([v[0], v[1], v[2], v[3]])
                });
                (rot.or_else(|| set(rotation[0]).then(|| quat(*rotation))), tr.or_else(|| set(translation.x).then_some(*translation)))
            }
            _ => (None, None),
        }
    }
}

/// A NIF quaternion (w x y z) as a rotation.
fn quat(q: [f32; 4]) -> Mat3 {
    Mat3::from_quat(Quat::from_xyzw(q[1], q[2], q[3], q[0]).normalize())
}

fn keyed_rotation(k: &Keyframes, t: f32) -> Option<Mat3> {
    if k.rotation_kind == 4 {
        if k.xyz.iter().all(Vec::is_empty) {
            return None;
        }
        let angle = |keys: &Vec<(f32, f32)>| if keys.is_empty() { 0.0 } else { lerp_keys(&keys.iter().map(|&(t, v)| (t, Vec3::splat(v))).collect::<Vec<_>>(), t).x };
        let (x, y, z) = (angle(&k.xyz[0]), angle(&k.xyz[1]), angle(&k.xyz[2]));
        return Some(Mat3::from_rotation_z(z) * Mat3::from_rotation_y(y) * Mat3::from_rotation_x(x));
    }
    let q = &k.quats;
    let (first, last) = (q.first()?, q.last()?);
    if t <= first.0 {
        return Some(quat(first.1));
    }
    for w in q.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if t0 <= t && t <= t1 {
            let (a, mut b) = (Quat::from_xyzw(a[1], a[2], a[3], a[0]), Quat::from_xyzw(b[1], b[2], b[3], b[0]));
            if a.dot(b) < 0.0 {
                b = -b;
            }
            return Some(Mat3::from_quat(a.slerp(b, (t - t0) / (t1 - t0).max(1e-9)).normalize()));
        }
    }
    Some(quat(last.1))
}

fn lerp_keys(keys: &[(f32, Vec3)], t: f32) -> Vec3 {
    let (Some(first), Some(last)) = (keys.first(), keys.last()) else { return Vec3::ZERO };
    if t <= first.0 {
        return first.1;
    }
    for w in keys.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if t0 <= t && t <= t1 {
            return a.lerp(b, (t - t0) / (t1 - t0).max(1e-9));
        }
    }
    last.1
}

/// A cubic B-spline through `points` (its knots open and uniform: it starts at the first point and
/// ends at the last), at `u` (0..1): de Boor's.
fn bspline(points: &[Vec<f32>], u: f32) -> Vec<f32> {
    const DEGREE: usize = 3;
    let n = points.len();
    if n <= DEGREE {
        return points.first().cloned().unwrap_or_default();
    }
    let m = n - DEGREE;
    let knot = |i: usize| -> f32 {
        if i <= DEGREE {
            0.0
        } else if i >= n {
            1.0
        } else {
            (i - DEGREE) as f32 / m as f32
        }
    };
    let u = u.clamp(0.0, 1.0 - 1e-6);
    let mut k = DEGREE;
    while k < n - 1 && knot(k + 1) <= u {
        k += 1;
    }
    let mut d: Vec<Vec<f32>> = (0..=DEGREE).map(|j| points[j + k - DEGREE].clone()).collect();
    for r in 1..=DEGREE {
        for j in (r..=DEGREE).rev() {
            let i = j + k - DEGREE;
            let a = (u - knot(i)) / (knot(i + DEGREE + 1 - r) - knot(i)).max(1e-9);
            d[j] = d[j].iter().zip(&d[j - 1]).map(|(x, y)| (1.0 - a) * y + a * x).collect();
        }
    }
    d.swap_remove(DEGREE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bspline_runs_from_its_first_point_to_its_last_through_the_middle() {
        let p: Vec<Vec<f32>> = (0..6).map(|i| vec![i as f32]).collect();
        assert!((bspline(&p, 0.0)[0] - 0.0).abs() < 1e-5);
        assert!((bspline(&p, 0.999_999)[0] - 5.0).abs() < 1e-3);
        // Points on a line come out on it, evenly.
        assert!((bspline(&p, 0.5)[0] - 2.5).abs() < 1e-4);
    }

    #[test]
    fn a_point_up_the_blade_moves_with_the_grip_and_the_spin() {
        let s = Swing { grip: Vec3::X * 2.0, spin: Vec3::Z * 10.0, blade: Vec3::Y };
        // The spin about Z moves a point up Y towards -X: against the grip.
        assert!((s.speed_at(0.0) - 2.0).abs() < 1e-5);
        assert!((s.speed_at(1.0) - 8.0).abs() < 1e-4);
    }
}
