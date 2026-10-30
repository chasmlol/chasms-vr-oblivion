//! The Ayleids' magic by hand (`docs/research/interact.md`, "The big magic moments"):
//!
//! * **An Ayleid well's light.** A hand put into the column of light over an Ayleid well and held
//!   there ([`WELL_HOLD`], felt building) is the player's activation of the well: its own script
//!   (`AyleidWell`), so its own rules: once a day its power (the `AyleidPower` spell cast on the
//!   player, its words, its animation), else "A drained well recharges slowly."
//! * **Welkynd and Varla stones crushed in the hand.** A Welkynd stone (magicka) or a Varla stone
//!   (recharging) in a hand, the trigger squeezed hard on it, is used as the inventory's click uses
//!   it (the game's `EquipItem`: its magicka or its recharge, its words, its sound, the stone gone):
//!   one drawn from the pack (the Ayleid Stones slot), or one taken from where it lies in a ruin (the
//!   game's own pick-up, then its use).

use glam::Vec3;

/// How long a hand is held in a well's light to draw its power (seconds).
pub const WELL_HOLD: f64 = 0.5;
/// The light's column over a well: how wide round its middle (metres), and from how far over its
/// reference's place to how far.
pub const WELL_RADIUS: f32 = 0.45;
pub const WELL_FROM: f32 = 0.2;
pub const WELL_TO: f32 = 3.0;
/// How hard the trigger is squeezed on a stone to crush it.
pub const CRUSH: f32 = 0.92;

