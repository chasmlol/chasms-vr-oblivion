//! The game's light on the player, for what the runtime draws (the body, what it holds), as
//! Oblivion's own shaders would light them (`rustvr_protocol::GameLighting` in its Gamebryo model,
//! docs/research/oblivion-lighting.md).
//!
//! The lights are the scene's: the world's `ShadowSceneNode` (made in 0x406950 and hung under the
//! "World" node, 0xB333CC) keeps the lights that are on (+0xE4, a list of `ShadowSceneLight`: its
//! `NiLight` +0x100, whether it's a point light +0xFC, its fade +0xD4). Each is given to the runtime as the game's shader setup (0x7ED6C0) gives it to
//! its shaders:
//!
//! * the ambient: the main (directional) light's ambient times its dimmer (at most 1), and with
//!   Night-Eye on, lifted to the game's Night-Eye ambient (0xB46924: brightened by 0.1 and scaled up
//!   to it);
//! * a light's colour: its diffuse times its dimmer (at most 1) and its fade;
//! * a point light's radius: its specular colour's red (Bethesda keeps it there), in game units.
//!
//! `NiLight` (xOBSE's `NiObjects.h`): dimmer +0xDC, ambient +0xE0, diffuse +0xEC, specular +0xF8;
//! `NiDirectionalLight`'s direction (the way its light travels) +0x108. The fog is the scene's fog
//! property (`TES` +0x1C): its colour +0x20, near +0x2C, far +0x30, linear between them.
//!
//! Positions and directions go in tracking space (the lighting's `eye` is the identity). [`hint`]
//! (the sun alone, `Sky::GetSingleton` 0x542EA0 +0x28, its light +0x1C) stays for the runtime's
//! own look.

use crate::{camera, engine, mem};
use glam::{Mat4, Vec3};
use rustvr_protocol::{GL_GB_DIRECTIONAL, GL_GB_POINT, GameLight, GameLighting, LM_GAMEBRYO, LightHint, Pose};

const SKY: usize = 0x0054_2EA0;
const SKY_SUN: usize = 0x28;
const SUN_LIGHT: usize = 0x1C;
const DIMMER: usize = 0xDC;
const AMBIENT: usize = 0xE0;
const DIFFUSE: usize = 0xEC;
const SPECULAR: usize = 0xF8;
const DIRECTION: usize = 0x108;

/// The `ShadowSceneNode` vtable; its lights (an `NiTPointerList`: first node +4, count +0xC; a
/// node: next +0, data +8). (0xB42F54 holds the last one made, which is the menus' own, for the
/// inventory's 3D view.)
const SHADOW_SCENE_VTABLE: usize = 0x00A9_04B4;
/// `NiNode`'s children (an `NiTArray`: data +4, capacity u16 +8).
const NODE_CHILDREN: usize = 0xAC;
const SCENE_LIGHTS: usize = 0xE4;
/// `ShadowSceneLight`: its fade, whether it's a point light, its `NiLight`.
const SSL_FADE: usize = 0xD4;
const SSL_POINT: usize = 0xFC;
const SSL_LIGHT: usize = 0x100;
/// `NiDynamicEffect::enable`.
const LIGHT_ON: usize = 0xAC;
/// The Night-Eye ambient (0 without Night-Eye) and the HDR switch (with HDR a dimmer may pass 1).
const NIGHT_EYE: usize = 0x00B4_6924;
const HDR: usize = 0x00B4_3070;
/// `TES*` and its fog property.
const TES: usize = 0x00B3_33A0;
const TES_FOG: usize = 0x1C;
const FOG_COLOR: usize = 0x20;
const FOG_NEAR: usize = 0x2C;
const FOG_FAR: usize = 0x30;

/// At most this many point lights on the player (the game's shader setup has eight slots).
pub const MAX_POINT_LIGHTS: usize = 8;
/// A point light whose sphere comes this close to the head (metres) can reach the player's body
/// or what the hands hold.
pub const REACH: f32 = 2.0;

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
    LightHint { sun_dir: to_sun.to_array(), sun_color: diff.map(|c| c * dimmer), ambient: amb, ..Default::default() }
}

#[cfg(not(target_arch = "x86"))]
pub fn hint() -> LightHint {
    LightHint::default()
}

