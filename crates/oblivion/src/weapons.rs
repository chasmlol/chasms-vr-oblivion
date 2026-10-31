//! The player's weapon, in the hand: the game's equipped weapon as an object the runtime draws (the
//! game's own model, `oblivion-extract export meshes/weapons` into `local_assets/oblivion`, named
//! by the hash of the path the weapon's form gives, `oblivion_assets::export::model_key`) and the
//! hands hold. It starts sheathed where the game wears it (a one-handed weapon at the left hip, a
//! two-handed one on the back) and the hand draws it from there.
//!
//! It's held and judged as the range holds and judges its weapons (the framework's
//! `docs/MELEE.md`): a [`MeleeWeapon`] measured on the game's model ([`measured_weapon`]) with the range's own
//! handles and damagers for its kind (`rustvr_melee::weapons`: a one-handed blade is the arming
//! sword, a short one its dagger, a two-handed one its two-handed slash; an axe's edge its axe, a
//! mace's or hammer's head its mace, a staff its staff), its balance where its model's metal is, and
//! the game's weight. The hand snaps onto its grip, a heavy one lags and swings on, two hands give
//! leverage, a sword's grip slides on over the guard to half-sword, and a point goes in and sticks.
//! What its blows do is the game's hit with the equipped weapon (`crate::combat::apply`): the game's
//! damage, skill and enchantment, as hard as the runtime judged the blow.
//!
//! The equipped shield is the other: taken from over the left shoulder by its strap, its face meets
//! blows (a shield that bashes too), and a block with it is the game's block with its shield.
//!
//! A staff casts: held in either hand, its trigger charges its enchantment at its tip and letting go
//! casts it from there along the staff (`OBJF_CASTS`): the game's own staff cast, its charge used
//! up (`crate::combat`). An enchanted one shimmers as the game draws it: its enchantment's own
//! effect shader over it (`crate::magic::enchant_overlay`).
//!
//! Forms: `MiddleHighProcess::equippedWeaponData` (+0xE4 of the player's process) is the
//! inventory entry of the equipped weapon (its form at +8), `equippedShieldData` (+0xF0) the
//! shield's (an armour: its male model's path at +0x70). `TESObjectWEAP` (xOBSE's
//! `GameForms.h`): its name at +0x28, its model's path at +0x34, its enchantment at +0x64
//! (`TESEnchantableForm` +0x60), its weight at +0x7C, its damage at +0x8C, its type, speed and reach
//! at +0x90..+0x98.

use crate::{engine, mem};
use glam::{Mat3, Quat, Vec3};
use rustvr_melee::weapons::*;
use rustvr_protocol::*;
use std::sync::Mutex;

/// The weapon's object id (stable while it's equipped), and the shield's.
pub const WEAPON_ID: u32 = 0x0B1E_0001;
pub const SHIELD_ID: u32 = 0x0B1E_0002;

/// `Actor::process` and `MiddleHighProcess::equippedWeaponData`; an inventory entry's form.
const PROCESS: usize = 0x58;
const EQUIPPED_WEAPON: usize = 0xE4;
const EQUIPPED_SHIELD: usize = 0xF0;
/// An armour's male biped model path (`TESBipedModelForm` +0x64, its first model's path +0xC).
const ARMOR_MODEL: usize = 0x70;
const FORM_ARMOR: u8 = 0x14;
const ENTRY_FORM: usize = 0x08;
/// `TESObjectWEAP` fields.
const WEAP_NAME: usize = 0x28;
const WEAP_MODEL: usize = 0x34;
const WEAP_ENCHANTMENT: usize = 0x64;
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
    /// Its enchantment's form id (0: none).
    pub enchantment: u32,
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
        enchantment: Some(mem::deref(form + WEAP_ENCHANTMENT)).filter(|&e| e != 0).and_then(|e| mem::at::<u32>(e + 0x0C)).unwrap_or(0),
    })
}

