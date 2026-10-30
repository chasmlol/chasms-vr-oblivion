//! Oblivion's bows and arrows in the hands (`docs/research/archery.md`): the equipped bow as the
//! runtime's bow ([`OBJ_BOW`]), the equipped arrows as the arrows its quiver hands out
//! ([`OBJ_ARROW`]), and each arrow the hands loose ([`CE_LOOSE`]) fired by the game as its own.
//!
//! * **The bow** is the game's own model, measured on it: its grip, where the arrow rests, its
//!   string at rest and how far the game's own bend draws it (`oblivion-extract` writes the bow as
//!   the game bends it at full draw, `#bent`, from the model's morph). It hangs on the back over the
//!   left shoulder, where Oblivion wears it (`BackWeapon`). Its limbs spring back after a shot as the
//!   player's bow animation has them (`bows.toml`, from `bowattack.kf`).
//! * **The arrows** are the equipped arrows' own model (`arrow.nif` without its quiver, which
//!   `oblivion-extract` writes as `#arrow`), as many as the game says the player has.
//! * **A shot** is the game's own, made at once the way the game makes it when its bow animation
//!   lets go (case 5 of the actor's attack actions, 0x5FCAB0): its `ArrowProjectile` (the arrow's
//!   speed, gravity, damage, the bow's and the arrow's enchantments and poison, its sounds in
//!   flight and where it hits and sticks), from the VR arrow's tip along it, with the power the
//!   game gives a bow held drawn that long: the draw stands in for the game's hold timer. Then the
//!   game's own after it: one arrow out of the stack, the shot's fatigue, invisibility broken, the
//!   bow worn. `WPNBowDraw` plays as the string comes back and `bowShoot` as it's loosed, at the
//!   bow, as the animation's sound keys play them; held drawn, the fatigue burns as the game's own
//!   hold burns it.

use glam::{Mat3, Quat, Vec3};
use rustvr_protocol::*;

/// The arrow template's object id (the bow is [`crate::weapons::WEAPON_ID`]).
pub const ARROWS_ID: u32 = 0x0B1E_0004;
/// What the bow and its arrows share (the runtime fits arrows to the bow by it).
pub const ARROW_AMMO: u32 = 0x0A77_0001;
/// `TESObjectWEAP::type` of a bow.
pub const BOW: u32 = 5;

/// The bow model's space (the cache's: the arrow along +X, the limbs along Z, across Y) in the
/// runtime's bow space (the guns': the arrow along +Z, up the limbs +Y, across X).
pub fn bow_space() -> Quat {
    Quat::from_mat3(&Mat3::from_cols(Vec3::Z, Vec3::X, Vec3::Y)).normalize()
}

fn vertices(m: &rustvr_asset::Model) -> impl Iterator<Item = Vec3> + '_ {
    m.meshes.iter().flat_map(|m| &m.vertices).map(|v| Vec3::from_array(v.pos))
}

/// How far back the string's middle is (model space: the least x of what's within 3 cm of the
/// bow's middle).
fn string_at(m: &rustvr_asset::Model) -> Option<f32> {
    vertices(m).filter(|p| p.z.abs() < 0.03).map(|p| p.x).reduce(f32::min)
}

