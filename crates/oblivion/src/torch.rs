//! The player's torch, in the hand: the equipped torch is an object on the body (at the right hip,
//! [`TORCH_SLOT`]) that either hand draws, waves about, holsters and drops, and the game draws it
//! where the runtime has it, burning: its own model, its own flame and its own light, flickering as
//! the game flickers a torch (docs/research/torches.md, "The player's torch").
//!
//! The game's own equipped torch is the first-person viewmodel's (and never shows in VR), so the
//! torch the hand has is a copy of its model from the game's model cache, hung where the game
//! puts its effects in the world (`crate::scene`): its `FlameNode` given the game's flame (0x4E26F0),
//! its light made from the torch's form as the game makes a torch's (its radius, colour and
//! attenuation, in the world's lights), flickered every frame. So it lights the world as the game's
//! torches do, and the body and hands with it (`crate::lighting` reads the world's lights).
//!
//! It crackles with the game's own held-torch loop (`ITMTorchHeldLP`, the process's own: its
//! virtual 0xD2 plays it on the actor, 0xD1 stops its sound slot 1).
//!
//! Holstered, it still burns: a light at the hip with both hands free. Let go anywhere else, it
//! falls and lies burning where it lands (the runtime's physics on the game's collision); left
//! behind, it's back at the hip. Unequipped in the inventory, it's gone.
//!
//! Forms: `MiddleHighProcess::equippedLightData` (+0xE8 of the player's process) is the equipped
//! light's inventory entry (its form at +8, a `TESObjectLIGH`: type 0x1A, its model's path at
//! +0x34, its weight at +0x64, its flags at +0x7C).

use crate::{engine, mem, nodes};
use glam::{Mat3, Quat, Vec3};
use rustvr_protocol::*;

/// The torch's object id. (The adapter's own objects each have their own: the weapon and shield
/// 1 and 2, the arrows 4, the map 5.)
pub const TORCH_ID: u32 = 0x0B1E_0003;
/// Where the torch hangs on the body (a `SLOT_*`): the right hip, across from the weapon at the
/// left, for the off hand to draw (section 6's holsters will place it).
pub const TORCH_SLOT: u32 = SLOT_HIP_RIGHT;
/// Left further than this from the player (metres), it's back at the hip.
pub const LEFT_BEHIND: f32 = 15.0;

const PROCESS: usize = 0x58;
const EQUIPPED_LIGHT: usize = 0xE8;
const ENTRY_FORM: usize = 0x08;
const FORM_LIGHT: u8 = 0x1A;
const LIGHT_MODEL: usize = 0x34;
const LIGHT_WEIGHT: usize = 0x64;
const LIGHT_FLAGS: usize = 0x7C;
const LIGHT_CAN_CARRY: u32 = 2;
/// The game's flame on a model's `FlameNode`s (cdecl, the 3D).
const FLAMES: usize = 0x004E_26F0;
/// The process's held-torch loop: played (virtual 0xD2, thiscall: the actor), stopped (virtual
/// 0xD1: its sound slot, 1), the sound itself in that slot (+0x224).
const VT_TORCH_SOUND_PLAY: usize = 0xD2 * 4;
const VT_SOUND_STOP: usize = 0xD1 * 4;
const TORCH_SOUND_SLOT: u32 = 1;
const TORCH_SOUND: usize = 0x224;

/// The equipped torch, if the player has one: its form and model's path.
pub fn equipped() -> Option<(usize, String)> {
    let process = mem::deref(engine::player() + PROCESS);
    let form = mem::deref(mem::deref(process + EQUIPPED_LIGHT) + ENTRY_FORM);
    if process == 0 || form == 0 || mem::at::<u8>(form + 4) != Some(FORM_LIGHT) || mem::at::<u32>(form + LIGHT_FLAGS).is_none_or(|f| f & LIGHT_CAN_CARRY == 0) {
        return None;
    }
    let model = crate::combat::read_str(mem::deref(form + LIGHT_MODEL));
    (!model.is_empty()).then_some((form, model))
}

