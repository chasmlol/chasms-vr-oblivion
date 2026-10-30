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
    /// (`mov esi, [B3F940]` at 0x40D059, then its release). Those two places read these instead
    /// (patched at install), so the eyes can be rendered under one hold of it ([`hold_loader`]);
    /// by default each render takes and gives it, as the game's one render does.
    ///
    /// The thread can change the scene only while no render holds the lock: on a desktop most of
    /// the frame. Two eyes and their copies held it for most of each frame (6..12 ms of 9), so it
    /// fell behind loading the cells ahead, and the game stood waiting for it at the next render's
    /// start (0x404D60), most of all crossing into new cells. Given back after each eye, it can work
    /// while that eye's picture is copied; a thing it adds then shows in the second eye a frame
    /// before the first.
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

    /// Testing, `RUSTVR_OBLIVION_HOLD_LOADER=1`: the background thread's lock is held across all the
    /// renders of a frame, as before, to compare the stutter.
    fn hold_loader() -> bool {
        static HOLD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *HOLD.get_or_init(|| {
            let hold = std::env::var("RUSTVR_OBLIVION_HOLD_LOADER").is_ok_and(|v| v == "1");
            if hold {
                tracing::info!("render: testing: the background thread's lock is held across the eyes (RUSTVR_OBLIVION_HOLD_LOADER)");
            }
            hold
        })
    }

    /// The game's wait for its background thread's lock (0x404D60, cdecl, a first timeout in
    /// milliseconds): the world render's at its start, and the game's own elsewhere. Timed on the
    /// game's thread, as how long its frame stood waiting for the thread (the stutter crossing into
    /// new cells); one over 20 ms is logged.
    const LOADER_WAIT: usize = 0x0040_4D60;
    static REAL_LOADER_WAIT: AtomicUsize = AtomicUsize::new(0);
    /// The thread the world render runs on (the game's), once it has.
    static GAME_THREAD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    unsafe extern "C" fn loader_wait(ms: u32) -> i32 {
        let real: unsafe extern "C" fn(u32) -> i32 = unsafe { std::mem::transmute(REAL_LOADER_WAIT.load(Ordering::Acquire)) };
        let thread = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
        if GAME_THREAD.load(Ordering::Acquire) != thread {
            return unsafe { real(ms) };
        }
        let t0 = std::time::Instant::now();
        let r = unsafe { real(ms) };
        let waited = t0.elapsed();
        hooks::guard_void("the background thread's lock's wait", || {
            crate::timing::add(crate::timing::Part::Loader, waited);
            if waited > std::time::Duration::from_millis(20) {
                tracing::info!("render: the game waited {:.0} ms for its background thread (loading)", waited.as_secs_f64() * 1000.0);
            }
        });
        r
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

    /// The world's shadow scene node (`*0xB42F54`) and its actors' shadow lights
    /// (`ShadowSceneLight`, an `NiTPointerList` at +0xF8: each entry's next at +0, its light at +8).
    const SHADOW_SCENE: usize = 0x00B4_2F54;
    const SHADOW_LIGHTS: usize = 0xF8;
    /// A shadow light's clip planes are made for this frame (u8). Each draw of an actor's shadow on
    /// what it falls on clips it to the shadow light's frustum with six user clip planes, which
    /// Direct3D takes in the camera's clip space: the first draw of a frame puts the light's
    /// planes (+0x150) into the camera's clip space, keeps them (+0x1B4) and sets this; the rest
    /// take the kept ones (0x7C9CCA, and 0x7FC556 for the SM3 shaders). Only the light's shadow map
    /// render clears it (0x7D4F67), with the first eye, so the second eye and the mirror clipped
    /// the shadows with the first eye's planes, which cut them away: the actors' shadows were in
    /// the left eye only.
    const CLIP_PLANES_MADE: usize = 0x214;

    /// The actors' shadow lights, as the list has them (bounded: the background thread's lock is
    /// held, but a broken list mustn't hang the render).
    fn shadow_lights() -> Vec<usize> {
        let ssn = mem::deref(SHADOW_SCENE);
        let mut lights = Vec::new();
        let mut entry = if ssn == 0 { 0 } else { mem::deref(ssn + SHADOW_LIGHTS) };
        while entry != 0 && lights.len() < 1024 {
            let light = mem::deref(entry + 8);
            if light != 0 {
                lights.push(light);
            }
            entry = mem::deref(entry);
        }
        lights
    }

    /// Testing: what the actors' shadow lights are after a render: how many, how many drew their
    /// shadow in it (their clip planes made), how many are seen (+0x118, 0xFF: not) and faded in
    /// (+0xD8), and the first one's clip planes as that render made them (+0x1B4).
    fn shadow_lights_state() -> String {
        let lights = shadow_lights();
        let made = lights.iter().filter(|&&l| mem::at::<u8>(l + CLIP_PLANES_MADE) == Some(1)).count();
        let seen = lights.iter().filter(|&&l| mem::at::<u16>(l + 0x118).is_some_and(|m| m != 0xFF)).count();
        let fades: Vec<f32> = lights.iter().filter_map(|&l| mem::at::<f32>(l + 0xD8)).collect();
        let planes = lights.first().and_then(|&l| mem::at::<[f32; 24]>(l + 0x1B4));
        format!("{} shadow lights, {made} drew a shadow, {seen} seen, fades {fades:.2?}; the first's clip planes {planes:.3?}", lights.len())
    }

    /// The actors' shadow lights' clip planes are made again, for the next render's camera.
    fn shadow_clip_planes_again() {
        for light in shadow_lights() {
            if mem::at::<u8>(light + CLIP_PLANES_MADE).is_some() {
                unsafe { ((light + CLIP_PLANES_MADE) as *mut u8).write(0) };
            }
        }
    }

    /// The world render centres the sky and the weather on the scene graph's camera node
    /// ([`engine::camera_node`], where the game's camera is): at its start it copies the node's
    /// position (`NiAVObject` +0x54) into the "Sky" node (`*0xB333D8`: the dome, the clouds, the
    /// sun, the moons, the stars) and the "Weather" node (`*0xB333DC`: rain, snow, the sun's glare)
    /// and updates them. An eye is elsewhere (where the head is: the room's offset, the real head's
    /// height, the steps smoothed), so the dome and its clouds were centred off the eyes, near
    /// enough to show parallax, and swam and bobbed as the player walked. The two copies take the
    /// node from these words instead (patched at install: `mov eax, [0xB333CC]` and the walk to its
    /// first child at 0x40C95F and 0x40C9AA become `mov eax, [word]` and a jump past the walk).
    /// For a single render they hold the camera node, as the game has it. In the eyes, the sky is
    /// centred on each render's camera (it's at infinity for both eyes, turning only as the head
    /// turns) and the weather on the head, one place for both eyes, so rain and snow keep their
    /// depth.
    static SKY_FROM: AtomicUsize = AtomicUsize::new(0);
    static WEATHER_FROM: AtomicUsize = AtomicUsize::new(0);
    const SKY_COPY_AT: usize = 0x0040_C95F;
    const WEATHER_COPY_AT: usize = 0x0040_C9AA;

    /// A stand-in for the camera node as those copies read it: its position (+0x54) only.
    #[repr(C)]
    struct Centre([std::sync::atomic::AtomicU32; 24]);
    static SKY_CENTRE: Centre = Centre([const { std::sync::atomic::AtomicU32::new(0) }; 24]);
    static WEATHER_CENTRE: Centre = Centre([const { std::sync::atomic::AtomicU32::new(0) }; 24]);

    impl Centre {
        /// Puts the stand-in at `p` (game units) and it in `from`'s word.
        fn centre(&self, from: &AtomicUsize, p: glam::Vec3) {
            let at = engine::av::LOCAL_TRANSLATE / 4;
            for (i, v) in p.to_array().into_iter().enumerate() {
                self.0[at + i].store(v.to_bits(), Ordering::Release);
            }
            from.store(self as *const Centre as usize, Ordering::Release);
        }
    }

    fn patch_centres() {
        let [b0, b1, b2, b3] = (crate::engine::SCENE_GRAPH as u32).to_le_bytes();
        let walk = [0xA1, b0, b1, b2, b3, 0x66, 0x83];
        let patch = |at: usize, word: &AtomicUsize| {
            let [w0, w1, w2, w3] = (word as *const AtomicUsize as u32).to_le_bytes();
            // (The jump goes past the 27 bytes of the walk, from the end of these 7.)
            crate::mem::patch_code(at, &walk, &[0xA1, w0, w1, w2, w3, 0xEB, 27 - 7])
        };
        let (sky, weather) = (patch(SKY_COPY_AT, &SKY_FROM), patch(WEATHER_COPY_AT, &WEATHER_FROM));
        if sky && weather {
            tracing::info!("render: the sky is centred on each eye and the weather on the head");
        } else {
            tracing::error!("render: the sky and the weather are centred on the game's camera (patched: sky {sky}, weather {weather})");
        }
    }

    /// The sky and the weather centred on the camera node, as the game has them.
    fn centre_on_camera_node() {
        let node = engine::camera_node();
        SKY_FROM.store(node, Ordering::Release);
        WEATHER_FROM.store(node, Ordering::Release);
    }

    /// The "Sky" node (`*0xB333D8`), whose world position is where the render put the sky.
    const SKY_NODE: usize = 0x00B3_33D8;

    /// The fixes for what the game makes once a frame from one camera (the sky's centre, a
    /// skinned model's bones, the actors' shadows' clip planes) are on. Testing, `RUSTVR_OBLIVION_MONO_VISUALS=1`: they're off, to
    /// see what they fix.
    fn per_eye_visuals() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| {
            let on = !std::env::var("RUSTVR_OBLIVION_MONO_VISUALS").is_ok_and(|v| v == "1");
            if !on {
                tracing::info!("render: testing: the sky is centred on the game's camera, the eyes share the renderer's frame and the shadows' clip planes (RUSTVR_OBLIVION_MONO_VISUALS)");
            }
            on
        })
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

    pub fn install() {
        set_lock(true, true);
        patch_lock();
        patch_centres();
        hooks::install_into(ACTOR_SHADOWS as *const (), actor_shadows as *const (), "actor shadows", &REAL_SHADOWS);
        hooks::install_into(LOADER_WAIT as *const (), loader_wait as *const (), "the background thread's lock's wait", &REAL_LOADER_WAIT);
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
        });
        unsafe { real(this) }
    }

    unsafe extern "thiscall" fn render_hook(this: *mut c_void, arg: i32) -> i32 {
        let real: RenderFn = unsafe { std::mem::transmute(REAL_RENDER.load(Ordering::Acquire)) };
        GAME_THREAD.store(unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }, Ordering::Release);
        // A picture of the world into a texture of the game's own making (a save's picture, taken
        // as an autosave goes through a door) is the game's, drawn as it draws it: two eyes drawn
        // into it, read back at its size, crashed it. (Its frame passes 0; the world behind its
        // menus goes into its world texture.)
        centre_on_camera_node();
        if arg != 0 && arg as usize != mem::deref(WORLD_TEXTURE) {
            return unsafe { real(this, arg) };
        }
        crate::timing::render_start();
        let result = unsafe { render_stereo(this, arg, real) };
        crate::timing::render_end();
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
        let frame = LOGGED.fetch_add(1, Ordering::Relaxed);
        let log = frame % 600 == 30;
        // (Testing, for the actors' shadows: every 90 frames, the first 200 times.)
        let log_shadows = frame % 90 == 45 && frame < 90 * 200;
        let mut mirror_done = None;
        let per_eye = per_eye_visuals();
        let eyes = [0, 1].map(|eye| camera::eye_camera(&view, &snap.frame.eyes[eye], &snap.frame.fov[eye], &game));
        if per_eye {
            WEATHER_CENTRE.centre(&WEATHER_FROM, (glam::Vec3::from(eyes[0].pos) + glam::Vec3::from(eyes[1].pos)) * 0.5);
        }
        for eye in 0..renders {
            // Each render after the first is a frame of the renderer's own. The bones of a skinned
            // model (the actors' bodies, clothes and armour) are made relative to the camera that
            // draws it first in a frame, so with one frame for all the renders the second eye and
            // the mirror drew the actors' bodies where the first eye's camera saw them: in the
            // second eye, the eyes' width aside from their heads, weapons and shadows, which are
            // drawn where they are.
            if eye > 0 && per_eye {
                engine::next_render_frame();
                shadow_clip_planes_again();
            }
            if eye == 2 {
                let s = mirror_cam.expect("a third render is the mirror's");
                camera::write(cam, &s);
                if per_eye {
                    SKY_CENTRE.centre(&SKY_FROM, glam::Vec3::from(s.pos));
                }
                set_lock(!hold_loader(), true);
                SECOND_EYE.store(true, Ordering::Release);
                result = time(Part::Eye, || unsafe { real(this, arg) });
                if log_shadows {
                    tracing::info!("render: the mirror: {}", shadow_lights_state());
                }
                let colour = hooks::guard("capture mirror", false, || link::with_adapter(|a| d3d9::capture(&dev, a.gpu(), d3d9::MIRROR)).unwrap_or(false));
                let depth = hooks::guard("capture mirror depth", false, || link::with_adapter(|a| d3d9::capture_depth(&dev, a.gpu(), 2)).unwrap_or(false));
                mirror_done = (colour && depth).then_some((s.frustum[4], s.frustum[5]));
                continue;
            }
            let s = eyes[eye];
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
            if per_eye {
                SKY_CENTRE.centre(&SKY_FROM, glam::Vec3::from(s.pos));
            }
            if hold_loader() {
                set_lock(eye == 0, eye + 1 == renders);
            } else {
                set_lock(true, true);
            }
            SECOND_EYE.store(eye == 1, Ordering::Release);
            result = time(Part::Eye, || unsafe { real(this, arg) });
            if log_shadows {
                tracing::info!("render: eye {eye}: {}", shadow_lights_state());
            }
            if log {
                let sky = mem::at::<[f32; 3]>(mem::deref(SKY_NODE) + engine::av::WORLD_TRANSLATE).map(glam::Vec3::from);
                tracing::info!(
                    "render: eye {eye}: the sky's centre is {:.2?} units from the eye (the game's camera {:.2} units)",
                    sky.map(|c| c.distance(glam::Vec3::from(s.pos))),
                    glam::Vec3::from(s.pos).distance(glam::Vec3::from(game.pos))
                );
            }
            captured[eye] = time(Part::Capture, || hooks::guard("capture", false, || link::with_adapter(|a| d3d9::capture(&dev, a.gpu(), eye)).unwrap_or(false)));
            depth[eye] = time(Part::Depth, || hooks::guard("capture depth", false, || link::with_adapter(|a| d3d9::capture_depth(&dev, a.gpu(), eye)).unwrap_or(false)));
        }
        set_lock(true, true);
        SECOND_EYE.store(false, Ordering::Release);
        camera::write(cam, &game);
        centre_on_camera_node();
        if let Some(f) = arms_flags {
            unsafe { ((arms + engine::av::FLAGS) as *mut u16).write(f) };
        }
        if let Some(f) = outside_flags {
            unsafe { ((outside + engine::av::FLAGS) as *mut u16).write(f) };
        }
        if captured == [true, true] {
            hooks::guard_void("eye pictures", || eye_pictures(link::now()));
        }
        hooks::guard_void("Render end", || {
            // The menus are drawn next: alone on black, they're the HUD layer.
            let _ = unsafe { dev.Clear(0, std::ptr::null(), D3DCLEAR_TARGET as u32, 0, 1.0, 0) };
            *EYES.lock().unwrap() = Some(Eyes { snap, captured, depth, near_far: (game.frustum[4], game.frustum[5]), mirror: mirror_done });
        });
        result
    }

    /// Testing, `RUSTVR_OBLIVION_EYE_PNG=<folder>`: both eyes' pictures (the world, before the HUD,
    /// at most 2048 pixels wide) are saved there every 10 seconds from 5 seconds in, up to 60 pairs
    /// (`<n>-left.png`, `<n>-right.png`), to hold the eyes up to each other.
    fn eye_pictures(now: f64) {
        static DIR: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        let Some(dir) = DIR.get_or_init(|| std::env::var_os("RUSTVR_OBLIVION_EYE_PNG").filter(|v| !v.is_empty()).map(std::path::PathBuf::from)) else { return };
        static NEXT: Mutex<(f64, u32)> = Mutex::new((0.0, 0));
        let n = {
            let mut next = NEXT.lock().unwrap();
            if next.0 == 0.0 {
                next.0 = now + 5.0;
            }
            if now < next.0 || next.1 >= 60 {
                return;
            }
            next.0 = now + 10.0;
            next.1 += 1;
            next.1
        };
        let mut pictures = Vec::new();
        for (eye, name) in [(0, "left"), (1, "right")] {
            match link::with_adapter(|a| d3d9::read_bgra(a.gpu(), eye, 2048)) {
                Some(Ok(p)) => pictures.push((dir.join(format!("{n:02}-{name}.png")), p)),
                Some(Err(e)) => tracing::warn!("render: the {name} eye's picture: {e:#}"),
                None => {}
            }
        }
        // (Written off the render thread: a picture takes longer than a frame.)
        std::thread::spawn(move || {
            for (path, (w, h, mut px)) in pictures {
                // (The copies are B8G8R8A8; the picture is opaque.)
                for p in px.as_chunks_mut::<4>().0 {
                    p.swap(0, 2);
                    p[3] = 255;
                }
                match rustvr_gpu::save_png_fast(&path, w, h, &px) {
                    Ok(()) => tracing::info!("render: the eye's picture is {}", path.display()),
                    Err(e) => tracing::warn!("render: saving {}: {e:#}", path.display()),
                }
            }
        });
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
        let snap = link::game_frame(&GameFrame { eye: Some(eye), look_yaw, menu, race_menu, now, dt })?;
        snap.view?;
        log_camera_once(&game);
        crate::player::trace_frame(now, &snap, &game);
        use crate::timing::{Part, time};
        time(Part::Combat, crate::combat::publish);
        time(Part::Collision, || hooks::guard_void("collision", || crate::collision::update(now)));
        if let Some(view) = snap.view.as_ref() {
            let head = glam::Vec3::from(snap.frame.hmd.pos);
            time(Part::Lighting, || hooks::guard_void("lighting", || crate::lighting::publish(view, head)));
            time(Part::Items, || hooks::guard_void("items", || crate::items::place(view, now)));
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
            static LAYOUT_SENT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            // (Again when the game's card of a thing has moved on its HUD.)
            let card_moved = hooks::guard("hud info", None, crate::hud_info::moved).is_some();
            if have_hud && (!LAYOUT_SENT.swap(true, Ordering::AcqRel) || card_moved) {
                a.write_hud_layout(&crate::hud::layout(crate::hud_info::last()));
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
            a.submit(info);
        });
    }
}