/// The equipped shield: its form and model path.
pub fn equipped_shield() -> Option<(usize, String)> {
    let process = mem::deref(engine::player() + PROCESS);
    let form = mem::deref(mem::deref(process + EQUIPPED_SHIELD) + ENTRY_FORM);
    if process == 0 || form == 0 || mem::at::<u8>(form + 4) != Some(FORM_ARMOR) {
        return None;
    }
    let model = crate::combat::read_str(mem::deref(form + ARMOR_MODEL));
    (!model.is_empty()).then_some((form, model))
}

/// A shield as an object, measured on its model (Oblivion's shields: the disc across X and Y, its
/// face to -Z, the strap standing out on the +Z side): held by the strap (the furthest out at +Z),
/// the fist round it with the thumb up the disc and the back of the hand to the shield. It's the range's
/// shield: its face meets blows and bashes, its drag a shield's.
pub fn shield(model: &rustvr_asset::Model, key: &str, id: u32) -> (ObjectDesc, MeleeWeapon) {
    let lo = Vec3::from_array(model.bounds_min);
    let hi = Vec3::from_array(model.bounds_max);
    let (mut strap, mut n) = (Vec3::ZERO, 0.0);
    for v in model.meshes.iter().flat_map(|m| &m.vertices) {
        let p = Vec3::from_array(v.pos);
        if p.z > hi.z - 0.015 {
            strap += p;
            n += 1.0;
        }
    }
    let strap = if n > 0.0 { strap / n } else { (lo + hi) * 0.5 };
    let centre = (lo + hi) * 0.5;
    let radius = ((hi.x - lo.x).min(hi.y - lo.y) * 0.5).max(0.1);
    let grip = fist(Vec3::Y, Vec3::NEG_Z);
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_MELEE;
    d.flags = OBJF_GRABBABLE | OBJF_PULLABLE | OBJF_HOLSTERABLE | OBJF_RUNTIME_DRAWS;
    d.shape = SHAPE_BOX;
    d.half_extents = ((hi - lo) * 0.5).max(Vec3::splat(0.01)).to_array();
    d.center = centre.to_array();
    d.mass = 3.0;
    d.grip = strap.to_array();
    d.grip_rot = grip.to_array();
    d.start_slot = 1 + SLOT_BACK_LEFT;
    set_name(&mut d.name, &format!("oblivion/{key}"));
    // The strap: the fist along it, a hand's width.
    let (a, b) = (strap - Vec3::Y * 0.05, strap + Vec3::Y * 0.05);
    d.grip_line = [a.to_array(), b.to_array()];
    d.grip_radius = 0.014;
    let face = Vec3::new(centre.x, centre.y, lo.z);
    let mut w = melee(MW_SHIELD, &[kind_of(handle(a, b, 0.014, HANDLE_MAIN, grip), GRIP_SHIELD, None, None)], &[blade(SHIELD, face, face + Vec3::Z * (hi.z - lo.z).max(0.02), radius * 0.9, Vec3::ZERO, 0.0)], centre);
    w.id = id;
    w.drag = [0.4, 0.05];
    (d, w)
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

    /// The middle width of the slices from `z0` to `z1` that have any.
    fn median_width(&self, z0: f32, z1: f32) -> f32 {
        let mut w: Vec<f32> = (0..self.lo.len()).filter(|&i| (z0..=z1).contains(&self.z(i))).map(|i| self.width(i)).filter(|&x| x > 0.0).collect();
        w.sort_by(f32::total_cmp);
        w.get(w.len() / 2).copied().unwrap_or(0.04)
    }

    /// Where its weight is: each slice weighed by its breadth and depth (a mace's head and a sword's
    /// guard heavier than a grip), at the middle of its extent.
    fn balance(&self) -> Vec3 {
        let (mut sum, mut total) = (Vec3::ZERO, 0.0);
        for i in 0..self.lo.len() {
            let (wx, wy) = (self.hi[i][0] - self.lo[i][0], self.hi[i][1] - self.lo[i][1]);
            // (An empty slice has none; a flat one, a little.)
            if wx < 0.0 || wy < 0.0 {
                continue;
            }
            let k = wx.max(0.002) * wy.max(0.002);
            sum += Vec3::new((self.lo[i][0] + self.hi[i][0]) * 0.5, (self.lo[i][1] + self.hi[i][1]) * 0.5, self.z(i)) * k;
            total += k;
        }
        if total > 0.0 { sum / total } else { Vec3::ZERO }
    }
}