/// The bow as the runtime's object, measured on its model (and its bent one): held by its grip
/// below the arrow (the fist round it, thumb up, the back of the hand out to the arrow's side),
/// the arrow resting on the bow hand's side of the riser at the bow's middle, the string's nock
/// point at rest and how far the game's bend draws it.
pub fn bow_object(model: &rustvr_asset::Model, bent: Option<&rustvr_asset::Model>, weight: f32, key: &str, id: u32) -> ObjectDesc {
    let to = bow_space();
    let string = string_at(model).unwrap_or(-0.22);
    // (A bow without its bent model draws the game's usual 40 cm.)
    let travel = bent.and_then(string_at).map(|s| string - s).filter(|t| *t > 0.05).unwrap_or(0.4);
    // The riser: what's in front of the string, within a hand's width or so of the middle.
    let riser: Vec<Vec3> = vertices(model).filter(|p| p.z.abs() < 0.12 && p.x > string * 0.5).collect();
    let (lo, hi) = riser.iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let (lo, hi) = if riser.is_empty() { (Vec3::new(-0.02, -0.02, -0.11), Vec3::new(0.02, 0.02, 0.11)) } else { (lo, hi) };
    let depth = (lo.x + hi.x) * 0.5;
    let half_width = (hi.y - lo.y) * 0.5;
    let half_height = hi.z.min(-lo.z).max(0.06);
    let (blo, bhi) = (Vec3::from_array(model.bounds_min), Vec3::from_array(model.bounds_max));
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_BOW;
    d.flags = OBJF_GRABBABLE | OBJF_PULLABLE | OBJF_HOLSTERABLE | OBJF_RUNTIME_DRAWS | OBJF_GAME_AMMO;
    // Its collision: the riser and the limbs, not the string's side of it (a hand goes there).
    d.shape = SHAPE_CAPSULE;
    d.radius = 0.03;
    d.half_extents = [0.03, ((bhi.z - blo.z) * 0.5 - 0.1).max(0.1), 0.03];
    d.center = (to * Vec3::new(depth - 0.02, 0.0, 0.0)).to_array();
    // The game's weights are heavy (an iron bow is 8): to kilograms, as the melee weapons'.
    d.mass = (weight * 0.06).clamp(0.3, 3.0);
    // The fist below the arrow's line: its top at the middle, where the arrow crosses.
    let grip = to * Vec3::new(depth, 0.0, -0.055);
    d.grip = grip.to_array();
    d.grip_line = [(to * Vec3::new(depth, 0.0, -(half_height - 0.01).max(0.06))).to_array(), (to * Vec3::new(depth, 0.0, -0.04)).to_array()];
    d.grip_radius = ((hi.x - lo.x) * 0.5).clamp(0.01, 0.03);
    d.grip_rot = fist(Vec3::Y, Vec3::X).to_array();
    // Where the arrow rests: beside the riser on the bow hand's side (the right hand's here), at the
    // bow's middle.
    d.muzzle = (to * Vec3::new(depth, half_width + 0.006, 0.0)).to_array();
    d.handle = (to * Vec3::new(string, 0.0, 0.0)).to_array();
    d.handle_travel = travel;
    d.ammo_type = ARROW_AMMO;
    d.start_slot = 1 + SLOT_BACK_LEFT;
    d.model_rot = to.to_array();
    set_name(&mut d.name, &format!("oblivion/{key}"));
    d
}

/// The equipped arrows as the template the quiver hands out: their model (the arrow alone: its tip
/// at the origin, the shaft back along -Z), as many as the player has.
pub fn arrow_object(model: &rustvr_asset::Model, key: &str, id: u32, count: u32) -> ObjectDesc {
    let (lo, hi) = (Vec3::from_array(model.bounds_min), Vec3::from_array(model.bounds_max));
    let len = (hi.z - lo.z).max(0.3);
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_ARROW;
    d.flags = OBJF_TEMPLATE | OBJF_GRABBABLE | OBJF_RUNTIME_DRAWS | OBJF_GAME_AMMO;
    d.shape = SHAPE_BOX;
    d.half_extents = [0.006, 0.006, len * 0.5];
    d.center = [0.0, 0.0, hi.z - len * 0.5];
    d.mass = 0.03;
    d.muzzle = [0.0, 0.0, hi.z];
    // The nock: a centimetre in from the back end, where the string sits in it.
    d.mag_well = [0.0, 0.0, lo.z + 0.01];
    d.grip = [0.0, 0.0, lo.z + 0.04];
    d.ammo_type = ARROW_AMMO;
    d.rounds = count;
    set_name(&mut d.name, &format!("oblivion/{key}"));
    d
}

/// The controller grip rotation for a fist round a handle along `tube` (little finger to thumb),
/// the back of the hand towards `x` (as `crate::weapons` makes it).
fn fist(tube: Vec3, x: Vec3) -> Quat {
    let z = tube.normalize();
    let x = (x - z * x.dot(z)).normalize();
    Quat::from_mat3(&Mat3::from_cols(x, z.cross(x), z)).normalize()
}

/// The game's power for a draw of `strength` (0..1 of the full draw): what its bow's charge is
/// after being held drawn as long as the game takes to draw that far (`fArrowBowTimerBase` +
/// `fArrowBowTimerMult` x the seconds, at most 1). So a full draw is the game's full charge and no
/// draw its least.
pub fn power(strength: f32, base: f32, mult: f32) -> f32 {
    let mult = if mult > 1e-4 { mult } else { 1.0 };
    let full_time = ((1.0 - base) / mult).max(0.0);
    (base + mult * strength.clamp(0.0, 1.0) * full_time).clamp(0.0, 1.0)
}