/// The torch as an object, measured on its model (the extracted copy, `local_assets/oblivion`):
/// its origin is where the game's hand holds it (the skeleton's `Torch` node), its length along
/// +Z, the handle from its foot up to below the flame (`flame`, model space).
pub fn object_desc(model: &rustvr_asset::Model, flame: Option<f32>, weight: f32, seq: u32) -> ObjectDesc {
    let lo = Vec3::from_array(model.bounds_min);
    let hi = Vec3::from_array(model.bounds_max);
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = TORCH_ID;
    d.kind = OBJ_PROP;
    d.flags = OBJF_GRABBABLE | OBJF_PULLABLE | OBJF_HOLSTERABLE | OBJF_UPRIGHT;
    d.spawn_seq = seq;
    d.shape = SHAPE_BOX;
    d.half_extents = ((hi - lo) * 0.5).max(Vec3::splat(0.01)).to_array();
    d.center = ((hi + lo) * 0.5).to_array();
    // (The game's weight is about pounds: a kilogram is two.)
    d.mass = (weight * 0.45).clamp(0.2, 3.0);
    d.grip = [0.0; 3];
    d.grip_rot = fist(Vec3::Z, Vec3::Y).to_array();
    // The handle: from the foot to a hand's width under the flame.
    let top = flame.unwrap_or(hi.z).min(hi.z) - 0.08;
    d.grip_line = [[0.0, 0.0, lo.z + 0.02], [0.0, 0.0, top.max(lo.z + 0.05)]];
    d.grip_radius = ((hi.x - lo.x).min(hi.y - lo.y) * 0.25).clamp(0.01, 0.03);
    d.muzzle = [0.0, 0.0, flame.unwrap_or(hi.z)];
    d.start_slot = 1 + TORCH_SLOT;
    set_name(&mut d.name, "torch");
    d
}

/// The controller grip rotation for a fist round a handle along `tube` (little finger to thumb),
/// the back of the hand towards `x` (as `crate::weapons` holds a weapon).
fn fist(tube: Vec3, x: Vec3) -> Quat {
    let z = tube.normalize();
    let x = (x - z * x.dot(z)).normalize();
    Quat::from_mat3(&Mat3::from_cols(x, z.cross(x), z)).normalize()
}

#[cfg(target_arch = "x86")]
pub use game::*;

#[cfg(not(target_arch = "x86"))]
pub fn publish(_a: &rustvr_adapter::Adapter) {}

#[cfg(target_arch = "x86")]
mod game {
    use super::*;
    use crate::{camera, scene};
    use std::sync::Mutex;

    /// The torch the hand has: the game's model and light, and its object.
    struct Held {
        form: usize,
        node: usize,
        light: Option<scene::FormLight>,
        desc: ObjectDesc,
        /// How it was held when last logged.
        logged: Option<u32>,
        /// When its crackle is looked at next.
        next_crackle: f64,
    }

