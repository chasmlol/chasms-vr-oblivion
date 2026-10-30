//! The player, from the headset: before each game frame the player faces where the head looks (so
//! walking forward goes that way, and the game's activation and aim follow the gaze), and the game's
//! keyboard and mouse read the runtime's actions ([`crate::input`]).

use crate::engine::{self, refr};
use crate::{input, link, mem, settings};
use std::sync::Mutex;

static BRIDGE: Mutex<Option<input::Bridge>> = Mutex::new(None);

fn bridge() -> std::sync::MutexGuard<'static, Option<input::Bridge>> {
    let mut b = BRIDGE.lock().unwrap();
    if b.is_none() {
        let ini = crate::game_settings::ini_path().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
        *b = Some(input::Bridge::new(input::parse_controls(&ini)));
    }
    b
}

/// The player's rotation (x pitch, positive looking down; z heading, 0 north, turning east).
pub fn rotation() -> Option<[f32; 3]> {
    let p = engine::player();
    if p == 0 { None } else { mem::at(p + refr::ROT) }
}

fn set_look(yaw: f32, pitch: f32) {
    let p = engine::player();
    if p == 0 || !mem::is_readable((p + refr::ROT) as *const u8, 12) {
        return;
    }
    let rot = (p + refr::ROT) as *mut f32;
    unsafe {
        rot.write(pitch);
        rot.add(2).write(yaw.rem_euclid(std::f32::consts::TAU));
    }
}

/// Oblivion's physics steps (0x889810, once a frame): by default (`iUpdateType` 0) in fixed steps
/// of `fMaxTime` (1/60 s), or one of the frame's whole time once that's half a step. At a headset's
/// 90 to 120 frames a second that's a step on some frames and none on others, so the player (and
/// all Havok moves) lurches along unevenly, and runs on a step after the stick is let go. Its update
/// type 10 steps once every frame by the frame's time, smoothed by a factor (0xB2E2F0); unsmoothed,
/// it's each frame's own time: everything moves every frame, as far as that frame's time takes it.
const HAVOK_UPDATE_TYPE: usize = 0x00BA_7918;
const HAVOK_SMOOTHING: usize = 0x00B2_E2F0;
const STEP_EVERY_FRAME: u32 = 10;

fn step_physics_every_frame() {
    if mem::at::<u32>(HAVOK_UPDATE_TYPE) == Some(STEP_EVERY_FRAME) && mem::at::<f32>(HAVOK_SMOOTHING) == Some(1.0) {
        return;
    }
    if !mem::is_readable(HAVOK_UPDATE_TYPE as *const u8, 4) || !mem::is_readable(HAVOK_SMOOTHING as *const u8, 4) {
        return;
    }
    unsafe {
        (HAVOK_UPDATE_TYPE as *mut u32).write(STEP_EVERY_FRAME);
        (HAVOK_SMOOTHING as *mut f32).write(1.0);
    }
    tracing::info!("player: the game's physics steps every frame, by the frame's own time");
}

pub fn before_update() {
    let snap = link::last();
    let vr = snap.is_some_and(|s| s.view.is_some());
    if vr || settings::get().background {
        step_physics_every_frame();
    }
    // The player's step on the ground is the stick's (crate::movement).
    #[cfg(target_arch = "x86")]
    crate::movement::set_stick(snap.filter(|_| vr).map(|s| (s.frame.intents.move_xy, s.frame.tracking.flags & rustvr_protocol::TRK_FOLLOW != 0)));
    let mut v = input::Virtual::default();
    if let Some(s) = snap {
        if let Some(b) = bridge().as_mut() {
            v = b.update(&s.frame.intents, game_controls(), link::now());
        }
        static LAST_MOVE: Mutex<[f32; 2]> = Mutex::new([0.0; 2]);
        let mv = s.frame.intents.move_xy;
        let mut last = LAST_MOVE.lock().unwrap();
        if (mv[0] - last[0]).abs() + (mv[1] - last[1]).abs() > 0.2 {
            tracing::info!("player: the runtime's move {mv:?}");
            *last = mv;
        }
        log_movement_toggles();
        trace_movement(mv);
        // The laser on the game's menus moves the game's cursor.
        if s.frame.intents.flags & rustvr_protocol::INT_CURSOR != 0 {
            crate::menus::point(s.frame.intents.cursor_uv);
        }
        if let Some(view) = s.view {
            log_once(view.look_yaw);
            first_person();
            set_look(view.look_yaw, view.look_pitch);
        }
    }
    #[cfg(windows)]
    input::set(v, vr || settings::get().background);
    crate::combat::apply_pending();
}

/// Testing (`RUSTVR_OBLIVION_TRACE_MOVE=1`): for two seconds after the stick changes, the player's
/// place every frame, to see how soon and how fast the game moves the player.
fn trace_movement(mv: [f32; 2]) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("RUSTVR_OBLIVION_TRACE_MOVE").is_ok_and(|v| v == "1")) {
        return;
    }
    static T: Mutex<([f32; 2], f64, Option<[f32; 3]>)> = Mutex::new(([0.0; 2], f64::MIN, None));
    let now = link::now();
    let mut t = T.lock().unwrap();
    if mv != t.0 {
        *t = (mv, now, None);
    }
    if now - t.1 > 2.0 {
        return;
    }
    let p = engine::player();
    let Some(pos) = (p != 0).then(|| mem::at::<[f32; 3]>(p + refr::POS)).flatten() else { return };
    let step = t.2.map_or(0.0, |q| ((pos[0] - q[0]).powi(2) + (pos[1] - q[1]).powi(2)).sqrt());
    tracing::info!("player: trace {:.3} s after the move {:?}: at {:?}, {:.2} units since the last frame", now - t.1, t.0, pos, step);
    t.2 = Some(pos);
}

