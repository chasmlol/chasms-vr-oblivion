//! Oblivion's lock, worked by the hand: while the game's lock is open (`LockPickMenu`, from the
//! game's own Activate on a locked door or container with picks), the hand is the pick. It's
//! Oblivion's own lock throughout: its five tumblers, their speeds from the lock's level, the
//! Security skill's windows, picks broken and used up, auto-attempt; the game draws it and decides
//! everything, and the hand only works its pick:
//!
//! * **Under a tumbler**: the hand moved sideways from where it was when the lock opened, a
//!   tumbler every [`SPACING`] (the game's own pick slides under it, with its scrape), as the
//!   game's cursor would put it there.
//! * **Pushed up**: the hand moved up ([`FLICK`]) pushes the tumbler up, as the mouse moved up
//!   does on a flat screen (the game's lift, at its speed for the lock).
//! * **Set**: the pick hand's trigger at the top of the tumbler's rise, as the mouse's click (the
//!   game's window, from the Security skill).
//! * **Felt**: the tumbler riding up on the pick ticks in the hand, a set tumbler clicks, a broken
//!   pick jolts it.
//!
//! The menu's own layout isn't read: the game's cursor is moved along the row of tumblers until the
//! game puts its pick under the one the hand is at (`LockPickMenu` +0x168, the tumbler under the
//! cursor), and where each tumbler was found is kept for the next time. The menu's own buttons are
//! the controller's: A is Auto Attempt (the game's own click on it), B closes it (Escape, as on any
//! menu).

use rustvr_protocol::RtFrame;

/// How far apart (metres) the hand's places for the five tumblers are.
pub const SPACING: f32 = 0.025;
/// How fast (m/s) the hand moves up to push the pick, and the mouse's movement (counts a frame, up)
/// that is to the game (it lifts the pick past 3).
pub const FLICK: f32 = 0.3;
const LIFT: i32 = -12;
/// The trigger's pull that sets a tumbler.
const SET_PULL: f32 = 0.75;
/// The row of tumblers on the screen (0..1 from the top left), where the game's lock menu puts its
/// lock (centred).
pub const TUMBLERS: [f32; 2] = [0.48, 0.44];