/// How long a one-handed blade's blade may be (guard to point) and still be a dagger: the range's dagger
/// has 22 cm and its shortest sword 72; Oblivion's daggers run to about 30 (metres).
const DAGGER_BLADE: f32 = 0.35;

/// The game's weights are heavy (an iron longsword is 22): this to kilograms (a longsword's 1.3,
/// the arming sword 1.25), between a knife's and the maul's 8.
const KG_PER_WEIGHT: f32 = 0.06;
const MASS: (f32, f32) = (0.3, 8.0);

/// The weapon's mass in kilograms, from the game's weight.
pub fn mass_of(weight: f32) -> f32 {
    (weight * KG_PER_WEIGHT).clamp(MASS.0, MASS.1)
}

/// The weapon as the melee has its kind (`rustvr_melee::weapons`), measured on its model
/// (grip at the origin, its length along +Z, a blade's edges to ±X, as Oblivion's weapon models
/// are):
/// * a one-handed blade is the arming sword (its grip a light handle sliding on over the guard
///   onto the blade to half-sword, the pommel a second hand's), or with a blade no longer than
///   [`DAGGER_BLADE`] its dagger (held either way, thrown point first); a two-handed one the
///   two-handed slash on a heavy grip;
/// * a blunt weapon whose head reaches out to one side is the axe (its edge along the head's far
///   side), any other's head the mace (two-handed, its heavier blow); its haft a handle the hands
///   slide along, its end a light blow;
/// * a staff is held anywhere along it, both ends bludgeoning;
/// * its balance is where its model's metal is (each centimetre's slice weighed by its breadth and
///   depth), which a grip turns it about.
pub fn measured_weapon(model: &rustvr_asset::Model, kind: u32, id: u32) -> MeleeWeapon {
    let s = Slices::of(model);
    let n = s.lo.len();
    let (z_lo, z_hi) = (model.bounds_min[2], model.bounds_max[2]);
    let grip = fist(Vec3::Z, Vec3::Y);
    let at = |z: f32| Vec3::new(0.0, 0.0, z);
    // A handle along the axis up to `b`, a hand's width long at least (within the model).
    let line = |a: f32, b: f32| (a.min(b - 0.08).max(z_lo), b);
    let mut w = match kind {
        BLADE_ONE_HAND | BLADE_TWO_HAND => {
            // The guard: the widest slice above the hand (the model's origin; the pommel's below it),
            // up to halfway along.
            let guard = (0..n).filter(|&i| (-0.02..=z_lo + (z_hi - z_lo) * 0.5).contains(&s.z(i))).max_by(|&a, &b| s.width(a).total_cmp(&s.width(b))).map_or(0.0, |i| s.z(i));
            let length = (z_hi - guard).max(0.05);
            let thick = (s.median_width(guard, z_hi) * 0.5).clamp(0.015, 0.03);
            let (ha, hb) = line(z_lo + 0.03, guard - 0.01);
            let pommel = |d: Dmg| blade(d, at(z_lo), at(z_lo + 0.03), 0.025, Vec3::ZERO, 0.0);
            if kind == BLADE_ONE_HAND && length <= DAGGER_BLADE {
                let w = melee(
                    0,
                    &[handle(at(ha), at(hb), 0.013, HANDLE_MAIN | HANDLE_REVERSIBLE, grip)],
                    &[
                        sliding(blade(DAGGER_SLASH, at(guard + 0.01), at(z_hi - 0.005), thick, Vec3::X, 0.01), length * 0.95),
                        blade(DAGGER_PIERCE, at(z_hi - 0.05), at(z_hi), 0.01, Vec3::X, length),
                        pommel(HANDLE_LIGHT),
                    ],
                    s.balance(),
                );
                let mut w = flies(w, 145.0, 3.0);
                w.drag = [0.4, 0.05];
                w
            } else {
                let (slash, grip_kind) = if kind == BLADE_TWO_HAND { (SWORD_SLASH_2H, GRIP_HEAVY) } else { (SWORD_SLASH, GRIP_LIGHT) };
                melee(
                    0,
                    &[
                        kind_of(handle(at(ha), at(hb), 0.015, HANDLE_MAIN | HANDLE_SLIDE | HANDLE_REVERSIBLE, grip), grip_kind, Some(2), None),
                        kind_of(handle(at(z_lo), at(ha), 0.022, HANDLE_REVERSIBLE, grip), GRIP_SUPPORTING, None, None),
                        kind_of(handle(at(guard + length * 0.3), at(guard + length * 0.8), 0.02, HANDLE_BLADE | HANDLE_SLIDE | HANDLE_REVERSIBLE, grip), GRIP_HALF_SWORD, None, Some(0)),
                    ],
                    &[
                        sliding(blade(slash, at(guard + 0.03), at(z_hi - 0.01), thick, Vec3::X, 0.01), length * 0.85),
                        blade(SWORD_PIERCE, at(z_hi - 0.1), at(z_hi), 0.015, Vec3::X, (length - 0.08).max(0.1)),
                        pommel(HANDLE_HEAVY),
                    ],
                    s.balance(),
                )
            }
        }
        BLUNT_ONE_HAND | BLUNT_TWO_HAND => {
            let two = kind == BLUNT_TWO_HAND;
            let mut haft_w: Vec<f32> = (0..n / 2).map(|i| s.width(i)).filter(|&x| x > 0.0).collect();
            haft_w.sort_by(f32::total_cmp);
            let haft_w = haft_w.get(haft_w.len() / 2).copied().unwrap_or(0.03);
            // The head: the topmost run of slices more than twice the haft's width, down from its top
            // until 3 cm of plain haft (slices as wide as half the haft or more: empty or thinner
            // ones, and the haft through a gap in the head, don't end it).
            let (mut head, mut haft) = (None, 0);
            for i in (n / 3..n).rev() {
                let w = s.width(i);
                if w > haft_w * 2.0 {
                    head = Some(i);
                    haft = 0;
                } else if head.is_some() && w >= haft_w * 0.5 {
                    haft += 1;
                    if haft >= 3 {
                        break;
                    }
                }
            }
            let head = head.unwrap_or(n * 3 / 4);
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
            let (ha, hb) = line(z_lo + 0.02, head_z - 0.02);
            let haft = kind_of(handle(at(ha), at(hb), (haft_w * 0.5).clamp(0.012, 0.022), HANDLE_MAIN | HANDLE_SLIDE | HANDLE_REVERSIBLE, grip), if two { GRIP_HEAVY } else { GRIP_LIGHT }, None, None);
            let end = blade(if two { HANDLE_HEAVY } else { HANDLE_LIGHT }, at(z_lo), at(z_lo + 0.03), 0.02, Vec3::ZERO, 0.0);
            if axe {
                // The cutting edge, along the far side of the head.
                let side = centre.signum();
                let x = if side > 0.0 { x_hi } else { x_lo } * 0.9;
                let edge = (z_hi - 0.01 - head_z).max(0.04);
                melee(0, &[haft], &[sliding(blade(AXE_SLASH, Vec3::new(x, 0.0, head_z), Vec3::new(x, 0.0, z_hi - 0.01), 0.01, Vec3::new(side, 0.0, 0.0), 0.08), edge), end], s.balance())
            } else {
                // A head along the haft (a mace's), or across it, wider than it's tall (a hammer's
                // face and its back spike): the capsule along its longer way.
                let (across, tall) = (x_hi - x_lo, z_hi - head_z);
                let (a, b, r) = if across > tall * 1.3 {
                    let r = (tall * 0.5).max(y_half).max(0.025) * 0.9;
                    let z = (head_z + z_hi) * 0.5;
                    (Vec3::new(x_lo + r, 0.0, z), Vec3::new((x_hi - r).max(x_lo + r), 0.0, z), r)
                } else {
                    let r = (across * 0.5).max(y_half).max(0.025) * 0.9;
                    (Vec3::new(centre, 0.0, head_z + r * 0.5), Vec3::new(centre, 0.0, (z_hi - r * 0.5).max(head_z + r * 0.5)), r)
                };
                melee(0, &[haft], &[blade(if two { MACE_2H } else { MACE }, a, b, r, Vec3::ZERO, 0.0), end], s.balance())
            }
        }
        STAFF => {
            let tip = |a: f32, b: f32| blade(HANDLE_HEAVY, at(a), at(b), 0.025, Vec3::ZERO, 0.0);
            let spare = (z_hi - z_lo) * 0.05;
            let w = melee(
                MW_STAFF,
                &[kind_of(handle(at(z_lo + spare), at(z_hi - spare), 0.016, HANDLE_MAIN | HANDLE_SLIDE | HANDLE_REVERSIBLE, grip), GRIP_HEAVY, None, None)],
                &[tip(z_hi - 0.1, z_hi), tip(z_lo, z_lo + 0.1)],
                s.balance(),
            );
            flies(w, 30.0, 5.0)
        }
        _ => melee(0, &[handle(at(z_lo), at(z_hi.min(z_lo + 0.15)), 0.015, HANDLE_MAIN, grip)], &[], s.balance()),
    };
    w.id = id;
    w
}

