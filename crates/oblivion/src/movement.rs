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
//! Swimming, the arms' strokes are the step ([`crate::swim::stroke_step`]): the game's own swimming
//! moves the player at the strokes' velocity, the stick's swimming added while it's pushed.

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
pub use win::{install, set_stick, set_swim};

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::{engine, mem};
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
    /// The arms' strokes this frame, while the runtime swims the player: their velocity in the
    /// world (game units a second).
    static SWIM: Mutex<Option<[f32; 3]>> = Mutex::new(None);
    static LOGGED: AtomicUsize = AtomicUsize::new(0);

    pub fn set_stick(stick: Option<[f32; 2]>) {
        *STICK.lock().unwrap() = stick;
    }

    pub fn set_swim(v: Option<[f32; 3]>) {
        *SWIM.lock().unwrap() = v;
    }

    /// Swimming by the arms: the frame's step is the strokes', the stick's swimming (the game's
    /// own step) added while it's pushed, never further than the game's speed takes the player.
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
                if crate::swim::swimming(this) {
                    if let Some(v) = *SWIM.lock().unwrap() {
                        swim_step(this, dt, unsafe { &mut *delta }, v, stick);
                    }
                    return;
                }
                if !on_ground_walking(this) {
                    return;
                }
                type Speed = unsafe extern "thiscall" fn(usize) -> f32;
                let speed = unsafe { std::mem::transmute::<usize, Speed>(MOVEMENT_SPEED)(this) };
                let [x, y] = super::step(stick, speed, dt);
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
