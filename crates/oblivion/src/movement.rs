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
//! **Up to people, not into them:** the game's people are its character controllers' capsules, a
//! good deal rounder than the bodies they hold (each sized from its model's bound, 0x895190), and
//! walked into someone, the player's capsule shoves theirs away: the player couldn't come close to
//! anyone, their hands could, their body not. So in VR the player's capsule, and those of the people
//! near them, are a body's round (`pockets`' torso), as the game's own `SetSize` makes them (the
//! console's, 0x501760: the radius at +0x3A8, Havok units, which the controller grows or shrinks to
//! over its next updates, 0x894C70; 0 for its own, +0x3A0). Each of the player's steps on the
//! ground, the stick's or room-scale's after the head, stops where its capsule would touch another's:
//! body to body, without shoving. People further off have their own again.
//!
//! The game's body is wider than the player's: up against a table, its edge is 30 cm from the
//! player's eyes, and they couldn't reach what's on it. So whatever of the step the player asked
//! for the game's body didn't go (held up on something, or slid along it), the view goes on
//! ([`lead`]), as far as the player's own body has room that way: up to something low (a table, a
//! counter) until their front is at it, their head over it; otherwise (a wall, a door, someone)
//! their head's breadth from it. The world's fixed things say which ([`crate::collision`]);
//! anything else, the view stays inside the game's body. Walking on freely, it comes back onto it.
//!
//! Swimming, the arms' strokes are the step ([`crate::swim::stroke_step`]): the game's own swimming
//! moves the player at the strokes' velocity, the stick's swimming added while it's pushed.

use glam::Vec3;

/// A step `d` (the world's x and y) kept from walking into others (`others`: where each is from the
/// walker, and how far apart their middles are when they touch): its part towards any it would come
/// nearer than touching is taken out.
pub fn clear_of(mut d: [f32; 2], others: &[([f32; 2], f32)]) -> [f32; 2] {
    for &(v, touch) in others {
        let dist = v[0].hypot(v[1]);
        if !(dist > 1e-3) {
            continue;
        }
        let n = [v[0] / dist, v[1] / dist];
        let toward = d[0] * n[0] + d[1] * n[1];
        let room = (dist - touch).max(0.0);
        if toward > room {
            d = [d[0] - n[0] * (toward - room), d[1] - n[1] * (toward - room)];
        }
    }
    d
}

