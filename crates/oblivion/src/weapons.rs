//! The player's weapon, in the hand: the game's equipped weapon as an object the runtime draws (the
//! game's own model, `oblivion-extract export meshes/weapons` into `local_assets/oblivion`, named
//! by the hash of the path the weapon's form gives, `oblivion_assets::export::model_key`) and the
//! hands hold. It starts sheathed where the game wears it (a one-handed weapon at the left hip, a
//! two-handed one on the back) and the hand draws it from there; its cutting edges, point and
//! pommel are measured on the model ([`melee_from_model`]), and what they hit is the game's hit
//! with the equipped weapon (`crate::combat::apply`): the game's damage, skill and enchantment.
//!
//! Forms: `MiddleHighProcess::equippedWeaponData` (+0xE4 of the player's process) is the
//! inventory entry of the equipped weapon (its form at +8). `TESObjectWEAP` (xOBSE's
//! `GameForms.h`): its name at +0x28, its model's path at +0x34, its weight at +0x7C, its damage at
//! +0x8C, its type, speed and reach at +0x90..+0x98.

use crate::{engine, mem};
use glam::{Mat3, Quat, Vec3};
use rustvr_protocol::*;
use std::sync::Mutex;

/// The weapon's object id (stable while it's equipped).
pub const WEAPON_ID: u32 = 0x0B1E_0001;

/// `Actor::process` and `MiddleHighProcess::equippedWeaponData`; an inventory entry's form.
const PROCESS: usize = 0x58;
const EQUIPPED_WEAPON: usize = 0xE4;
const ENTRY_FORM: usize = 0x08;
/// `TESObjectWEAP` fields.
const WEAP_NAME: usize = 0x28;
const WEAP_MODEL: usize = 0x34;
const WEAP_WEIGHT: usize = 0x7C;
const WEAP_DAMAGE: usize = 0x8C;
const WEAP_TYPE: usize = 0x90;
const WEAP_REACH: usize = 0x98;

/// `TESObjectWEAP::type`.
const BLADE_ONE_HAND: u32 = 0;
const BLADE_TWO_HAND: u32 = 1;
const BLUNT_ONE_HAND: u32 = 2;
const BLUNT_TWO_HAND: u32 = 3;
const STAFF: u32 = 4;

/// The equipped weapon as the game has it.
#[derive(Clone, Debug, PartialEq)]
pub struct Equipped {
    pub form: usize,
    pub name: String,
    pub model: String,
    pub kind: u32,
    pub damage: u16,
    pub reach: f32,
    pub weight: f32,
}

/// The player's equipped weapon, if any.
pub fn equipped() -> Option<Equipped> {
    let player = engine::player();
    let process = mem::deref(player + PROCESS);
    let entry = mem::deref(process + EQUIPPED_WEAPON);
    let form = mem::deref(entry + ENTRY_FORM);
    if player == 0 || process == 0 || entry == 0 || form == 0 {
        return None;
    }
    // (A form whose type byte isn't a weapon's is something else: TESForm::typeID at +4, 0x21.)
    if mem::at::<u8>(form + 4) != Some(0x21) {
        return None;
    }
    Some(Equipped {
        form,
        name: crate::combat::read_str(mem::deref(form + WEAP_NAME)),
        model: crate::combat::read_str(mem::deref(form + WEAP_MODEL)),
        kind: mem::at(form + WEAP_TYPE)?,
        damage: mem::at(form + WEAP_DAMAGE)?,
        reach: mem::at(form + WEAP_REACH)?,
        weight: mem::at(form + WEAP_WEIGHT)?,
    })
}

/// The controller grip rotation for a fist round a handle along `tube` (little finger to thumb),
/// the back of the hand towards `x` (`rustvr_guns::fist`, kept here to spare the DLL the avatar).
fn fist(tube: Vec3, x: Vec3) -> Quat {
    let z = tube.normalize();
    let x = (x - z * x.dot(z)).normalize();
    Quat::from_mat3(&Mat3::from_cols(x, z.cross(x), z)).normalize()
}

/// A weapon model's extent along its length, slice by slice (a centimetre each): the least and
/// most x and y of the vertices in it.
struct Slices {
    z0: f32,
    lo: Vec<[f32; 2]>,
    hi: Vec<[f32; 2]>,
}

const SLICE: f32 = 0.01;

impl Slices {
    fn of(model: &rustvr_asset::Model) -> Slices {
        let (z0, z1) = (model.bounds_min[2], model.bounds_max[2]);
        let n = (((z1 - z0) / SLICE).ceil() as usize).max(1);
        let mut s = Slices { z0, lo: vec![[f32::MAX; 2]; n], hi: vec![[f32::MIN; 2]; n] };
        for v in model.meshes.iter().flat_map(|m| &m.vertices) {
            let i = (((v.pos[2] - z0) / SLICE) as usize).min(n - 1);
            for k in 0..2 {
                s.lo[i][k] = s.lo[i][k].min(v.pos[k]);
                s.hi[i][k] = s.hi[i][k].max(v.pos[k]);
            }
        }
        s
    }

