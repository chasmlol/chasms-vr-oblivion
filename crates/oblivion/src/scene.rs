//! The game's own models, put in its world where the player's hands have them: loaded through the
//! game's model cache, hung under `MagicProjectileRoot` (which the game updates every frame with
//! its time and controllers, as it does its spells in flight, and which the world render draws: lit,
//! fogged, in its depth, in both eyes), and placed each frame before the eyes are drawn. Their
//! lights are the game's own too: a light's form makes its point light and flickers it as the game
//! does for a torch, and a spell effect's casting light is the game's `MagicLight`.
//!
//! The functions (docs/research/torches.md):
//! * 0x69FBF0 (cdecl, a model's path): a fresh copy of the model's root from the game's cache. Its
//!   collision object (+0xA8), if it has one, sets the node's world transform from its Havok body
//!   whenever the node updates (`NiNode::UpdateWorldData` 0x70C120 calls it last): a copy that the
//!   hand moves has none.
//! * `NiNode` virtual +0x84 `AttachChild(child, 1)` and +0x88 `DetachChild(&out, child)`.
//! * 0x560920 (cdecl: the `NiControllerManager` type, a controller): the model's controller
//!   manager; 0x55E000 (thiscall on its sequence map +0x58: a name, &sequence) a sequence by name;
//!   0x4715C0 (thiscall on the manager, 0.0) and `NiControllerSequence::Activate` 0x6C9BA0
//!   (thiscall on the sequence: 0, 0, 1.0, 0.0, 0, 0) start it, as the game starts a spell's
//!   casting art (0x5EDCE0). A sequence's length is at +0x30.
//! * `TESObjectLIGH` 0x4B1600 (thiscall on the form: a reference, the 3D): the form's point light
//!   under the 3D's `AttachLight`, in the world's `ShadowSceneNode`; its flicker 0x4B22E0 (thiscall
//!   on the form: {the light, a scale}, 0) every frame; out of the shadow scene again 0x7C7DC0
//!   (thiscall on the scene, from 0x7B4280).
//! * `MagicLight` (28 bytes): 0x69E250 (thiscall: the LIGH's form id, the node it hangs under),
//!   0x69DD50 (fade in over seconds), 0x69DDA0 (every frame, the time passed), 0x69E0B0 (gone).
//! * 0x401F00 and 0x401F20: the game's own allocation and its free.

use crate::engine::av;
use crate::{camera, mem, nodes};
use glam::{Mat3, Quat, Vec3};

/// `MagicProjectileRoot`.
const PROJECTILE_ROOT: usize = 0x00B3_33A4;
const LOAD_MODEL: usize = 0x0069_FBF0;
const VT_ATTACH: usize = 0x84;
/// `NiAVObject`'s collision object.
const NODE_COLLISION: usize = 0xA8;
const VT_DETACH: usize = 0x88;
const AS_MANAGER: usize = 0x0056_0920;
const MANAGER_TYPE: usize = 0x00B3_CAC0;
const SEQUENCE_BY_NAME: usize = 0x0055_E000;
const MANAGER_SEQUENCES: usize = 0x58;
const MANAGER_RESET: usize = 0x0047_15C0;
const ACTIVATE: usize = 0x006C_9BA0;
/// `NiTimeController` flags (the game sets 8 on a casting art's manager as it starts it).
const CONTROLLER_FLAGS: usize = 0x08;
const SEQUENCE_LENGTH: usize = 0x30;
const SEQUENCE_OFFSET: usize = 0x48;
const LIGHT_MAKE: usize = 0x004B_1600;
const LIGHT_FLICKER: usize = 0x004B_22E0;
const SHADOW_SCENE: usize = 0x007B_4280;
const SHADOW_SCENE_REMOVE: usize = 0x007C_7DC0;
const MAGIC_LIGHT_NEW: usize = 0x0069_E250;
const MAGIC_LIGHT_FADE: usize = 0x0069_DD50;
const MAGIC_LIGHT_UPDATE: usize = 0x0069_DDA0;
const MAGIC_LIGHT_GONE: usize = 0x0069_E0B0;
const MAGIC_LIGHT_SIZE: usize = 28;
const ALLOC: usize = 0x0040_1F00;
const FREE: usize = 0x0040_1F20;
/// `NiAVObject::Update(time, flags)` (thiscall).
const UPDATE_NODE: usize = 0x0070_7370;
pub const LOCAL_SCALE: usize = 0x60;
/// The frame's time, seconds.
pub const FRAME_TIME: usize = 0x00B3_3E9C;

/// The root the game's spells in flight hang under, or 0.
pub fn projectile_root() -> usize {
    mem::deref(PROJECTILE_ROOT)
}