/// The weapon as an object: held at its grip (the model's origin, where the game's hand holds it),
/// a fist round the handle with the blade out past the thumb and its edges fore and aft, as heavy as
/// the game says ([`mass_of`]).
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
    d.mass = mass_of(e.weight);
    d.grip = [0.0; 3];
    d.grip_rot = fist(Vec3::Z, Vec3::Y).to_array();
    d.muzzle = [0.0, 0.0, hi.z];
    // (Its handles are its melee weapon's: [`measured_weapon`]. This is the main one's line.)
    d.grip_line = [[0.0, 0.0, lo.z + 0.03], [0.0, 0.0, 0.0]];
    d.grip_radius = 0.015;
    // An enchanted staff casts its enchantment from its tip (over the spell hand's casting time).
    if e.kind == STAFF && e.enchantment != 0 {
        d.flags |= OBJF_CASTS;
        d.casts = e.enchantment;
    }
    d.start_slot = 1 + if two_handed { SLOT_BACK_RIGHT } else { SLOT_HIP_LEFT };
    // The model's name, `<game>/<key>` (as `rustvr_guns::desc::model_name` makes it).
    set_name(&mut d.name, &format!("oblivion/{key}"));
    d
}

struct Registered {
    /// The weapon's form and the shield's, and with a bow its arrows' and how many.
    forms: (usize, usize),
    arrows: (usize, u32),
    seq: u32,
}