    fn z(&self, i: usize) -> f32 {
        self.z0 + (i as f32 + 0.5) * SLICE
    }

    /// Width across x of slice `i` (0 where it's empty).
    fn width(&self, i: usize) -> f32 {
        (self.hi[i][0] - self.lo[i][0]).max(0.0)
    }
}

/// Where a weapon of the game's `kind` hurts, measured on its model (grip at the origin, its length
/// along +Z, a blade's edges to ±X, as Oblivion's weapon models are):
/// * a blade cuts with both edges from its guard (the widest slice near the handle) to its point,
///   stabs with its last 10 cm, and the pommel bludgeons;
/// * a blunt weapon's head (from where it grows past twice the handle's width to its end)
///   bludgeons, and an axe's (a head that reaches out to one side) cuts with its outer edge;
/// * a staff bludgeons with its top fifth.
pub fn melee_from_model(model: &rustvr_asset::Model, kind: u32, id: u32, damage: f32) -> MeleeDesc {
    let s = Slices::of(model);
    let n = s.lo.len();
    let (z_lo, z_hi) = (model.bounds_min[2], model.bounds_max[2]);
    let mut w = MeleeDesc { id, damage, reach: z_hi, ..Default::default() };
    let mut add = |d: Damager| {
        if (w.count as usize) < MAX_DAMAGERS {
            w.damagers[w.count as usize] = d;
            w.count += 1;
        }
    };
    let pommel = Damager { kind: DMG_BLUNT, radius: 0.025, a: [0.0, 0.0, z_lo], b: [0.0, 0.0, z_lo + 0.03], ..Default::default() };
    match kind {
        BLADE_ONE_HAND | BLADE_TWO_HAND => {
            // The guard: the widest slice from the pommel to 40% of the way up.
            let guard = (0..n).filter(|&i| s.z(i) <= z_lo + (z_hi - z_lo) * 0.4).max_by(|&a, &b| s.width(a).total_cmp(&s.width(b))).map_or(0.0, |i| s.z(i));
            let from = (guard + 0.03).max(0.0);
            add(Damager { kind: DMG_SLASH, flags: DMGF_BOTH_WAYS, radius: 0.02, a: [0.0, 0.0, from], b: [0.0, 0.0, z_hi - 0.03], edge: [1.0, 0.0, 0.0], ..Default::default() });
            add(Damager { kind: DMG_PIERCE, radius: 0.012, depth: 0.25, a: [0.0, 0.0, z_hi - 0.1], b: [0.0, 0.0, z_hi], ..Default::default() });
            add(pommel);
        }
        BLUNT_ONE_HAND | BLUNT_TWO_HAND => {
            let mut handle: Vec<f32> = (0..n / 2).map(|i| s.width(i)).filter(|&x| x > 0.0).collect();
            handle.sort_by(f32::total_cmp);
            let handle = handle.get(handle.len() / 2).copied().unwrap_or(0.03);
            let head = (n / 2..n).find(|&i| s.width(i) > handle * 2.0).unwrap_or(n * 3 / 4);
            let head_z = s.z(head);
            let (mut x_lo, mut x_hi, mut y_half) = (0f32, 0f32, 0f32);
            for i in head..n {
                if s.width(i) > 0.0 {
                    x_lo = x_lo.min(s.lo[i][0]);
                    x_hi = x_hi.max(s.hi[i][0]);
                    y_half = y_half.max(s.hi[i][1].abs().max(s.lo[i][1].abs()));
                }
            }
            let centre = (x_lo + x_hi) * 0.5;
            let axe = x_hi - x_lo > 0.06 && centre.abs() > (x_hi - x_lo) * 0.2;
            if axe {
                // The cutting edge, along the far side of the head.
                let side = centre.signum();
                let x = if side > 0.0 { x_hi } else { x_lo } * 0.85;
                add(Damager { kind: DMG_SLASH, radius: 0.02, a: [x, 0.0, head_z], b: [x, 0.0, z_hi - 0.01], edge: [side, 0.0, 0.0], ..Default::default() });
                add(Damager { kind: DMG_BLUNT, radius: y_half.max(0.02), a: [0.0, 0.0, head_z], b: [0.0, 0.0, z_hi - 0.01], ..Default::default() });
            } else {
                let r = ((x_hi - x_lo) * 0.5).max(y_half).max(0.025) * 0.9;
                add(Damager { kind: DMG_BLUNT, radius: r, a: [centre, 0.0, head_z + r * 0.5], b: [centre, 0.0, (z_hi - r * 0.5).max(head_z + r * 0.5)], ..Default::default() });
            }
            add(pommel);
        }
        STAFF => {
            let top = z_hi - (z_hi - z_lo) * 0.2;
            add(Damager { kind: DMG_BLUNT, radius: 0.03, a: [0.0, 0.0, top], b: [0.0, 0.0, z_hi], ..Default::default() });
            add(Damager { kind: DMG_BLUNT, radius: 0.025, a: [0.0, 0.0, z_lo], b: [0.0, 0.0, z_lo + 0.05], ..Default::default() });
        }
        _ => {}
    }
    w
}

