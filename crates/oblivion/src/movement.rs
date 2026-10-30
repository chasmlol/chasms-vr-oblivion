//! The player's walking, as the stick has it. Oblivion moves an actor by its animation's own stride
//! (`Actor::Move`, 0x5EFAE0: the frame's step, in the actor's own space, +Y ahead and +X to its
//! right, which `MobileObject::Move` turns to the world and gives its character controller), and
//! its walking keys go only eight ways. Let go of them, its walk or run blends out over a tenth of a
//! second still striding at full pace: the player runs on 10 to 20 cm after the stick is let go,
//! faster than it was walking.
//!
//! So on the ground the player's step is the stick's: its direction, whichever way it points, and
//! the game's own pace for how it's moving now (walking, running, sneaking, laden: its movement
//! speed, 0x5E65B0) times how far it's pushed; let go, no step at all. Jumping, falling and swimming
//! stay the game's (the character controller in the air, state 1 or 2 at +0x1EC, or the swimming
//! flag 0x800), as does the step's rise and fall.
//!
//! The game's body is wider than the player's: up against a table, its edge is 30 cm from the
//! player's eyes, and they couldn't reach what's on it. So whatever of the step the player asked
//! for the game's body didn't go (held up on something, or slid along it), the view goes on
//! ([`lead`]), as far as the player's own body has room that way: up to something low (a table, a
//! counter) until their front is at it, their head over it; otherwise (a wall, a door, someone)
//! their head's breadth from it. The world's fixed things say which ([`crate::collision`]);
//! anything else, the view stays inside the game's body. Walking on freely, it comes back onto it.

use glam::Vec3;

/// How far round the player the game's body reaches (metres). Its character controller's shape
/// (made at 0x895190) is round, as wide as the middle of the actor's box: the skeleton's `BBX`
/// (half 23.1 by 17.6 units across), so 20.4 units times the actor's scale, and Havok's 0.1 (0.7
/// units) round that. At least 0.30 m, then; a little less, to be sure of being inside it.
pub const BODY_RADIUS: f32 = 0.29;
/// How far ahead of their eyes the player's front is at a table's height (metres), and how far from
/// a wall their eyes stay: a head's breadth (the runtime fades the view out as a head that round
/// goes into one).
pub const BODY_FRONT: f32 = 0.05;
pub const HEAD_CLEAR: f32 = 0.12;
/// Walking on freely, the view comes back onto the game's body by this much of the way walked;
/// with less room than it's led by (it turned from a table to a wall), at this speed (m/s).
pub const RETURN: f32 = 0.2;
pub const BACK_SPEED: f32 = 1.0;
/// Where the world's fixed things are looked for from the eyes' line: heights over the feet
/// (metres, every 2.5 cm: a table's top is a few centimetres thick) below the head, round the eyes'
/// height for the head, either side of the line (metres), and how far (metres).
pub const LOW_FROM: f32 = 0.35;
pub const HEAD_BELOW: f32 = 0.15;
pub const HEAD_ABOVE: f32 = 0.1;
pub const LOOK_EVERY: f32 = 0.025;
pub const LOOK_SIDE: f32 = 0.12;
pub const LOOK: f32 = 0.6;

/// How far the view may lead the game's body (metres) one way, from what the world's fixed things
/// are that way from the eyes' line: `low` the nearest below the head (a table, a counter), `head`
/// the nearest at its height; `ahead` how far the eyes already are ahead of the body's middle that
/// way.
pub fn lead_room(low: Option<f32>, head: Option<f32>, ahead: f32) -> f32 {
    let head_room = head.map_or(f32::MAX, |h| h - HEAD_CLEAR);
    let room = match low {
        // Something low, nearer than anything at the head's height: the player's front up to it,
        // their head over it.
        Some(l) if head.is_none_or(|h| l < h) => (l - BODY_FRONT).min(BODY_RADIUS - ahead),
        // Else what held the game's body isn't one of the world's fixed things (a door, someone):
        // the head stays its breadth inside the game's body.
        _ => BODY_RADIUS - HEAD_CLEAR - ahead,
    };
    room.min(head_room).max(0.0)
}

/// The view's lead after a frame (metres, along the floor), before it's held to the room there is:
/// `asked` the step the player asked of the game's body (none: the game moved it, falling, pushed),
/// `went` the step it took. What it didn't go, the view does; walking on, it comes back onto the
/// body by [`RETURN`] of the way.
pub fn lead_step(lead: Vec3, asked: Vec3, went: Vec3) -> Vec3 {
    let short = if asked.length() > 1e-5 { asked - went } else { Vec3::ZERO };
    let lead = lead + Vec3::new(short.x, 0.0, short.z);
    lead.clamp_length_max((lead.length() - RETURN * went.length()).max(0.0))
}

