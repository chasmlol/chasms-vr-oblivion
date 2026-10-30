//! The game's things lying about near the player, for the hands: every one Oblivion lets the player
//! take (weapons, armour, clothes, books, potions, ingredients, keys, soul gems, sigil stones,
//! apparatus, carriable lights, clutter) within reach, offered to the runtime as its own physics
//! props ([`OBJF_GAME_SIMULATED`]): taken in a hand or pulled by the gloves, and taken into the
//! inventory when let go in one of the body's slots ([`OBJF_LOOT`]: the game's own pick-up, with its
//! stealing, its message and its sound).
//!
//! Each is the game's: its collision is its own Havok body's shape ([`SHAPE_HULL`]), its weight its
//! Havok mass, and the game draws it, with its own lighting and effects (a candle in the hand still
//! burns). While a hand has it, the adapter puts the game's object where the runtime has it, as the
//! game's own `SetPos` does (the node moved, its collision told to follow); let go, it's Havok's
//! again at once, its body given the throw's velocity and woken (`hkEntity::activate`). What it is
//! goes to a card beside it ([`ItemInfos`]): its name, its numbers as the game's inventory shows
//! them, and whether it's stolen.
//!
//! Forms (xOBSE's `GameForms.h`): `TESForm` type +4, flags +8 (deleted 0x20, quest item 0x400,
//! disabled 0x800), form id +0xC; `TESObjectREFR` base form +0x1C, 3D +0x3C, extra data +0x44; the
//! form's parts by the game's own dynamic cast (`TESFullName`, `TESValueForm`, `TESWeightForm`...).
//! Cells: `TES` (0xB333A0) grid of loaded exterior cells +8 (`GridCellArray`: size +0xC, cells
//! +0x10, `{cell, info}` each), interior +0x34; a cell's references at +0x48 (`{refr, next}`).

use glam::{Mat3, Quat, Vec3};
use rustvr_protocol::*;

/// Form types the player can take (`TESForm::typeID`).
pub const APPARATUS: u8 = 0x13;
pub const ARMOR: u8 = 0x14;
pub const BOOK: u8 = 0x15;
pub const CLOTHING: u8 = 0x16;
pub const INGREDIENT: u8 = 0x19;
pub const LIGHT: u8 = 0x1A;
pub const MISC: u8 = 0x1B;
pub const WEAPON: u8 = 0x21;
pub const AMMO: u8 = 0x22;
pub const SOUL_GEM: u8 = 0x26;
pub const KEY: u8 = 0x27;
pub const POTION: u8 = 0x28;
pub const SIGIL_STONE: u8 = 0x2A;

pub fn takeable(kind: u8) -> bool {
    matches!(kind, APPARATUS | ARMOR | BOOK | CLOTHING | INGREDIENT | LIGHT | MISC | WEAPON | AMMO | SOUL_GEM | KEY | POTION | SIGIL_STONE)
}

/// Metres around the player things are offered within (the gloves reach about this far), and at
/// most this many, nearest first.
pub const REACH: f32 = 8.0;
pub const MAX_ITEMS: usize = 40;
/// Seconds between looks round the loaded cells.
pub const SCAN_EVERY: f64 = 0.25;
/// Seconds a thing put in a slot may still be in the world before it's offered again (the game
/// didn't take it: a script kept it).
pub const TAKE_WAIT: f64 = 1.0;
/// A thing let go is at rest once it's moved slower than this (m/s) for this long (s); it's
/// given back to the game after this long whatever.
pub const REST_SPEED: f32 = 0.05;
pub const REST_TIME: f64 = 0.4;
pub const MAX_FLIGHT: f64 = 10.0;

/// What the game says about a thing, for its card.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    pub name: String,
    pub kind: u8,
    pub count: u32,
    pub value: Option<i32>,
    pub weight: Option<f32>,
    pub damage: Option<u16>,
    /// `TESObjectARMO::armorRating` (a hundredth of what the inventory shows).
    pub armor: Option<u16>,
    /// Condition, 0..1 (what's left of its health), where it wears.
    pub condition: Option<f32>,
    pub enchanted: bool,
    pub stolen: bool,
    pub quest: bool,
}

/// A weight as the inventory shows it: whole numbers plain, else one place.
pub fn weight_text(w: f32) -> String {
    if (w - w.round()).abs() < 0.05 { format!("{:.0}", w.round()) } else { format!("{w:.1}") }
}

/// The card's contents: the numbers the game's inventory shows for its kind.
pub fn card(id: u32, f: &Facts) -> ItemInfo {
    let mut i = ItemInfo::new(id, if f.name.is_empty() { "Something" } else { &f.name });
    i.count = f.count.max(1);
    if let Some(d) = f.damage.filter(|_| matches!(f.kind, WEAPON | AMMO)) {
        i.push_stat("Damage", &d.to_string());
    }
    if let Some(a) = f.armor.filter(|_| f.kind == ARMOR) {
        i.push_stat("Armor", &format!("{}", a / 100));
    }
    if let Some(w) = f.weight {
        i.push_stat("Weight", &weight_text(w));
    }
    if let Some(v) = f.value {
        i.push_stat("Value", &v.to_string());
    }
    if let Some(c) = f.condition.filter(|_| matches!(f.kind, WEAPON | ARMOR)) {
        i.push_stat("Condition", &format!("{:.0}%", (c * 100.0).clamp(0.0, 100.0)));
    }
    i.flags = if f.stolen { ITEM_STOLEN } else { 0 } | if f.enchanted { ITEM_ENCHANTED } else { 0 } | if f.quest { ITEM_QUEST } else { 0 };
    i
}

/// Gamebryo's `NiMatrix3` (rows) as a matrix, and back.
pub fn ni_matrix(rows: [f32; 9]) -> Mat3 {
    Mat3::from_cols_array(&rows).transpose()
}

