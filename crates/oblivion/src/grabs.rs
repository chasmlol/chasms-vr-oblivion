//! Oblivion's side of grabbing people and creatures (the framework's `rustvr_protocol::grab`):
//! a hand on any part of an actor moves it with the game's own Havok ragdoll, the skeleton's bodies
//! (`docs/research/npcs.md`).
//!
//! * **What grabbing needs** ([`publish`], with the creatures every frame): each part's rotation
//!   (its bone's), velocity and mass (its Havok body's), the body's mass, whether it's all Havok's
//!   now (dead, knocked down), and how strong the player is (the game's Strength: 50 is 1).
//! * **A grab** ([`apply`], before the game's frame) is the game's own grab, as holding Activate on
//!   a dead body does: a `bhkMouseSpringAction` on the Havok body of the bone held, at the point the
//!   hand has, pulled every frame to where the hand would have it, no harder than the hand pulls.
//!   A dead or knocked-down body is all Havok's, so any part of it drags the whole body.
//! * **A living actor** keeps animating. The bones from the one held out to the limb's end go
//!   physical (their blend objects' heir and velocity gains 0: each bone where its body is), and
//!   the spine partly (a heir between: it leans after a pulled limb, softly, and comes back): the
//!   limb hangs from the body that's still animated and the hand moves it. Let go, they blend back
//!   to the animation.
//! * **Off its feet**: pulled up by the head, the neck or the torso past where the standing body
//!   goes, or yanked far from where its body is, it's knocked down: the game's own knockback
//!   (`KnockbackActor`, every bone Havok's), and then the whole body hangs from the hand. While a
//!   hand holds it, it can't get up (its knockdown's time is kept fresh); let go, it gets up when
//!   the game says.
//! * **Pulled along the ground** while it stands, it's pushed after the pull (its character
//!   controller's push, the game's knockback without a ragdoll), stumbling along.
//! * **Choking**: the throat squeezed drains its fatigue (`DamageActorValue`, faster off its feet)
//!   until the game knocks it out itself; it stays out while it's choked. Held up by the throat and
//!   still conscious, it struggles: its hands go for the hand and its legs kick, weaker as it tires.
//! * **The reaction is the game's**: a living actor choked, knocked down, thrown or held a while
//!   has been attacked by the player (`OnAttacked`, as at the end of every blow: its disposition,
//!   tolerance, fleeing, combat and the assault crime).
//! * **Throws**: let go, every body keeps what the spring gave it, and gets the hand's throw on top,
//!   as much of it as the player's pull could give that mass.
//! * **Pushes** (the runtime's `GamePushes` on creatures' parts): a body that's all physics gets what
//!   its bone lacks of the push. A standing one gives under it and springs back, wobbling: the bones
//!   about the touch go partly physical, driven back to the animation by their own blend objects
//!   ([`touch`]), and the body nearest the push gets it. An open hand coming in fast shoves it too.
//!
//! Engine facts (`Oblivion.exe` 1.2.0.416; thiscall unless said): a skeleton's bone with a body has
//! a `bhkBlendCollisionObject` at its node's +0xA8 (vtables 0xA9643C and 0xA96384): its flags +0xC,
//! its `bhkRigidBody` +0x10 (the `hkRigidBody` at +8 of that), its heir gain +0x14 and velocity gain
//! +0x18. Every frame its update keyframes the body to the bone (heir 1), puts the bone where the
//! body is (heir 0, velocity 0: a ragdoll), or blends the two (between), switching the body's motion
//! type itself. `hkRigidBody`: its world +8, its filter info +0x30 (layer bits 0-6, 8 the biped;
//! the body part bits 8-12), its motion +0x50 (a pointer: transform +0x10, translation +0x40, the
//! centre of mass +0x60, inverse mass +0xC0, velocities +0xD0 and +0xE0). Havok's unit is 7 game
//! units. A living actor's bodies aren't in Havok's world while they're keyframed; 0x88CE30 (cdecl:
//! root, on, recurse, force) puts them all in and takes them out again.

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn publish(_a: &rustvr_adapter::Adapter, _table: &rustvr_protocol::Actors, _by_id: &std::collections::HashMap<u32, usize>) {}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn apply(_now: f64) {}

/// Body part codes (a body's filter info, bits 8-12; nif.xml's order).
pub const HEAD: u32 = 1;
pub const PELVIS: u32 = 2;
pub const SPINE1: u32 = 3;
pub const SPINE2: u32 = 4;
pub const TAIL: u32 = 17;

/// The bones that go physical when a living actor is held by the bone with this part code (out to
/// the limb's end), and the ones that bend after it softly (the spine).
pub fn limb(code: u32) -> (&'static [u32], &'static [u32]) {
    match code {
        HEAD => (&[HEAD], &[SPINE2]),
        PELVIS | SPINE1 | SPINE2 => (&[], &[SPINE1, SPINE2]),
        5..=7 => (&[5, 6, 7], &[SPINE2]),
        11..=13 => (&[11, 12, 13], &[SPINE2]),
        8..=10 => (&[8, 9, 10], &[]),
        14..=16 => (&[14, 15, 16], &[]),
        TAIL => (&[TAIL], &[]),
        _ => (&[], &[]),
    }
}

/// The parts a hand pulls up to lift the whole body by (the head, the neck, the torso).
pub fn lifts_by(code: u32) -> bool {
    matches!(code, HEAD | PELVIS | SPINE1 | SPINE2)
}

/// A standing actor's bone touched gives under the hand and springs back, as its blend object
/// drives its body back to the animation by its gains (see [`touch`]): (heir, velocity). The heir
/// is how far towards the animation the body's driven each frame (the rest of the bone is where the
/// body is: it bends under the hand), and the velocity gain how much of the body's speed is set to
/// that each frame, the rest kept: below 1 the body carries on past where it's driven and comes
/// back, a wobble. Loose for what hangs (an arm, the head): leant on, it gives most of the way and
/// wobbles back in about a second. Firm for the spine: it gives about a third and is back in a
/// quarter of a second. Stiff for a leg it stands on, and the chest an arm or the head hangs from.
pub const LOOSE: (f32, f32) = (0.2, 0.1);
pub const FIRM: (f32, f32) = (0.5, 0.3);
pub const STIFF: (f32, f32) = (0.7, 0.5);

