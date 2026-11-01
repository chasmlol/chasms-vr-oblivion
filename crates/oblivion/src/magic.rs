//! The game's own casting art where the player's casting is: the spell each hand holds in its palm,
//! charging and charged, or a staff's enchantment at its tip, as the game shows a spell in its
//! caster's hands (docs/research/torches.md, "Spells and their art"; docs/research/spells.md).
//!
//! The runtime says each hand's casting every frame (`rustvr_protocol::Casters`). For it the
//! adapter does what the game's own casting art does (0x5EDCE0), with the hand where the game has
//! its caster's `magicNode`: the spell's main effect (its farthest range's costliest, 0x419B30)
//! gives its model, which is put in the world where the art goes (`crate::scene`, set up as the
//! game sets up its effects) playing its `SpecialIdle_Cast` sequence: the art made for the caster's
//! hands (a fire spell's flames, a frost spell's mist, a healing spell's sparkles).
//!
//! The game plays that sequence once, a flare as the cast animation lets go. A spell held in the
//! hand plays its first part round and round ([`CYCLE_SHARE`] of it, where its particles are made),
//! so the flare burns on in the palm for as long as the spell is held. Held, it's small, and smaller
//! as the player's magicka runs down (as Magic Improvements shows it), with no light (up close in a
//! headset a palm's light only dazzles, which is why the big Skyrim VR lists drop it). Charging, it
//! grows to the game's own size, the effect's casting light fading in as the game's does, and the
//! effect's casting sound plays as the charge starts, as the game plays it with its casting
//! animation (0x663520 for the player). Cast, the art goes: the game's own cast takes over (its
//! projectile with its own art and the casting light, its sounds, its impact).
//!
//! `EffectSetting`: its model's path +0x1C (`TESModel` +0x18), its light +0x70 (a LIGH), its
//! casting sound +0x80 (a `TESSound`).

use crate::mem;
use rustvr_protocol::*;

/// The art's size while the spell is only held (with the player's magicka full; empty, that share of
/// it, as Magic Improvements has it), and charged (a share of the game's own).
pub const READIED_SIZE: f32 = 0.4;
pub const EMPTY_SHARE: f32 = 0.35;
pub const CHARGED_SIZE: f32 = 1.0;
/// The sequence of a spell effect's model that is its art in the caster's hands.
pub const CAST_SEQUENCE: &str = "SpecialIdle_Cast";
/// The casting light fades in over this long as the charge starts (seconds: about the game's own
/// casting animation to its release).
pub const LIGHT_FADE: f32 = 0.3;
/// The share of the art's sequence (at least `CYCLE_MIN` seconds) played round and round while the
/// spell is held: the flare at its start, where its particles are made.
pub const CYCLE_SHARE: f32 = 0.25;
pub const CYCLE_MIN: f32 = 0.3;

const MAIN_EFFECT: usize = 0x0041_9B30;
const EFFECT_MODEL: usize = 0x1C;
const EFFECT_LIGHT: usize = 0x70;
const EFFECT_CAST_SOUND: usize = 0x80;
const FORM_ID: usize = 0x0C;
/// `MagicItem` in a spell or an enchantment.
const MAGIC_ITEM: usize = 0x18;
const LOOKUP_FORM: usize = 0x0046_B250;
const PLAY_CASTING_SOUND: usize = 0x0066_3520;

/// The art's size for a casting state and its `charge` (held: the player's magicka, 0..1).
pub fn size(state: u32, charge: f32) -> f32 {
    let c = charge.clamp(0.0, 1.0);
    match state {
        CAST_READIED => READIED_SIZE * (EMPTY_SHARE + (1.0 - EMPTY_SHARE) * c),
        CAST_CHARGING => READIED_SIZE + (CHARGED_SIZE - READIED_SIZE) * c,
        _ => CHARGED_SIZE,
    }
}

/// How much of an art's sequence (of `length` seconds) is played round and round while it's held.
pub fn cycle(length: f32) -> f32 {
    (length * CYCLE_SHARE).max(CYCLE_MIN).min(length)
}

