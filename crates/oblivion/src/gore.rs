//! Oblivion's side of the framework's gore (`rustvr_protocol::gore`): its blood, and the limbs its
//! creatures lose.
//!
//! * **Its blood.** Each creature bleeds what the game says it does: its base form's blood (the
//!   part at +0x24, virtual +0x38 its decal texture, +0x40 its spray model; what the game's own hit
//!   asks, `Actor` 0x5EC400): a creature's own (its CREA record), else the game settings' defaults
//!   (`sBloodTextureDefault`, `sBloodParticleDefault`). Each kind is the game's art for it
//!   ([`look`]): people's red decal atlas, the dungeons' splatter and stains, the arena's runs, the
//!   spray's droplet and mist, the Kvatch head pike's neck for a stump; a lich's black; a
//!   will-o'-the-wisp's glow; a skeleton's bone dust.
//! * **The cut** ([`after_hit`], on the game's thread right after its hit): with the game's
//!   numbers (its damage for the weapon and its kind, the hit's strength, the creature's health
//!   before and after) the framework's rule decides ([`rustvr_protocol::severs`]). It cuts at the
//!   joint the blade crossed: the neck (`Bip01 Head`), the shoulder, the elbow, the wrist, the hip,
//!   the knee or the ankle.
//! * **On the body** every node under the cut bone is drawn shrunk to nothing at the joint, its
//!   world transform written before the eyes are drawn (after the game's animation and ragdoll
//!   have placed it): the meshes skinned to those bones close up there, so a hand or a foot goes
//!   whole (they're meshes of their own), an arm or a leg ends at the joint, the neck at the jaw.
//! * **The severed part** is a clone of the creature's 3D (`NiObject::Clone`, 0x700900: its face,
//!   hair, what it wears) put in the world beside it, with every bone but the cut ones shrunk to
//!   nothing at the joint: only the limb shows, as it was when it was cut. It's one of the
//!   runtime's props (the runtime's physics against the game's collision: it falls, rolls, and the
//!   hands can take it), and the clone is drawn where the runtime has it. It stays while the body
//!   does.
//! * **The stumps** go to the runtime every frame (it pumps blood from them) and spray the
//!   creature's own blood effect with the pulses (the game's `BSTempEffectParticle`, 0x5713F0, as
//!   its hit does).

use rustvr_protocol::*;

/// The art of a kind of blood, from what the game says the creature bleeds: (its decal texture,
/// its spray model) -> the blood's flags and textures (paths under `textures\`).
pub struct Look {
    pub flags: u32,
    pub atlas: u32,
    pub splat: &'static str,
    pub splatter: &'static str,
    pub stain: &'static str,
    pub drip: &'static str,
    pub drop: &'static str,
    pub mist: &'static str,
    pub flesh: &'static str,
}

/// The game's own blood art for a creature's blood (decal texture and spray model, as the game
/// gives them: `Effects\blooddecal.dds`, `Effects\BloodSpray.nif`). `None`: it doesn't bleed.
pub fn look(decal: &str, spray: &str) -> Option<(Look, Option<String>)> {
    let d = decal.to_ascii_lowercase().replace('/', "\\");
    let s = spray.to_ascii_lowercase();
    let red = Look {
        flags: BLOOD_LIQUID,
        atlas: 2,
        splat: "effects\\blooddecal.dds",
        splatter: "dungeons\\misc\\bloodsplatter01.dds",
        stain: "dungeons\\misc\\bloodstain01.dds",
        drip: "architecture\\arena\\arenablooddecal01.dds",
        drop: "effects\\blood01.dds",
        mist: "effects\\blood06.dds",
        flesh: "clutter\\kvatch\\neck.dds",
    };
    if s.contains("skeletonbloodspray") {
        // Bone dust: it puffs and settles.
        return Some((Look { flags: BLOOD_DUST, atlas: 1, splat: "", splatter: "", stain: "", drip: "", drop: "effects\\bonedust.dds", mist: "effects\\bonedust.dds", flesh: "creatures\\zombie\\zombiebone.dds" }, None));
    }
    if d.ends_with("lichblooddecal.dds") {
        return Some((Look { splat: "effects\\lichblooddecal.dds", splatter: "", stain: "", drip: "", drop: "effects\\lichblood01.dds", mist: "", ..red }, None));
    }
    if d.ends_with("willothewispblooddecal.dds") {
        return Some((Look { flags: BLOOD_LIQUID | BLOOD_GLOW, splat: "effects\\willothewispblooddecal.dds", splatter: "", stain: "", drip: "", drop: "", mist: "", ..red }, None));
    }
    if d.is_empty() && s.is_empty() {
        return None;
    }
    if d.is_empty() || d.ends_with("effects\\blooddecal.dds") {
        return Some((red, None));
    }
    // A creature's own decal: its splats, the rest from its colour.
    let own = d.strip_prefix("textures\\").unwrap_or(&d).to_owned();
    Some((Look { splatter: "", stain: "", drip: "", drop: "", mist: "", ..red }, Some(own)))
}

/// A limb of the game's skeleton: the bone it's cut at, the bone above it (the joint's other side),
/// the bones its axis runs between (out of the body), what it is, and its radius at the joint (m).
pub struct Limb {
    pub bone: &'static str,
    pub parent: &'static str,
    pub axis: (&'static str, &'static str),
    pub limb: u32,
    pub radius: f32,
    /// How far past its last bone the limb reaches (m): a head's crown, a hand's fingertips.
    pub reach: f32,
    /// What it weighs when the game's ragdoll doesn't say (kg).
    pub mass: f32,
}