/// One of the scene's lights, as the game's shaders get it (game units, the game's axes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLight {
    pub point: bool,
    /// A point light's place, or the way a directional light's light travels.
    pub at: Vec3,
    pub radius: f32,
    pub color: Vec3,
    /// The light's own ambient times its dimmer (a directional light's is the scene's ambient).
    pub ambient: Vec3,
}

/// A dimmer as the game's shader setup takes it: at most 1 without HDR.
pub fn dimmed(dimmer: f32, hdr: bool) -> f32 {
    if hdr { dimmer } else { dimmer.min(1.0) }
}

/// The ambient as the game's shader setup makes it (0x7ED6C0), Night-Eye's lift included
/// (`night_eye` 0: none).
pub fn ambient(a: Vec3, night_eye: f32) -> Vec3 {
    if night_eye <= 0.0 {
        return a;
    }
    let lum = a.x * 0.33 + a.y * 0.34 + a.z * 0.33;
    if lum <= 0.0 {
        return Vec3::splat(night_eye);
    }
    let scale = if lum <= night_eye { night_eye / lum } else { 1.0 };
    (a + Vec3::splat(0.1)) * scale
}

/// The terms for the runtime from the scene's lights (in tracking space, through
/// `world_to_tracking` from the runtime's world), for a player whose head is at `head` (tracking
/// space). `fog`: colour, near and far in game units.
pub fn lighting(lights: &[SceneLight], night_eye: f32, fog: Option<(Vec3, f32, f32)>, world_to_tracking: Mat4, head: Vec3, out: &mut GameLighting) {
    let basis = camera::basis();
    out.model = LM_GAMEBRYO;
    out.eye = Pose::IDENTITY;
    out.hemi = 1.0;
    out.depth_squash = 1.0;
    out.ssao = 1.0;
    out.lut_valid = 0;
    let mut n = 0;
    // The main light: the first directional one; its ambient is the scene's.
    let main = lights.iter().find(|l| !l.point);
    let amb = ambient(main.map_or(Vec3::ZERO, |l| l.ambient), night_eye);
    out.ambient = [amb.x, amb.y, amb.z, 0.0];
    if let Some(l) = main {
        let to = world_to_tracking.transform_vector3(basis.dir_from_game(-l.at)).normalize_or_zero();
        out.lights[n] = GameLight { pos: [to.x, to.y, to.z, 0.0], color: [l.color.x, l.color.y, l.color.z, 1.0], kind: GL_GB_DIRECTIONAL, ..Default::default() };
        n += 1;
    }
    // The point lights that reach the player, nearest (by their reach) first.
    let mut points: Vec<(f32, Vec3, f32, Vec3)> = lights
        .iter()
        .filter(|l| l.point && l.radius > 0.0 && l.color.max_element() > 0.0)
        .map(|l| {
            let p = world_to_tracking.transform_point3(basis.point_from_game(l.at));
            let r = l.radius / camera::UNITS_PER_METER;
            (p.distance(head) - r, p, r, l.color)
        })
        .filter(|(gap, ..)| *gap < REACH)
        .collect();
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, p, r, c) in points.into_iter().take(MAX_POINT_LIGHTS) {
        out.lights[n] = GameLight { pos: [p.x, p.y, p.z, 1.0 / (r * r)], color: [c.x, c.y, c.z, 1.0], kind: GL_GB_POINT, ..Default::default() };
        n += 1;
    }
    out.count = n as u32;
    // Linear fog, in metres: amount = saturate(d·y + x).
    match fog {
        Some((c, near, far)) if far > near => {
            let (near, far) = (near / camera::UNITS_PER_METER, far / camera::UNITS_PER_METER);
            out.fog_color = [c.x, c.y, c.z, 0.0];
            out.fog_fade = [-near / (far - near), 1.0 / (far - near), 0.0, 0.0];
        }
        _ => out.fog_fade = [0.0; 4],
    }
}