/// The weapon as an object: held at its grip (the model's origin, where the game's hand holds it),
/// a fist round the handle with the blade out past the thumb and its edges fore and aft; a
/// two-handed one's other hand lower on the handle.
pub fn object_desc(model: &rustvr_asset::Model, e: &Equipped, key: &str, id: u32) -> ObjectDesc {
    let lo = Vec3::from_array(model.bounds_min);
    let hi = Vec3::from_array(model.bounds_max);
    let two_handed = matches!(e.kind, BLADE_TWO_HAND | BLUNT_TWO_HAND | STAFF);
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_MELEE;
    d.flags = OBJF_GRABBABLE | OBJF_PULLABLE | OBJF_HOLSTERABLE | OBJF_RUNTIME_DRAWS | if two_handed { OBJF_TWO_HANDED } else { 0 };
    d.shape = SHAPE_BOX;
    d.half_extents = ((hi - lo) * 0.5).max(Vec3::splat(0.01)).to_array();
    d.center = ((hi + lo) * 0.5).to_array();
    // The game's weights are heavy (an iron longsword is 22): to kilograms, a sword's 1.3.
    d.mass = (e.weight * 0.06).clamp(0.4, 4.0);
    d.grip = [0.0; 3];
    d.grip_rot = fist(Vec3::Z, Vec3::Y).to_array();
    d.muzzle = [0.0, 0.0, hi.z];
    if two_handed {
        d.foregrip = [0.0, 0.0, lo.z * 0.6];
        d.support_rot = d.grip_rot;
    }
    d.start_slot = 1 + if two_handed { SLOT_BACK_RIGHT } else { SLOT_HIP_LEFT };
    // The model's name, `<game>/<key>` (as `rustvr_guns::desc::model_name` makes it).
    set_name(&mut d.name, &format!("oblivion/{key}"));
    d
}

struct Registered {
    form: usize,
    seq: u32,
}

static REGISTERED: Mutex<Option<Registered>> = Mutex::new(None);
static MISSING: Mutex<Option<String>> = Mutex::new(None);

/// Once a game frame (in the world): the equipped weapon registered with the runtime when it
/// changes, or taken away when there's none.
pub fn publish(a: &rustvr_adapter::Adapter) {
    let now = equipped();
    let mut reg = REGISTERED.lock().unwrap();
    let form = now.as_ref().map_or(0, |e| e.form);
    if reg.as_ref().is_some_and(|r| r.form == form) {
        return;
    }
    let seq = reg.as_ref().map_or(0, |r| r.seq).wrapping_add(1).max(1);
    let mut objects: Box<ObjectTable> = zeroed();
    let mut melee: Box<MeleeTable> = zeroed();
    objects.seq = seq;
    melee.seq = seq;
    if let Some(e) = &now {
        let key = oblivion_assets::export::model_key(&e.model);
        match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
            Ok(model) => {
                let mut d = object_desc(&model, e, &key, WEAPON_ID);
                d.spawn_seq = seq;
                objects.descs[0] = d;
                objects.count = 1;
                melee.weapons[0] = melee_from_model(&model, e.kind, WEAPON_ID, e.damage as f32);
                melee.count = 1;
                tracing::info!(
                    "weapons: {} ({}, model {key}, type {}, damage {}, reach {}): {} damagers, {:.2} m long",
                    e.name,
                    e.model,
                    e.kind,
                    e.damage,
                    e.reach,
                    melee.weapons[0].count,
                    model.bounds_max[2] - model.bounds_min[2]
                );
            }
            Err(err) => {
                let mut missing = MISSING.lock().unwrap();
                if missing.as_deref() != Some(e.model.as_str()) {
                    tracing::warn!("weapons: {} has no extracted model ({}, {key}: {err:#}); run oblivion-extract export meshes/weapons", e.name, e.model);
                    *missing = Some(e.model.clone());
                }
            }
        }
    } else {
        tracing::info!("weapons: none equipped");
    }
    // The melee table first: when the runtime sees the weapon, its damagers are there.
    if a.write_melee(&melee) && a.write_objects(&objects) {
        *reg = Some(Registered { form, seq });
    }
}

