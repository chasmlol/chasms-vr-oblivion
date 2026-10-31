//! Sitting in the game's furniture (`docs/research/interact.md`, "Sitting and sleeping"). A chair,
//! a bench or a stool is sat on as the game has it: its activation (the trigger with the reticle on
//! it and the game's icon for it, from as far as the game reaches), the game turning the player to
//! its marker and its sitting animation, its ownership and crime; a bed asks its own way (its sleep
//! and wait menu). Getting up is the game's Activate while seated: the trigger again. The rest is
//! the headset's:
//!
//! * **Dark while it moves the player.** Sitting down and getting up, the game's animation moves the
//!   eye as the player doesn't: the view is dark meanwhile and a moment after ([`SEAT_MOVING`], as
//!   getting on or off a horse is), and for [`SETTLE`] after any change of it.
//! * **The view seated.** Once seated (and dark), the view is recentred on the seat facing the way
//!   the game sat them (out of the chair), the eye over the seat at a sitting body's height
//!   ([`SEATED_EYE`] of the game's standing eye over the feet): however the player's head is then
//!   (standing in their room or sat on a real chair) is the seated eye. The head looks round and
//!   leans as far as the room's limit lets the view go from it.
//! * **The body seated.** The runtime's body sits on the seat (the furniture's own mesh under where
//!   the game sat them, [`seat_on`]), its feet on the floor before it ([`Ride::seat`]).
//! * **Not walked.** While the game has them sitting down, seated or getting up, the stick and the
//!   room's walking don't move the game's player, and the head's look doesn't turn it (the game
//!   turns them to the seat): only the trigger gets them up.
//!
//! [`SEAT_MOVING`]: rustvr_protocol::SEAT_MOVING
//! [`Ride::seat`]: rustvr_protocol::Ride::seat

use glam::{Vec2, Vec3};

/// The eye over the seat when seated, as a share of the game's standing eye over the feet (a sitting
/// body's eyes are a little under half its standing eye height over what it sits on).
pub const SEATED_EYE: f32 = 0.48;
/// The game's standing eye over the feet (metres) until it's been seen.
pub const STANDING_EYE: f32 = 1.65;
/// How long after the game's sitting changes (seconds) the view is dark at least; seated, it's
/// recentred on the seat this long after (dark by then).
pub const SETTLE: f64 = 0.3;
/// A seat's top is this high over the feet at the least and the most (metres).
pub const SEAT_LOW: f32 = 0.2;
pub const SEAT_HIGH: f32 = 0.8;
/// A seat's height over the feet when its mesh doesn't say (metres).
pub const SEAT_FALLBACK: f32 = 0.45;

/// The view's anchor while the game has the player seated: the eye, the way the view faces when
/// it's recentred, and whether it's recentred this frame.
#[derive(Clone, Copy, Debug)]
pub struct Carry {
    pub eye: Vec3,
    pub look_yaw: f32,
    pub recenter: bool,
}

/// The top of a seat on the furniture's triangles `tris` (the protocol's world) for a player the
/// game has sat at `at` (their feet), facing `fwd`: straight down onto its flat tops, between
/// [`SEAT_LOW`] and [`SEAT_HIGH`] over the feet, the highest there; under the feet, else a little
/// behind or before them (the game's place for the feet needn't be over the seat's middle).
pub fn seat_on(tris: &[[Vec3; 3]], at: Vec3, fwd: Vec3) -> Option<Vec3> {
    let fwd = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or(Vec3::Z);
    for d in [0.0, -0.1, 0.1, -0.2, 0.2, -0.3] {
        let p = at + fwd * d;
        let top = tris.iter().filter_map(|t| down_onto(t, p)).filter(|y| (at.y + SEAT_LOW..=at.y + SEAT_HIGH).contains(y)).reduce(f32::max);
        if let Some(y) = top {
            return Some(p.with_y(y));
        }
    }
    None
}

/// Where the upright line through `p` meets the triangle `t`, if it's flat enough to sit on.
fn down_onto(t: &[Vec3; 3], p: Vec3) -> Option<f32> {
    let n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
    if n.y.abs() < 0.7 {
        return None;
    }
    let flat = |v: Vec3| Vec2::new(v.x, v.z);
    let (a, b, c) = (flat(t[0]), flat(t[1]), flat(t[2]));
    let (v0, v1, v2) = (c - a, b - a, flat(p) - a);
    let den = v0.perp_dot(v1);
    if den.abs() < 1e-9 {
        return None;
    }
    // p = a + u (c - a) + v (b - a), on the floor's plane.
    let u = v2.perp_dot(v1) / den;
    let v = v0.perp_dot(v2) / den;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0).then(|| t[0].y + u * (t[2].y - t[0].y) + v * (t[1].y - t[0].y))
}