/// Tests (`RUSTVR_OBLIVION_TRACE_MOVE=1`): every rendered frame, the runtime's move, the game's
/// analog speed (0xB14E58), where the player is, where the game's camera is and where the view
/// puts tracking's origin.
pub fn trace_frame(now: f64, snap: &link::Snapshot, game: &crate::camera::CameraState) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("RUSTVR_OBLIVION_TRACE_MOVE").is_ok_and(|v| v == "1")) {
        return;
    }
    let p = engine::player();
    let pos = (p != 0).then(|| mem::at::<[f32; 3]>(p + refr::POS)).flatten();
    let anchor = snap.view.map(|v| v.anchor);
    // The process's movement flags (virtual 0x2C0) and the actor's movement speed (0x5E65B0).
    #[cfg(target_arch = "x86")]
    let (flags, base) = {
        let process = mem::deref(p + 0x58);
        let f = mem::deref(mem::deref(process) + 0x2C0);
        type Flags = unsafe extern "thiscall" fn(usize) -> u16;
        type Speed = unsafe extern "thiscall" fn(usize) -> f32;
        if process != 0 && f != 0 { unsafe { (std::mem::transmute::<usize, Flags>(f)(process), std::mem::transmute::<usize, Speed>(0x005E_65B0)(p)) } } else { (0, 0.0) }
    };
    #[cfg(not(target_arch = "x86"))]
    let (flags, base) = (0u16, 0.0f32);
    tracing::info!(
        "move: {now:.4} stick {:?} flags {flags:#x} base {base:.1} speed {:?} refr {:?} eye {:?} anchor {:?}",
        snap.frame.intents.move_xy,
        mem::at::<f32>(0x00B1_4E58),
        pos,
        game.pos,
        anchor.map(|a| a.to_array())
    );
}

/// Holds one of the game's controls (by its name in `Oblivion.ini`) for `seconds`.
pub fn hold_control(name: &str, seconds: f64) {
    if let Some(b) = bridge().as_mut() {
        b.hold(name, link::now() + seconds);
    }
}

/// In VR the view is always the player's own eyes: a third-person camera goes back to first.
fn first_person() {
    let p = engine::player();
    if p == 0 || mem::at::<u8>(p + engine::pc::THIRD_PERSON).unwrap_or(0) == 0 {
        return;
    }
    tracing::info!("player: back to first person");
    #[cfg(target_arch = "x86")]
    unsafe {
        type ToggleFn = unsafe extern "thiscall" fn(*mut std::ffi::c_void, u8) -> u8;
        std::mem::transmute::<usize, ToggleFn>(engine::pc::TOGGLE_POV)(p as *mut _, 1);
    }
}

/// The node of the game's own first-person arms (and what they hold), or 0.
pub fn first_person_node() -> usize {
    let p = engine::player();
    if p == 0 { 0 } else { mem::deref(p + engine::pc::FIRST_PERSON_NODE) }
}

fn log_once(yaw: f32) {
    static DONE: std::sync::Once = std::sync::Once::new();
    DONE.call_once(|| tracing::info!("player: the game's rotation {:?}, the head's yaw {yaw}", rotation()));
}

/// The game's own movement toggles (a save can bring Auto Move back on: the player then walks
/// until the stick pushes forward or back, as in the game).
/// The game's state the controls depend on.
fn game_controls() -> input::GameControls {
    let p = engine::player();
    input::GameControls { always_run: p != 0 && mem::at::<u8>(p + engine::pc::ALWAYS_RUN).unwrap_or(0) != 0 }
}

/// The player sneaks (its process's movement flags: `GetMovementFlags`, vtable +0x2C0; sneaking
/// 0x400, which swimming 0x800 overrides).
#[cfg(target_arch = "x86")]
pub fn sneaking() -> bool {
    const PROCESS: usize = 0x58;
    const MOVEMENT_FLAGS: usize = 0x2C0;
    let p = engine::player();
    let process = if p != 0 { mem::deref(p + PROCESS) } else { 0 };
    let vt = if process != 0 { mem::deref(process) } else { 0 };
    let f = if vt != 0 { mem::deref(vt + MOVEMENT_FLAGS) } else { 0 };
    if !(0x0040_1000..0x00A0_0000).contains(&f) {
        return false;
    }
    let flags = unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> u32>(f)(process) };
    flags & 0x400 != 0 && flags & 0x800 == 0
}

#[cfg(not(target_arch = "x86"))]
pub fn sneaking() -> bool {
    false
}

fn log_movement_toggles() {
    static LAST: Mutex<Option<(u8, u8, bool)>> = Mutex::new(None);
    let p = engine::player();
    if p == 0 {
        return;
    }
    let now = (mem::at::<u8>(p + engine::pc::ALWAYS_RUN).unwrap_or(0), mem::at::<u8>(p + engine::pc::AUTO_MOVE).unwrap_or(0), sneaking());
    let mut last = LAST.lock().unwrap();
    if *last != Some(now) {
        tracing::info!("player: always run {}, auto move {}, sneaking {}", now.0, now.1, now.2);
        *last = Some(now);
    }
}