pub const LIMBS: &[Limb] = &[
    // The head goes with its neck: cut at the neck's base (`Neck1`, at the collar), the head's mesh
    // (skinned down to the clavicles) closes up there, under the collar, instead of leaving a cone of
    // neck on the body.
    Limb { bone: "Bip01 Neck1", parent: "Bip01 Neck", axis: ("Bip01 Neck", "Bip01 Head"), limb: LIMB_HEAD, radius: 0.065, reach: 0.2, mass: 4.5 },
    Limb { bone: "Bip01 L UpperArm", parent: "Bip01 L Clavicle", axis: ("Bip01 L Clavicle", "Bip01 L UpperArm"), limb: LIMB_UPPER_ARM | LIMB_LEFT, radius: 0.055, reach: 0.1, mass: 4.0 },
    Limb { bone: "Bip01 L Forearm", parent: "Bip01 L UpperArm", axis: ("Bip01 L UpperArm", "Bip01 L Forearm"), limb: LIMB_FOREARM | LIMB_LEFT, radius: 0.045, reach: 0.1, mass: 2.0 },
    Limb { bone: "Bip01 L Hand", parent: "Bip01 L Forearm", axis: ("Bip01 L Forearm", "Bip01 L Hand"), limb: LIMB_HAND | LIMB_LEFT, radius: 0.035, reach: 0.1, mass: 0.6 },
    Limb { bone: "Bip01 R UpperArm", parent: "Bip01 R Clavicle", axis: ("Bip01 R Clavicle", "Bip01 R UpperArm"), limb: LIMB_UPPER_ARM, radius: 0.055, reach: 0.1, mass: 4.0 },
    Limb { bone: "Bip01 R Forearm", parent: "Bip01 R UpperArm", axis: ("Bip01 R UpperArm", "Bip01 R Forearm"), limb: LIMB_FOREARM, radius: 0.045, reach: 0.1, mass: 2.0 },
    Limb { bone: "Bip01 R Hand", parent: "Bip01 R Forearm", axis: ("Bip01 R Forearm", "Bip01 R Hand"), limb: LIMB_HAND, radius: 0.035, reach: 0.1, mass: 0.6 },
    Limb { bone: "Bip01 L Thigh", parent: "Bip01 Pelvis", axis: ("Bip01 Pelvis", "Bip01 L Thigh"), limb: LIMB_THIGH | LIMB_LEFT, radius: 0.075, reach: 0.12, mass: 10.0 },
    Limb { bone: "Bip01 L Calf", parent: "Bip01 L Thigh", axis: ("Bip01 L Thigh", "Bip01 L Calf"), limb: LIMB_CALF | LIMB_LEFT, radius: 0.055, reach: 0.12, mass: 4.5 },
    Limb { bone: "Bip01 L Foot", parent: "Bip01 L Calf", axis: ("Bip01 L Calf", "Bip01 L Foot"), limb: LIMB_FOOT | LIMB_LEFT, radius: 0.045, reach: 0.1, mass: 1.2 },
    Limb { bone: "Bip01 R Thigh", parent: "Bip01 Pelvis", axis: ("Bip01 Pelvis", "Bip01 R Thigh"), limb: LIMB_THIGH, radius: 0.075, reach: 0.12, mass: 10.0 },
    Limb { bone: "Bip01 R Calf", parent: "Bip01 R Thigh", axis: ("Bip01 R Thigh", "Bip01 R Calf"), limb: LIMB_CALF, radius: 0.055, reach: 0.12, mass: 4.5 },
    Limb { bone: "Bip01 R Foot", parent: "Bip01 R Calf", axis: ("Bip01 R Calf", "Bip01 R Foot"), limb: LIMB_FOOT, radius: 0.045, reach: 0.1, mass: 1.2 },
];

/// The head of a skeleton without a neck's base (`Bip01 Neck1`: a creature's): cut at the head.
pub const HEAD_ALONE: Limb = Limb { bone: "Bip01 Head", parent: "Bip01 Neck", axis: ("Bip01 Neck", "Bip01 Head"), limb: LIMB_HEAD, radius: 0.06, reach: 0.2, mass: 4.5 };

/// Which joint a hit on a body part (a capsule from bone `from` to bone `to`, `t` along it from the
/// one to the other, past 1 beyond `to`; the bones `len` metres apart) cuts through: the nearer
/// end's, as a blade takes a limb off at the joint it crosses. None for the torso.
pub fn joint_for(from: &str, to: &str, t: f32, len: f32) -> Option<&'static Limb> {
    let by = |b: &str| LIMBS.iter().find(|l| l.bone == b);
    let t_in = t.clamp(0.0, 1.0);
    match (from, to) {
        // The neck: the head comes off, struck anywhere up to the jaw (a blow on the skull above
        // it doesn't).
        ("Bip01 Neck", "Bip01 Head") if (t - 1.0) * len < 0.08 => by("Bip01 Neck1"),
        // An upper arm or thigh: at the shoulder or hip in its upper half, the elbow or knee below.
        (f, t2) if f.ends_with("UpperArm") || f.ends_with("Thigh") => by(if t_in < 0.5 { f } else { t2 }),
        // A forearm or calf: the elbow or knee in its upper part, the wrist or ankle in the rest.
        (f, t2) if f.ends_with("Forearm") || f.ends_with("Calf") => by(if t_in < 0.4 { f } else { t2 }),
        _ => None,
    }
}

