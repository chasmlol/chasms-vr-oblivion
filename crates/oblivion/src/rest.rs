//! Sitting and sleeping with the body (`docs/research/interact.md`, "Sitting and sleeping"):
//!
//! * **Sitting down on furniture.** The player sitting down for real (the head below [`SIT`] of
//!   their standing eye height, held [`SIT_HOLD`]) on a chair, a bench or a bed (its collision
//!   under the head, within [`SEAT_REACH`] along the floor) is the player's activation of it: the
//!   game's own furniture, so a chair or bench seats them (its marker, its animation, its sit state)
//!   and a bed asks its own way (its sleep and wait menu, with its ownership and trespass rules: "You
//!   cannot sleep in an owned bed", its crime). Standing up again for real (the head over
//!   [`STAND`]) is the game's own Activate while seated: it stands them up.
//! * **The view while seated**: the game lowers its eye onto the seat; the player has sat down for
//!   real too, so the view's anchor stays at the height the game's eye has over the feet standing
//!   (the body's own sitting does the lowering, once).

/// The head below this share of the standing eye height: sitting (or lying); back over this:
/// standing up.
pub const SIT: f32 = 0.78;
pub const STAND: f32 = 0.9;
/// How long the head stays low (seconds) before the furniture is sat on, and high before standing.
pub const SIT_HOLD: f64 = 0.3;
pub const STAND_HOLD: f64 = 0.3;
/// The furniture's collision within this of the head along the floor (metres), at a seat's height.
pub const SEAT_REACH: f32 = 0.45;
pub const SEAT_HEIGHT: f32 = 0.45;

/// Whether the head `height` over the floor is a sitting one for a player whose standing eye height
/// is `standing` (and a standing one, `up`).
pub fn sitting(height: f32, standing: f32) -> bool {
    standing > 0.5 && height < standing * SIT
}

pub fn standing_up(height: f32, standing: f32) -> bool {
    standing > 0.5 && height > standing * STAND
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn anchor_eye(eye: glam::Vec3) -> glam::Vec3 {
    eye
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem};
    use glam::Vec3;
    use rustvr_protocol::*;
    use std::sync::Mutex;

    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DISABLED: u32 = 0x800;
    const FURNITURE: u8 = 0x20;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    /// The player's sit and sleep state (`BaseProcess::GetSitSleepState`): 0 normal, 4 seated.
    const NORMAL: u32 = 0;
    const SEATED: u32 = 4;
    /// Furniture within this of the player (metres) is looked at.
    const NEAR: f32 = 2.5;

    #[derive(Default)]
    struct State {
        /// The head low since when; already sat (not again until it's up).
        low_since: Option<f64>,
        high_since: Option<f64>,
        sat: bool,
        sit_on: Option<usize>,
        stand: bool,
        /// The game's eye over the feet standing (metres), as last seen standing.
        standing_eye: Option<f32>,
        furniture: Vec<(usize, Vec<[Vec3; 3]>)>,
        next_scan: f64,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn feet() -> Option<Vec3> {
        mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)))
    }

    /// The view's anchor for the game's eye `eye`: seated in furniture, at the game's standing eye
    /// height over the feet (the player has sat down for real); else the game's eye as it is.
    pub fn anchor_eye(eye: Vec3) -> Vec3 {
        let Some(feet) = feet() else { return eye };
        let state = crate::riding::sit_state(engine::player());
        with_state(|s| {
            if state == NORMAL && !crate::riding::mounted() && !crate::player::sneaking() {
                s.standing_eye = Some(eye.y - feet.y);
            }
            match s.standing_eye {
                Some(up) if state == SEATED && !crate::riding::mounted() => eye.with_y(feet.y + up),
                _ => eye,
            }
        })
    }

    fn scan(s: &mut State) {
        let b = camera::basis();
        let Some(me) = feet() else { return };
        let mut found = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let base = if r != 0 { mem::deref(r + REFR_BASE) } else { 0 };
                if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(FURNITURE) || mem::at::<u32>(r + FORM_FLAGS).is_none_or(|f| f & DISABLED != 0) || mem::deref(r + REFR_NODE) == 0 {
                    continue;
                }
                let Some(p) = mem::at::<[f32; 3]>(r + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { continue };
                if p.distance(me) < NEAR {
                    found.push((r, crate::lock::triangles_of(r)));
                }
            }
        }
        s.furniture = found;
    }

    /// Before the eyes are drawn: the player sitting down by furniture, or standing up from it.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        if crate::menus::menu_mode() || crate::riding::mounted() {
            return;
        }
        let Some(body) = crate::link::with_adapter(|a| a.read_body()).flatten() else { return };
        let (height, standing) = (f.hmd.pos[1], body.eye_height);
        let Some(feet) = feet() else { return };
        let state = crate::riding::sit_state(engine::player());
        with_state(|s| {
            if now >= s.next_scan {
                s.next_scan = now + 0.5;
                scan(s);
            }
            if state == SEATED {
                // Seated: standing up for real is the game's own Activate, which stands them up.
                s.low_since = None;
                if standing_up(height, standing) {
                    let since = *s.high_since.get_or_insert(now);
                    if now - since > STAND_HOLD && !s.stand {
                        s.stand = true;
                        tracing::info!("rest: standing up from the furniture");
                    }
                } else {
                    s.high_since = None;
                }
                return;
            }
            s.high_since = None;
            if !sitting(height, standing) {
                s.low_since = None;
                s.sat = false;
                return;
            }
            if s.sat || state != NORMAL {
                return;
            }
            let since = *s.low_since.get_or_insert(now);
            if now - since < SIT_HOLD {
                return;
            }
            // Under the head, at a seat's height over the feet.
            let seat = view.head_pos.with_y(feet.y + SEAT_HEIGHT);
            let under = s.furniture.iter().filter_map(|(r, tris)| crate::lock::nearest(tris, seat).map(|n| (*r, (n.0 - seat).with_y(0.0).length()))).filter(|n| n.1 < SEAT_REACH).min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((r, d)) = under {
                tracing::info!("rest: sitting down on {:08X} ({d:.2} m from the head along the floor)", mem::at::<u32>(r + FORM_ID).unwrap_or(0));
                s.sat = true;
                s.sit_on = Some(r);
            }
        });
    }

    /// Each game frame: the furniture sat on (its activation: sat in, or its bed's own asking), or
    /// stood up from (the game's Activate while seated).
    pub fn frame() {
        let (sit, stand) = with_state(|s| (s.sit_on.take(), std::mem::take(&mut s.stand)));
        if let Some(r) = sit {
            let ok = crate::world::activate(r);
            tracing::info!("rest: the furniture's activation ({ok})");
        }
        if stand {
            crate::player::press("Activate");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitting_and_standing_by_the_heads_height() {
        assert!(sitting(1.0, 1.6));
        assert!(!sitting(1.5, 1.6));
        assert!(standing_up(1.5, 1.6));
        assert!(!standing_up(1.2, 1.6), "half up");
    }
}