/// The tumbler (0..4) the hand is under, `lateral` metres to the right of where it began.
pub fn tumbler(lateral: f32) -> usize {
    ((lateral / SPACING).round() + 2.0).clamp(0.0, 4.0) as usize
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::mem;
    use glam::{Quat, Vec3};
    use std::sync::Mutex;

    /// `kMenuType_LockPick`; the menus by type (`g_TileMenuArray`'s data); a menu tile's `Menu`
    /// (+0x44); the lock menu's vtable.
    const LOCKPICK: u32 = 0x3F6;
    const TILE_MENUS: usize = 0x00B1_3974;
    const TILE_MENU: usize = 0x44;
    const LOCKPICK_VTABLE: usize = 0x00A6_C73C;
    /// `LockPickMenu`: the picks left (+0x3C); its five tumblers (40 bytes each from +0x7C): moving
    /// (+0x94), set (+0x95); the tumbler under the cursor (+0x168, -1 none).
    const PICKS: usize = 0x3C;
    const TUMBLER: usize = 0x7C;
    const TUMBLER_SIZE: usize = 40;
    const MOVING: usize = 0x94 - TUMBLER;
    const SET: usize = 0x95 - TUMBLER;
    const UNDER_CURSOR: usize = 0x168;
    /// `LockPickMenu::HandleClick(tile id, tile)` (thiscall, its vtable's third), and its Auto
    /// Attempt button's id.
    const HANDLE_CLICK: usize = 0x005B_0A00;
    const AUTO_ATTEMPT: u32 = 5;
    /// How far the cursor moves a frame (of the screen's width) while it looks for a tumbler.
    const STEP: f32 = 0.004;

    #[derive(Default)]
    struct State {
        open: bool,
        /// The hand where the lock opened (tracking), and the head's right then.
        start: Vec3,
        right: Vec3,
        /// The cursor (the screen, 0..1), and where each tumbler was found under it.
        cursor: [f32; 2],
        found: [Option<f32>; 5],
        picks: i32,
        set: [bool; 5],
        next_tick: f64,
        a: bool,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn menu() -> usize {
        let data = mem::deref(TILE_MENUS);
        let tile = if data != 0 { mem::deref(data + 4 * (LOCKPICK - 0x3E9) as usize) } else { 0 };
        let menu = if tile != 0 { mem::deref(tile + TILE_MENU) } else { 0 };
        (menu != 0 && mem::deref(menu) == LOCKPICK_VTABLE).then_some(menu).unwrap_or(0)
    }

    fn haptic(hand: u32, seconds: f32, freq: f32, amplitude: f32) {
        crate::link::with_adapter(|a| a.haptic(hand, seconds, freq, amplitude));
    }

    /// Each game frame, after the laser has put the game's cursor: with the lock open, the hand
    /// `hand` (0 left, 1 right) works the pick (the cursor's place along the lock, the mouse's
    /// movement and its button in `v`).
    pub fn frame(f: &RtFrame, hand: usize, v: &mut crate::input::Virtual, now: f64) {
        let mut st = STATE.lock().unwrap();
        let s = st.get_or_insert_with(State::default);
        let menu = if crate::menus::menu_open(LOCKPICK) { menu() } else { 0 };
        let c = &f.hands[hand];
        let at = Vec3::from(c.grip.pos);
        if menu == 0 {
            if s.open {
                tracing::info!("lock: closed");
            }
            s.open = false;
            return;
        }
        let picks = mem::at::<i32>(menu + PICKS).unwrap_or(0);
        let tumblers: Vec<(bool, bool)> = (0..5).map(|k| (mem::at::<u8>(menu + TUMBLER + k * TUMBLER_SIZE + MOVING).unwrap_or(0) != 0, mem::at::<u8>(menu + TUMBLER + k * TUMBLER_SIZE + SET).unwrap_or(0) != 0)).collect();
        if !s.open {
            let head = Quat::from_array(f.hmd.rot);
            let right = (head * Vec3::X).with_y(0.0).normalize_or_zero();
            *s = State { open: true, start: at, right, cursor: TUMBLERS, picks, set: [false; 5], ..State::default() };
            for (k, t) in tumblers.iter().enumerate() {
                s.set[k] = t.1;
            }
            tracing::info!("lock: open, {picks} picks; the hand is the pick");
        }
        // Under which tumbler: the cursor to where the game put that one, or on along the lock
        // until it does.
        let want = tumbler((at - s.start).dot(s.right));
        let under = mem::at::<i32>(menu + UNDER_CURSOR).unwrap_or(-1);
        let moving = tumblers.iter().any(|t| t.0);
        if (0..5).contains(&under) {
            s.found[under as usize] = Some(s.cursor[0]);
        }
        if !moving && under != want as i32 {
            s.cursor[0] = match s.found[want] {
                Some(x) => x,
                None if under < 0 => s.cursor[0] + (0.5 - s.cursor[0]).clamp(-STEP, STEP),
                None => s.cursor[0] + if (under as usize) < want { STEP } else { -STEP },
            }
            .clamp(0.0, 1.0);
        }
        s.cursor[1] = TUMBLERS[1];
        crate::menus::point(s.cursor);
        // Pushed up: the mouse moved up, to the game. Set: its button. A: Auto Attempt.
        if c.vel[1] > FLICK {
            v.mouse[1] = LIFT;
        }
        if c.trigger > SET_PULL {
            v.buttons |= 1;
        }
        let a = c.buttons & rustvr_protocol::CTRL_A != 0;
        if a && !s.a {
            type Click = unsafe extern "thiscall" fn(usize, u32, usize);
            tracing::info!("lock: Auto Attempt");
            unsafe { std::mem::transmute::<usize, Click>(HANDLE_CLICK)(menu, AUTO_ATTEMPT, 0) };
        }
        s.a = a;
        // Felt: the tumbler riding the pick, one set, a pick broken.
        if moving && now >= s.next_tick {
            s.next_tick = now + 0.1;
            haptic(hand as u32, 0.012, 200.0, 0.25);
        }
        for (k, t) in tumblers.iter().enumerate() {
            if t.1 && !s.set[k] {
                haptic(hand as u32, 0.04, 150.0, 0.8);
                tracing::info!("lock: tumbler {k} set");
            }
            s.set[k] = t.1;
        }
        if picks < s.picks {
            haptic(hand as u32, 0.12, 60.0, 1.0);
            tracing::info!("lock: a pick broke, {picks} left");
        }
        s.picks = picks;
    }
}
