//! Oblivion's water, for the runtime's swimming by the arms (`rustvr_protocol::WaterState`), and the
//! strokes given back to the game's own swimming ([`crate::movement`]). Research and design:
//! `docs/research/swimming.md`.
//!
//! The game decides it all, as it always does: an actor 70% under its cell's water swims (the
//! process's movement flag 0x800), 87.5% under it holds its breath (the actor's update, 0x603CA0,
//! asking `IsUnderWater` of its height). The water's height is the cell's (0x4CACE0: none is
//! −FLT_MAX). The swim speed is the game's for the player's Speed, Athletics, Swim Speed effects and
//! load (0x5E3920 swimming, 0x5E3AD0 running: `fMoveSwim*`), and breath is its own timer against
//! its maximum for the player's Endurance (0x548960: `fActorSwimBreath*`). Water Breathing (actor
//! value 55) and god mode breathe; Water Walking (56) walks on the water instead.

use rustvr_protocol::{WATER_HEAD_UNDER, WATER_HERE, WATER_SWIMMING, WATER_WALKING, WaterState};

/// The share of the player's height under the surface the game counts as their head under.
pub const HEAD_UNDER: f32 = 0.875;

/// The game's water, as the runtime is told it, and what it says of the player's.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Water {
    /// The surface's height (game units), if the cell has water.
    pub height: Option<f32>,
    pub swimming: bool,
    pub head_under: bool,
    pub walking: bool,
    /// Breath left, 0..1.
    pub breath: f32,
    /// The game's swim speed (units a second).
    pub speed: f32,
}

impl Water {
    /// For the runtime: the surface in the world's metres (the protocol's Y is the game's Z).
    pub fn state(&self) -> WaterState {
        let mut w = WaterState::default();
        let Some(h) = self.height else { return w };
        w.flags = WATER_HERE
            | if self.swimming { WATER_SWIMMING } else { 0 }
            | if self.head_under { WATER_HEAD_UNDER } else { 0 }
            | if self.walking { WATER_WALKING } else { 0 };
        w.surface = h / crate::camera::UNITS_PER_METER;
        w.swim_speed = self.speed / crate::camera::UNITS_PER_METER;
        w.breath = self.breath.clamp(0.0, 1.0);
        w
    }
}

