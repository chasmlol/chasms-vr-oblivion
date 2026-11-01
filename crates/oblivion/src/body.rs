//! The player's body as the game makes it up, for the runtime's avatar
//! (`rustvr_protocol::AvatarDesc`): the game's skeleton, the race's skin where nothing is worn
//! (upper body, lower body, hands, feet; a beast's tail), and whatever armour and clothes are worn
//! over each of those, plus rings and an amulet; and the head: the race's head and the parts made
//! to it (ears, mouth, teeth, tongue, eyes), the hair, and the helmet or hood worn over it. Each is
//! the game's own model, extracted by `oblivion-extract export` and named by the hash of its path
//! (`oblivion_assets::export::model_key`). The race's own skin goes on the parts made with the
//! Imperial one, and the race's, the eyes' and the hair's own textures on the head's parts
//! (texture swaps).
//!
//! The face (`rustvr_protocol::FaceDesc`, docs/design/mirror.md): where it is in FaceGen's face
//! space (the race's coordinates plus the NPC's, as the game adds them, 0x552990), the age map the
//! game lays over the head's skin (FaceGen's age from those coordinates picks it, as 0x551A00
//! does) and the hair's colour. The race menu changes all of it in place, so while it's open it's
//! read every frame.
//!
//! Forms (xOBSE's headers, checked in the log): the player's base (`TESNPC`, the reference's +0x1C)
//! has its flags at +0x28 (female: bit 0) and its race at +0xE8. Worn items are the inventory
//! entries (`ExtraContainerChanges` in the reference's extra data, +0x44) with an extra data list
//! marked worn (`ExtraWorn` 0x1B, `ExtraWornLeft` 0x1C). An armour's `TESBipedModelForm` is at
//! +0x64, clothing's at +0x5C: the slots it covers (+4), its male and female models (+0xC, +0x24).
//! The NPC's hair (`TESHair*`, +0x1C8: its model's path +0x28, its texture's +0x40), its eyes
//! (`TESEyes*`, +0x1D0: its texture's path +0x28) and its hair's colour (+0x1E8, RGB bytes). The
//! race's face parts: nine `TESModel`s from +0xE0 (head, male ears, female ears, mouth, lower teeth,
//! upper teeth, tongue, left eye, right eye; their paths at +4) and their textures, nine
//! `TESTexture`s of 0xC from +0x1B8 (paths at +4). FaceGen's coordinates: the race's at +0x29C, the
//! NPC's where the game's 0x521A10 says (+0x108 or +0x168); each four vectors of 0x18 (symmetric
//! and asymmetric geometry, symmetric and asymmetric texture), each its count at +0 and its floats
//! from the pointer at +0xC.
//!
//! Arrows worn hang their quiver on the body as the game hangs it (the arrows' model's `Prn`, the
//! skeleton's `Quiver`: on the back, its arrows at the right shoulder), showing as many of its five
//! arrows as there are left (`oblivion-extract` writes `#quiver0` .. `#quiver5`).

use crate::{combat::read_str, engine, mem};
use oblivion_assets::export::{model_key, texture_key, variant_key};
use rustvr_protocol::*;
use std::collections::HashMap;
use std::sync::Mutex;