/// How well a weapon cuts, as the game rates it ([`rustvr_protocol::weapon_cut`]): its damage, and
/// its kind (`TESObjectWEAP::type`: 0 a one-handed blade, 1 a two-handed one, 2 and 3 blunt, which
/// are Oblivion's axes when they have an edge), short blades less (the reach a dagger has).
pub fn weapon_cut(damage: f32, kind: u32, reach: f32) -> f32 {
    let k = match kind {
        0 => (0.55 + 0.45 * reach).clamp(0.5, 1.0),
        1 => 1.35,
        2 => 1.15,
        3 => 1.45,
        _ => 0.0,
    };
    rustvr_protocol::weapon_cut(damage, k)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
mod stubs {
    pub fn health(_actor: usize) -> f32 {
        0.0
    }
    pub fn after_hit(_actor: usize, _ev: &rustvr_protocol::CombatEvent, _before: f32) {}
    pub fn frame(_view: Option<&rustvr_adapter::View>, _now: f64) {}
    pub fn cut_off(_actor: u32, _bone: &str) -> bool {
        false
    }
    pub fn cut_at(_actor: u32, _bone: &str) -> bool {
        false
    }
    pub fn test_cut(_bone: &str, _kill: bool) -> bool {
        false
    }
}
#[cfg(not(all(windows, target_arch = "x86")))]
pub use stubs::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, av};
    use crate::items::{ni_matrix, ni_rows};
    use crate::{camera, mem};
    use glam::{Mat3, Quat, Vec3};
    use std::collections::HashMap;
    use std::sync::Mutex;

    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A3_0000;
    /// `TESObjectREFR`: its 3D, its cell; `TESForm::refID`.
    const REFR_NODE: usize = 0x3C;
    const REFR_CELL: usize = 0x40;
    const FORM_ID: usize = 0x0C;
    /// `TESObjectREFR` virtuals: its base object, and whether it has one to ask.
    const VT_BASE: usize = 0x170;
    const VT_HAS_BASE: usize = 0x190;
    /// The base form's blood: its part at +0x24, virtuals +0x38 (the decal) and +0x40 (the spray).
    const BLOOD_PART: usize = 0x24;
    const VT_BLOOD_DECAL: usize = 0x38;
    const VT_BLOOD_SPRAY: usize = 0x40;
    /// `Actor::GetActorValue` (vtable +0x284) and the health.
    const VT_GET_AV: usize = 0x284;
    const AV_HEALTH: u32 = 8;
    /// `NiObjectNET`: name, controller; `NiAVObject`: flags, parent, world bound, world transform,
    /// collision object; `NiNode`: children.
    const NAME: usize = 0x08;
    const CONTROLLER: usize = 0x0C;
    const PARENT: usize = 0x1C;
    const BOUND: usize = 0x20;
    const COLLISION: usize = 0xA8;
    const CHILDREN: usize = 0xB0;
    const CHILD_COUNT: usize = 0xB6;
    /// `NiObject::GetRTTI` (virtual 1): `NiRTTI` {name, parent}.
    const VT_RTTI: usize = 0x04;
    /// `NiObject::Clone` (thiscall): a copy of the object and everything under it.
    const CLONE: usize = 0x0070_0900;
    /// `NiNode::AttachChild(child, first free)`, `DetachChild(out, child)` (virtuals).
    const VT_ATTACH: usize = 0x84;
    const VT_DETACH: usize = 0x88;
    /// The game's allocator; `BSTempEffectParticle`'s constructor (thiscall: cell, 1.0, node,
    /// model, direction, position, 1.0, 1) and the process manager's temporary effects (thiscall).
    const ALLOC: usize = 0x0040_1F00;
    const TEMP_PARTICLE: usize = 0x0057_13F0;
    const ADD_TEMP_EFFECT: usize = 0x0067_8D30;
    const PROCESS_MANAGER: usize = 0x00B3_BD00;
    /// The quarter of an exterior cell a reference is in (cdecl), and a cell's node.
    const CELL_QUARTER: usize = 0x004C_9BE0;
    const CELL_NODE: usize = 0x54;
    /// A severed piece's object ids.
    const PIECE_IDS: u32 = 0x0B1E_8000;
    /// Shrunk to nothing (a world scale of zero breaks the game's bounds).
    const NOTHING: f32 = 1e-4;
    /// The game's own decals (its blood's on a hit, as much as ours): the settings `fDecalLifetime`
    /// and `iMaxDecalsPerFrame` (`Oblivion.ini` [Display]; {value, name} in memory). Its 10 s
    /// lifetime takes the game's blood away long before ours dries; in VR it lasts as ours does.
    const DECAL_LIFETIME: usize = 0x00B0_97C8;
    const DECALS_PER_FRAME: usize = 0x00B0_97D0;
    const GAME_DECALS_LAST: f32 = 600.0;
    const GAME_DECALS_PER_FRAME: i32 = 20;
    /// The stumps spray the game's blood effect with every second pulse for this long (s).
    const SPRAYS_FOR: f64 = 10.0;
    const SPRAY_EVERY: f64 = 1.6;

    /// A node's world transform: rotation, translation (game units), scale.
    #[derive(Clone, Copy, Debug)]
    struct Xf {
        r: Mat3,
        t: Vec3,
        s: f32,
    }

    impl Xf {
        fn read(node: usize) -> Option<Xf> {
            let r: [f32; 9] = mem::at(node + av::WORLD_ROTATE)?;
            let t: [f32; 3] = mem::at(node + av::WORLD_TRANSLATE)?;
            let s: f32 = mem::at(node + av::WORLD_SCALE)?;
            let r = ni_matrix(r);
            (r.is_finite() && Vec3::from(t).is_finite() && s.is_finite()).then_some(Xf { r, t: Vec3::from(t), s })
        }

        fn write(&self, node: usize) {
            unsafe {
                ((node + av::WORLD_ROTATE) as *mut [f32; 9]).write_unaligned(ni_rows(self.r));
                ((node + av::WORLD_TRANSLATE) as *mut [f32; 3]).write_unaligned(self.t.to_array());
                ((node + av::WORLD_SCALE) as *mut f32).write_unaligned(self.s);
            }
        }

        /// `self` then `o` (o in self's frame).
        fn then(&self, o: &Xf) -> Xf {
            Xf { r: self.r * o.r, t: self.t + self.r * (o.t * self.s), s: self.s * o.s }
        }
    }

    /// One node of a 3D tree, in order (parents before children).
    #[derive(Clone, Debug)]
    struct Node {
        ptr: usize,
        parent: Option<usize>,
        name: String,
    }

    /// Whether `obj` is an `NiNode` (or one of its kinds), by its `NiRTTI` chain.
    fn is_node(obj: usize) -> bool {
        is_a(obj, "NiNode")
    }

    /// Whether `obj` is a `kind` (or one of its kinds), by its `NiRTTI` chain; cached by vtable.
    fn is_a(obj: usize, kind: &'static str) -> bool {
        static KNOWN: Mutex<Option<HashMap<(usize, &'static str), bool>>> = Mutex::new(None);
        let vt = mem::deref(obj);
        if vt == 0 {
            return false;
        }
        let mut known = KNOWN.lock().unwrap();
        let known = known.get_or_insert_with(HashMap::new);
        if let Some(&k) = known.get(&(vt, kind)) {
            return k;
        }
        let f = mem::deref(vt + VT_RTTI);
        let mut yes = false;
        if CODE.contains(&f) {
            type Rtti = unsafe extern "thiscall" fn(usize) -> usize;
            let mut rtti = unsafe { std::mem::transmute::<usize, Rtti>(f)(obj) };
            for _ in 0..16 {
                if rtti == 0 {
                    break;
                }
                if crate::combat::read_str(mem::deref(rtti)) == kind {
                    yes = true;
                    break;
                }
                rtti = mem::deref(rtti + 4);
            }
        }
        known.insert((vt, kind), yes);
        yes
    }

    /// `NiGeometry`'s skin instance; `NiSkinInstance`'s data (its bone count at +0x40) and bones.
    const GEOM_SKIN: usize = 0xB8;
    const SKIN_DATA: usize = 0x08;
    const SKIN_BONES: usize = 0x14;
    const SKIN_DATA_COUNT: usize = 0x40;

    /// Whether a mesh of the clone shows any of the piece: skinned to a bone under the cut, or a
    /// rigid one under it. (The rest would only be drawn shrunk to nothing: they're hidden, so a
    /// piece costs the game only its own meshes.)
    fn shows_piece(geom: usize, inside: bool, piece: &std::collections::HashSet<usize>) -> bool {
        let skin = mem::deref(geom + GEOM_SKIN);
        if skin == 0 {
            return inside;
        }
        let count = mem::at::<u32>(mem::deref(skin + SKIN_DATA) + SKIN_DATA_COUNT).unwrap_or(0).min(256) as usize;
        let bones = mem::deref(skin + SKIN_BONES);
        bones != 0 && (0..count).any(|i| piece.contains(&mem::deref(bones + i * 4)))
    }

    /// Every object under `root` (itself first), parents before children.
    fn tree(root: usize) -> Vec<Node> {
        let mut out = vec![Node { ptr: root, parent: None, name: crate::combat::read_str(mem::deref(root + NAME)) }];
        let mut i = 0;
        while i < out.len() && out.len() < 1500 {
            let n = out[i].ptr;
            if is_node(n) {
                let data = mem::deref(n + CHILDREN);
                let count = mem::at::<u16>(n + CHILD_COUNT).unwrap_or(0).min(256) as usize;
                for k in 0..count {
                    let c = mem::deref(data + k * 4);
                    if c != 0 && mem::is_readable(c as *const u8, 0xB8) {
                        out.push(Node { ptr: c, parent: Some(i), name: crate::combat::read_str(mem::deref(c + NAME)) });
                    }
                }
            }
            i += 1;
        }
        out
    }

    /// Every object under `root` (itself first), without their names: what's under a cut bone,
    /// walked every frame (what's attached under it can change: a weapon taken from a hand).
    fn under_node(root: usize, out: &mut Vec<usize>) {
        out.clear();
        out.push(root);
        let mut i = 0;
        while i < out.len() && out.len() < 400 {
            let n = out[i];
            if is_node(n) {
                let data = mem::deref(n + CHILDREN);
                let count = mem::at::<u16>(n + CHILD_COUNT).unwrap_or(0).min(256) as usize;
                for k in 0..count {
                    let c = mem::deref(data + k * 4);
                    if c != 0 && mem::is_readable(c as *const u8, 0xB8) {
                        out.push(c);
                    }
                }
            }
            i += 1;
        }
    }

    /// The indices of `tree`'s nodes under `top` (itself included).
    fn under(tree: &[Node], top: usize) -> Vec<bool> {
        let mut inside = vec![false; tree.len()];
        inside[top] = true;
        for i in top + 1..tree.len() {
            if let Some(p) = tree[i].parent {
                inside[i] = inside[p];
            }
        }
        inside
    }

    fn actor_value(actor: usize, av: u32) -> f32 {
        type F = unsafe extern "thiscall" fn(usize, u32) -> i32;
        let f = mem::deref(mem::deref(actor) + VT_GET_AV);
        if !CODE.contains(&f) {
            return 0.0;
        }
        unsafe { std::mem::transmute::<usize, F>(f)(actor, av) as f32 }
    }

    /// An actor's health now (the game's).
    pub fn health(actor: usize) -> f32 {
        actor_value(actor, AV_HEALTH)
    }

    /// What the game says an actor bleeds: its decal texture and its spray model.
    fn blood_paths(actor: usize) -> Option<(String, String)> {
        type F = unsafe extern "thiscall" fn(usize) -> usize;
        type B = unsafe extern "thiscall" fn(usize) -> u8;
        let vt = mem::deref(actor);
        let (get, has) = (mem::deref(vt + VT_BASE), mem::deref(vt + VT_HAS_BASE));
        if !CODE.contains(&get) || !CODE.contains(&has) {
            return None;
        }
        let base = unsafe { std::mem::transmute::<usize, F>(get)(actor) };
        if base == 0 || unsafe { std::mem::transmute::<usize, B>(has)(actor) } == 0 {
            return None;
        }
        let part = base + BLOOD_PART;
        let pvt = mem::deref(part);
        let (decal, spray) = (mem::deref(pvt + VT_BLOOD_DECAL), mem::deref(pvt + VT_BLOOD_SPRAY));
        if !CODE.contains(&decal) || !CODE.contains(&spray) {
            return None;
        }
        let d = unsafe { std::mem::transmute::<usize, F>(decal)(part) };
        let s = unsafe { std::mem::transmute::<usize, F>(spray)(part) };
        Some((crate::combat::read_str(d), crate::combat::read_str(s)))
    }

    /// A limb cut off one of the game's creatures.
    struct Sever {
        limb: &'static Limb,
        /// The creature (a corpse may leave the game's list of those it's processing: it's kept
        /// while it's still this creature with this 3D).
        actor: usize,
        /// The creature's 3D it was cut from (another means it was loaded again: forgotten), and
        /// its bones either side of the cut (the skeleton's: they last as long as the 3D does).
        root: usize,
        cut_node: usize,
        axis_nodes: (usize, usize),
        /// The piece: its object, the clone that shows it, and the clone's nodes with their world
        /// transforms at the cut (the collapsed ones already collapsed).
        object: u32,
        clone: usize,
        attached_to: usize,
        clone_nodes: Vec<(usize, Xf, [f32; 4])>,
        /// The cut bone's world transform at the cut.
        cut: Xf,
        desc: ObjectDesc,
        born: f64,
        next_spray: f64,
    }

    struct State {
        /// The kinds of blood met so far: (decal, spray) -> index.
        kinds: Vec<(String, String)>,
        table: Box<BloodTable>,
        table_dirty: bool,
        by_base: HashMap<usize, u32>,
        severs: HashMap<u32, Vec<Sever>>,
        next_object: u32,
        bodies: Box<GoreBodies>,
        states: Box<ObjectStates>,
        pending: Vec<GoreEvent>,
        objects_dirty: bool,
        frames: u64,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        let s = s.get_or_insert_with(|| State {
            kinds: Vec::new(),
            table: bytemuck::zeroed_box(),
            table_dirty: false,
            by_base: HashMap::new(),
            severs: HashMap::new(),
            next_object: PIECE_IDS,
            bodies: bytemuck::zeroed_box(),
            states: bytemuck::zeroed_box(),
            pending: Vec::new(),
            objects_dirty: false,
            frames: 0,
        });
        f(s)
    }

    /// The blood an actor bleeds: an index into the table (`u32::MAX`: none).
    fn blood_of(s: &mut State, actor: usize) -> u32 {
        let base = mem::deref(actor + 0x1C);
        if let Some(&k) = s.by_base.get(&base) {
            return k;
        }
        let k = match blood_paths(actor) {
            Some((decal, spray)) => match look(&decal, &spray) {
                Some((l, own)) => {
                    let key = (decal.to_ascii_lowercase(), spray.to_ascii_lowercase());
                    match s.kinds.iter().position(|x| *x == key) {
                        Some(i) => i as u32,
                        None if s.kinds.len() < MAX_BLOODS => {
                            let i = s.kinds.len();
                            s.kinds.push(key);
                            let name = |p: &str| if p.is_empty() { String::new() } else { format!("oblivion/{}", oblivion_assets::export::texture_key(p)) };
                            let b = &mut s.table.bloods[i];
                            b.flags = l.flags;
                            b.amount = 1.0;
                            b.viscosity = 1.0;
                            b.atlas = l.atlas;
                            let splat = own.unwrap_or_else(|| l.splat.to_owned());
                            Blood::set_texture(&mut b.splat, &name(&splat));
                            Blood::set_texture(&mut b.splatter, &name(l.splatter));
                            Blood::set_texture(&mut b.stain, &name(l.stain));
                            Blood::set_texture(&mut b.drip, &name(l.drip));
                            Blood::set_texture(&mut b.drop, &name(l.drop));
                            Blood::set_texture(&mut b.mist, &name(l.mist));
                            Blood::set_texture(&mut b.flesh, &name(l.flesh));
                            s.table.count = s.kinds.len() as u32;
                            s.table_dirty = true;
                            tracing::info!("gore: blood {i}: the game's {decal:?} and {spray:?} -> splat {}", Blood::texture(&b.splat));
                            i as u32
                        }
                        None => 0,
                    }
                }
                None => u32::MAX,
            },
            None => u32::MAX,
        };
        s.by_base.insert(base, k);
        k
    }

    /// Whether `bone` is cut off `actor` (it, or a bone above it, was cut): its capsule isn't
    /// offered for hits any more.
    pub fn cut_off(actor: u32, bone: &str) -> bool {
        with_state(|s| {
            s.severs.get(&actor).is_some_and(|list| {
                list.iter().any(|v| {
                    let Some(top) = bone_index(v.limb.bone) else { return false };
                    bone_index(bone).is_some_and(|b| below(b, top))
                })
            })
        })
    }

    /// Whether `actor` was cut at `bone` itself (its capsule ends at the stump).
    pub fn cut_at(actor: u32, bone: &str) -> bool {
        with_state(|s| s.severs.get(&actor).is_some_and(|l| l.iter().any(|v| v.limb.bone == bone)))
    }

    /// The skeleton's limbs as a chain, for "is under": each limb bone's parent limb bone.
    const CHAINS: &[&[&str]] = &[
        &["Bip01 Neck1", "Bip01 Head"],
        &["Bip01 L UpperArm", "Bip01 L Forearm", "Bip01 L Hand"],
        &["Bip01 R UpperArm", "Bip01 R Forearm", "Bip01 R Hand"],
        &["Bip01 L Thigh", "Bip01 L Calf", "Bip01 L Foot"],
        &["Bip01 R Thigh", "Bip01 R Calf", "Bip01 R Foot"],
    ];

    fn bone_index(b: &str) -> Option<(usize, usize)> {
        CHAINS.iter().enumerate().find_map(|(c, chain)| chain.iter().position(|x| *x == b).map(|i| (c, i)))
    }

    fn below(b: (usize, usize), top: (usize, usize)) -> bool {
        b.0 == top.0 && b.1 >= top.1
    }

    /// Right after the game's own hit on `actor` (its health `before` it): whether the blow took a
    /// limb, and if it did, it's cut.
    pub fn after_hit(actor: usize, ev: &CombatEvent, before: f32) {
        crate::hooks::guard_void("gore", || hit(actor, ev, before));
    }

    fn hit(actor: usize, ev: &CombatEvent, before: f32) {
        if ev.object == 0 {
            return;
        }
        let after = health(actor);
        let (killed, dead_before) = (before > 0.0 && after <= 0.0, before <= 0.0);
        tracing::info!("gore: a hit on {:#x} (part {}, kind {}, strength {:.2}): health {before:.1} -> {after:.1}", mem::at::<u32>(actor + FORM_ID).unwrap_or(0), ev.part, ev.what, ev.strength);
        if !(killed || dead_before) || ev.what != DMG_SLASH {
            return;
        }
        let Some(w) = crate::weapons::equipped() else { return };
        let root = mem::deref(actor + REFR_NODE);
        let Some(id) = mem::at::<u32>(actor + FORM_ID) else { return };
        if root == 0 {
            return;
        }
        // Which part it struck (the capsule the combat offered), and where along it.
        let Some((from, to)) = crate::combat::part_bones(ev.part) else { return };
        let bones = crate::combat::bones(root);
        let (Some(&fa), Some(&fb)) = (bones.get(from), bones.get(to)) else { return };
        let (Some(a), Some(b)) = (Xf::read(fa), Xf::read(fb)) else { return };
        let basis = camera::basis();
        let (pa, pb) = (basis.point_from_game(a.t), basis.point_from_game(b.t));
        let p = Vec3::from_array(ev.point);
        let t = (p - pa).dot(pb - pa) / (pb - pa).length_squared().max(1e-6);
        let Some(limb) = joint_for(from, to, t, pa.distance(pb)) else {
            tracing::info!("gore: {from} -> {to} struck {t:.2} along: no joint to cut there");
            return;
        };
        if cut_off(id, limb.bone) {
            return;
        }
        let power = cut_power(ev.strength, weapon_cut(w.damage as f32, w.kind, w.reach));
        let cuts = severs(limb.limb, ev.what, ev.flags, killed, dead_before, power);
        tracing::info!(
            "gore: {:#x}'s {} struck by {} ({} damage, kind {}) at strength {:.2}: power {:.2} against {:.2}{}, health {before:.0} -> {after:.0}: {}",
            id,
            limb.bone,
            w.name,
            w.damage,
            w.kind,
            ev.strength,
            power,
            toughness(limb.limb),
            if dead_before { " (a corpse)" } else { "" },
            if cuts { "cut off" } else { "holds" }
        );
        if cuts {
            sever(actor, id, root, limb, ev);
        }
    }

    /// Cuts `limb` off the actor: the clone that's the piece, the piece's object, the event.
    fn sever(actor: usize, id: u32, root: usize, limb: &'static Limb, ev: &CombatEvent) {
        let nodes = tree(root);
        // (A creature's skeleton may have no neck's base: its head is cut at the head.)
        let limb = if limb.limb == LIMB_HEAD && !nodes.iter().any(|n| n.name == limb.bone) { &HEAD_ALONE } else { limb };
        if cut_off(id, limb.bone) {
            return;
        }
        let Some(top) = nodes.iter().position(|n| n.name == limb.bone) else {
            tracing::info!("gore: {id:#x} has no {} to cut", limb.bone);
            return;
        };
        let find = |name: &str| nodes.iter().find(|n| n.name == name).map(|n| n.ptr);
        let (Some(axis_from), Some(axis_to)) = (find(limb.axis.0), find(limb.axis.1)) else { return };
        let (Some(cut), Some(from), Some(to)) = (Xf::read(nodes[top].ptr), Xf::read(axis_from), Xf::read(axis_to)) else { return };
        let inside = under(&nodes, top);
        let skeleton = nodes.iter().position(|n| n.name == "Bip01").map(|i| under(&nodes, i)).unwrap_or_else(|| vec![false; nodes.len()]);
        let basis = camera::basis();
        // The piece's frame: the cut bone's, in the runtime's world.
        let pos = basis.point_from_game(cut.t);
        let rot = basis.rot_from_game(cut.r).normalize();
        let inv = rot.inverse();
        let axis = (to.t - from.t).normalize_or(cut.r.x_axis);
        let out = basis.dir_from_game(axis).normalize_or(Vec3::Y);
        // Its shape: a box round its bones (in its frame), and the limb's reach past the last.
        let mut pts: Vec<Vec3> = nodes.iter().enumerate().filter(|(i, n)| inside[*i] && n.name.starts_with("Bip01")).filter_map(|(_, n)| Xf::read(n.ptr)).map(|x| inv * (basis.point_from_game(x.t) - pos)).collect();
        let far = pts.iter().copied().fold(Vec3::ZERO, |m, p| if p.length() > m.length() { p } else { m });
        pts.push(far + inv * out * limb.reach);
        pts.push(Vec3::ZERO);
        let Some((centre, half)) = crate::items::bounds(&pts) else { return };
        // (A head is round, wider than its neck.)
        let half = half + Vec3::splat(if limb.limb == LIMB_HEAD { 0.09 } else { limb.radius * 0.9 });
        let mass = piece_mass(&nodes, &inside).unwrap_or(limb.mass);
        let s = with_state(|s| {
            let object = s.next_object;
            s.next_object += 1;
            let blood = blood_of(s, actor);
            (object, blood)
        });
        let (object, blood) = s;
        let mut desc = crate::items::object_desc(object, 1, (pos, rot), centre, half, mass, false);
        desc.flags = OBJF_GRABBABLE | OBJF_PULLABLE;
        // The clone: the actor's 3D as it is, with no controllers or collision (nothing of it is
        // animated or simulated by the game: it's drawn where the runtime has the piece).
        let clone = clone_bare(&nodes);
        let mut sv = Sever { limb, actor, root, cut_node: nodes[top].ptr, axis_nodes: (axis_from, axis_to), object, clone: 0, attached_to: 0, clone_nodes: Vec::new(), cut, desc, born: crate::link::now(), next_spray: 0.0 };
        if clone != 0 {
            let copy = tree(clone);
            if copy.len() == nodes.len() && copy.iter().zip(&nodes).all(|(c, n)| c.name == n.name) {
                // Every bone but the cut ones shrunk to nothing at the joint; the rest as it was.
                let collapse = Xf { r: cut.r, t: cut.t, s: NOTHING };
                sv.clone_nodes = copy
                    .iter()
                    .zip(&nodes)
                    .enumerate()
                    .filter_map(|(i, (c, n))| {
                        let x = Xf::read(n.ptr)?;
                        let bound: [f32; 4] = mem::at(n.ptr + BOUND)?;
                        Some((c.ptr, if skeleton[i] && !inside[i] { collapse } else { x }, bound))
                    })
                    .collect();
                // Only the meshes that show the piece are drawn.
                let piece: std::collections::HashSet<usize> = copy.iter().enumerate().filter(|(i, _)| inside[*i]).map(|(_, c)| c.ptr).collect();
                let mut hidden = 0;
                for (i, c) in copy.iter().enumerate() {
                    if is_a(c.ptr, "NiGeometry") && !shows_piece(c.ptr, inside[i], &piece) {
                        unsafe {
                            let f = (c.ptr + av::FLAGS) as *mut u16;
                            f.write(f.read() | av::APP_CULLED);
                        }
                        hidden += 1;
                    }
                }
                tracing::info!("gore: the piece's clone hides {hidden} meshes that don't show it");
                let attach_to = mem::deref(root + PARENT);
                if attach_to != 0 && is_node(attach_to) {
                    // (A reference of ours keeps it whatever happens to where it's attached.)
                    unsafe { (*((clone + 4) as *const std::sync::atomic::AtomicU32)).fetch_add(1, std::sync::atomic::Ordering::AcqRel) };
                    type Attach = unsafe extern "thiscall" fn(usize, usize, u8);
                    unsafe { std::mem::transmute::<usize, Attach>(mem::deref(mem::deref(attach_to) + VT_ATTACH))(attach_to, clone, 1) };
                    sv.clone = clone;
                    sv.attached_to = attach_to;
                    place_clone(&sv, &cut);
                }
            } else {
                tracing::warn!("gore: the clone of {id:#x}'s 3D doesn't match it ({} nodes against {})", copy.len(), nodes.len());
            }
        }
        tracing::info!("gore: {id:#x} lost its {} ({} nodes cloned), the piece {object:#x}: {:.2} kg, box {:.2?} at {:.2?}", limb.bone, sv.clone_nodes.len(), mass, half, centre);
        let ev = GoreEvent {
            kind: GE_SEVER,
            actor: id,
            limb: limb.limb,
            object,
            at: pos.to_array(),
            radius: limb.radius,
            out: out.to_array(),
            blood,
            cap: [0.0; 3],
            strength: ev.strength,
            cap_out: (inv * -out).to_array(),
            vel: [0.0; 3],
            push_speed: ev.speed,
            push: ev.dir,
            ..Default::default()
        };
        with_state(|s| {
            s.severs.entry(id).or_default().push(sv);
            s.pending.push(ev);
            s.objects_dirty = true;
        });
    }

    /// Tests (`GoreCut <bone>`, `crate::console`): the nearest creature killed by the game (its
    /// `kill`; not with `kill` false) and the limb at `bone` cut off it, as a full-strength blow of
    /// the blade would.
    pub fn test_cut(bone: &str, kill: bool) -> bool {
        // (`Bip01 Head` names the head too: it's cut at its neck.)
        let Some(limb) = LIMBS.iter().find(|l| l.bone.eq_ignore_ascii_case(bone) || (l.limb == LIMB_HEAD && bone.eq_ignore_ascii_case("Bip01 Head"))) else { return false };
        let player = engine::player();
        let Some(me) = mem::at::<[f32; 3]>(player + engine::refr::POS).map(Vec3::from) else { return false };
        let nearest = crate::combat::known_actors().into_iter().filter(|&(_, a)| a != player).filter_map(|(id, a)| Some((id, a, mem::at::<[f32; 3]>(a + engine::refr::POS).map(Vec3::from)?.distance(me)))).min_by(|a, b| a.2.total_cmp(&b.2));
        let Some((id, actor, _)) = nearest else {
            tracing::info!("gore: no creature near to cut");
            return false;
        };
        let root = mem::deref(actor + REFR_NODE);
        if root == 0 || cut_off(id, limb.bone) {
            return false;
        }
        if kill {
            crate::console::run_script(&format!("{id:08X}.kill"));
        }
        let at = crate::combat::bones(root).get(limb.bone).and_then(|&n| Xf::read(n)).map(|x| camera::basis().point_from_game(x.t)).unwrap_or_default();
        let ev = CombatEvent { kind: CE_HIT, actor: id, object: crate::weapons::WEAPON_ID, what: DMG_SLASH, strength: 1.0, speed: 6.0, point: at.to_array(), dir: [1.0, 0.0, 0.0], ..Default::default() };
        tracing::info!("gore: a test cut: {id:#x}'s {}", limb.bone);
        sever(actor, id, root, limb, &ev);
        true
    }

    /// The piece's mass: the game's ragdoll bodies under the cut (their Havok masses), scaled so a
    /// whole body weighs 75 kg (the game's add up to about 630).
    fn piece_mass(nodes: &[Node], inside: &[bool]) -> Option<f32> {
        let mut sum = 0.0;
        for (i, n) in nodes.iter().enumerate().filter(|(i, _)| inside[*i]) {
            let _ = i;
            let co = mem::deref(n.ptr + COLLISION);
            let hk = if co != 0 { mem::deref(mem::deref(co + 0x10) + 0x8) } else { 0 };
            let motion = if hk != 0 { mem::deref(hk + 0x50) } else { 0 };
            if let Some(inv) = mem::at::<f32>(motion + 0xC0).filter(|&m| motion != 0 && m > 1e-4 && m.is_finite()) {
                sum += 1.0 / inv;
            }
        }
        (sum > 0.0).then(|| (sum * 75.0 / 630.0).clamp(0.3, 15.0))
    }

    /// A clone of the tree's root without its controllers and collision (taken off the originals
    /// for the clone and put back).
    fn clone_bare(nodes: &[Node]) -> usize {
        let saved: Vec<(usize, usize, usize)> = nodes.iter().map(|n| (n.ptr, mem::deref(n.ptr + CONTROLLER), mem::deref(n.ptr + COLLISION))).collect();
        for &(p, ..) in &saved {
            unsafe {
                ((p + CONTROLLER) as *mut usize).write(0);
                ((p + COLLISION) as *mut usize).write(0);
            }
        }
        type Clone = unsafe extern "thiscall" fn(usize) -> usize;
        let clone = unsafe { std::mem::transmute::<usize, Clone>(CLONE)(nodes[0].ptr) };
        for &(p, c, k) in &saved {
            unsafe {
                ((p + CONTROLLER) as *mut usize).write(c);
                ((p + COLLISION) as *mut usize).write(k);
            }
        }
        if clone != 0 {
            // Not hidden, not parented (the original's parent pointer came with it).
            unsafe {
                let f = (clone + av::FLAGS) as *mut u16;
                f.write(f.read() & !av::APP_CULLED);
                ((clone + PARENT) as *mut usize).write(0);
            }
        }
        clone
    }

    /// The clone where the cut bone is now `at` (game world): every node moved with it.
    fn place_clone(sv: &Sever, at: &Xf) {
        // The move from where the bone was cut to where it is.
        let r = at.r * sv.cut.r.transpose();
        let m = Xf { r, t: at.t - r * sv.cut.t, s: 1.0 };
        for &(node, x, bound) in &sv.clone_nodes {
            m.then(&x).write(node);
            let c = m.r * Vec3::new(bound[0], bound[1], bound[2]) + m.t;
            unsafe { ((node + BOUND) as *mut [f32; 4]).write_unaligned([c.x, c.y, c.z, bound[3]]) };
        }
    }

    /// Lets go of a piece's clone: off where it was attached (if that's still there: a node that
    /// goes lets go of its children), and our reference given back.
    fn drop_clone(sv: &mut Sever) {
        if sv.clone == 0 {
            return;
        }
        let clone = sv.clone;
        sv.clone = 0;
        if sv.attached_to != 0 && mem::deref(clone + PARENT) == sv.attached_to && is_node(sv.attached_to) {
            type Detach = unsafe extern "thiscall" fn(usize, *mut usize, usize) -> *mut usize;
            let mut out = 0usize;
            unsafe { std::mem::transmute::<usize, Detach>(mem::deref(mem::deref(sv.attached_to) + VT_DETACH))(sv.attached_to, &mut out, clone) };
            if out != 0 {
                release(out);
            }
        } else {
            unsafe { ((clone + PARENT) as *mut usize).write(0) };
        }
        release(clone);
    }

    fn release(obj: usize) {
        let rc = unsafe { &*((obj + 4) as *const std::sync::atomic::AtomicU32) };
        if rc.fetch_sub(1, std::sync::atomic::Ordering::AcqRel) == 1 {
            type Del = unsafe extern "thiscall" fn(usize, u32);
            unsafe { std::mem::transmute::<usize, Del>(mem::deref(mem::deref(obj)))(obj, 1) };
        }
    }

    /// The creature's own blood effect at `at` (game units) along `dir`: its spray model as the
    /// game's hit makes it.
    fn spray(actor: usize, at: Vec3, dir: Vec3) {
        let Some((_, model)) = blood_paths(actor) else { return };
        if model.is_empty() {
            return;
        }
        let cell = mem::deref(actor + REFR_CELL);
        let cell_node = mem::deref(cell + CELL_NODE);
        if cell == 0 || cell_node == 0 || !is_node(cell_node) {
            return;
        }
        type Quarter = unsafe extern "C" fn(usize) -> i32;
        let q = unsafe { std::mem::transmute::<usize, Quarter>(CELL_QUARTER)(actor) }.max(0) as usize + 2;
        let child = |n: usize, i: usize| if mem::at::<u16>(n + CHILD_COUNT).unwrap_or(0) as usize > i { mem::deref(mem::deref(n + CHILDREN) + i * 4) } else { 0 };
        let quarter = child(cell_node, q);
        let node = if quarter != 0 && mem::at::<u16>(quarter + CHILD_COUNT).unwrap_or(0) > 3 { child(quarter, 3) } else { 0 };
        let path = std::ffi::CString::new(model).unwrap_or_default();
        type Alloc = unsafe extern "C" fn(usize) -> usize;
        type Make = unsafe extern "thiscall" fn(usize, usize, f32, usize, *const i8, f32, f32, f32, f32, f32, f32, f32, i32) -> usize;
        type Add = unsafe extern "thiscall" fn(usize, usize);
        let d = dir.normalize_or(Vec3::Z);
        unsafe {
            let mem = std::mem::transmute::<usize, Alloc>(ALLOC)(0x20);
            if mem == 0 {
                return;
            }
            let fx = std::mem::transmute::<usize, Make>(TEMP_PARTICLE)(mem, cell, 1.0, node, path.as_ptr(), d.x, d.y, d.z, at.x, at.y, at.z, 1.0, 1);
            static SAID: std::sync::Once = std::sync::Once::new();
            SAID.call_once(|| tracing::info!("gore: the game's blood effect {:?} from a stump (under node {node:#x})", path));
            // (The game's model cache keeps the path's model; the string is copied.)
            std::mem::transmute::<usize, Add>(ADD_TEMP_EFFECT)(PROCESS_MANAGER, fx);
        }
    }

    /// The game's own blood decals last as long as the runtime's (its settings, in memory: the
    /// player's `Oblivion.ini` is left as it is).
    fn game_decals_last() {
        let named = |at: usize, name: &str| crate::combat::read_str(mem::deref(at + 4)).starts_with(name);
        if named(DECAL_LIFETIME, "fDecalLifetime") && mem::at::<f32>(DECAL_LIFETIME).is_some_and(|v| v > 0.0 && v < GAME_DECALS_LAST) {
            unsafe { (DECAL_LIFETIME as *mut f32).write(GAME_DECALS_LAST) };
            tracing::info!("gore: the game's own blood decals last {GAME_DECALS_LAST} s");
        }
        if named(DECALS_PER_FRAME, "iMaxDecalsPerFrame") && mem::at::<i32>(DECALS_PER_FRAME).is_some_and(|v| v < GAME_DECALS_PER_FRAME) {
            unsafe { (DECALS_PER_FRAME as *mut i32).write(GAME_DECALS_PER_FRAME) };
        }
    }

    /// Once a frame, before the eyes: the bodies' blood and stumps for the runtime, the cut limbs
    /// shrunk away on the bodies, the pieces' clones where the runtime has the pieces.
    pub fn frame(view: Option<&rustvr_adapter::View>, now: f64) {
        crate::hooks::guard_void("gore", || frame_inner(view, now));
    }

    /// Whether `actor` is still the game's creature `id` with its 3D `root`.
    fn still(actor: usize, id: u32, root: usize) -> bool {
        const ACTOR_VTABLES: [usize; 2] = [0x00A6_FC9C, 0x00A7_10F4];
        mem::is_readable(actor as *const u8, 0x60) && ACTOR_VTABLES.contains(&mem::deref(actor)) && mem::at::<u32>(actor + FORM_ID) == Some(id) && mem::deref(actor + REFR_NODE) == root
    }

    fn frame_inner(view: Option<&rustvr_adapter::View>, now: f64) {
        let mut actors = crate::combat::known_actors();
        let basis = camera::basis();
        with_state(|s| {
            // (The bodies the gore has cut, whether or not the game still processes them.)
            for (id, list) in &s.severs {
                if let Some(v) = list.first()
                    && still(v.actor, *id, v.root)
                {
                    actors.entry(*id).or_insert(v.actor);
                }
            }
            let have_pieces = s.severs.values().any(|l| l.iter().any(|v| v.clone != 0));
            if have_pieces {
                crate::link::with_adapter(|a| a.read_object_states(&mut s.states));
            }
            // Forget what's gone (the body unloaded, or its 3D made again).
            let mut gone = Vec::new();
            for (id, list) in s.severs.iter_mut() {
                list.retain_mut(|v| {
                    let ok = still(v.actor, *id, v.root);
                    if !ok {
                        drop_clone(v);
                    }
                    ok
                });
                if list.is_empty() {
                    gone.push(*id);
                }
            }
            for id in gone {
                tracing::info!("gore: {id:#x}'s body is gone: its pieces go too");
                s.severs.remove(&id);
                s.objects_dirty = true;
            }
            // The bodies.
            let mut bodies: Box<GoreBodies> = bytemuck::zeroed_box();
            for (&id, &actor) in actors.iter().take(MAX_GORE_BODIES) {
                let i = bodies.count as usize;
                let b = &mut bodies.bodies[i];
                b.id = id;
                b.blood = blood_of(s, actor);
                b.flags = if health(actor) > 0.0 { GB_ALIVE } else { 0 };
                if let Some(list) = s.severs.get_mut(&id) {
                    let mut nodes = Vec::new();
                    for v in list.iter_mut() {
                        let cut_node = v.cut_node;
                        let (Some(cut), Some(from), Some(to)) = (Xf::read(cut_node), Xf::read(v.axis_nodes.0), Xf::read(v.axis_nodes.1)) else { continue };
                        // (The axis from bones on the body: a severed head's own is shrunk away.)
                        let to_t = if v.axis_nodes.1 == cut_node || to.s < 0.01 { cut.t + (cut.t - from.t) } else { to.t };
                        let axis = (to_t - from.t).normalize_or(cut.r.x_axis);
                        if (b.count as usize) < MAX_STUMPS {
                            b.stumps[b.count as usize] = Stump { at: basis.point_from_game(cut.t).to_array(), radius: v.limb.radius, out: basis.dir_from_game(axis).normalize_or(Vec3::Y).to_array(), limb: v.limb.limb };
                            b.count += 1;
                        }
                        // The game's own blood effect from the stump, with the pulses.
                        if now - v.born < SPRAYS_FOR && now >= v.next_spray {
                            v.next_spray = now + SPRAY_EVERY;
                            spray(actor, cut.t, axis);
                        }
                        // Shrunk to nothing at the joint on the body: the bone and all under it.
                        let collapse = Xf { r: cut.r, t: cut.t, s: NOTHING };
                        under_node(cut_node, &mut nodes);
                        for &n in &nodes {
                            collapse.write(n);
                            unsafe { ((n + BOUND) as *mut [f32; 4]).write_unaligned([cut.t.x, cut.t.y, cut.t.z, 1.0]) };
                        }
                        // The piece's clone where the runtime has it.
                        if v.clone != 0
                            && let Some(st) = s.states.states[..(s.states.count as usize).min(MAX_OBJECTS)].iter().find(|st| st.id == v.object)
                        {
                            let tracking = st.flags & OS_TRACKING != 0;
                            let (pos, rot) = match (tracking, view) {
                                (true, Some(view)) => view.pose(&st.pose),
                                (true, None) => continue,
                                _ => (Vec3::from_array(st.pose.pos), Quat::from_array(st.pose.rot)),
                            };
                            if pos.is_finite() && rot.is_finite() && rot.length_squared() > 1e-6 {
                                let at = Xf { r: basis.rot_to_game(rot.normalize()), t: basis.point_to_game(pos), s: v.cut.s };
                                place_clone(v, &at);
                            }
                        }
                    }
                }
                bodies.count += 1;
            }
            let pieces: Vec<ObjectDesc> = s.severs.values().flat_map(|l| l.iter().map(|v| v.desc)).collect();
            let events = std::mem::take(&mut s.pending);
            s.frames += 1;
            if s.frames % 300 == 1 {
                game_decals_last();
            }
            let table = (s.table_dirty || (s.table.count > 0 && s.frames % 120 == 0)).then(|| s.table.clone());
            s.table_dirty = false;
            let objects = std::mem::take(&mut s.objects_dirty);
            crate::link::with_adapter(|a| {
                if let Some(t) = table {
                    a.write_bloods(&t);
                }
                if objects {
                    crate::objects::publish(crate::objects::Owner::Gore, &pieces, a);
                }
                a.write_gore_bodies(&bodies);
                for ev in &events {
                    a.push_gore_event(ev);
                }
            });
            s.bodies = bodies;
        });
    }
}