/// How far round the player the game's body reaches (metres). Its character controller's shape
/// (made at 0x895190) is round, as wide as the middle of the actor's box: the skeleton's `BBX`
/// (half 23.1 by 17.6 units across), so 20.4 units times the actor's scale, and Havok's 0.1 (0.7
/// units) round that. At least 0.30 m, then; a little less, to be sure of being inside it. (In VR
/// the player's capsule is a body's round, `fit_people`: the view still goes this far, the
/// closest the player gets to things.)
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
pub use win::{fit_people, install, lead, set_stick, set_swim};

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::collision::{self, HAVOK_SCALE};
    use crate::engine::{self, refr};
    use crate::{camera, mem};
    use glam::Vec3;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    const ACTOR_MOVE: usize = 0x005E_FAE0;
    /// `Actor::GetCharController` (thiscall), its state (+0x1EC), and the movement speed.
    const CHAR_CONTROLLER: usize = 0x0065_A2C0;
    const CC_STATE: usize = 0x1EC;
    /// The character controller's radius (Havok units), the one it has of its own (as its model's
    /// bound has it), and `SetSize` (thiscall, the radius or 0 for its own: the console's, 0x501760).
    const CC_RADIUS: usize = 0x3A8;
    const CC_OWN_RADIUS: usize = 0x3A0;
    const SET_SIZE: usize = 0x0089_4BD0;
    /// `Actor`: dead (vtable +0x198, 0), knocked out (+0x19C).
    const VT_IS_DEAD: usize = 0x198;
    const VT_KNOCKED_OUT: usize = 0x19C;
    /// `TESObjectREFR`'s base form, `TESForm`'s type: people (NPCs, not creatures) are a body's round.
    const REFR_BASE: usize = 0x1C;
    const FORM_TYPE: usize = 0x4;
    const NPC: u8 = 0x23;
    /// A body's round (game units): `pockets`' torso, the one the hands meet on others.
    const BODY: f32 = crate::pockets::BODY * crate::camera::UNITS_PER_METER;
    /// How far others are looked at for touching, and made a body's round (game units): 3 m.
    const NEAR: f32 = 210.0;
    /// How far off they have their own round again (game units): a metre further, not back and
    /// forth at the edge.
    const FAR: f32 = 280.0;
    /// How far above or below the player someone's feet can be and still be in their way (game
    /// units): a person's height, the game's own for a model with no bound (64 each way of its
    /// middle, 0x895190).
    const TALL: f32 = 128.0;
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
    /// The arms' strokes this frame, while the runtime swims the player: their velocity in the
    /// world (game units a second).
    static SWIM: Mutex<Option<[f32; 3]>> = Mutex::new(None);
    static LOGGED: AtomicUsize = AtomicUsize::new(0);
    /// Who the body last stopped at, and who was last made a body's round, for the log; whether the
    /// player's capsule has been said.
    static STOPPED_AT: AtomicUsize = AtomicUsize::new(0);
    static FITTED: AtomicUsize = AtomicUsize::new(0);
    static PLAYER_SAID: AtomicBool = AtomicBool::new(false);
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

    pub fn set_swim(v: Option<[f32; 3]>) {
        *SWIM.lock().unwrap() = v;
    }

    /// Swimming by the arms: the frame's step is the strokes', the stick's swimming (the game's
    /// own step) added while it's pushed, never further than the faster of the two takes the player.
    fn swim_step(this: usize, dt: f32, d: &mut [f32; 3], v: [f32; 3], stick: [f32; 2]) {
        let Some(rot) = mem::at::<[f32; 3]>(this + crate::engine::refr::ROT) else { return };
        let s = crate::swim::stroke_step(v, rot[2], rot[0], dt);
        let mut out = if stick[0].hypot(stick[1]) > 0.05 { [d[0] + s[0], d[1] + s[1], d[2] + s[2]] } else { s };
        type Speed = unsafe extern "thiscall" fn(usize) -> f32;
        let speed = unsafe { std::mem::transmute::<usize, Speed>(MOVEMENT_SPEED)(this) };
        let cap = speed.max((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()) * dt;
        let len = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt();
        if len > cap && len > 0.0 {
            out = out.map(|c| c * cap / len);
        }
        let n = LOGGED.fetch_add(1, Ordering::Relaxed);
        if n % 120 == 0 && out != [0.0; 3] {
            let at = mem::at::<[f32; 3]>(this + crate::engine::refr::POS);
            let eye = crate::camera::read(crate::engine::world_camera()).map(|c| c.pos[2]);
            tracing::info!("swim: the game's step {:.2?}, the strokes' {:.2?} for {dt:.4} s (pitch {:.2}, heading {:.2}), the player at {at:.1?}, the game's camera at {eye:.1?}", [d[0], d[1], d[2]], out, rot[0], rot[2]);
        }
        *d = out;
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

    fn controller(actor: usize) -> usize {
        type Get = unsafe extern "thiscall" fn(usize) -> usize;
        unsafe { std::mem::transmute::<usize, Get>(CHAR_CONTROLLER)(actor) }
    }

    /// Dead or knocked out (`Actor::IsDead` with 0, `IsKnockedOut`): a body on the ground is walked
    /// over, not stopped at.
    fn down(actor: usize) -> bool {
        type IsDead = unsafe extern "thiscall" fn(usize, u32) -> u8;
        type KnockedOut = unsafe extern "thiscall" fn(usize) -> u8;
        let vt = mem::deref(actor);
        let (dead, out) = (mem::deref(vt + VT_IS_DEAD), mem::deref(vt + VT_KNOCKED_OUT));
        dead == 0
            || out == 0
            || unsafe { std::mem::transmute::<usize, IsDead>(dead)(actor, 0) } != 0
            || unsafe { std::mem::transmute::<usize, KnockedOut>(out)(actor) } != 0
    }

    /// An actor's capsule's radius (game units), if it has one: the one it's growing or shrinking to.
    fn radius(actor: usize) -> Option<f32> {
        let cc = controller(actor);
        let r = (cc != 0).then(|| mem::at::<f32>(cc + CC_RADIUS)).flatten()?;
        (r.is_finite() && r > 0.0).then_some(r * HAVOK_SCALE)
    }

    /// Makes an actor's capsule `round` (game units, never rounder than its own), or its own again
    /// (None), through the game's `SetSize`, unless it is already: its radius before, if it's changed.
    fn fit(actor: usize, round: Option<f32>) -> Option<f32> {
        let cc = controller(actor);
        if cc == 0 {
            return None;
        }
        let (now, own) = (mem::at::<f32>(cc + CC_RADIUS)?, mem::at::<f32>(cc + CC_OWN_RADIUS)?);
        if !(own > 0.0) {
            return None;
        }
        let want = round.map_or(own, |r| (r / HAVOK_SCALE).min(own));
        if (now - want).abs() < 1e-3 {
            return None;
        }
        type SetSize = unsafe extern "thiscall" fn(usize, f32);
        unsafe { std::mem::transmute::<usize, SetSize>(SET_SIZE)(cc, if want < own { want } else { 0.0 }) };
        Some(now * HAVOK_SCALE)
    }

    fn is_person(actor: usize) -> bool {
        mem::at::<u8>(mem::deref(actor + REFR_BASE) + FORM_TYPE) == Some(NPC)
    }

    /// Once a frame: in VR, the player and the people near them a body's round, those further off
    /// their own; out of it, the player's own again.
    pub fn fit_people(vr: bool) {
        let player = engine::player();
        if player == 0 {
            return;
        }
        if !vr {
            fit(player, None);
            return;
        }
        if let Some(was) = fit(player, Some(BODY))
            && !PLAYER_SAID.swap(true, Ordering::Relaxed)
        {
            tracing::info!("movement: the player's capsule is a body's round, {BODY:.1} units (the game's {was:.1})");
        }
        let Some(me) = mem::at::<[f32; 3]>(player + refr::POS) else { return };
        for a in crate::combat::high_actors() {
            if !is_person(a) {
                continue;
            }
            let Some(p) = mem::at::<[f32; 3]>(a + refr::POS) else { continue };
            let (off, up) = ((p[0] - me[0]).hypot(p[1] - me[1]), (p[2] - me[2]).abs());
            if off < NEAR && up < TALL && !down(a) {
                if let Some(was) = fit(a, Some(BODY))
                    && FITTED.swap(a, Ordering::Relaxed) != a
                {
                    tracing::info!("movement: {a:#x} near the player is a body's round, {BODY:.1} units (the game's {was:.1})");
                }
            } else if off > FAR || up > TALL {
                fit(a, None);
            }
        }
    }

    /// The step (world x and y) of the player at `at`, kept from walking into anyone near: each
    /// actor standing with a character controller, its middle where their capsules would touch.
    fn clear_of_people(player: usize, at: [f32; 3], d: [f32; 2]) -> [f32; 2] {
        let Some(mine) = radius(player) else { return d };
        let others: Vec<(usize, [f32; 2], f32)> = crate::combat::high_actors()
            .into_iter()
            .filter_map(|a| {
                let p = mem::at::<[f32; 3]>(a + refr::POS)?;
                let v = [p[0] - at[0], p[1] - at[1]];
                (v[0].hypot(v[1]) < NEAR && (p[2] - at[2]).abs() < TALL && !down(a)).then_some((a, v, mine + radius(a)?))
            })
            .collect();
        let kept = super::clear_of(d, &others.iter().map(|&(_, v, t)| (v, t)).collect::<Vec<_>>());
        if kept != d
            && let Some(&(who, v, touch)) = others.iter().min_by(|a, b| a.1[0].hypot(a.1[1]).total_cmp(&b.1[0].hypot(b.1[1])))
            && STOPPED_AT.swap(who, Ordering::Relaxed) != who
        {
            tracing::info!("movement: the body stops at {who:#x}, {:.0} units off (they touch at {touch:.0}: the player's {mine:.1} and theirs)", v[0].hypot(v[1]));
        }
        kept
    }

    fn on_ground_walking(actor: usize) -> bool {
        type Flags = unsafe extern "thiscall" fn(usize) -> u16;
        let cc = controller(actor);
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
                if crate::swim::swimming(this) {
                    if let Some(v) = *SWIM.lock().unwrap() {
                        swim_step(this, dt, unsafe { &mut *delta }, v, stick);
                    }
                    return;
                }
                // (On a horse the horse moves, and the game moves its rider with it.)
                if !on_ground_walking(this) || crate::riding::mounted() {
                    return;
                }
                type Speed = unsafe extern "thiscall" fn(usize) -> f32;
                let speed = unsafe { std::mem::transmute::<usize, Speed>(MOVEMENT_SPEED)(this) };
                let [mut x, mut y] = super::step(stick, speed, dt);
                // (What's asked of the body, for the view's lead, before it's kept from people: the
                // view goes on with all of it. Its own space in the world: +Y ahead is (sin, cos)
                // of its heading, +X to its right (cos, -sin).)
                let h = mem::at::<[f32; 3]>(this + refr::ROT).map_or(0.0, |r| r[2]);
                let mut asked = ASKED.lock().unwrap();
                asked[0] += x * h.cos() + y * h.sin();
                asked[1] += y * h.cos() - x * h.sin();
                drop(asked);
                // Up to people, not into them (in the world, by the player's heading: ahead is
                // (sin, cos), to the right (cos, -sin)).
                if let Some(at) = mem::at::<[f32; 3]>(this + refr::POS) {
                    let (s, c) = h.sin_cos();
                    let world = [x * c + y * s, -x * s + y * c];
                    let [wx, wy] = clear_of_people(this, at, world);
                    (x, y) = (wx * c - wy * s, wx * s + wy * c);
                }
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