const BASE_FORM: usize = 0x1C;
const NPC_FLAGS: usize = 0x28;
const NPC_RACE: usize = 0xE8;
const NPC_HAIR: usize = 0x1C8;
const NPC_EYES: usize = 0x1D0;
const NPC_HAIR_COLOR: usize = 0x1E8;
/// `TESNPC`'s FaceGen coordinates, the set the game uses (thiscall, the NPC -> its coordinates).
const NPC_FACE_COORDS: usize = 0x0052_1A10;
const RACE_NAME: usize = 0x1C;
/// `TESRace::tails[2]` (`TESModel`, its path at +4).
const RACE_TAILS: usize = 0xB0;
const RACE_FACE_MODELS: usize = 0xE0;
const RACE_FACE_TEXTURES: usize = 0x1B8;
const RACE_FACE_COORDS: usize = 0x29C;
const HAIR_MODEL: usize = 0x28;
const HAIR_TEXTURE: usize = 0x40;
const EYES_TEXTURE: usize = 0x28;
const MODEL_SIZE: usize = 0x18;
const TEXTURE_SIZE: usize = 0xC;
const COORDS_SIZE: usize = 0x18;
const EXTRA_CONTAINER_CHANGES: u8 = 0x1A;
const EXTRA_WORN: u8 = 0x1B;
const EXTRA_WORN_LEFT: u8 = 0x1C;
const FORM_TYPE: usize = 0x04;
const ARMOR: u8 = 0x14;
const CLOTHING: u8 = 0x16;
const AMMO: u8 = 0x22;
/// `TESAmmo`'s model path; the arrows a quiver shows at most.
const AMMO_MODEL: usize = 0x34;
const QUIVER_ARROWS: u32 = 5;

/// Biped slots (bits of a `TESBipedModelForm`'s mask).
const HEAD: u16 = 1 << 0;
const HAIR: u16 = 1 << 1;
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

/// The race's face parts the body has (indices into its nine): the head, the ears (by sex), the
/// mouth, the teeth, the tongue, the eyes.
const FACE_PARTS: [usize; 7] = [0, 3, 4, 5, 6, 7, 8];
const EARS: [usize; 2] = [1, 2];
const LEFT_EYE: usize = 7;
const RIGHT_EYE: usize = 8;

/// The body as the player wears it.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Body {
    pub race: String,
    pub female: bool,
    pub beast: bool,
    /// Worn items: (slots, model path).
    pub worn: Vec<(u16, String)>,
    pub tail: String,
    /// The head's parts: (which of the race's, model path, the race's texture for it).
    pub face: Vec<(usize, String, String)>,
    /// The hair: its model and texture.
    pub hair: Option<(String, String)>,
    /// The eyes' texture.
    pub eyes: String,
    /// Arrows worn: their model and how many of the quiver's arrows show.
    pub quiver: Option<(String, u32)>,
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

/// What the player carries: each form in the inventory's changes and how many (their count, as the
/// changes keep it; the player's base form starts with nothing an alchemist needs).
pub fn carried() -> Vec<(usize, i32)> {
    let player = engine::player();
    if player == 0 {
        return Vec::new();
    }
    let changes = extra(player + 0x44, EXTRA_CONTAINER_CHANGES);
    let data = mem::deref(changes + 0x0C);
    if changes == 0 || data == 0 {
        return Vec::new();
    }
    list(mem::deref(data), 1024).into_iter().map(|entry| (mem::deref(entry + 8), mem::at::<i32>(entry + 4).unwrap_or(0))).filter(|(form, _)| *form != 0).collect()
}

/// How many an inventory entry's extra data list stands for (`ExtraDataList::GetCount`).
#[cfg(target_arch = "x86")]
fn stack_count(list: usize) -> u32 {
    type Count = unsafe extern "thiscall" fn(usize) -> i16;
    (unsafe { std::mem::transmute::<usize, Count>(0x0041_E860)(list) }).max(0) as u32
}

#[cfg(not(target_arch = "x86"))]
fn stack_count(_list: usize) -> u32 {
    1
}

/// The player's base form and race, if there's a player.
fn npc() -> Option<(usize, usize)> {
    let player = engine::player();
    let base = mem::deref(player + BASE_FORM);
    let race = mem::deref(base + NPC_RACE);
    (player != 0 && base != 0 && race != 0).then_some((base, race))
}

