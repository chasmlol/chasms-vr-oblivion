//! People look at the player's real head: the headset, wherever the room and the player's height
//! put it, not the game's own eye (the first-person skeleton's `Camera01`, which only the
//! thumbstick moves). Every look at the player, anywhere in the game (idle head tracking, being
//! spoken to, the dialogue menu, greetings, combat, `Look`, the voice's listeners), turns the
//! player into a point with `PlayerCharacter`'s look-at point (0x6604C0, a reference's vtable
//! +0x11C), and the actor's head tracking update (0x603500) aims the head, the eyes and the upper
//! body's twist at it. That point is the headset's here. See `docs/research/npcs.md`.
//!
//! The game's frame may not run its people's head tracking while a menu is open, so in the
//! dialogue menu whoever looks at the player is aimed at the headset each frame here, as 0x603500
//! aims them (the same aim twice if it does run).

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::{camera, combat, engine, hooks, mem};
    use rustvr_adapter::View;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    /// `PlayerCharacter`'s look-at point (thiscall: the player, the point out; returns it): in
    /// first person `Camera01`'s position, 6 units down (the eye to the face), and its first bytes.
    const PLAYER_LOOK_AT: usize = 0x0066_04C0;
    const PLAYER_LOOK_AT_BYTES: [u8; 12] = [0x83, 0xEC, 0x0C, 0x80, 0xB9, 0x88, 0x05, 0x00, 0x00, 0x00, 0x75, 0x43];
    /// The game's drop from the eye to the point looked at (its constant at 0xA3F3A0).
    const FACE_DROP: f32 = 6.0;
    /// A reference's look-at point (vtable +0x11C), the actor's process (+0x58, a `HighProcess`
    /// near the player), its pick of what to look at (vtable +0x4CC) and its face's animation
    /// (vtable +0xE8, thiscall: the actor); the face's look (vtable +0x84: the direction, 0).
    const VT_LOOK_AT: usize = 0x11C;
    const PROCESS: usize = 0x58;
    const HIGH_PROCESS_VTABLE: usize = 0x00A7_1814;
    const VT_LOOKING_AT: usize = 0x4CC;
    const VT_FACE: usize = 0xE8;
    const FACE_VTABLE: usize = 0x00A6_424C;
    const VT_FACE_LOOK: usize = 0x84;
    /// The headset's point is the game's this long after the view last came (a frame's, really).
    const FRESH: Duration = Duration::from_millis(500);

    static REAL_LOOK_AT: AtomicUsize = AtomicUsize::new(0);
    /// The headset's point (the game's units), and when.
    static HEAD: Mutex<Option<([f32; 3], Instant)>> = Mutex::new(None);

    pub fn install() {
        if mem::at::<[u8; 12]>(PLAYER_LOOK_AT) != Some(PLAYER_LOOK_AT_BYTES) {
            tracing::warn!("look: PlayerCharacter's look-at point isn't the one known: people look at the game's eye");
            return;
        }
        hooks::install_into(PLAYER_LOOK_AT as *const (), look_at as *const (), "PlayerCharacter look-at point", &REAL_LOOK_AT);
    }

    /// Once a frame, from the runtime's view (none: the game's own eye again).
    pub fn publish(view: Option<&View>) {
        let point = view.map(|v| camera::basis().point_to_game(v.head_pos)).filter(|p| p.is_finite());
        *HEAD.lock().unwrap() = point.map(|p| ([p.x, p.y, p.z - FACE_DROP], Instant::now()));
    }

    /// The headset's point people look at (the game's units), while there is one.
    pub fn head() -> Option<[f32; 3]> {
        HEAD.lock().unwrap().filter(|(_, at)| at.elapsed() < FRESH).map(|(p, _)| p)
    }

    unsafe extern "thiscall" fn look_at(this: usize, out: *mut [f32; 3]) -> *mut [f32; 3] {
        let real: unsafe extern "thiscall" fn(usize, *mut [f32; 3]) -> *mut [f32; 3] = unsafe { std::mem::transmute(REAL_LOOK_AT.load(Ordering::Acquire)) };
        // (the game's point first: it stays when there's no headset's)
        let r = unsafe { real(this, out) };
        if let Some(p) = hooks::guard("look-at", None, head) {
            unsafe { out.write_unaligned(p) };
        }
        r
    }

    fn vcall(obj: usize, slot: usize) -> usize {
        let f = mem::deref(mem::deref(obj) + slot);
        if !engine::CODE.contains(&f) {
            return 0;
        }
        type F = unsafe extern "thiscall" fn(usize) -> usize;
        unsafe { std::mem::transmute::<usize, F>(f)(obj) }
    }

    fn look_point(refr: usize) -> Option<[f32; 3]> {
        let f = mem::deref(mem::deref(refr) + VT_LOOK_AT);
        if !engine::CODE.contains(&f) {
            return None;
        }
        type F = unsafe extern "thiscall" fn(usize, *mut [f32; 3]) -> *mut [f32; 3];
        let mut p = [0.0f32; 3];
        unsafe { std::mem::transmute::<usize, F>(f)(refr, &mut p) };
        p.iter().all(|c| c.is_finite()).then_some(p)
    }

    /// In the dialogue menu: everyone looking at the player has their face aimed at the headset,
    /// as the head tracking update aims it (the direction from their own look-at point).
    pub fn aim_in_dialogue() {
        hooks::guard_void("look in dialogue", || {
            let Some(head) = head() else { return };
            let player = engine::player();
            for actor in combat::high_actors() {
                let process = mem::deref(actor + PROCESS);
                if process == 0 || mem::deref(process) != HIGH_PROCESS_VTABLE || vcall(process, VT_LOOKING_AT) != player {
                    continue;
                }
                let face = {
                    let f = mem::deref(mem::deref(process) + VT_FACE);
                    if !engine::CODE.contains(&f) {
                        continue;
                    }
                    type F = unsafe extern "thiscall" fn(usize, usize) -> usize;
                    unsafe { std::mem::transmute::<usize, F>(f)(process, actor) }
                };
                if face == 0 || mem::deref(face) != FACE_VTABLE {
                    continue;
                }
                let Some(from) = look_point(actor) else { continue };
                let d = [head[0] - from[0], head[1] - from[1], head[2] - from[2]];
                let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if length < 1.0 {
                    continue;
                }
                let f = mem::deref(mem::deref(face) + VT_FACE_LOOK);
                if !engine::CODE.contains(&f) {
                    continue;
                }
                type Look = unsafe extern "thiscall" fn(usize, f32, f32, f32, u32);
                unsafe { std::mem::transmute::<usize, Look>(f)(face, d[0] / length, d[1] / length, d[2] / length, 0) };
            }
        });
    }
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn publish(_view: Option<&rustvr_adapter::View>) {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn aim_in_dialogue() {}
