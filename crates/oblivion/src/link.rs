//! The link to the RustVR runtime: the SDK adapter, the frame the runtime sent, and where that
//! frame puts the headset in Oblivion's world. Everything runs on Oblivion's main thread (the render
//! hook), once per game frame.

use glam::Vec3;
use rustvr_adapter::{Adapter, AdapterConfig, Anchor, View};
use rustvr_gpu::Gpu;
use rustvr_protocol::*;
use std::sync::Mutex;

static ADAPTER: Mutex<Option<Adapter>> = Mutex::new(None);
static LAST: Mutex<Option<Snapshot>> = Mutex::new(None);

/// The latest frame from the runtime (the previous game frame's), for what happens before the
/// game's update (its input, where the player faces).
pub fn last() -> Option<Snapshot> {
    *LAST.lock().unwrap()
}

pub fn now() -> f64 {
    static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    T0.get_or_init(std::time::Instant::now).elapsed().as_secs_f64()
}

/// Creates the SDK adapter on the device the eyes are copied with ([`crate::d3d9`]).
pub fn init(gpu: Gpu, w: u32, h: u32) {
    let mut a = ADAPTER.lock().unwrap();
    if a.is_none() {
        let mut cfg = AdapterConfig::new("oblivion");
        cfg.request = request(w, h);
        *a = Some(Adapter::new(gpu, cfg));
    }
}

/// The eyes and a HUD panel, the size of Oblivion's back buffer, in its format (BGRA), and the
/// mirror's world with its depth (the aux view, the eyes' size: [`crate::render`]).
fn request(w: u32, h: u32) -> TargetRequest {
    const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 87;
    TargetRequest { color_format: DXGI_FORMAT_B8G8R8A8_UNORM, flags: TGT_HUD | TGT_DEPTH | TGT_AUX | TGT_AUX_DEPTH, eye_w: w, eye_h: h, hud_w: w, hud_h: h, ..Default::default() }
}

pub fn set_size(w: u32, h: u32) {
    with_adapter(|a| {
        if (a.cfg.request.eye_w, a.cfg.request.eye_h) != (w, h) {
            a.cfg.request = request(w, h);
        }
    });
}

pub fn is_initialised() -> bool {
    ADAPTER.lock().unwrap().is_some()
}

pub fn with_adapter<R>(f: impl FnOnce(&mut Adapter) -> R) -> Option<R> {
    ADAPTER.lock().unwrap().as_mut().map(f)
}

/// What the game is doing this frame, read before the eyes are rendered.
pub struct GameFrame {
    /// The game's own camera: the player's eye and where they look (None outside the world).
    pub eye: Option<Vec3>,
    pub look_yaw: f32,
    pub menu: bool,
    /// The race menu is open: the player stays in the world for it, before the mirror, its menu
    /// the part of the picture `menu_uv` says.
    pub race_menu: Option<[f32; 4]>,
    /// The game's own first-person bob ([`crate::camera::first_person_bob`]): in `eye`, and held
    /// out of the headset's view.
    pub bob: Option<crate::camera::Bob>,
    /// The game's camera, which sees the bob (None outside the world, and in the race menu).
    pub camera: Option<crate::camera::CameraState>,
    pub now: f64,
    pub dt: f32,
}

/// The most of the game's bob held out of the headset's view (metres). Further (a dodge's dip is
/// 0.26 m), the adapter SDK would take the gap between the eye and the eye without it for a scripted
/// camera and follow it; the rest of a dip that deep goes with the view, never a jump.
const BOB_HELD: f32 = 0.2;

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub frame: RtFrame,
    /// Tracking space -> Oblivion's world for this frame; None outside the world.
    pub view: Option<View>,
}

/// One game frame: the runtime's latest frame, the tracking mapping, the game's state.
pub fn game_frame(g: &GameFrame) -> Option<Snapshot> {
    let mut guard = ADAPTER.lock().unwrap();
    let adapter = guard.as_mut()?;
    let frame = adapter.poll()?;
    let mut view_bob = ViewBob::default();
    let view = g.eye.and_then(|eye| {
        // On a horse the view turns with it, and is anchored on the rider's eye over the saddle held
        // steady on it (crate::riding): just seated, it faces the horse's way.
        let ride = crate::riding::carry(eye, g.dt);
        if let Some(c) = &ride {
            adapter.tracking.carry_turn(c.turn);
            if c.recenter {
                adapter.tracking.recenter();
            }
        }
        // (Up against something the game's body is held up on, the view goes on: crate::movement.
        // Not on a horse: the view is the rider's eye over the saddle.)
        #[cfg(all(windows, target_arch = "x86"))]
        adapter.tracking.set_lead(if ride.is_some() { Vec3::ZERO } else { crate::movement::lead(eye, g.dt, g.now) });
        // The game's own bob: on foot its first-person animations' (held out of the headset's view,
        // up to `BOB_HELD`), on a horse the gait's under the rider's eye. Given to the runtime for
        // its desktop view, as the game's camera sees it.
        let held = g.bob.map_or(Vec3::ZERO, |b| b.offset.clamp_length_max(BOB_HELD));
        let anchor = Anchor {
            eye: ride.as_ref().map_or(eye, |c| c.eye),
            base_eye: ride.as_ref().map_or(eye - held, |c| c.base_eye),
            look_yaw: ride.as_ref().map_or(g.look_yaw, |c| c.look_yaw),
            scripted: false,
        };
        if let Some(cam) = &g.camera {
            match (&ride, g.bob) {
                (Some(c), _) => view_bob = crate::camera::view_bob(cam, c.eye - c.base_eye, 0.0, 0.0),
                (None, Some(b)) => view_bob = crate::camera::view_bob(cam, b.offset, b.yaw, b.pitch),
                (None, None) => {}
            }
        }
        adapter.tracking.update(&frame, &anchor, g.now, g.dt)
    });
    let mut gs: GameState = bytemuck::Zeroable::zeroed();
    gs.flags = GS_PLAYER_RIG | if view.is_some() { GS_IN_GAME } else { 0 } | if g.menu { GS_MENU } else { 0 };
    // (The runtime keeps its crouch in step with the game's sneaking.)
    if view.is_some() && crate::player::sneaking() {
        gs.flags |= GS_CROUCHED;
    }
    if let (Some(uv), true) = (g.race_menu, view.is_some()) {
        gs.flags |= GS_MENU_IN_WORLD | GS_MIRROR;
        gs.menu_uv = uv;
    }
    gs.game_zoom = 1.0;
    if view.is_some() {
        gs.view_bob = view_bob;
        gs.light = crate::lighting::hint();
        // The game's water, for swimming by the arms.
        #[cfg(target_arch = "x86")]
        {
            gs.water = crate::hooks::guard("water", Default::default(), || crate::swim::state(crate::player::running(&frame.intents)));
        }
    }
    adapter.write_state(&mut gs);
    let snap = Snapshot { frame, view };
    *LAST.lock().unwrap() = Some(snap);
    Some(snap)
}
