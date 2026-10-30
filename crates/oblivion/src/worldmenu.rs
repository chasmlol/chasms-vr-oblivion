//! The main menu in a real place in Cyrodiil (`docs/research/worldmenu.md`). At the main menu with no
//! game (the boot's, or after quitting to it), the game's own console lines take the default player
//! the boot made to one of the [`SPOTS`]: `set gamehour`, then `coc` to its cell (or `cow` to its
//! place in Tamriel), then `setpos`, `setangle` and `fw` (ForceWeather). The main menu stays open
//! over the place, as the game's own main menu: its question boxes, music, sounds, and what each
//! button does are the game's. The adapter only lets the game draw its world under it and cuts the
//! menu into floating words over the view ([`crate::hud`]).
//!
//! * **Drawn.** The frame's draw (0x40D4D0) renders the world only when the top menu isn't the main
//!   menu (the call at 0x40D606, then `cmp eax, 414h`): that call answers "no main menu" while the
//!   place is up, so the world is rendered under it in stereo as under any other menu.
//! * **Alive.** A menu freezes the world: the frame calls the sky, water and wind update (0x4424D0 on
//!   `TES`) with no time. While the place is up it's called with the frame's own time, as the game
//!   does in play: the clouds drift, the water moves, the grass and trees sway, and the hour and
//!   everyone in the place stay still (the clock, the actors and the scripts are the frame's, and it
//!   skips them in a menu).
//! * **New** starts from the game's clean state, as from the flat main menu. Its answer's callback
//!   (0x5B5DF0) is held back over the place: the game's own exit to the main menu (`OSGlobals` +1)
//!   resets the world, and on the fresh main menu the same callback runs with the same answer.
//!   Continue and Load load a save, which replaces the world as it does from a game.
//!
//! `menu_world = false` in `rustvr-oblivion.toml` keeps the flat main menu; `menu_spot` names the
//! spot (or several, taken in turn as the game starts), and `RUSTVR_OBLIVION_MENU_SPOT` overrides it
//! for a playtest.

use rustvr_protocol::{SIDE_LEFT, SIDE_RIGHT};

/// A place for the main menu: where the player stands and faces, at what hour and in what weather,
/// and what the view is of (the menu floats beside it, on `side`).
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub name: &'static str,
    /// The cell `coc` goes to (a named exterior cell), or "" for `cow` to the spot's cell of Tamriel.
    pub cell: &'static str,
    /// On the ground (the land's height there), game units.
    pub pos: [f32; 3],
    /// Degrees from north, turning east (the game's `setangle z`).
    pub heading: f32,
    pub hour: f32,
    /// The weather's form id (Clear is 0x38EEE).
    pub weather: u32,
    /// What the view is of, game units.
    pub subject: [f32; 3],
    pub side: u32,
}

const CLEAR: u32 = 0x0003_8EEE;
/// The White-Gold Tower (its `ICPalaceTower01` in Tamriel, a little up its height).
const WHITE_GOLD: [f32; 3] = [28672.0, 63424.0, 9000.0];

/// The places players call Oblivion's most beautiful views, placed from `Oblivion.esm`'s own map
/// markers and land heights (`docs/research/worldmenu.md`, and a `.bat` for each at the worktree's
/// root).
pub const SPOTS: &[Spot] = &[
    // The Imperial City across Lake Rumare at sunset, from the slope above Weye: the great bridge
    // leading the eye east to the White-Gold Tower, the sun setting behind.
    Spot { name: "imperial-city", cell: "WeyeExterior", pos: [-2000.0, 66000.0, 1167.0], heading: 95.0, hour: 18.5, weather: CLEAR, subject: WHITE_GOLD, side: SIDE_LEFT },
    // Cloud Ruler Temple in the Jerall Mountains, morning: south over Bruma and the Great Forest.
    Spot { name: "cloud-ruler", cell: "CloudRulerTempleExterior02", pos: [13391.0, 160255.0, 31594.0], heading: 171.0, hour: 8.5, weather: CLEAR, subject: WHITE_GOLD, side: SIDE_LEFT },
    // Dive Rock at dawn: the summit, south-west over the Nibenay to the Imperial City.
    Spot { name: "dive-rock", cell: "DiveRock", pos: [138134.0, 142738.0, 34178.0], heading: 234.0, hour: 6.5, weather: CLEAR, subject: WHITE_GOLD, side: SIDE_LEFT },
    // Anvil's lighthouse on the Gold Coast at sunset, over the Abecean Sea.
    Spot { name: "anvil", cell: "", pos: [-196600.0, -39000.0, 740.0], heading: 300.0, hour: 19.0, weather: CLEAR, subject: [-213920.0, -29000.0, 0.0], side: SIDE_LEFT },
    // Lake Rumare by moonlight from Vilverin's ruin, the city across the water.
    Spot { name: "rumare-night", cell: "", pos: [50804.0, 87399.0, 405.0], heading: 223.0, hour: 22.0, weather: CLEAR, subject: WHITE_GOLD, side: SIDE_LEFT },
    // The first view: Lake Rumare from the prison sewer's exit, facing Vilverin.
    Spot { name: "first-view", cell: "ICPrisonSewerExit01", pos: [47110.0, 83000.0, 297.0], heading: 40.0, hour: 9.0, weather: CLEAR, subject: [50804.0, 87399.0, 900.0], side: SIDE_RIGHT },
];

