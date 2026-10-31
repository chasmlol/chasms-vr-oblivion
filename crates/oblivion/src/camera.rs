//! Oblivion's world camera, and putting it where a headset eye is.
//!
//! Oblivion's world is right-handed, Z up, X east, Y north, 70 units to the metre (Bethesda's
//! figure): [`BASIS`]. A Gamebryo camera looks along its local X, with Y up and Z to its right, so its
//! world rotation's columns are (view direction, up, right). Its frustum is the tangents of the
//! view's edges at distance 1 (left and bottom negative), with the near and far planes.

use crate::engine::{av, cam};
use crate::mem;
use glam::{Mat3, Quat, Vec3};
use rustvr_adapter::{Basis, View};
use rustvr_protocol::{Fov, Pose};

pub const UNITS_PER_METER: f32 = 70.0;

pub fn basis() -> Basis {
    Basis::z_up_right_handed(UNITS_PER_METER)
}

/// A camera's world transform and frustum, as the game keeps them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraState {
    /// Row-major 3x3: columns (direction, up, right), game axes.
    pub rot: [f32; 9],
    pub pos: [f32; 3],
    /// left, right, top, bottom, near, far.
    pub frustum: [f32; 6],
}

impl CameraState {
    pub fn dir(&self) -> Vec3 {
        Vec3::new(self.rot[0], self.rot[3], self.rot[6])
    }

    pub fn up(&self) -> Vec3 {
        Vec3::new(self.rot[1], self.rot[4], self.rot[7])
    }

    /// The camera's position in the protocol's space (metres).
    pub fn eye(&self) -> Vec3 {
        basis().point_from_game(Vec3::from(self.pos))
    }

    /// Where it looks, as a yaw in the protocol's space (radians, 0 = +Z, turning towards +X).
    pub fn look_yaw(&self) -> f32 {
        let d = basis().dir_from_game(self.dir());
        d.x.atan2(d.z)
    }
}

pub fn read(camera: usize) -> Option<CameraState> {
    if camera == 0 {
        return None;
    }
    let p = camera as *const u8;
    Some(CameraState {
        rot: mem::read(unsafe { p.add(av::WORLD_ROTATE) })?,
        pos: mem::read(unsafe { p.add(av::WORLD_TRANSLATE) })?,
        frustum: mem::read(unsafe { p.add(cam::FRUSTUM) })?,
    })
}

pub fn write(camera: usize, s: &CameraState) {
    if camera == 0 || !mem::is_readable(camera as *const u8, cam::FRUSTUM + 24) {
        return;
    }
    let p = camera as *mut u8;
    unsafe {
        (p.add(av::WORLD_ROTATE) as *mut [f32; 9]).write_unaligned(s.rot);
        (p.add(av::WORLD_TRANSLATE) as *mut [f32; 3]).write_unaligned(s.pos);
        (p.add(cam::FRUSTUM) as *mut [f32; 6]).write_unaligned(s.frustum);
    }
}

/// How far Oblivion's sneaking has lowered its first-person camera (protocol metres, the world's
/// axes), while the camera is the first-person one. Every first-person sneak animation keys the
/// skeleton's `Bip01 NonAccum`, the node `Camera01` hangs from, at (0, 0, -24.8) units (35 cm at a
/// race height of 1: the eye at 79% of its standing height), crossfaded in over the animations'
/// blend (0.1 s); the standing ones key it at 0 (`docs/research/stealth.md`).
pub fn sneak_drop() -> Option<Vec3> {
    let player = crate::engine::player();
    if player == 0 || mem::at::<u8>(player + crate::engine::pc::THIRD_PERSON)? != 0 || mem::at::<u8>(crate::engine::CAMERA_OFF_EYES)? != 0 {
        return None;
    }
    let camera = mem::deref(crate::engine::FIRST_PERSON_CAMERA);
    let posture = if camera != 0 { mem::deref(camera + crate::nodes::PARENT) } else { 0 };
    let root = if posture != 0 { mem::deref(posture + crate::nodes::PARENT) } else { 0 };
    if root == 0 || crate::nodes::name(posture) != "Bip01 NonAccum" {
        return None;
    }
    let at: [f32; 3] = mem::read(unsafe { (posture as *const u8).add(av::WORLD_TRANSLATE) })?;
    let from: [f32; 3] = mem::read(unsafe { (root as *const u8).add(av::WORLD_TRANSLATE) })?;
    let d = Vec3::from(at) - Vec3::from(from);
    // (Not a posture past 60 units: the skeleton's being made or taken away.)
    (d.is_finite() && d.length() < 60.0).then(|| basis().point_from_game(d))
}

/// Whether the headset's view leaves the game's sneak drop out: from when the player crouches for
/// real (`body_crouched`, the runtime's `Intents::crouch`: their head is already as low as they
/// went) until the game's camera is back up (`drop`, metres). A sneak from the button is the game's
/// own, its camera down as the flat game has it.
pub fn hold_sneak_drop(holding: bool, body_crouched: bool, drop: f32) -> bool {
    body_crouched || (holding && drop > 0.01)
}

/// The camera for one headset eye: the eye's pose mapped into the world by `view`, its field of
/// view, and the game's own near and far planes.
pub fn eye_camera(view: &View, eye: &Pose, fov: &Fov, game: &CameraState) -> CameraState {
    let (pos, rot) = view.pose(eye);
    let b = basis();
    let dir = b.dir_to_game(rot * Vec3::Z).normalize();
    let up = b.dir_to_game(rot * Vec3::Y).normalize();
    let right = b.dir_to_game(rot * Vec3::X).normalize();
    let m = Mat3::from_cols(dir, up, right);
    let mut out = CameraState { rot: [0.0; 9], pos: b.point_to_game(pos).to_array(), frustum: game.frustum };
    for r in 0..3 {
        for c in 0..3 {
            out.rot[r * 3 + c] = m.col(c)[r];
        }
    }
    // (The protocol's field of view is already the tangents of its edges, as a frustum wants.)
    out.frustum[0] = fov.left;
    out.frustum[1] = fov.right;
    out.frustum[2] = fov.up;
    out.frustum[3] = fov.down;
    out
}

/// The rotation of a camera state in the protocol's space.
pub fn rotation(s: &CameraState) -> Quat {
    let b = basis();
    let dir = b.dir_from_game(s.dir());
    let up = b.dir_from_game(s.up());
    let right = up.cross(dir);
    Quat::from_mat3(&Mat3::from_cols(right, up, dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_camera_looking_north_looks_along_canonical_z() {
        // Looking north (+Y), up +Z, right +X.
        let s = CameraState { rot: [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0], pos: [70.0, 140.0, 7.0], frustum: [-1.0, 1.0, 1.0, -1.0, 5.0, 1000.0] };
        assert!(s.look_yaw().abs() < 1e-6);
        assert!(s.eye().abs_diff_eq(Vec3::new(1.0, 0.1, 2.0), 1e-5));
        assert!(rotation(&s).abs_diff_eq(Quat::IDENTITY, 1e-5));
    }

    #[test]
    fn the_sneak_drop_is_held_out_only_for_a_real_crouch() {
        // The button's sneak: the game's camera goes down.
        assert!(!hold_sneak_drop(false, false, 0.35));
        // Crouched for real: held out, and still while standing up, until the camera's back up.
        assert!(hold_sneak_drop(false, true, 0.35));
        assert!(hold_sneak_drop(true, false, 0.2));
        assert!(!hold_sneak_drop(true, false, 0.0));
    }
}
