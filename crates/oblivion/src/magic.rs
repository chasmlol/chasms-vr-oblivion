//! The game's own casting art where the player's casting is: a spell readied in the palm, charging
//! and charged, or a staff's enchantment at its tip, as the game shows a spell in its caster's hands
//! (docs/research/torches.md, "Spells and their art").
//!
//! The runtime says each hand's casting every frame (`rustvr_protocol::Casters`). For it the
//! adapter does what the game's own casting art does (0x5EDCE0), with the hand where the game has
//! its caster's `magicNode`: the spell's main effect (the costliest, 0x419B30) gives its model,
//! which is put in the world where the art goes (`crate::scene`, set up as the game sets up its
//! effects) playing its `SpecialIdle_Cast` sequence: the art made for the caster's hands (a fire
//! spell's flames, a frost spell's mist, a healing spell's sparkles), with the effect's own casting
//! light fading in over three quarters of it, as the game's does.
//!
//! The game plays that sequence once, a flare as the cast animation lets go. A spell held in the
//! hand plays its first part round and round ([`CYCLE_SHARE`] of it, where its particles are made),
//! so the flare burns on in the palm for as long as the spell is held. Readied (the finger on the
//! trigger) it's small; it grows to the game's own size as it charges. As the charge starts the
//! effect's casting sound plays, as the game plays it with its casting animation (0x663520 for the
//! player). Cast, the art goes: the game's own cast takes over (its projectile with its own art,
//! its sounds, its impact).
//!
//! `EffectSetting`: its model's path +0x1C (`TESModel` +0x18), its light +0x70 (a LIGH), its
//! casting sound +0x80 (a `TESSound`).

use crate::mem;
use rustvr_protocol::*;

/// The art's size while the spell is only readied, and charged (a share of the game's own).
pub const READIED_SIZE: f32 = 0.4;
pub const CHARGED_SIZE: f32 = 1.0;
/// The sequence of a spell effect's model that is its art in the caster's hands.
pub const CAST_SEQUENCE: &str = "SpecialIdle_Cast";
/// The share of the art's sequence its light takes to fade in (the game's: 0.75).
pub const LIGHT_FADE: f32 = 0.75;
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

/// The art's size for a casting state and charge.
pub fn size(state: u32, charge: f32) -> f32 {
    match state {
        CAST_READIED => READIED_SIZE,
        CAST_CHARGING => READIED_SIZE + (CHARGED_SIZE - READIED_SIZE) * charge.clamp(0.0, 1.0),
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
        crate::combat::spell_item(id).unwrap_or(0)
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
        if let Some(l) = art.light {
            l.remove();
        }
        scene::unload(art.node);
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
        let light = (light_form != 0).then(|| scene::CastingLight::new(mem::at::<u32>(light_form + FORM_ID).unwrap_or(0), node, length.unwrap_or(1.0) * LIGHT_FADE)).flatten();
        tracing::info!(
            "magic: the art of {:#x} ({path}): {CAST_SEQUENCE} {length:?} s (its first {:?} s held), its light {:?}",
            mem::at::<u32>(effect + FORM_ID).unwrap_or(0),
            length.map(cycle),
            light.as_ref().map(|l| format!("{:#x} radius {:?}", l.light(), mem::at::<f32>(l.light() + 0xF8)))
        );
        Some(Art { what, node, light, sequence, phase: 0.0, state: CAST_NONE })
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
            if c.state == CAST_CHARGING && art.state != CAST_CHARGING {
                casting_sound(effect);
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
    }
}
