//! Oblivion's own functions and globals (Steam 1.2.0.416, the build OBSE also targets). Addresses are
//! absolute: the exe has no relocations and always loads at 0x400000. Each is checked by its first
//! bytes before any hook goes in, so another build turns VR off instead of crashing.

use crate::mem;

/// `Main::Render` (thiscall, one argument): the world render. The world camera's picture, the
/// first-person node's, then HDR or bloom into the back buffer. Menus are drawn after it returns.
pub const RENDER: usize = 0x0040_C830;
/// The game's frame (thiscall on `OSGlobals`): input, scripts, the world's update, then the render.
pub const FRAME: usize = 0x0040_D800;
/// `NiDX9Renderer*`; its Direct3D 9 device at +0x280.
pub const RENDERER: usize = 0x00B3_F928;
pub const RENDERER_DEVICE: usize = 0x280;
/// The world's `SceneGraph*`; its camera (`NiCamera*`) at +0xDC.
pub const SCENE_GRAPH: usize = 0x00B3_33CC;
pub const SCENE_GRAPH_CAMERA: usize = 0xDC;
/// `PlayerCharacter*`.
pub const PLAYER: usize = 0x00B3_33C4;
/// `OSGlobals*`: its window at +8.
pub const OS_GLOBALS: usize = 0x00B3_3398;

/// `NiAVObject` fields.
pub mod av {
    pub const FLAGS: usize = 0x18;
    pub const LOCAL_ROTATE: usize = 0x30;
    pub const LOCAL_TRANSLATE: usize = 0x54;
    pub const WORLD_ROTATE: usize = 0x64;
    pub const WORLD_TRANSLATE: usize = 0x88;
    pub const WORLD_SCALE: usize = 0x94;
    /// `kFlag_AppCulled`.
    pub const APP_CULLED: u16 = 1;
}

/// `NiCamera` fields: its `NiFrustum` (left, right, top, bottom, near, far, ortho) and viewport.
pub mod cam {
    pub const FRUSTUM: usize = 0xEC;
    pub const PORT: usize = 0x110;
}

/// `TESObjectREFR` fields: rotation (x, y, z radians) and position (game units).
pub mod refr {
    pub const ROT: usize = 0x20;
    pub const POS: usize = 0x2C;
}

/// `PlayerCharacter` fields and functions.
pub mod pc {
    /// `UInt8 isThirdPerson`.
    pub const THIRD_PERSON: usize = 0x588;
    /// `NiNode* firstPersonNiNode`: the first-person arms and what they hold.
    pub const FIRST_PERSON_NODE: usize = 0x5D0;
    /// `TogglePOV(this, firstPerson)` (thiscall).
    pub const TOGGLE_POV: usize = 0x0066_C580;
}

/// The first bytes of each function hooked, as in the 1.2.0.416 exe.
const RENDER_BYTES: [u8; 16] = [0x6a, 0xff, 0x68, 0x63, 0xa1, 0x9a, 0x00, 0x64, 0xa1, 0x00, 0x00, 0x00, 0x00, 0x50, 0x83, 0xec];

/// The game is the build these addresses are for, its code decrypted (the exe is wrapped by
/// Steam's DRM, which decrypts it at start).
pub fn supported() -> bool {
    mem::at::<[u8; 16]>(RENDER) == Some(RENDER_BYTES)
}

/// The world camera, or 0.
pub fn world_camera() -> usize {
    let sg = mem::deref(SCENE_GRAPH);
    if sg == 0 { 0 } else { mem::deref(sg + SCENE_GRAPH_CAMERA) }
}

/// The game's Direct3D 9 device (`IDirect3DDevice9*`), or 0.
pub fn device() -> usize {
    let r = mem::deref(RENDERER);
    if r == 0 { 0 } else { mem::deref(r + RENDERER_DEVICE) }
}

pub fn player() -> usize {
    mem::deref(PLAYER)
}

/// The game's window.
pub fn window() -> usize {
    let os = mem::deref(OS_GLOBALS);
    if os == 0 { 0 } else { mem::deref(os + 8) }
}
