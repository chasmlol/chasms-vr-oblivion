//! The game's own sounds for everything the runtime sounds: the sound pack `oblivion-extract
//! sounds` writes into `local_assets/oblivion/sounds` (`oblivion_assets::sounds`), which of its
//! sets each object sounds like, picked the way the game picks its own sounds, and the sounds the
//! game plays itself.
//!
//! - A thing lying about: the inventory's sound of a thing of its kind (the game's item-sound
//!   choice) as it's taken, put away or dropped, and the collision sound of what it's made of and
//!   how heavy it is (the game's collision-sound choice: the Bethesda material of its Havok shape, its Havok mass) as it knocks into the world.
//! - The weapon on the body: the game's draw and sheathe of its type, with the enchanted ring over
//!   an enchanted one's draw; the shield: the inventory's sound of it.
//! - Things drawn from the body's slots: the inventory's sound of a thing of their kind, carried
//!   by each of the player's things (`InvItem::sound`, [`kind`]); the game plays its own as they're
//!   drawn, put back and stowed (`crate::inventory`, `crate::items`), so the runtime doesn't
//!   ([`SND_GAME_INVENTORY`]).

use rustvr_protocol::*;

/// The sound pack for the runtime: the pack's folder (if it's been extracted), the sounds the game
/// plays itself, and each object's set (`sets`: object id, set).
pub fn pack<'a>(sets: impl IntoIterator<Item = &'a (u32, String)>) -> Box<SoundPack> {
    let dir = rustvr_adapter::local_sound_pack("oblivion").map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
    // SAFETY: SoundPack is Zeroable (all-zero bytes are a valid one); built in place, not on the
    // stack.
    let mut p = unsafe { Box::<SoundPack>::new_zeroed().assume_init() };
    p.set_path(&dir);
    p.flags = SND_GAME_INVENTORY;
    for (id, set) in sets {
        p.add_object(*id, set, 0);
    }
    p
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

/// (Outside the game, for the adapter's tests: no game to ask.)
#[cfg(not(all(windows, target_arch = "x86")))]
pub fn kind(_form: usize) -> Option<String> {
    None
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn of_weapon(_form: usize, _type: u32) -> Option<String> {
    None
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use crate::{collision, combat, engine};

    /// The item-sound choice (thiscall on an actor: the thing's form, taken out rather than put
    /// away, used): the editor id of the inventory's sound for it.
    const ITEM_SOUND_NAME: usize = 0x005E_96E0;
    /// The collision-sound choice (cdecl: Bethesda's collision material, mass): the collision sound
    /// (`TESSound`).
    const COLLISION_SOUND: usize = 0x006B_0350;
    /// The sound by its editor id (stdcall: an editor id): the sound of that name, or none.
    const SOUND_NAMED: usize = 0x0044_7490;
    /// A form's enchantment (cdecl: a form), if it can have one and has.
    const ENCHANTMENT: usize = 0x0046_95A0;

    /// The sound set of things of `form`'s kind: the inventory's sound of one, as the game names it
    /// (`ITMPotionUp`: `potion`). None outside the game (no player) or for a form it has none for.
    pub fn kind(form: usize) -> Option<String> {
        let player = engine::player();
        if player == 0 || form == 0 {
            return None;
        }
        type Name = unsafe extern "thiscall" fn(usize, usize, i32, u8) -> usize;
        let name = unsafe { std::mem::transmute::<usize, Name>(ITEM_SOUND_NAME)(player, form, 1, 0) };
        oblivion_assets::sounds::kind_of(&combat::read_str(name))
    }

    /// The collision sound the game gives a thing of Bethesda's collision material `material` (its
    /// Havok shape's, 0..31) and Havok mass `mass`, by editor id (one of
    /// `oblivion_assets::sounds::COLLISIONS`).
    pub fn collision(material: u32, mass: f32) -> Option<&'static str> {
        use std::sync::OnceLock;
        // The game's sounds of those names (its lookup by name, once: they're the game's own records).
        static NAMED: OnceLock<Vec<(usize, &'static str)>> = OnceLock::new();
        let named = NAMED.get_or_init(|| {
            type Named = unsafe extern "stdcall" fn(*const u8) -> usize;
            oblivion_assets::sounds::COLLISIONS
                .iter()
                .filter_map(|&n| {
                    let c = std::ffi::CString::new(n).ok()?;
                    let s = unsafe { std::mem::transmute::<usize, Named>(SOUND_NAMED)(c.as_ptr().cast()) };
                    (s != 0).then_some((s, n))
                })
                .collect()
        });
        type Pick = unsafe extern "C" fn(i32, f32) -> usize;
        let material = if (15..30).contains(&material) { material - 15 } else { material };
        let s = unsafe { std::mem::transmute::<usize, Pick>(COLLISION_SOUND)(material as i32, mass) };
        named.iter().find(|(p, _)| *p == s && s != 0).map(|(_, n)| *n)
    }

    /// The sound set of a thing lying about: its kind's, with its own collision sound when the game
    /// gives it one (its Havok body `body`, of mass `mass`).
    pub fn of_thing(form: usize, body: usize, mass: f32) -> Option<String> {
        let kind = kind(form)?;
        let c = collision::body_material(body).and_then(|m| collision(m, mass));
        Some(match c {
            Some(c) => oblivion_assets::sounds::with_collision(&kind, c),
            None => kind,
        })
    }

    /// The sound set of the weapon on the body (`TESObjectWEAP` `form`, of type `type_`).
    pub fn of_weapon(form: usize, type_: u32) -> Option<String> {
        type Enchantment = unsafe extern "C" fn(usize) -> usize;
        let enchanted = form != 0 && unsafe { std::mem::transmute::<usize, Enchantment>(ENCHANTMENT)(form) } != 0;
        oblivion_assets::sounds::weapon_set(type_, enchanted)
    }
}