/// The player's body, if there's a player.
pub fn body() -> Option<Body> {
    let player = engine::player();
    let (base, race) = npc()?;
    let female = mem::at::<u32>(base + NPC_FLAGS)? & 1 != 0;
    let tail = read_str(mem::deref(race + RACE_TAILS + if female { MODEL_SIZE } else { 0 } + 4));
    let mut b = Body { race: read_str(mem::deref(race + RACE_NAME)), female, beast: !tail.is_empty(), tail, ..Default::default() };
    let changes = extra(player + 0x44, EXTRA_CONTAINER_CHANGES);
    let data = mem::deref(changes + 0x0C);
    for entry in list(mem::deref(data), 512) {
        let form = mem::deref(entry + 8);
        // Arrows worn: their quiver, with as many showing as are left (up to its five).
        if mem::at::<u8>(form + FORM_TYPE) == Some(AMMO)
            && let Some(l) = list(mem::deref(entry), 64).into_iter().find(|&l| extra(l, EXTRA_WORN) != 0)
        {
            let path = read_str(mem::deref(form + AMMO_MODEL));
            if !path.is_empty() {
                b.quiver = Some((path, stack_count(l).min(QUIVER_ARROWS)));
            }
            continue;
        }
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
    // The race's head and the parts made to it, each with the race's texture for it.
    let ears = EARS[female as usize];
    for i in FACE_PARTS.into_iter().chain([ears]) {
        let model = read_str(mem::deref(race + RACE_FACE_MODELS + i * MODEL_SIZE + 4));
        let texture = read_str(mem::deref(race + RACE_FACE_TEXTURES + i * TEXTURE_SIZE + 4));
        if !model.is_empty() {
            b.face.push((i, model, texture));
        }
    }
    let hair = mem::deref(base + NPC_HAIR);
    if hair != 0 {
        let model = read_str(mem::deref(hair + HAIR_MODEL));
        if !model.is_empty() {
            b.hair = Some((model, read_str(mem::deref(hair + HAIR_TEXTURE))));
        }
    }
    let eyes = mem::deref(base + NPC_EYES);
    b.eyes = if eyes != 0 { read_str(mem::deref(eyes + EYES_TEXTURE)) } else { String::new() };
    Some(b)
}

/// A cached model's meshes' base, normal and detail textures (keys; a hair's detail is its layer
/// map), for the swaps onto them.
fn mesh_textures(key: &str) -> Vec<(String, String, String)> {
    static CACHE: Mutex<Option<HashMap<String, Vec<(String, String, String)>>>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap();
    cache
        .get_or_insert_with(HashMap::new)
        .entry(key.to_owned())
        .or_insert_with(|| {
            rustvr_asset::load_model(&dir(), key)
                .map(|m| m.meshes.iter().map(|m| (m.texture.clone(), m.material.bump.clone(), m.material.detail.clone())).collect())
                .unwrap_or_default()
        })
        .clone()
}

/// The asset cache's folder for Oblivion.
fn dir() -> std::path::PathBuf {
    rustvr_asset::default_root().join("oblivion")
}

/// The cache has this texture.
fn have_texture(key: &str) -> bool {
    dir().join("textures").join(format!("{key}.{}", rustvr_asset::TEXTURE_EXT)).is_file()
}

/// A game texture path (under `textures\`), its normal map (`<path>_n.dds`) and its hair's layer
/// map (`<path>_hl.dds`), as keys.
fn with_normal(path: &str) -> (String, String, String) {
    let full = format!("textures\\{}", oblivion_assets::export::normalise(path, "textures\\"));
    let with = |suffix: &str| full.strip_suffix(".dds").map(|s| format!("{s}{suffix}.dds")).unwrap_or_default();
    (texture_key(&full), texture_key(&with("_n")), texture_key(&with("_hl")))
}

/// The avatar for a body: the skeleton, a worn item's model over each slot it covers, the race's
/// skin on the slots nothing covers, rings and the amulet, a beast's tail; the head's parts, the
/// hair unless something worn covers it, and the helmet or hood; and the race's skin under what's
/// worn (`rustvr_protocol::UNDER`).
pub fn avatar(b: &Body) -> AvatarDesc {
    let mut d: AvatarDesc = bytemuck::Zeroable::zeroed();
    let name = |path: &str| format!("oblivion/{}", model_key(path));
    let skeleton = if b.beast { "characters\\_male\\skeletonbeast.nif" } else { "characters\\_male\\skeleton.nif" };
    set_name(&mut d.skeleton, &name(skeleton));
    let mut covered = 0u16;
    for (slots, path) in &b.worn {
        if slots & (HEAD | HAIR | UPPER_BODY | LOWER_BODY | HAND | FOOT | RINGS_AMULET) != 0 {
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
    if let Some((path, shown)) = &b.quiver {
        d.push_part(&format!("oblivion/{}", variant_key(path, &format!("quiver{shown}"))));
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
    // The head: each part with the race's own texture (and its normal map) on it; the eyes with
    // the chosen eyes' texture; the hair, unless what's worn covers its slot (a helmet, a hood),
    // with its own texture.
    let swap_onto = |d: &mut AvatarDesc, model: &str, texture: &str| {
        let key = model_key(model);
        if texture.is_empty() {
            return;
        }
        let (to, to_normal, to_layer) = with_normal(texture);
        for (from, from_normal, from_layer) in mesh_textures(&key) {
            if !from.is_empty() && from != to {
                d.push_swap(&from, &to);
            }
            if !from_normal.is_empty() && from_normal != to_normal && have_texture(&to_normal) {
                d.push_swap(&from_normal, &to_normal);
            }
            if !from_layer.is_empty() && from_layer != to_layer && have_texture(&to_layer) {
                d.push_swap(&from_layer, &to_layer);
            }
        }
    };
    for (i, model, texture) in &b.face {
        d.push_part(&name(model));
        let eye = *i == LEFT_EYE || *i == RIGHT_EYE;
        let texture = if eye && !b.eyes.is_empty() { &b.eyes } else { texture };
        swap_onto(&mut d, model, texture);
    }
    if let Some((model, texture)) = b.hair.as_ref().filter(|_| covered & HAIR == 0) {
        d.push_part(&name(model));
        swap_onto(&mut d, model, texture);
    }
    // Under what's worn, the skin all the same: the game never shows it (its armour bends with the
    // body, closed), but a rigid plate's open end does as the joint bends out of it (a cuff with the
    // hand bent away), and the arm inside shows there, not the armour's hollow. (Last: with too many
    // parts, it's what goes.)
    for (slot, male, female) in SKIN {
        if covered & slot != 0 {
            d.push_part(&format!("{}{UNDER}", name(&format!("characters\\_male\\{}", if b.female { female } else { male }))));
        }
    }
    d
}

/// FaceGen's age control (`facegen.toml`, written by the extractor from `FaceGen\si.ctl`).
#[derive(serde::Deserialize)]
struct AgeControl {
    age: Vec<f32>,
    age_offset: f32,
}

fn age_control() -> Option<&'static AgeControl> {
    static AGE: std::sync::OnceLock<Option<AgeControl>> = std::sync::OnceLock::new();
    AGE.get_or_init(|| {
        let text = std::fs::read_to_string(dir().join("facegen.toml")).map_err(|e| tracing::warn!("body: FaceGen's age control: {e}")).ok()?;
        toml::from_str(&text).map_err(|e| tracing::warn!("body: FaceGen's age control: {e}")).ok()
    })
    .as_ref()
}

/// The player's FaceGen coordinate sets, as the race menu edits them: the NPC's own (the set the
/// game's 0x521A10 picks) and its race's, with the NPC and the race: (npc set, race set, npc, race).
pub fn face_sets() -> Option<(usize, usize, usize, usize)> {
    let (base, race) = npc()?;
    #[cfg(target_arch = "x86")]
    let npc_set = unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(NPC_FACE_COORDS)(base) };
    #[cfg(not(target_arch = "x86"))]
    let npc_set = base + 0x108;
    (npc_set != 0).then_some((npc_set, race + RACE_FACE_COORDS, base, race))
}

/// One of a coordinate set's four vectors: its floats.
pub(crate) fn coords(set: usize, which: usize) -> Vec<f32> {
    let v = set + which * COORDS_SIZE;
    let n = mem::at::<u32>(v).unwrap_or(0).min(64) as usize;
    let first = mem::deref(v + 0x0C);
    if first == 0 {
        return Vec::new();
    }
    (0..n).filter_map(|i| mem::at::<f32>(first + 4 * i)).collect()
}

/// The race's and the NPC's coordinates added, as the game adds them.
fn added(a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let n = a.len().max(b.len());
    (0..n).map(|i| a.get(i).copied().unwrap_or(0.0) + b.get(i).copied().unwrap_or(0.0)).collect()
}

/// The player's face, if there's a player.
pub fn face() -> Option<FaceDesc> {
    let (base, race) = npc()?;
    let female = mem::at::<u32>(base + NPC_FLAGS)? & 1 != 0;
    #[cfg(target_arch = "x86")]
    let npc_set = unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(NPC_FACE_COORDS)(base) };
    #[cfg(not(target_arch = "x86"))]
    let npc_set = base + 0x108;
    let race_set = race + RACE_FACE_COORDS;
    let mut f: FaceDesc = bytemuck::Zeroable::zeroed();
    f.flags = FACE_GIVEN;
    let gs = added(coords(race_set, 0), coords(npc_set, 0));
    for (to, from) in [(&mut f.geometry_symmetric[..], gs.clone()), (&mut f.geometry_asymmetric[..], added(coords(race_set, 1), coords(npc_set, 1))), (&mut f.texture_symmetric[..], added(coords(race_set, 2), coords(npc_set, 2)))] {
        for (t, v) in to.iter_mut().zip(from) {
            *t = if v.is_finite() { v } else { 0.0 };
        }
    }
    // The eyes: the race's (they turn to look).
    for (side, i) in [LEFT_EYE, RIGHT_EYE].into_iter().enumerate() {
        let model = read_str(mem::deref(race + RACE_FACE_MODELS + i * MODEL_SIZE + 4));
        if !model.is_empty() {
            set_name(&mut f.eyes[side], &format!("oblivion/{}", model_key(&model)));
        }
    }
    // In the race menu the face may be shaped by hand (`crate::facesculpt`), within its sliders.
    if crate::menus::menu_open(crate::menus::MENU_RACE_SEX) {
        f.flags |= FACE_SCULPT;
        f.limits = crate::facesculpt::limits(race);
    }
    let rgb = mem::at::<[u8; 3]>(base + NPC_HAIR_COLOR).unwrap_or([128; 3]);
    f.hair_color = [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0];
    // The age map: FaceGen's age (truncated, as the game's ftol does), the head's texture with M or F
    // and the age after it, or the nearest ten below that the game has.
    let head = read_str(mem::deref(race + RACE_FACE_TEXTURES + 4));
    if let (Some(c), false) = (age_control(), head.is_empty()) {
        let age = c.age.iter().zip(&gs).map(|(a, g)| a * g).sum::<f32>() + c.age_offset;
        let full = format!("textures\\{}", oblivion_assets::export::normalise(&head, "textures\\"));
        let stem = full.strip_suffix(".dds").unwrap_or(&full);
        let sex = if female { 'f' } else { 'm' };
        if (0.0..=100.0).contains(&age) {
            let age = age as u32;
            let tens = (0..=age / 10).rev().map(|t| t * 10);
            if let Some(path) = std::iter::once(age).chain(tens).map(|a| format!("{stem}{sex}{a}.dds")).find(|p| have_texture(&texture_key(p))) {
                let normal = format!("{}_n.dds", path.strip_suffix(".dds").unwrap_or(&path));
                set_name(&mut f.detail_on, &texture_key(&full));
                set_name(&mut f.detail, &texture_key(&path));
                set_name(&mut f.detail_normal, &texture_key(&normal));
            }
        }
    }
    Some(f)
}

static SENT: Mutex<Option<Body>> = Mutex::new(None);
static SENT_FACE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// Every so often in the world (every frame while the race menu is open): the body and the face,
/// each sent again when it has changed.
pub fn publish(a: &rustvr_adapter::Adapter) {
    if let Some(b) = body() {
        let mut sent = SENT.lock().unwrap();
        if sent.as_ref() != Some(&b) {
            let d = avatar(&b);
            tracing::info!(
                "body: {} {} ({}), wearing {:?}, a quiver {:?}, head {:?}, hair {:?}, eyes {:?}: {} parts, {} texture swaps",
                if b.female { "female" } else { "male" },
                b.race,
                if b.beast { "beast" } else { "human" },
                b.worn,
                b.quiver,
                b.face.first().map(|f| &f.1),
                b.hair.as_ref().map(|h| &h.0),
                b.eyes,
                d.count,
                d.swap_count
            );
            if a.write_avatar(&d) {
                *sent = Some(b);
            }
        }
    }
    if let Some(f) = face() {
        let bytes = bytemuck::bytes_of(&f).to_vec();
        let mut sent = SENT_FACE.lock().unwrap();
        if sent.as_ref() != Some(&bytes) {
            // (Said at most every second or so: a slider dragged changes it every frame.)
            static SAID: Mutex<Option<std::time::Instant>> = Mutex::new(None);
            let mut said = SAID.lock().unwrap();
            if said.is_none_or(|t| t.elapsed().as_secs_f32() > 1.0) {
                *said = Some(std::time::Instant::now());
                let (_, detail, _) = f.detail();
                tracing::info!(
                    "body: the face: geometry {:.2?}.., texture {:.2?}.., hair colour {:.2?}, age map {detail:?}",
                    &f.geometry_symmetric[..4],
                    &f.texture_symmetric[..4],
                    &f.hair_color[..3]
                );
            }
            if a.write_face(&f) {
                *sent = Some(bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worn_items_cover_their_slots_and_the_skin_fills_the_rest() {
        let b = Body { race: "Nord".into(), worn: vec![(UPPER_BODY | LOWER_BODY, "clothes\\robe.nif".into()), (1 << 6, "clothes\\ring.nif".into())], ..Default::default() };
        let d = avatar(&b);
        let parts: Vec<&str> = (0..d.count as usize).map(|i| d.part(i)).collect();
        assert_eq!(parts[0], format!("oblivion/{}", model_key("clothes\\robe.nif")));
        assert_eq!(parts[1], format!("oblivion/{}", model_key("clothes\\ring.nif")));
        // Hands and feet are the skin's; the upper and lower body aren't, but are under the robe.
        assert_eq!(parts[4..], [format!("oblivion/{}:under", model_key("characters\\_male\\upperbody.nif")), format!("oblivion/{}:under", model_key("characters\\_male\\lowerbody.nif"))]);
        assert_eq!(parts[2..4], [format!("oblivion/{}", model_key("characters\\_male\\hand.nif")), format!("oblivion/{}", model_key("characters\\_male\\foot.nif"))]);
        assert_eq!(name_str(&d.skeleton), format!("oblivion/{}", model_key("characters\\_male\\skeleton.nif")));
        // A Nord's skin for the Imperial one.
        assert_eq!(d.swap(0), (texture_key("textures\\characters\\imperial\\male\\upperbodymale.dds").as_str(), texture_key("textures\\characters\\nord\\male\\upperbodymale.dds").as_str()));
    }

    #[test]
    fn a_beast_has_its_own_skeleton_and_its_tail() {
        let b = Body { race: "Khajiit".into(), female: true, beast: true, tail: "characters\\khajiit\\tail.nif".into(), ..Default::default() };
        let d = avatar(&b);
        assert_eq!(name_str(&d.skeleton), format!("oblivion/{}", model_key("characters\\_male\\skeletonbeast.nif")));
        assert_eq!(d.count, 5);
        assert_eq!(d.part(4), format!("oblivion/{}", model_key("characters\\khajiit\\tail.nif")));
    }
}