/// The scene's lights now (none outside the world).
#[cfg(target_arch = "x86")]
pub fn scene_lights() -> Vec<SceneLight> {
    let mut out = Vec::new();
    let Some(scene) = world_scene() else { return out };
    let hdr = mem::at::<u8>(HDR).unwrap_or(0) != 0;
    let count = mem::at::<u32>(scene + SCENE_LIGHTS + 0xC).unwrap_or(0).min(512);
    let mut node = mem::deref(scene + SCENE_LIGHTS + 4);
    for _ in 0..count {
        if node == 0 {
            break;
        }
        let ssl = mem::deref(node + 8);
        node = mem::deref(node);
        let light = if ssl != 0 { mem::deref(ssl + SSL_LIGHT) } else { 0 };
        if light == 0 || mem::at::<u8>(light + LIGHT_ON) == Some(0) {
            continue;
        }
        let point = mem::at::<u8>(ssl + SSL_POINT).unwrap_or(0) != 0;
        let (Some(fade), Some(dimmer), Some(amb), Some(diff), Some(spec)) = (
            mem::at::<f32>(ssl + SSL_FADE),
            mem::at::<f32>(light + DIMMER),
            mem::at::<[f32; 3]>(light + AMBIENT),
            mem::at::<[f32; 3]>(light + DIFFUSE),
            mem::at::<[f32; 3]>(light + SPECULAR),
        ) else {
            continue;
        };
        let at = if point { mem::at::<[f32; 3]>(light + engine::av::WORLD_TRANSLATE) } else { mem::at::<[f32; 3]>(light + DIRECTION) };
        let Some(at) = at.map(Vec3::from).filter(|v| v.is_finite()) else { continue };
        let d = dimmed(dimmer, hdr);
        out.push(SceneLight { point, at, radius: spec[0], color: Vec3::from(diff) * d * fade.max(0.0), ambient: Vec3::from(amb) * d });
    }
    // (The sun from the sky, should the scene not list it.)
    if !out.iter().any(|l| !l.point)
        && let Some(sun) = sky_sun(hdr)
    {
        out.insert(0, sun);
    }
    out
}

/// The scene's lights: each `NiLight` and its `ShadowSceneLight`.
#[cfg(target_arch = "x86")]
pub fn scene_light_nodes() -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let Some(scene) = world_scene() else { return out };
    let count = mem::at::<u32>(scene + SCENE_LIGHTS + 0xC).unwrap_or(0).min(512);
    let mut node = mem::deref(scene + SCENE_LIGHTS + 4);
    for _ in 0..count {
        if node == 0 {
            break;
        }
        let ssl = mem::deref(node + 8);
        node = mem::deref(node);
        let light = if ssl != 0 { mem::deref(ssl + SSL_LIGHT) } else { 0 };
        if light != 0 {
            out.push((light, ssl));
        }
    }
    out
}

#[cfg(not(target_arch = "x86"))]
pub fn scene_light_nodes() -> Vec<(usize, usize)> {
    Vec::new()
}

/// The sky's sun as a scene light.
#[cfg(target_arch = "x86")]
fn sky_sun(hdr: bool) -> Option<SceneLight> {
    let sky = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn() -> usize>(SKY)() };
    let light = mem::deref(mem::deref(sky + SKY_SUN) + SUN_LIGHT);
    if light == 0 {
        return None;
    }
    let d = dimmed(mem::at::<f32>(light + DIMMER)?, hdr);
    Some(SceneLight {
        point: false,
        at: Vec3::from(mem::at::<[f32; 3]>(light + DIRECTION)?),
        radius: 0.0,
        color: Vec3::from(mem::at::<[f32; 3]>(light + DIFFUSE)?) * d,
        ambient: Vec3::from(mem::at::<[f32; 3]>(light + AMBIENT)?) * d,
    })
}

#[cfg(not(target_arch = "x86"))]
pub fn scene_lights() -> Vec<SceneLight> {
    Vec::new()
}

/// The world's `ShadowSceneNode`: the "World" node's child that is one.
#[cfg(target_arch = "x86")]
fn world_scene() -> Option<usize> {
    let world = mem::deref(engine::SCENE_GRAPH);
    if world == 0 {
        return None;
    }
    let data = mem::deref(world + NODE_CHILDREN + 4);
    let n = mem::at::<u16>(world + NODE_CHILDREN + 8).unwrap_or(0).min(64) as usize;
    (0..n).map(|i| mem::deref(data + i * 4)).find(|&c| c != 0 && mem::deref(c) == SHADOW_SCENE_VTABLE)
}