fn zeroed<T: bytemuck::Zeroable>() -> Box<T> {
    // SAFETY: T is Zeroable (all-zero bytes are a valid T); built in place, not on the stack.
    unsafe { Box::<T>::new_zeroed().assume_init() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustvr_asset::{Mesh, Model, Vertex};

    fn model(points: &[[f32; 3]]) -> Model {
        let vertices: Vec<Vertex> = points.iter().map(|&pos| Vertex { pos, ..Default::default() }).collect();
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in points {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        Model { meshes: vec![Mesh { vertices, ..Default::default() }], bounds_min: lo, bounds_max: hi, ..Default::default() }
    }

    #[test]
    fn a_sword_cuts_from_its_guard_to_its_point() {
        // Pommel at -0.17, a guard 16 cm wide at 0, a blade 5 cm wide to 0.95.
        let m = model(&[[0.0, 0.0, -0.17], [-0.015, 0.0, -0.1], [0.015, 0.0, -0.1], [-0.08, 0.0, 0.0], [0.08, 0.0, 0.0], [-0.025, 0.0, 0.3], [0.025, 0.0, 0.3], [0.0, 0.0, 0.95]]);
        let w = melee_from_model(&m, BLADE_ONE_HAND, 9, 8.0);
        assert_eq!(w.count, 3);
        let cut = w.damagers[0];
        assert_eq!(cut.kind, DMG_SLASH);
        assert!((cut.a[2] - 0.035).abs() < 0.011, "from just past the guard: {:?}", cut.a);
        assert!((cut.b[2] - 0.92).abs() < 1e-4);
        assert_eq!(w.damagers[1].kind, DMG_PIERCE);
        assert_eq!(w.damagers[2].kind, DMG_BLUNT);
        assert!((w.reach - 0.95).abs() < 1e-5);
    }

    #[test]
    fn an_axe_cuts_with_the_side_its_head_reaches_to() {
        // A 3 cm handle to 0.5, a head from 0.4 reaching out to +0.14.
        let m = model(&[[-0.015, 0.0, -0.1], [0.015, 0.0, -0.1], [-0.015, 0.0, 0.2], [0.015, 0.0, 0.2], [-0.02, 0.0, 0.42], [0.14, 0.0, 0.42], [0.14, 0.0, 0.55], [-0.02, 0.0, 0.55]]);
        let w = melee_from_model(&m, BLUNT_ONE_HAND, 9, 8.0);
        let cut = w.damagers[0];
        assert_eq!(cut.kind, DMG_SLASH);
        assert_eq!(cut.edge, [1.0, 0.0, 0.0]);
        assert!(cut.a[0] > 0.1);
        assert_eq!(w.damagers[1].kind, DMG_BLUNT);
    }

    #[test]
    fn a_mace_bludgeons_with_its_head() {
        let m = model(&[[-0.015, 0.0, -0.1], [0.015, 0.0, -0.1], [-0.015, 0.0, 0.2], [0.015, 0.0, 0.2], [-0.05, 0.0, 0.4], [0.05, 0.0, 0.4], [0.05, 0.0, 0.5], [-0.05, 0.0, 0.5]]);
        let w = melee_from_model(&m, BLUNT_ONE_HAND, 9, 8.0);
        assert_eq!(w.damagers[0].kind, DMG_BLUNT);
        assert!(w.damagers[0].radius > 0.03);
    }

    #[test]
    fn a_one_handed_weapon_hangs_at_the_left_hip_a_two_handed_on_the_back() {
        let m = model(&[[0.0, 0.0, -0.2], [0.0, 0.0, 1.0]]);
        let mut e = Equipped { form: 1, name: "x".into(), model: "weapons\\iron\\longsword.nif".into(), kind: BLADE_ONE_HAND, damage: 8, reach: 1.0, weight: 22.0 };
        let d = object_desc(&m, &e, "m1", WEAPON_ID);
        assert_eq!(d.start_slot, 1 + SLOT_HIP_LEFT);
        assert_eq!(name_str(&d.name), "oblivion/m1");
        assert!(d.flags & OBJF_RUNTIME_DRAWS != 0 && d.flags & OBJF_TWO_HANDED == 0);
        e.kind = BLADE_TWO_HAND;
        let d = object_desc(&m, &e, "m1", WEAPON_ID);
        assert_eq!(d.start_slot, 1 + SLOT_BACK_RIGHT);
        assert!(d.flags & OBJF_TWO_HANDED != 0 && d.foregrip[2] < 0.0);
    }
}
