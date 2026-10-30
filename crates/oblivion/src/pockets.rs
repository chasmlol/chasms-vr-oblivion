//! Picking pockets by hand (`rustvr_protocol::pockets`; the research and the design are in
//! `docs/research/pickpocket.md`). While the player sneaks, the people near them wear pockets on
//! their own skeletons, holding their own things; a hand that pulls one out is the game's own
//! pickpocketing of it, and one that lets a thing go in a pocket is the game's own planting.
//!
//! * **Who:** the NPCs (not creatures) the game is fully processing within [`REACH`], alive and not
//!   in combat: whom activating while sneaking would open the game's pickpocketing on. One that has
//!   caught the player before is shut (its high process's flag, +0x168): the hand finds it so, and
//!   the game says why (`sNoPickPocketAgain`).
//! * **Where:** on its bones, every frame ([`places`]): the belt pouches a little behind the hips,
//!   the purse at the belt's front, an inner pocket at the chest, a bag on the back (only while
//!   there's something big for it).
//! * **What:** its inventory as the game's pickpocketing lists it: the game's own entries
//!   (`GetInventoryItem`, 0x4D88F0) and its own filter (0x4854F0: nothing worn, nothing unplayable
//!   or bound), shared out by kind ([`pocket_for`]). Each thing comes out as the game's own model
//!   (a template the runtime draws, from `local_assets/oblivion`), with its card.
//! * **Taking** is the game's: its own chance (0x546660: both Sneaks with their Luck, 0x547B90, and
//!   the value taken) against its own roll, then its own `RemoveItem` into the player's things
//!   (marked stolen) or its own crime and reaction when caught (`PlayerCharacter` vtable +0x23C),
//!   as its container menu does them. Gold comes out as the whole purse.
//! * **The session** is the game's menu: opened as a hand goes in, closed once the hands are away.
//!   Closed having tried nothing, the game's own rummaging check (a Sneak use, then its chance with
//!   nothing taken: failing it is being caught); having taken things, its own check for witnesses of
//!   the theft (vtable +0x238 with the value taken).
//! * **Planting** is the game's rule: a weightless thing goes into the NPC's things, anything else
//!   gets the game's `sInvalidPickpocket`.
//! * **Detection:** each pocket says how aware its NPC is of the player, from the NPC's own
//!   detection of them (`HighProcess` +0x18C: noticed half, seen fully).

use glam::Vec3;
use rustvr_protocol::*;

/// The pockets on a person, each its own place on the skeleton.
pub const POUCH_RIGHT: usize = 0;
pub const POUCH_LEFT: usize = 1;
pub const PURSE: usize = 2;
pub const CHEST: usize = 3;
pub const BAG: usize = 4;
pub const PLACES: usize = 5;
pub const NAMES: [&str; PLACES] = ["the right belt pouch", "the left belt pouch", "the purse", "the inner pocket", "the bag on the back"];

/// People further than this from the player (metres) have no pockets: out of a hand's reach.
pub const REACH: f32 = 3.0;
/// Seconds between looks at who's near and what they carry.
pub const REFRESH: f64 = 0.5;

/// Form types (`TESForm::typeID`) and gold's form id.
const ARMOR: u8 = 0x14;
const BOOK: u8 = 0x15;
const CLOTHING: u8 = 0x16;
const INGREDIENT: u8 = 0x19;
const MISC: u8 = 0x1B;
const SOUL_GEM: u8 = 0x26;
const KEY: u8 = 0x27;
const POTION: u8 = 0x28;
const SIGIL_STONE: u8 = 0x2A;
const APPARATUS: u8 = 0x13;
pub const GOLD: u32 = 0xF;

/// Where a person's bones are (the protocol's space).
#[derive(Clone, Copy, Debug)]
pub struct Skeleton {
    pub pelvis: Vec3,
    pub spine1: Vec3,
    pub spine2: Vec3,
    pub neck: Vec3,
    pub thigh_l: Vec3,
    pub thigh_r: Vec3,
}

/// A person's frame at the hips: up their spine, to their right across their thighs, and forward.
pub fn body_frame(s: &Skeleton) -> Option<(Vec3, Vec3, Vec3)> {
    let up = (s.neck - s.pelvis).try_normalize()?;
    let across = s.thigh_r - s.thigh_l;
    let right = (across - up * across.dot(up)).try_normalize()?;
    Some((up, right, right.cross(up)))
}

/// How far the body's surface is round the spine: its capsule's radius (`combat`'s torso).
const BODY: f32 = 0.17;

/// Each pocket's opening, which way it faces and how far round it a hand is in it: the pouches
/// hanging at the belt a little behind the hips (felt from the side or from behind), the purse at
/// its front, the inner pocket at the left of the chest, the bag at the small of the back. They sit
/// at the body's surface, where the fingers of a hand stopped by the body are in them.
pub fn places(s: &Skeleton) -> Option<[(Vec3, Vec3, f32); PLACES]> {
    let (up, right, fwd) = body_frame(s)?;
    let belt = s.pelvis + up * 0.05;
    // (30 degrees behind the side: a hand in one is anywhere from the side of the hip round to
    // most of the way to the back.)
    let (side, back) = (30f32.to_radians().cos(), 30f32.to_radians().sin());
    let (hip_r, hip_l) = (right * side - fwd * back, -right * side - fwd * back);
    Some([
        (belt + hip_r * BODY, hip_r, 0.11),
        (belt + hip_l * BODY, hip_l, 0.11),
        (belt + fwd * BODY, fwd, 0.09),
        (s.spine2 + fwd * 0.15 - right * 0.08, fwd, 0.08),
        (s.spine1 - fwd * 0.18, -fwd, 0.11),
    ])
}

/// Rings and amulets (`TESBipedModelForm` slot bits: right ring, left ring, amulet).
const SMALL_SLOTS: u16 = (1 << 6) | (1 << 7) | (1 << 8);