pub fn ni_rows(m: Mat3) -> [f32; 9] {
    m.transpose().to_cols_array()
}

/// The local transform that puts a node at `world` (rotation, translation) under a parent at
/// `parent` (rotation, translation, scale).
pub fn local_of(parent: (Mat3, Vec3, f32), world: (Mat3, Vec3)) -> (Mat3, Vec3) {
    let (pr, pt, ps) = parent;
    let ps = if ps.abs() < 1e-6 { 1.0 } else { ps };
    (pr.transpose() * world.0, pr.transpose() * (world.1 - pt) / ps)
}

/// At most `max` of `points` that keep their convex hull's shape: the furthest out along evenly
/// spread directions.
pub fn hull_points(points: &[Vec3], max: usize) -> Vec<Vec3> {
    let mut unique: Vec<Vec3> = Vec::new();
    for p in points {
        if !unique.iter().any(|u| u.distance_squared(*p) < 1e-8) {
            unique.push(*p);
        }
    }
    if unique.len() <= max {
        return unique;
    }
    // A Fibonacci sphere of directions; each one's furthest point.
    let n = max;
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let mut out: Vec<Vec3> = Vec::new();
    for k in 0..n {
        let y = 1.0 - 2.0 * (k as f32 + 0.5) / n as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let d = Vec3::new(r * (golden * k as f32).cos(), y, r * (golden * k as f32).sin());
        let best = unique.iter().copied().max_by(|a, b| a.dot(d).total_cmp(&b.dot(d))).unwrap();
        if !out.iter().any(|o| o.distance_squared(best) < 1e-8) {
            out.push(best);
        }
    }
    out
}

/// The box round `points`: centre and half extents.
pub fn bounds(points: &[Vec3]) -> Option<(Vec3, Vec3)> {
    let lo = points.iter().copied().reduce(Vec3::min)?;
    let hi = points.iter().copied().reduce(Vec3::max)?;
    Some(((lo + hi) * 0.5, (hi - lo) * 0.5))
}