/// The main effect (an `EffectSetting*`) of the player's spell with form id `id`, or (`object`)
/// of the enchantment with that id; 0 if there's none.
#[cfg(target_arch = "x86")]
pub fn main_effect(id: u32, object: bool) -> usize {
    let form = if object {
        type Lookup = unsafe extern "C" fn(u32) -> usize;
        unsafe { std::mem::transmute::<usize, Lookup>(LOOKUP_FORM)(id) }
    } else {
        crate::spells::spell_item(id).unwrap_or(0)
    };
    if form == 0 {
        return 0;
    }
    type Main = unsafe extern "thiscall" fn(usize, i32) -> usize;
    unsafe { std::mem::transmute::<usize, Main>(MAIN_EFFECT)(form + MAGIC_ITEM, 0) }
}

#[cfg(target_arch = "x86")]
pub use game::*;

#[cfg(target_arch = "x86")]
mod game {
    use super::*;
    use crate::scene;
    use std::sync::Mutex;

    /// One hand's art.
    struct Art {
        /// What it's for: (spell, object).
        what: (u32, u32),
        node: usize,
        /// The effect's light form (its id), and its casting light while it charges.
        light_form: u32,
        light: Option<scene::CastingLight>,
        sequence: Option<scene::Sequence>,
        /// Where its sequence is (seconds into it).
        phase: f32,
        state: u32,
    }

    struct State {
        arts: [Option<Art>; 2],
        casters: Box<Casters>,
        /// The effects whose art couldn't be made (not asked again).
        none: Vec<usize>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn drop_art(art: Art) {
        // (Its geometry isn't to be drawn apart any more: gone, another's could be where it was.)
        crate::d3d9::set_art_geometry(Vec::new());
        if let Some(l) = art.light {
            l.remove();
        }
        scene::unload(art.node);
    }

    /// The geometries under `node` (its own, if it's one): what the game draws of it.
    fn geometry(node: usize, out: &mut Vec<usize>, depth: u32) {
        if node == 0 || depth > 16 {
            return;
        }
        if crate::engine::is_a(node, "NiGeometry") {
            out.push(node);
        }
        for c in crate::nodes::children(node) {
            geometry(c, out, depth + 1);
        }
    }

    /// The art of `effect` (its model playing its casting sequence, its casting light), or None.
    fn make(effect: usize, what: (u32, u32)) -> Option<Art> {
        let path = crate::combat::read_str(mem::deref(effect + EFFECT_MODEL));
        let node = scene::load(&path);
        if node == 0 {
            tracing::warn!("magic: the game didn't load the art {path:?}");
            return None;
        }
        let sequence = scene::play(node, CAST_SEQUENCE);
        let length = sequence.map(|s| s.length);
        let light_form = mem::deref(effect + EFFECT_LIGHT);
        let light_form = if light_form != 0 { mem::at::<u32>(light_form + FORM_ID).unwrap_or(0) } else { 0 };
        tracing::info!("magic: the art of {:#x} ({path}): {CAST_SEQUENCE} {length:?} s (its first {:?} s held), its light {light_form:#x}", mem::at::<u32>(effect + FORM_ID).unwrap_or(0), length.map(cycle));
        Some(Art { what, node, light_form, light: None, sequence, phase: 0.0, state: CAST_NONE })
    }

    /// The effect's casting sound, for the player (as the game's casting animation plays it).
    fn casting_sound(effect: usize) {
        let sound = mem::deref(effect + EFFECT_CAST_SOUND);
        let player = crate::engine::player();
        if sound == 0 || player == 0 {
            return;
        }
        let id = mem::at::<u32>(sound + FORM_ID).unwrap_or(0);
        type Play = unsafe extern "thiscall" fn(usize, u32) -> usize;
        unsafe { std::mem::transmute::<usize, Play>(PLAY_CASTING_SOUND)(player, id) };
    }

