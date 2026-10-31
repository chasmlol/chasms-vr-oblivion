//! Keys at the lock (`docs/research/interact.md`, "Keys at the lock"): a key drawn from the body
//! (the Keys slot, a ring on the belt just behind the left hip) with its tip put to a locked door
//! or container goes in if it's that lock's key, a click in the hand; turned in it ([`TURN`] either
//! way, a detent felt every [`DETENT`]) it's the player's activation of the thing, so the game opens
//! the lock with the key in the pack: it unlocks it, and the turn is the Ok of its message box ("You
//! opened the door with ...", the box's own click), so the door or chest opens; its words are the
//! game's corner notice instead. The wrong key bumps and stays out; one taken [`OUT`] from where it
//! went in is out.
//!
//! The Keys slot gives the key to the nearest locked thing it opens first ([`key_strength`]: the
//! slot's Strongest order).

use glam::{Quat, Vec3};

/// How far a key is turned in its lock to open it (radians), and how far apart its detents are.
pub const TURN: f32 = std::f32::consts::FRAC_PI_3;
pub const DETENT: f32 = std::f32::consts::PI / 9.0;
/// How far (metres) the tip goes from where it went in to be out of the lock.
pub const OUT: f32 = 0.06;
/// Locked things within this of the player (metres) put their keys first.
pub const LOCKS_WITHIN: f32 = 10.0;

/// A key's strength for the Keys slot's order: how much nearer than [`LOCKS_WITHIN`] the nearest
/// locked thing it opens is, in whole metres (the order changes as a metre is passed, not every
/// step).
pub fn key_strength(distance: f32) -> f32 {
    (LOCKS_WITHIN - distance).max(0.0).ceil()
}