/// A thing's object: the runtime's prop that is the game's thing.
pub fn object_desc(id: u32, seq: u32, at: (Vec3, Quat), centre: Vec3, half: Vec3, mass: f32, hull: bool) -> ObjectDesc {
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_PROP;
    d.flags = OBJF_GRABBABLE | OBJF_PULLABLE | OBJF_GAME_SIMULATED | OBJF_LOOT;
    d.spawn_seq = seq;
    d.spawn_pose = Pose { pos: at.0.to_array(), valid: 1, rot: at.1.to_array() };
    d.shape = if hull { SHAPE_HULL } else { SHAPE_BOX };
    d.center = centre.to_array();
    d.half_extents = half.max(Vec3::splat(0.01)).to_array();
    d.mass = mass;
    d
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, av, refr};
    use crate::{camera, mem};
    use std::collections::HashMap;
    use std::sync::Mutex;

    const TES: usize = 0x00B3_33A0;
    const TES_GRID: usize = 0x8;
    const TES_INTERIOR: usize = 0x34;
    const GRID_SIZE: usize = 0xC;
    const GRID_CELLS: usize = 0x10;
    const GRID_ENTRY: usize = 8;
    const CELL_OBJECTS: usize = 0x48;

    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DELETED: u32 = 0x20;
    const QUEST_ITEM: u32 = 0x400;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const REFR_EXTRA: usize = 0x44;
    /// `TESObjectLIGH` flags: carriable.
    const LIGHT_FLAGS: usize = 0x7C;
    const LIGHT_CAN_CARRY: u32 = 2;
    const ARMOR_RATING: usize = 0xE4;

    /// `NiAVObject`: parent, collision object; `NiNode`: children.
    const NODE_PARENT: usize = 0x1C;
    const NODE_COLLISION: usize = 0xA8;
    const NODE_CHILDREN: usize = 0xB0;
    const NODE_COUNT: usize = 0xB6;
    /// `bhkNiCollisionObject` -> `bhkRigidBody` (+0x10) -> `hkRigidBody` (+8); its motion (+0x50):
    /// inverse mass +0xC0, linear and angular velocity +0xD0, +0xE0 (Havok units: 7 game units).
    /// `hkEntity::activate` (thiscall): its island marked active, so Havok moves it again.
    const COLLISION_BODY: usize = 0x10;
    const HK_OBJ: usize = 0x8;
    const BODY_MOTION: usize = 0x50;
    const MOTION_MASS_INV: usize = 0xC0;
    const MOTION_LIN_VEL: usize = 0xD0;
    const MOTION_ANG_VEL: usize = 0xE0;
    const ACTIVATE: usize = 0x008A_6410;
    /// `hkEntity`'s collision filter info (+0x30): Bethesda's layer its low 7 bits; layer 15 meets
    /// nothing (the player's character controller's casts read it as they go).
    const ENTITY_FILTER: usize = 0x30;
    const LAYER_MASK: u32 = 0x7F;
    const LAYER_NONCOLLIDABLE: u32 = 15;

    /// The game's dynamic cast (cdecl: object, 0, from, to, 0), and the parts' type descriptors.
    const DYNAMIC_CAST: usize = 0x0098_32E6;
    const RTTI_FORM: usize = 0x00B0_2F9C;
    const RTTI_FULL_NAME: usize = 0x00B0_3298;
    const RTTI_VALUE: usize = 0x00B0_5C24;
    const RTTI_WEIGHT: usize = 0x00B0_333C;
    const RTTI_HEALTH: usize = 0x00B0_5CF4;
    const RTTI_DAMAGE: usize = 0x00B0_5D48;
    const RTTI_ENCHANTABLE: usize = 0x00B0_3718;
    const RTTI_SCRIPTABLE: usize = 0x00B0_331C;
    const RTTI_MODEL: usize = 0x00B0_3304;
    /// `TESBipedModelForm` in armour and clothing (+0x64, +0x5C); its male ground model's path (the
    /// model it has lying about: `groundModel[0]` +0x38, its path +4).
    const ARMOR_BIPED: usize = 0x64;
    const CLOTHING_BIPED: usize = 0x5C;
    const GROUND_MODEL_PATH: usize = 0x3C;

    /// `BaseExtraList::GetExtra(type)` and `GetCount` (thiscall); `ExtraHealth` (0x2B) health +0xC.
    const GET_EXTRA: usize = 0x0041_E210;
    const GET_COUNT: usize = 0x0041_E860;
    const EXTRA_HEALTH: u32 = 0x2B;
    /// `TESObjectREFR::GetOwner` and whether an actor counts as its owner (thiscall: actor, 1).
    const GET_OWNER: usize = 0x004D_B6B0;
    const OWNED_BY: usize = 0x004D_E770;
    /// `TESObjectREFR::SetPos(x, y, z)`; the collision under a node told to follow it (cdecl: node,
    /// all below); `NiAVObject::Update(0, 0)` (thiscall). The game's `SetPos` command does these.
    const SET_POS: usize = 0x004D_8A30;
    const RESET_COLLISION: usize = 0x0089_7A20;
    const UPDATE_NODE: usize = 0x0070_7370;
    /// `Actor` virtual 0xB3: takes a reference into the inventory (thiscall: refr, count, 0); the
    /// game's items call it when the player activates them.
    const VT_PICK_UP: usize = 0xB3 * 4;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A0_0000;

    /// The game's dynamic cast of a form to one of its parts (`to`: its type descriptor), or 0.
    pub fn cast(obj: usize, to: usize) -> usize {
        type Cast = unsafe extern "C" fn(usize, u32, usize, usize, u32) -> usize;
        if obj == 0 || !mem::is_readable(obj as *const u8, 8) {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, Cast>(DYNAMIC_CAST)(obj, 0, RTTI_FORM, to, 0) }
    }

    fn extra(refr: usize, kind: u32) -> usize {
        type Get = unsafe extern "thiscall" fn(usize, u32) -> usize;
        unsafe { std::mem::transmute::<usize, Get>(GET_EXTRA)(refr + REFR_EXTRA, kind) }
    }

    fn count(refr: usize) -> u32 {
        type Count = unsafe extern "thiscall" fn(usize) -> i16;
        (unsafe { std::mem::transmute::<usize, Count>(GET_COUNT)(refr + REFR_EXTRA) }).max(1) as u32
    }

    fn stolen(refr: usize) -> bool {
        type Owner = unsafe extern "thiscall" fn(usize) -> usize;
        type OwnedBy = unsafe extern "thiscall" fn(usize, usize, u8) -> u8;
        let owner = unsafe { std::mem::transmute::<usize, Owner>(GET_OWNER)(refr) };
        owner != 0 && unsafe { std::mem::transmute::<usize, OwnedBy>(OWNED_BY)(refr, engine::player(), 1) } == 0
    }

    pub fn facts(refr: usize) -> Facts {
        let base = mem::deref(refr + REFR_BASE);
        let health = cast(base, RTTI_HEALTH);
        let most = (health != 0).then(|| mem::at::<i32>(health + 4)).flatten().filter(|h| *h > 0);
        let now = extra(refr, EXTRA_HEALTH);
        let condition = most.map(|m| if now != 0 { mem::at::<f32>(now + 0xC).unwrap_or(m as f32) / m as f32 } else { 1.0 });
        Facts { count: count(refr), condition, stolen: stolen(refr), ..form_facts(base) }
    }

    /// What the game says about a base form (a thing in an inventory: no reference of its own, at
    /// its full condition), as [`facts`] says it of one lying about.
    pub fn form_facts(base: usize) -> Facts {
        let kind = mem::at::<u8>(base + FORM_TYPE).unwrap_or(0);
        let part = |rtti| cast(base, rtti);
        let name = part(RTTI_FULL_NAME);
        let enchant = part(RTTI_ENCHANTABLE);
        Facts {
            name: if name != 0 { crate::combat::read_str(mem::deref(name + 4)) } else { String::new() },
            kind,
            count: 1,
            value: Some(part(RTTI_VALUE)).filter(|&v| v != 0).and_then(|v| mem::at::<i32>(v + 4)),
            weight: Some(part(RTTI_WEIGHT)).filter(|&w| w != 0).and_then(|w| mem::at::<f32>(w + 4)),
            damage: Some(part(RTTI_DAMAGE)).filter(|&d| d != 0).and_then(|d| mem::at::<u16>(d + 4)),
            armor: (kind == ARMOR).then(|| mem::at::<u16>(base + ARMOR_RATING)).flatten(),
            condition: (part(RTTI_HEALTH) != 0).then_some(1.0),
            enchanted: enchant != 0 && mem::deref(enchant + 4) != 0,
            stolen: false,
            quest: mem::at::<u32>(base + FORM_FLAGS).is_some_and(|f| f & QUEST_ITEM != 0),
        }
    }

    /// The model a base form has lying about: armour's and clothing's ground model, anything
    /// else's own.
    pub fn model_of(base: usize) -> Option<String> {
        let path = match mem::at::<u8>(base + FORM_TYPE)? {
            ARMOR => mem::deref(base + ARMOR_BIPED + GROUND_MODEL_PATH),
            CLOTHING => mem::deref(base + CLOTHING_BIPED + GROUND_MODEL_PATH),
            _ => {
                let model = cast(base, RTTI_MODEL);
                if model == 0 {
                    return None;
                }
                mem::deref(model + 4)
            }
        };
        let s = crate::combat::read_str(path);
        (path != 0 && !s.is_empty()).then_some(s)
    }

    /// The Havok body of a thing's 3D (its root's, or a child's): `hkRigidBody*`, or 0.
    fn body_of(node: usize) -> usize {
        let of = |n: usize| {
            let co = mem::deref(n + NODE_COLLISION);
            let hk = if co != 0 { mem::deref(mem::deref(co + COLLISION_BODY) + HK_OBJ) } else { 0 };
            if hk != 0 && mem::is_readable(hk as *const u8, 0xC4) { hk } else { 0 }
        };
        let root = of(node);
        if root != 0 {
            return root;
        }
        let data = mem::deref(node + NODE_CHILDREN);
        let n = mem::at::<u16>(node + NODE_COUNT).unwrap_or(0).min(32) as usize;
        (0..n).map(|i| mem::deref(data + i * 4)).filter(|&c| c != 0).map(of).find(|&b| b != 0).unwrap_or(0)
    }

    /// A node's world rotation, translation (game units) and scale.
    fn world_of(node: usize) -> Option<(Mat3, Vec3, f32)> {
        let r: [f32; 9] = mem::at(node + av::WORLD_ROTATE)?;
        let t: [f32; 3] = mem::at(node + av::WORLD_TRANSLATE)?;
        let s: f32 = mem::at(node + av::WORLD_SCALE)?;
        let m = ni_matrix(r);
        (m.is_finite() && Vec3::from(t).is_finite()).then_some((m, Vec3::from(t), s))
    }

    /// Where a thing is, in the runtime's world.
    fn pose_of(node: usize) -> Option<(Vec3, Quat)> {
        let (m, t, _) = world_of(node)?;
        let b = camera::basis();
        Some((b.point_from_game(t), b.rot_from_game(m).normalize()))
    }

    /// Puts a thing's 3D and its collision at a pose in the runtime's world, as `SetPos` does (the
    /// game draws a thing with a body where its body is).
    fn put(node: usize, pos: Vec3, rot: Quat) -> bool {
        type Reset = unsafe extern "C" fn(usize, u8);
        type Update = unsafe extern "thiscall" fn(usize, f32, u32) -> u32;
        if !mem::is_readable(node as *const u8, 0xB8) {
            return false;
        }
        let b = camera::basis();
        let parent = mem::deref(node + NODE_PARENT);
        let p = (parent != 0).then(|| world_of(parent)).flatten().unwrap_or((Mat3::IDENTITY, Vec3::ZERO, 1.0));
        let (r, t) = local_of(p, (b.rot_to_game(rot), b.point_to_game(pos)));
        if !r.is_finite() || !t.is_finite() {
            return false;
        }
        unsafe {
            ((node + av::LOCAL_ROTATE) as *mut [f32; 9]).write_unaligned(ni_rows(r));
            ((node + av::LOCAL_TRANSLATE) as *mut [f32; 3]).write_unaligned(t.to_array());
            std::mem::transmute::<usize, Reset>(RESET_COLLISION)(node, 1);
            std::mem::transmute::<usize, Update>(UPDATE_NODE)(node, 0.0, 0);
        }
        true
    }

    /// Sets a Havok body moving (the runtime's world, metres and radians a second) and wakes it.
    fn launch(body: usize, lin: Vec3, ang: Vec3) {
        let motion = mem::deref(body + BODY_MOTION);
        if motion == 0 || !mem::is_readable((motion + MOTION_LIN_VEL) as *const u8, 32) {
            return;
        }
        let b = camera::basis();
        let v = b.point_to_game(lin) / crate::collision::HAVOK_SCALE;
        let w = b.dir_to_game(ang);
        type Activate = unsafe extern "thiscall" fn(usize) -> u32;
        unsafe {
            ((motion + MOTION_LIN_VEL) as *mut [f32; 4]).write_unaligned([v.x, v.y, v.z, 0.0]);
            ((motion + MOTION_ANG_VEL) as *mut [f32; 4]).write_unaligned([w.x, w.y, w.z, 0.0]);
            std::mem::transmute::<usize, Activate>(ACTIVATE)(body);
        }
    }

    fn set_filter(body: usize, filter: u32) {
        if mem::is_readable((body + ENTITY_FILTER) as *const u8, 4) {
            unsafe { ((body + ENTITY_FILTER) as *mut u32).write(filter) };
        }
    }

    fn body_speed(body: usize) -> Option<f32> {
        let v: [f32; 4] = mem::at(mem::deref(body + BODY_MOTION) + MOTION_LIN_VEL)?;
        Some(Vec3::new(v[0], v[1], v[2]).length() * crate::collision::HAVOK_SCALE / camera::UNITS_PER_METER)
    }

    /// The player takes it, as activating it does (the game's own pick-up: stealing, the message,
    /// the sound); one with a script is activated by the player, its script deciding.
    pub fn take(refr: usize) -> bool {
        let base = mem::deref(refr + REFR_BASE);
        let scriptable = cast(base, RTTI_SCRIPTABLE);
        if scriptable != 0 && mem::deref(scriptable + 4) != 0 {
            return crate::world::activate(refr);
        }
        type PickUp = unsafe extern "thiscall" fn(usize, usize, i32, i32) -> i32;
        let player = engine::player();
        let f = mem::deref(mem::deref(player) + VT_PICK_UP);
        if player == 0 || !CODE.contains(&f) {
            return false;
        }
        unsafe { std::mem::transmute::<usize, PickUp>(f)(player, refr, count(refr) as i32, 0) };
        true
    }

    /// The loaded cells near the player: the interior, or the exterior grid's middle 3 by 3.
    pub(crate) fn cells() -> Vec<usize> {
        let tes = mem::deref(TES);
        if tes == 0 {
            return Vec::new();
        }
        let interior = mem::deref(tes + TES_INTERIOR);
        if interior != 0 {
            return vec![interior];
        }
        let grid = mem::deref(tes + TES_GRID);
        let size = mem::at::<u32>(grid + GRID_SIZE).unwrap_or(0).min(16) as usize;
        let entries = mem::deref(grid + GRID_CELLS);
        if size == 0 || entries == 0 {
            return Vec::new();
        }
        let c = size / 2;
        let mut out = Vec::new();
        for x in c.saturating_sub(1)..(c + 2).min(size) {
            for y in c.saturating_sub(1)..(c + 2).min(size) {
                let cell = mem::deref(entries + (x * size + y) * GRID_ENTRY);
                if cell != 0 {
                    out.push(cell);
                }
            }
        }
        out
    }

    /// A reference the player could take now: its base form's kind, if so.
    pub fn takeable_ref(refr: usize) -> Option<u8> {
        let flags = mem::at::<u32>(refr + FORM_FLAGS)?;
        if flags & (DELETED | DISABLED) != 0 || mem::deref(refr + REFR_NODE) == 0 {
            return None;
        }
        let base = mem::deref(refr + REFR_BASE);
        let kind = mem::at::<u8>(base + FORM_TYPE)?;
        if !takeable(kind) || (kind == LIGHT && mem::at::<u32>(base + LIGHT_FLAGS).is_none_or(|f| f & LIGHT_CAN_CARRY == 0)) {
            return None;
        }
        Some(kind)
    }

    /// The takeable things within `reach` metres of `at` (runtime world), nearest first:
    /// (reference, form id, distance).
    fn nearby(at: Vec3, reach: f32) -> Vec<(usize, u32, f32)> {
        let b = camera::basis();
        let mut out = Vec::new();
        for cell in cells() {
            let mut entry = cell + CELL_OBJECTS;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                if r == 0 || takeable_ref(r).is_none() {
                    continue;
                }
                let Some(p) = mem::at::<[f32; 3]>(r + refr::POS) else { continue };
                let d = b.point_from_game(Vec3::from(p)).distance(at);
                if d <= reach
                    && let Some(id) = mem::at::<u32>(r + FORM_ID).filter(|&id| id != 0)
                {
                    out.push((r, id, d));
                }
            }
        }
        out.sort_by(|a, b| a.2.total_cmp(&b.2));
        out
    }

    struct Item {
        refr: usize,
        node: usize,
        body: usize,
        desc: ObjectDesc,
        info: ItemInfo,
        hull: Vec<Vec3>,
        /// A hand has it (held or pulled): the game's object goes where the runtime has it.
        owned: bool,
        /// Its body's own collision filter, while a hand has it (it's out of the way meanwhile).
        filter: Option<u32>,
        /// Let go and still moving (since when, still since when): the runtime simulates it, on the
        /// world's collision, and the game's object goes where it has it until it comes to rest.
        flying: Option<(f64, Option<f64>)>,
        /// Put in a slot and taken (when).
        taken_at: Option<f64>,
        /// The game's sound pack's set it sounds like ([`crate::sounds::of_thing`]).
        sound: Option<String>,
    }

    struct State {
        items: HashMap<u32, Item>,
        /// The ids and spawn numbers offered, as last published.
        sent: Vec<(u32, u32)>,
        /// Spawn numbers for things offered again (the runtime starts them afresh).
        seqs: HashMap<u32, u32>,
        next_scan: f64,
        to_take: Vec<u32>,
        states: Box<ObjectStates>,
        poses: Box<GamePoses>,
        hulls: Box<GameHulls>,
        infos: Box<ItemInfos>,
        logged: f64,
        /// Things just let go: logged as they fly (until when, next log).
        trace: Vec<(u32, f64, f64)>,
        /// What the hands hold now (as the runtime last said).
        held: Vec<Held>,
    }

    /// A thing in a hand: its object, the game's reference, its kind and the hand.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Held {
        pub id: u32,
        pub refr: usize,
        pub kind: u8,
        pub hand: usize,
    }

    /// What the hands hold now.
    pub fn held() -> Vec<Held> {
        with_state(|s| s.held.clone())
    }

    /// The middle of what's seen of a thing (its model's bound) in its own space (its object's: the
    /// runtime's axes, metres), by its reference.
    pub fn visual_centre(refr: usize) -> Option<Vec3> {
        let node = mem::deref(refr + REFR_NODE);
        // (As its collision is put in its own space: world size, its turn undone.)
        let (m, t, _) = world_of(node)?;
        let b: [f32; 4] = mem::at(node + 0x20)?;
        let c = camera::basis().point_from_game(m.transpose() * (Vec3::new(b[0], b[1], b[2]) - t));
        (c.is_finite() && c.length() < 1.0).then_some(c)
    }

    /// The object of a thing lying about that's offered to the hands, by its reference.
    pub fn offered(refr: usize) -> Option<u32> {
        with_state(|s| s.items.values().find(|i| i.refr == refr && i.taken_at.is_none()).map(|i| i.desc.id))
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State {
            items: HashMap::new(),
            sent: Vec::new(),
            seqs: HashMap::new(),
            next_scan: 0.0,
            to_take: Vec::new(),
            states: bytemuck::zeroed_box(),
            poses: bytemuck::zeroed_box(),
            hulls: bytemuck::zeroed_box(),
            infos: bytemuck::zeroed_box(),
            logged: 0.0,
            trace: Vec::new(),
            held: Vec::new(),
        }))
    }

    /// A thing of base form `base` just put in the world at `at` (the runtime's world): set moving
    /// at `lin` and turning at `ang` (m/s, rad/s, the runtime's world), as a thing let go of is, once
    /// the game has given it its 3D and its Havok body. False until then.
    pub fn throw_new(base: usize, at: Vec3, lin: Vec3, ang: Vec3) -> bool {
        let found = nearby(at, 0.6).into_iter().find(|&(r, _, _)| mem::deref(r + REFR_BASE) == base);
        let Some((refr, _, _)) = found else { return false };
        let body = body_of(mem::deref(refr + REFR_NODE));
        if body == 0 {
            return false;
        }
        launch(body, lin, ang);
        true
    }

    /// The reference of the thing offered as object `id`, while it's offered.
    pub fn reference(id: u32) -> Option<usize> {
        with_state(|s| s.items.get(&id).filter(|i| i.taken_at.is_none()).map(|i| i.refr))
    }

    /// A thing as an object: its pose, its Havok shape in its own space, its mass, its card.
    fn make(refr: usize, id: u32, seq: u32) -> Option<Item> {
        let node = mem::deref(refr + REFR_NODE);
        let body = body_of(node);
        let (m, t, _) = world_of(node)?;
        let at = pose_of(node)?;
        let b = camera::basis();
        // Its collision, in its own space (the runtime's axes, metres).
        let tris = if body != 0 { crate::collision::body_triangles(body) } else { Vec::new() };
        let local: Vec<Vec3> = tris.iter().flatten().map(|w| b.point_from_game(m.transpose() * (*w - t))).filter(|p| p.is_finite() && p.length() < 5.0).collect();
        let hull = hull_points(&local, MAX_HULL_POINTS);
        let (centre, half) = bounds(&hull).unwrap_or((Vec3::ZERO, Vec3::splat(0.05)));
        let f = facts(refr);
        let mass_inv = mem::at::<f32>(mem::deref(body + BODY_MOTION) + MOTION_MASS_INV).filter(|m| *m > 1e-4 && m.is_finite());
        // (The game's weight is about pounds: a kilogram is two.)
        let mass = mass_inv.map(|m| 1.0 / m).or(f.weight.map(|w| w * 0.45)).unwrap_or(1.0).clamp(0.05, 60.0);
        let desc = object_desc(id, seq, at, centre, half, mass, hull.len() >= 4);
        // (The game's collision sounds go by its Havok mass, 5 without a body.)
        let sound = crate::sounds::of_thing(mem::deref(refr + REFR_BASE), body, mass_inv.map_or(5.0, |m| 1.0 / m));
        Some(Item { refr, node, body, desc, info: card(id, &f), hull: if hull.len() >= 4 { hull } else { Vec::new() }, owned: false, filter: None, flying: None, taken_at: None, sound })
    }

    /// Once a game frame, where its scripts run: things put in a slot taken into the inventory;
    /// a look round for what's near, offered when that changes.
    pub fn frame(now: f64) {
        let player = engine::player();
        if player == 0 || mem::deref(player + REFR_NODE) == 0 {
            return;
        }
        with_state(|s| {
            // Put in a slot: the player takes it.
            for id in std::mem::take(&mut s.to_take) {
                let Some(item) = s.items.get_mut(&id) else { continue };
                if item.taken_at.is_some() {
                    continue;
                }
                let ok = takeable_ref(item.refr).is_some() && take(item.refr);
                tracing::info!("items: {} ({id:#010x}) put in a slot: {}", item.info.name(), if ok { "taken" } else { "not taken" });
                item.taken_at = Some(now);
            }
            if now < s.next_scan {
                return;
            }
            s.next_scan = now + SCAN_EVERY;
            let Some(me) = mem::at::<[f32; 3]>(player + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p))) else { return };
            let near = nearby(me, REACH);

            let mut keep: Vec<u32> = Vec::new();
            for &(r, id, _) in near.iter().take(MAX_ITEMS) {
                keep.push(id);
                match s.items.get(&id) {
                    Some(it) if it.refr == r && it.taken_at.is_none() => {}
                    Some(it) if it.refr == r => {
                        // Still there after it was put in a slot: offered again, afresh.
                        if now - it.taken_at.unwrap() > TAKE_WAIT {
                            let seq = s.seqs.entry(id).or_insert(1);
                            *seq += 1;
                            let seq = *seq;
                            if let Some(item) = make(r, id, seq) {
                                tracing::info!("items: {} wasn't taken: offered again", item.info.name());
                                s.items.insert(id, item);
                            }
                        }
                    }
                    _ => {
                        let seq = *s.seqs.get(&id).unwrap_or(&1);
                        if let Some(item) = make(r, id, seq) {
                            s.items.insert(id, item);
                        }
                    }
                }
            }
            // Gone (taken, moved off, out of reach), unless a hand has it.
            s.items.retain(|id, it| {
                let there = mem::at::<u32>(it.refr + FORM_ID) == Some(*id) && takeable_ref(it.refr).is_some() && mem::deref(it.refr + REFR_NODE) == it.node;
                there && (keep.contains(id) || it.owned || it.flying.is_some()) && !(it.taken_at.is_some() && !keep.contains(id))
            });
            let mut ids: Vec<(u32, u32)> = s.items.values().map(|i| (i.desc.id, i.desc.spawn_seq)).collect();
            ids.sort();
            if ids != s.sent {
                publish(s, now);
                s.sent = ids;
            }
        });
    }

    fn publish(s: &mut State, now: f64) {
        let items: Vec<&Item> = {
            let mut v: Vec<&Item> = s.items.values().collect();
            v.sort_by_key(|i| i.desc.id);
            v
        };
        let me = mem::at::<[f32; 3]>(engine::player() + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p))).unwrap_or_default();
        let mut by_distance = items.clone();
        by_distance.sort_by(|a, b| Vec3::from(a.desc.spawn_pose.pos).distance(me).total_cmp(&Vec3::from(b.desc.spawn_pose.pos).distance(me)));
        s.hulls.count = 0;
        for it in items.iter().filter(|i| !i.hull.is_empty()).take(MAX_GAME_HULLS) {
            let h = &mut s.hulls.hulls[s.hulls.count as usize];
            h.id = it.desc.id;
            h.count = it.hull.len().min(MAX_HULL_POINTS) as u32;
            for (k, p) in it.hull.iter().take(MAX_HULL_POINTS).enumerate() {
                h.points[k] = [p.x, p.y, p.z, 0.0];
            }
            s.hulls.count += 1;
        }
        s.infos.count = 0;
        for it in items.iter().take(MAX_ITEM_INFOS) {
            s.infos.infos[s.infos.count as usize] = it.info;
            s.infos.count += 1;
        }
        let descs: Vec<ObjectDesc> = items.iter().map(|i| i.desc).collect();
        let sounds: Vec<(u32, String)> = items.iter().filter_map(|i| Some((i.desc.id, i.sound.clone()?))).collect();
        let hulls = &s.hulls.hulls[..s.hulls.count as usize];
        let infos = &s.infos.infos[..s.infos.count as usize];
        let ok = crate::link::with_adapter(|a| crate::objects::publish_hulls(crate::objects::Owner::Items, hulls, a) && crate::objects::publish_infos(crate::objects::Owner::Items, infos, a) && crate::objects::publish(crate::objects::Owner::Items, &descs, &sounds, a)).unwrap_or(false);
        if ok {
            s.logged = now;
            tracing::info!(
                "items: {} offered: {}",
                items.len(),
                by_distance.iter().take(6).map(|i| format!("{} {:#x} ({} pts, {:.2} kg, at {:.2?})", i.info.name(), i.desc.id, i.hull.len(), i.desc.mass, i.desc.spawn_pose.pos)).collect::<Vec<_>>().join(", ")
            );
        }
    }

    /// Before the eyes are drawn: the things a hand has, where the runtime has them; let go, the
    /// game's again with the throw; where the rest are, for the runtime.
    pub fn place(view: &rustvr_adapter::View, now: f64) {
        with_state(|s| {
            if s.items.is_empty() {
                s.held.clear();
                return;
            }
            if !crate::link::with_adapter(|a| a.read_object_states(&mut s.states)).unwrap_or(false) {
                return;
            }
            let n = (s.states.count as usize).min(MAX_OBJECTS);
            s.held = s.states.states[..n]
                .iter()
                .filter(|st| st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0)
                .filter_map(|st| {
                    let it = s.items.get(&st.id).filter(|it| it.taken_at.is_none())?;
                    let kind = mem::at::<u8>(mem::deref(it.refr + REFR_BASE) + FORM_TYPE).unwrap_or(0);
                    Some(Held { id: st.id, refr: it.refr, kind, hand: if st.flags & OS_HELD_LEFT != 0 { 0 } else { 1 } })
                })
                .collect();
            for st in &s.states.states[..n] {
                let Some(it) = s.items.get_mut(&st.id) else { continue };
                if it.taken_at.is_some() {
                    continue;
                }
                if st.flags & OS_STOWED != 0 {
                    if let Some(f) = it.filter.take() {
                        set_filter(it.body, f);
                    }
                    if !s.to_take.contains(&st.id) {
                        s.to_take.push(st.id);
                    }
                    continue;
                }
                let tracking = st.flags & OS_TRACKING != 0;
                let (pos, rot) = if tracking { view.pose(&st.pose) } else { (Vec3::from_array(st.pose.pos), Quat::from_array(st.pose.rot)) };
                if !pos.is_finite() || !rot.is_finite() || rot.length_squared() < 1e-6 {
                    continue;
                }
                let in_hand = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT | OS_PULLED) != 0;
                if in_hand {
                    if !it.owned {
                        tracing::info!("items: {} taken in hand ({:#x})", it.info.name(), st.flags);
                    }
                    if !it.owned && it.body != 0 {
                        // Out of everything's way while a hand has it (the player walks into it
                        // otherwise): Havok's layer for things nothing meets.
                        it.filter = mem::at::<u32>(it.body + ENTITY_FILTER);
                        if let Some(f) = it.filter {
                            set_filter(it.body, (f & !LAYER_MASK) | LAYER_NONCOLLIDABLE);
                        }
                    }
                    it.owned = true;
                    put(it.node, pos, rot.normalize());
                } else if it.owned && it.body != 0 {
                    // Let go: Havok's again at once, moving as it left the hand.
                    it.owned = false;
                    let dir = |v: [f32; 3]| if tracking { view.dir(Vec3::from_array(v)) } else { Vec3::from_array(v) };
                    let (v, w) = (dir(st.lin_vel), dir(st.ang_vel));
                    put(it.node, pos, rot.normalize());
                    if let Some(f) = it.filter.take() {
                        set_filter(it.body, f);
                    }
                    let g = camera::basis().point_to_game(pos);
                    type SetPos = unsafe extern "thiscall" fn(usize, f32, f32, f32);
                    unsafe { std::mem::transmute::<usize, SetPos>(SET_POS)(it.refr, g.x, g.y, g.z) };
                    launch(it.body, v, w);
                    tracing::info!("items: {} let go at {pos:.2?}, {:.2} m/s: Havok's", it.info.name(), v.length());
                    s.trace.push((st.id, now + 1.0, now));
                } else if it.owned || it.flying.is_some() {
                    // Let go of one without a Havok body: it flies on in the runtime (the game's
                    // object where it has it) until it comes to rest, then it's the game's again.
                    let speed = Vec3::from_array(st.lin_vel).length();
                    if it.owned {
                        it.owned = false;
                        it.flying = Some((now, None));
                        tracing::info!("items: {} let go at {pos:.2?}, {speed:.2} m/s", it.info.name());
                    }
                    put(it.node, pos, rot.normalize());
                    let (since, still) = it.flying.unwrap();
                    let still = if speed < REST_SPEED && Vec3::from_array(st.ang_vel).length() < 0.5 { Some(still.unwrap_or(now)) } else { None };
                    if still.is_some_and(|t| now - t > REST_TIME) || now - since > MAX_FLIGHT {
                        it.flying = None;
                        let g = camera::basis().point_to_game(pos);
                        type SetPos = unsafe extern "thiscall" fn(usize, f32, f32, f32);
                        unsafe { std::mem::transmute::<usize, SetPos>(SET_POS)(it.refr, g.x, g.y, g.z) };
                        tracing::info!("items: {} came to rest at {pos:.2?}: the game's again", it.info.name());
                    } else {
                        it.flying = Some((since, still));
                    }
                }
            }
            // Just let go: how the game's body flies (a second of it, in the log).
            s.trace.retain_mut(|(id, until, next)| {
                let Some(it) = s.items.get(id).filter(|_| now < *until) else { return false };
                if now >= *next {
                    *next = now + 0.1;
                    let at = pose_of(it.node).map(|p| p.0);
                    let v = if it.body != 0 { body_speed(it.body) } else { None };
                    tracing::info!("items: {} flying at {at:.2?}, Havok has it moving {v:.2?} m/s", it.info.name());
                }
                true
            });
            // The game's card of a thing is the held things'.
            let held = s.items.iter().filter(|(_, i)| i.owned && i.taken_at.is_none()).map(|(&id, i)| crate::hud_info::Thing { key: id, refr: i.refr, from: 0, world: true });
            crate::hud_info::set_world(held.collect());
            s.poses.count = 0;
            for it in s.items.values().filter(|i| !i.owned && i.flying.is_none() && i.taken_at.is_none()) {
                let i = s.poses.count as usize;
                if i >= MAX_OBJECTS {
                    break;
                }
                if let Some((p, q)) = pose_of(it.node) {
                    s.poses.poses[i] = GamePose { id: it.desc.id, pose: Pose { pos: p.to_array(), valid: 1, rot: q.to_array() }, ..Default::default() };
                    s.poses.count += 1;
                }
            }
            let poses = &s.poses.poses[..s.poses.count as usize];
            crate::link::with_adapter(|a| crate::objects::publish_poses(crate::objects::Owner::Items, poses, a));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_show_what_the_inventory_shows() {
        let sword = Facts { name: "Iron Longsword".into(), kind: WEAPON, count: 1, value: Some(25), weight: Some(22.0), damage: Some(9), condition: Some(0.85), ..Default::default() };
        let c = card(7, &sword);
        assert_eq!(c.name(), "Iron Longsword");
        assert_eq!(c.stats().collect::<Vec<_>>(), [("Damage", "9"), ("Weight", "22"), ("Value", "25"), ("Condition", "85%")]);
        let cuirass = Facts { name: "Iron Cuirass".into(), kind: ARMOR, armor: Some(500), weight: Some(30.0), value: Some(50), stolen: true, ..Default::default() };
        let c = card(8, &cuirass);
        assert_eq!(c.stats().next(), Some(("Armor", "5")));
        assert!(c.flags & ITEM_STOLEN != 0 && c.count == 1);
        let arrows = Facts { name: "Iron Arrow".into(), kind: AMMO, count: 12, damage: Some(6), weight: Some(0.1), ..Default::default() };
        let c = card(9, &arrows);
        assert_eq!(c.count, 12);
        assert_eq!(c.stats().collect::<Vec<_>>(), [("Damage", "6"), ("Weight", "0.1")]);
        // A thing that isn't a weapon has no damage shown, whatever its parts say.
        let apple = Facts { name: "Apple".into(), kind: INGREDIENT, damage: Some(3), weight: Some(0.1), ..Default::default() };
        assert_eq!(card(10, &apple).stats().collect::<Vec<_>>(), [("Weight", "0.1")]);
    }

    #[test]
    fn a_node_goes_where_it_is_asked_under_its_parent() {
        let parent = (Mat3::from_rotation_z(0.7), Vec3::new(10.0, -4.0, 2.0), 2.0);
        let world = (Mat3::from_rotation_x(0.3) * Mat3::from_rotation_z(-1.1), Vec3::new(3.0, 5.0, 7.0));
        let (r, t) = local_of(parent, world);
        // The parent's world times the local is the world asked for.
        assert!((parent.0 * r - world.0).abs_diff_eq(Mat3::ZERO, 1e-5));
        assert!((parent.0 * (t * parent.2) + parent.1).abs_diff_eq(world.1, 1e-4));
        // Gamebryo's rows round trip.
        let m = Mat3::from_rotation_y(0.4);
        assert!(ni_matrix(ni_rows(m)).abs_diff_eq(m, 1e-6));
        assert_eq!(ni_rows(Mat3::from_cols(Vec3::X, Vec3::Z, -Vec3::Y))[1], 0.0);
    }

    #[test]
    fn hulls_keep_their_extremes_and_fit() {
        // A dense ball of points: cut down to the limit, its extent kept.
        let mut pts = Vec::new();
        for i in 0..40 {
            for j in 0..20 {
                let (a, b) = (i as f32 * 0.157, j as f32 * 0.157);
                pts.push(Vec3::new(a.cos() * b.sin(), b.cos(), a.sin() * b.sin()) * 0.1);
            }
        }
        let h = hull_points(&pts, 64);
        assert!(h.len() <= 64 && h.len() > 20);
        let (c, half) = bounds(&h).unwrap();
        assert!(c.length() < 0.02 && (half.y - 0.1).abs() < 0.01, "{c:?} {half:?}");
        assert_eq!(hull_points(&[Vec3::ZERO, Vec3::ZERO, Vec3::X], 64).len(), 2, "duplicates go");
    }
}
