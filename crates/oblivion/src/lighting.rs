//! The game's light on the player, for what the runtime draws (the body, what it holds): the sun
//! the game lights its world with, and its ambient (`rustvr_protocol::LightHint`).
//!
//! `Sky::GetSingleton` (0x542EA0, cdecl) +0x28 is the sun; its +0x1C is the directional light the
//! scene is lit with (`NiDirectionalLight`, xOBSE's `NiObjects.h`: dimmer +0xDC, ambient +0xE0,
//! diffuse +0xEC, the direction its light travels +0x108).

use crate::{camera, mem};
use glam::Vec3;
use rustvr_protocol::LightHint;

const SKY: usize = 0x0054_2EA0;
const SKY_SUN: usize = 0x28;
const SUN_LIGHT: usize = 0x1C;
const DIMMER: usize = 0xDC;
const AMBIENT: usize = 0xE0;
const DIFFUSE: usize = 0xEC;
const DIRECTION: usize = 0x108;

/// The sun's light now (the default hint when there's no sky: inside, or loading).
#[cfg(target_arch = "x86")]
pub fn hint() -> LightHint {
    let sky = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn() -> usize>(SKY)() };
    let light = mem::deref(mem::deref(sky + SKY_SUN) + SUN_LIGHT);
    let (Some(dimmer), Some(amb), Some(diff), Some(dir)) = (
        mem::at::<f32>(light + DIMMER),
        mem::at::<[f32; 3]>(light + AMBIENT),
        mem::at::<[f32; 3]>(light + DIFFUSE),
        mem::at::<[f32; 3]>(light + DIRECTION),
    ) else {
        return LightHint::default();
    };
    // Towards the sun: against the way its light travels.
    let to_sun = camera::basis().dir_from_game(-Vec3::from(dir)).normalize_or(Vec3::Y);
    let h = LightHint { sun_dir: to_sun.to_array(), sun_color: diff.map(|c| c * dimmer), ambient: amb, ..Default::default() };
    static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 3000 == 100 {
        tracing::info!("lighting: the sun towards {:?}, colour {:?} (dimmer {dimmer}), ambient {:?}", h.sun_dir, h.sun_color, h.ambient);
    }
    h
}

#[cfg(not(target_arch = "x86"))]
pub fn hint() -> LightHint {
    LightHint::default()
}