/// A copy of a model (a path under `meshes\`), hung under `MagicProjectileRoot`, or 0.
#[cfg(target_arch = "x86")]
pub fn load(path: &str) -> usize {
    let root = projectile_root();
    let Ok(c) = std::ffi::CString::new(path) else { return 0 };
    if root == 0 || path.is_empty() {
        return 0;
    }
    type Load = unsafe extern "C" fn(*const u8) -> usize;
    let node = unsafe { std::mem::transmute::<usize, Load>(LOAD_MODEL)(c.as_ptr() as *const u8) };
    if node == 0 {
        return 0;
    }
    // A model with a Havok body (a torch's) has its world transform set from the body's, which
    // isn't in the game's physics and never moves: this copy goes where it's put instead.
    let collision = mem::deref(node + NODE_COLLISION);
    if collision != 0 {
        unsafe { ((node + NODE_COLLISION) as *mut u32).write(0) };
        release(collision);
    }
    attach(root, node);
    node
}

#[cfg(target_arch = "x86")]
fn attach(parent: usize, child: usize) {
    type Attach = unsafe extern "thiscall" fn(usize, usize, u8);
    let f = mem::deref(mem::deref(parent) + VT_ATTACH);
    unsafe { std::mem::transmute::<usize, Attach>(f)(parent, child, 1) };
}

/// Takes a model put up by [`load`] down again (the root's hold on it let go: the game frees it).
#[cfg(target_arch = "x86")]
pub fn unload(node: usize) {
    let parent = mem::deref(node + nodes::PARENT);
    if parent == 0 {
        return;
    }
    type Detach = unsafe extern "thiscall" fn(usize, *mut usize, usize) -> usize;
    let f = mem::deref(mem::deref(parent) + VT_DETACH);
    let mut out = 0usize;
    unsafe { std::mem::transmute::<usize, Detach>(f)(parent, &mut out, node) };
    release(out);
}

/// Lets go of a hold on one of the game's `NiRefObject`s (its count at +4; at 0 it deletes itself).
#[cfg(target_arch = "x86")]
fn release(obj: usize) {
    if obj == 0 {
        return;
    }
    let count = unsafe { &*((obj + 4) as *const std::sync::atomic::AtomicI32) };
    if count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel) == 1 {
        type Delete = unsafe extern "thiscall" fn(usize, u32);
        let f = mem::deref(mem::deref(obj));
        unsafe { std::mem::transmute::<usize, Delete>(f)(obj, 1) };
    }
}

/// Puts a model's root at a pose in the runtime's world, `scale` times its size, and updates its
/// world transforms (not its controllers: the game runs those with its own time).
#[cfg(target_arch = "x86")]
pub fn place(node: usize, pos: Vec3, rot: Quat, scale: f32) -> bool {
    if !mem::is_readable(node as *const u8, 0xB8) || !pos.is_finite() || !rot.is_finite() {
        return false;
    }
    // (Under `MagicProjectileRoot`, at the origin unrotated: its local transform is the world's.)
    let b = camera::basis();
    let r = b.rot_to_game(rot.normalize());
    let t = b.point_to_game(pos);
    type Update = unsafe extern "thiscall" fn(usize, f32, u32) -> u32;
    unsafe {
        ((node + av::LOCAL_ROTATE) as *mut [f32; 9]).write_unaligned(crate::items::ni_rows(r));
        ((node + av::LOCAL_TRANSLATE) as *mut [f32; 3]).write_unaligned(t.to_array());
        ((node + LOCAL_SCALE) as *mut f32).write_unaligned(scale);
        std::mem::transmute::<usize, Update>(UPDATE_NODE)(node, 0.0, 0);
    }
    true
}

/// Starts one of a model's controller sequences by name (`SpecialIdle_Cast`): its length
/// (seconds), or None if it has none.
#[cfg(target_arch = "x86")]
pub fn play(node: usize, sequence: &str) -> Option<f32> {
    type AsManager = unsafe extern "C" fn(usize, usize) -> usize;
    type ByName = unsafe extern "thiscall" fn(usize, *const u8, *mut usize) -> u8;
    type Reset = unsafe extern "thiscall" fn(usize, f32);
    type Activate = unsafe extern "thiscall" fn(usize, i32, u8, f32, f32, usize, i32) -> u8;
    let ctl = mem::deref(node + nodes::CONTROLLER);
    if ctl == 0 {
        return None;
    }
    let manager = unsafe { std::mem::transmute::<usize, AsManager>(AS_MANAGER)(MANAGER_TYPE, ctl) };
    let name = std::ffi::CString::new(sequence).ok()?;
    let mut seq = 0usize;
    if manager == 0 || unsafe { std::mem::transmute::<usize, ByName>(SEQUENCE_BY_NAME)(manager + MANAGER_SEQUENCES, name.as_ptr() as *const u8, &mut seq) } == 0 || seq == 0 {
        return None;
    }
    unsafe {
        std::mem::transmute::<usize, Reset>(MANAGER_RESET)(manager, 0.0);
        std::mem::transmute::<usize, Activate>(ACTIVATE)(seq, 0, 0, 1.0, 0.0, 0, 0);
        let flags = (manager + CONTROLLER_FLAGS) as *mut u16;
        flags.write_unaligned(flags.read_unaligned() | 8);
        ((seq + SEQUENCE_OFFSET) as *mut f32).write_unaligned(-f32::MAX);
    }
    mem::at::<f32>(seq + SEQUENCE_LENGTH)
}

