//! Plants harvested by hand (`docs/research/interact.md`, "Harvesting plants by hand"): a hand
//! closing its grip in a plant (a flower's head, a mushroom's cap, a Nirnroot) and pulled away
//! ([`PULL`]) while still closed is the player's activation of it: the game's own harvest (its
//! season's chance, "You successfully harvest %s." or "You find nothing of use.", its sound, the
//! plant marked harvested). What it gave is in the hand that plucked it, if it's still closed: the
//! runtime's copy of the ingredient, as if drawn from a slot (put in a slot it's in the pack, held
//! at the mouth it's eaten, let go it drops).

use glam::Vec3;

/// How far (metres) a closed hand is pulled from where it closed on a plant to pluck it.
pub const PULL: f32 = 0.07;
/// The grip's squeeze that closes the hand on a plant, and that opens it again.
pub const CLOSE: f32 = 0.6;
pub const OPEN: f32 = 0.35;
/// How far outside one of a plant's meshes' bounds (metres) a hand still closes on it.
pub const SPARE: f32 = 0.02;

/// Whether a hand at `p` is in a plant, of the bounds (centre, radius; the protocol's world) of its
/// meshes.
pub fn in_plant(bounds: &[(Vec3, f32)], p: Vec3) -> bool {
    bounds.iter().any(|(c, r)| c.distance(p) <= r + SPARE)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem, nodes};
    use rustvr_protocol::*;
    use std::sync::Mutex;

    /// `kFormType_Flora`; a reference's base points 0xC into its `TESFlora`, whose produce comes
    /// first: the ingredient is at the base - 8.
    const FLORA: u8 = 0x1F;
    const INGREDIENT: isize = -8;
    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    /// A scene graph object's world bound (centre, radius; game units).
    const NODE_BOUND: usize = 0x20;
    /// Plants within this of the player (metres) are looked at.
    const NEAR: f32 = 3.0;

    struct Plant {
        refr: usize,
        bounds: Vec<(Vec3, f32)>,
    }

    #[derive(Default)]
    struct State {
        plants: Vec<Plant>,
        next_scan: f64,
        /// Each hand closed on a plant: the plant, where the hand closed; whether its grip was closed
        /// the last frame.
        closed: [Option<(usize, Vec3)>; 2],
        gripped: [bool; 2],
        next_tick: [f64; 2],
        /// A plant plucked, by which hand: its activation in the next game frame.
        plucked: Option<(usize, usize)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    /// The bounds of a 3D's meshes (the protocol's world, metres): a plant's leaves and heads, not
    /// the ball round all of it.
    fn mesh_bounds(obj: usize, depth: u32, out: &mut Vec<(Vec3, f32)>) {
        if obj == 0 || depth > 6 || out.len() >= 32 {
            return;
        }
        if engine::is_node(obj) {
            for c in nodes::children(obj) {
                mesh_bounds(c, depth + 1, out);
            }
        } else if engine::is_a(obj, "NiGeometry")
            && let Some(b) = mem::at::<[f32; 4]>(obj + NODE_BOUND).filter(|b| b.iter().all(|v| v.is_finite()) && b[3] > 0.0)
        {
            out.push((camera::basis().point_from_game(Vec3::new(b[0], b[1], b[2])), b[3] / camera::UNITS_PER_METER));
        }
    }

    fn scan(s: &mut State) {
        let b = camera::basis();
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { return };
        let mut found = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let base = if r != 0 { mem::deref(r + REFR_BASE) } else { 0 };
                if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(FLORA) || mem::at::<u32>(r + FORM_FLAGS).is_none_or(|f| f & DISABLED != 0) {
                    continue;
                }
                let node = mem::deref(r + REFR_NODE);
                let Some(at) = mem::at::<[f32; 3]>(r + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { continue };
                if node == 0 || at.distance(me) > NEAR {
                    continue;
                }
                let mut bounds = Vec::new();
                mesh_bounds(node, 0, &mut bounds);
                if !bounds.is_empty() {
                    found.push(Plant { refr: r, bounds });
                }
            }
        }
        if found.len() != s.plants.len() {
            tracing::info!("plants: {} near", found.len());
        }
        s.plants = found;
    }

    /// Before the eyes are drawn: a hand closing in a plant, and pulled away: plucked.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        if crate::menus::menu_mode() {
            with_state(|s| s.closed = [None; 2]);
            return;
        }
        let body = crate::link::with_adapter(|a| a.read_body()).flatten();
        with_state(|s| {
            if now >= s.next_scan {
                s.next_scan = now + 0.5;
                scan(s);
            }
            for i in 0..2 {
                let hand = view.point(Vec3::from_array(f.hands[i].grip.pos));
                let squeeze = f.hands[i].squeeze;
                let holding = body.is_some_and(|b| b.hands[i].flags & HAND_HOLDING != 0);
                let was = s.gripped[i];
                s.gripped[i] = if was { squeeze > OPEN } else { squeeze >= CLOSE };
                // Closing on a plant (with nothing in the hand: a thing lying in it is the hand's
                // instead); opened, or the hand has taken something after all, it's let go.
                if !was && s.gripped[i] && !holding {
                    if let Some(p) = s.plants.iter().find(|p| in_plant(&p.bounds, hand)) {
                        tracing::info!("plants: the {} hand closed on {:08X}", if i == 0 { "left" } else { "right" }, mem::at::<u32>(p.refr + FORM_ID).unwrap_or(0));
                        s.closed[i] = Some((p.refr, hand));
                        crate::link::with_adapter(|a| a.haptic(i as u32, 0.04, 90.0, 0.2));
                    }
                }
                if !s.gripped[i] || holding {
                    s.closed[i] = None;
                }
                let Some((refr, at)) = s.closed[i] else { continue };
                let pulled = hand.distance(at);
                if pulled >= PULL {
                    tracing::info!("plants: {:08X} plucked", mem::at::<u32>(refr + FORM_ID).unwrap_or(0));
                    s.closed[i] = None;
                    s.plucked = Some((refr, i));
                    crate::link::with_adapter(|a| a.haptic(i as u32, 0.06, 160.0, 0.7));
                } else if now >= s.next_tick[i] {
                    // (The stem's tension, growing as it's pulled.)
                    s.next_tick[i] = now + 0.06;
                    let k = pulled / PULL;
                    crate::link::with_adapter(|a| a.haptic(i as u32, 0.03, 60.0 + 80.0 * k, 0.1 + 0.35 * k));
                }
            }
        });
    }

    /// Each game frame: a plant plucked is the player's activation of it (the game's harvest); what
    /// it gave into the hand that plucked it.
    pub fn frame(now: f64) {
        let Some((refr, hand)) = with_state(|s| s.plucked.take()) else { return };
        let base = mem::deref(refr + REFR_BASE);
        if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(FLORA) {
            return;
        }
        let ingredient = mem::deref(base.wrapping_add_signed(INGREDIENT));
        let before = if ingredient != 0 { crate::inventory::count(ingredient) } else { 0 };
        let ok = crate::world::activate(refr);
        let after = if ingredient != 0 { crate::inventory::count(ingredient) } else { 0 };
        tracing::info!("plants: the harvest ({ok}): {before} -> {after} of {:08X}", mem::at::<u32>(ingredient + FORM_ID).unwrap_or(0));
        if after > before {
            crate::inventory::give(ingredient, hand, now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hand_in_a_plant() {
        let bounds = [(Vec3::new(0.0, 0.3, 0.0), 0.1), (Vec3::new(0.0, 0.1, 0.0), 0.05)];
        assert!(in_plant(&bounds, Vec3::new(0.05, 0.32, 0.0)), "in its head");
        assert!(in_plant(&bounds, Vec3::new(0.0, 0.1, 0.06)), "just outside its stem");
        assert!(!in_plant(&bounds, Vec3::new(0.3, 0.3, 0.0)), "beside it");
    }
}
