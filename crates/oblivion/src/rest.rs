//! Sitting in the game's furniture (`docs/research/interact.md`, "Sitting and sleeping"). A chair,
//! a bench or a stool is sat on as the game has it: its activation (the game turning the player to
//! its marker and its sitting animation, its ownership and crime); a bed asks its own way (its sleep
//! and wait menu). Getting up is the game's Activate while seated: the trigger again. The rest is
//! the headset's:
//!
//! * **Pointed at with a hand**, as the gravity gloves point ([`point`]): along the arm from the
//!   shoulder out through an empty hand, the seat nearest the middle of the gloves' cone (or under
//!   the line itself), in front of the eyes, out to [`POINT_RANGE`]. The seat pointed at is the
//!   game's crosshair's target (its icon and its name, which float at it), so the trigger (the
//!   game's Activate) sits the player in it, from as far as it's pointed at.
//! * **Dark while it moves the player.** Sitting down and getting up, the game's animation moves the
//!   eye as the player doesn't: the view is dark meanwhile and a moment after ([`SEAT_MOVING`], as
//!   getting on or off a horse is), and for [`SETTLE`] after any change of it.
//! * **The view seated.** Once seated (and dark), the view is recentred on the seat facing the way
//!   the game sat them (out of the chair), the eye over the seat as high as the runtime's body has
//!   it sitting upright ([`SEATED_EYE`] of the player's standing eye height): however the player's
//!   head is then (standing in their room or sat on a real chair) is the seated eye. The head looks
//!   round and leans as far as the room's limit lets the view go from it.
//! * **The body seated.** The runtime's body sits on the seat (the furniture's own mesh under where
//!   the game put the feet, [`seat_on`]), its feet on the floor before it ([`Ride::seat`]).
//! * **Not walked.** While the game has them sitting down, seated or getting up, the stick and the
//!   room's walking don't move the game's player, and the head's look doesn't turn it (the game
//!   turns them to the seat): only the trigger gets them up.
//!
//! [`SEAT_MOVING`]: rustvr_protocol::SEAT_MOVING
//! [`Ride::seat`]: rustvr_protocol::Ride::seat

use glam::{Quat, Vec2, Vec3};

/// The eye over the seat when seated, as a share of the player's standing eye height: where the
/// runtime's body has it sitting upright (its skeleton's eye over a seat, 0.87 of its scale, over
/// its standing eye, 1.70 of it).
pub const SEATED_EYE: f32 = 0.51;
/// The player's standing eye height (metres) until the runtime's measured it.
pub const EYE_HEIGHT: f32 = 1.70;
/// How long after the game's sitting changes (seconds) the view is dark at least; seated, it's
/// recentred on the seat this long after (dark by then).
pub const SETTLE: f64 = 0.3;
/// A seat's top is this high over the feet at the least and the most (metres).
pub const SEAT_LOW: f32 = 0.2;
pub const SEAT_HIGH: f32 = 0.8;
/// A seat's height over the feet when its mesh doesn't say (metres).
pub const SEAT_FALLBACK: f32 = 0.45;

/// Pointing as the runtime's gravity gloves point (Half-Life: Alyx's, `Rig::glove_dir` and its
/// `alyx` settings): from the shoulder (this far below the eyes and out to the hand's side, turned
/// with the head's yaw) out through the hand.
pub const SHOULDER_DROP: f32 = 0.25;
pub const SHOULDER_OUT: f32 = 0.18;
/// The gloves' cone (the cosine of 15 degrees about the arm's line, from 25 cm behind the hand); the
/// seat pointed at keeps hold out to 20.
pub const CONE_COS: f32 = 0.965;
pub const HOLD_COS: f32 = 0.94;
pub const CONE_BACK: f32 = 0.254;
/// Only what's in front of the eyes: within these (degrees) of a point a little below the line of
/// sight (its tangent's offset down).
pub const VIEW_HALF_WIDE: f32 = 30.0;
pub const VIEW_HALF_HIGH: f32 = 35.0;
pub const VIEW_DOWN: f32 = -0.2;
/// A seat pointed at this long (seconds) is the one pointed at.
pub const SELECT_TIME: f64 = 0.1;
/// Seats are pointed at out to this far from the hand (metres).
pub const POINT_RANGE: f32 = 6.0;

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

