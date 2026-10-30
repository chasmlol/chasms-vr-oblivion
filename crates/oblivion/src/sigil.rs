//! The Sigil Stone taken from its flame by hand (`docs/research/interact.md`, "The big magic
//! moments"). At the top of an Oblivion gate's Sigil Keep the stone hangs in its column of fire: an
//! activator whose script, on the player's activation, plays its `Unequip` art, gives the player a
//! sigil stone, and sets off the gate's collapse (`SigilStoneRandomSCRIPT`: its parent activated).
//!
//! The stone is offered to the hands as one of the game's own props on the game's constraints
//! ([`OBJF_CONSTRAINED`]): solid to the hands, taken in a hand where it is (the fingers on it), and
//! held where the game has it. It comes out of the flame after the hand, dragged by the flame
//! ([`DRAG`]), the game's own 3D of it moved with it; pulled [`PULL`] out of its place, it's the
//! player's own activation of it, and its script does the rest (the art, the stone in the pack, the
//! gate's collapse) with the stone still in the hand until the game takes it. Let go before that, it
//! drifts back into its flame.

/// How far (metres) the stone is pulled out of its place to be taken.
pub const PULL: f32 = 0.3;
/// How long (seconds) the flame drags on the stone: it comes after the hand this much behind.
pub const DRAG: f32 = 0.1;
/// How long a stone let go takes to drift back into its flame (seconds).
pub const BACK: f32 = 0.6;