/// The scene's fog now: colour, near, far (game units).
#[cfg(target_arch = "x86")]
pub fn fog() -> Option<(Vec3, f32, f32)> {
    let tes = mem::deref(TES);
    let f = if tes != 0 { mem::deref(tes + TES_FOG) } else { 0 };
    if f == 0 {
        return None;
    }
    Some((Vec3::from(mem::at::<[f32; 3]>(f + FOG_COLOR)?), mem::at::<f32>(f + FOG_NEAR)?, mem::at::<f32>(f + FOG_FAR)?))
}

#[cfg(not(target_arch = "x86"))]
pub fn fog() -> Option<(Vec3, f32, f32)> {
    None
}

/// Publishes the scene's lighting of the player for this frame (`view`: the tracking mapping, `head`
/// the head in tracking space).
pub fn publish(view: &rustvr_adapter::View, head: Vec3) {
    use std::sync::Mutex;
    static BUF: Mutex<Option<Box<GameLighting>>> = Mutex::new(None);
    static FRAME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let lights = scene_lights();
    let mut buf = BUF.lock().unwrap();
    let g = buf.get_or_insert_with(Default::default);
    let (night, fog) = (night_eye(), fog());
    lighting(&lights, night, fog, view.tracking_to_world.inverse(), head, g);
    g.frame = FRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // (Every half minute, and whenever the lights on the player change.)
    static LAST_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
    if g.frame % 1800 == 60 || LAST_COUNT.swap(g.count, std::sync::atomic::Ordering::Relaxed) != g.count {
        let points = lights.iter().filter(|l| l.point).count();
        tracing::info!(
            "lighting: {} scene lights ({points} point), {} for the player; ambient {:?}, main light {:?}, Night-Eye {night}, fog {fog:?}",
            lights.len(),
            g.count,
            &g.ambient[..3],
            lights.iter().find(|l| !l.point).map(|l| (l.color, l.at)),
        );
    }
    crate::link::with_adapter(|a| a.write_game_lighting(g));
}

/// Night-Eye's ambient now (0 without it).
pub fn night_eye() -> f32 {
    mem::at::<f32>(NIGHT_EYE).filter(|v| v.is_finite()).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_eye_lifts_a_dark_ambient() {
        assert_eq!(ambient(Vec3::splat(0.2), 0.0), Vec3::splat(0.2));
        let a = ambient(Vec3::splat(0.05), 0.3);
        assert!((a.x - 0.15 * 0.3 / 0.05).abs() < 1e-4);
        assert_eq!(ambient(Vec3::ZERO, 0.3), Vec3::splat(0.3));
    }

    #[test]
    fn lights_for_the_runtime() {
        let sun = SceneLight { point: false, at: Vec3::new(0.0, 0.0, -1.0), radius: 0.0, color: Vec3::splat(0.8), ambient: Vec3::splat(0.25) };
        // A torch 70 units (1 m) east of the player, 140 units (2 m) radius; one far away.
        let torch = SceneLight { point: true, at: Vec3::new(70.0, 0.0, 0.0), radius: 140.0, color: Vec3::new(1.0, 0.6, 0.3), ambient: Vec3::ZERO };
        let far = SceneLight { point: true, at: Vec3::new(7000.0, 0.0, 0.0), radius: 140.0, color: Vec3::ONE, ambient: Vec3::ZERO };
        let mut g = GameLighting::default();
        lighting(&[torch, sun, far], 0.0, Some((Vec3::splat(0.5), 0.0, 7000.0)), Mat4::IDENTITY, Vec3::ZERO, &mut g);
        assert_eq!(g.count, 2);
        assert_eq!(g.ambient[0], 0.25);
        assert_eq!(g.lights[0].kind, GL_GB_DIRECTIONAL);
        // Towards the sun: up.
        assert!((g.lights[0].pos[1] - 1.0).abs() < 1e-5);
        assert_eq!(g.lights[1].kind, GL_GB_POINT);
        assert!((g.lights[1].pos[3] - 0.25).abs() < 1e-5);
        assert!((g.fog_fade[1] - 0.01).abs() < 1e-6);
    }
}