    struct State {
        torch: Option<Held>,
        /// The form whose model couldn't be read (not asked again).
        missing: usize,
        seq: u32,
        states: Box<ObjectStates>,
        /// The game's own torch on its skeletons was seen (logged once).
        game_torch_seen: bool,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State { torch: None, missing: 0, seq: 0, states: bytemuck::zeroed_box(), game_torch_seen: false }))
    }

    /// The torch's weight (its form's `TESWeightForm`).
    fn weight(form: usize) -> f32 {
        mem::at::<f32>(form + LIGHT_WEIGHT).filter(|w| w.is_finite() && *w >= 0.0).unwrap_or(1.0)
    }

    /// Once a game frame (in the world): the equipped torch made into the hand's, or taken away.
    pub fn publish(a: &rustvr_adapter::Adapter) {
        with_state(|s| {
            let now = equipped();
            if s.torch.as_ref().map(|t| t.form) == now.as_ref().map(|n| n.0) {
                return;
            }
            if let Some(old) = s.torch.take() {
                tracing::info!("torch: put away (no longer equipped)");
                crackle(false);
                if let Some(l) = old.light {
                    l.remove();
                }
                scene::unload(old.node);
            }
            let mut descs = Vec::new();
            if let Some((form, path)) = now {
                if s.missing == form {
                    return;
                }
                let key = oblivion_assets::export::model_key(&path);
                let model = match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("torch: {path} has no extracted model ({key}: {e:#}); run oblivion-extract export meshes/lights");
                        s.missing = form;
                        return;
                    }
                };
                let node = scene::load(&path);
                if node == 0 {
                    tracing::warn!("torch: the game didn't load {path}");
                    s.missing = form;
                    return;
                }
                // The game's flame on its flame node, its light at its `AttachLight`.
                unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(usize) -> u8>(FLAMES)(node) };
                let light = scene::FormLight::new(form, engine::player(), node);
                // Where its flame is, in its own space (the runtime's axes, metres).
                let flame = nodes::find(node, "AttachLight").or_else(|| nodes::find(node, "FlameNode1")).and_then(|n| mem::at::<[f32; 3]>(n + crate::engine::av::LOCAL_TRANSLATE)).map(|t| camera::basis().point_from_game(Vec3::from(t)).z);
                s.seq = s.seq.wrapping_add(1).max(1);
                let desc = object_desc(&model, flame, weight(form), s.seq);
                tracing::info!(
                    "torch: {path} (model {key}) in the game at {node:#x}, its light {:?}, flame at {flame:?} m, handle {:?}, {:.2} kg, at the hip (slot {TORCH_SLOT})",
                    light.as_ref().map(|l| format!("{:#x} {} radius {:?}", l.light(), nodes::class(l.light()), mem::at::<f32>(l.light() + 0xF8))),
                    desc.grip_line,
                    desc.mass
                );
                descs.push(desc);
                crackle(true);
                s.torch = Some(Held { form, node, light, desc, logged: None, next_crackle: 0.0 });
            }
            crate::objects::publish(crate::objects::Owner::Torch, &descs, &[], a);
        });
    }

    /// Before the eyes are drawn: the game's torch where the runtime has it, its light flickering.
    pub fn place(view: &rustvr_adapter::View) {
        with_state(|s| {
            note_game_torch(s);
            let Some(t) = s.torch.as_mut() else { return };
            // (Once a second: the game lets its loop go when it thinks no torch is out.)
            let now = crate::link::now();
            if now >= t.next_crackle {
                t.next_crackle = now + 1.0;
                let had = mem::deref(mem::deref(engine::player() + PROCESS) + TORCH_SOUND) != 0;
                if !had {
                    tracing::info!("torch: its crackle (again)");
                }
                crackle(true);
            }
            if let Some(l) = t.light.as_mut() {
                l.flicker();
            }
            if !crate::link::with_adapter(|a| a.read_object_states(&mut s.states)).unwrap_or(false) {
                return;
            }
            let n = (s.states.count as usize).min(MAX_OBJECTS);
            let Some(st) = s.states.states[..n].iter().find(|st| st.id == TORCH_ID).copied() else {
                cull(t.node, true);
                return;
            };
            let (pos, rot) = if st.flags & OS_TRACKING != 0 { view.pose(&st.pose) } else { (Vec3::from_array(st.pose.pos), Quat::from_array(st.pose.rot)) };
            cull(t.node, false);
            scene::place(t.node, pos, rot, 1.0);
            scene::animate(t.node);
            // (Logged as it changes hands: held, holstered, let go.)
            let how = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT | OS_HOLSTERED | OS_PULLED);
            if t.logged != Some(how) {
                t.logged = Some(how);
                tracing::info!("torch: {} at {pos:.2?}, its light at {:?}", match how {
                    0 => "lying",
                    OS_HOLSTERED => "at the hip",
                    OS_PULLED => "flying to the hand",
                    _ => "in the hand",
                }, t.light.as_ref().and_then(|l| nodes::world_pos(l.light())));
            }
            // Left behind, lying somewhere: back to the hip.
            let free = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT | OS_HOLSTERED | OS_PULLED) == 0;
            let me = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)));
            if free && me.is_some_and(|m| m.distance(pos) > LEFT_BEHIND) {
                s.seq = s.seq.wrapping_add(1).max(1);
                t.desc.spawn_seq = s.seq;
                let desc = t.desc;
                tracing::info!("torch: left behind at {pos:.2?}: back at the hip");
                crate::link::with_adapter(|a| crate::objects::publish(crate::objects::Owner::Torch, &[desc], &[], a));
            }
        });
    }

    /// The game's held-torch loop on the player (`on`), or stopped; played again if the game let it
    /// go.
    fn crackle(on: bool) {
        let player = engine::player();
        let process = mem::deref(player + PROCESS);
        let vt = mem::deref(process);
        if process == 0 || vt == 0 {
            return;
        }
        if on {
            if mem::deref(process + TORCH_SOUND) != 0 {
                return;
            }
            type Play = unsafe extern "thiscall" fn(usize, usize) -> usize;
            unsafe { std::mem::transmute::<usize, Play>(mem::deref(vt + VT_TORCH_SOUND_PLAY))(process, player) };
        } else {
            type Stop = unsafe extern "thiscall" fn(usize, u32);
            unsafe { std::mem::transmute::<usize, Stop>(mem::deref(vt + VT_SOUND_STOP))(process, TORCH_SOUND_SLOT) };
        }
    }

    fn cull(node: usize, culled: bool) {
        let p = (node + crate::engine::av::FLAGS) as *mut u16;
        if let Some(f) = mem::at::<u16>(node + crate::engine::av::FLAGS) {
            let f = if culled { f | crate::engine::av::APP_CULLED } else { f & !crate::engine::av::APP_CULLED };
            unsafe { p.write_unaligned(f) };
        }
    }

    /// The game's own torch on the player's skeletons (it shouldn't show in VR): said once if it does,
    /// with its light.
    fn note_game_torch(s: &mut State) {
        if s.game_torch_seen {
            return;
        }
        let p = engine::player();
        for root in [mem::deref(p + 0x3C), crate::player::first_person_node()] {
            if let Some(t) = nodes::find(root, "Torch").filter(|&t| !nodes::children(t).is_empty()) {
                s.game_torch_seen = true;
                tracing::warn!("torch: the game's own torch is on the player's skeleton:{}", nodes::dump(t, 4));
            }
        }
    }
}