static REGISTERED: Mutex<Option<Registered>> = Mutex::new(None);
/// The melee weapons as last sent (the weapon's and the shield's melee numbers, for what their hits do).
static SENT: Mutex<Vec<MeleeWeapon>> = Mutex::new(Vec::new());

/// Weapon `id`'s melee numbers as the runtime has them (the equipped weapon's, the shield's).
pub fn melee_weapon(id: u32) -> Option<MeleeWeapon> {
    SENT.lock().unwrap().iter().find(|w| w.id == id).copied()
}
static MISSING: Mutex<Option<String>> = Mutex::new(None);
/// A weapon drawn from one of the body's slots and equipped by the game (`crate::inventory`): its
/// form, and the runtime's copy of it in a hand, whose place it takes as it's registered.
static REPLACING: Mutex<Option<(usize, u32)>> = Mutex::new(None);

/// The weapon of `form`, equipped from the runtime's copy `copy` in a hand: it takes that copy's
/// place in the hand when it's registered.
pub fn replacing(form: usize, copy: u32) {
    *REPLACING.lock().unwrap() = Some((form, copy));
}

/// Once a game frame (in the world): the equipped weapon registered with the runtime when it
/// changes, or taken away when there's none.
pub fn publish(a: &rustvr_adapter::Adapter) {
    let now = equipped();
    // (With a bow the game holds no shield: both hands are on the bow.)
    let shield_now = equipped_shield().filter(|_| now.as_ref().is_none_or(|e| e.kind != crate::archery::BOW));
    let mut reg = REGISTERED.lock().unwrap();
    let forms = (now.as_ref().map_or(0, |e| e.form), shield_now.as_ref().map_or(0, |s| s.0));
    let arrows = crate::archery::arrows_key();
    if reg.as_ref().is_some_and(|r| r.forms == forms && r.arrows == arrows) {
        return;
    }
    // A new weapon is put where it starts (its holster); only the arrows changing (one shot) leaves
    // the bow where it is.
    let seq = match reg.as_ref() {
        Some(r) if r.forms == forms => r.seq,
        r => r.map_or(0, |r| r.seq).wrapping_add(1).max(1),
    };
    let mut objects: Box<ObjectTable> = zeroed();
    let mut melee: Box<MeleeWeapons> = zeroed();
    objects.seq = seq;
    if let Some(e) = now.as_ref().filter(|e| e.kind == crate::archery::BOW) {
        // A bow and its arrows (crate::archery).
        let mut descs = Vec::new();
        crate::archery::register(e, seq, &mut descs, a);
        // (A bow drawn from a slot takes its copy's place in the hand, as any weapon does.)
        if let Some((_, copy)) = REPLACING.lock().unwrap().take_if(|(form, _)| *form == e.form)
            && let Some(d) = descs.iter_mut().find(|d| d.id == WEAPON_ID)
        {
            d.replaces = copy;
            tracing::info!("weapons: {} takes the place of its copy {copy:#x} in the hand", e.name);
        }
        for d in descs {
            objects.descs[objects.count as usize] = d;
            objects.count += 1;
        }
    } else if let Some(e) = &now {
        let key = oblivion_assets::export::model_key(&e.model);
        match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
            Ok(model) => {
                let mut d = object_desc(&model, e, &key, WEAPON_ID);
                d.spawn_seq = seq;
                if let Some((_, copy)) = REPLACING.lock().unwrap().take_if(|(form, _)| *form == e.form) {
                    d.replaces = copy;
                    tracing::info!("weapons: {} takes the place of its copy {copy:#x} in the hand", e.name);
                }
                objects.descs[0] = d;
                objects.count = 1;
                melee.weapons[0] = measured_weapon(&model, e.kind, WEAPON_ID);
                melee.count = 1;
                let w = &melee.weapons[0];
                tracing::info!(
                    "weapons: {} ({}, model {key}, type {}, damage {}, reach {}, enchantment {:#x}): {} handles, blades {:?}, {:.2} m long, {:.2} kg balanced at {:.2?}",
                    e.name,
                    e.model,
                    e.kind,
                    e.damage,
                    e.reach,
                    e.enchantment,
                    w.handles,
                    w.blades().iter().map(|b| b.kind).collect::<Vec<_>>(),
                    model.bounds_max[2] - model.bounds_min[2],
                    d.mass,
                    w.balance
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
    if let Some((_, path)) = &shield_now {
        let key = oblivion_assets::export::model_key(path);
        match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
            Ok(model) => {
                let (mut d, w) = shield(&model, &key, SHIELD_ID);
                d.spawn_seq = seq;
                let (i, k) = (objects.count as usize, melee.count as usize);
                objects.descs[i] = d;
                objects.count += 1;
                melee.weapons[k] = w;
                melee.count += 1;
                tracing::info!("weapons: the shield {path} (model {key}), held by its strap at {:?}", d.grip);
            }
            Err(err) => tracing::warn!("weapons: the shield {path} has no extracted model ({key}: {err:#})"),
        }
    }
    // An enchanted weapon's shimmer: its enchantment's own effect shader.
    #[cfg(target_arch = "x86")]
    crate::magic::set_weapon_overlay(now.as_ref().filter(|e| e.enchantment != 0).and_then(|e| crate::magic::enchant_overlay(e.enchantment, WEAPON_ID)));
    // What they sound like: the weapon its type's draw and sheathe, the shield and the arrows as the
    // inventory sounds them.
    let mut sounds = Vec::new();
    if let Some(set) = now.as_ref().and_then(|e| crate::sounds::of_weapon(e.form, e.kind)) {
        sounds.push((WEAPON_ID, set));
    }
    if let Some(set) = shield_now.as_ref().and_then(|(form, _)| crate::sounds::kind(*form)) {
        sounds.push((SHIELD_ID, set));
    }
    #[cfg(all(windows, target_arch = "x86"))]
    if let Some(set) = now.as_ref().filter(|e| e.kind == crate::archery::BOW).and_then(|_| crate::archery::equipped_arrows()).and_then(|ar| crate::sounds::kind(ar.form)) {
        sounds.push((crate::archery::ARROWS_ID, set));
    }
    // The melee weapons first: when the runtime sees the weapon, its handles and damagers are there.
    if a.write_melee_weapons(&melee) && crate::objects::publish(crate::objects::Owner::Weapons, &objects.descs[..objects.count as usize], &sounds, a) {
        *reg = Some(Registered { forms, arrows, seq });
        *SENT.lock().unwrap() = melee.weapons[..melee.count as usize].to_vec();
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
    fn a_sword_is_held_by_its_grip_and_cuts_from_its_guard_to_its_point() {
        // Pommel at -0.17, a guard 16 cm wide at 0, a blade 5 cm wide to 0.95.
        let m = model(&[[0.0, 0.0, -0.17], [-0.015, 0.0, -0.1], [0.015, 0.0, -0.1], [-0.08, 0.0, 0.0], [0.08, 0.0, 0.0], [-0.025, 0.0, 0.3], [0.025, 0.0, 0.3], [0.0, 0.0, 0.95]]);
        let w = measured_weapon(&m, BLADE_ONE_HAND, 9);
        assert_eq!((w.id, w.handles, w.blades), (9, 3, 3));
        let grip = w.handle[0];
        assert!(grip.flags & HANDLE_MAIN != 0 && grip.kind == GRIP_LIGHT && grip.a[2] < -0.1 && grip.b[2] < 0.0, "{grip:?}");
        // (Sliding off the grip's top goes on onto the blade, to half-sword, and back.)
        assert_eq!((grip.links & 0xFF, w.handle[2].kind, w.handle[2].links >> 8), (3, GRIP_HALF_SWORD, 1));
        let cut = w.blade[0];
        assert_eq!((cut.kind, cut.modifier), (DMG_SLASH, rustvr_melee::MOD_SLASH));
        assert!((cut.a[2] - 0.035).abs() < 0.011, "from just past the guard: {:?}", cut.a);
        assert!((cut.b[2] - 0.94).abs() < 1e-4);
        assert_eq!(w.blade[1].kind, DMG_PIERCE);
        assert!(w.blade[1].depth > 0.8);
        assert_eq!(w.blade[2].kind, DMG_BLUNT);
    }

    #[test]
    fn a_short_blade_is_a_dagger_thrown_point_first() {
        let m = model(&[[0.0, 0.0, -0.1], [-0.04, 0.0, 0.0], [0.04, 0.0, 0.0], [-0.015, 0.0, 0.1], [0.015, 0.0, 0.1], [0.0, 0.0, 0.25]]);
        let w = measured_weapon(&m, BLADE_ONE_HAND, 9);
        assert_eq!((w.handles, w.blade[0].modifier, w.blade[1].modifier), (1, rustvr_melee::MOD_SLASH_WEAK, rustvr_melee::MOD_PIERCE_FINE));
        assert!(w.flags & MW_FLIES != 0);
    }

    #[test]
    fn an_axe_cuts_with_the_side_its_head_reaches_to() {
        // A 3 cm handle to 0.5, a head from 0.4 reaching out to +0.14.
        let m = model(&[[-0.015, 0.0, -0.1], [0.015, 0.0, -0.1], [-0.015, 0.0, 0.2], [0.015, 0.0, 0.2], [-0.02, 0.0, 0.42], [0.14, 0.0, 0.42], [0.14, 0.0, 0.55], [-0.02, 0.0, 0.55]]);
        let w = measured_weapon(&m, BLUNT_ONE_HAND, 9);
        let cut = w.blade[0];
        assert_eq!(cut.kind, DMG_SLASH);
        assert!(cut.flags & DMGF_AXE != 0);
        assert_eq!(cut.edge, [1.0, 0.0, 0.0]);
        assert!(cut.a[0] > 0.1);
        // (Its weight is out towards its head.)
        assert!(w.balance[2] > 0.1 && w.balance[0] > 0.0, "{:?}", w.balance);
    }

    #[test]
    fn a_mace_bludgeons_with_its_head() {
        let m = model(&[[-0.015, 0.0, -0.1], [0.015, 0.0, -0.1], [-0.015, 0.0, 0.2], [0.015, 0.0, 0.2], [-0.05, 0.0, 0.4], [0.05, 0.0, 0.4], [0.05, 0.0, 0.5], [-0.05, 0.0, 0.5]]);
        let w = measured_weapon(&m, BLUNT_ONE_HAND, 9);
        assert_eq!((w.blade[0].kind, w.blade[0].modifier), (DMG_BLUNT, rustvr_melee::MOD_BLUNT));
        assert!(w.blade[0].radius > 0.03);
        assert_eq!(measured_weapon(&m, BLUNT_TWO_HAND, 9).handle[0].kind, GRIP_HEAVY);
    }

    #[test]
    fn a_shield_is_held_by_its_strap_and_bashes_with_its_face() {
        // A disc 60 cm across in X and Y, its face at z -0.1, the strap standing out to +0.04.
        let m = model(&[[-0.05, -0.3, -0.1], [0.55, 0.3, -0.1], [0.25, 0.0, -0.15], [0.2, -0.05, 0.04], [0.2, 0.05, 0.04]]);
        let (d, w) = shield(&m, "m1", SHIELD_ID);
        assert!((Vec3::from_array(d.grip) - Vec3::new(0.2, 0.0, 0.04)).length() < 1e-5, "{:?}", d.grip);
        assert_eq!(d.start_slot, 1 + SLOT_BACK_LEFT);
        assert!(w.flags & MW_SHIELD != 0 && w.handle[0].kind == GRIP_SHIELD);
        assert!(w.blades == 1 && w.blade[0].radius > 0.2);
    }

    #[test]
    fn a_one_handed_weapon_hangs_at_the_left_hip_a_two_handed_on_the_back() {
        let m = model(&[[0.0, 0.0, -0.2], [0.0, 0.0, 1.0]]);
        let mut e = Equipped { form: 1, name: "x".into(), model: "weapons\\iron\\longsword.nif".into(), kind: BLADE_ONE_HAND, damage: 8, reach: 1.0, weight: 22.0, enchantment: 0 };
        let d = object_desc(&m, &e, "m1", WEAPON_ID);
        assert_eq!(d.start_slot, 1 + SLOT_HIP_LEFT);
        assert_eq!(name_str(&d.name), "oblivion/m1");
        assert!(d.flags & OBJF_RUNTIME_DRAWS != 0 && d.flags & OBJF_TWO_HANDED == 0);
        e.kind = BLADE_TWO_HAND;
        let d = object_desc(&m, &e, "m1", WEAPON_ID);
        assert_eq!(d.start_slot, 1 + SLOT_BACK_RIGHT);
        assert!(d.flags & OBJF_TWO_HANDED != 0);
        assert!((d.mass - 1.32).abs() < 1e-4);
    }
}
