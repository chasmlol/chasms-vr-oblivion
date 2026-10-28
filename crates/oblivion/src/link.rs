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

/// The eyes and a HUD panel, the size of Oblivion's back buffer, in its format (BGRA).
fn request(w: u32, h: u32) -> TargetRequest {
    const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 87;
    TargetRequest { color_format: DXGI_FORMAT_B8G8R8A8_UNORM, flags: TGT_HUD | TGT_DEPTH, eye_w: w, eye_h: h, hud_w: w, hud_h: h, ..Default::default() }
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
    pub now: f64,
    pub dt: f32,
}

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
    let view = g.eye.and_then(|eye| {
        let anchor = Anchor { eye, base_eye: eye, look_yaw: g.look_yaw, scripted: false };
        adapter.tracking.update(&frame, &anchor, g.now, g.dt)
    });
    let mut gs: GameState = bytemuck::Zeroable::zeroed();
    gs.flags = GS_PLAYER_RIG | if view.is_some() { GS_IN_GAME } else { 0 } | if g.menu { GS_MENU } else { 0 };
    gs.game_zoom = 1.0;
    if view.is_some() {
        gs.light = crate::lighting::hint();
    }
    adapter.write_state(&mut gs);
    let snap = Snapshot { frame, view };
    *LAST.lock().unwrap() = Some(snap);
    Some(snap)
}
