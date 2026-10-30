//! The eyes. Oblivion's world render (`Main::Render`, [`crate::engine::RENDER`]) runs once per eye
//! each game frame, its camera put where that eye is ([`crate::camera`]); after each, the back
//! buffer holds that eye's finished picture (world, first person, HDR) and is copied out
//! ([`crate::d3d9::capture`]). Then the back buffer is cleared, so what the game draws over it before
//! it presents is its HUD and menus alone: that goes to the runtime as the HUD layer, with the eyes,
//! in one slot ([`on_present`]).
//!
//! Outside the world (the main menu, loading screens) the render isn't called: the whole picture
//! goes to the runtime as a menu panel.
//!
//! **Under the game's menus the world goes on.** Opening most menus, the game renders the world once
//! into a texture and from then on draws that frozen picture behind them ([`world_picture`]); in VR
//! the head goes on moving, so the picture isn't taken and the world is rendered for both eyes every
//! frame, as the game itself does under dialogue. The menus float in it ([`crate::hud`]).
//!
//! **The UI has real alpha.** The back buffer is made with an alpha channel ([`crate::d3d9`]); it's
//! cleared to clear after the eyes, and while the game draws its UI over it the alpha is blended
//! as coverage ([`crate::d3d9::ui_pass`]): the HUD texture is the UI as the game drew it,
//! premultiplied, over nothing.

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::camera::{self, CameraState};
    use crate::d3d9::{self, HUD};
    use crate::link::{self, GameFrame, Snapshot};
    use crate::{engine, hooks, mem};
    use rustvr_protocol::{DEPTH_LINEAR, MIR_RENDER, SLOT_AUX, SLOT_DEPTH, SLOT_HUD, SLOT_MENU, SLOT_MIRROR, SLOT_STEREO, SlotInfo};
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
        /// The mirror's world was rendered and copied with its depth, and its near and far planes.
        mirror: Option<(f32, f32)>,
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

    /// The game's redraw from the outdoor cell grid (thiscall), which it also runs as soon as its
    /// lost device is reset: it reads the grid's middle cell without asking if there is one, and
    /// indoors there isn't (a crash on any device reset inside a building; the device is lost when
    /// the desktop's displays change, as Virtual Desktop adds its monitor when the headset
    /// connects). Skipped with no middle cell.
    const GRID_REDRAW: usize = 0x0049_A200;
    /// `GridCellArray::GetCell(x, y)` (thiscall): the slot holding the cell at (x, y) of the outdoor
    /// grid (`TES` +8).
    const GRID_CELL: usize = 0x0048_2150;
    const TES: usize = 0x00B3_33A0;
    /// The texture the game draws the world into behind its menus (`Main::Render`'s argument then).
    const WORLD_TEXTURE: usize = 0x00B3_33E8;
    /// A reference's 3D (`TESObjectREFR` +0x3C): the player's third-person body.
    const PLAYER_3D: usize = 0x3C;
    static REAL_GRID_REDRAW: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "thiscall" fn grid_redraw(this: *mut c_void) {
        let grid = mem::deref(mem::deref(TES) + 8);
        let middle = grid != 0 && {
            let cell_at: unsafe extern "thiscall" fn(usize, i32, i32) -> usize = unsafe { std::mem::transmute(GRID_CELL) };
            let slot = unsafe { cell_at(grid, 1, 1) };
            slot != 0 && mem::deref(slot) != 0
        };
        if !middle {
            static SAID: std::sync::Once = std::sync::Once::new();
            SAID.call_once(|| tracing::info!("render: the game's redraw from the outdoor grid skipped indoors (it would crash)"));
            return;
        }
        let real: unsafe extern "thiscall" fn(*mut c_void) = unsafe { std::mem::transmute(REAL_GRID_REDRAW.load(Ordering::Acquire)) };
        unsafe { real(this) }
    }

    /// The world's frozen picture behind the game's menus (0x40D160, thiscall on `OSGlobals`), which
    /// the frame takes on opening a menu (the call at 0x40DA62; at 0x40DEF5 for sleep and wait):
    /// rendered once into a texture (0xB333E8), it's what 0x40D4D0 draws behind the menus from
    /// then on (while 0xB33397 says it's there) instead of rendering the world. In VR it isn't
    /// taken: with it never there, 0x40D4D0 renders the world, both eyes, every frame. (Its other
    /// callers, a door's or a save's picture, are the game's as ever.)
    const WORLD_PICTURE: usize = 0x0040_D160;
    const WORLD_PICTURE_CALLS: [(usize, [u8; 5]); 2] = [(0x0040_DA62, [0xE8, 0xF9, 0xF6, 0xFF, 0xFF]), (0x0040_DEF5, [0xE8, 0x66, 0xF2, 0xFF, 0xFF])];

    unsafe extern "thiscall" fn world_picture(this: *mut c_void) {
        if link::last().is_some_and(|s| s.view.is_some()) {
            static SAID: std::sync::Once = std::sync::Once::new();
            SAID.call_once(|| tracing::info!("render: a menu opened: the world goes on being rendered under it (no frozen picture)"));
            return;
        }
        let real: unsafe extern "thiscall" fn(*mut c_void) = unsafe { std::mem::transmute(WORLD_PICTURE) };
        unsafe { real(this) }
    }

    fn patch_world_picture() {
        let ours = world_picture as *const () as usize;
        let mut done = 0;
        for (at, expect) in WORLD_PICTURE_CALLS {
            debug_assert_eq!(i32::from_le_bytes([expect[1], expect[2], expect[3], expect[4]]), WORLD_PICTURE as i32 - (at as i32 + 5));
            let rel = (ours as i32).wrapping_sub(at as i32 + 5).to_le_bytes();
            done += mem::patch_code(at, &expect, &[0xE8, rel[0], rel[1], rel[2], rel[3]]) as u32;
        }
        tracing::info!("render: the menus' frozen picture of the world is skipped in VR ({done} of 2 calls)");
    }

    pub fn install() {
        set_lock(true, true);
        patch_lock();
        patch_world_picture();
        hooks::install_into(ACTOR_SHADOWS as *const (), actor_shadows as *const (), "actor shadows", &REAL_SHADOWS);
        hooks::install_into(engine::FRAME as *const (), frame_hook as *const (), "Main::Frame", &REAL_FRAME);
        hooks::install_into(engine::RENDER as *const (), render_hook as *const (), "Main::Render", &REAL_RENDER);
        hooks::install_into(GRID_REDRAW as *const (), grid_redraw as *const (), "the outdoor grid's redraw", &REAL_GRID_REDRAW);
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
            crate::items::frame(link::now());
            crate::archery::frame(link::now());
            crate::grabs::apply(link::now());
            crate::world::frame(link::now());
            crate::props::frame(link::now());
            crate::riding::frame(link::now());
            crate::books::frame();
            crate::inventory::frame(link::now());
            crate::repair::frame();
            crate::soul::frame();
            crate::enchant::frame(link::now());
            crate::sigil::frame(link::now());
            crate::ayleid::frame();
            crate::shrines::frame(link::now());
            crate::rest::frame();
            crate::slot_menu::frame();
            crate::pockets::frame(link::now());
            crate::alchemy::frame(link::now());
        });
        unsafe { real(this) }
    }

    unsafe extern "thiscall" fn render_hook(this: *mut c_void, arg: i32) -> i32 {
        let real: RenderFn = unsafe { std::mem::transmute(REAL_RENDER.load(Ordering::Acquire)) };
        // A picture of the world into a texture of the game's own making (a save's picture, taken
        // as an autosave goes through a door) is the game's, drawn as it draws it: two eyes drawn
        // into it, read back at its size, crashed it. (Its frame passes 0; the world behind its
        // menus goes into its world texture.)
        d3d9::rendering(true);
        let result = if arg != 0 && arg as usize != mem::deref(WORLD_TEXTURE) {
            unsafe { real(this, arg) }
        } else {
            crate::timing::render_start();
            // (The world's own blending, not the UI's.)
            if let Some(dev) = d3d9::device(engine::device() as *mut c_void) {
                d3d9::ui_pass(&dev, false);
            }
            let r = unsafe { render_stereo(this, arg, real) };
            crate::timing::render_end();
            r
        };
        d3d9::rendering(false);
        result
    }

    unsafe fn render_stereo(this: *mut c_void, arg: i32, real: RenderFn) -> i32 {
        use crate::timing::{Part, time};
        // A single render takes and gives the background thread's lock, and does its shadows, as
        // the game's does.
        set_lock(true, true);
        SECOND_EYE.store(false, Ordering::Release);
        let stereo = time(Part::Prepare, || hooks::guard("Render", None, prepare));
        let Some((snap, game, cam)) = stereo else { return unsafe { real(this, arg) } };
        let view = snap.view.expect("prepared with a view");
        hooks::guard_void("reticle", crate::reticle::hide_crosshair);
        hooks::guard_void("hud info action", crate::hud_info::keep_action);
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
        // Nor, while the race menu shows the player from outside, the game's own third-person
        // player: the runtime draws the player, in the world and in the mirror.
        let race_menu = crate::menus::menu_open(crate::menus::MENU_RACE_SEX);
        let outside = if race_menu { mem::deref(engine::player() + PLAYER_3D) } else { 0 };
        let outside_flags = if outside != 0 { crate::mem::at::<u16>(outside + engine::av::FLAGS) } else { None };
        if let Some(f) = outside_flags {
            unsafe { ((outside + engine::av::FLAGS) as *mut u16).write(f | engine::av::APP_CULLED) };
        }
        // The mirror's world, rendered after the eyes from the camera the runtime asked for: the
        // head reflected in the glass, looking through it, its near plane on the glass.
        let mirror = snap.frame.mirror;
        let mirror_cam = (mirror.flags & MIR_RENDER != 0 && mirror.camera.valid != 0 && mirror.near > 0.0).then(|| {
            let mut s = camera::eye_camera(&view, &mirror.camera, &mirror.fov, &game);
            s.frustum[4] = (mirror.near * view.world_per_meter * camera::UNITS_PER_METER).min(s.frustum[5] * 0.5);
            s
        });
        let renders = if mirror_cam.is_some() { 3 } else { 2 };
        static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let log = LOGGED.fetch_add(1, Ordering::Relaxed) % 600 == 30;
        let mut mirror_done = None;
        for eye in 0..renders {
            if eye == 2 {
                let s = mirror_cam.expect("a third render is the mirror's");
                camera::write(cam, &s);
                set_lock(false, true);
                SECOND_EYE.store(true, Ordering::Release);
                result = time(Part::Eye, || unsafe { real(this, arg) });
                let colour = hooks::guard("capture mirror", false, || link::with_adapter(|a| d3d9::capture(&dev, a.gpu(), d3d9::MIRROR)).unwrap_or(false));
                let depth = hooks::guard("capture mirror depth", false, || link::with_adapter(|a| d3d9::capture_depth(&dev, a.gpu(), 2)).unwrap_or(false));
                mirror_done = (colour && depth).then_some((s.frustum[4], s.frustum[5]));
                continue;
            }
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
            set_lock(eye == 0, eye + 1 == renders);
            SECOND_EYE.store(eye == 1, Ordering::Release);
            result = time(Part::Eye, || unsafe { real(this, arg) });
            captured[eye] = time(Part::Capture, || hooks::guard("capture", false, || link::with_adapter(|a| d3d9::capture(&dev, a.gpu(), eye)).unwrap_or(false)));
            depth[eye] = time(Part::Depth, || hooks::guard("capture depth", false, || link::with_adapter(|a| d3d9::capture_depth(&dev, a.gpu(), eye)).unwrap_or(false)));
        }
        set_lock(true, true);
        SECOND_EYE.store(false, Ordering::Release);
        camera::write(cam, &game);
        if let Some(f) = arms_flags {
            unsafe { ((arms + engine::av::FLAGS) as *mut u16).write(f) };
        }
        if let Some(f) = outside_flags {
            unsafe { ((outside + engine::av::FLAGS) as *mut u16).write(f) };
        }
        hooks::guard_void("Render end", || {
            // The menus are drawn next: alone over clear, with their alpha, they're the HUD layer.
            let _ = unsafe { dev.Clear(0, std::ptr::null(), D3DCLEAR_TARGET as u32, 0, 1.0, 0) };
            d3d9::ui_pass(&dev, true);
            crate::menus::hide_menu_3d(true);
            *EYES.lock().unwrap() = Some(Eyes { snap, captured, depth, near_far: (game.frustum[4], game.frustum[5]), mirror: mirror_done });
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
        // (A menu over the world: the runtime shows the menus as a panel to point at. The race menu
        // keeps the world round it, before the mirror, its menu the part of the picture it's in.)
        let menu = crate::menus::menu_mode();
        let race_menu = crate::menus::menu_open(crate::menus::MENU_RACE_SEX).then(|| crate::hud_info::menu_uv(crate::menus::MENU_RACE_SEX).unwrap_or([0.0, 0.0, 0.5, 1.0]));
        {
            static SAID: Mutex<Option<[f32; 4]>> = Mutex::new(None);
            let mut said = SAID.lock().unwrap();
            if race_menu.is_some() && *said != race_menu {
                tracing::info!("render: the race menu is open, its menu at {:.3?} of the picture: the world stays round it, before the mirror", race_menu.unwrap());
            }
            *said = race_menu;
        }
        // The race menu puts the game's camera in front of the player's face, looking back at them:
        // the player stays where they stood (the eye and its look before it opened), so the world
        // doesn't jump forward and back as it opens and closes.
        let (eye, look_yaw) = {
            static STOOD: Mutex<Option<(glam::Vec3, f32)>> = Mutex::new(None);
            let mut stood = STOOD.lock().unwrap();
            match (race_menu.is_some(), *stood) {
                (true, Some(s)) => s,
                _ => {
                    let s = (game.eye(), game.look_yaw());
                    if race_menu.is_none() {
                        *stood = Some(s);
                    }
                    s
                }
            }
        };
        // (Seated in furniture, the anchor stays at the standing eye's height: the player sat down for
        // real, crate::rest.)
        let eye = crate::rest::anchor_eye(eye);
        let snap = link::game_frame(&GameFrame { eye: Some(eye), look_yaw, menu, race_menu, now, dt })?;
        snap.view?;
        log_camera_once(&game);
        crate::player::trace_frame(now, &snap, &game);
        use crate::timing::{Part, time};
        time(Part::Combat, crate::combat::publish);
        time(Part::Combat, || hooks::guard_void("people's capsules", || crate::movement::fit_people(true)));
        time(Part::Combat, || hooks::guard_void("pockets", crate::pockets::publish));
        time(Part::Collision, || hooks::guard_void("collision", || crate::collision::update(now)));
        if let Some(view) = snap.view.as_ref() {
            let head = glam::Vec3::from(snap.frame.hmd.pos);
            time(Part::Lighting, || hooks::guard_void("lighting", || crate::lighting::publish(view, head)));
            time(Part::Items, || hooks::guard_void("items", || crate::items::place(view, now)));
            time(Part::Items, || hooks::guard_void("torch", || crate::torch::place(view)));
            time(Part::Items, || hooks::guard_void("magic", || crate::magic::place(view)));
            time(Part::Items, || hooks::guard_void("world", || crate::world::place(now)));
            time(Part::Items, || hooks::guard_void("props", || crate::props::place(view, now)));
            hooks::guard_void("riding", || crate::riding::publish(view, now));
            hooks::guard_void("map", || crate::map::place(now));
            time(Part::Items, || hooks::guard_void("alchemy", || crate::alchemy::place(view, now)));
            time(Part::Items, || hooks::guard_void("lock", || crate::lock::place(view, &snap.frame, now)));
            time(Part::Items, || hooks::guard_void("repair", || crate::repair::place(view, &snap.frame, now)));
            time(Part::Items, || hooks::guard_void("soul", || crate::soul::place(view, &snap.frame, now)));
            time(Part::Items, || hooks::guard_void("enchant", || crate::enchant::place(view, now)));
            time(Part::Items, || hooks::guard_void("sigil", || crate::sigil::place(view, now)));
            time(Part::Items, || hooks::guard_void("ayleid", || crate::ayleid::place(view, &snap.frame, now)));
            time(Part::Items, || hooks::guard_void("shrines", || crate::shrines::place(view, &snap.frame, now)));
            time(Part::Items, || hooks::guard_void("rest", || crate::rest::place(view, &snap.frame, now)));
        }
        time(Part::Combat, || crate::gore::frame(snap.view.as_ref(), now));
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
        d3d9::ui_pass(&dev, false);
        hooks::guard_void("menu 3D", || crate::menus::hide_menu_3d(false));
    }

    fn stereo(dev: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9, e: &Eyes, aspect: f32) {
        let Some(view) = e.snap.view else { return };
        link::with_adapter(|a| {
            let gpu = a.gpu().clone();
            let hud = d3d9::capture(dev, &gpu, HUD);
            let mut have_hud = false;
            let mut have_depth = false;
            let mut have_mirror = false;
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
                // The mirror's world and its depth.
                if let (Some((near, far)), Some(aux), Some(aux_depth)) = (e.mirror, slot.aux.as_ref(), slot.aux_depth.as_ref()) {
                    have_mirror = d3d9::deliver(&gpu, d3d9::MIRROR, &aux.tex) && d3d9::deliver_depth(&gpu, 2, aux_depth, near, far);
                }
            }
            let mut info: SlotInfo = bytemuck::Zeroable::zeroed();
            info.frame_index = e.snap.frame.frame_index;
            info.eye_pose = e.snap.frame.eyes;
            info.fov = e.snap.frame.fov;
            info.eye_map = [view.slot_map(); 2];
            info.flags = SLOT_STEREO | if have_hud { SLOT_HUD } else { 0 } | if have_depth { SLOT_DEPTH } else { 0 };
            if have_mirror {
                info.flags |= SLOT_AUX | SLOT_MIRROR;
                info.aux_frame = e.snap.frame.mirror.frame;
                info.aux_pose = e.snap.frame.mirror.camera;
            }
            if have_depth {
                info.depth_near = e.near_far.0 / crate::camera::UNITS_PER_METER;
                info.depth_far = e.near_far.1 / crate::camera::UNITS_PER_METER;
                info.depth_flags = DEPTH_LINEAR;
            }
            info.hud_aspect = aspect;
            // The game's card of the things on the hands: which it's of now (`crate::hud_info`).
            hooks::guard_void("hud info", || {
                crate::hud_info::moved();
            });
            // What the HUD and the menus are, piece by piece (again whenever it changes).
            if have_hud && let Some(layout) = hooks::guard("hud layout", None, || crate::hud::changed(crate::hud::layout())) {
                a.write_hud_layout(&layout);
            }
            // (The layout this picture's HUD was drawn with: the runtime cuts it by that one, not the
            // one before or after.)
            if have_hud {
                info.hud_layout = a.hud_layout_seq();
                // (Testing what the runtime gets: how solid a menu's pieces are, once it's open.)
                if let Some(l) = crate::hud::measure_due() {
                    for (i, e) in l.entries().iter().enumerate().filter(|(_, e)| e.group != 0) {
                        if let Some((mean, least)) = d3d9::coverage(&gpu, e.uv) {
                            tracing::info!("hud: piece {i} (group {}) is covered {mean:.2} at its middle (least {least:.2})", e.group);
                        }
                    }
                    crate::menus::log_drawn_under();
                }
            }
            a.submit(info);
        });
    }

    fn menu(dev: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9, aspect: f32) {
        // Keeps the link alive and says the game is in its menus.
        let _ = link::game_frame(&GameFrame { eye: None, look_yaw: 0.0, menu: true, race_menu: None, now: link::now(), dt: 0.0 });
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
            if let Some(layout) = hooks::guard("hud layout", None, || crate::hud::changed(crate::hud::outside())) {
                a.write_hud_layout(&layout);
            }
            info.hud_layout = a.hud_layout_seq();
            a.submit(info);
        });
    }
}
