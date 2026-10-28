//! The player's body as the game makes it up, for the runtime's avatar
//! (`rustvr_protocol::AvatarDesc`): the game's skeleton, the race's skin where nothing is worn
//! (upper body, lower body, hands, feet; a beast's tail), and whatever armour and clothes are worn
//! over each of those, plus rings and an amulet. Each is the game's own model, extracted by
//! `oblivion-extract export` and named by the hash of its path (`oblivion_assets::export::
//! model_key`). The race's own skin goes on the parts made with the Imperial one (texture swaps).
//!
//! Forms (xOBSE's headers, checked in the log): the player's base (`TESNPC`, the reference's +0x1C)
//! has its flags at +0x28 (female: bit 0) and its race at +0xE8. Worn items are the inventory
//! entries (`ExtraContainerChanges` in the reference's extra data, +0x44) with an extra data list
//! marked worn (`ExtraWorn` 0x1B, `ExtraWornLeft` 0x1C). An armour's `TESBipedModelForm` is at
//! +0x64, clothing's at +0x5C: the slots it covers (+4), its male and female models (+0xC, +0x24).

use crate::{combat::read_str, engine, mem};
use oblivion_assets::export::{model_key, texture_key};
use rustvr_protocol::*;
use std::sync::Mutex;

const BASE_FORM: usize = 0x1C;
const NPC_FLAGS: usize = 0x28;
const NPC_RACE: usize = 0xE8;
const RACE_NAME: usize = 0x1C;
/// `TESRace::tails[2]` (`TESModel`, its path at +4).
const RACE_TAILS: usize = 0xB0;
const MODEL_SIZE: usize = 0x18;
const EXTRA_CONTAINER_CHANGES: u8 = 0x1A;
const EXTRA_WORN: u8 = 0x1B;
const EXTRA_WORN_LEFT: u8 = 0x1C;
const FORM_TYPE: usize = 0x04;
const ARMOR: u8 = 0x14;
const CLOTHING: u8 = 0x16;

/// Biped slots (bits of a `TESBipedModelForm`'s mask).
const UPPER_BODY: u16 = 1 << 2;
const LOWER_BODY: u16 = 1 << 3;
const HAND: u16 = 1 << 4;
const FOOT: u16 = 1 << 5;
const RINGS_AMULET: u16 = (1 << 6) | (1 << 7) | (1 << 8);
const TAIL: u16 = 1 << 15;

/// The skin parts and the slot each fills (male, female), under `characters\_male\`.
const SKIN: [(u16, &str, &str); 4] = [
    (UPPER_BODY, "upperbody.nif", "femaleupperbody.nif"),
    (LOWER_BODY, "lowerbody.nif", "femalelowerbody.nif"),
    (HAND, "hand.nif", "femalehand.nif"),
    (FOOT, "foot.nif", "femalefoot.nif"),
];

/// The Imperial skin textures the generic body parts are made with (male, female), under
/// `textures\characters\imperial\`, swapped for the race's own.
const SKIN_TEXTURES: [&str; 5] = ["upperbody", "leg", "hand", "foot", "groin"];

/// The body as the player wears it.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Body {
    pub race: String,
    pub female: bool,
    pub beast: bool,
    /// Worn items: (slots, model path).
    pub worn: Vec<(u16, String)>,
    pub tail: String,
}

/// Walks a `tList` from its inline first node: its items.
fn list(head: usize, max: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut node = head;
    while node != 0 && out.len() < max {
        let item = mem::deref(node);
        if item != 0 {
            out.push(item);
        }
        node = mem::deref(node + 4);
    }
    out
}

/// The extra data of `kind` in an extra data list, or 0.
fn extra(list: usize, kind: u8) -> usize {
    let mut e = mem::deref(list + 4);
    for _ in 0..64 {
        if e == 0 {
            break;
        }
        if mem::at::<u8>(e + 4) == Some(kind) {
            return e;
        }
        e = mem::deref(e + 8);
    }
    0
}

/// The player's body, if there's a player.
pub fn body() -> Option<Body> {
    let player = engine::player();
    let base = mem::deref(player + BASE_FORM);
    let race = mem::deref(base + NPC_RACE);
    if player == 0 || base == 0 || race == 0 {
        return None;
    }
    let female = mem::at::<u32>(base + NPC_FLAGS)? & 1 != 0;
    let tail = read_str(mem::deref(race + RACE_TAILS + if female { MODEL_SIZE } else { 0 } + 4));
    let mut b = Body { race: read_str(mem::deref(race + RACE_NAME)), female, beast: !tail.is_empty(), worn: Vec::new(), tail };
    let changes = extra(player + 0x44, EXTRA_CONTAINER_CHANGES);
    let data = mem::deref(changes + 0x0C);
    for entry in list(mem::deref(data), 512) {
        let form = mem::deref(entry + 8);
        let biped = match mem::at::<u8>(form + FORM_TYPE) {
            Some(ARMOR) => form + 0x64,
            Some(CLOTHING) => form + 0x5C,
            _ => continue,
        };
        let worn = list(mem::deref(entry), 64).into_iter().any(|l| extra(l, EXTRA_WORN) != 0 || extra(l, EXTRA_WORN_LEFT) != 0);
        if !worn {
            continue;
        }
        let slots = mem::at::<u16>(biped + 4).unwrap_or(0);
        let path = read_str(mem::deref(biped + if female { 0x24 } else { 0x0C }));
        let path = if path.is_empty() && female { read_str(mem::deref(biped + 0x0C)) } else { path };
        if !path.is_empty() {
            b.worn.push((slots, path));
        }
    }
    b.worn.sort();
    Some(b)
}