/// Where the stone is after `dt` seconds, coming towards `want` from `at`, `lag` seconds behind.
pub fn follow(at: glam::Vec3, want: glam::Vec3, lag: f32, dt: f32) -> glam::Vec3 {
    at + (want - at) * (dt / lag.max(1e-3)).min(1.0)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::av;
    use crate::{camera, engine, mem};
    use glam::{Quat, Vec3};
    use rustvr_protocol::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Forms: type (+4), flags (+8: deleted, disabled), id (+0xC); a reference's base (+0x1C), 3D
    /// (+0x3C); an activator's model (`TESModel` +0x30, its path +4).
    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DELETED: u32 = 0x20;
    const DISABLED: u32 = 0x800;
    const ACTIVATOR: u8 = 0x12;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const ACTI_MODEL: usize = 0x34;
    /// `NiAVObject`: its world bound (+0x20: centre, radius); `Update(time, flags)` (thiscall).
    const NODE_BOUND: usize = 0x20;
    const UPDATE_NODE: usize = 0x0070_7370;
    /// The stone's models (`Oblivion\Sigil\SigilStone01.NIF`, the Great Sigil Stone's
    /// `SigilStoneGREAT01.NIF`): the activators that are sigil stones.
    const STONE_MODEL: &str = "sigil\\sigilstone";
    /// Metres round the player the stones are offered within.
    const REACH: f32 = 4.0;
    /// The stones' object ids: the reference's, in a range of their own (`0xFB`).
    const ID_BASE: u32 = 0xFB00_0000;
    /// Seconds the stone stays in the hand once taken, waiting for the game to take it.
    const TAKEN_FOR: f64 = 6.0;

    struct Stone {
        refr: usize,
        node: usize,
        desc: ObjectDesc,
        /// Its 3D's local translation at rest (game units), and where it is in the protocol's
        /// world now and at rest.
        rest_local: [f32; 3],
        rest: Vec3,
        rot: Quat,
        at: Vec3,
        /// When the game's activation was made (taken).
        taken: Option<f64>,
        pending: bool,
    }

    #[derive(Default)]
    struct State {
        stones: HashMap<u32, Stone>,
        next_scan: f64,
        sent: Vec<u32>,
        states: Option<Box<ObjectStates>>,
        last: f64,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    /// A sigil stone's activator reference, there and drawn.
    fn is_stone(refr: usize) -> bool {
        let base = mem::deref(refr + REFR_BASE);
        if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(ACTIVATOR) || mem::at::<u32>(refr + FORM_FLAGS).is_none_or(|f| f & (DELETED | DISABLED) != 0) || mem::deref(refr + REFR_NODE) == 0 {
            return false;
        }
        crate::combat::read_str(mem::deref(base + ACTI_MODEL)).to_ascii_lowercase().contains(STONE_MODEL)
    }

    fn scan(s: &mut State) {
        let b = camera::basis();
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { return };
        let mut keep = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let Some(p) = (r != 0).then(|| mem::at::<[f32; 3]>(r + engine::refr::POS)).flatten() else { continue };
                if b.point_from_game(Vec3::from(p)).distance(me) > REACH || !is_stone(r) {
                    continue;
                }
                let id = ID_BASE | (mem::at::<u32>(r + FORM_ID).unwrap_or(0) & 0x00FF_FFFF);
                keep.push(id);
                if s.stones.contains_key(&id) {
                    continue;
                }
                if let Some(stone) = make(r, id) {
                    tracing::info!("sigil: the stone {:08X} offered to the hands at {:.2?}", mem::at::<u32>(r + FORM_ID).unwrap_or(0), stone.rest);
                    s.stones.insert(id, stone);
                }
            }
        }
        s.stones.retain(|id, st| keep.contains(id) || st.taken.is_some());
    }

    /// The stone as the runtime's prop: its 3D's pose, a ball the size of its stone (its bound's
    /// core: the bound takes in the corona round it).
    fn make(refr: usize, id: u32) -> Option<Stone> {
        let node = mem::deref(refr + REFR_NODE);
        let rest_local: [f32; 3] = mem::at(node + av::LOCAL_TRANSLATE)?;
        let t: [f32; 3] = mem::at(node + av::WORLD_TRANSLATE)?;
        let r: [f32; 9] = mem::at(node + av::WORLD_ROTATE)?;
        let bound: [f32; 4] = mem::at(node + NODE_BOUND)?;
        let b = camera::basis();
        let pos = b.point_from_game(Vec3::from(t));
        let rot = b.rot_from_game(crate::items::ni_matrix(r));
        let centre = rot.inverse() * (b.point_from_game(Vec3::new(bound[0], bound[1], bound[2])) - pos);
        let radius = (bound[3] / camera::UNITS_PER_METER * 0.3).clamp(0.08, 0.2);
        let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
        d.id = id;
        d.kind = OBJ_PROP;
        d.flags = OBJF_GRABBABLE | OBJF_GAME_SIMULATED | OBJF_CONSTRAINED;
        d.spawn_seq = 1;
        d.spawn_pose = Pose { pos: pos.to_array(), valid: 1, rot: rot.to_array() };
        d.shape = SHAPE_SPHERE;
        d.radius = radius;
        d.half_extents = [radius; 3];
        d.center = centre.to_array();
        d.mass = 1.0;
        set_name(&mut d.name, "sigil stone");
        Some(Stone { refr, node, desc: d, rest_local, rest: pos, rot, at: pos, taken: None, pending: false })
    }

    /// The stone's 3D where it is now (the game draws it there).
    fn put(st: &Stone) {
        let b = camera::basis();
        let local = Vec3::from(st.rest_local) + b.point_to_game(st.at) - b.point_to_game(st.rest);
        type Update = unsafe extern "thiscall" fn(usize, f32, u32) -> u32;
        if mem::is_readable(st.node as *const u8, 0xB8) {
            unsafe {
                ((st.node + av::LOCAL_TRANSLATE) as *mut [f32; 3]).write_unaligned(local.to_array());
                std::mem::transmute::<usize, Update>(UPDATE_NODE)(st.node, 0.0, 0);
            }
        }
    }

    fn publish(s: &mut State) {
        let mut ids: Vec<u32> = s.stones.keys().copied().collect();
        ids.sort();
        if ids == s.sent {
            return;
        }
        let descs: Vec<ObjectDesc> = ids.iter().map(|id| s.stones[id].desc).collect();
        if crate::link::with_adapter(|a| crate::objects::publish(crate::objects::Owner::Magic, &descs, &[], a)).unwrap_or(false) {
            s.sent = ids;
        }
    }

    /// Before the eyes are drawn: each stone where the hand has it (or drifting back), the game's
    /// 3D with it; pulled out far enough, the game's activation asked for.
    pub fn place(view: &rustvr_adapter::View, now: f64) {
        with_state(|s| {
            if !crate::menus::in_world() {
                s.stones.clear();
            }
            if now >= s.next_scan {
                s.next_scan = now + 0.5;
                scan(s);
                publish(s);
            }
            let dt = ((now - s.last) as f32).clamp(0.0, 0.1);
            s.last = now;
            if s.stones.is_empty() {
                return;
            }
            let states = s.states.get_or_insert_with(bytemuck::zeroed_box);
            let read = crate::link::with_adapter(|a| a.read_object_states(states)).unwrap_or(false);
            let n = if read { (states.count as usize).min(MAX_OBJECTS) } else { 0 };
            let mut gone = Vec::new();
            for (id, st) in s.stones.iter_mut() {
                let held = states.states[..n].iter().find(|o| o.id == *id && o.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0);
                match held {
                    Some(o) => {
                        let (pos, _) = if o.flags & OS_TRACKING != 0 { view.pose(&o.pose) } else { (Vec3::from_array(o.pose.pos), Quat::from_array(o.pose.rot)) };
                        if pos.is_finite() {
                            st.at = follow(st.at, pos, DRAG, dt);
                        }
                        let hand = if o.flags & OS_HELD_LEFT != 0 { 0 } else { 1 };
                        if st.taken.is_none() && !st.pending && st.at.distance(st.rest) > PULL {
                            st.pending = true;
                            tracing::info!("sigil: the stone pulled from its flame");
                            crate::link::with_adapter(|a| a.haptic(hand, 0.3, 50.0, 1.0));
                        }
                    }
                    None if st.taken.is_none() => st.at = follow(st.at, st.rest, BACK / 3.0, dt),
                    None => {}
                }
                put(st);
                let culled = mem::at::<u16>(st.node + av::FLAGS).is_some_and(|f| f & av::APP_CULLED != 0);
                if st.taken.is_some_and(|t| now - t > TAKEN_FOR) || mem::deref(st.refr + REFR_NODE) != st.node || culled {
                    gone.push(*id);
                }
            }
            for id in gone {
                s.stones.remove(&id);
            }
            publish(s);
            let poses: Vec<GamePose> = s.stones.values().map(|st| GamePose { id: st.desc.id, pose: Pose { pos: st.at.to_array(), valid: 1, rot: st.rot.to_array() }, ..Default::default() }).collect();
            crate::link::with_adapter(|a| crate::objects::publish_poses(crate::objects::Owner::Magic, &poses, a));
        });
    }

    /// Each game frame: a stone pulled out is the player's activation of it (its script: the art,
    /// the stone in the pack, the gate's collapse).
    pub fn frame(now: f64) {
        let pending: Vec<usize> = with_state(|s| {
            s.stones
                .values_mut()
                .filter(|st| st.pending)
                .map(|st| {
                    st.pending = false;
                    st.taken = Some(now);
                    st.refr
                })
                .collect()
        });
        for refr in pending {
            let ok = crate::world::activate(refr);
            tracing::info!("sigil: the stone taken: the game's activation ({ok})");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn the_stone_comes_after_the_hand() {
        let at = follow(Vec3::ZERO, Vec3::X, DRAG, DRAG / 2.0);
        assert!((at.x - 0.5).abs() < 1e-5);
        assert_eq!(follow(Vec3::ZERO, Vec3::X, DRAG, 1.0), Vec3::X, "never past the hand");
    }
}
