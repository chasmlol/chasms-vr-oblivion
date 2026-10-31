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

/// The game's own first-person view bob (docs/research/view-bob.md): its first-person animations
/// move its camera. In first person the world camera is put on the first-person skeleton's
/// `Camera01` (`PlayerCharacter::UpdateCamera`, 0x66B710), and every first-person animation keys
/// that node: a walk drops it 0.4 units (5 mm) once a stride, a run 1.3 (2 cm) at each step, a
/// landing from a jump 14 (20 cm), swimming up to 3.5. The first-person node is put at the player's
/// feet and eye height, turned with the look (the player's update, 0x603CA0), and the skeleton under
/// it is lowered by `Camera01`'s height at rest (118 units, as the first-person model loads,
/// 0x667BE0): at rest `Camera01` is at the node, its animations move it off it. `UpdateCamera` also
/// turns the camera by `Camera01`'s own turn (right by its rotation's -m[1][0] and down by m[1][1],
/// each times a quarter turn); no walk keys that.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bob {
    /// Where the animation has moved the camera off the first-person node (protocol metres, the
    /// world's axes): up and down, and to the sides. (Not forward: at rest the camera is a little
    /// behind the node anyway, and sneaking leans it forward; a posture, not a bob.)
    pub offset: Vec3,
    /// How it turns the camera after where it looks: right and down (radians).
    pub yaw: f32,
    pub pitch: f32,
}

/// The game's own first-person bob this frame, while its camera is the first-person one.
pub fn first_person_bob() -> Option<Bob> {
    let player = crate::engine::player();
    if player == 0 || mem::at::<u8>(player + crate::engine::pc::THIRD_PERSON)? != 0 || mem::at::<u8>(crate::engine::CAMERA_OFF_EYES)? != 0 {
        return None;
    }
    let node = mem::deref(player + crate::engine::pc::FIRST_PERSON_NODE) as *const u8;
    let camera = mem::deref(crate::engine::FIRST_PERSON_CAMERA) as *const u8;
    if node.is_null() || camera.is_null() {
        return None;
    }
    let rot: [f32; 9] = mem::read(unsafe { node.add(av::WORLD_ROTATE) })?;
    let at: [f32; 3] = mem::read(unsafe { node.add(av::WORLD_TRANSLATE) })?;
    let scale: f32 = mem::read(unsafe { node.add(av::WORLD_SCALE) })?;
    let eye: [f32; 3] = mem::read(unsafe { camera.add(av::WORLD_TRANSLATE) })?;
    let turn: [f32; 9] = mem::read(unsafe { camera.add(av::LOCAL_ROTATE) })?;
    // (Gamebryo's rotations are row-major: the node's own axes, x right, y forward, z up, are its
    // columns.)
    let m = Mat3::from_cols_array(&rot).transpose();
    let scale = if scale.is_finite() && scale > 1e-3 { scale } else { 1.0 };
    let off = m.transpose() * (Vec3::from(eye) - Vec3::from(at)) / scale;
    let moved = m * Vec3::new(off.x, 0.0, off.z) * scale;
    // (`UpdateCamera` leaves the turn out while the dialogue menu is open.)
    const MENU_DIALOG: u32 = 0x3F1;
    let quarter = if crate::menus::menu_open(MENU_DIALOG) { 0.0 } else { std::f32::consts::FRAC_PI_2 };
    Some(Bob { offset: basis().point_from_game(moved), yaw: -turn[3] * quarter, pitch: turn[4] * quarter })
}

/// Says in the log every five seconds how far the game's bob went (while it went anywhere), and how
/// far the game's camera `game` was from `Camera01`, which `UpdateCamera` puts it on: a playtest's
/// check that the bob read is the one the game shows.
pub fn note_bob(bob: Option<&Bob>, game: &CameraState, now: f64) {
    struct Notes {
        since: f64,
        low: f32,
        high: f32,
        side: f32,
        turn: f32,
        off: f32,
    }
    static NOTES: std::sync::Mutex<Option<Notes>> = std::sync::Mutex::new(None);
    let mut notes = NOTES.lock().unwrap_or_else(|e| e.into_inner());
    let n = notes.get_or_insert(Notes { since: now, low: 0.0, high: 0.0, side: 0.0, turn: 0.0, off: 0.0 });
    if let Some(b) = bob {
        n.low = n.low.min(b.offset.y);
        n.high = n.high.max(b.offset.y);
        n.side = n.side.max(Vec3::new(b.offset.x, 0.0, b.offset.z).length());
        n.turn = n.turn.max(b.yaw.abs().max(b.pitch.abs()));
        let at: Option<[f32; 3]> = mem::read((mem::deref(crate::engine::FIRST_PERSON_CAMERA) + av::WORLD_TRANSLATE) as *const u8);
        n.off = n.off.max(at.map_or(0.0, |a| Vec3::from(a).distance(Vec3::from(game.pos))));
    }
    if now - n.since >= 5.0 {
        if n.high - n.low > 1e-4 || n.side > 1e-4 || n.turn > 1e-4 {
            tracing::info!(
                "view bob: the game's own in the last {:.0} s: {:.1} to {:.1} cm up, {:.1} cm to the side, turned {:.2} deg; the game's camera up to {:.2} units off Camera01",
                now - n.since,
                n.low * 100.0,
                n.high * 100.0,
                n.side * 100.0,
                n.turn.to_degrees(),
                n.off
            );
        }
        *n = Notes { since: now, low: 0.0, high: 0.0, side: 0.0, turn: 0.0, off: 0.0 };
    }
}

/// The game's view bob for the runtime ([`rustvr_protocol::ViewBob`]): `moved` (protocol metres,
/// the world's axes) and the turn as the game's camera `game` sees them, in its own frame (right, up,
/// forward).
pub fn view_bob(game: &CameraState, moved: Vec3, yaw: f32, pitch: f32) -> rustvr_protocol::ViewBob {
    // (Along the camera's own axes in the game's space, which keeps lengths and angles: the
    // protocol's is a mirror of it.)
    let along = basis().point_to_game(moved);
    let right = Vec3::new(game.rot[2], game.rot[5], game.rot[8]);
    let offset = Vec3::new(along.dot(right), along.dot(game.up()), along.dot(game.dir())) / UNITS_PER_METER;
    let rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
    rustvr_protocol::ViewBob { flags: rustvr_protocol::BOB_GIVEN, offset: offset.to_array(), rot: rot.to_array() }
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
}