/// How a bow's limbs spring back after a shot, from `bows.toml` (written by `oblivion-extract`
/// from the game's bow animation); None without one.
pub fn spring() -> Option<Vec<[f32; 2]>> {
    #[derive(serde::Deserialize)]
    struct File {
        spring: Vec<[f32; 2]>,
    }
    let path = rustvr_asset::default_root().join("oblivion").join("bows.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str::<File>(&text).ok().map(|f| f.spring)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

/// The game's crosshair image (its HUD's reticle), for the runtime's optional crosshair.
pub const RETICLE: &str = r"textures\menus\misc\hud_reticle.dds";

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn arrows_key() -> (usize, u32) {
    (0, 0)
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn register(_e: &crate::weapons::Equipped, _seq: u32, _descs: &mut Vec<ObjectDesc>, _a: &rustvr_adapter::Adapter) {}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, refr};
    use crate::{camera, mem};
    use std::sync::Mutex;

    /// `Actor::process`; `MiddleHighProcess` virtuals: the equipped ammo's and weapon's inventory
    /// entries (argument 1); an entry's form.
    const PROCESS: usize = 0x58;
    const VT_AMMO_ENTRY: usize = 0xF4;
    const VT_WEAPON_ENTRY: usize = 0xEC;
    const ENTRY_FORM: usize = 0x08;
    /// `TESForm` virtual 0x78: an arrow the game won't fire.
    const VT_FORM_UNFIREABLE: usize = 0x78;
    const FORM_AMMO: u8 = 0x22;
    /// `TESAmmo`: name, model path, its speed.
    const AMMO_NAME: usize = 0x28;
    const AMMO_MODEL: usize = 0x34;
    const AMMO_SPEED: usize = 0x7C;
    /// `ExtraDataList::GetCount` (thiscall): an inventory entry's stack's count, from its list.
    const GET_COUNT: usize = 0x0041_E860;
    /// `BaseExtraList::GetExtra` (thiscall), a reference's extra data (+0x44), and the container
    /// changes' extra.
    const GET_EXTRA: usize = 0x0041_E210;
    const REFR_EXTRA: usize = 0x44;
    const EXTRA_CONTAINER_CHANGES: u32 = 0x1A;
    /// `FormHeap_Allocate` (cdecl size), `ArrowProjectile`'s size and constructor (thiscall: the
    /// shooter, x, y, z, heading, pitch, power, the arrows' entry, the bow's), and the process
    /// lists it joins (`ActorProcessManager` 0xB3BD00, thiscall: the projectile, 0, 0, 0, 0).
    const ALLOCATE: usize = 0x0040_1F00;
    const ARROW_SIZE: usize = 0x9C;
    const ARROW_PROJECTILE: usize = 0x0060_C940;
    const PROCESS_MANAGER: usize = 0x00B3_BD00;
    const ADD_TO_PROCESS: usize = 0x0067_3A90;
    /// After the shot: `UseAmmo` (actor virtual), the skill's perk that waives the fatigue (thiscall:
    /// actor, skill), the fatigue taken (thiscall: actor, -amount), invisibility broken (thiscall
    /// on the actor's `MagicTarget` +0x68: the effect's code, 0), the bow's wear (cdecl: its
    /// damage, then actor virtual 0x2C4: the bow's entry, the wear, 0).
    const VT_USE_AMMO: usize = 0x2E8;
    const PERK_WAIVES: usize = 0x005F_23B0;
    const DAMAGE_FATIGUE: usize = 0x005E_07D0;
    const AV_INVISIBILITY: u32 = 47;
    const MAGIC_TARGET: usize = 0x68;
    const DISPEL: usize = 0x006A_24B0;
    const INVISIBILITY: u32 = u32::from_le_bytes(*b"INVI");
    const WEAR: usize = 0x0054_7240;
    const VT_DAMAGE_WEAPON: usize = 0x2C4;
    const WEAP_DAMAGE_FORM: usize = 0x88;
    /// Swimming (thiscall on the actor): the shot is a tenth as strong.
    const SWIMMING: usize = 0x005E_0530;
    /// `PlayerCharacter` +0x5C0: set, the game fires nothing.
    const NO_SHOT: usize = 0x5C0;
    /// Marksman.
    const MARKSMAN: u32 = 28;
    /// The game's settings (each value at the setting's address): `fArrowBowTimerBase`,
    /// `...Mult`, `fArrowSpeedMult`, `fArrowWeakSpeed`, `fMarksmanFatigueBurnPerShot`,
    /// `...PerSecond`, `fArrowMaxDistance`.
    const TIMER_BASE: usize = 0x00B3_7080;
    const TIMER_MULT: usize = 0x00B3_7088;
    const SPEED_MULT: usize = 0x00B3_7040;
    const WEAK_SPEED: usize = 0x00B3_7090;
    const BURN_PER_SHOT: usize = 0x00B3_7020;
    const BURN_PER_SECOND: usize = 0x00B3_7010;
    const MAX_DISTANCE: usize = 0x00B3_7118;
    /// The arrow's gravity for a power and a Marksman skill (cdecl: power, skill, 28).
    const GRAVITY: usize = 0x0054_7700;
    /// Sounds: a `TESSound` by its editor id (stdcall), the sound manager (`OSGlobals` +0x24) making
    /// one from its form id (thiscall: form id, flags, 1), its place, volume, and its playing; the
    /// handle freed after.
    const SOUND_BY_ID: usize = 0x0044_7490;
    const SOUND_MANAGER: usize = 0x24;
    const MAKE_SOUND: usize = 0x006A_E0A0;
    const SOUND_AT: usize = 0x006B_7360;
    const SOUND_VOLUME: usize = 0x006B_7280;
    const SOUND_FLAG: usize = 0x006B_7190;
    const SOUND_PLAY: usize = 0x006B_73E0;
    const FREE: usize = 0x0040_1F20;
    const SOUND_3D: u32 = 0x102;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A3_0000;
    /// The game's arrow gravity (0x547700's) in m/s^2: measured on its arrows' flights, 0.3 at full
    /// draw fell 2.1 m/s^2 and 0.76 at two thirds of it 5.4 (Havok's tenth of a metre, at 70 game
    /// units to the metre, makes it 7).
    const GRAVITY_UNITS: f32 = 7.0;

    fn f32_at(a: usize) -> f32 {
        mem::at::<f32>(a).unwrap_or(0.0)
    }

    /// The equipped arrows: their form and inventory entry, name, model path, speed and how many.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Arrows {
        pub form: usize,
        pub entry: usize,
        pub name: String,
        pub model: String,
        pub speed: f32,
        pub count: u32,
    }

    fn process_entry(slot: usize) -> usize {
        type Get = unsafe extern "thiscall" fn(usize, u32) -> usize;
        let process = mem::deref(engine::player() + PROCESS);
        let f = mem::deref(mem::deref(process) + slot);
        if process == 0 || !CODE.contains(&f) {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, Get>(f)(process, 1) }
    }

    /// How many of `form` the player has: its entry in the player's container changes
    /// (`ExtraContainerChanges`, extra 0x1A: data +0xC, its list of entries +0 as (entry, next)
    /// nodes; an entry's count +4, more than the base container's, which holds no arrows).
    pub fn inventory_count(form: usize) -> Option<u32> {
        type Get = unsafe extern "thiscall" fn(usize, u32) -> usize;
        let player = engine::player();
        let changes = unsafe { std::mem::transmute::<usize, Get>(GET_EXTRA)(player + REFR_EXTRA, EXTRA_CONTAINER_CHANGES) };
        let mut node = mem::deref(mem::deref(changes + 0xC));
        let mut guard = 0;
        while node != 0 && guard < 4096 {
            guard += 1;
            let entry = mem::deref(node);
            if entry != 0 && mem::deref(entry + ENTRY_FORM) == form {
                return mem::at::<i32>(entry + 4).map(|n| n.max(0) as u32);
            }
            node = mem::deref(node + 4);
        }
        None
    }

    /// Arrows drawn from one of the body's slots and put on the bow by the game
    /// (`crate::inventory`): their form, and the runtime's copy of one in a hand, whose place one of
    /// the bow's own arrows takes as they're registered.
    static REPLACING: Mutex<Option<(usize, u32)>> = Mutex::new(None);

    /// The arrows of `form`, equipped from the runtime's copy `copy` in a hand: one of them takes
    /// that copy's place when they're registered.
    pub fn replacing(form: usize, copy: u32) {
        *REPLACING.lock().unwrap() = Some((form, copy));
    }

    pub fn equipped_arrows() -> Option<Arrows> {
        let entry = process_entry(VT_AMMO_ENTRY);
        let form = mem::deref(entry + ENTRY_FORM);
        if entry == 0 || form == 0 || mem::at::<u8>(form + 4) != Some(FORM_AMMO) {
            return None;
        }
        // How many are on the bow's side: the equipped stack's (its first extra list's), what the
        // game fires before it takes them off (`UseAmmo`); and how many the player has in all.
        type Count = unsafe extern "thiscall" fn(usize) -> i16;
        let list = mem::deref(mem::deref(entry));
        let count = if list != 0 { (unsafe { std::mem::transmute::<usize, Count>(GET_COUNT)(list) }).max(0) as u32 } else { 1 };
        let total = inventory_count(form);
        static SAID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);
        let key = ((count as u64) << 32) | total.unwrap_or(u32::MAX) as u64;
        if SAID.swap(key, std::sync::atomic::Ordering::Relaxed) != key {
            tracing::info!("archery: {count} arrows equipped, {total:?} in all");
        }
        Some(Arrows {
            form,
            entry,
            name: crate::combat::read_str(mem::deref(form + AMMO_NAME)),
            model: crate::combat::read_str(mem::deref(form + AMMO_MODEL)),
            speed: f32_at(form + AMMO_SPEED),
            count,
        })
    }

    /// Plays the game's sound `id` (its editor id) at `at` (game units), as the animation's sound
    /// keys play theirs.
    pub fn play_sound(id: &str, at: Vec3) {
        type ById = unsafe extern "stdcall" fn(*const u8) -> usize;
        type Make = unsafe extern "thiscall" fn(usize, u32, u32, u32) -> usize;
        type At = unsafe extern "thiscall" fn(usize, f32, f32, f32) -> i32;
        type Volume = unsafe extern "thiscall" fn(usize, f32) -> i32;
        type Flag = unsafe extern "thiscall" fn(usize, u32) -> i32;
        type Play = unsafe extern "thiscall" fn(usize) -> i32;
        type Free = unsafe extern "C" fn(usize) -> i32;
        let name = std::ffi::CString::new(id).unwrap_or_default();
        let form = unsafe { std::mem::transmute::<usize, ById>(SOUND_BY_ID)(name.as_ptr().cast()) };
        let manager = mem::deref(mem::deref(engine::OS_GLOBALS) + SOUND_MANAGER);
        if form == 0 || manager == 0 {
            return;
        }
        let Some(form_id) = mem::at::<u32>(form + 0xC) else { return };
        unsafe {
            let h = std::mem::transmute::<usize, Make>(MAKE_SOUND)(manager, form_id, SOUND_3D, 1);
            if h == 0 {
                return;
            }
            std::mem::transmute::<usize, At>(SOUND_AT)(h, at.x, at.y, at.z);
            std::mem::transmute::<usize, Volume>(SOUND_VOLUME)(h, 1.0);
            std::mem::transmute::<usize, Flag>(SOUND_FLAG)(h, 0);
            std::mem::transmute::<usize, Play>(SOUND_PLAY)(h);
            std::mem::transmute::<usize, Free>(FREE)(h);
        }
    }

    /// The Marksman perk waives the bow's fatigue.
    fn fatigue_waived(player: usize) -> bool {
        type Waives = unsafe extern "thiscall" fn(usize, u32) -> u32;
        unsafe { std::mem::transmute::<usize, Waives>(PERK_WAIVES)(player, MARKSMAN) != 0 }
    }

    fn damage_fatigue(player: usize, amount: f32) {
        type Damage = unsafe extern "thiscall" fn(usize, f32);
        if amount > 0.0 {
            unsafe { std::mem::transmute::<usize, Damage>(DAMAGE_FATIGUE)(player, -amount) };
        }
    }

    /// How the game's arrow flies at the power a draw of `strength` gives: (m/s, the gravity the
    /// game gives it (its own units, logged to be measured against the flight)).
    pub fn flight(arrows: &Arrows, strength: f32) -> (f32, f32) {
        type Gravity = unsafe extern "C" fn(f32, f32, u32) -> f32;
        let p = power(strength, f32_at(TIMER_BASE), f32_at(TIMER_MULT));
        let speed = arrows.speed * f32_at(SPEED_MULT) * (p + (1.0 - p) * f32_at(WEAK_SPEED)) / camera::UNITS_PER_METER;
        let player = engine::player();
        let skill = if player != 0 { crate::combat::actor_value(player, MARKSMAN) as f32 } else { 0.0 };
        let g = unsafe { std::mem::transmute::<usize, Gravity>(GRAVITY)(p, skill, MARKSMAN) };
        (speed, g)
    }

    /// An arrow in flight, followed for a second (where the game has it), to measure its speed and
    /// fall against what the runtime was told.
    struct Flying {
        arrow: usize,
        since: f64,
        next: f64,
        from: Vec3,
    }

    struct State {
        flying: Vec<Flying>,
        states: Box<ObjectStates>,
        /// The bow's draw last frame, and whether the draw's sound has played for this pull.
        drawn: f32,
        drew: bool,
        last: f64,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State { flying: Vec::new(), states: bytemuck::zeroed_box(), drawn: 0.0, drew: false, last: 0.0 }))
    }

    /// An arrow the hands loosed: fired as the game fires its own when its bow animation lets go.
    pub fn loose(ev: &CombatEvent) {
        let player = engine::player();
        if player == 0 {
            return;
        }
        let (ammo, bow) = (process_entry(VT_AMMO_ENTRY), process_entry(VT_WEAPON_ENTRY));
        let (ammo_form, bow_form) = (mem::deref(ammo + ENTRY_FORM), mem::deref(bow + ENTRY_FORM));
        if ammo == 0 || bow == 0 || ammo_form == 0 || bow_form == 0 || mem::at::<u8>(ammo_form + 4) != Some(FORM_AMMO) {
            tracing::info!("archery: an arrow loosed, but the game has no arrows or bow equipped");
            return;
        }
        type Check = unsafe extern "thiscall" fn(usize) -> u8;
        let check = mem::deref(mem::deref(ammo_form) + VT_FORM_UNFIREABLE);
        if CODE.contains(&check) && unsafe { std::mem::transmute::<usize, Check>(check)(ammo_form) } != 0 {
            tracing::info!("archery: the game won't fire these arrows");
            return;
        }
        if mem::at::<u8>(player + NO_SHOT).is_some_and(|b| b != 0) {
            tracing::info!("archery: the game fires nothing now (+0x5C0 set)");
            return;
        }
        type Swimming = unsafe extern "thiscall" fn(usize) -> u8;
        let mut p = power(ev.strength, f32_at(TIMER_BASE), f32_at(TIMER_MULT));
        if unsafe { std::mem::transmute::<usize, Swimming>(SWIMMING)(player) } != 0 {
            p *= 0.1;
        }
        let at = camera::basis().point_to_game(Vec3::from_array(ev.point));
        let (pitch, heading) = crate::combat::angles(Vec3::from_array(ev.dir));
        let heading = heading.rem_euclid(std::f32::consts::TAU);
        type Allocate = unsafe extern "C" fn(usize) -> usize;
        type Construct = unsafe extern "thiscall" fn(usize, usize, f32, f32, f32, f32, f32, f32, usize, usize) -> usize;
        type Add = unsafe extern "thiscall" fn(usize, usize, u32, u32, u32, u32) -> u32;
        type Actor0 = unsafe extern "thiscall" fn(usize);
        let arrow = unsafe {
            let mem = std::mem::transmute::<usize, Allocate>(ALLOCATE)(ARROW_SIZE);
            if mem == 0 {
                return;
            }
            let arrow = std::mem::transmute::<usize, Construct>(ARROW_PROJECTILE)(mem, player, at.x, at.y, at.z, heading, pitch, p, ammo, bow);
            if arrow != 0 {
                std::mem::transmute::<usize, Add>(ADD_TO_PROCESS)(PROCESS_MANAGER, arrow, 0, 0, 0, 0);
            }
            arrow
        };
        // What the game does after its own shot: an arrow out of the stack, the shot's fatigue,
        // invisibility broken, the bow worn.
        unsafe {
            let use_ammo = mem::deref(mem::deref(player) + VT_USE_AMMO);
            if CODE.contains(&use_ammo) {
                std::mem::transmute::<usize, Actor0>(use_ammo)(player);
            }
            if !fatigue_waived(player) {
                damage_fatigue(player, f32_at(BURN_PER_SHOT));
            }
            if crate::combat::actor_value(player, AV_INVISIBILITY) > 0 {
                type Dispel = unsafe extern "thiscall" fn(usize, u32, u32) -> u8;
                std::mem::transmute::<usize, Dispel>(DISPEL)(player + MAGIC_TARGET, INVISIBILITY, 0);
            }
            type Damage = unsafe extern "thiscall" fn(usize) -> u16;
            type Wear = unsafe extern "C" fn(f32) -> f32;
            type DamageWeapon = unsafe extern "thiscall" fn(usize, usize, f32, u32) -> u8;
            let dmg_form = bow_form + WEAP_DAMAGE_FORM;
            let get = mem::deref(mem::deref(dmg_form) + 0x10);
            let wear_on = mem::deref(mem::deref(player) + VT_DAMAGE_WEAPON);
            if CODE.contains(&get) && CODE.contains(&wear_on) {
                let damage = std::mem::transmute::<usize, Damage>(get)(dmg_form) as f32;
                let wear = std::mem::transmute::<usize, Wear>(WEAR)(damage);
                std::mem::transmute::<usize, DamageWeapon>(wear_on)(player, bow, wear, 0);
            }
        }
        play_sound("bowShoot", at);
        let skill = crate::combat::actor_value(player, MARKSMAN);
        tracing::info!(
            "archery: the game's arrow {arrow:#x} loosed at {:.0}% draw (power {p:.2}, the runtime's speed {:.1} m/s), from {at:.0?} heading {heading:.3} pitch {pitch:.3}; Marksman {skill}; arrows left {:?}",
            ev.strength * 100.0,
            ev.speed,
            equipped_arrows().map(|a| a.count)
        );
        if arrow != 0 {
            let now = crate::link::now();
            with_state(|s| s.flying.push(Flying { arrow, since: now, next: now, from: at }));
        }
    }

    /// Once a game frame: the bow's draw sound as its string first comes back, the fatigue it
    /// burns held drawn, and the arrows just loosed followed (logged) for a second.
    pub fn frame(now: f64) {
        let player = engine::player();
        if player == 0 {
            return;
        }
        with_state(|s| {
            let dt = if s.last > 0.0 { (now - s.last).clamp(0.0, 0.1) as f32 } else { 0.0 };
            s.last = now;
            // The bow's state: its draw (0..1 of full) and whether an arrow's on it.
            let read = crate::link::with_adapter(|a| a.read_object_states(&mut s.states)).unwrap_or(false);
            let bow = read.then(|| s.states.states[..(s.states.count as usize).min(MAX_OBJECTS)].iter().find(|st| st.id == crate::weapons::WEAPON_ID).copied()).flatten();
            let (draw, nocked, at) = match bow {
                Some(b) => (b.fuse, b.rounds > 0, b.pose),
                None => (0.0, false, Pose::IDENTITY),
            };
            let pos = crate::link::last().and_then(|snap| snap.view.map(|v| v.pose(&at).0)).map(|p| camera::basis().point_to_game(p));
            if nocked && draw > 0.08 && !s.drew {
                s.drew = true;
                if let Some(p) = pos {
                    play_sound("WPNBowDraw", p);
                }
            }
            if !nocked || draw < 0.02 {
                s.drew = false;
            }
            // Held drawn, the fatigue burns as the game's own hold burns it.
            if nocked && draw > 0.05 && dt > 0.0 && !fatigue_waived(player) {
                damage_fatigue(player, f32_at(BURN_PER_SECOND) * dt);
            }
            s.drawn = draw;
            // The arrows just loosed: where the game has them, every tenth of a second.
            s.flying.retain_mut(|f| {
                if now - f.since > 1.0 {
                    return false;
                }
                if now >= f.next {
                    f.next = now + 0.1;
                    match mem::at::<[f32; 3]>(f.arrow + refr::POS) {
                        Some(p) => {
                            let p = Vec3::from(p);
                            let d = (p - f.from) / camera::UNITS_PER_METER;
                            tracing::info!("archery: the arrow {:#x} {:.2} s out: {:.2} m along, {:.2} m up", f.arrow, now - f.since, Vec3::new(d.x, d.y, 0.0).length(), d.z);
                        }
                        None => return false,
                    }
                }
                true
            });
        });
    }

    /// The game's settings for its arrows, logged once (and what the runtime is told from them).
    pub fn log_settings(arrows: &Arrows) {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let (slow, g0) = flight(arrows, 0.0);
            let (fast, g1) = flight(arrows, 1.0);
            tracing::info!(
                "archery: the game's arrows: fArrowBowTimerBase {}, Mult {}, fArrowSpeedMult {}, fArrowWeakSpeed {}, fArrowMaxDistance {}, fatigue {} a shot, {} a second; {} speed {}: {slow:.1} m/s at no draw (gravity {g0}), {fast:.1} m/s at full draw (gravity {g1})",
                f32_at(TIMER_BASE),
                f32_at(TIMER_MULT),
                f32_at(SPEED_MULT),
                f32_at(WEAK_SPEED),
                f32_at(MAX_DISTANCE),
                f32_at(BURN_PER_SHOT),
                f32_at(BURN_PER_SECOND),
                arrows.name,
                arrows.speed
            );
        });
    }

    /// The equipped arrows and how many (what the registration changes with).
    pub fn arrows_key() -> (usize, u32) {
        equipped_arrows().map_or((0, 0), |a| (a.form, a.count))
    }

    /// The equipped bow (`e`) and its arrows as the runtime's objects (into `descs`), the bows'
    /// table written for them first: its bent model, its limbs' spring, how the game's arrows fly
    /// and the game's crosshair.
    pub fn register(e: &crate::weapons::Equipped, seq: u32, descs: &mut Vec<ObjectDesc>, a: &rustvr_adapter::Adapter) {
        use oblivion_assets::export::{model_key, texture_key, variant_key};
        let dir = rustvr_asset::default_root().join("oblivion");
        let key = model_key(&e.model);
        let model = match rustvr_asset::load_model(&dir, &key) {
            Ok(m) => m,
            Err(err) => {
                tracing::warn!("archery: {} has no extracted model ({}, {key}: {err:#}); run oblivion-extract export meshes/weapons", e.name, e.model);
                return;
            }
        };
        let bent_key = variant_key(&e.model, "bent");
        let bent = rustvr_asset::load_model(&dir, &bent_key).ok();
        let mut d = bow_object(&model, bent.as_ref(), e.weight, &key, crate::weapons::WEAPON_ID);
        d.spawn_seq = seq;
        let arrows = equipped_arrows();
        let mut bows: Box<Bows> = bytemuck::zeroed_box();
        let mut b: BowDesc = bytemuck::Zeroable::zeroed();
        b.id = crate::weapons::WEAPON_ID;
        b.flags = BOWF_GAME_SOUNDS;
        if bent.is_some() {
            set_name(&mut b.bent, &format!("oblivion/{bent_key}"));
        }
        if let Some(spring) = spring() {
            for (k, key) in spring.iter().take(BOW_SPRING_KEYS).enumerate() {
                b.spring[k] = *key;
            }
            b.spring_count = spring.len().min(BOW_SPRING_KEYS) as u32;
        }
        if let Some(ar) = &arrows {
            log_settings(ar);
            let ((s0, g0), (s1, g1)) = (flight(ar, 0.0), flight(ar, 1.0));
            b.speed = [s0, s1];
            b.gravity = [g0 * GRAVITY_UNITS, g1 * GRAVITY_UNITS];
        }
        // (The game's arrows fly until they hit something or age out: `fArrowMaxDistance` is how
        // far its archers shoot from, not how far an arrow goes.)
        b.range = 0.0;
        set_name(&mut b.crosshair, &format!("oblivion/{}", texture_key(RETICLE)));
        bows.bows[0] = b;
        bows.count = 1;
        a.write_bows(&bows);
        tracing::info!(
            "archery: the bow {} ({}, model {key}{}): grip {:?}, rest {:?}, string {:?} drawn {:.2} m; arrows {:?}; the limbs' spring {} keys",
            e.name,
            e.model,
            if bent.is_some() { ", bent" } else { ", no bent model: run oblivion-extract" },
            d.grip,
            d.muzzle,
            d.handle,
            d.handle_travel,
            arrows.as_ref().map(|a| (&a.name, a.count)),
            b.spring_count
        );
        descs.push(d);
        if let Some(ar) = arrows {
            // The arrow alone, never the plain key: that's the whole model, the quiver and its five
            // arrows with it.
            let akey = variant_key(&ar.model, "arrow");
            match rustvr_asset::load_model(&dir, &akey) {
                Ok(m) => {
                    let mut d = arrow_object(&m, &akey, ARROWS_ID, ar.count);
                    if let Some((_, copy)) = REPLACING.lock().unwrap().take_if(|(form, _)| *form == ar.form) {
                        d.replaces = copy;
                        tracing::info!("archery: {} put on the bow from a slot: one takes the place of its copy {copy:#x}", ar.name);
                    }
                    descs.push(d);
                }
                Err(err) => tracing::warn!("archery: {} have no extracted model ({}#arrow, {akey}: {err:#}); run oblivion-extract export meshes/weapons", ar.name, ar.model),
            }
        }
    }
}