/// The eye seated on `seat`, for a player whose standing eye height is `eye_height` (the runtime's
/// measure, in its own metres) in a world `world_per_meter` times theirs.
pub fn seated_eye(seat: Vec3, eye_height: Option<f32>, world_per_meter: f32) -> Vec3 {
    seat + Vec3::Y * eye_height.filter(|h| *h > 1.0).unwrap_or(EYE_HEIGHT) * SEATED_EYE * world_per_meter
}

/// The way a hand points (the left one if `left`), for the eyes at `head` turned `head_rot`.
pub fn glove_dir(head: Vec3, head_rot: Quat, hand: Vec3, left: bool) -> Vec3 {
    let fwd = (head_rot * Vec3::Z).with_y(0.0).normalize_or(Vec3::Z);
    let right = Vec3::Y.cross(fwd).normalize_or(Vec3::X);
    let shoulder = head - Vec3::Y * SHOULDER_DROP + right * if left { -SHOULDER_OUT } else { SHOULDER_OUT };
    (hand - shoulder).normalize_or(fwd)
}

/// Whether `p` is in front of the eyes, as the gloves have it.
pub fn in_view(head: Vec3, head_rot: Quat, p: Vec3) -> bool {
    let local = head_rot.inverse() * (p - head);
    if local.z <= 0.0 {
        return false;
    }
    let x = (local.x / local.z).atan().to_degrees().abs();
    let y = (local.y / local.z - VIEW_DOWN).atan().to_degrees().abs();
    x <= VIEW_HALF_WIDE && y <= VIEW_HALF_HIGH
}

/// How far along `dir` (a unit vector) from `from` the first of `tris` is, out to `max`.
pub fn ray_hit(tris: &[[Vec3; 3]], from: Vec3, dir: Vec3, max: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    for t in tris {
        let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
        let p = dir.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-9 {
            continue;
        }
        let s = from - t[0];
        let u = s.dot(p) / det;
        let q = s.cross(e1);
        let v = dir.dot(q) / det;
        let d = e2.dot(q) / det;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 && d > 0.0 && d <= max && best.is_none_or(|b| d < b) {
            best = Some(d);
        }
    }
    best
}