/// The spot until the player picks: the view players mention most.
pub const DEFAULT_SPOT: &str = "imperial-city";

/// The spot named by `menu_spot` (a name, or several separated by commas: one of them, `turn` picks
/// which), or the default.
pub fn spot_named(menu_spot: &str, turn: usize) -> &'static Spot {
    let named: Vec<&'static Spot> = menu_spot.split(',').map(str::trim).filter_map(|n| SPOTS.iter().find(|s| s.name.eq_ignore_ascii_case(n))).collect();
    if named.is_empty() {
        return SPOTS.iter().find(|s| s.name == DEFAULT_SPOT).unwrap_or(&SPOTS[0]);
    }
    named[turn % named.len()]
}

/// The game's console lines that take the player there: the hour first (the place loads in its
/// light), then the cell.
pub fn go_lines(s: &Spot) -> Vec<String> {
    let place = if s.cell.is_empty() {
        format!("cow Tamriel {} {}", (s.pos[0] / 4096.0).floor() as i32, (s.pos[1] / 4096.0).floor() as i32)
    } else {
        format!("coc {}", s.cell)
    };
    vec![format!("set gamehour to {}", s.hour), place]
}

/// The lines that set the scene once the player's in the cell: on the spot, facing the view, the
/// weather forced (so it's the spot's, not what the region would pick).
pub fn place_lines(s: &Spot) -> Vec<String> {
    vec![
        format!("player.setpos x {}", s.pos[0]),
        format!("player.setpos y {}", s.pos[1]),
        format!("player.setpos z {}", s.pos[2]),
        format!("player.setangle z {}", s.heading),
        format!("fw {:08X}", s.weather),
    ]
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use super::*;
    use crate::{camera, engine, hooks, link, mem, menus};
    use glam::Vec3;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// `OSGlobals*`, and its `exitToMainMenu` (the idle loop resets the game and opens the main menu).
    const OS_GLOBALS: usize = 0x00B3_3398;
    const EXIT_TO_MAIN_MENU: usize = 1;
    /// `InterfaceManager*` and the answer to its last question box (+0xB0; 1: the first button, the
    /// main menu's yes).
    const INTERFACE_MANAGER: usize = 0x00B3_A6E0;
    const ANSWER: usize = 0xB0;
    const YES: u8 = 1;
    /// The main menu's New, answered (0x5B5DF0, cdecl, no arguments).
    const NEW_GAME: usize = 0x005B_5DF0;
    /// The top menu (cdecl), as the frame's draw asks it at 0x40D606 before `cmp eax, 414h`.
    const TOP_MENU: usize = 0x0057_9060;
    const DRAW_ASKS_AT: usize = 0x0040_D606;
    /// The sky, water and wind's update (thiscall on `TES`, the time passed), and the multiplier the
    /// frame gives the frame's time for it in play (the frame's own time may stand still in a menu:
    /// the real time is taken).
    const SKY_UPDATE: usize = 0x0044_24D0;
    const TES: usize = 0x00B3_33A0;
    const SKY_TIME_SCALE: usize = 0x00B0_6530;
    /// A reference's cell.
    const PARENT_CELL: usize = 0x40;
    /// How long the place may take to come before the main menu is left flat.
    const GIVE_UP: f64 = 45.0;
    /// A standing player's eye over their feet (game units): the game's first-person camera for a
    /// race of height 1, about 1.7 m.
    const EYE_HEIGHT: f32 = 120.0;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum State {
        /// Not at a main menu without a game (or it's left flat).
        Idle,
        /// The lines that go there have run; waiting for the player to be in the cell (`from`: the
        /// one they were in, a game's left over after quitting to the main menu, or none).
        Going { since: f64, from: usize },
        /// The place is there with the main menu over it.
        Up,
        /// It didn't come; this main menu stays flat.
        Failed,
    }

    struct World {
        state: State,
        spot: Option<&'static Spot>,
        /// The view to face (a look yaw), for the next game frame's view.
        recenter: Option<f32>,
    }
    static WORLD: Mutex<World> = Mutex::new(World { state: State::Idle, spot: None, recenter: None });

    /// New was answered over the place: the game's exit to the main menu is under way, then New
    /// (since when: given up on after [`GIVE_UP`]).
    static NEW_PENDING: AtomicBool = AtomicBool::new(false);
    static NEW_SINCE: Mutex<f64> = Mutex::new(0.0);
    static REAL_NEW: AtomicUsize = AtomicUsize::new(0);

    pub fn install() {
        if !crate::settings::get().menu_world {
            tracing::info!("worldmenu: off (menu_world = false): the main menu is the flat one");
            return;
        }
        let ours = top_menu_for_draw as *const () as usize;
        let expect = (TOP_MENU as i32).wrapping_sub(DRAW_ASKS_AT as i32 + 5).to_le_bytes();
        let rel = (ours as i32).wrapping_sub(DRAW_ASKS_AT as i32 + 5).to_le_bytes();
        let drawn = mem::patch_code(DRAW_ASKS_AT, &[0xE8, expect[0], expect[1], expect[2], expect[3]], &[0xE8, rel[0], rel[1], rel[2], rel[3]]);
        let new = hooks::install_into(NEW_GAME as *const (), new_game as *const (), "MainMenu New (answered)", &REAL_NEW);
        tracing::info!("worldmenu: the main menu in the world (the world drawn under it: {drawn}; New from a fresh game: {new})");
    }

    /// The top menu as the frame's draw sees it: none while the main menu is over the place, so the
    /// world is rendered under it.
    unsafe extern "C" fn top_menu_for_draw() -> u32 {
        let real: unsafe extern "C" fn() -> u32 = unsafe { std::mem::transmute(TOP_MENU) };
        let top = unsafe { real() };
        if top == menus::MENU_MAIN && up() { 0 } else { top }
    }

    /// The main menu's New, answered: over the place, the game's own exit to the main menu first.
    unsafe extern "C" fn new_game() -> i32 {
        let real: unsafe extern "C" fn() -> i32 = unsafe { std::mem::transmute(REAL_NEW.load(Ordering::Acquire)) };
        let answer = mem::deref(INTERFACE_MANAGER) + ANSWER;
        if up() && mem::at::<u8>(answer) == Some(YES) {
            let os = mem::deref(OS_GLOBALS);
            if os != 0 {
                // (The answer is taken, as the callback would; it's given again on the fresh menu.)
                unsafe {
                    (answer as *mut u8).write(0xFF);
                    ((os + EXIT_TO_MAIN_MENU) as *mut u8).write(1);
                }
                *NEW_SINCE.lock().unwrap() = link::now();
                NEW_PENDING.store(true, Ordering::Release);
                tracing::info!("worldmenu: New: the game's exit to the main menu first (a fresh game), then New");
                return 0;
            }
        }
        unsafe { real() }
    }

    fn player_cell() -> usize {
        let p = engine::player();
        if p != 0 { mem::deref(p + PARENT_CELL) } else { 0 }
    }

    /// The place is there with the main menu over it.
    pub fn up() -> bool {
        WORLD.lock().unwrap().state == State::Up && menus::menu_open(menus::MENU_MAIN) && player_cell() != 0
    }

    /// The main menu is waiting for its place (going there, or about to: between quitting to the
    /// main menu and the place, or on the way to New): what was in the headset is held meanwhile.
    pub fn waiting() -> bool {
        if !crate::settings::get().menu_world || !menus::menu_open(menus::MENU_MAIN) {
            return NEW_PENDING.load(Ordering::Acquire);
        }
        matches!(WORLD.lock().unwrap().state, State::Idle | State::Going { .. }) || NEW_PENDING.load(Ordering::Acquire)
    }

    /// Where the view's eye is while the place is up: on the spot, at a standing eye's height (the
    /// protocol's world, metres). Under the main menu every frame is a menu's, so the game never lifts
    /// its camera from the feet its teleport put it at (its player's update, which does, is skipped
    /// in menus); the eyes are drawn from here whatever the game's camera says.
    pub fn eye() -> Option<Vec3> {
        if !up() {
            return None;
        }
        let s = WORLD.lock().unwrap().spot?;
        Some(camera::basis().point_from_game(Vec3::new(s.pos[0], s.pos[1], s.pos[2] + EYE_HEIGHT)))
    }

    /// The look to face, once, for the view (the spot's way), taken.
    pub fn take_recenter() -> Option<f32> {
        WORLD.lock().unwrap().recenter.take()
    }

    /// What the view is of and which side of it the menu goes, in the protocol's world (metres).
    pub fn subject() -> Option<(Vec3, u32)> {
        let s = WORLD.lock().unwrap().spot?;
        Some((camera::basis().point_from_game(Vec3::from(s.subject)), s.side))
    }

    /// A look yaw (the protocol's: radians, 0 = +Z, turning towards +X) for a heading in the game's
    /// degrees.
    fn look_yaw(heading: f32) -> f32 {
        let h = heading.to_radians();
        let d = camera::basis().dir_from_game(Vec3::new(h.sin(), h.cos(), 0.0));
        d.x.atan2(d.z)
    }

    fn run(lines: &[String]) -> bool {
        lines.iter().all(|l| {
            let ok = crate::console::run(l);
            tracing::info!("worldmenu: {l} -> {}", if ok { "ran" } else { "failed" });
            ok
        })
    }

    /// Once a game frame, before it: the place for a main menu without a game, the sky's time while
    /// it's up, and New once the game is fresh again.
    pub fn frame() {
        if !crate::settings::get().menu_world {
            return;
        }
        let now = link::now();
        let main = menus::menu_open(menus::MENU_MAIN);
        let cell = player_cell();
        // New, held back over the place: once the game's exit to the main menu has reset it.
        if NEW_PENDING.load(Ordering::Acquire) {
            let os = mem::deref(OS_GLOBALS);
            let reset = os != 0 && mem::at::<u8>(os + EXIT_TO_MAIN_MENU) == Some(0);
            if reset && main {
                NEW_PENDING.store(false, Ordering::Release);
                WORLD.lock().unwrap().state = State::Idle;
                let answer = mem::deref(INTERFACE_MANAGER) + ANSWER;
                tracing::info!("worldmenu: the game is fresh: New, as the main menu answered it");
                unsafe { (answer as *mut u8).write(YES) };
                let real: unsafe extern "C" fn() -> i32 = unsafe { std::mem::transmute(REAL_NEW.load(Ordering::Acquire)) };
                hooks::guard_void("New", || {
                    unsafe { real() };
                });
            } else if now - *NEW_SINCE.lock().unwrap() > GIVE_UP {
                tracing::warn!("worldmenu: the game's exit to the main menu didn't come: New given up (answer it again)");
                NEW_PENDING.store(false, Ordering::Release);
            }
            return;
        }
        let mut w = WORLD.lock().unwrap();
        match w.state {
            // (The main menu is open with a game only over its place: open from Idle, there's no
            // game, whatever cell the player was last in.)
            State::Idle if main => {
                if crate::console::pending_at_menu() {
                    return;
                }
                // (A different one of the named spots each time the game starts.)
                static TURN: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
                let turn = *TURN.get_or_init(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as usize));
                // (`RUSTVR_OBLIVION_MENU_SPOT` names it for a playtest: the test tools write the
                // settings afresh each time the game starts.)
                let named = std::env::var("RUSTVR_OBLIVION_MENU_SPOT").ok().filter(|s| !s.trim().is_empty());
                let spot = spot_named(named.as_deref().unwrap_or(&crate::settings::get().menu_spot), turn);
                tracing::info!("worldmenu: the main menu without a game: to {} ({:?} at {:?}, facing {}, at {} o'clock)", spot.name, spot.cell, spot.pos, spot.heading, spot.hour);
                w.spot = Some(spot);
                w.state = State::Going { since: now, from: cell };
                drop(w);
                if !run(&go_lines(spot)) {
                    tracing::warn!("worldmenu: the game didn't take the player there: the main menu stays flat");
                    WORLD.lock().unwrap().state = State::Failed;
                }
            }
            // (In: a cell other than the one they were in, or that one again once its load is done.)
            State::Going { since, from } if cell != 0 && (cell != from || now - since > 1.0 && !menus::menu_open(menus::MENU_LOADING)) => {
                let spot = w.spot.expect("going to a spot");
                // (The frame draws the world only for a player with 3D: their `vtable+340`, as it asks.)
                let player = engine::player();
                let has_3d = player != 0 && {
                    let f = mem::deref(mem::deref(player) + 340);
                    type F = unsafe extern "thiscall" fn(usize) -> usize;
                    f != 0 && unsafe { std::mem::transmute::<usize, F>(f)(player) } != 0
                };
                tracing::info!("worldmenu: in the place after {:.1} s (the player's 3D there: {has_3d}): setting the scene", now - since);
                w.recenter = Some(look_yaw(spot.heading));
                w.state = State::Up;
                drop(w);
                run(&place_lines(spot));
                show_main_menu();
            }
            State::Going { since, .. } if now - since > GIVE_UP || !main => {
                tracing::warn!("worldmenu: no place after {:.0} s: the main menu stays flat", now - since);
                w.state = State::Failed;
            }
            // (Gone from the main menu: a game's loaded, or the place left it. Back at a main menu
            // without a game, it's taken there again.)
            State::Up | State::Failed if !main => w.state = State::Idle,
            State::Up => {
                drop(w);
                sky(now);
            }
            _ => {}
        }
    }

    /// The sky, water and wind with the time since the last frame, as in play (see the module's
    /// notes).
    fn sky(now: f64) {
        static LAST: Mutex<f64> = Mutex::new(0.0);
        let dt = {
            let mut last = LAST.lock().unwrap();
            let dt = if *last == 0.0 { 0.0 } else { (now - *last) as f32 };
            *last = now;
            dt.clamp(0.0, 0.1)
        };
        let tes = mem::deref(TES);
        let k = mem::at::<f32>(SKY_TIME_SCALE).filter(|k| k.is_finite() && *k > 0.0).unwrap_or(1.0);
        if tes == 0 {
            return;
        }
        let update: unsafe extern "thiscall" fn(usize, f32) = unsafe { std::mem::transmute(SKY_UPDATE) };
        hooks::guard_void("the place's sky", || unsafe { update(tes, k * dt) });
        static SAID: std::sync::Once = std::sync::Once::new();
        SAID.call_once(|| tracing::info!("worldmenu: the place lives under the main menu (the sky, water and wind with the time passing, x {k})"));
    }

    /// The main menu's root tile shown, and its logo and buttons (`user0`, `user1`): a film the boot
    /// started behind it (which the place's loading stops) keeps it hidden until it ends.
    fn show_main_menu() {
        let tile = menus::tile_menu(menus::MENU_MAIN);
        if tile == 0 {
            tracing::warn!("worldmenu: the main menu closed on the way to its place");
            return;
        }
        type Update = unsafe extern "thiscall" fn(usize, u32, f32);
        let update: Update = unsafe { std::mem::transmute(crate::hud::UPDATE_FLOAT) };
        for (id, name) in [(crate::tiles::VISIBLE, "visible"), (crate::tiles::USER0, "its logo"), (crate::tiles::USER0 + 1, "its buttons")] {
            if crate::tiles::value(tile, id) != Some(crate::tiles::TRUE) {
                tracing::info!("worldmenu: the main menu's {name} shown");
                hooks::guard_void("main menu shown", || unsafe { update(tile, id as u32, crate::tiles::TRUE) });
            }
        }
    }
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn up() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spots_by_name_in_turn() {
        assert_eq!(spot_named("", 3).name, DEFAULT_SPOT);
        assert_eq!(spot_named("nowhere", 0).name, DEFAULT_SPOT);
        assert_eq!(spot_named("Dive-Rock", 7).name, "dive-rock");
        let names: Vec<_> = (0..4).map(|t| spot_named("anvil, first-view", t).name).collect();
        assert_eq!(names, ["anvil", "first-view", "anvil", "first-view"]);
    }

    #[test]
    fn going_there_is_the_game_own_lines() {
        let s = spot_named("anvil", 0);
        assert_eq!(go_lines(s), ["set gamehour to 19", "cow Tamriel -48 -10"]);
        let s = spot_named("imperial-city", 0);
        assert_eq!(go_lines(s)[1], "coc WeyeExterior");
        assert_eq!(place_lines(s).last().unwrap(), "fw 00038EEE");
    }

    #[test]
    fn every_spot_is_in_the_cell_it_goes_to() {
        // (A named cell's spot is within the grid the game loads round it: two cells either way.)
        for s in SPOTS {
            let g = ((s.pos[0] / 4096.0).floor(), (s.pos[1] / 4096.0).floor());
            assert!(g.0.abs() < 64.0 && g.1.abs() < 64.0, "{}", s.name);
        }
    }
}