/// The step the strokes' velocity `v` (the world's game units a second: x east, y north, z up) takes
/// in the player's own frame (x right, y ahead, z up) over `dt`, for a player heading `heading`
/// (0 north, turning east) and pitched `pitch` (positive looking down). The game's swimming turns
/// its step down by the pitch (so the stick dives where the head looks); the strokes' step is turned
/// back up by it first, to go where the hands pushed.
pub fn stroke_step(v: [f32; 3], heading: f32, pitch: f32, dt: f32) -> [f32; 3] {
    let (s, c) = heading.sin_cos();
    let local = [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]];
    let (sp, cp) = pitch.sin_cos();
    [local[0] * dt, (local[1] * cp - local[2] * sp) * dt, (local[1] * sp + local[2] * cp) * dt]
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::engine::{self, refr};
    use crate::mem;
    use std::sync::Mutex;

    const PARENT_CELL: usize = 0x40;
    const CELL_WATER_HEIGHT: usize = 0x004C_ACE0;
    /// The reference's height: its bounds times its scale (thiscall).
    const HEIGHT: usize = 0x005E_0660;
    const PROCESS: usize = 0x58;
    const VT_MOVEMENT_FLAGS: usize = 0x2C0;
    const SWIMMING: u16 = 0x800;
    /// `Actor::GetActorValue` (vtable), the process's breath (vtable: seconds left), the maximum for
    /// an Endurance (cdecl).
    const VT_ACTOR_VALUE: usize = 0x284;
    const VT_BREATH: usize = 0x1F4;
    const MAX_BREATH: usize = 0x0054_8960;
    const ENDURANCE: i32 = 5;
    const WATER_BREATHING: i32 = 55;
    const WATER_WALKING_AV: i32 = 56;
    const GOD_MODE: usize = 0x00B3_BB06;
    /// The swim speeds (thiscall on the actor).
    const SWIM_WALK: usize = 0x005E_3920;
    const SWIM_RUN: usize = 0x005E_3AD0;

    static LAST: Mutex<Option<Water>> = Mutex::new(None);
    /// The menus by id (cdecl: the menu's tile, or 0), and the breath meter's (`BreathMenu`).
    const MENU_TILE_BY_ID: usize = 0x0058_9B70;
    const BREATH_MENU: i32 = 1045;
    static METER: Mutex<Option<[f32; 4]>> = Mutex::new(None);

    /// The breath meter's place on the HUD, when it's newly showing there or has moved (for the
    /// HUD's layout): the game makes its meter when the player goes into the water and shows it
    /// while their head's under.
    pub fn meter_moved() -> Option<[f32; 4]> {
        let tile = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(i32) -> usize>(MENU_TILE_BY_ID)(BREATH_MENU) };
        let now = crate::hud_info::tile_uv(tile, 4.0, true)?;
        let mut last = METER.lock().unwrap();
        let far = last.is_none_or(|l| l.iter().zip(now).any(|(a, b)| (a - b).abs() > 0.01));
        if far {
            *last = Some(now);
            tracing::info!("hud: the game's breath meter is at {now:.3?} of its HUD");
        }
        far.then_some(now)
    }

    pub fn meter() -> Option<[f32; 4]> {
        *METER.lock().unwrap()
    }

    /// Under the water or above it, as the headset's eyes are. The renderer's underwater state (its
    /// shaders' underwater variants, the water's surface seen from below) is decided in the world
    /// render itself (the water's update, 0x49E880, called from `Main::Render`), from the camera:
    /// each eye's, as each is rendered. But the fog (the sky's update, 0x542F20: under when the
    /// player's cell's water is above the scene graph's first camera, then the water's own fog colour
    /// and distances for the depth) is decided once a frame, before the eyes, from the game's own
    /// camera: where the game has the player's eye, which in VR is the headset's only until the head
    /// moves. Swimming at the surface (the game's eye half a metre above the water) a player who
    /// ducks their face into it saw the water clear as air. So for the sky's update the camera is
    /// where the headset's eyes are.
    const SKY_UPDATE: usize = 0x0054_2F20;
    /// The scene graph's cameras (an array and its count) and a camera's world height.
    const SCENE_CAMERAS: usize = 0xB0;
    const SCENE_CAMERA_COUNT: usize = 0xB6;
    const WORLD_Z: usize = crate::engine::av::WORLD_TRANSLATE + 8;
    static REAL_SKY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    /// The headset's eyes' height in the world (game units), while in VR.
    static HEAD_Z: Mutex<Option<f32>> = Mutex::new(None);

    pub fn set_head(z: Option<f32>) {
        *HEAD_Z.lock().unwrap() = z.filter(|z| z.is_finite());
    }

    /// (Thiscall with one argument, passed on as it is.)
    unsafe extern "thiscall" fn sky_update(sky: usize, arg: u32) {
        let real: unsafe extern "thiscall" fn(usize, u32) = unsafe { std::mem::transmute(REAL_SKY.load(std::sync::atomic::Ordering::Acquire)) };
        let head = *HEAD_Z.lock().unwrap();
        let sg = mem::deref(engine::SCENE_GRAPH);
        let cam = if sg != 0 && mem::at::<u16>(sg + SCENE_CAMERA_COUNT).unwrap_or(0) > 0 { mem::deref(mem::deref(sg + SCENE_CAMERAS)) } else { 0 };
        let was = (cam != 0).then(|| mem::at::<f32>(cam + WORLD_Z)).flatten();
        if let (Some(z), Some(_)) = (head, was) {
            unsafe { ((cam + WORLD_Z) as *mut f32).write(z) };
        }
        unsafe { real(sky, arg) };
        if let (Some(_), Some(w)) = (head, was) {
            unsafe { ((cam + WORLD_Z) as *mut f32).write(w) };
        }
    }

    pub fn install() {
        if crate::hooks::install_into(SKY_UPDATE as *const (), sky_update as *const (), "Sky::Update", &REAL_SKY) {
            tracing::info!("swim: the sky's underwater fog is decided where the headset's eyes are");
        }
    }

    fn call_f32(f: usize, this: usize) -> f32 {
        unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> f32>(f)(this) }
    }

    fn actor_value(actor: usize, av: i32) -> i32 {
        let f = mem::deref(mem::deref(actor) + VT_ACTOR_VALUE);
        if f == 0 {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, i32) -> i32>(f)(actor, av) }
    }

    /// The process's movement flags, if the actor has a process.
    pub fn movement_flags(actor: usize) -> Option<u16> {
        let process = mem::deref(actor + PROCESS);
        let f = mem::deref(mem::deref(process) + VT_MOVEMENT_FLAGS);
        if process == 0 || !(0x0040_1000..0x00A0_0000).contains(&f) {
            return None;
        }
        Some(unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> u16>(f)(process) })
    }

    /// The game has the player swimming.
    pub fn swimming(actor: usize) -> bool {
        movement_flags(actor).is_some_and(|f| f & SWIMMING != 0)
    }

    /// The player's water now; `running`: the player swims at the game's running pace (Always Run,
    /// or Run held).
    pub fn read(running: bool) -> Option<Water> {
        let p = engine::player();
        let cell = mem::deref(p + PARENT_CELL);
        let pos = mem::at::<[f32; 3]>(p + refr::POS)?;
        if p == 0 || cell == 0 {
            return None;
        }
        let h = call_f32(CELL_WATER_HEIGHT, cell);
        let mut w = Water { height: (h.is_finite() && h > -1.0e30).then_some(h), ..Default::default() };
        let Some(height) = w.height else { return Some(w) };
        w.swimming = swimming(p);
        w.head_under = height > pos[2] + call_f32(HEIGHT, p) * HEAD_UNDER;
        w.walking = actor_value(p, WATER_WALKING_AV) > 0;
        w.speed = call_f32(if running { SWIM_RUN } else { SWIM_WALK }, p).max(0.0);
        let process = mem::deref(p + PROCESS);
        let get = mem::deref(mem::deref(process) + VT_BREATH);
        let max = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(i32) -> f32>(MAX_BREATH)(actor_value(p, ENDURANCE)) };
        let breathes = actor_value(p, WATER_BREATHING) > 0 || mem::at::<u8>(GOD_MODE).unwrap_or(0) != 0;
        w.breath = if breathes || !(max > 0.0) || get == 0 { 1.0 } else { (call_f32(get, process) / max).clamp(0.0, 1.0) };
        Some(w)
    }

    /// The water this frame for the runtime (its log when the player goes in, swims, goes under or
    /// comes out).
    pub fn state(running: bool) -> WaterState {
        let Some(w) = read(running) else { return WaterState::default() };
        let mut last = LAST.lock().unwrap();
        let changed = last.is_none_or(|l| (l.height.is_some(), l.swimming, l.head_under, l.walking) != (w.height.is_some(), w.swimming, w.head_under, w.walking));
        if changed {
            let p = engine::player();
            tracing::info!(
                "swim: water {:?} (the player at {:?}, turned {:?}), swimming {}, head under {}, water walking {}, breath {:.2}, swim speed {:.0} units/s",
                w.height,
                mem::at::<[f32; 3]>(p + refr::POS),
                mem::at::<[f32; 3]>(p + refr::ROT),
                w.swimming,
                w.head_under,
                w.walking,
                w.breath,
                w.speed
            );
        }
        *last = Some(w);
        w.state()
    }
}