/// How well a seat (its triangles, their middle `centre`) is pointed at by a hand at `hand` pointing
/// `dir`, for the eyes at `head` turned `head_rot` (lower is better; None: not pointed at). Under the
/// arm's line itself, the nearer the better; else within the cone about it (wider for the seat
/// already pointed at, `held`), the nearer its middle is to the line the better.
pub fn pointing_at(tris: &[[Vec3; 3]], centre: Vec3, hand: Vec3, dir: Vec3, head: Vec3, head_rot: Quat, held: bool) -> Option<f32> {
    if let Some(d) = ray_hit(tris, hand, dir, POINT_RANGE)
        && in_view(head, head_rot, hand + dir * d)
    {
        return Some(d * 0.001);
    }
    let d = centre.distance(hand);
    if d > POINT_RANGE || !in_view(head, head_rot, centre) {
        return None;
    }
    let cos = dir.dot((centre - (hand - dir * CONE_BACK)).normalize_or_zero());
    (cos >= if held { HOLD_COS } else { CONE_COS }).then(|| (1.0 - cos) + d * 0.001)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn point(_view: &rustvr_adapter::View, _f: &rustvr_protocol::RtFrame, _now: f64) {}

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
    use rustvr_protocol::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
    /// Furniture within this of the player's feet (metres) is looked at for the seat they're sat on.
    const NEAR: f32 = 2.5;
    /// Seconds between looks for the furniture that can be pointed at (it doesn't move).
    const SEATS_EVERY: f64 = 0.5;
    /// The game's update of its crosshair (`InterfaceManager` 0x5806D0, thiscall: the manager, from
    /// the player's update before it acts on its controls): the reference the crosshair is on and
    /// can activate (+0xC8, which the Activate control activates, 0x579540), and the one it's on at
    /// all (+0xCC); then the HUD's card of it (`HUDInfoMenu::SetTarget`, 0x5A4980, cdecl: the
    /// reference, whether there's none to activate, 0) and, if that took it, the reticle's icon for
    /// it (0x578D50, cdecl: the reference), else none (0x578D30, cdecl: 0).
    const CROSSHAIR_UPDATE: usize = 0x0058_06D0;
    const CROSSHAIR_REF: usize = 0xC8;
    const PICKED_REF: usize = 0xCC;
    const SET_TARGET: usize = 0x005A_4980;
    const RETICLE_ICON: usize = 0x0057_8D50;
    const RETICLE_NONE: usize = 0x0057_8D30;

    static REAL_UPDATE: AtomicUsize = AtomicUsize::new(0);
    /// The seat a hand points at (its reference; 0: none), for the crosshair.
    static POINTED: AtomicUsize = AtomicUsize::new(0);

    #[derive(Default)]
    struct State {
        /// The game's sitting now (0 to 5), the one before it, and since when.
        sit: u32,
        was: u32,
        since: f64,
        /// Seated: the seat (the world), the way the game sat them facing (its heading), the floor's
        /// height; once found.
        seat: Option<(Vec3, f32, f32)>,
        /// The view recentred on it.
        recentred: bool,
        /// The furniture that can be pointed at: each one's triangles and their middle; looked for
        /// again then.
        seats: Vec<(usize, Vec<[Vec3; 3]>, Vec3)>,
        next_scan: f64,
        /// The seat pointed at and by which hand; a change to it waiting out [`SELECT_TIME`].
        pointed: Option<(usize, usize)>,
        pending: Option<(Option<usize>, f64)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn haptic(hand: usize, seconds: f32, freq: f32, amplitude: f32) {
        crate::link::with_adapter(|a| a.haptic(hand as u32, seconds, freq, amplitude));
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

    /// The furniture within `within` of `at`.
    fn furniture_near(at: Vec3, within: f32) -> Vec<usize> {
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
                if p.distance(at) < within {
                    found.push(r);
                }
            }
        }
        found
    }

    pub fn install() {
        crate::hooks::install_into(CROSSHAIR_UPDATE as *const (), crosshair_update as *const (), "the crosshair's update", &REAL_UPDATE);
    }

    /// The game's crosshair, then the seat a hand points at in its place: what the Activate control
    /// activates, its card and its icon.
    unsafe extern "thiscall" fn crosshair_update(this: usize) -> i32 {
        type Update = unsafe extern "thiscall" fn(usize) -> i32;
        let real: Update = unsafe { std::mem::transmute(REAL_UPDATE.load(Ordering::Acquire)) };
        let r = unsafe { real(this) };
        crate::hooks::guard_void("the seat pointed at", || {
            let seat = POINTED.load(Ordering::Acquire);
            if seat == 0 || this == 0 || mem::deref(seat + REFR_NODE) == 0 || !mem::is_readable((this + CROSSHAIR_REF) as *const u8, 8) {
                return;
            }
            unsafe {
                ((this + CROSSHAIR_REF) as *mut usize).write(seat);
                ((this + PICKED_REF) as *mut usize).write(seat);
            }
            type SetTarget = unsafe extern "C" fn(usize, u8, u8) -> u8;
            type Icon = unsafe extern "C" fn(usize);
            if unsafe { std::mem::transmute::<usize, SetTarget>(SET_TARGET)(seat, 0, 0) } != 0 {
                unsafe { std::mem::transmute::<usize, Icon>(RETICLE_ICON)(seat) };
            } else {
                unsafe { std::mem::transmute::<usize, Icon>(RETICLE_NONE)(0) };
            }
        });
        r
    }

    /// Before the eyes are drawn: the seat an empty hand points at, as the gloves point (a new one,
    /// or none, once it's held [`SELECT_TIME`]: a tick in the hand as it's taken).
    pub fn point(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        let off = engine::player() == 0 || crate::menus::menu_mode() || crate::riding::mounted() || seated();
        let busy: Vec<usize> = crate::items::held().iter().map(|h| h.hand).chain(crate::inventory::held_copies().iter().map(|c| c.hand)).collect();
        let me = feet();
        with_state(|s| {
            let (Some(me), false) = (me, off) else {
                s.pointed = None;
                s.pending = None;
                POINTED.store(0, Ordering::Release);
                return;
            };
            if now >= s.next_scan {
                s.next_scan = now + SEATS_EVERY;
                let mut seats = Vec::new();
                for r in furniture_near(me, POINT_RANGE + 1.5) {
                    // (Its triangles kept while it's near: it doesn't move.)
                    let tris = match s.seats.iter().position(|k| k.0 == r) {
                        Some(i) => std::mem::take(&mut s.seats[i].1),
                        None => crate::lock::triangles_of(r),
                    };
                    let Some((lo, hi)) = tris.iter().flatten().fold(None, |b: Option<(Vec3, Vec3)>, p| Some(b.map_or((*p, *p), |(lo, hi)| (lo.min(*p), hi.max(*p))))) else { continue };
                    seats.push((r, tris, (lo + hi) * 0.5));
                }
                s.seats = seats;
            }
            let (head, rot) = (view.head_pos, view.head_rot);
            let current = s.pointed.map(|p| p.0);
            let mut best: Option<(usize, usize, f32)> = None;
            for (i, h) in f.hands.iter().enumerate() {
                if h.connected == 0 || h.grip.valid == 0 || busy.contains(&i) {
                    continue;
                }
                let hand = view.point(Vec3::from_array(h.grip.pos));
                let dir = glove_dir(head, rot, hand, i == HAND_LEFT);
                for (r, tris, centre) in &s.seats {
                    if let Some(score) = pointing_at(tris, *centre, hand, dir, head, rot, Some(*r) == current)
                        && best.is_none_or(|b| score < b.2)
                    {
                        best = Some((*r, i, score));
                    }
                }
            }
            let want = best.map(|b| (b.0, b.1));
            if want.map(|w| w.0) == current {
                s.pending = None;
                s.pointed = want;
            } else {
                let since = match s.pending {
                    Some((p, t)) if p == want.map(|w| w.0) => t,
                    _ => {
                        s.pending = Some((want.map(|w| w.0), now));
                        now
                    }
                };
                if now - since >= SELECT_TIME {
                    s.pending = None;
                    s.pointed = want;
                    match want {
                        Some((r, hand)) => {
                            tracing::info!("rest: the {} hand points at the seat {:08X}", if hand == HAND_LEFT { "left" } else { "right" }, mem::at::<u32>(r + FORM_ID).unwrap_or(0));
                            haptic(hand, 0.02, 160.0, 0.35);
                        }
                        None => tracing::info!("rest: no seat pointed at"),
                    }
                }
            }
            POINTED.store(s.pointed.map_or(0, |p| p.0), Ordering::Release);
        });
    }

    /// The seat the game has sat the player on (the top of the furniture under where it put their
    /// feet), the way it faces them, and the floor's height there.
    fn find_seat() -> Option<(Vec3, f32, f32)> {
        let at = feet()?;
        let heading = mem::at::<f32>(engine::player() + engine::refr::ROT + 8).filter(|h| h.is_finite())?;
        let fwd = Vec3::new(heading.sin(), 0.0, heading.cos());
        let mut best: Option<(f32, usize, Vec3)> = None;
        for r in furniture_near(at, NEAR) {
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
        // (The player's standing eye height as the runtime's body has it, and the world's scale.)
        let (height, wpm) = crate::link::with_adapter(|a| (a.read_body().map(|b| b.eye_height), a.tracking.last_view().map_or(1.0, |v| v.world_per_meter))).unwrap_or((None, 1.0));
        with_state(|s| {
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
            let at = seated_eye(seat, height, wpm);
            let recenter = !std::mem::replace(&mut s.recentred, true);
            if recenter {
                tracing::info!(
                    "rest: the view recentred on the seat, facing {:.0} degrees: the eye {:.2} m over it (the player's standing eye height {height:.2?} m; the game's eye {:.2} m off it)",
                    heading.to_degrees(),
                    at.y - seat.y,
                    at.distance(eye)
                );
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
                ride.seat = Pose { pos: view.to_tracking(seat).to_array(), valid: 1, rot: glam::Quat::from_rotation_y(heading - view.body_yaw).to_array() };
                ride.floor = view.to_tracking(seat.with_y(floor)).y;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square `size` across at `height`, round (0, 0), as two triangles.
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
    fn the_eye_seated_as_the_body_sits() {
        let eye = seated_eye(Vec3::new(1.0, 0.45, 2.0), Some(1.6), 1.0);
        assert!((eye.y - (0.45 + 1.6 * SEATED_EYE)).abs() < 1e-5 && eye.x == 1.0 && eye.z == 2.0);
        // A sitting body's eyes about 80 cm over the seat; not measured yet, a grown-up's.
        assert!((0.75..0.9).contains(&(eye.y - 0.45)), "{eye}");
        assert!((seated_eye(Vec3::ZERO, None, 1.0).y - EYE_HEIGHT * SEATED_EYE).abs() < 1e-5);
    }

    #[test]
    fn a_hand_held_out_points_ahead() {
        // Eyes 1.6 m up looking along +Z, the right hand held out before the right shoulder.
        let head = Vec3::new(0.0, 1.6, 0.0);
        let dir = glove_dir(head, Quat::IDENTITY, Vec3::new(SHOULDER_OUT, 1.35, 0.5), false);
        assert!(dir.dot(Vec3::Z) > 0.99, "{dir}");
        assert!(in_view(head, Quat::IDENTITY, Vec3::new(0.0, 1.0, 3.0)));
        assert!(!in_view(head, Quat::IDENTITY, Vec3::new(0.0, 1.0, -3.0)), "behind");
        assert!(!in_view(head, Quat::IDENTITY, Vec3::new(4.0, 1.0, 1.0)), "off to the side");
    }

    #[test]
    fn a_seat_pointed_at() {
        let head = Vec3::new(0.0, 1.6, 0.0);
        // A seat 3 m ahead, its top 45 cm up, 40 cm across; the hand pointing at it.
        let tris: Vec<[Vec3; 3]> = square(0.45, 0.4).into_iter().map(|t| t.map(|p| p + Vec3::Z * 3.0)).collect();
        let centre = Vec3::new(0.0, 0.45, 3.0);
        let hand = Vec3::new(0.2, 1.3, 0.4);
        let at = (centre - hand).normalize();
        let hit = pointing_at(&tris, centre, hand, at, head, Quat::IDENTITY, false).unwrap();
        assert!(ray_hit(&tris, hand, at, POINT_RANGE).is_some() && hit < 0.01, "under the line: {hit}");
        // A little off it (10 degrees), still in the cone; 25 off, not.
        // (Swung sideways about the line, by that much.)
        let off = |deg: f32| Quat::from_axis_angle(at.cross(Vec3::X).normalize(), deg.to_radians()) * at;
        assert!(pointing_at(&tris, centre, hand, off(10.0), head, Quat::IDENTITY, false).is_some());
        assert!(pointing_at(&tris, centre, hand, off(25.0), head, Quat::IDENTITY, false).is_none());
        // Pointed at already, it holds on a little further.
        assert!(pointing_at(&tris, centre, hand, off(18.0), head, Quat::IDENTITY, false).is_none());
        assert!(pointing_at(&tris, centre, hand, off(18.0), head, Quat::IDENTITY, true).is_some());
        // Too far.
        let far: Vec<[Vec3; 3]> = tris.iter().map(|t| t.map(|p| p + Vec3::Z * 8.0)).collect();
        assert!(pointing_at(&far, centre + Vec3::Z * 8.0, hand, (centre + Vec3::Z * 8.0 - hand).normalize(), head, Quat::IDENTITY, false).is_none());
    }
}