/// A light form's own light on one of the game's models: made as the game makes a torch's, and
/// flickered every frame as the game flickers it.
pub struct FormLight {
    form: usize,
    light: usize,
    /// What the flicker is given: the light and its scale.
    state: Box<[u32; 2]>,
}

#[cfg(target_arch = "x86")]
impl FormLight {
    /// The light of the `TESObjectLIGH` `form` on `node` (the reference that carries it: `refr`).
    pub fn new(form: usize, refr: usize, node: usize) -> Option<FormLight> {
        type Make = unsafe extern "thiscall" fn(usize, usize, usize);
        if form == 0 || refr == 0 || node == 0 {
            return None;
        }
        unsafe { std::mem::transmute::<usize, Make>(LIGHT_MAKE)(form, refr, node) };
        // (It's hung under the model's `AttachLight`, or its root.)
        let at = nodes::find(node, "AttachLight").unwrap_or(node);
        let light = nodes::children(at).into_iter().find(|&c| nodes::class(c).contains("Light"))?;
        Some(FormLight { form, light, state: Box::new([light as u32, 1f32.to_bits()]) })
    }

    pub fn light(&self) -> usize {
        self.light
    }

    /// This frame's flicker (the game's own: it moves and dims the light by the frame's time).
    pub fn flicker(&mut self) {
        type Flicker = unsafe extern "thiscall" fn(usize, *mut [u32; 2], u32);
        self.state[0] = self.light as u32;
        unsafe { std::mem::transmute::<usize, Flicker>(LIGHT_FLICKER)(self.form, &mut *self.state, 0) };
    }

    /// Out of the world's lights (the model it's on takes it down with it).
    pub fn remove(self) {
        type Scene = unsafe extern "C" fn(u32) -> usize;
        type Remove = unsafe extern "thiscall" fn(usize, usize);
        let scene = unsafe { std::mem::transmute::<usize, Scene>(SHADOW_SCENE)(0) };
        if scene != 0 {
            unsafe { std::mem::transmute::<usize, Remove>(SHADOW_SCENE_REMOVE)(scene, self.light) };
        }
    }
}

/// A spell effect's casting light (the game's `MagicLight`: the effect's light form, fading in),
/// hung under one of the game's models.
pub struct CastingLight(usize);

#[cfg(target_arch = "x86")]
impl CastingLight {
    /// `light_id`: the light form's id; `fade`: seconds it fades in over.
    pub fn new(light_id: u32, node: usize, fade: f32) -> Option<CastingLight> {
        type Alloc = unsafe extern "C" fn(usize) -> usize;
        type New = unsafe extern "thiscall" fn(usize, u32, usize) -> usize;
        type Fade = unsafe extern "thiscall" fn(usize, u8, f32) -> u8;
        if light_id == 0 || node == 0 {
            return None;
        }
        let obj = unsafe { std::mem::transmute::<usize, Alloc>(ALLOC)(MAGIC_LIGHT_SIZE) };
        if obj == 0 {
            return None;
        }
        unsafe {
            std::mem::transmute::<usize, New>(MAGIC_LIGHT_NEW)(obj, light_id, node);
            std::mem::transmute::<usize, Fade>(MAGIC_LIGHT_FADE)(obj, 1, fade.max(0.01));
        }
        Some(CastingLight(obj))
    }

    /// This frame: its fade and flicker, `dt` seconds on.
    pub fn update(&self, dt: f32) {
        type Update = unsafe extern "thiscall" fn(usize, f32) -> i32;
        unsafe { std::mem::transmute::<usize, Update>(MAGIC_LIGHT_UPDATE)(self.0, dt) };
    }

    /// The light itself (`NiPointLight`).
    pub fn light(&self) -> usize {
        mem::deref(self.0 + 8)
    }

    pub fn remove(self) {
        type Gone = unsafe extern "thiscall" fn(usize) -> i32;
        type Free = unsafe extern "C" fn(usize);
        unsafe {
            std::mem::transmute::<usize, Gone>(MAGIC_LIGHT_GONE)(self.0);
            std::mem::transmute::<usize, Free>(FREE)(self.0);
        }
    }
}

/// The game's frame time now (seconds).
pub fn frame_time() -> f32 {
    mem::at::<f32>(FRAME_TIME).filter(|t| t.is_finite() && *t >= 0.0).unwrap_or(0.0)
}

/// A rotation whose +Z is `forward` (the protocol's space), +Y as near `up` as it can be.
pub fn facing(forward: Vec3, up: Vec3) -> Quat {
    let z = forward.normalize_or(Vec3::Z);
    let x = up.cross(z).normalize_or(z.any_orthonormal_vector());
    Quat::from_mat3(&Mat3::from_cols(x, z.cross(x), z)).normalize()
}
