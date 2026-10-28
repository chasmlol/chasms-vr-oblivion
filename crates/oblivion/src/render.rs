//! The eyes. Oblivion's world render (`Main::Render`, [`crate::engine::RENDER`]) runs once per eye
//! each game frame, its camera put where that eye is ([`crate::camera`]); after each, the back
//! buffer holds that eye's finished picture (world, first person, HDR) and is copied out
//! ([`crate::d3d9::capture`]). Then the back buffer is cleared, so what the game draws over it before
//! it presents is its HUD and menus alone: that goes to the runtime as the HUD layer, with the eyes,
//! in one slot ([`on_present`]).
//!
//! Outside the world (the main menu, loading screens) the render isn't called: the whole picture
//! goes to the runtime as a menu panel.

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::camera::{self, CameraState};
    use crate::d3d9::{self, HUD};
    use crate::link::{self, GameFrame, Snapshot};
    use crate::{engine, hooks};
    use rustvr_protocol::{DEPTH_LINEAR, SLOT_DEPTH, SLOT_HUD, SLOT_MENU, SLOT_STEREO, SlotInfo};
    use std::ffi::c_void;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use windows::Win32::Graphics::Direct3D9::D3DCLEAR_TARGET;

    static REAL_RENDER: AtomicUsize = AtomicUsize::new(0);
    type RenderFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;

    /// This game frame's eyes, waiting for the HUD at Present.
    struct Eyes {
        snap: Snapshot,
        captured: [bool; 2],
        /// Each eye's depth was copied, and the projection's near and far planes (game units).
        depth: [bool; 2],
        near_far: (f32, f32),
    }
    static EYES: Mutex<Option<Eyes>> = Mutex::new(None);
    static LAST: Mutex<f64> = Mutex::new(0.0);

    static REAL_FRAME: AtomicUsize = AtomicUsize::new(0);
    type FrameFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;

    /// The game's background thread (0xB3F940: it loads and changes the scene graph, cells
    /// and actors' 3D) shares the scene with the render through a semaphore: the world render takes
    /// it at its start (`mov eax, [B3F940]` at 0x40C8F7, then its wait) and gives it back at its end
    /// (`mov esi, [B3F940]` at 0x40D059, then its release). Rendered once per eye, the thread could
    /// run between the eyes and change the scene under the second. Those two places read these
    /// instead (patched at install): the first eye takes the lock and keeps it, the second gives it
    /// back; a single render does both, as the game does.
    static LOCK_TAKE: AtomicUsize = AtomicUsize::new(0);
    static LOCK_GIVE: AtomicUsize = AtomicUsize::new(0);
    const BACKGROUND: usize = 0x00B3_F940;
    const TAKE_AT: usize = 0x0040_C8F7;
    const GIVE_AT: usize = 0x0040_D059;

    fn patch_lock() {
        let addr = |a: &AtomicUsize| (a as *const AtomicUsize as u32).to_le_bytes();
        let [b0, b1, b2, b3] = (BACKGROUND as u32).to_le_bytes();
        let [t0, t1, t2, t3] = addr(&LOCK_TAKE);
        let [g0, g1, g2, g3] = addr(&LOCK_GIVE);
        let take = crate::mem::patch_code(TAKE_AT, &[0xA1, b0, b1, b2, b3], &[0xA1, t0, t1, t2, t3]);
        let give = crate::mem::patch_code(GIVE_AT, &[0x8B, 0x35, b0, b1, b2, b3], &[0x8B, 0x35, g0, g1, g2, g3]);
        if take && give {
            tracing::info!("render: both eyes are rendered under one hold of the background thread's lock");
        } else {
            tracing::error!("render: the background thread's lock isn't held across the eyes (patched: take {take}, give {give})");
        }
    }

    /// The lock as the next render takes and gives it.
    fn set_lock(take: bool, give: bool) {
        let bg = crate::mem::deref(BACKGROUND);
        LOCK_TAKE.store(if take { bg } else { 0 }, Ordering::Release);
        LOCK_GIVE.store(if give { bg } else { 0 }, Ordering::Release);
    }

    /// The actors' shadows (0x4073D0, thiscall, called at the start of the world render): it picks
    /// the actors that cast shadows, renders their shadow maps (from the lights: the same for
    /// both eyes) and hides the caster of a shadow that has faded out. Run a second time in the
    /// same frame it left actors hidden from the second eye, so it runs once a frame, with the
    /// first eye; the second eye uses the same shadow maps.
    const ACTOR_SHADOWS: usize = 0x0040_73D0;
    static REAL_SHADOWS: AtomicUsize = AtomicUsize::new(0);
    /// The eye being rendered is the frame's second (its shadows are done).
    static SECOND_EYE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    unsafe extern "thiscall" fn actor_shadows(this: *mut c_void) -> i32 {
        if SECOND_EYE.load(Ordering::Acquire) {
            return 0;
        }
        let real: unsafe extern "thiscall" fn(*mut c_void) -> i32 = unsafe { std::mem::transmute(REAL_SHADOWS.load(Ordering::Acquire)) };
        unsafe { real(this) }
    }

    pub fn install() {
        set_lock(true, true);
        patch_lock();
        hooks::install_into(ACTOR_SHADOWS as *const (), actor_shadows as *const (), "actor shadows", &REAL_SHADOWS);
        hooks::install_into(engine::FRAME as *const (), frame_hook as *const (), "Main::Frame", &REAL_FRAME);
        hooks::install_into(engine::RENDER as *const (), render_hook as *const (), "Main::Render", &REAL_RENDER);
    }

    /// The game's frame: test commands run first, where the game's own scripts and console would
    /// (a save can be loaded from here, not from inside the render or Present).
    unsafe extern "thiscall" fn frame_hook(this: *mut c_void) -> i32 {
        let real: FrameFn = unsafe { std::mem::transmute(REAL_FRAME.load(Ordering::Acquire)) };
        static FIRST: std::sync::Once = std::sync::Once::new();
        FIRST.call_once(|| tracing::info!("render: the game's frames run"));
        hooks::guard_void("Frame", || {
            crate::console::tick();
            crate::player::before_update();
        });
        unsafe { real(this) }
    }

    unsafe extern "thiscall" fn render_hook(this: *mut c_void, arg: i32) -> i32 {
        let real: RenderFn = unsafe { std::mem::transmute(REAL_RENDER.load(Ordering::Acquire)) };
        // A single render takes and gives the background thread's lock, and does its shadows, as
        // the game's does.
        set_lock(true, true);
        SECOND_EYE.store(false, Ordering::Release);
        let stereo = hooks::guard("Render", None, prepare);
        let Some((snap, game, cam)) = stereo else { return unsafe { real(this, arg) } };
        let view = snap.view.expect("prepared with a view");
        let Some(dev) = d3d9::device(engine::device() as *mut c_void) else { return unsafe { real(this, arg) } };
        let mut result = 0;
        let mut captured = [false; 2];
        let mut depth = [false; 2];
        // The game's own first-person arms aren't drawn: the runtime draws the player's hands.
        // (Testing, `RUSTVR_OBLIVION_GAME_ARMS=1`: they are, to hold the runtime's lighting of the
        // same armour up to the game's own.)
        static GAME_ARMS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let keep = *GAME_ARMS.get_or_init(|| std::env::var("RUSTVR_OBLIVION_GAME_ARMS").is_ok_and(|v| v == "1"));
        let arms = crate::player::first_person_node();
        let arms_flags = if keep { None } else { crate::mem::at::<u16>(arms + engine::av::FLAGS) };
        if let Some(f) = arms_flags {
            unsafe { ((arms + engine::av::FLAGS) as *mut u16).write(f | engine::av::APP_CULLED) };
        }
        static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let log = LOGGED.fetch_add(1, Ordering::Relaxed) % 600 == 30;
        for eye in 0..2 {
            let s = camera::eye_camera(&view, &snap.frame.eyes[eye], &snap.frame.fov[eye], &game);
            if log {
                tracing::info!(
                    "render: eye {eye}: tracking pos {:?} rot {:?} -> game pos {:?} dir {:?} up {:?} frustum {:?}; game camera pos {:?} dir {:?}; view anchor {:?} yaw {} wpm {}",
                    snap.frame.eyes[eye].pos,
                    snap.frame.eyes[eye].rot,
                    s.pos,
                    s.dir(),
                    s.up(),
                    s.frustum,
                    game.pos,
                    game.dir(),
                    view.anchor,
                    view.body_yaw,
                    view.world_per_meter
                );
            }
            camera::write(cam, &s);
            set_lock(eye == 0, eye == 1);
            SECOND_EYE.store(eye == 1, Ordering::Release);
            result = unsafe { real(this, arg) };
            captured[eye] = hooks::guard("capture", false, || link::with_adapter(|a| d3d9::capture(&dev, a.gpu(), eye)).unwrap_or(false));
            depth[eye] = hooks::guard("capture depth", false, || link::with_adapter(|a| d3d9::capture_depth(&dev, a.gpu(), eye)).unwrap_or(false));
        }
        set_lock(true, true);
        SECOND_EYE.store(false, Ordering::Release);
        camera::write(cam, &game);
        if let Some(f) = arms_flags {
            unsafe { ((arms + engine::av::FLAGS) as *mut u16).write(f) };
        }
        hooks::guard_void("Render end", || {
            // The menus are drawn next: alone on black, they're the HUD layer.
            let _ = unsafe { dev.Clear(0, std::ptr::null(), D3DCLEAR_TARGET as u32, 0, 1.0, 0) };
            *EYES.lock().unwrap() = Some(Eyes { snap, captured, depth, near_far: (game.frustum[4], game.frustum[5]) });
        });
        result
    }

    /// The runtime's frame and the game's camera, when the eyes are to be rendered.
    fn prepare() -> Option<(Snapshot, CameraState, usize)> {
        let cam = engine::world_camera();
        let game = camera::read(cam)?;
        let now = link::now();
        let dt = {
            let mut last = LAST.lock().unwrap();
            let dt = if *last == 0.0 { 1.0 / 90.0 } else { (now - *last) as f32 };
            *last = now;
            dt.clamp(0.0, 0.25)
        };
        // (A menu over the world: the runtime shows the menus as a panel to point at.)
        let menu = crate::menus::menu_mode();
        let snap = link::game_frame(&GameFrame { eye: Some(game.eye()), look_yaw: game.look_yaw(), menu, now, dt })?;
        snap.view?;
        log_camera_once(&game);
        crate::combat::publish();
        hooks::guard_void("collision", || crate::collision::update(now));
        if let Some(view) = snap.view.as_ref() {
            let head = glam::Vec3::from(snap.frame.hmd.pos);
            hooks::guard_void("lighting", || crate::lighting::publish(view, head));
        }
        Some((snap, game, cam))
    }

    fn log_camera_once(game: &CameraState) {
        static DONE: std::sync::Once = std::sync::Once::new();
        DONE.call_once(|| tracing::info!("render: the game's camera: rot {:?}, pos {:?}, frustum {:?}", game.rot, game.pos, game.frustum));
    }

    /// Present: the eyes and the HUD to the runtime; outside the world, the whole picture as a
    /// menu panel.
    pub fn on_present(dev_raw: *mut c_void) {
        let Some(dev) = d3d9::device(dev_raw) else { return };
        if let Some((w, h)) = d3d9::backbuffer_size(&dev) {
            link::set_size(w, h);
        }
        let aspect = d3d9::backbuffer_size(&dev).map(|(w, h)| w as f32 / h.max(1) as f32).unwrap_or(1.0);
        let eyes = EYES.lock().unwrap().take();
        match eyes {
            Some(e) if e.captured == [true, true] => stereo(&dev, &e, aspect),
            _ => menu(&dev, aspect),
        }
    }

    fn stereo(dev: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9, e: &Eyes, aspect: f32) {
        let Some(view) = e.snap.view else { return };
        link::with_adapter(|a| {
            let gpu = a.gpu().clone();
            let hud = d3d9::capture(dev, &gpu, HUD);
            let mut have_hud = false;
            let mut have_depth = false;
            {
                let Some(slot) = a.begin_slot() else { return };
                for eye in 0..2 {
                    if !d3d9::deliver(&gpu, eye, &slot.eyes[eye].tex) {
                        return;
                    }
                }
                if let (true, Some(dst)) = (hud, slot.hud.as_ref()) {
                    have_hud = d3d9::deliver(&gpu, HUD, &dst.tex);
                }
                // The game's depth, which the runtime hides what it draws behind.
                if let (true, Some(d)) = (e.depth == [true, true], slot.depth.as_ref()) {
                    have_depth = (0..2).all(|eye| d3d9::deliver_depth(&gpu, eye, &d[eye], e.near_far.0, e.near_far.1));
                }
            }
            let mut info: SlotInfo = bytemuck::Zeroable::zeroed();
            info.frame_index = e.snap.frame.frame_index;
            info.eye_pose = e.snap.frame.eyes;
            info.fov = e.snap.frame.fov;
            info.eye_map = [view.slot_map(); 2];
            info.flags = SLOT_STEREO | if have_hud { SLOT_HUD } else { 0 } | if have_depth { SLOT_DEPTH } else { 0 };
            if have_depth {
                info.depth_near = e.near_far.0 / crate::camera::UNITS_PER_METER;
                info.depth_far = e.near_far.1 / crate::camera::UNITS_PER_METER;
                info.depth_flags = DEPTH_LINEAR;
            }
            info.hud_aspect = aspect;
            static LAYOUT_SENT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if have_hud && !LAYOUT_SENT.swap(true, Ordering::AcqRel) {
                a.write_hud_layout(&crate::hud::layout());
            }
            a.submit(info);
        });
    }

    fn menu(dev: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9, aspect: f32) {
        // Keeps the link alive and says the game is in its menus.
        let _ = link::game_frame(&GameFrame { eye: None, look_yaw: 0.0, menu: true, now: link::now(), dt: 0.0 });
        link::with_adapter(|a| {
            let gpu = a.gpu().clone();
            if !d3d9::capture(dev, &gpu, HUD) {
                return;
            }
            {
                let Some(slot) = a.begin_slot() else { return };
                let Some(dst) = slot.hud.as_ref() else { return };
                if !d3d9::deliver(&gpu, HUD, &dst.tex) {
                    return;
                }
            }
            let mut info: SlotInfo = bytemuck::Zeroable::zeroed();
            info.flags = SLOT_MENU | SLOT_HUD;
            info.hud_aspect = aspect;
            a.submit(info);
        });
    }
}