/// The eye seated on `seat`, for a player whose standing eye is `standing` over their feet.
pub fn seated_eye(seat: Vec3, standing: Option<f32>) -> Vec3 {
    seat + Vec3::Y * standing.unwrap_or(STANDING_EYE) * SEATED_EYE
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn carry(_eye: Vec3, _now: f64) -> Option<Carry> {
    None
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn seated() -> bool {
    false
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn publish(_view: &rustvr_adapter::View, _ride: &mut rustvr_protocol::Ride, _now: f64) {}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem};
    use glam::Quat;
    use rustvr_protocol::*;
    use std::sync::Mutex;

    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DISABLED: u32 = 0x800;
    const FURNITURE: u8 = 0x20;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    /// The player's sit and sleep state (`BaseProcess::GetSitSleepState`): 0 normal, 1 loading the
    /// sit idle, 2 wanting to sit, 3 waiting for the sit animation, 4 sitting, 5 wanting to stand;
    /// past it, the sleep's own.
    const NORMAL: u32 = 0;
    const SEATED: u32 = 4;
    const STANDING_UP: u32 = 5;
    /// Furniture within this of the player (metres) is looked at for the seat.
    const NEAR: f32 = 2.5;

    #[derive(Default)]
    struct State {
        /// The game's sitting now (0 to 5), the one before it, and since when.
        sit: u32,
        was: u32,
        since: f64,
        /// The game's eye over the feet standing (metres), as last seen standing.
        standing_eye: Option<f32>,
        /// Seated: the seat (the world), the way the game sat them facing (its heading), the floor's
        /// height; once found.
        seat: Option<(Vec3, f32, f32)>,
        /// The view recentred on it.
        recentred: bool,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn feet() -> Option<Vec3> {
        mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)))
    }

    /// The game's sitting for the player: 0 to 5 (not on a horse, which is riding's; not the sleep's).
    fn sitting() -> u32 {
        let p = engine::player();
        if p == 0 || crate::riding::mounted() {
            return NORMAL;
        }
        let s = crate::riding::sit_state(p);
        if s <= STANDING_UP { s } else { NORMAL }
    }

    /// The game has the player sitting down, seated or getting up in furniture: the stick, the
    /// room's walking and the head's look aren't its player's then.
    pub fn seated() -> bool {
        sitting() != NORMAL
    }

    /// Dark: sitting down or getting up, or just after the sitting changed either way.
    fn moving(s: &State, now: f64) -> bool {
        matches!(s.sit, 1..=3 | STANDING_UP) || (now - s.since < SETTLE && (s.sit == SEATED || s.was != NORMAL))
    }

    /// The furniture near the player's feet `at`.
    fn furniture_near(at: Vec3) -> Vec<usize> {
        let b = camera::basis();
        let mut found = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let base = if r != 0 { mem::deref(r + REFR_BASE) } else { 0 };
                if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(FURNITURE) || mem::at::<u32>(r + FORM_FLAGS).is_none_or(|f| f & DISABLED != 0) || mem::deref(r + REFR_NODE) == 0 {
                    continue;
                }
                let Some(p) = mem::at::<[f32; 3]>(r + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { continue };
                if p.distance(at) < NEAR {
                    found.push(r);
                }
            }
        }
        found
    }

    /// The seat the game has sat the player on (the top of the furniture under where it put their
    /// feet), the way it faces them, and the floor's height there.
    fn find_seat() -> Option<(Vec3, f32, f32)> {
        let at = feet()?;
        let heading = mem::at::<f32>(engine::player() + engine::refr::ROT + 8).filter(|h| h.is_finite())?;
        let fwd = Vec3::new(heading.sin(), 0.0, heading.cos());
        let mut best: Option<(f32, usize, Vec3)> = None;
        for r in furniture_near(at) {
            if let Some(top) = seat_on(&crate::lock::triangles_of(r), at, fwd) {
                let d = Vec3::new(top.x - at.x, 0.0, top.z - at.z).length();
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, r, top));
                }
            }
        }
        let seat = match best {
            Some((d, r, top)) => {
                tracing::info!("rest: seated on {:08X}: its seat {:.2} m over the feet, {d:.2} m from them along the floor", mem::at::<u32>(r + FORM_ID).unwrap_or(0), top.y - at.y);
                top
            }
            None => {
                tracing::info!("rest: seated, no seat found under the feet: {SEAT_FALLBACK} m over them");
                at + Vec3::Y * SEAT_FALLBACK
            }
        };
        Some((seat, heading, at.y))
    }

    /// Before the frame's view (with the game's eye): the view's anchor while seated (recentred on
    /// the seat once it's dark), else None (the game's eye).
    pub fn carry(eye: Vec3, now: f64) -> Option<Carry> {
        let sit = sitting();
        let feet = feet();
        with_state(|s| {
            if sit == NORMAL && s.was == NORMAL && !crate::riding::mounted() && !crate::player::sneaking() {
                if let Some(f) = feet {
                    s.standing_eye = Some(eye.y - f.y);
                }
            }
            if sit != s.sit {
                tracing::info!("rest: the game's sitting {} -> {sit}", s.sit);
                s.was = s.sit;
                s.sit = sit;
                s.since = now;
                if sit != SEATED {
                    s.seat = None;
                    s.recentred = false;
                }
            }
            // (Standing again for good: its eye is a standing one from here.)
            if sit == NORMAL && now - s.since > SETTLE {
                s.was = NORMAL;
            }
            if sit != SEATED || now - s.since < SETTLE {
                return None;
            }
            if s.seat.is_none() {
                s.seat = find_seat();
            }
            let (seat, heading, _) = s.seat?;
            let at = seated_eye(seat, s.standing_eye);
            let recenter = !std::mem::replace(&mut s.recentred, true);
            if recenter {
                tracing::info!("rest: the view recentred on the seat, facing {:.0} degrees: the eye {:.2} m over it (the game's eye {:.2} m off it)", heading.to_degrees(), at.y - seat.y, at.distance(eye));
            }
            Some(Carry { eye: at, look_yaw: heading, recenter })
        })
    }

    /// For the runtime, every game frame (with this frame's mapping): sitting down or getting up,
    /// and the seat once the view is on it.
    pub fn publish(view: &rustvr_adapter::View, ride: &mut Ride, now: f64) {
        with_state(|s| {
            if moving(s, now) {
                ride.sitting |= SEAT_MOVING;
            }
            if let (SEATED, true, Some((seat, heading, floor))) = (s.sit, s.recentred, s.seat) {
                ride.sitting |= SEAT_SEATED;
                ride.seat = Pose { pos: view.to_tracking(seat).to_array(), valid: 1, rot: Quat::from_rotation_y(heading - view.body_yaw).to_array() };
                ride.floor = view.to_tracking(seat.with_y(floor)).y;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A seat's top, a square `size` across at `height`, round (0, 0), as two triangles.
    fn square(height: f32, size: f32) -> Vec<[Vec3; 3]> {
        let h = size / 2.0;
        let c = |x: f32, z: f32| Vec3::new(x, height, z);
        vec![[c(-h, -h), c(h, -h), c(h, h)], [c(-h, -h), c(h, h), c(-h, h)]]
    }

    #[test]
    fn the_seat_under_the_feet() {
        let tris = square(0.45, 0.4);
        let top = seat_on(&tris, Vec3::ZERO, Vec3::Z).unwrap();
        assert!((top.y - 0.45).abs() < 1e-5 && top.x.abs() < 1e-5 && top.z.abs() < 1e-5, "{top}");
        // Sat a little before it: the seat just behind the feet.
        let top = seat_on(&tris, Vec3::new(0.0, 0.0, 0.25), Vec3::Z).unwrap();
        assert!((top.z - 0.15).abs() < 1e-5, "{top}");
    }

    #[test]
    fn not_the_floor_nor_the_back() {
        // The floor itself, a chair's back's top over the band, an upright side: none of them a seat.
        let mut tris = square(0.05, 2.0);
        tris.extend(square(0.95, 0.4));
        tris.push([Vec3::new(-0.2, 0.0, 0.0), Vec3::new(0.2, 0.0, 0.0), Vec3::new(0.0, 0.6, 0.0)]);
        assert_eq!(seat_on(&tris, Vec3::ZERO, Vec3::Z), None);
        // With the seat there too, the seat.
        tris.extend(square(0.5, 0.4));
        assert!((seat_on(&tris, Vec3::ZERO, Vec3::Z).unwrap().y - 0.5).abs() < 1e-5);
    }

    #[test]
    fn the_eye_seated() {
        let eye = seated_eye(Vec3::new(1.0, 0.45, 2.0), Some(1.7));
        assert!((eye.y - (0.45 + 1.7 * SEATED_EYE)).abs() < 1e-5 && eye.x == 1.0 && eye.z == 2.0);
    }
}