/// Which pocket a thing goes in, by what it is: gold in the purse; keys, lockpicks, rings, amulets,
/// gems and soul gems at the left of the belt; potions, food and small things at the right; books,
/// notes and scrolls inside the coat; anything bigger (weapons, armour, clothes, apparatus, arrows,
/// heavy clutter) in the bag on the back.
pub fn pocket_for(kind: u8, form_id: u32, weight: f32, slots: u16) -> usize {
    match kind {
        _ if form_id == GOLD => PURSE,
        KEY | SOUL_GEM | SIGIL_STONE => POUCH_LEFT,
        CLOTHING if slots & SMALL_SLOTS != 0 => POUCH_LEFT,
        MISC if weight <= 0.5 => POUCH_LEFT,
        POTION | INGREDIENT => POUCH_RIGHT,
        MISC if weight <= 2.0 => POUCH_RIGHT,
        BOOK => CHEST,
        _ => BAG,
    }
}

/// How a thing feels to the fingers (`FEEL_*`), by what it is and what its model is.
pub fn feel_of(kind: u8, form_id: u32, weight: f32, model: &str) -> u32 {
    let m = model.to_ascii_lowercase();
    match kind {
        _ if form_id == GOLD => FEEL_COINS,
        KEY => FEEL_KEY,
        POTION | SOUL_GEM | SIGIL_STONE | APPARATUS => FEEL_GLASS,
        BOOK => FEEL_PAPER,
        INGREDIENT => FEEL_SOFT,
        MISC if m.contains("gems") || m.contains("bottle") => FEEL_GLASS,
        MISC if m.contains("key") || m.contains("lockpick") || weight <= 0.5 => FEEL_KEY,
        CLOTHING if m.contains("ring") || m.contains("amulet") => FEEL_KEY,
        CLOTHING => FEEL_SOFT,
        ARMOR if weight < 5.0 => FEEL_SOFT,
        _ => FEEL_HARD,
    }
}

/// A thing in a pocket as an object: a template (the runtime draws copies of it in the hand) with
/// its model's box round it and its weight (the game's weight is about pounds).
pub fn template_desc(id: u32, key: &str, bounds: Option<([f32; 3], [f32; 3])>, weight: f32) -> ObjectDesc {
    let (lo, hi) = bounds.map(|(lo, hi)| (Vec3::from_array(lo), Vec3::from_array(hi))).unwrap_or((Vec3::splat(-0.03), Vec3::splat(0.03)));
    let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
    d.id = id;
    d.kind = OBJ_PROP;
    d.flags = OBJF_TEMPLATE | OBJF_GRABBABLE | OBJF_RUNTIME_DRAWS;
    d.shape = SHAPE_BOX;
    d.center = ((lo + hi) * 0.5).to_array();
    d.half_extents = ((hi - lo) * 0.5).max(Vec3::splat(0.008)).to_array();
    d.mass = (weight * 0.45).clamp(0.02, 20.0);
    set_name(&mut d.name, &format!("oblivion/{key}"));
    d
}