/// Testing: what the game has made of the player's torch and lights, to the log.
pub fn inspect() -> bool {
    let p = engine::player();
    if p == 0 {
        return false;
    }
    let third = mem::deref(p + 0x3C);
    let first = crate::player::first_person_node();
    for (what, root) in [("third", third), ("first", first)] {
        if let Some(t) = nodes::find(root, "Torch") {
            tracing::info!("inspect: {what} person's Torch node:{}", nodes::dump(t, 6));
        }
        if let Some(m) = nodes::find(root, "magicNode") {
            tracing::info!("inspect: {what} person's magicNode:{}", nodes::dump(m, 4));
        }
    }
    let root = mem::deref(0x00B3_33A4);
    tracing::info!("inspect: MagicProjectileRoot:{}", nodes::dump(root, 3));
    let me = mem::at::<[f32; 3]>(p + engine::refr::POS).map(Vec3::from).unwrap_or_default();
    for (light, _) in crate::lighting::scene_light_nodes() {
        let at = nodes::world_pos(light).unwrap_or_default();
        if at.distance(me) < 300.0 {
            tracing::info!("inspect: a light {:.0} units from the player: '{}' radius {:?}, diffuse {:?}, dimmer {:?}", at.distance(me), nodes::name(light), mem::at::<f32>(light + 0xF8), mem::at::<[f32; 3]>(light + 0xEC), mem::at::<f32>(light + 0xDC));
        }
    }
    true
}