/// The frame's step in the actor's own space (x right, y ahead, game units) for the stick (x right,
/// y ahead, its length 0..1 how far it's pushed) at the game's `speed` (units a second).
pub fn step(stick: [f32; 2], speed: f32, dt: f32) -> [f32; 2] {
    let m = stick[0].hypot(stick[1]);
    if !(m > 1e-4) || !(dt > 0.0) || !speed.is_finite() {
        return [0.0, 0.0];
    }
    let k = m.min(1.0) / m * speed.max(0.0) * dt;
    [stick[0] * k, stick[1] * k]
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::{install, lead, set_stick};

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::engine::{self, refr};
    use crate::{camera, collision, mem};
    use glam::Vec3;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const ACTOR_MOVE: usize = 0x005E_FAE0;
    /// `Actor::GetCharController` (thiscall), its state (+0x1EC), and the movement speed.
    const CHAR_CONTROLLER: usize = 0x0065_A2C0;
    const CC_STATE: usize = 0x1EC;
    const CC_JUMPING: u32 = 1;
    const CC_FALLING: u32 = 2;
    const MOVEMENT_SPEED: usize = 0x005E_65B0;
    /// The process's movement flags (virtual +0x2C0): swimming.
    const PROCESS: usize = 0x58;
    const VT_MOVEMENT_FLAGS: usize = 0x2C0;
    const SWIMMING: u16 = 0x800;

    static REAL: AtomicUsize = AtomicUsize::new(0);
    /// The runtime's stick this frame, while in VR (None: the game walks as it would).
    static STICK: Mutex<Option<[f32; 2]>> = Mutex::new(None);
    static LOGGED: AtomicUsize = AtomicUsize::new(0);
    /// The steps asked of the player's body since the view last took them (the world, game units).
    static ASKED: Mutex<[f32; 2]> = Mutex::new([0.0; 2]);

    /// The view's lead (the runtime's world), where the feet were last frame, and when it was last
    /// logged.
    struct Lead {
        lead: Vec3,
        feet: Option<Vec3>,
        said: f64,
    }
    static LEAD: Mutex<Lead> = Mutex::new(Lead { lead: Vec3::ZERO, feet: None, said: f64::MIN });

    pub fn set_stick(stick: Option<[f32; 2]>) {
        *STICK.lock().unwrap() = stick;
    }

    /// Where the view stands ahead of the game's body this frame (the runtime's world, metres),
    /// for the eyes at `eye`; see the module's notes.
    pub fn lead(eye: Vec3, dt: f32, now: f64) -> Vec3 {
        let asked = std::mem::take(&mut *ASKED.lock().unwrap());
        let player = engine::player();
        let b = camera::basis();
        let feet = (player != 0).then(|| mem::at::<[f32; 3]>(player + refr::POS)).flatten().map(|p| b.point_from_game(Vec3::from(p)));
        let mut l = LEAD.lock().unwrap();
        let went = feet.zip(l.feet).map(|(f, last)| Vec3::new(f.x - last.x, 0.0, f.z - last.z));
        l.feet = feet;
        // (A jump of the body, a load or a door, puts the view back on it.)
        let (Some(feet), Some(went)) = (feet, went.filter(|w| w.length() < 0.25)) else {
            l.lead = Vec3::ZERO;
            return Vec3::ZERO;
        };
        let asked = b.point_from_game(Vec3::new(asked[0], asked[1], 0.0));
        let mut lead = super::lead_step(l.lead, Vec3::new(asked.x, 0.0, asked.z), went);
        if let Some(dir) = lead.try_normalize() {
            // The world's fixed things that way from the eyes' line, below the head and at it.
            let side = Vec3::new(-dir.z, 0.0, dir.x) * super::LOOK_SIDE;
            let line = |from: f32, to: f32| {
                let n = ((to - from) / super::LOOK_EVERY).ceil().max(0.0) as usize;
                (0..=n).flat_map(move |i| {
                    let at = Vec3::new(eye.x, from + i as f32 * super::LOOK_EVERY, eye.z);
                    [at - side, at, at + side]
                })
            };
            let head_from = eye.y - super::HEAD_BELOW;
            let low = collision::nearest_along(&line(feet.y + super::LOW_FROM, head_from).collect::<Vec<_>>(), dir, super::LOOK);
            let head = collision::nearest_along(&line(head_from, eye.y + super::HEAD_ABOVE).collect::<Vec<_>>(), dir, super::LOOK);
            let ahead = Vec3::new(eye.x - feet.x, 0.0, eye.z - feet.z).dot(dir);
            let room = super::lead_room(low, head, ahead);
            // (Less room than it's led by, it comes back at a pace, never all at once.)
            lead = lead.clamp_length_max(room.max(l.lead.length() - super::BACK_SPEED * dt.max(0.0)));
            if lead.length() > 0.02 && now - l.said > 2.0 {
                l.said = now;
                tracing::info!(
                    "movement: the view {:.2} m past the game's body, the room that way {room:.2} m (the eyes {ahead:.2} m ahead of its middle; the world's fixed things that way: below the head {low:.2?}, at it {head:.2?})",
                    lead.length()
                );
            }
        }
        l.lead = lead;
        lead
    }

    fn on_ground_walking(actor: usize) -> bool {
        type Get = unsafe extern "thiscall" fn(usize) -> usize;
        type Flags = unsafe extern "thiscall" fn(usize) -> u16;
        let cc = unsafe { std::mem::transmute::<usize, Get>(CHAR_CONTROLLER)(actor) };
        let state = if cc != 0 { mem::at::<u32>(cc + CC_STATE).unwrap_or(CC_FALLING) } else { CC_FALLING };
        let process = mem::deref(actor + PROCESS);
        let f = mem::deref(mem::deref(process) + VT_MOVEMENT_FLAGS);
        let flags = if process != 0 && f != 0 { unsafe { std::mem::transmute::<usize, Flags>(f)(process) } } else { SWIMMING };
        cc != 0 && state != CC_JUMPING && state != CC_FALLING && flags & SWIMMING == 0
    }

    unsafe extern "thiscall" fn actor_move(this: usize, dt: f32, delta: *mut [f32; 3], flags: u32) -> usize {
        type Move = unsafe extern "thiscall" fn(usize, f32, *mut [f32; 3], u32) -> usize;
        let real: Move = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        if this == engine::player() && !delta.is_null() {
            let stick = *STICK.lock().unwrap();
            crate::hooks::guard_void("player step", || {
                let Some(stick) = stick else { return };
                if !on_ground_walking(this) {
                    return;
                }
                type Speed = unsafe extern "thiscall" fn(usize) -> f32;
                let speed = unsafe { std::mem::transmute::<usize, Speed>(MOVEMENT_SPEED)(this) };
                let [x, y] = super::step(stick, speed, dt);
                // (Its own space in the world: +Y ahead is (sin, cos) of its heading, +X to its
                // right (cos, -sin).)
                let h = mem::at::<[f32; 3]>(this + refr::ROT).map_or(0.0, |r| r[2]);
                let mut asked = ASKED.lock().unwrap();
                asked[0] += x * h.cos() + y * h.sin();
                asked[1] += y * h.cos() - x * h.sin();
                drop(asked);
                let d = unsafe { &mut *delta };
                let n = LOGGED.fetch_add(1, Ordering::Relaxed);
                if (stick[0] != 0.0 || stick[1] != 0.0) && n % 600 == 0 {
                    tracing::info!("movement: the game's step {:.2?} for {dt:.4} s; the stick's {:.2?} at {speed:.0} units/s", [d[0], d[1]], [x, y]);
                }
                d[0] = x;
                d[1] = y;
            });
        }
        unsafe { real(this, dt, delta, flags) }
    }

    /// The player's step from the stick, from now on (while VR gives it one).
    pub fn install() {
        if crate::hooks::install_into(ACTOR_MOVE as *const (), actor_move as *const (), "Actor::Move", &REAL) {
            tracing::info!("movement: the player walks as the stick points");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_step_is_the_sticks_way_at_the_games_pace() {
        // Full ahead at 270 units a second for 10 ms: 2.7 units ahead.
        let [x, y] = step([0.0, 1.0], 270.0, 0.01);
        assert!(x.abs() < 1e-6 && (y - 2.7).abs() < 1e-4);
        // Half way to the front right: half the pace, that way.
        let s = std::f32::consts::FRAC_1_SQRT_2 * 0.5;
        let [x, y] = step([s, s], 270.0, 0.01);
        assert!((x - y).abs() < 1e-6 && (x.hypot(y) - 1.35).abs() < 1e-4);
        // Let go: nothing (no running on).
        assert_eq!(step([0.0, 0.0], 270.0, 0.01), [0.0, 0.0]);
        // Never past full pace.
        assert!((step([0.0, 1.4], 100.0, 0.01)[1] - 1.0).abs() < 1e-5);
    }
}