/// How aware a person is of the player, from its detection of them (lost 0, unseen 1, noticed 2,
/// seen 3).
pub fn awareness(state: Option<u8>) -> f32 {
    match state {
        Some(3) => 1.0,
        Some(2) => 0.5,
        _ => 0.0,
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::{self, refr};
    use crate::items::Facts;
    use crate::{camera, combat, items, mem};
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// `TESObjectREFR`: its base form, its 3D; `TESForm`: its type and id; `Actor::process`.
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const FORM_TYPE: usize = 0x4;
    const FORM_ID: usize = 0xC;
    const NPC: u8 = 0x23;
    const PROCESS: usize = 0x58;
    /// `TESBipedModelForm` slots in armour and clothing (+0x64, +0x5C; its mask at +4).
    const ARMOR_SLOTS: usize = 0x68;
    const CLOTHING_SLOTS: usize = 0x60;

    /// The NPC's inventory, as the container menu walks it: how many entries (thiscall: the
    /// reference, whether a merchant's wares), and each (thiscall: index, wares): a heap copy, its
    /// extra lists +0, count +4, form +8, freed by 0x484470 then `FormHeap_Free` (0x401F20).
    const INV_COUNT: usize = 0x004D_8950;
    const INV_ITEM: usize = 0x004D_88F0;
    const ENTRY_FREE: usize = 0x0048_4470;
    const HEAP_FREE: usize = 0x0040_1F20;
    /// Whether the menu shows an entry (thiscall: the actor, barter, the hour, all, 0), and who owns
    /// what an entry is (thiscall).
    const ENTRY_SHOWN: usize = 0x0048_54F0;
    const ENTRY_OWNER: usize = 0x0048_4B70;
    /// A form's value and weight (cdecl).
    const ITEM_VALUE: usize = 0x0047_0520;
    const ITEM_WEIGHT: usize = 0x0047_0620;
    /// A skill with its Luck (cdecl: skill, luck), the pickpocketing chance (cdecl: the thief's
    /// skill, the victim's, the value taken) and the game's random number (cdecl: 0).
    const LUCK_SKILL: usize = 0x0054_7B90;
    const CHANCE: usize = 0x0054_6660;
    const RANDOM: usize = 0x0047_DF80;
    const AV_LUCK: u32 = 7;
    const AV_SNEAK: u32 = 31;
    const AV_BOUNTY: u32 = 0x25;
    /// `TESObjectREFR::RemoveItem` (vtable +0x100: form, extra list, count, the stealing flag, drop,
    /// whose it goes to, where it's dropped, its angles, 1, 0).
    const VT_REMOVE_ITEM: usize = 0x100;
    /// `Actor`: dead (vtable +0x198, 0), knocked out (+0x19C), in combat (+0x334, 1).
    const VT_IS_DEAD: usize = 0x198;
    const VT_KNOCKED_OUT: usize = 0x19C;
    const VT_IN_COMBAT: usize = 0x334;
    /// `PlayerCharacter`: caught pickpocketing (vtable +0x23C: the victim, what, how many), a theft
    /// seen (+0x238: the victim, 0, 0, the value, its owner), a skill used (+0x39C: the skill, the
    /// use, 0).
    const VT_CAUGHT: usize = 0x23C;
    const VT_THEFT: usize = 0x238;
    const VT_USE_SKILL: usize = 0x39C;
    /// Items Pickpocketed and stolen, the player's statistics.
    const PC_PICKPOCKETED: usize = 0x6CC;
    const PC_STOLEN: usize = 0x6C8;
    /// `BaseProcess` vtable +0xAC: it caught the player pickpocketing it.
    const VT_CAUGHT_BEFORE: usize = 0xAC;
    /// `HighProcess` (the game's dynamic cast from `BaseProcess`): its detection list (+0x18C:
    /// `{data, next}`, the data `{actor, state +4, has LOS +8, level +0xC}`).
    const DYNAMIC_CAST: usize = 0x0098_32E6;
    const RTTI_BASE_PROCESS: usize = 0x00B0_5908;
    const RTTI_HIGH_PROCESS: usize = 0x00B0_5924;
    const HP_DETECTION: usize = 0x18C;
    /// `TESObjectREFR::GetOwner` (thiscall).
    const GET_OWNER: usize = 0x004D_B6B0;
    /// The game's corner message (cdecl: text, 0, 1, -1), and its words: "can't pickpocket again",
    /// "can't put things in while pickpocketing" (game settings: their text first).
    const MESSAGE: usize = 0x0057_ACC0;
    const S_NO_PICKPOCKET_AGAIN: usize = 0x00B3_8B28;
    const S_INVALID_PICKPOCKET: usize = 0x00B3_8C40;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A3_0000;

    /// The first template object's id (below the runtime's spawned objects, 0x8000_0000).
    const TEMPLATE_BASE: u32 = 0x7A00_0001;
    /// A reference of the game's own making for a thing in a hand that has none (as xOBSE's
    /// inventory references): `FormHeap_Allocate` (cdecl: size), `TESObjectREFR`'s constructor
    /// (thiscall), its size. Never placed in the world: only for the game's card of it.
    const HEAP_ALLOC: usize = 0x0040_1F00;
    const REFR_CONSTRUCT: usize = 0x004D_9A70;
    const REFR_SIZE: usize = 0x58;
    /// Its count, for a stack (a purse): an `ExtraCount` (its vtable, size; the count at +0xC) at the
    /// head of the reference's extra list (+0x44: the list +4, which types it has +8).
    const EXTRA_COUNT_VTABLE: usize = 0x00A3_583C;
    const EXTRA_COUNT_SIZE: usize = 0x10;
    const EXTRA_COUNT: u8 = 0x2A;
    const EXTRA_LIST: usize = 0x44;

    fn vcall(obj: usize, slot: usize) -> Option<usize> {
        let f = mem::deref(mem::deref(obj) + slot);
        (obj != 0 && CODE.contains(&f)).then_some(f)
    }

    fn is_dead(actor: usize) -> bool {
        vcall(actor, VT_IS_DEAD).is_none_or(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32) -> u8>(f)(actor, 0) } != 0)
    }

    fn knocked_out(actor: usize) -> bool {
        vcall(actor, VT_KNOCKED_OUT).is_some_and(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> u8>(f)(actor) } != 0)
    }

    fn in_combat(actor: usize) -> bool {
        vcall(actor, VT_IN_COMBAT).is_some_and(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32) -> u8>(f)(actor, 1) } != 0)
    }

    /// It caught the player pickpocketing it: the game won't open its pockets again.
    fn caught_before(actor: usize) -> bool {
        let process = mem::deref(actor + PROCESS);
        vcall(process, VT_CAUGHT_BEFORE).is_some_and(|f| unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> u8>(f)(process) } != 0)
    }

    /// Its detection of the player now (lost 0, unseen 1, noticed 2, seen 3), if it has any.
    fn detection_of_player(actor: usize) -> Option<u8> {
        type Cast = unsafe extern "C" fn(usize, u32, usize, usize, u32) -> usize;
        let process = mem::deref(actor + PROCESS);
        if process == 0 {
            return None;
        }
        let high = unsafe { std::mem::transmute::<usize, Cast>(DYNAMIC_CAST)(process, 0, RTTI_BASE_PROCESS, RTTI_HIGH_PROCESS, 0) };
        let player = engine::player();
        let mut node = mem::deref(high + HP_DETECTION);
        let mut guard = 0;
        while high != 0 && node != 0 && guard < 128 {
            guard += 1;
            let data = mem::deref(node);
            if data != 0 && mem::deref(data) == player {
                return mem::at::<u8>(data + 4);
            }
            node = mem::deref(node + 4);
        }
        None
    }

    fn message(text: &str) {
        let c = std::ffi::CString::new(text.replace('\0', "")).unwrap_or_default();
        type Msg = unsafe extern "C" fn(*const u8, u32, u32, f32) -> u8;
        unsafe { std::mem::transmute::<usize, Msg>(MESSAGE)(c.as_ptr() as *const u8, 0, 1, -1.0) };
    }

    /// A game setting's text.
    fn setting_text(at: usize) -> String {
        combat::read_str(mem::deref(at))
    }

    fn value_of(form: usize) -> i32 {
        unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(usize) -> i32>(ITEM_VALUE)(form) }.max(0)
    }

    fn weight_of(form: usize) -> f32 {
        (unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(usize) -> f64>(ITEM_WEIGHT)(form) } as f32).max(0.0)
    }

    /// The game's pickpocketing chance of `thief` taking `value` from `victim` (percent).
    fn chance(thief: usize, victim: usize, value: f32) -> i32 {
        type Skill = unsafe extern "C" fn(i32, i32) -> f64;
        type Chance = unsafe extern "C" fn(i32, i32, f32) -> i32;
        let skill = |a: usize| unsafe { std::mem::transmute::<usize, Skill>(LUCK_SKILL)(combat::actor_value(a, AV_SNEAK), combat::actor_value(a, AV_LUCK)) } as i32;
        unsafe { std::mem::transmute::<usize, Chance>(CHANCE)(skill(thief), skill(victim), value) }
    }

    /// The game's roll, 0..99.
    fn roll() -> i32 {
        (unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(u32) -> i32>(RANDOM)(0) }).rem_euclid(100)
    }

    #[allow(clippy::too_many_arguments)]
    fn remove_item(from: usize, form: usize, extra: usize, count: i32, stealing: bool, drop: bool, to: usize, at: Option<([f32; 3], [f32; 3])>) {
        type Remove = unsafe extern "thiscall" fn(usize, usize, usize, i32, u32, u32, usize, *const f32, *const f32, u32, u32);
        let Some(f) = vcall(from, VT_REMOVE_ITEM) else { return };
        let (pos, rot) = at.unwrap_or(([0.0; 3], [0.0; 3]));
        let (p, r) = if at.is_some() { (pos.as_ptr(), rot.as_ptr()) } else { (std::ptr::null(), std::ptr::null()) };
        unsafe { std::mem::transmute::<usize, Remove>(f)(from, form, extra, count, stealing as u32, drop as u32, to, p, r, 1, 0) };
    }

    fn owner_of(refr: usize) -> usize {
        unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(GET_OWNER)(refr) }
    }

    /// One entry of a person's things as the game's pickpocketing lists them.
    #[derive(Clone, Debug)]
    struct Entry {
        form: usize,
        extra: usize,
        count: i32,
        owner: usize,
    }

    /// A person's things as the game's pickpocketing lists them (its own entries, its own filter).
    fn inventory(actor: usize) -> Vec<Entry> {
        type Count = unsafe extern "thiscall" fn(usize, u32) -> i32;
        type Item = unsafe extern "thiscall" fn(usize, i32, u32) -> usize;
        type Shown = unsafe extern "thiscall" fn(usize, usize, u32, i32, u32, u32) -> u8;
        type Owner = unsafe extern "thiscall" fn(usize) -> usize;
        type Free = unsafe extern "thiscall" fn(usize);
        type HeapFree = unsafe extern "C" fn(usize);
        let n = unsafe { std::mem::transmute::<usize, Count>(INV_COUNT)(actor, 0) };
        let mut out = Vec::new();
        for i in 0..n.clamp(0, 256) {
            let e = unsafe { std::mem::transmute::<usize, Item>(INV_ITEM)(actor, i, 0) };
            if e == 0 {
                continue;
            }
            let form = mem::deref(e + 8);
            let count = mem::at::<i32>(e + 4).unwrap_or(0);
            if form != 0 && count > 0 && unsafe { std::mem::transmute::<usize, Shown>(ENTRY_SHOWN)(e, actor, 0, 12, 0, 0) } != 0 {
                let lists = mem::deref(e);
                let extra = if lists != 0 { mem::deref(lists) } else { 0 };
                out.push(Entry { form, extra, count, owner: unsafe { std::mem::transmute::<usize, Owner>(ENTRY_OWNER)(e) } });
            }
            unsafe {
                std::mem::transmute::<usize, Free>(ENTRY_FREE)(e);
                std::mem::transmute::<usize, HeapFree>(HEAP_FREE)(e);
            }
        }
        out
    }

    /// One thing in a person's pockets.
    #[derive(Clone, Debug)]
    struct Thing {
        form: usize,
        id: u32,
        count: i32,
        value: i32,
        facts: Facts,
        template: u32,
        pocket: usize,
        feel: u32,
    }

    impl Thing {
        /// How many come out together: a purse's coins all at once, anything else one at a time.
        fn takes(&self) -> i32 {
            if self.id == GOLD { self.count } else { 1 }
        }
    }

    struct Person {
        actor: usize,
        name: String,
        things: Vec<Thing>,
        shut: bool,
    }

    /// A creature's pickpocketing, open: whether anything has been tried (the menu's "nothing
    /// clicked", 0xB13E90), and the value of what's been taken (its stolen total, 0xB3B27C).
    #[derive(Default)]
    struct Session {
        tried: bool,
        stolen: i32,
    }

    /// A thing the player took, in the runtime's hand: what it is and how many.
    struct Taken {
        form: usize,
        count: i32,
        template: u32,
        name: String,
    }

    struct Template {
        desc: ObjectDesc,
        info: ItemInfo,
    }

    struct State {
        people: HashMap<u32, Person>,
        templates: HashMap<usize, u32>,
        descs: HashMap<u32, Template>,
        next_template: u32,
        /// Model boxes by asset key (None: not extracted).
        bounds: HashMap<String, Option<([f32; 3], [f32; 3])>>,
        taken: HashMap<u32, Taken>,
        sessions: HashMap<u32, Session>,
        table: Box<Pockets>,
        /// The templates as last published.
        sent: Vec<u32>,
        next_refresh: f64,
        /// Who had pockets when last said, for the log.
        said: Vec<u32>,
        /// The references made for the game's card of a thing out of a pocket, by template.
        card_refs: HashMap<u32, usize>,
        states: Box<ObjectStates>,
        bounty: i32,
        /// Things just dropped, to set flying as they left the hand once the game has their bodies.
        throws: Vec<Throw>,
        /// What each hand is on in the pockets, as the runtime last said ([`PE_FEEL`]): whose
        /// pockets (0: none), the template of the thing under its fingers (0: none), since when.
        felt: [Felt; 2],
    }

    /// What a hand is on in the pockets.
    #[derive(Clone, Copy, Default)]
    struct Felt {
        actor: u32,
        template: u32,
        since: f64,
        /// Whose pockets it was last in (kept as it comes out: whom a thing in it is being taken
        /// from until the game has decided).
        from: u32,
    }

    struct Throw {
        form: usize,
        at: Vec3,
        lin: Vec3,
        until: f64,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut s = STATE.lock().unwrap();
        f(s.get_or_insert_with(|| State {
            people: HashMap::new(),
            templates: HashMap::new(),
            descs: HashMap::new(),
            next_template: TEMPLATE_BASE,
            bounds: HashMap::new(),
            taken: HashMap::new(),
            sessions: HashMap::new(),
            table: bytemuck::zeroed_box(),
            sent: Vec::new(),
            next_refresh: 0.0,
            said: Vec::new(),
            card_refs: HashMap::new(),
            states: bytemuck::zeroed_box(),
            bounty: 0,
            throws: Vec::new(),
            felt: [Felt::default(); 2],
        }))
    }

    fn player_pos() -> Option<Vec3> {
        mem::at::<[f32; 3]>(engine::player() + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p)))
    }

    /// Someone whose pockets are open to a sneaking player now: an NPC, alive, not fighting,
    /// within reach.
    fn pickable(actor: usize, me: Vec3) -> bool {
        if !combat::is_actor(actor) || mem::deref(actor + REFR_NODE) == 0 {
            return false;
        }
        let base = mem::deref(actor + REFR_BASE);
        if mem::at::<u8>(base + FORM_TYPE) != Some(NPC) {
            return false;
        }
        let Some(at) = mem::at::<[f32; 3]>(actor + refr::POS).map(|p| camera::basis().point_from_game(Vec3::from(p))) else { return false };
        at.distance(me) <= REACH && !is_dead(actor) && !in_combat(actor)
    }

    impl State {
        /// The template for a form: the game's model as the runtime draws it, and its card.
        fn template(&mut self, form: usize, facts: &Facts, takes: i32) -> u32 {
            if let Some(&id) = self.templates.get(&form) {
                if let Some(t) = self.descs.get_mut(&id) {
                    t.info.count = takes.max(1) as u32;
                }
                return id;
            }
            let id = self.next_template;
            self.next_template += 1;
            let model = items::model_of(form).unwrap_or_default();
            let key = oblivion_assets::export::model_key(&model);
            let bounds = *self.bounds.entry(key.clone()).or_insert_with(|| match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
                Ok(m) => Some((m.bounds_min, m.bounds_max)),
                Err(e) => {
                    tracing::warn!("pockets: {} has no extracted model ({model}, {key}: {e:#}); run oblivion-extract export meshes/clutter", facts.name);
                    None
                }
            });
            let desc = template_desc(id, &key, bounds, facts.weight.unwrap_or(0.0));
            // (Taking it is stealing: it's the NPC's.)
            let mut info = items::card(id, &Facts { stolen: true, count: takes.max(1) as u32, ..facts.clone() });
            info.count = takes.max(1) as u32;
            self.templates.insert(form, id);
            self.descs.insert(id, Template { desc, info });
            id
        }

        /// What a person carries, shared out into its pockets.
        fn things(&mut self, actor: usize) -> Vec<Thing> {
            let mut out: Vec<Thing> = Vec::new();
            for e in inventory(actor) {
                let id = mem::at::<u32>(e.form + FORM_ID).unwrap_or(0);
                // (Stacks of one thing, merged: the hand takes from them as one.)
                if let Some(t) = out.iter_mut().find(|t| t.form == e.form) {
                    t.count += e.count;
                    continue;
                }
                let facts = items::form_facts(e.form);
                let weight = weight_of(e.form);
                let kind = facts.kind;
                let slots = match kind {
                    ARMOR => mem::at::<u16>(e.form + ARMOR_SLOTS).unwrap_or(0),
                    CLOTHING => mem::at::<u16>(e.form + CLOTHING_SLOTS).unwrap_or(0),
                    _ => 0,
                };
                let model = items::model_of(e.form).unwrap_or_default();
                out.push(Thing {
                    form: e.form,
                    id,
                    count: e.count,
                    value: value_of(e.form),
                    template: 0,
                    pocket: pocket_for(kind, id, weight, slots),
                    feel: feel_of(kind, id, weight, &model),
                    facts,
                });
            }
            for t in out.iter_mut() {
                t.template = self.template(t.form, &t.facts, t.takes());
            }
            // A pocket holds twelve; more go on to the next with room.
            let mut filled = [0usize; PLACES];
            for t in out.iter_mut() {
                let mut k = t.pocket;
                for _ in 0..PLACES {
                    if filled[k] < MAX_POCKET_ITEMS {
                        break;
                    }
                    k = (k + 1) % PLACES;
                }
                t.pocket = k;
                filled[k] += 1;
            }
            out
        }

        /// Who's near and what they carry, again.
        fn refresh(&mut self) {
            // (What being caught comes to, as the game settles it after: the bounty.)
            let bounty = combat::actor_value(engine::player(), AV_BOUNTY);
            if bounty != self.bounty {
                tracing::info!("pockets: the player's bounty {} -> {bounty}", self.bounty);
                self.bounty = bounty;
            }
            let open = crate::player::sneaking() && !crate::menus::menu_mode();
            let Some(me) = player_pos().filter(|_| open) else {
                self.people.clear();
                return;
            };
            let mut near = Vec::new();
            for (id, actor) in combat::known_actors() {
                if !pickable(actor, me) {
                    continue;
                }
                near.push(id);
                let things = self.things(actor);
                let shut = caught_before(actor);
                let name = items::form_facts(mem::deref(actor + REFR_BASE)).name;
                self.people.insert(id, Person { actor, name, things, shut });
            }
            self.people.retain(|id, _| near.contains(id));
            let mut ids: Vec<u32> = self.people.iter().map(|(&id, p)| id ^ if p.shut { 0x8000_0000 } else { 0 }).collect();
            ids.sort();
            if ids != self.said {
                let list: Vec<String> = self
                    .people
                    .iter()
                    .map(|(id, p)| {
                        let things: Vec<String> = p.things.iter().map(|t| format!("{} x{} in {}", t.facts.name, t.count, NAMES[t.pocket])).collect();
                        format!("{} ({id:#x}{}): {}", p.name, if p.shut { ", shut" } else { "" }, things.join(", "))
                    })
                    .collect();
                tracing::info!("pockets: {} people near with pockets: {}", list.len(), list.join("; "));
                self.said = ids;
            }
        }

        /// The templates of what's in the pockets and in the player's hands, when they change.
        fn publish_templates(&mut self) {
            let mut ids: Vec<u32> = self.people.values().flat_map(|p| p.things.iter().map(|t| t.template)).chain(self.taken.values().map(|t| t.template)).collect();
            ids.sort();
            ids.dedup();
            if ids == self.sent {
                return;
            }
            let descs: Vec<ObjectDesc> = ids.iter().filter_map(|id| self.descs.get(id)).map(|t| t.desc).collect();
            let infos: Vec<ItemInfo> = ids.iter().filter_map(|id| self.descs.get(id)).map(|t| t.info).collect();
            let ok = crate::link::with_adapter(|a| crate::objects::publish_infos(crate::objects::Owner::Pockets, &infos, a) && crate::objects::publish(crate::objects::Owner::Pockets, &descs, a)).unwrap_or(false);
            if ok {
                self.sent = ids;
            }
        }

        /// A reference for the game's card of a template's thing (made once, kept), as many as come
        /// out together.
        fn card_ref(&mut self, template: u32) -> usize {
            let count = self.descs.get(&template).map_or(1, |t| t.info.count.max(1)) as i16;
            if let Some(&r) = self.card_refs.get(&template) {
                let extra = mem::deref(r + EXTRA_LIST + 4);
                if extra != 0 && mem::at::<u8>(extra + 4) == Some(EXTRA_COUNT) {
                    unsafe { ((extra + 0xC) as *mut i16).write(count) };
                }
                return r;
            }
            let Some(form) = self.templates.iter().find(|(_, t)| **t == template).map(|(f, _)| *f) else { return 0 };
            let r = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(usize) -> usize>(HEAP_ALLOC)(REFR_SIZE) };
            if r == 0 {
                return 0;
            }
            unsafe {
                std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> usize>(REFR_CONSTRUCT)(r);
                ((r + REFR_BASE) as *mut usize).write(form);
            }
            // (A stack's count, kept up to date above: "Gold (120)".)
            let extra = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(usize) -> usize>(HEAP_ALLOC)(EXTRA_COUNT_SIZE) };
            if extra != 0 {
                unsafe {
                    std::ptr::write_bytes(extra as *mut u8, 0, EXTRA_COUNT_SIZE);
                    (extra as *mut usize).write(EXTRA_COUNT_VTABLE);
                    ((extra + 4) as *mut u8).write(EXTRA_COUNT);
                    ((extra + 0xC) as *mut i16).write(count);
                    let list = r + EXTRA_LIST;
                    ((extra + 8) as *mut usize).write(mem::deref(list + 4));
                    ((list + 4) as *mut usize).write(extra);
                    let bits = (list + 8 + (EXTRA_COUNT / 8) as usize) as *mut u8;
                    bits.write(bits.read() | 1 << (EXTRA_COUNT % 8));
                }
            }
            self.card_refs.insert(template, r);
            r
        }

        /// The game's card of what a hand has out of a pocket, else of the thing a hand's fingers are
        /// on in one (the latest hand to come on one); and whom taking it is from, until the game has
        /// made it the player's.
        fn card(&mut self) {
            if !crate::link::with_adapter(|a| a.read_object_states(&mut self.states)).unwrap_or(false) {
                return;
            }
            let n = (self.states.count as usize).min(MAX_OBJECTS);
            let held = self.states.states[..n].iter().find(|s| s.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0 && self.descs.contains_key(&s.desc));
            let shown = match held {
                Some(s) => {
                    let hand = if s.flags & OS_HELD_LEFT != 0 { 0 } else { 1 };
                    Some((s.desc, if self.taken.contains_key(&s.id) { 0 } else { self.felt[hand].from }))
                }
                None => self.felt.iter().filter(|f| f.template != 0).max_by(|a, b| a.since.total_cmp(&b.since)).map(|f| (f.template, f.actor)),
            };
            let refr = shown.map_or(0, |(t, _)| self.card_ref(t));
            let from = shown.and_then(|(_, id)| self.people.get(&id)).map_or(0, |p| p.actor);
            crate::hud_info::set_pocket_held(refr, from);
        }

        fn answer(&mut self, n: u64, object: u32, outcome: u32) {
            self.table.push_outcome(PocketOutcome { event: n + 1, object, outcome });
        }

        fn handle(&mut self, n: u64, ev: &PocketEvent) {
            let player = engine::player();
            match ev.kind {
                PE_OPEN => {
                    self.sessions.insert(ev.actor, Session::default());
                    if let Some(p) = self.people.get(&ev.actor) {
                        tracing::info!("pockets: a hand into {} of {} ({:#x}): its pickpocketing opens{}", NAMES[(ev.pocket & 7).saturating_sub(1) as usize % PLACES], p.name, ev.actor, if p.shut { " (shut: it caught the player before)" } else { "" });
                        if p.shut {
                            message(&format!("{} {}", p.name, setting_text(S_NO_PICKPOCKET_AGAIN)));
                        }
                    }
                }
                PE_TAKE => self.take(n, ev, player),
                PE_PLANT => self.plant(n, ev, player),
                PE_DROP => {
                    let Some(t) = self.taken.remove(&ev.object) else { return };
                    let b = camera::basis();
                    let pos = b.point_to_game(Vec3::from_array(ev.pos)).to_array();
                    let fwd = glam::Quat::from_array(ev.rot) * Vec3::Z;
                    let (_, heading) = combat::angles(fwd);
                    remove_item(player, t.form, 0, t.count, false, true, 0, Some((pos, [0.0, 0.0, heading])));
                    tracing::info!("pockets: {} x{} let go in the world: the game's own there now, dropped at {pos:?}", t.name, t.count);
                    self.answer(n, ev.object, PO_DROPPED);
                    // It flies on as it left the hand, once the game has its body.
                    self.throws.push(Throw { form: t.form, at: Vec3::from_array(ev.pos), lin: Vec3::from_array(ev.vel), until: crate::link::now() + 1.0 });
                }
                PE_STOW => {
                    if let Some(t) = self.taken.remove(&ev.object) {
                        tracing::info!("pockets: {} x{} put in a body slot: the player keeps it", t.name, t.count);
                    }
                }
                PE_CLOSE => {
                    // (Whatever a hand was on in them is gone with them.)
                    for f in self.felt.iter_mut().filter(|f| f.actor == ev.actor) {
                        *f = Felt { from: f.from, ..Felt::default() };
                    }
                    self.close(ev.actor, player)
                }
                PE_FEEL => {
                    let hand = (ev.hand as usize).min(1);
                    let f = &mut self.felt[hand];
                    *f = Felt { actor: ev.actor, template: ev.object, since: crate::link::now(), from: if ev.actor != 0 { ev.actor } else { f.from } };
                    if let Some(t) = self.people.get(&ev.actor).and_then(|p| p.things.iter().find(|t| t.template == ev.object && ev.object != 0)) {
                        tracing::info!("pockets: the {} hand's fingers on {} in {}: its card", if hand == 0 { "left" } else { "right" }, t.facts.name, NAMES[t.pocket]);
                    }
                }
                _ => {}
            }
        }

        /// A thing pulled out: the game's chance against its roll, then its own transfer or its own
        /// crime.
        fn take(&mut self, n: u64, ev: &PocketEvent, player: usize) {
            let Some(p) = self.people.get(&ev.actor).filter(|p| combat::is_actor(p.actor)) else {
                tracing::info!("pockets: {:#x} isn't near any more: {:#x} stays with them", ev.actor, ev.item);
                self.answer(n, ev.object, PO_CAUGHT);
                return;
            };
            let actor = p.actor;
            let name = p.name.clone();
            let Some(thing) = p.things.iter().find(|t| t.id == ev.item).cloned() else {
                self.answer(n, ev.object, PO_CAUGHT);
                return;
            };
            self.sessions.entry(ev.actor).or_default().tried = true;
            // As it is now: the game's entry for it (its extra list and owner).
            let Some(entry) = inventory(actor).into_iter().find(|e| e.form == thing.form) else {
                tracing::info!("pockets: {} hasn't {} any more", name, thing.facts.name);
                self.answer(n, ev.object, PO_CAUGHT);
                return;
            };
            let count = if thing.id == GOLD { entry.count } else { 1 };
            let value = thing.value * count;
            let chance = chance(player, actor, value as f32);
            let r = roll();
            let knocked = knocked_out(actor);
            tracing::info!(
                "pockets: {} x{count} (value {value}) out of {} of {name} at {:.2} m/s: the game's chance {chance} against its roll {r}{}",
                thing.facts.name,
                NAMES[thing.pocket],
                ev.speed,
                if knocked { " (knocked out: it can't catch the player)" } else { "" }
            );
            if chance > r || knocked {
                remove_item(actor, thing.form, entry.extra, count, true, false, player, None);
                if thing.id != GOLD {
                    if let Some(c) = mem::at::<i32>(player + PC_PICKPOCKETED) {
                        unsafe { ((player + PC_PICKPOCKETED) as *mut i32).write(c + count) };
                    }
                }
                // (What was the player's own already isn't stolen.)
                let theirs = entry.owner != 0 && entry.owner == mem::deref(player + REFR_BASE);
                if !theirs {
                    self.sessions.entry(ev.actor).or_default().stolen += value;
                    if let Some(c) = mem::at::<i32>(player + PC_STOLEN) {
                        unsafe { ((player + PC_STOLEN) as *mut i32).write(c + count) };
                    }
                }
                self.taken.insert(ev.object, Taken { form: thing.form, count, template: thing.template, name: thing.facts.name.clone() });
                tracing::info!("pockets: taken: {} x{count} is the player's", thing.facts.name);
                self.answer(n, ev.object, PO_TAKEN);
            } else {
                // The game closes its menu, then catches the player: its crime, the victim's
                // reaction, and no pickpocketing it again.
                self.sessions.remove(&ev.actor);
                let bounty = combat::actor_value(player, AV_BOUNTY);
                if let Some(f) = vcall(player, VT_CAUGHT) {
                    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, usize, usize, i32)>(f)(player, actor, thing.form, count) };
                }
                tracing::info!(
                    "pockets: caught by {name}: {} stays with them; the player's bounty {bounty} -> {}{}",
                    thing.facts.name,
                    combat::actor_value(player, AV_BOUNTY),
                    if caught_before(actor) { " (it won't be robbed again)" } else { "" }
                );
                self.answer(n, ev.object, PO_CAUGHT);
            }
            let things = self.things(actor);
            let shut = caught_before(actor);
            if let Some(p) = self.people.get_mut(&ev.actor) {
                p.things = things;
                p.shut = shut;
            }
            self.publish_templates();
        }

        /// A thing let go in a pocket: the game's rule, a weightless thing goes over and anything else
        /// is refused.
        fn plant(&mut self, n: u64, ev: &PocketEvent, player: usize) {
            let Some(p) = self.people.get(&ev.actor).filter(|p| combat::is_actor(p.actor)) else {
                self.answer(n, ev.object, PO_REFUSED);
                return;
            };
            let (actor, name) = (p.actor, p.name.clone());
            let opened = self.sessions.contains_key(&ev.actor);
            self.sessions.entry(ev.actor).or_default().tried = true;
            // One the player took (its form and count), or a thing lying about the hand had (the
            // object is its reference).
            let (form, count, refr) = match self.taken.get(&ev.object) {
                Some(t) => (t.form, t.count, 0),
                None => match items::reference(ev.object) {
                    Some(r) => (mem::deref(r + REFR_BASE), items::facts(r).count as i32, r),
                    None => {
                        self.answer(n, ev.object, PO_REFUSED);
                        return;
                    }
                },
            };
            let what = items::form_facts(form).name;
            let weight = weight_of(form);
            if weight > 0.0 {
                tracing::info!("pockets: {what} (weight {weight}) let go in a pocket of {name}: the game won't have anything with weight planted");
                message(&setting_text(S_INVALID_PICKPOCKET));
                self.answer(n, ev.object, PO_REFUSED);
            } else {
                // A thing lying about is the player's first, by the game's own pick-up.
                if refr != 0 && !items::take(refr, ev.object) {
                    self.answer(n, ev.object, PO_REFUSED);
                    return;
                }
                remove_item(player, form, 0, count, false, false, actor, None);
                self.taken.remove(&ev.object);
                tracing::info!("pockets: {what} x{count} planted on {name}");
                self.answer(n, ev.object, PO_PLANTED);
                let things = self.things(actor);
                if let Some(p) = self.people.get_mut(&ev.actor) {
                    p.things = things;
                }
            }
            // Planted without a hand having been in its pockets first: opened and closed at once.
            if !opened {
                self.close(ev.actor, player);
            }
            self.publish_templates();
        }

        /// Its pickpocketing closes: the game's close (`ContainerMenu` 0x5982A0).
        fn close(&mut self, id: u32, player: usize) {
            let Some(s) = self.sessions.remove(&id) else { return };
            let Some(actor) = self.people.get(&id).map(|p| p.actor).or_else(|| combat::known_actors().into_iter().find(|a| a.0 == id).map(|a| a.1)).filter(|&a| combat::is_actor(a)) else { return };
            if !s.tried {
                if is_dead(actor) || knocked_out(actor) {
                    return;
                }
                // Looked through them and took nothing: a use of Sneak, and the game's chance with
                // nothing taken that they noticed.
                if let Some(f) = vcall(player, VT_USE_SKILL) {
                    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, u32, u32, f32)>(f)(player, AV_SNEAK, 1, 0.0) };
                }
                let (c, r) = (chance(player, actor, 0.0), roll());
                tracing::info!("pockets: the hands came away from {id:#x} with nothing: the game's chance {c} against its roll {r}");
                if c < r && let Some(f) = vcall(player, VT_CAUGHT) {
                    tracing::info!("pockets: {id:#x} noticed the hands in its pockets");
                    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, usize, usize, i32)>(f)(player, actor, 0, 0) };
                }
            } else if s.stolen > 0 {
                // Things taken: whoever sees the player now saw a theft.
                tracing::info!("pockets: done with {id:#x}, {} gold's worth taken: the game's check for witnesses", s.stolen);
                if let Some(f) = vcall(player, VT_THEFT) {
                    let owner = owner_of(actor);
                    unsafe { std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, usize, usize, usize, i32, usize) -> i32>(f)(player, actor, 0, 0, s.stolen, owner) };
                }
            }
        }
    }

    /// Once a game frame, where its scripts run: what the hands did, done by the game; and every
    /// [`REFRESH`], who's near and what they carry.
    pub fn frame(now: f64) {
        if engine::player() == 0 || mem::deref(engine::player() + REFR_NODE) == 0 {
            return;
        }
        let events = crate::link::with_adapter(|a| a.pocket_events()).unwrap_or_default();
        with_state(|s| {
            for (n, ev) in &events {
                s.handle(*n, ev);
            }
            s.throws.retain(|t| {
                let thrown = items::throw_new(t.form, t.at, t.lin, Vec3::ZERO);
                if thrown {
                    tracing::info!("pockets: the dropped thing flies on at {:.2} m/s", t.lin.length());
                }
                now < t.until && !thrown
            });
            if now >= s.next_refresh {
                s.next_refresh = now + REFRESH;
                s.refresh();
                s.publish_templates();
            }
        });
    }

    /// Before the eyes are drawn: each person's pockets where its bones are now, what's in them and
    /// how aware it is of the player.
    pub fn publish() {
        with_state(|s| {
            s.card();
            let State { people, table, .. } = s;
            table.count = 0;
            for (&id, p) in people.iter() {
                if !combat::is_actor(p.actor) {
                    continue;
                }
                let root = mem::deref(p.actor + REFR_NODE);
                if root == 0 {
                    continue;
                }
                let b = combat::bones(root);
                let at = |name: &str| b.get(name).and_then(|&n| combat::node_pos(n));
                let (Some(pelvis), Some(spine1), Some(spine2), Some(neck), Some(thigh_l), Some(thigh_r)) =
                    (at("Bip01 Pelvis"), at("Bip01 Spine1"), at("Bip01 Spine2"), at("Bip01 Neck"), at("Bip01 L Thigh"), at("Bip01 R Thigh"))
                else {
                    static SAID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                    if SAID.swap(id, std::sync::atomic::Ordering::Relaxed) != id {
                        let mut names: Vec<&String> = b.keys().collect();
                        names.sort();
                        tracing::warn!("pockets: {} hasn't all the bones pockets go on: {names:?}", p.name);
                    }
                    continue;
                };
                let Some(places) = places(&Skeleton { pelvis, spine1, spine2, neck, thigh_l, thigh_r }) else { continue };
                let aware = awareness(detection_of_player(p.actor));
                for (k, &(centre, mouth, radius)) in places.iter().enumerate() {
                    let things: Vec<&Thing> = p.things.iter().filter(|t| t.pocket == k).collect();
                    // The bag's there only while there's something big for it.
                    if (k == BAG && things.is_empty()) || table.count as usize >= MAX_POCKETS {
                        continue;
                    }
                    let mut pk: Pocket = bytemuck::Zeroable::zeroed();
                    pk.id = (id << 3) | (k as u32 + 1);
                    pk.actor = id;
                    pk.part = 1;
                    pk.flags = if p.shut { POCKET_SHUT } else { 0 };
                    pk.center = centre.to_array();
                    pk.radius = radius;
                    pk.mouth = mouth.to_array();
                    pk.awareness = aware;
                    for t in things.iter().take(MAX_POCKET_ITEMS) {
                        pk.items[pk.count as usize] = PocketItem { id: t.id, object: t.template, feel: t.feel, count: t.takes().max(1) as u32 };
                        pk.count += 1;
                    }
                    table.pockets[table.count as usize] = pk;
                    table.count += 1;
                }
            }
            table.frame = table.frame.wrapping_add(1);
            // Every two seconds while there are any: where they are from the game's eye, as
            // (right, up, forward) where it looks, for checking and for scripting a hand to them.
            if table.count > 0 && table.frame % 180 == 1
                && let Some(cam) = camera::read(engine::world_camera())
            {
                let (eye, yaw) = (cam.eye(), cam.look_yaw());
                let (right, fwd) = (Vec3::new(yaw.cos(), 0.0, -yaw.sin()), Vec3::new(yaw.sin(), 0.0, yaw.cos()));
                let list: Vec<String> = table
                    .pockets()
                    .iter()
                    .map(|p| {
                        let d = Vec3::from_array(p.center) - eye;
                        format!("{:#x} [{:.2}, {:.2}, {:.2}] r {:.2} ({} things, aware {:.1})", p.id, d.dot(right), d.y, d.dot(fwd), p.radius, p.count, p.awareness)
                    })
                    .collect();
                tracing::info!("pockets: from the eye (right, up, forward): {}", list.join(", "));
            }
            crate::link::with_adapter(|a| a.write_pockets(table));
        });
    }
}
