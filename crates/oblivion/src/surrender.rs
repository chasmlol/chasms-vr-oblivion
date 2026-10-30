//! Yielding and surrendering with the hands up (`docs/research/interact.md`, "Surrendering and
//! yielding"): both hands raised over the head, apart, held [`HOLD`], before someone fighting the
//! player (or a guard, while the player has a bounty) is the game's own yield: the player's
//! activation of them with Block held, the way the game's player yields (0x524660): in a fight, the
//! foe's own answer ("accepts your yield.", "rejects your yield!", "Creatures never accept a
//! yield!"), and a guard who accepts goes back to arresting; a guard pursuing a criminal opens their
//! crime dialogue (pay the fine, go to jail, resist arrest) floating at them as any dialogue.

use glam::Vec3;

/// How long both hands are held up (seconds).
pub const HOLD: f64 = 0.7;
/// Each hand this far over the head (metres), and the hands at least this far apart.
pub const OVER_HEAD: f32 = 0.05;
pub const APART: f32 = 0.25;
/// Who's yielded to: within this (metres), and this near the way the head looks (its cosine).
pub const REACH: f32 = 8.0;
pub const FACING: f32 = 0.6;

/// Both hands up: each over the head, and apart.
pub fn hands_up(head: Vec3, left: Vec3, right: Vec3) -> bool {
    left.y > head.y + OVER_HEAD && right.y > head.y + OVER_HEAD && left.distance(right) > APART
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem};
    use rustvr_protocol::*;
    use std::sync::Mutex;

    const FORM_ID: usize = 0xC;
    /// An actor fighting (vtable +0x334, thiscall: 1); the player's bounty (vtable +0x250);
    /// whether an actor is dead (vtable +0x198, thiscall: 0; as [`crate::grabs`] reads it).
    const VT_IN_COMBAT: usize = 0x334;
    const VT_CRIME_GOLD: usize = 0x250;
    const VT_DEAD: usize = 0x198;
    /// Frames the game's Block is held before the activation (its input reads it the frame after).
    const BLOCK_FIRST: u32 = 2;

    #[derive(Default)]
    struct State {
        up_since: Option<f64>,
        done: bool,
        next_tick: f64,
        /// Whom to yield to, and the frames left before the activation.
        to: Option<(usize, u32)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn virt(obj: usize, at: usize) -> Option<usize> {
        let f = mem::deref(mem::deref(obj) + at);
        (obj != 0 && (0x0040_1000..0x00A3_0000).contains(&f)).then_some(f)
    }

    fn in_combat(actor: usize) -> bool {
        virt(actor, VT_IN_COMBAT).is_some_and(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u8) -> u8>(f)(actor, 1) } != 0)
    }

    fn dead(actor: usize) -> bool {
        virt(actor, VT_DEAD).is_some_and(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u8) -> u8>(f)(actor, 0) } != 0)
    }

    fn bounty() -> f32 {
        let p = engine::player();
        virt(p, VT_CRIME_GOLD).map_or(0.0, |f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> f32>(f)(p) })
    }

    /// Whom the player yields to: of those fighting (or, with a bounty, anyone: a guard's to arrest
    /// them), the one the head looks at most nearly, within reach.
    fn yields_to(head: Vec3, look: Vec3) -> Option<usize> {
        let b = camera::basis();
        let wanted = bounty() > 0.0;
        crate::combat::known_actors()
            .into_values()
            .filter(|&a| crate::combat::is_actor(a) && !dead(a) && (in_combat(a) || wanted))
            .filter_map(|a| {
                let p = b.point_from_game(Vec3::from(mem::at::<[f32; 3]>(a + engine::refr::POS)?)) + Vec3::Y * 1.2;
                let to = p - head;
                let facing = to.normalize_or_zero().dot(look);
                (to.length() < REACH && facing > FACING).then_some((a, facing))
            })
            .max_by(|x, y| x.1.total_cmp(&y.1))
            .map(|x| x.0)
    }

    /// Before the eyes are drawn: both hands up before a foe (or a guard), held: the yield asked for.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        if crate::menus::menu_mode() {
            return;
        }
        let head = view.head_pos;
        let hand = |i: usize| view.point(Vec3::from_array(f.hands[i].grip.pos));
        let up = hands_up(head, hand(0), hand(1));
        with_state(|s| {
            if !up {
                s.up_since = None;
                s.done = false;
                return;
            }
            if s.done {
                return;
            }
            let since = *s.up_since.get_or_insert(now);
            let held = now - since;
            if held < HOLD {
                if now >= s.next_tick {
                    s.next_tick = now + 0.12;
                    crate::link::with_adapter(|a| {
                        a.haptic(0, 0.02, 60.0, 0.15);
                        a.haptic(1, 0.02, 60.0, 0.15);
                    });
                }
                return;
            }
            s.done = true;
            match yields_to(head, view.head_rot * Vec3::Z) {
                Some(a) => {
                    tracing::info!("surrender: both hands up before {:08X}", mem::at::<u32>(a + FORM_ID).unwrap_or(0));
                    s.to = Some((a, BLOCK_FIRST));
                }
                None => tracing::info!("surrender: both hands up, but nobody's fighting the player before them"),
            }
        });
    }

    /// Each game frame: the yield, as the game's player yields: Block held, then the activation of
    /// the one yielded to (the frame after: the game reads its input then).
    pub fn frame() {
        let act = with_state(|s| match s.to.as_mut() {
            Some((a, 0)) => {
                let a = *a;
                s.to = None;
                Some(a)
            }
            Some((_, n)) => {
                *n -= 1;
                crate::player::press_for("Block", 0.3);
                None
            }
            None => None,
        });
        if let Some(a) = act.filter(|&a| crate::combat::is_actor(a)) {
            let ok = crate::world::activate(a);
            tracing::info!("surrender: the game's yield (its activation with Block held: {ok})");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hands_up_over_the_head() {
        let head = Vec3::new(0.0, 1.6, 0.0);
        assert!(hands_up(head, Vec3::new(-0.25, 1.85, 0.1), Vec3::new(0.25, 1.85, 0.1)));
        assert!(!hands_up(head, Vec3::new(-0.25, 1.85, 0.1), Vec3::new(0.25, 1.2, 0.3)), "one hand down");
        assert!(!hands_up(head, Vec3::new(0.0, 1.85, 0.1), Vec3::new(0.05, 1.85, 0.1)), "together over the head");
    }
}