/// The avatar for a body: the skeleton, a worn item's model over each slot it covers, the race's
/// skin on the slots nothing covers, rings and the amulet, a beast's tail.
pub fn avatar(b: &Body) -> AvatarDesc {
    let mut d: AvatarDesc = bytemuck::Zeroable::zeroed();
    let name = |path: &str| format!("oblivion/{}", model_key(path));
    let skeleton = if b.beast { "characters\\_male\\skeletonbeast.nif" } else { "characters\\_male\\skeleton.nif" };
    set_name(&mut d.skeleton, &name(skeleton));
    let mut covered = 0u16;
    for (slots, path) in &b.worn {
        if slots & (UPPER_BODY | LOWER_BODY | HAND | FOOT | RINGS_AMULET) != 0 {
            d.push_part(&name(path));
            covered |= slots;
        }
    }
    for (slot, male, female) in SKIN {
        if covered & slot == 0 {
            d.push_part(&name(&format!("characters\\_male\\{}", if b.female { female } else { male })));
        }
    }
    if b.beast && covered & TAIL == 0 && !b.tail.is_empty() {
        d.push_part(&name(&b.tail));
    }
    // The race's own skin: its folder for the Imperial one.
    let folder = b.race.to_ascii_lowercase().replace(' ', "");
    if !folder.is_empty() && folder != "imperial" {
        let sex = if b.female { "female" } else { "male" };
        for part in SKIN_TEXTURES {
            for suffix in ["", "_n"] {
                let file = format!("{part}{sex}{suffix}.dds");
                let from = format!("textures\\characters\\imperial\\{sex}\\{file}");
                let to = format!("textures\\characters\\{folder}\\{sex}\\{file}");
                d.push_swap(&texture_key(&from), &texture_key(&to));
            }
        }
    }
    d
}

static SENT: Mutex<Option<Body>> = Mutex::new(None);

/// Every so often in the world: the body, sent again when it has changed.
pub fn publish(a: &rustvr_adapter::Adapter) {
    let Some(b) = body() else { return };
    let mut sent = SENT.lock().unwrap();
    if sent.as_ref() == Some(&b) {
        return;
    }
    let d = avatar(&b);
    tracing::info!(
        "body: {} {} ({}), wearing {:?}: {} parts, {} texture swaps",
        if b.female { "female" } else { "male" },
        b.race,
        if b.beast { "beast" } else { "human" },
        b.worn,
        d.count,
        d.swap_count
    );
    if a.write_avatar(&d) {
        *sent = Some(b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worn_items_cover_their_slots_and_the_skin_fills_the_rest() {
        let b = Body { race: "Nord".into(), female: false, beast: false, worn: vec![(UPPER_BODY | LOWER_BODY, "clothes\\robe.nif".into()), (1 << 6, "clothes\\ring.nif".into())], tail: String::new() };
        let d = avatar(&b);
        let parts: Vec<&str> = (0..d.count as usize).map(|i| d.part(i)).collect();
        assert_eq!(parts[0], format!("oblivion/{}", model_key("clothes\\robe.nif")));
        assert_eq!(parts[1], format!("oblivion/{}", model_key("clothes\\ring.nif")));
        // Hands and feet are the skin's; the upper and lower body aren't.
        assert_eq!(parts[2..], [format!("oblivion/{}", model_key("characters\\_male\\hand.nif")), format!("oblivion/{}", model_key("characters\\_male\\foot.nif"))]);
        assert_eq!(name_str(&d.skeleton), format!("oblivion/{}", model_key("characters\\_male\\skeleton.nif")));
        // A Nord's skin for the Imperial one.
        assert_eq!(d.swap(0), (texture_key("textures\\characters\\imperial\\male\\upperbodymale.dds").as_str(), texture_key("textures\\characters\\nord\\male\\upperbodymale.dds").as_str()));
    }

    #[test]
    fn a_beast_has_its_own_skeleton_and_its_tail() {
        let b = Body { race: "Khajiit".into(), female: true, beast: true, worn: Vec::new(), tail: "characters\\khajiit\\tail.nif".into() };
        let d = avatar(&b);
        assert_eq!(name_str(&d.skeleton), format!("oblivion/{}", model_key("characters\\_male\\skeletonbeast.nif")));
        assert_eq!(d.count, 5);
        assert_eq!(d.part(4), format!("oblivion/{}", model_key("characters\\khajiit\\tail.nif")));
    }
}