/// How far `now` is turned from `was` about `axis` (radians, either way: the twist, whatever the
/// swing).
pub fn twist(was: Quat, now: Quat, axis: Vec3) -> f32 {
    let r = (now * was.inverse()).normalize();
    let along = axis.normalize_or_zero().dot(Vec3::new(r.x, r.y, r.z));
    let a = 2.0 * along.atan2(r.w);
    if a > std::f32::consts::PI {
        a - std::f32::consts::TAU
    } else if a < -std::f32::consts::PI {
        a + std::f32::consts::TAU
    } else {
        a
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn keys_near() -> Vec<(usize, f32)> {
    Vec::new()
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::lock::{nearest, tip_of, triangles_of, TOUCH};
    use crate::{camera, engine, mem};
    use rustvr_protocol::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const FORM_TYPE: usize = 0x4;
    const FORM_ID: usize = 0xC;
    const REFR_BASE: usize = 0x1C;
    const DOOR: u8 = 0x18;
    /// The message box's words, as the game's door and container say them (`sOpenWithKey`, then
    /// the key's name and a full stop; `sOpenedContainer`): `char*`s.
    const OPEN_WITH_KEY: usize = 0x00B3_86A0;
    const OPENED_CONTAINER: usize = 0x00B3_8C30;
    /// Locked things within this of the tip (metres, their references' places) are looked at.
    const NEAR: f32 = 3.0;
    /// Seconds a thing's triangles are kept before they're read again.
    const TRIANGLES_FOR: f64 = 1.0;
    /// Seconds after the turn its message box may come.
    const ASKS_WITHIN: f64 = 2.0;

    /// A key in its lock: the thing, where the tip went in, the key's way along (grip to tip) and
    /// its turn then, the hand, the detents passed.
    struct InLock {
        refr: usize,
        at: Vec3,
        axis: Vec3,
        rot: Quat,
        hand: usize,
        detents: i32,
    }

    #[derive(Default)]
    struct State {
        in_lock: Option<InLock>,
        /// Not to try again until the tip has gone from here (the wrong key, a key turned).
        spent: Option<Vec3>,
        triangles: HashMap<usize, (f64, Vec<[Vec3; 3]>)>,
        /// A lock turned open: its activation in the next game frame (the thing, the key); then its
        /// message box to answer, until when.
        turned: Option<(usize, usize)>,
        answer: Option<(usize, usize, f64)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn haptic(hand: usize, seconds: f32, freq: f32, amplitude: f32) {
        crate::link::with_adapter(|a| a.haptic(hand as u32, seconds, freq, amplitude));
    }

    /// The locked doors and containers near `p` (the protocol's world): each reference and how far.
    fn locked_near(p: Vec3, within: f32) -> Vec<(usize, f32)> {
        let b = camera::basis();
        let mut out = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let Some(at) = (r != 0).then(|| mem::at::<[f32; 3]>(r + engine::refr::POS)).flatten() else { continue };
                let d = b.point_from_game(Vec3::from(at)).distance(p);
                if d < within && crate::lock::locked_thing(r) {
                    out.push((r, d));
                }
            }
        }
        out
    }

    /// The keys to the locked things near the player, each with how near the nearest it opens is
    /// (metres): for the Keys slot's order.
    pub fn keys_near() -> Vec<(usize, f32)> {
        let b = camera::basis();
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { return Vec::new() };
        let mut keys: HashMap<usize, f32> = HashMap::new();
        for (r, d) in locked_near(me, LOCKS_WITHIN) {
            let key = crate::world::lock_key(r);
            if key != 0 {
                let k = keys.entry(key).or_insert(d);
                *k = k.min(d);
            }
        }
        keys.into_iter().collect()
    }

    /// The key in a hand (a copy drawn from the body's slots): its form, tip, way along (grip to
    /// tip), turn, and the hand.
    fn key_in_hand(view: &rustvr_adapter::View, f: &RtFrame) -> Option<(usize, Vec3, Vec3, Quat, usize)> {
        crate::inventory::held_copies().into_iter().find(|c| mem::at::<u8>(c.form + FORM_TYPE) == Some(crate::items::KEY)).map(|c| {
            let (pos, rot) = if c.flags & OS_TRACKING != 0 { view.pose(&c.pose) } else { (Vec3::from_array(c.pose.pos), Quat::from_array(c.pose.rot)) };
            let grip = view.point(Vec3::from_array(f.hands[c.hand].grip.pos));
            let tip = tip_of(pos, rot, c.center, c.half, grip);
            (c.form, tip, (tip - grip).normalize_or(rot * Vec3::Z), rot, c.hand)
        })
    }

    /// Before the eyes are drawn: a key's tip put to a locked thing (in, if it's that lock's key),
    /// and turned in it.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        if crate::menus::menu_mode() {
            return;
        }
        let key = key_in_hand(view, f);
        with_state(|s| {
            let Some((form, tip, axis, rot, hand)) = key else {
                s.in_lock = None;
                s.spent = None;
                return;
            };
            if let Some(k) = s.in_lock.as_mut() {
                // Taken out: not in any more.
                if tip.distance(k.at) > OUT || hand != k.hand {
                    tracing::info!("keys: the key taken out of {:08X}", mem::at::<u32>(k.refr + FORM_ID).unwrap_or(0));
                    s.in_lock = None;
                    return;
                }
                let turned = twist(k.rot, rot, k.axis);
                let detents = (turned / DETENT).trunc() as i32;
                if detents != k.detents {
                    k.detents = detents;
                    haptic(hand, 0.02, 140.0, 0.35);
                }
                if turned.abs() >= TURN {
                    tracing::info!("keys: the key turned in {:08X} ({:.0} degrees)", mem::at::<u32>(k.refr + FORM_ID).unwrap_or(0), turned.to_degrees());
                    haptic(hand, 0.08, 50.0, 0.9);
                    s.turned = Some((k.refr, form));
                    s.spent = Some(tip);
                    s.in_lock = None;
                }
                return;
            }
            if s.spent.is_some_and(|at| at.distance(tip) < OUT) || s.turned.is_some() {
                return;
            }
            s.spent = None;
            let near = locked_near(tip, NEAR);
            s.triangles.retain(|r, (t, _)| near.iter().any(|n| n.0 == *r) && now - *t < TRIANGLES_FOR);
            for (refr, _) in near {
                let tris = &s.triangles.entry(refr).or_insert_with(|| (now, triangles_of(refr))).1;
                let Some((at, _, d)) = nearest(tris, tip) else { continue };
                if d > TOUCH {
                    continue;
                }
                let id = mem::at::<u32>(refr + FORM_ID).unwrap_or(0);
                if crate::world::lock_key(refr) == form {
                    tracing::info!("keys: the key in {id:08X}'s lock at {at:.2?}");
                    haptic(hand, 0.03, 180.0, 0.6);
                    s.in_lock = Some(InLock { refr, at: tip, axis, rot, hand, detents: 0 });
                } else {
                    // (The wrong key: it doesn't go in.)
                    tracing::info!("keys: not {id:08X}'s key");
                    haptic(hand, 0.05, 40.0, 0.5);
                    s.spent = Some(tip);
                }
                return;
            }
        });
    }

    /// Each game frame: a key turned is the player's activation of its lock (the game unlocks it with
    /// the key in the pack), and its message box's Ok is the turn.
    pub fn frame(now: f64) {
        if let Some((refr, key)) = with_state(|s| s.turned.take()) {
            let ok = crate::world::activate(refr);
            tracing::info!("keys: the lock's activation ({ok})");
            with_state(|s| s.answer = Some((refr, key, now + ASKS_WITHIN)));
            return;
        }
        let Some((refr, key, until)) = with_state(|s| s.answer) else { return };
        if now > until {
            with_state(|s| s.answer = None);
            return;
        }
        // (Its one button, Ok: a box with a question is someone else's.)
        let Some((menu, 1)) = crate::menus::message_box() else { return };
        with_state(|s| s.answer = None);
        let door = mem::at::<u8>(mem::deref(refr + REFR_BASE) + FORM_TYPE) == Some(DOOR);
        let words = if door {
            format!("{}{}.", crate::combat::read_str(mem::deref(OPEN_WITH_KEY)), crate::inventory::name_of(key))
        } else {
            crate::combat::read_str(mem::deref(OPENED_CONTAINER))
        };
        tracing::info!("keys: its box answered by the turn: {words}");
        crate::menus::click_message(menu, 0);
        crate::menus::notice(&words);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_nearest_lock_first() {
        assert_eq!(key_strength(0.4), 10.0);
        assert_eq!(key_strength(3.5), 7.0);
        assert!(key_strength(2.0) > key_strength(6.0));
        assert_eq!(key_strength(12.0), 0.0);
    }

    #[test]
    fn a_key_turned_about_itself() {
        let axis = Vec3::Z;
        let was = Quat::from_rotation_y(0.3);
        let now = Quat::from_axis_angle(axis, 1.1) * was;
        assert!((twist(was, now, axis) - 1.1).abs() < 1e-4);
        assert!((twist(was, Quat::from_axis_angle(axis, -0.7) * was, axis) + 0.7).abs() < 1e-4, "the other way");
        // (Swung, not turned: the key's tip moved, not the key about itself.)
        assert!(twist(was, Quat::from_rotation_x(0.8) * was, axis).abs() < 1e-4);
    }
}