/// The bones a touch on the bone with this part code moves, and how (see [`LOOSE`]): the head
/// turns on its neck, loosely (only its turn is drawn, at the animation's place, and only so much
/// of it as the heir leaves: firm, a face pushed hardly moved); the chest gives, and the head and
/// the arms swing with it; an arm swings from its shoulder; a leg gives a little. (The pelvis holds
/// the body up: it stays the animation's.)
pub fn touch(code: u32) -> &'static [(u32, (f32, f32))] {
    match code {
        HEAD => &[(HEAD, LOOSE), (SPINE2, STIFF)],
        PELVIS | SPINE1 | SPINE2 => &[(SPINE1, FIRM), (SPINE2, FIRM), (HEAD, LOOSE), (5, LOOSE), (6, LOOSE), (7, LOOSE), (11, LOOSE), (12, LOOSE), (13, LOOSE)],
        5..=7 => &[(5, LOOSE), (6, LOOSE), (7, LOOSE), (SPINE2, STIFF)],
        11..=13 => &[(11, LOOSE), (12, LOOSE), (13, LOOSE), (SPINE2, STIFF)],
        8..=10 => &[(8, STIFF), (9, STIFF), (10, STIFF)],
        14..=16 => &[(14, STIFF), (15, STIFF), (16, STIFF)],
        TAIL => &[(TAIL, LOOSE)],
        _ => &[],
    }
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, av, refr};
    use crate::{camera, combat, mem};
    use glam::{Mat3, Vec3};
    use rustvr_protocol::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const NODE_NAME: usize = 0x08;
    const NODE_CHILDREN: usize = 0xB0;
    const NODE_COUNT: usize = 0xB6;
    const NODE_COLLISION: usize = 0xA8;
    /// `bhkBlendCollisionObject` (and the subclass that adds a rotation).
    const BLEND_VTABLES: [usize; 2] = [0x00A9_643C, 0x00A9_6384];
    const BLEND_BODY: usize = 0x10;
    const BLEND_HEIR: usize = 0x14;
    const BLEND_VEL: usize = 0x18;
    const HK_OBJ: usize = 0x8;
    const BODY_WORLD: usize = 0x8;
    const BODY_FILTER: usize = 0x30;
    const BODY_MOTION: usize = 0x50;
    const MOTION_ROT: usize = 0x10;
    const MOTION_POS: usize = 0x40;
    const MOTION_COM: usize = 0x60;
    const MOTION_MASS_INV: usize = 0xC0;
    const MOTION_LIN_VEL: usize = 0xD0;
    const MOTION_ANG_VEL: usize = 0xE0;
    const MOTION_SAVED: usize = 0xF0;
    /// The motion's `getInertiaInvWorld` (vtable +0x3C: three columns out) and `applyPointImpulse`
    /// (+0x60: the impulse, the point; both world, Havok's units), as the game's own grab uses them
    /// (`hkMouseSpringAction`'s step, 0x8B8B60).
    const VT_INERTIA_INV_WORLD: usize = 0x3C;
    const VT_POINT_IMPULSE: usize = 0x60;
    const LAYER_BIPED: u32 = 8;
    const BODIES_IN_WORLD: usize = 0x0088_CE30;
    const ACTIVATE: usize = 0x008A_6410;

    /// The game's grab: its allocator (cdecl: size), a mouse spring's construction info (64 bytes,
    /// set to the game's defaults by 0x47F9F0: +4 the body, +0x10 the point in the body, +0x20 the
    /// target in the world, +0x30 damping, +0x34 elasticity, +0x38 the most force relative to the
    /// body's mass, +0x3C the body's own damping), the `bhkMouseSpringAction` made from it
    /// (0x47DE90: 16 bytes, refcounted at +4, its `hkMouseSpringAction` at +8), added to the cell's
    /// `bhkWorld` (vtable +0x5C) and taken out (+0x60); `setMousePosition` (0x8B8A10) moves its
    /// target. The `hkMouseSpringAction`: damping +0x40, elasticity +0x44, most force +0x48.
    const ALLOC: usize = 0x0040_1F00;
    const SPRING_INFO: usize = 0x0047_F9F0;
    const SPRING_NEW: usize = 0x0047_DE90;
    const VT_ADD_TO_WORLD: usize = 0x5C;
    const VT_REMOVE: usize = 0x60;
    const SET_MOUSE: usize = 0x008B_8A10;
    const SPRING_MAX_FORCE: usize = 0x48;
    /// `TESObjectREFR::GetParentCell`, and a cell's `bhkWorld`.
    const PARENT_CELL: usize = 0x006E_CC80;
    const CELL_WORLD: usize = 0x0044_40C0;

    /// `Actor`: its process (+0x58), its 3D (+0x3C); `IsDead(0)` (vtable +0x198), `OnAttacked`
    /// (+0x3A8: attacker, 0), `DamageActorValue` (+0x2A4: value, amount (negative hurts), attacker).
    const PROCESS: usize = 0x58;
    const NI_NODE: usize = 0x3C;
    const FORM_ID: usize = 0x0C;
    const VT_IS_DEAD: usize = 0x198;
    const VT_ON_ATTACKED: usize = 0x3A8;
    const VT_DAMAGE_AV: usize = 0x2A4;
    /// `Character`: the horse it rides (its vtable +0x380 returns it).
    const MOUNT: usize = 0xD4;
    const AV_STRENGTH: u32 = 0;
    const AV_FATIGUE: u32 = 10;
    /// The process: whether it's below the high process (vtable +8: 0 for a high one), its knock
    /// state (+0x11C: 0 standing, 1 knocked down, 2 its lead-in, 3 knocked out, 4 its lead-in,
    /// 5 its get-up queued, 6 getting up), when it was knocked down (+0x14, the game's hour) and its
    /// `KnockbackActor` (+0x2F0: the actor, where the push comes from (game units), the force).
    const VT_NOT_HIGH: usize = 0x08;
    const KNOCK_STATE: usize = 0x11C;
    const KNOCK_TIME: usize = 0x14;
    const VT_KNOCKBACK: usize = 0x2F0;
    /// The game's clock: the hour (`GameHour`'s value).
    const GAME_TIME: usize = 0x00B3_32E0;
    /// The head tracking (`BSFaceGenAnimationData`'s): its measure of where an actor looks (0x54B010;
    /// thiscall: the face animation, the frame's time, the `Bip01 Head` bone, `Bip01 NonAccum`), the
    /// angles from the head bone's world turn now to what it looks at, and its look (0x54B5A0;
    /// thiscall: the face animation, the turn out, the time, the head bone). The look undoes the
    /// head bone's turn and puts its own in its place (its reference bone's turn by the angles), and
    /// the face's update (`BSFaceGenNiNode`'s, 0x55D4C0) turns the head bone by it: whatever the
    /// physics made of the head is gone. (Its own turn has the push in it too: a bone whose body is
    /// dynamic has its blend object write its own turn as well, flag 8, set as its motion changes,
    /// 0x89ED20.) A bone's blend update (0x88F3D0, thiscall: the blend object, its bone at +8) starts
    /// with the bone where its animation has it.
    const HEAD_TRACK_MEASURE: usize = 0x0054_B010;
    const HEAD_TRACK_LOOK: usize = 0x0054_B5A0;
    const BLEND_UPDATE: usize = 0x0088_F3D0;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A3_0000;

    /// Metres in a Havok unit.
    const HK_METRES: f32 = crate::collision::HAVOK_SCALE / camera::UNITS_PER_METER;
    /// A person's mass (kg): Oblivion's ragdolls weigh more than people do (a man about 600 of its
    /// units), so the player's own ragdoll is taken as this, and every body weighs in proportion.
    const PERSON_KG: f32 = 75.0;
    /// The spring: how much of the way it pulls the point each step, how much of its own speed it
    /// takes off there, and how much of the body's speed is kept each step. The game's own grab of
    /// a body (0.2, 0.5, 0.75) drags it a bone at a time; it's measured against the bone held alone,
    /// so a whole body hanging from one bone sags far below it: stiffer, it hangs a hand's breadth
    /// below a hand strong enough to lift it, and livelier, so a swing carries.
    const SPRING_ELASTICITY: f32 = 0.7;
    const SPRING_DAMPING: f32 = 0.6;
    const SPRING_BODY_DAMPING: f32 = 0.85;
    /// A held spine's heir: how much of the animation it keeps each frame (the rest is where the
    /// limb's pull bends it).
    const SPINE_HEIR: f32 = 0.5;
    /// Let go, a living actor's limb blends back to its animation over this long (seconds).
    const BACK_TIME: f64 = 0.4;
    /// A standing actor is knocked down when the hand lifts its head, neck or torso this far above
    /// where the body has it (metres) for this long, or pulls an arm, the head or the torso this
    /// far from it (a yank).
    const LIFT: f32 = 0.12;
    const LIFT_TIME: f64 = 0.12;
    const YANK: f32 = 0.55;
    /// A leg is tripped sooner: pulled this far from where the standing body has it.
    const TRIP: f32 = 0.35;
    /// Let go of faster than this (m/s), a standing actor is thrown down.
    const THROW_DOWN: f32 = 2.5;
    /// How long the hand's pull acts on a throw (seconds): the most it adds to the whole body is
    /// the pull times this, over its mass.
    const THROW_TIME: f32 = 0.3;
    /// Squeezed fully, a throat's fatigue is gone in this long (seconds); off its feet, quicker.
    const CHOKE_TIME: f32 = 6.0;
    const CHOKE_LIFTED: f32 = 1.5;
    const CHOKE_SQUEEZE: f32 = 0.15;
    /// The choke takes its fatigue this far below nothing (a share of its most): out cold for a while
    /// after it's let go (as long as it takes the game to bring its fatigue back up from there: a
    /// quarter of it had it up again in a second and a half), the game's own knockout from fatigue.
    const CHOKED_OUT: f32 = 1.0;
    /// Held this long, a living actor takes it as an attack (seconds).
    const HELD_ATTACK: f64 = 4.0;
    /// A standing actor pulled further than this (metres, along the ground) from where its body has
    /// the point held is pushed after the pull (its character controller's push, the game's own
    /// knockback without a ragdoll): this many m/s for each metre further, at most this fast, a push
    /// this long at a time (seconds).
    const STEP_START: f32 = 0.12;
    const STEP_GAIN: f32 = 6.0;
    const STEP_MAX: f32 = 2.5;
    const STEP_TIME: f32 = 0.1;
    /// Choked and still conscious, it struggles, weaker as its fatigue goes: its hands clutch at the
    /// hand at its throat (drawn to it this fast at most, m/s, this many m/s for each metre away,
    /// this much of the way from how they move each frame), and held off its feet its legs kick every
    /// so often (seconds, and up to this much more), this fast.
    const CLAW_SPEED: f32 = 2.0;
    const CLAW_GAIN: f32 = 6.0;
    const CLAW_BLEND: f32 = 0.35;
    const KICK_EVERY: f64 = 0.25;
    const KICK_SPREAD: f64 = 0.35;
    const KICK_SPEED: f32 = 2.5;
    /// A living actor's arms, which go physical when it's choked standing (they're its own to move,
    /// reaching for the hand).
    const ARMS: [u32; 6] = [5, 6, 7, 11, 12, 13];
    /// `Actor`'s character controller (thiscall: the actor), and its push (thiscall: controller, a
    /// move in game units, over this long).
    const CONTROLLER: usize = 0x0065_A2C0;
    const CONTROLLER_PUSH: usize = 0x0089_07A0;
    /// An open hand (or what it holds) pushing a standing actor this hard (m/s into it) shoves it,
    /// as fast as it's pushed up to the most; slower, it leans on it. A body that's all physics is
    /// pushed however gently.
    const SHOVE: f32 = 1.5;
    const SHOVE_MAX: f32 = 3.0;
    /// A shove carries the actor on this long (seconds): a firm push moves a person about 40 cm.
    const SHOVE_TIME: f32 = 0.25;
    /// Untouched this long (seconds), a touched body's bones blend back to the animation (its
    /// wobble's settled by then).
    const TOUCH_SETTLE: f64 = 1.5;
    /// The bones a struggle moves: the hands, the calves and the feet.
    const HANDS: [u32; 2] = [7, 13];
    const LEGS: [u32; 4] = [9, 10, 15, 16];

    #[repr(C, align(16))]
    #[derive(Clone, Copy, Default)]
    struct V4([f32; 4]);

    #[repr(C, align(16))]
    struct SpringInfo([u8; 64]);

    /// A bone of a skeleton with a Havok body.
    #[derive(Clone, Copy, Debug)]
    struct Bone {
        node: usize,
        blend: usize,
        body: usize,
        code: u32,
    }

    fn read_str(p: usize) -> String {
        combat::read_str(p)
    }

    /// The bones under `root` with bodies on the biped layer.
    fn skeleton(root: usize) -> Vec<Bone> {
        let mut out = Vec::new();
        let mut stack = vec![(root, 0u32)];
        let mut seen = 0;
        while let Some((node, depth)) = stack.pop() {
            seen += 1;
            if seen > 800 || depth > 48 || !mem::is_readable(node as *const u8, 0xB8) {
                continue;
            }
            let name = read_str(mem::deref(node + NODE_NAME));
            let blend = mem::deref(node + NODE_COLLISION);
            if blend != 0 && BLEND_VTABLES.contains(&mem::deref(blend)) {
                let body = mem::deref(mem::deref(blend + BLEND_BODY) + HK_OBJ);
                if body != 0
                    && mem::is_readable(body as *const u8, 0xC4)
                    && let Some(filter) = mem::at::<u32>(body + BODY_FILTER)
                    && filter & 0x7F == LAYER_BIPED
                {
                    out.push(Bone { node, blend, body, code: (filter >> 8) & 0x1F });
                }
            }
            if depth > 2 && !name.starts_with("Bip01") {
                continue;
            }
            let data = mem::deref(node + NODE_CHILDREN);
            let n = mem::at::<u16>(node + NODE_COUNT).unwrap_or(0).min(64) as usize;
            if data == 0 {
                continue;
            }
            for i in 0..n {
                let child = mem::deref(data + i * 4);
                if child != 0 {
                    stack.push((child, depth + 1));
                }
            }
        }
        out
    }

    /// Whether `actor` is still actor `id` with its 3D (a save loaded or a cell let go of can leave
    /// another actor, or none, where it was).
    fn still(actor: usize, id: u32) -> bool {
        combat::is_actor(actor) && mem::at::<u32>(actor + FORM_ID) == Some(id) && mem::deref(actor + NI_NODE) != 0
    }

    fn motion(body: usize) -> usize {
        let m = mem::deref(body + BODY_MOTION);
        if m != 0 && mem::is_readable(m as *const u8, 0xF0) { m } else { 0 }
    }

    /// A body's mass (kg): a keyframed one's from its own motion, kept aside while it's keyframed
    /// (`hkKeyframedRigidMotion::savedMotion`).
    fn mass(body: usize) -> f32 {
        let m = motion(body);
        let inv = |m: usize| mem::at::<f32>(m + MOTION_MASS_INV).filter(|x| *x > 1e-5 && x.is_finite());
        let saved = || {
            let s = mem::deref(m + MOTION_SAVED);
            (s != 0 && mem::is_readable(s as *const u8, 0xD0)).then(|| inv(s)).flatten()
        };
        inv(m).or_else(saved).map_or(0.0, |x| 1.0 / x)
    }

    /// A body's transform in Havok's units: its rotation's columns and its translation.
    fn transform(body: usize) -> Option<(Mat3, Vec3)> {
        let m = motion(body);
        let c: [[f32; 4]; 3] = mem::at(m + MOTION_ROT)?;
        let t: [f32; 4] = mem::at(m + MOTION_POS)?;
        let r = Mat3::from_cols(Vec3::new(c[0][0], c[0][1], c[0][2]), Vec3::new(c[1][0], c[1][1], c[1][2]), Vec3::new(c[2][0], c[2][1], c[2][2]));
        let t = Vec3::new(t[0], t[1], t[2]);
        (r.is_finite() && t.is_finite()).then_some((r, t))
    }

    /// A point of the runtime's world in Havok's units, and back.
    fn to_hk(p: Vec3) -> Vec3 {
        camera::basis().point_to_game(p) / crate::collision::HAVOK_SCALE
    }

    fn from_hk(p: Vec3) -> Vec3 {
        camera::basis().point_from_game(p * crate::collision::HAVOK_SCALE)
    }

    /// Where a body's centre of mass is (the runtime's world).
    fn centre(body: usize) -> Option<Vec3> {
        let c: [f32; 4] = mem::at(motion(body) + MOTION_COM)?;
        let p = from_hk(Vec3::new(c[0], c[1], c[2]));
        p.is_finite().then_some(p)
    }

    /// A body's velocity (the runtime's world, m/s).
    fn velocity(body: usize) -> Vec3 {
        let v: [f32; 4] = mem::at(motion(body) + MOTION_LIN_VEL).unwrap_or_default();
        camera::basis().dir_from_game(Vec3::new(v[0], v[1], v[2])) * HK_METRES
    }

    /// Pushes a standing actor along the ground (the runtime's world, m/s), as the game's own
    /// knockback pushes one without a ragdoll.
    fn push(actor: usize, v: Vec3, time: f32) {
        type Get = unsafe extern "thiscall" fn(usize) -> usize;
        type Push = unsafe extern "thiscall" fn(usize, *const [f32; 3], f32);
        let controller = unsafe { std::mem::transmute::<usize, Get>(CONTROLLER)(actor) };
        if controller == 0 || !v.is_finite() {
            return;
        }
        let g = camera::basis().dir_to_game(v) * camera::UNITS_PER_METER * time;
        unsafe { std::mem::transmute::<usize, Push>(CONTROLLER_PUSH)(controller, &g.to_array(), time) };
    }

    /// Sets a body moving (the runtime's world, m/s) and wakes it.
    fn set_velocity(body: usize, v: Vec3) {
        let m = motion(body);
        if m == 0 || !v.is_finite() {
            return;
        }
        let g = camera::basis().dir_to_game(v) / HK_METRES;
        unsafe { ((m + MOTION_LIN_VEL) as *mut [f32; 4]).write_unaligned([g.x, g.y, g.z, 0.0]) };
        activate(body);
    }

    /// A body pushed at `point` (the runtime's world) along `dir`, `speed` m/s: the point gets what
    /// its speed lacks of the push, by an impulse there (Havok's own, as the game's grab pulls), so a
    /// push off its middle turns it too: a cheek pushed turns the head. Its mass and inertia say how
    /// much impulse that is. (A body Havok can't push, a keyframed one, gets nothing.)
    fn give(body: usize, point: Vec3, dir: Vec3, speed: f32) {
        let m = motion(body);
        let (Some(com), Some(inv_mass)) = (mem::at::<[f32; 4]>(m + MOTION_COM), mem::at::<f32>(m + MOTION_MASS_INV)) else { return };
        let (Some(inertia), Some(impulse_at)) = (virt(m, VT_INERTIA_INV_WORLD), virt(m, VT_POINT_IMPULSE)) else { return };
        let mut i = [V4::default(); 3];
        type Inertia = unsafe extern "thiscall" fn(usize, *mut V4);
        unsafe { std::mem::transmute::<usize, Inertia>(inertia)(m, i.as_mut_ptr()) };
        let i_inv = Mat3::from_cols(Vec3::from_slice(&i[0].0[..3]), Vec3::from_slice(&i[1].0[..3]), Vec3::from_slice(&i[2].0[..3]));
        // In Havok's world: the lever from its centre of mass, and how fast the point moves along
        // the push now.
        let at = to_hk(point);
        let r = at - Vec3::new(com[0], com[1], com[2]);
        let n = camera::basis().dir_to_game(dir).normalize_or_zero();
        let v: [f32; 4] = mem::at(m + MOTION_LIN_VEL).unwrap_or_default();
        let w: [f32; 4] = mem::at(m + MOTION_ANG_VEL).unwrap_or_default();
        let moving = (Vec3::new(v[0], v[1], v[2]) + Vec3::new(w[0], w[1], w[2]).cross(r)).dot(n);
        let lack = speed / HK_METRES - moving;
        // (How much the point's speed along the push changes for each unit of impulse there.)
        let rn = r.cross(n);
        let k = inv_mass + rn.dot(i_inv * rn);
        if !(lack > 0.0) || !(k > 1e-6) || !k.is_finite() || !at.is_finite() || n == Vec3::ZERO {
            return;
        }
        let j = n * (lack / k);
        activate(body);
        let (impulse, at) = (V4([j.x, j.y, j.z, 0.0]), V4([at.x, at.y, at.z, 0.0]));
        type Impulse = unsafe extern "thiscall" fn(usize, *const V4, *const V4);
        unsafe { std::mem::transmute::<usize, Impulse>(impulse_at)(m, &impulse, &at) };
    }

    fn activate(body: usize) {
        type Activate = unsafe extern "thiscall" fn(usize) -> u32;
        unsafe { std::mem::transmute::<usize, Activate>(ACTIVATE)(body) };
    }

    fn virt(obj: usize, slot: usize) -> Option<usize> {
        let f = mem::deref(mem::deref(obj) + slot);
        (obj != 0 && CODE.contains(&f)).then_some(f)
    }

    fn is_dead(actor: usize) -> bool {
        type F = unsafe extern "thiscall" fn(usize, u8) -> u8;
        virt(actor, VT_IS_DEAD).is_some_and(|f| unsafe { std::mem::transmute::<usize, F>(f)(actor, 0) } != 0)
    }

    /// The actor's high process, if it has one (only those are knocked down).
    fn high_process(actor: usize) -> Option<usize> {
        let p = mem::deref(actor + PROCESS);
        type F = unsafe extern "thiscall" fn(usize) -> u32;
        let f = virt(p, VT_NOT_HIGH)?;
        (unsafe { std::mem::transmute::<usize, F>(f)(p) } == 0).then_some(p)
    }

    fn knock_state(actor: usize) -> u8 {
        high_process(actor).and_then(|p| mem::at::<u8>(p + KNOCK_STATE)).unwrap_or(0)
    }

    fn game_hour() -> Option<f32> {
        mem::at::<f32>(mem::deref(GAME_TIME + 0xC) + 0x24)
    }

    /// The heads the hands have (their `Bip01 Head` bones), and how many. The head tracking turns
    /// them to look where they looked (its measure held, not taken again from where the push has
    /// them), and the push on top of that: how far the physics has turned the head from where its
    /// animation has it. Let be, the head tracking measures them again.
    static HANDS_HEADS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
    static HANDS_HEAD_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    /// Each held head's turn as its animation has it this frame (taken as its blend starts).
    static ANIMATED: Mutex<Vec<(usize, [f32; 9])>> = Mutex::new(Vec::new());
    static REAL_MEASURE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static REAL_LOOK: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static REAL_BLEND: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn held_head(node: usize) -> bool {
        HANDS_HEAD_COUNT.load(std::sync::atomic::Ordering::Relaxed) != 0 && HANDS_HEADS.lock().is_ok_and(|h| h.contains(&node))
    }

    /// A bone's blend update: a held head's animated turn kept first (the blend's about to put the
    /// physics in it).
    unsafe extern "thiscall" fn blend(this: usize) {
        let real: unsafe extern "thiscall" fn(usize) = unsafe { std::mem::transmute(REAL_BLEND.load(std::sync::atomic::Ordering::Acquire)) };
        let node = mem::deref(this + 8);
        if held_head(node)
            && let Some(turn) = mem::at::<[f32; 9]>(node + av::WORLD_ROTATE)
            && let Ok(mut animated) = ANIMATED.lock()
        {
            animated.retain(|x| x.0 != node);
            animated.push((node, turn));
        }
        unsafe { real(this) }
    }

    /// The head tracking's look: for a held head, its own look with the push on top (the physics'
    /// turn from the animated one, in the head's frame), so the head looks where the head tracking
    /// has it and gives under the hand.
    unsafe extern "thiscall" fn look(face: usize, out: *mut [f32; 9], time: u32, head: usize) -> *mut [f32; 9] {
        let real: unsafe extern "thiscall" fn(usize, *mut [f32; 9], u32, usize) -> *mut [f32; 9] = unsafe { std::mem::transmute(REAL_LOOK.load(std::sync::atomic::Ordering::Acquire)) };
        let r = unsafe { real(face, out, time, head) };
        // (Its head tracking off, the game's own, the look is none and the push already shows.)
        if r.is_null() || !held_head(head) || mem::at::<u8>(face + 474) != Some(0) {
            return r;
        }
        let animated = ANIMATED.lock().ok().and_then(|mut a| a.iter().position(|x| x.0 == head).map(|i| a.swap_remove(i).1));
        crate::hooks::guard_void("head tracking look", || {
            let (Some(animated), Some(now)) = (animated, mem::at::<[f32; 9]>(head + av::WORLD_ROTATE)) else { return };
            let push = crate::items::ni_matrix(animated).transpose() * crate::items::ni_matrix(now);
            let pushed = crate::items::ni_matrix(unsafe { r.read_unaligned() }) * push;
            if pushed.is_finite() {
                unsafe { r.write_unaligned(crate::items::ni_rows(pushed)) };
            }
        });
        r
    }

    /// The head tracking measuring a head: not a head the hands have (its look held as it was).
    unsafe extern "thiscall" fn measure(face: usize, time: u32, head: usize, non_accum: usize) -> u8 {
        let real: unsafe extern "thiscall" fn(usize, u32, usize, usize) -> u8 = unsafe { std::mem::transmute(REAL_MEASURE.load(std::sync::atomic::Ordering::Acquire)) };
        if HANDS_HEADS.lock().is_ok_and(|h| h.contains(&head)) {
            static SAID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            if SAID.swap(head, std::sync::atomic::Ordering::Relaxed) != head {
                tracing::info!("grabs: the head tracking holds its look on the head {head:#x}");
            }
            // (Its result is the caller's to ignore: the measure says whether it measured.)
            return 1;
        }
        unsafe { real(face, time, head, non_accum) }
    }

    /// The game's knockback with no push: the actor goes down where it is, every bone Havok's.
    fn knock_down(actor: usize) -> bool {
        let Some(p) = high_process(actor) else { return false };
        let (Some(f), Some(at)) = (virt(p, VT_KNOCKBACK), mem::at::<[f32; 3]>(actor + refr::POS)) else { return false };
        if knock_state(actor) != 0 || is_dead(actor) {
            return false;
        }
        type F = unsafe extern "thiscall" fn(usize, usize, f32, f32, f32, f32);
        unsafe { std::mem::transmute::<usize, F>(f)(p, actor, at[0], at[1], at[2], 0.0) };
        knock_state(actor) != 0
    }

    /// The player attacks `actor`, as far as the actor is concerned (the end of every blow).
    fn attacked(actor: usize) {
        type F = unsafe extern "thiscall" fn(usize, usize, u32);
        let player = engine::player();
        if let Some(f) = virt(actor, VT_ON_ATTACKED)
            && player != 0
        {
            unsafe { std::mem::transmute::<usize, F>(f)(actor, player, 0) };
        }
    }

    /// Whether an actor is fighting (vtable +0x334), and the player's bounty (the crime gold the
    /// guards want, vtable +0x250): the game's reaction, for the log.
    fn in_combat(actor: usize) -> bool {
        type F = unsafe extern "thiscall" fn(usize, u8) -> u8;
        virt(actor, 0x334).is_some_and(|f| unsafe { std::mem::transmute::<usize, F>(f)(actor, 1) } != 0)
    }

    fn crime_gold() -> f32 {
        type F = unsafe extern "thiscall" fn(usize) -> f32;
        let p = engine::player();
        virt(p, 0x250).map_or(0.0, |f| unsafe { std::mem::transmute::<usize, F>(f)(p) })
    }

    fn damage(actor: usize, av: u32, amount: f32) {
        type F = unsafe extern "thiscall" fn(usize, u32, f32, usize);
        if let Some(f) = virt(actor, VT_DAMAGE_AV) {
            unsafe { std::mem::transmute::<usize, F>(f)(actor, av, -amount, engine::player()) };
        }
    }

    fn bodies_in_world(root: usize, on: bool) {
        type F = unsafe extern "C" fn(usize, u8, u8, u8) -> u8;
        unsafe { std::mem::transmute::<usize, F>(BODIES_IN_WORLD)(root, on as u8, 1, 1) };
    }

    fn set_gains(blend: usize, heir: f32, vel: f32) {
        unsafe {
            ((blend + BLEND_HEIR) as *mut f32).write(heir);
            ((blend + BLEND_VEL) as *mut f32).write(vel);
        }
    }

    fn gains(blend: usize) -> (f32, f32) {
        (mem::at::<f32>(blend + BLEND_HEIR).unwrap_or(1.0), mem::at::<f32>(blend + BLEND_VEL).unwrap_or(1.0))
    }

    /// The `bhkWorld` the actor is in.
    fn world_of(actor: usize) -> usize {
        type Cell = unsafe extern "thiscall" fn(usize) -> usize;
        let cell = unsafe { std::mem::transmute::<usize, Cell>(PARENT_CELL)(actor) };
        if cell == 0 {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, Cell>(CELL_WORLD)(cell) }
    }

    /// A mouse spring on `body` at `point` (Havok units, the world) pulling it to the same point
    /// (no pull yet), in `world`: the game's own grab.
    fn spring(body: usize, point: Vec3, world: usize) -> usize {
        let Some((r, t)) = transform(body) else { return 0 };
        let local = r.transpose() * (point - t);
        let mut info = SpringInfo([0; 64]);
        type Info = unsafe extern "thiscall" fn(*mut SpringInfo);
        type New = unsafe extern "thiscall" fn(usize, *const SpringInfo) -> usize;
        type Alloc = unsafe extern "C" fn(usize) -> usize;
        type Add = unsafe extern "thiscall" fn(usize, usize) -> u8;
        unsafe {
            std::mem::transmute::<usize, Info>(SPRING_INFO)(&mut info);
            let base = info.0.as_mut_ptr();
            (base.add(4) as *mut usize).write(body);
            (base.add(0x10) as *mut [f32; 4]).write([local.x, local.y, local.z, 0.0]);
            (base.add(0x20) as *mut [f32; 4]).write([point.x, point.y, point.z, 0.0]);
            (base.add(0x30) as *mut [f32; 4]).write([SPRING_DAMPING, SPRING_ELASTICITY, 0.0, SPRING_BODY_DAMPING]);
            let m = std::mem::transmute::<usize, Alloc>(ALLOC)(16);
            if m == 0 {
                return 0;
            }
            let action = std::mem::transmute::<usize, New>(SPRING_NEW)(m, &info);
            if action == 0 {
                return 0;
            }
            // Ours: one reference held until it's let go.
            let rc = (action + 4) as *const std::sync::atomic::AtomicU32;
            (*rc).fetch_add(1, std::sync::atomic::Ordering::AcqRel);
            if let Some(f) = virt(action, VT_ADD_TO_WORLD) {
                std::mem::transmute::<usize, Add>(f)(action, world);
            }
            action
        }
    }

    /// Moves a spring's target (Havok units) and sets the most it pulls with (relative to its body's
    /// mass).
    fn pull(action: usize, to: Vec3, max_rel: f32) {
        let hk = mem::deref(action + HK_OBJ);
        if hk == 0 || !to.is_finite() {
            return;
        }
        let v = V4([to.x, to.y, to.z, 0.0]);
        type Set = unsafe extern "thiscall" fn(usize, *const V4) -> u32;
        unsafe {
            ((hk + SPRING_MAX_FORCE) as *mut f32).write(max_rel);
            std::mem::transmute::<usize, Set>(SET_MOUSE)(hk, &v);
        }
    }

    /// Takes a spring out of the world and lets it go (the body woken, as the game's own let-go).
    fn unspring(action: usize, body: usize) {
        type Remove = unsafe extern "thiscall" fn(usize) -> u32;
        type Delete = unsafe extern "thiscall" fn(usize, u32);
        unsafe {
            if let Some(f) = virt(action, VT_REMOVE) {
                std::mem::transmute::<usize, Remove>(f)(action);
            }
            if body != 0 {
                activate(body);
            }
            let rc = (action + 4) as *const std::sync::atomic::AtomicU32;
            if (*rc).fetch_sub(1, std::sync::atomic::Ordering::AcqRel) == 1
                && let Some(f) = virt(action, 0)
            {
                std::mem::transmute::<usize, Delete>(f)(action, 1);
            }
        }
    }

    /// A hand's hold on an actor.
    struct Hold {
        serial: u32,
        actor_id: u32,
        actor: usize,
        bone: Bone,
        action: usize,
        /// The point held, in the body's own frame (Havok units).
        local: Vec3,
        since: f64,
        /// Lifted (pulled up past the standing body) since.
        lifting: Option<f64>,
        choking: bool,
        attacked: bool,
        /// When it next struggles (held up by the throat).
        next_kick: f64,
    }

    /// A living actor whose bodies the hands put in Havok's world, and the bones they made
    /// physical (their gains before).
    struct Limp {
        actor: usize,
        root: usize,
        bones: Vec<(usize, f32, f32)>,
        since: f64,
        /// When a hand (or what it holds) last pressed into it, and the bone (its part code) it last
        /// said was pressed.
        touched: Option<f64>,
        touched_bone: u32,
        /// Its head bone, while it's the hands' (the head tracking holds its look on it as it was, the
        /// push on top).
        head: usize,
        /// Let go by every hand and untouched a while: blending back since, from these gains; back
        /// to its gains (its bodies out of the world the frame after).
        back: Option<f64>,
        leaving: Vec<(f32, f32)>,
        restored: bool,
    }

    struct State {
        holds: [Option<Hold>; 2],
        /// The last grab each hand let go of (so its last word isn't taken as a new grab).
        done: [u32; 2],
        limp: HashMap<u32, Limp>,
        /// The actors published, by id, and each part's bone (the one it starts at) and where its
        /// capsule starts (the frame a hold on it is in: its start, turned as its bone is).
        actors: HashMap<u32, usize>,
        part_nodes: HashMap<(u32, usize), (usize, Vec3)>,
        bodies: Box<ActorBodies>,
        grabs: Box<CreatureGrabs>,
        pushes: Box<GamePushes>,
        last: f64,
        /// The game's masses per kilogram (see [`PERSON_KG`]), once known.
        scale: Option<f32>,
        /// For the struggles' variety.
        seed: u32,
        /// Each actor's knock state as last seen (logged as it changes: down, getting up).
        knocks: HashMap<u32, u8>,
        /// Each actor's skeleton, found once for its 3D (walking it reads thousands of values).
        skeletons: HashMap<u32, Skeleton>,
    }

    /// An actor's bones with bodies, and each of its parts' bone (the one it starts at), for its 3D
    /// `root` and that many parts.
    struct Skeleton {
        root: usize,
        parts: u32,
        bones: Vec<Bone>,
        part_nodes: Vec<Option<usize>>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State {
            holds: [None, None],
            done: [0; 2],
            limp: HashMap::new(),
            actors: HashMap::new(),
            part_nodes: HashMap::new(),
            bodies: bytemuck::zeroed_box(),
            grabs: bytemuck::zeroed_box(),
            pushes: bytemuck::zeroed_box(),
            last: 0.0,
            scale: None,
            seed: 0x9E37_79B9,
            knocks: HashMap::new(),
            skeletons: HashMap::new(),
        }))
    }

    /// The game's masses per kilogram: the player's own ragdoll is a person's.
    fn mass_scale(s: &mut State) -> f32 {
        if let Some(k) = s.scale {
            return k;
        }
        let player = engine::player();
        let root = if player != 0 { mem::deref(player + NI_NODE) } else { 0 };
        let total: f32 = if root != 0 { skeleton(root).iter().map(|b| mass(b.body)).sum() } else { 0.0 };
        if total <= 0.0 {
            return 1.0;
        }
        let k = (total / PERSON_KG).clamp(1.0, 20.0);
        tracing::info!("grabs: the player's ragdoll weighs {total:.0}: the game's masses are {k:.2} to the kilogram");
        s.scale = Some(k);
        k
    }

    /// Every frame, with the creatures (`table`, in its order; `by_id` the actors): what grabbing
    /// them needs.
    pub fn publish(a: &rustvr_adapter::Adapter, table: &Actors, by_id: &HashMap<u32, usize>) {
        let player = engine::player();
        with_state(|s| {
            let k = mass_scale(s);
            s.actors = by_id.clone();
            s.part_nodes.clear();
            s.skeletons.retain(|id, _| by_id.contains_key(id));
            let out = &mut s.bodies;
            out.count = 0;
            out.frame += 1;
            out.strength = if player != 0 { (combat::actor_value(player, AV_STRENGTH) as f32 / 50.0).clamp(0.2, 3.0) } else { 1.0 };
            let b = camera::basis();
            // (The horse the player rides isn't the hands' to take: its neck is where the reins are.)
            let mount = if player != 0 { mem::deref(player + MOUNT) } else { 0 };
            for actor in &table.actors[..(table.count as usize).min(MAX_ACTORS)] {
                let mut body: ActorBody = bytemuck::Zeroable::zeroed();
                body.id = actor.id;
                if let Some(&ptr) = by_id.get(&actor.id).filter(|&&p| combat::is_actor(p)) {
                    let root = mem::deref(ptr + NI_NODE);
                    let fresh = s.skeletons.get(&actor.id).is_some_and(|k| k.root == root && k.parts == actor.count);
                    if !fresh {
                        // Each part's bone: the one it starts at.
                        let named = if root != 0 { combat::bones(root) } else { HashMap::new() };
                        let part_nodes = actor.parts[..(actor.count as usize).min(MAX_ACTOR_PARTS)]
                            .iter()
                            .map(|part| combat::part_bone(part.game_part).and_then(|name| named.get(name).copied()))
                            .collect();
                        let bones = if root != 0 { skeleton(root) } else { Vec::new() };
                        s.skeletons.insert(actor.id, Skeleton { root, parts: actor.count, bones, part_nodes });
                    }
                    let sk = &s.skeletons[&actor.id];
                    let bones = &sk.bones;
                    body.mass = bones.iter().map(|x| mass(x.body)).sum::<f32>() / k;
                    let dead = is_dead(ptr);
                    let knocked = knock_state(ptr);
                    if s.knocks.insert(actor.id, knocked).is_some_and(|k| k != knocked) {
                        tracing::info!("grabs: {:#x}'s knock state {knocked}{}", actor.id, if dead { " (dead)" } else { "" });
                    }
                    body.flags = if bones.is_empty() || ptr == mount { 0 } else { BODY_GRABBABLE }
                        | if dead || !matches!(knocked, 0 | 6) { BODY_RAGDOLL } else { 0 }
                        | if !dead && !matches!(knocked, 3 | 4) && combat::actor_value(ptr, AV_FATIGUE) >= 0 { BODY_CONSCIOUS } else { 0 };
                    // Each part turned as its bone is; its body's mass and speed.
                    for i in 0..(actor.count as usize).min(MAX_ACTOR_PARTS) {
                        let Some(node) = sk.part_nodes.get(i).copied().flatten() else { continue };
                        s.part_nodes.insert((actor.id, i), (node, Vec3::from_array(actor.parts[i].a)));
                        let Some(rot) = mem::at::<[f32; 9]>(node + av::WORLD_ROTATE) else { continue };
                        let q = b.rot_from_game(crate::items::ni_matrix(rot)).normalize();
                        let own = bones.iter().find(|x| x.node == node);
                        body.parts[i] = PartBody {
                            rot: if q.is_finite() { q.to_array() } else { [0.0; 4] },
                            vel: own.map_or(Vec3::from_array(actor.velocity), |x| velocity(x.body)).to_array(),
                            mass: own.map_or(0.0, |x| mass(x.body) / k),
                        };
                    }
                }
                let i = out.count as usize;
                out.bodies[i] = body;
                out.count += 1;
            }
            if out.frame % 120 == 5 && out.count > 0 {
                let list: Vec<String> = out.bodies[..out.count as usize]
                    .iter()
                    .map(|b| format!("{:#x}: {:.0} kg, flags {:#x}, {} parts turned", b.id, b.mass, b.flags, b.parts.iter().filter(|p| p.rot != [0.0; 4]).count()))
                    .collect();
                tracing::info!("grabs: bodies {}; strength {:.2}", list.join("; "), out.strength);
            }
            a.write_actor_bodies(out);
        });
    }

    /// Before the game's frame: each hand's grab, done by the game.
    pub fn apply(now: f64) {
        if engine::player() == 0 {
            return;
        }
        static HOOKED: std::sync::Once = std::sync::Once::new();
        HOOKED.call_once(|| {
            crate::hooks::install_into(HEAD_TRACK_MEASURE as *const (), measure as *const (), "head tracking measure", &REAL_MEASURE);
            crate::hooks::install_into(HEAD_TRACK_LOOK as *const (), look as *const (), "head tracking look", &REAL_LOOK);
            crate::hooks::install_into(BLEND_UPDATE as *const (), blend as *const (), "bone blend", &REAL_BLEND);
        });
        with_state(|s| {
            let dt = ((now - s.last) as f32).clamp(0.0, 0.1);
            s.last = now;
            let mut grabs = std::mem::replace(&mut s.grabs, bytemuck::zeroed_box());
            let read = crate::link::with_adapter(|a| a.read_creature_grabs(&mut grabs)).unwrap_or(false);
            if !read {
                // No runtime: nothing holds anyone.
                grabs.grabs = [CreatureGrab::default(); 2];
            }
            for hand in 0..2 {
                let g = grabs.grabs[hand];
                let ours = s.holds[hand].as_ref().map(|h| h.serial);
                let held = g.flags & GRAB_HELD != 0;
                match ours {
                    Some(serial) if serial == g.serial && held => s.hold(hand, &g, now, dt),
                    Some(serial) => {
                        // Let go (its last word, with the throw), or a new grab in its place.
                        let throw = (serial == g.serial && g.flags & GRAB_SLIPPED == 0).then(|| Vec3::from_array(g.vel));
                        s.let_go(hand, throw, g.force, now);
                        if held && g.serial != serial {
                            s.take(hand, &g, now);
                        }
                    }
                    None if held && g.serial != 0 && g.serial != s.done[hand] => s.take(hand, &g, now),
                    None => {}
                }
            }
            s.grabs = grabs;
            s.blend_back(now);
            if let Ok(mut heads) = HANDS_HEADS.lock() {
                let now_held: Vec<usize> = s.limp.values().map(|l| l.head).filter(|&h| h != 0).collect();
                if *heads != now_held {
                    tracing::info!("grabs: the heads the hands have: {now_held:x?} (the head tracking holds its look on them, the push on top)");
                    HANDS_HEAD_COUNT.store(now_held.len(), std::sync::atomic::Ordering::Relaxed);
                    *heads = now_held;
                    if let Ok(mut animated) = ANIMATED.lock() {
                        animated.retain(|x| heads.contains(&x.0));
                    }
                }
            }
            let mut pushes = std::mem::replace(&mut s.pushes, bytemuck::zeroed_box());
            if crate::link::with_adapter(|a| a.read_game_pushes(&mut pushes)).unwrap_or(false) {
                let n = (pushes.count as usize).min(MAX_GAME_PUSHES);
                for p in pushes.pushes[..n].iter().filter(|p| p.part != 0) {
                    s.pushed(p, now);
                }
            }
            s.pushes = pushes;
        });
    }

    impl State {
        /// A number from 0 to 1 (xorshift).
        fn random(&mut self) -> f32 {
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 17;
            self.seed ^= self.seed << 5;
            (self.seed >> 8) as f32 / (1u32 << 24) as f32
        }

        /// Hand `hand` takes hold of an actor (`g`: its first word). A standing actor's bodies go into
        /// Havok's world first, and the hold is taken the frame after, with them in their bones'
        /// places.
        fn take(&mut self, hand: usize, g: &CreatureGrab, now: f64) {
            let Some(&actor) = self.actors.get(&g.actor).filter(|&&a| combat::is_actor(a)) else {
                self.done[hand] = g.serial;
                return;
            };
            let root = mem::deref(actor + NI_NODE);
            if root == 0 {
                self.done[hand] = g.serial;
                return;
            }
            let dead = is_dead(actor);
            let standing = !dead && knock_state(actor) == 0;
            if standing && !self.limp.contains_key(&g.actor) {
                bodies_in_world(root, true);
                self.limp.insert(g.actor, Limp { actor, root, bones: Vec::new(), since: now, touched: None, touched_bone: 0, head: 0, back: None, leaving: Vec::new(), restored: false });
                return;
            }
            self.done[hand] = g.serial;
            let bones = skeleton(root);
            // Where the hand has it: the point in its part's frame (its capsule's start, turned as
            // its bone is), where the part is now; else where the hand touched it.
            let b = camera::basis();
            let frame = self.part_nodes.get(&(g.actor, g.part as usize)).and_then(|&(n, start)| {
                let rot: [f32; 9] = mem::at(n + av::WORLD_ROTATE)?;
                Some((start, b.rot_from_game(crate::items::ni_matrix(rot)).normalize()))
            });
            let at = frame.map_or(Vec3::from_array(g.target), |(p, q)| p + q * Vec3::from_array(g.point));
            // The bone held: on the throat, the head's; else the one whose body's middle is nearest.
            let throat = (g.flags & GRAB_THROAT != 0).then(|| bones.iter().find(|b| b.code == HEAD)).flatten();
            let nearest = || bones.iter().filter_map(|b| Some((b, centre(b.body)?.distance(at)))).min_by(|x, y| x.1.total_cmp(&y.1)).map(|x| x.0);
            let Some(&bone) = throat.or_else(nearest) else {
                tracing::info!("grabs: {:#x} has no bodies to take hold of", g.actor);
                return;
            };
            if standing && let Some(l) = self.limp.get_mut(&g.actor) {
                l.back = None;
                l.leaving.clear();
                l.restored = false;
                let (full, soft) = limb(bone.code);
                for b in &bones {
                    let heir = if full.contains(&b.code) {
                        0.0
                    } else if soft.contains(&b.code) {
                        SPINE_HEIR
                    } else {
                        continue;
                    };
                    if !l.bones.iter().any(|x| x.0 == b.blend) {
                        let (h0, v0) = gains(b.blend);
                        l.bones.push((b.blend, h0, v0));
                    }
                    set_gains(b.blend, heir, if heir == 0.0 { 0.0 } else { 1.0 });
                }
                if full.contains(&HEAD) {
                    l.head = bones.iter().find(|b| b.code == HEAD).map_or(0, |b| b.node);
                }
            }
            let world = world_of(actor);
            let point = to_hk(at);
            let in_world = mem::deref(bone.body + BODY_WORLD) != 0;
            let action = if world != 0 && in_world { spring(bone.body, point, world) } else { 0 };
            let local = transform(bone.body).map_or(Vec3::ZERO, |(r, t)| r.transpose() * (point - t));
            tracing::info!(
                "grabs: the {} hand takes {:#x} by its bone {} at {at:.2?} ({:.1} kg of {:.0}){}{}: spring {action:#x}{}",
                if hand == 0 { "left" } else { "right" },
                g.actor,
                bone.code,
                mass(bone.body),
                bones.iter().map(|b| mass(b.body)).sum::<f32>(),
                if dead { ", dead" } else if standing { ", standing" } else { ", down" },
                if g.flags & GRAB_THROAT != 0 { ", by the throat" } else { "" },
                if in_world { "" } else { " (its body isn't in Havok's world)" }
            );
            if action == 0 {
                return;
            }
            self.holds[hand] = Some(Hold { serial: g.serial, actor_id: g.actor, actor, bone, action, local, since: now, lifting: None, choking: false, attacked: false, next_kick: now + KICK_EVERY });
        }

        /// A hand (or what it holds) pressed into a part of an actor: a body that's all physics gets
        /// what its bone lacks of the push there; a standing one gives under it and springs back,
        /// and is shoved by a hard push.
        fn pushed(&mut self, p: &GamePush, now: f64) {
            let Some(&actor) = self.actors.get(&p.id).filter(|&&a| combat::is_actor(a)) else { return };
            let (point, dir, speed) = (Vec3::new(p.point[0], p.point[1], p.point[2]), Vec3::new(p.normal[0], p.normal[1], p.normal[2]), p.normal[3]);
            if !dir.is_finite() || dir.length_squared() < 0.5 || !(speed > 0.0) {
                return;
            }
            // (The bone of these whose body is nearest the push.)
            let nearest = |bones: &mut dyn Iterator<Item = &Bone>| bones.filter_map(|b| Some((*b, centre(b.body)?.distance(point)))).min_by(|a, b| a.1.total_cmp(&b.1)).map(|x| x.0);
            let root = mem::deref(actor + NI_NODE);
            let Some(sk) = self.skeletons.get(&p.id).filter(|k| k.root == root && root != 0) else { return };
            let dead = is_dead(actor);
            if dead || knock_state(actor) != 0 {
                if let Some(b) = nearest(&mut sk.bones.iter())
                    && mem::deref(b.body + BODY_WORLD) != 0
                {
                    give(b.body, point, dir, speed);
                }
                return;
            }
            // Standing, it gives under the touch: the bones about it go partly physical (its bodies
            // put in Havok's world first, and touched the frame after, with them in their bones'
            // places), and the soft body nearest the push gets it. (Held, the hand's grab moves it.)
            let held = self.holds.iter().flatten().any(|h| h.actor_id == p.id);
            let bone = nearest(&mut sk.bones.iter()).map_or(0, |b| b.code);
            let touched = touch(bone);
            if !held && !touched.is_empty() {
                match self.limp.get_mut(&p.id) {
                    None => {
                        bodies_in_world(root, true);
                        self.limp.insert(p.id, Limp { actor, root, bones: Vec::new(), since: now, touched: Some(now), touched_bone: 0, head: 0, back: None, leaving: Vec::new(), restored: false });
                    }
                    Some(l) => {
                        // (Touched again as it blends back, it stays as the hands left it, and
                        // blends back from there once it's let be.)
                        l.back = None;
                        l.leaving.clear();
                        l.restored = false;
                        if l.touched_bone != bone && (l.touched_bone == 0 || l.touched.is_none_or(|t| now - t > 0.5)) {
                            l.touched_bone = bone;
                            tracing::info!("grabs: {:#x} touched on its bone {bone} at {speed:.2} m/s", p.id);
                        }
                        l.touched = Some(now);
                        for &(code, (heir, vel)) in touched {
                            for b in sk.bones.iter().filter(|b| b.code == code) {
                                if !l.bones.iter().any(|x| x.0 == b.blend) {
                                    let (h0, v0) = gains(b.blend);
                                    l.bones.push((b.blend, h0, v0));
                                }
                                set_gains(b.blend, heir, vel);
                            }
                        }
                        // (Its head the hand's, the game's head tracking would turn it back.)
                        if touched.iter().any(|t| t.0 == HEAD) {
                            l.head = sk.bones.iter().find(|b| b.code == HEAD).map_or(0, |b| b.node);
                        }
                        let soft = sk.bones.iter().filter(|b| l.bones.iter().any(|x| x.0 == b.blend) && mem::deref(b.body + BODY_WORLD) != 0);
                        if let Some(b) = nearest(&mut soft.into_iter()) {
                            give(b.body, point, dir, speed);
                        }
                    }
                }
            }
            if p.moving >= SHOVE {
                let along = Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero();
                push(actor, along * p.moving.min(SHOVE_MAX), SHOVE_TIME);
                static SAID: Mutex<f64> = Mutex::new(0.0);
                let mut said = SAID.lock().unwrap();
                if now - *said > 0.5 {
                    *said = now;
                    tracing::info!("grabs: {:#x} shoved at {:.1} m/s", p.id, p.moving);
                }
            }
        }

        /// A hand holding on: its spring pulled to where the hand would have the point, the actor
        /// knocked down when it's lifted or yanked, kept down while it's held, choked.
        fn hold(&mut self, hand: usize, g: &CreatureGrab, now: f64, dt: f32) {
            let Some(h) = self.holds[hand].as_mut() else { return };
            if !still(h.actor, h.actor_id) {
                tracing::info!("grabs: {:#x} went from the hand", h.actor_id);
                let h = self.holds[hand].take().unwrap();
                self.done[hand] = h.serial;
                // (Its body may be gone with it: the spring's let go without waking it.)
                unspring(h.action, 0);
                self.limp.remove(&h.actor_id);
                return;
            }
            let m = mass(h.bone.body);
            let target = to_hk(Vec3::from_array(g.target));
            // The most the hand pulls with, as the force relative to the body's mass the spring
            // takes (Havok units a second squared). (A body still keyframed, a standing actor's
            // pelvis, isn't moved by it: Havok gives keyframed bodies no mass to push.)
            let max_rel = if m > 0.0 { g.force * self.scale.unwrap_or(1.0) / m / HK_METRES } else { 0.0 };
            pull(h.action, target, max_rel);
            let dead = is_dead(h.actor);
            let state = knock_state(h.actor);
            let held = transform(h.bone.body).map(|(r, t)| from_hk(t + r * h.local));
            if (now / 1.0).floor() != ((now - dt as f64) / 1.0).floor() {
                let hk = mem::deref(h.action + HK_OBJ);
                let kind = {
                    type F = unsafe extern "thiscall" fn(usize) -> u32;
                    let m = motion(h.bone.body);
                    virt(m, 8).map_or(99, |f| unsafe { std::mem::transmute::<usize, F>(f)(m) })
                };
                tracing::info!(
                    "grabs: holding {:#x}: its point at {held:.2?} for {:.2?} ({:.2} m off), moving {:.2} m/s; body type {kind}, {:.1} kg, in world {}; spring entity {:#x} world {:#x} target {:?} force {:?}; knock state {state}; flags {:#x}, squeeze {:.2}, fatigue {}; in combat {}, the player's bounty {}; the actor at {:.2?}",
                    h.actor_id,
                    Vec3::from_array(g.target),
                    held.map_or(0.0, |p| p.distance(Vec3::from_array(g.target))),
                    velocity(h.bone.body).length(),
                    m,
                    mem::deref(h.bone.body + BODY_WORLD) != 0,
                    mem::deref(hk + 0x18),
                    mem::deref(hk + 0x8),
                    mem::at::<[f32; 3]>(hk + 0x30),
                    mem::at::<f32>(hk + SPRING_MAX_FORCE),
                    g.flags,
                    g.squeeze,
                    combat::actor_value(h.actor, AV_FATIGUE),
                    in_combat(h.actor),
                    crime_gold(),
                    mem::at::<[f32; 3]>(h.actor + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p))),
                );
            }
            let to = Vec3::from_array(g.target);
            // Standing: lifted by the head, neck or torso, or yanked away, it goes down.
            if !dead && state == 0 && let Some(p) = held {
                let up = to.y - p.y;
                h.lifting = (lifts_by(h.bone.code) && up > LIFT).then(|| h.lifting.unwrap_or(now));
                let lifted = h.lifting.is_some_and(|t| now - t > LIFT_TIME);
                let leg = matches!(h.bone.code, 8..=10 | 14..=16);
                if lifted || to.distance(p) > if leg { TRIP } else { YANK } {
                    let actor = h.actor;
                    if knock_down(actor) {
                        tracing::info!("grabs: {:#x} {} by the hand: knocked down", h.actor_id, if lifted { "lifted off its feet" } else if leg { "tripped" } else { "yanked over" });
                        self.limp.entry(h.actor_id).and_modify(|l| l.bones.clear());
                        // (One assault a hold, whatever it comes to.)
                        if !h.attacked {
                            attacked(actor);
                            h.attacked = true;
                        }
                    }
                }
            }
            // Standing and pulled along the ground: it's pushed after the pull, stumbling along
            // rather than standing there stretched.
            if !dead && knock_state(h.actor) == 0 && let Some(p) = held {
                let pull = Vec3::new(to.x - p.x, 0.0, to.z - p.z);
                let over = pull.length() - STEP_START;
                if over > 0.0 {
                    push(h.actor, pull.normalize_or_zero() * (over * STEP_GAIN).min(STEP_MAX), STEP_TIME);
                }
            }
            // Down and held: it can't get up while a hand has it.
            if !dead && state == 1
                && let (Some(p), Some(hour)) = (high_process(h.actor), game_hour())
            {
                unsafe { ((p + KNOCK_TIME) as *mut f32).write(hour) };
            }
            // On the throat: its fatigue goes, as hard as it's squeezed.
            if !dead && g.flags & GRAB_THROAT != 0 && g.squeeze > CHOKE_SQUEEZE {
                let most = combat::base_actor_value(h.actor, AV_FATIGUE).max(1) as f32;
                let rate = most / CHOKE_TIME * g.squeeze * if state != 0 { CHOKE_LIFTED } else { 1.0 };
                let left = combat::actor_value(h.actor, AV_FATIGUE) as f32 + most * CHOKED_OUT;
                damage(h.actor, AV_FATIGUE, (rate * dt).min(left.max(0.0)));
                if !h.choking {
                    h.choking = true;
                    tracing::info!("grabs: {:#x} choked (fatigue {} of {most})", h.actor_id, combat::actor_value(h.actor, AV_FATIGUE));
                    if !h.attacked {
                        attacked(h.actor);
                        h.attacked = true;
                    }
                }
            } else {
                h.choking = false;
            }
            // Choked (or held up by the head) and conscious: it fights for air, its hands clutching at
            // the hand at its throat, and off its feet its legs kicking, weaker as its fatigue goes.
            let throat = g.flags & GRAB_THROAT != 0 || h.bone.code == HEAD;
            let fatigue = combat::actor_value(h.actor, AV_FATIGUE) as f32;
            if !dead && throat && fatigue > 0.0 && (state != 0 || h.choking) && let Some(p) = held {
                let most = combat::base_actor_value(h.actor, AV_FATIGUE).max(1) as f32;
                let k = (fatigue / most).clamp(0.3, 1.0);
                let (id, kick) = (h.actor_id, now >= h.next_kick);
                let first = h.next_kick <= h.since + KICK_EVERY + 1e-3;
                let bones = self.skeletons.get(&id).map(|sk| sk.bones.clone()).unwrap_or_default();
                // Standing, its arms are its own to move now (they come back to its animation when
                // it's let go of).
                if state == 0
                    && let Some(l) = self.limp.get_mut(&id)
                {
                    for b in bones.iter().filter(|b| ARMS.contains(&b.code)) {
                        if !l.bones.iter().any(|x| x.0 == b.blend) {
                            let (h0, v0) = gains(b.blend);
                            l.bones.push((b.blend, h0, v0));
                            set_gains(b.blend, 0.0, 0.0);
                        }
                    }
                }
                for b in bones.iter().filter(|b| HANDS.contains(&b.code)) {
                    let Some(at) = centre(b.body) else { continue };
                    let v = velocity(b.body);
                    let to = ((p - at) * CLAW_GAIN).clamp_length_max(CLAW_SPEED * k);
                    set_velocity(b.body, v.lerp(to, CLAW_BLEND));
                }
                if kick && state != 0 {
                    for b in bones.iter().filter(|b| LEGS.contains(&b.code)) {
                        let a = self.random() * std::f32::consts::TAU;
                        let v = Vec3::new(a.cos(), 0.3 + 0.4 * self.random(), a.sin()) * KICK_SPEED * k * (0.5 + 0.5 * self.random());
                        set_velocity(b.body, velocity(b.body) * 0.3 + v);
                    }
                }
                if kick {
                    let wait = KICK_EVERY + KICK_SPREAD * self.random() as f64;
                    if let Some(h) = self.holds[hand].as_mut() {
                        h.next_kick = now + wait;
                    }
                }
                if first && kick {
                    tracing::info!("grabs: {id:#x} struggles (fatigue {fatigue:.0} of {most:.0})");
                }
            }
            let Some(h) = self.holds[hand].as_mut() else { return };
            // Held a while, a living actor takes it as an attack.
            if !dead && !h.attacked && now - h.since > HELD_ATTACK {
                attacked(h.actor);
                h.attacked = true;
            }
        }

        /// Hand `hand` lets go: the spring off, and the throw (none: it slipped). A standing actor
        /// let go of fast is thrown down; a living one's limb blends back to its animation once no
        /// hand has it.
        fn let_go(&mut self, hand: usize, throw: Option<Vec3>, force: f32, now: f64) {
            let Some(h) = self.holds[hand].take() else { return };
            self.done[hand] = h.serial;
            unspring(h.action, h.bone.body);
            let alive = still(h.actor, h.actor_id);
            let speed = throw.map_or(0.0, |v| v.length());
            tracing::info!("grabs: the {} hand lets go of {:#x}{}", if hand == 0 { "left" } else { "right" }, h.actor_id, if throw.is_some() { format!(" at {speed:.2} m/s") } else { ": its grip slipped".into() });
            if alive && let Some(v) = throw.filter(|_| speed > 0.3) {
                let dead = is_dead(h.actor);
                if !dead && knock_state(h.actor) == 0 && speed > THROW_DOWN && knock_down(h.actor) {
                    tracing::info!("grabs: {:#x} thrown down", h.actor_id);
                    self.limp.entry(h.actor_id).and_modify(|l| l.bones.clear());
                    if !h.attacked {
                        attacked(h.actor);
                    }
                }
                if dead || knock_state(h.actor) != 0 {
                    // The whole body takes what the hand's pull could give it, the held bone the
                    // rest of the hand's speed.
                    let bones = skeleton(mem::deref(h.actor + NI_NODE));
                    let total: f32 = bones.iter().map(|b| mass(b.body)).sum::<f32>().max(1.0);
                    let want = v - velocity(h.bone.body);
                    let most = force * self.scale.unwrap_or(1.0) * THROW_TIME / total;
                    let dv = want.clamp_length_max(most);
                    for b in &bones {
                        let extra = if b.body == h.bone.body { (want - dv) * 0.5 } else { Vec3::ZERO };
                        set_velocity(b.body, velocity(b.body) + dv + extra);
                    }
                    tracing::info!("grabs: {:#x} thrown: {:.2} m/s on all {:.0} kg of it", h.actor_id, dv.length(), total);
                }
            }
            let still_held = self.holds.iter().flatten().any(|o| o.actor_id == h.actor_id);
            if !still_held && let Some(l) = self.limp.get_mut(&h.actor_id) {
                l.back = Some(now);
            }
        }

        /// The limbs let go of blending back to the animation; done, the bodies out of the world.
        fn blend_back(&mut self, now: f64) {
            let held: Vec<u32> = self.holds.iter().flatten().map(|h| h.actor_id).collect();
            self.limp.retain(|id, l| {
                let gone = !still(l.actor, *id) || mem::deref(l.actor + NI_NODE) != l.root;
                if gone {
                    return false;
                }
                // Put in for a hold that never came (let go at once), or untouched a while: back as
                // it was.
                let touching = l.touched.is_some_and(|t| now - t < TOUCH_SETTLE);
                if l.back.is_none() && !held.contains(id) && !touching && now - l.since > 0.25 {
                    l.back = Some(now);
                }
                let Some(t0) = l.back else { return true };
                // Knocked down meanwhile, the game has its bones now.
                let down = is_dead(l.actor) || knock_state(l.actor) != 0;
                if l.restored || down {
                    bodies_in_world(l.root, false);
                    tracing::info!("grabs: {id:#x} let go{}", if down { " (down: its bones are the game's)" } else { ": back to its animation" });
                    return false;
                }
                // From the gains the hands left it (a held limb's none, a touched one's soft) back
                // to its own.
                if l.leaving.len() != l.bones.len() {
                    l.leaving = l.bones.iter().map(|&(blend, ..)| gains(blend)).collect();
                }
                let k = ((now - t0) / BACK_TIME).clamp(0.0, 1.0) as f32;
                for (&(blend, h0, v0), &(h1, v1)) in l.bones.iter().zip(&l.leaving) {
                    if k < 1.0 {
                        set_gains(blend, h1 + (h0 - h1) * k, (v1 + (v0.min(1.0) - v1) * k).max(0.05));
                    } else {
                        set_gains(blend, h0, v0);
                    }
                }
                l.restored = k >= 1.0;
                true
            });
        }
    }
}