/// A point in a well's column of light (`well`: its reference's place; the protocol's world, Y up).
pub fn in_light(well: Vec3, p: Vec3) -> bool {
    let up = p.y - well.y;
    (WELL_FROM..=WELL_TO).contains(&up) && Vec3::new(p.x - well.x, 0.0, p.z - well.z).length() <= WELL_RADIUS
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem};
    use rustvr_protocol::*;
    use std::sync::Mutex;

    /// The well's script (`AyleidWell`, Oblivion.esm's 0x6A8E1); an activator's script
    /// (`TESScriptableForm` +0x48, its script +4).
    const WELL_SCRIPT: u32 = 0x0006_A8E1;
    const ACTIVATOR: u8 = 0x12;
    const ACTI_SCRIPT: usize = 0x4C;
    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    /// Wells within this of the player (metres) are looked at.
    const NEAR: f32 = 4.0;

    #[derive(Default)]
    struct State {
        /// The well a hand is in the light of, since when, which hand; whether it was drawn on
        /// (not again until the hand has left the light).
        in_light: Option<(usize, f64, usize)>,
        drawn: bool,
        next_tick: f64,
        activate: Option<usize>,
        /// A stone crushed in a hand (its object), and one lying about taken and crushed (its
        /// reference and form).
        crushed: Vec<u32>,
        squeezed: [bool; 2],
        picked: Vec<(usize, usize)>,
        next_scan: f64,
        wells: Vec<(usize, Vec3)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    /// The Ayleid wells near the player: each reference and its place.
    fn wells() -> Vec<(usize, Vec3)> {
        let b = camera::basis();
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { return Vec::new() };
        let mut out = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let base = if r != 0 { mem::deref(r + REFR_BASE) } else { 0 };
                if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(ACTIVATOR) || mem::at::<u32>(r + FORM_FLAGS).is_none_or(|f| f & DISABLED != 0) {
                    continue;
                }
                let script = mem::deref(base + ACTI_SCRIPT);
                if script == 0 || mem::at::<u32>(script + FORM_ID) != Some(WELL_SCRIPT) {
                    continue;
                }
                let Some(p) = mem::at::<[f32; 3]>(r + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { continue };
                if p.distance(me) < NEAR {
                    out.push((r, p));
                }
            }
        }
        out
    }

    fn is_stone(form: usize) -> bool {
        mem::at::<u32>(form + FORM_ID).is_some_and(|id| crate::inventory::AYLEID_STONE_FORMS.contains(&id))
    }

    /// Before the eyes are drawn: a hand in a well's light; a stone crushed in a hand.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        if crate::menus::menu_mode() {
            return;
        }
        let copies = crate::inventory::held_copies();
        let lying = crate::items::held();
        with_state(|s| {
            if now >= s.next_scan {
                s.next_scan = now + 1.0;
                s.wells = wells();
            }
            // A hand in a well's light.
            let hands: Vec<Vec3> = (0..2).map(|i| view.point(Vec3::from_array(f.hands[i].grip.pos))).collect();
            let found = s.wells.iter().find_map(|&(r, at)| (0..2).find(|&i| in_light(at, hands[i])).map(|i| (r, i)));
            match found {
                Some((r, hand)) => {
                    let since = match s.in_light {
                        Some((w, t, _)) if w == r => t,
                        _ => {
                            s.drawn = false;
                            now
                        }
                    };
                    s.in_light = Some((r, since, hand));
                    let held = now - since;
                    if !s.drawn && held < WELL_HOLD && now >= s.next_tick {
                        s.next_tick = now + 0.07;
                        let k = (held / WELL_HOLD) as f32;
                        crate::link::with_adapter(|a| a.haptic(hand as u32, 0.03, 80.0 + 160.0 * k, 0.15 + 0.5 * k));
                    }
                    if !s.drawn && held >= WELL_HOLD {
                        s.drawn = true;
                        s.activate = Some(r);
                        tracing::info!("ayleid: a hand in the well's light ({:08X})", mem::at::<u32>(r + FORM_ID).unwrap_or(0));
                        crate::link::with_adapter(|a| a.haptic(hand as u32, 0.4, 60.0, 0.9));
                    }
                }
                None => {
                    s.in_light = None;
                    s.drawn = false;
                }
            }
            // A stone squeezed hard in a hand: crushed.
            for hand in 0..2 {
                let hard = f.hands[hand].trigger >= CRUSH;
                let was = std::mem::replace(&mut s.squeezed[hand], hard);
                if !hard || was {
                    continue;
                }
                if let Some(c) = copies.iter().find(|c| c.hand == hand && is_stone(c.form)) {
                    tracing::info!("ayleid: a stone crushed in the hand");
                    s.crushed.push(c.id);
                    crate::link::with_adapter(|a| a.haptic(hand as u32, 0.08, 40.0, 1.0));
                } else if let Some(h) = lying.iter().find(|h| h.hand == hand && is_stone(mem::deref(h.refr + REFR_BASE))) {
                    tracing::info!("ayleid: a stone taken from where it lay and crushed");
                    s.picked.push((h.refr, mem::deref(h.refr + REFR_BASE)));
                    crate::link::with_adapter(|a| a.haptic(hand as u32, 0.08, 40.0, 1.0));
                }
            }
        });
    }

    /// Each game frame: a well drawn on (its own activation); stones crushed, used as the game uses
    /// them.
    pub fn frame() {
        let (well, crushed, picked) = with_state(|s| (s.activate.take(), std::mem::take(&mut s.crushed), std::mem::take(&mut s.picked)));
        if let Some(w) = well {
            let ok = crate::world::activate(w);
            tracing::info!("ayleid: the well's activation ({ok})");
        }
        for id in crushed {
            crate::inventory::use_copy(id);
        }
        for (refr, form) in picked {
            let taken = crate::items::pick_up(refr);
            let used = taken && crate::inventory::use_now(form);
            tracing::info!("ayleid: taken ({taken}) and used ({used})");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_light_is_a_column_over_the_well() {
        let well = Vec3::new(1.0, 0.0, 1.0);
        assert!(in_light(well, Vec3::new(1.2, 1.0, 1.1)));
        assert!(!in_light(well, Vec3::new(1.0, 0.05, 1.0)), "not in the grate");
        assert!(!in_light(well, Vec3::new(1.6, 1.0, 1.0)), "beside it");
    }
}