    /// Before the eyes are drawn: each hand's casting art where the runtime has it.
    pub fn place(view: &rustvr_adapter::View) {
        let mut s = STATE.lock().unwrap();
        let s = s.get_or_insert_with(|| State { arts: [None, None], casters: bytemuck::zeroed_box(), none: Vec::new() });
        if !crate::link::with_adapter(|a| a.read_casters(&mut s.casters)).unwrap_or(false) {
            return;
        }
        let dt = scene::frame_time();
        for h in 0..2 {
            let c = s.casters.hands[h];
            let what = (c.spell, c.object);
            let wanted = c.state != CAST_NONE && c.flags & CASTF_CAST == 0;
            if s.arts[h].as_ref().is_some_and(|a| !wanted || a.what != what) {
                drop_art(s.arts[h].take().unwrap());
            }
            if !wanted {
                continue;
            }
            let effect = main_effect(c.spell, c.flags & CASTF_OBJECT != 0);
            if effect == 0 || s.none.contains(&effect) {
                continue;
            }
            if s.arts[h].is_none() {
                match make(effect, what) {
                    Some(a) => s.arts[h] = Some(a),
                    None => {
                        s.none.push(effect);
                        continue;
                    }
                }
            }
            let art = s.arts[h].as_mut().unwrap();
            let charging = matches!(c.state, CAST_CHARGING | CAST_READY);
            if charging && !matches!(art.state, CAST_CHARGING | CAST_READY) {
                casting_sound(effect);
                art.light = scene::CastingLight::new(art.light_form, art.node, LIGHT_FADE);
            } else if !charging && let Some(l) = art.light.take() {
                l.remove();
            }
            art.state = c.state;
            let (pos, rot) = view.pose(&c.at);
            scene::place(art.node, pos, rot, size(c.state, c.charge));
            if let Some(seq) = art.sequence {
                seq.cycle(&mut art.phase, 0.0, cycle(seq.length), dt);
            }
            scene::animate(art.node);
            if let Some(l) = &art.light {
                l.update(dt);
            }
        }
        // What the game draws of the art, drawn apart from its picture (`crate::d3d9::art_eye`).
        let mut drawn = Vec::new();
        for a in s.arts.iter().flatten() {
            geometry(a.node, &mut drawn, 0);
        }
        crate::d3d9::set_art_geometry(drawn);
        super::publish_overlays();
    }
}

/// `TESEffectShader`: its `EFSH` data at +0x18 as the record has it (flags; the membrane's blend;
/// the fill's colour and alpha (fade in, full, fade out, persistent ratio, pulse amplitude and
/// frequency) at +0x28; its texture's speed +0x44; the edge's falloff +0x4C, colour and alpha at
/// +0x50; their full alpha ratios +0x6C and +0x70; the membrane's destination blend +0x74), its
/// fill texture's path +0xFC.
const SHADER_FLAGS: usize = 0x18;
const SHADER_FILL: usize = 0x28;
const SHADER_SPEED: usize = 0x44;
const SHADER_FALLOFF: usize = 0x4C;
const SHADER_EDGE: usize = 0x50;
const SHADER_DEST_BLEND: usize = 0x74;
const SHADER_FILL_FULL: usize = 0x6C;
const SHADER_EDGE_FULL: usize = 0x70;
const SHADER_FILL_TEXTURE: usize = 0xFC;
/// The shader's flags: no membrane, an inverse edge.
const SHADER_NO_MEMBRANE: u8 = 0x01;
const SHADER_EDGE_INVERSE: u8 = 0x10;
/// Direct3D's `D3DBLEND_ONE`: the membrane is added.
const BLEND_ONE: u32 = 2;
/// `EffectSetting`'s enchant effect (the shader an enchanted thing wears).
const EFFECT_ENCHANT_SHADER: usize = 0x7C;

/// One of the game's membranes (the fill at +0x28, the edge at +0x50, each followed by its full
/// alpha ratio at `full`): its colour (0..1), its alpha `age` seconds after its effect began (None:
/// settled, its persistent ratio) and its pulse ([amplitude, frequency]). The game's curve: it
/// fades in to its full ratio, holds it, then fades to its persistent ratio and stays there.
fn membrane(shader: usize, at: usize, full: usize, age: Option<f32>) -> Option<([f32; 3], f32, [f32; 2])> {
    let c = mem::at::<[u8; 4]>(shader + at)?;
    // Fade in, full alpha time, fade out, persistent ratio, pulse amplitude, pulse frequency.
    let m = mem::at::<[f32; 6]>(shader + at + 4)?;
    let full = mem::at::<f32>(shader + full)?;
    let alpha = match age {
        None => m[3],
        Some(t) if t < m[0] => full * t / m[0].max(1e-3),
        Some(t) if t < m[0] + m[1] => full,
        Some(t) if t < m[0] + m[1] + m[2] => full + (m[3] - full) * (t - m[0] - m[1]) / m[2].max(1e-3),
        Some(_) => m[3],
    };
    Some(([c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0], alpha.clamp(0.0, 1.0), [m[4], m[5]]))
}

/// The game's effect shader `shader` (a `TESEffectShader`) over `object`, `age` seconds after its
/// effect began (None: settled): each membrane at its alpha then, pulsing as it pulses, its fill
/// texture scrolling as it scrolls, its edge's falloff, blended over or added as its destination
/// blend says. None: it draws no membrane, or nothing of it shows now.
pub fn shader_overlay(shader: usize, object: u32, age: Option<f32>) -> Option<Overlay> {
    let flags = mem::at::<u8>(shader + SHADER_FLAGS)?;
    if flags & SHADER_NO_MEMBRANE != 0 {
        return None;
    }
    let (fill, fill_a, fill_pulse) = membrane(shader, SHADER_FILL, SHADER_FILL_FULL, age)?;
    let (edge, edge_a, edge_pulse) = membrane(shader, SHADER_EDGE, SHADER_EDGE_FULL, age)?;
    let mut o: Overlay = bytemuck::Zeroable::zeroed();
    o.object = object;
    o.flags = if edge_a > 0.0 { OVL_EDGE } else { 0 } | if fill_a > 0.0 { OVL_FILL } else { 0 } | if flags & SHADER_EDGE_INVERSE != 0 { OVL_EDGE_INVERSE } else { 0 } | if mem::at::<u32>(shader + SHADER_DEST_BLEND) == Some(BLEND_ONE) { OVL_ADDITIVE } else { 0 };
    o.fill = [fill[0], fill[1], fill[2], fill_a];
    o.fill_pulse = fill_pulse;
    o.scroll = mem::at::<[f32; 2]>(shader + SHADER_SPEED).unwrap_or_default();
    o.edge = [edge[0], edge[1], edge[2], edge_a];
    o.edge_pulse = edge_pulse;
    o.edge_falloff = mem::at::<f32>(shader + SHADER_FALLOFF).unwrap_or(1.0);
    let texture = crate::combat::read_str(mem::deref(shader + SHADER_FILL_TEXTURE));
    if !texture.is_empty() {
        o.set_texture(&format!("oblivion/{}", oblivion_assets::export::texture_key(&format!("textures\\{texture}"))));
    }
    (o.flags & (OVL_EDGE | OVL_FILL) != 0).then_some(o)
}

/// The shimmer of what's enchanted with `enchantment` (its main effect's enchant effect shader),
/// settled, over the object `object`. None: no enchantment, or its shader draws no membrane.
#[cfg(target_arch = "x86")]
pub fn enchant_overlay(enchantment: u32, object: u32) -> Option<Overlay> {
    let effect = main_effect(enchantment, true);
    let shader = mem::deref(effect + EFFECT_ENCHANT_SHADER);
    if effect == 0 || shader == 0 {
        return None;
    }
    let o = shader_overlay(shader, object, None);
    tracing::info!(
        "magic: the shimmer of {enchantment:#x} (its effect {:#x}, shader {:#x}): {:?}",
        mem::at::<u32>(effect + FORM_ID).unwrap_or(0),
        mem::at::<u32>(shader + FORM_ID).unwrap_or(0),
        o.map(|o| (o.flags, o.fill, o.edge, o.edge_falloff, o.texture().to_owned()))
    );
    o
}

/// The overlays the runtime has (the weapon's shimmer, set as the weapon changes; the spells'
/// shaders on the player, every frame), as last written.
struct OverlayState {
    weapon: Option<Overlay>,
    sent: Option<Vec<u8>>,
    table: Box<Overlays>,
}

static OVERLAYS: std::sync::Mutex<Option<OverlayState>> = std::sync::Mutex::new(None);

/// The equipped weapon's shimmer from now on (None: none).
pub fn set_weapon_overlay(o: Option<Overlay>) {
    let mut s = OVERLAYS.lock().unwrap();
    s.get_or_insert_with(|| OverlayState { weapon: None, sent: None, table: bytemuck::zeroed_box() }).weapon = o;
}

/// Once a frame: the weapon's shimmer and the spells' shaders on the player's body (each where its
/// curve has it now), written when they change.
#[cfg(target_arch = "x86")]
fn publish_overlays() {
    let mut s = OVERLAYS.lock().unwrap();
    let s = s.get_or_insert_with(|| OverlayState { weapon: None, sent: None, table: bytemuck::zeroed_box() });
    let mut all: Vec<Overlay> = s.weapon.into_iter().collect();
    for (_, shader, age) in player_hit_shaders() {
        if let Some(o) = shader_overlay(shader, OVERLAY_BODY, Some(age)) {
            all.push(o);
        }
    }
    all.truncate(MAX_OVERLAYS);
    let bytes: Vec<u8> = all.iter().flat_map(|o| bytemuck::bytes_of(o).to_vec()).collect();
    if s.sent.as_ref() == Some(&bytes) {
        return;
    }
    s.table.count = all.len() as u32;
    for (i, o) in all.iter().enumerate() {
        s.table.overlays[i] = *o;
    }
    if crate::link::with_adapter(|a| a.write_overlays(&s.table)).unwrap_or(false) {
        s.sent = Some(bytes);
    }
}

/// `ActorProcessManager`'s two lists of the game's temporary effects in the world (`tList`s,
/// inline first nodes, 0x678D30 puts each in one): its spells' shaders on actors
/// (`MagicShaderHitEffect`: the target +0x1C, its age +0x20, on what the actor holds +0x28, the
/// shader +0x34) among them.
const PROCESS_MANAGER: usize = 0x00B3_BD00;
const TEMP_EFFECTS: [usize; 2] = [0x40, 0x48];
const HIT_TARGET: usize = 0x1C;
const HIT_AGE: usize = 0x20;
/// Set on one that's on what the actor holds (its enchanted weapon), not on its body (seen: 1 for a
/// weapon's enchantment, 0 for a healing spell on the player).
const HIT_ON_ITEM: usize = 0x28;
const HIT_SHADER: usize = 0x34;

/// The game's spells' shaders on the player now (each `MagicShaderHitEffect` whose target is the
/// player): (the effect, its shader, how long it has run: seconds).
#[cfg(target_arch = "x86")]
pub fn player_hit_shaders() -> Vec<(usize, usize, f32)> {
    let player = crate::engine::player();
    let mut out = Vec::new();
    for list in TEMP_EFFECTS {
        let mut node = PROCESS_MANAGER + list;
        let mut guard = 0;
        while node != 0 && guard < 512 {
            guard += 1;
            let e = mem::deref(node);
            node = mem::deref(node + 4);
            // (One on what the actor holds, its enchanted weapon's shimmer, isn't on its body: the
            // weapon's own overlay draws that.)
            if e == 0 || crate::nodes::class(e) != "MagicShaderHitEffect" || mem::deref(e + HIT_TARGET) != player || mem::at::<u8>(e + HIT_ON_ITEM) != Some(0) {
                continue;
            }
            let shader = mem::deref(e + HIT_SHADER);
            if shader != 0 {
                out.push((e, shader, mem::at::<f32>(e + HIT_AGE).filter(|t| t.is_finite()).unwrap_or(0.0)));
            }
        }
    }
    out
}
