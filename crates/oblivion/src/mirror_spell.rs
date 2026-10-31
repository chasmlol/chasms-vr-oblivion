//! The mirror spell: "Mirror", a lesser power every player has from the start of the game, that
//! summons the runtime's mirror in front of them, or sends away the one they look at
//! (`rustvr_protocol::EV_MIRROR_CAST`; docs/research/mirror.md). Oblivion has no mirror of its own,
//! as decoration or as an item, so the mirror comes the way the game brings things into the world
//! by magic: conjured.
//!
//! * **The spell** is made as the game makes the spells a player makes at a spellmaking altar: a
//!   created `SpellItem` (a form id from 0xFF000000) in the game's list of created objects, so a
//!   save keeps it with the player's other spells. A save without it (a new game, or one from
//!   before) is given it with the game's own `AddSpell`; one with it is left as it is.
//!   * It's a lesser power: cast as often as the player likes, for no magicka, by anyone.
//!   * Its one effect is a script effect called "Mirror", on the player, with no script (the game
//!     runs none: 0x6A45F0). Its school is Conjuration, and it has the look of Bound Shield
//!     (`BASH`, a thing conjured): its light in the casting hand, its sounds and its icon.
//! * **The mirror's arrival** is a summoned creature's (0x6A65B0): the model of Summon Dremora's
//!   effect (`ZDRE`: `summonDaedra.nif`, every Daedra's) as one of the game's temporary effects at
//!   the mirror's foot, facing the player, playing its `SpecialIdle_SummonEffect`. The glass
//!   appears at that sequence's "Hit" key, where the game brings the creature in
//!   (`rustvr_protocol::EV_MIRROR_HIT`), with the spell's hit sound at the glass. A mirror sent away
//!   goes the same way.

use crate::{combat, engine, link, mem, menus};
use std::sync::Mutex;

/// The spell's name and its effect's (the game's text is English: so are these).
const NAME: &str = "Mirror";
/// The look the spell is given (Bound Shield) and the effect the mirror arrives with (Summon
/// Dremora's, every Daedra's arrival).
const LOOK: u32 = u32::from_le_bytes(*b"BASH");
const ARRIVAL: u32 = u32::from_le_bytes(*b"ZDRE");
/// The arrival's sequence, and its key where the summoned thing comes in.
const ARRIVAL_SEQUENCE: &[u8] = b"SpecialIdle_SummonEffect\0";
const ARRIVAL_KEY: &[u8] = b"Hit\0";

/// `SpellItem`: its name (its `MagicItem`'s `TESFullName`, a `BSStringT` {text, length, size}),
/// its effects (`EffectItemList`, its first entry {effect, next} at +4), its type, cost, mastery and
/// flags.
const SPELL_NAME: usize = 0x1C;
const SPELL_EFFECTS: usize = 0x24;
const SPELL_FIRST_EFFECT: usize = 0x28;
const SPELL_TYPE: usize = 0x34;
const SPELL_COST: usize = 0x38;
const SPELL_LEVEL: usize = 0x3C;
const SPELL_FLAGS: usize = 0x40;
const LESSER_POWER: u32 = 3;
/// `SpellItem` flags: its cost is its own (not worked out from its effects).
const MANUAL_COST: u8 = 0x01;
/// `EffectItem`: its code and its script effect's info (`ScriptEffectInfo`: its script's form id,
/// its school, its name, its visual effect's code, hostile).
const EFFECT_CODE: usize = 0x00;
const EFFECT_SCRIPT_INFO: usize = 0x18;
const SCRIPT_ID: usize = 0x00;
const SCRIPT_SCHOOL: usize = 0x04;
const SCRIPT_NAME: usize = 0x08;
const SCRIPT_VISUAL: usize = 0x10;
const SCRIPT_HOSTILE: usize = 0x14;
const CONJURATION: u32 = 1;
/// `EffectSetting`: its model (`TESModel`, its path at +4) and its hit sound (`TESSound*`).
const SETTING_MODEL: usize = 0x18;
const SETTING_HIT_SOUND: usize = 0x88;
/// `TESForm`: its form id.
const FORM_ID: usize = 0x0C;
/// `TESActorBase::spellList`'s entries (as [`crate::combat`] walks them).
const BASE_FORM: usize = 0x1C;
const SPELL_LIST: usize = 0x58;
/// `TESObjectREFR::parentCell`.
const PARENT_CELL: usize = 0x40;
/// The game's time (`TimeGlobal`): paused, and the frame's time (seconds).
const TIME_GLOBALS: usize = 0x00B3_3E90;
const TIME_PAUSED: usize = 0x0;
const FRAME_TIME: usize = 0xC;

/// How often a game in progress is looked at for the spell (seconds), and how many times the game
/// may refuse to give it before the adapter stops asking (a spell made is kept by the save: never
/// one made each time).
const CHECK_EVERY: f64 = 1.0;
const MOST_TRIES: u32 = 3;

/// The arrival playing now: its summons, the game's seconds to its moment, and the glass's centre
/// (game units) for its sound.
#[derive(Clone, Copy, Debug)]
struct Arrival {
    count: u32,
    left: f32,
    at: [f32; 3],
}

struct State {
    /// When the player's spells were last looked at (seconds), and the spell's form id then.
    checked: f64,
    had: u32,
    /// The spell the adapter made (its form id and `SpellItem`), and how often the game didn't give
    /// one to the player.
    made: Option<(u32, usize)>,
    tries: u32,
    /// The summons count the runtime's frame said last (None before its first frame).
    seen: Option<u32>,
    arrival: Option<Arrival>,
    /// The player is to have no spells at all ([`withhold`]): the spell isn't given again.
    withheld: bool,
}

static STATE: Mutex<State> = Mutex::new(State { checked: f64::MIN, had: 0, made: None, tries: 0, seen: None, arrival: None, withheld: false });

/// The spell isn't given back once it's taken away, until the game closes (the test console's
/// `RemoveAllSpells`: a player with no spells at all).
pub fn withhold() {
    STATE.lock().unwrap().withheld = true;
}

/// Whether `spell` (a `SpellItem`) is the mirror spell.
fn is_mirror(spell: usize) -> bool {
    if spell == 0 || mem::at::<u32>(spell + SPELL_TYPE) != Some(LESSER_POWER) || combat::read_str(mem::deref(spell + SPELL_NAME)) != NAME {
        return false;
    }
    let effect = mem::deref(spell + SPELL_FIRST_EFFECT);
    let info = mem::deref(effect + EFFECT_SCRIPT_INFO);
    effect != 0
        && mem::at::<u32>(effect + EFFECT_CODE) == Some(combat::SCRIPT_EFFECT)
        && info != 0
        && mem::at::<u32>(info + SCRIPT_ID) == Some(0)
        && mem::at::<u32>(info + SCRIPT_VISUAL) == Some(LOOK)
        && combat::read_str(mem::deref(info + SCRIPT_NAME)) == NAME
}

/// The player's mirror spell (its `SpellItem`), or 0.
fn find(player: usize) -> usize {
    let mut entry = mem::deref(player + BASE_FORM) + SPELL_LIST;
    let mut guard = 0;
    while entry != 0 && guard < 512 {
        guard += 1;
        let spell = mem::deref(entry);
        entry = mem::deref(entry + 4);
        if is_mirror(spell) {
            return spell;
        }
    }
    0
}

/// After a cast the adapter made ([`crate::combat`]): the mirror spell calls the runtime's mirror.
pub fn cast(spell: usize) {
    if is_mirror(spell) {
        tracing::info!("mirror spell: cast");
        link::with_adapter(|a| a.mirror_cast());
    }
}

/// Once a game frame, in the world: the spell on the player, and the arrival at the glass of each
/// summons the runtime makes.
pub fn tick() {
    let Some(snap) = link::last() else { return };
    let Some(view) = snap.view else { return };
    let player = engine::player();
    if player == 0 || !menus::in_world() {
        return;
    }
    let now = link::now();
    let mut st = STATE.lock().unwrap();
    // (Not while a menu is open: the magic menu shows the list the spell goes into.)
    if now - st.checked > CHECK_EVERY && !menus::menu_mode() && st.tries < MOST_TRIES && !st.withheld {
        st.checked = now;
        let mut spell = find(player);
        if spell == 0 {
            // The one made before, while it's still the game's (loading a save takes it away), else
            // a new one.
            let made = st.made.filter(|&(id, s)| game::lookup(id) == s && is_mirror(s));
            let s = made.map_or_else(game::make, |(_, s)| s);
            if s != 0 {
                st.made = mem::at::<u32>(s + FORM_ID).map(|id| (id, s));
                spell = game::give(player, s);
            }
            if spell == 0 {
                st.tries += 1;
                if st.tries == MOST_TRIES {
                    tracing::warn!("mirror spell: the game didn't give it {MOST_TRIES} times: not asked again");
                }
            }
        }
        if spell != 0 {
            st.tries = 0;
        }
        let id = mem::at::<u32>(spell + FORM_ID).unwrap_or(0);
        if id != st.had {
            st.had = id;
            if id != 0 {
                tracing::info!("mirror spell: the player has {NAME} ({id:08X})");
            }
        }
    }
    // A summons the runtime made since the last frame: the arrival at its glass.
    let m = snap.frame.mirror;
    let seen = st.seen.replace(m.summons);
    if seen.is_some_and(|s| s != m.summons) && m.flags & rustvr_protocol::MIR_SHOWN != 0 {
        st.arrival = Some(game::arrive(&view, &m, player));
    }
    // Its moment, by the game's own clock (stopped while it's paused or in a menu, as the effect is).
    if let Some(a) = st.arrival.as_mut() {
        let paused = mem::at::<u8>(TIME_GLOBALS + TIME_PAUSED).unwrap_or(0) != 0 || menus::menu_mode();
        if !paused {
            a.left -= mem::at::<f32>(TIME_GLOBALS + FRAME_TIME).unwrap_or(0.0).max(0.0);
        }
        if a.left <= 0.0 {
            let a = *a;
            st.arrival = None;
            tracing::info!("mirror spell: the arrival's moment (summons {})", a.count);
            link::with_adapter(|ad| ad.mirror_hit(a.count));
            game::sound(combat::effect_setting(LOOK), a.at);
        }
    }
}

#[cfg(target_arch = "x86")]
mod game {
    use super::*;
    use crate::camera;
    use glam::Vec3;
    use rustvr_adapter::View;
    use rustvr_protocol::MirrorView;

    /// `CreateFormInstance` (cdecl: the form's type) and a spell's type.
    const CREATE_FORM: usize = 0x0044_7DF0;
    const FORM_SPELL: u8 = 0x10;
    /// `BSStringT::Set` (thiscall: text, least size), the game's allocator.
    const STRING_SET: usize = 0x0040_28D0;
    const ALLOC: usize = 0x0040_1F00;
    /// `EffectItem`'s constructor (thiscall: its magic effect; a script effect gets its info) and
    /// size; `EffectItemList::AddItem` (thiscall).
    const EFFECT_ITEM_NEW: usize = 0x0041_4790;
    const EFFECT_ITEM_SIZE: usize = 0x24;
    const EFFECT_LIST_ADD: usize = 0x0041_4B90;
    /// `DataHandler` and adding a form to it (thiscall), and the created objects a save keeps
    /// (`TESSaveLoadGame`) and adding one to them (thiscall).
    const DATA_HANDLER: usize = 0x00B3_3A98;
    const ADD_TO_DATA_HANDLER: usize = 0x0044_D950;
    const CREATED_OBJECTS: usize = 0x00B3_3B00;
    const ADD_CREATED: usize = 0x0045_9800;
    /// A temporary effect: `BSTempEffectParticle`'s constructor (thiscall: cell, 1.0, node, model,
    /// direction, position, scale, 0) and size, playing one of its model's sequences (thiscall: its
    /// name), the process manager's list of them (thiscall), its model's node, and a sequence's
    /// text key's time (cdecl: node, sequence, key).
    const TEMP_PARTICLE: usize = 0x0057_13F0;
    const TEMP_PARTICLE_SIZE: usize = 0x20;
    const PLAY_SEQUENCE: usize = 0x0057_0C00;
    const PROCESS_MANAGER: usize = 0x00B3_BD00;
    const ADD_TEMP_EFFECT: usize = 0x0067_8D30;
    const TEMP_NODE: usize = 0x18;
    const KEY_TIME: usize = 0x0048_0B00;
    /// The cell's node for effects near a reference (the reference's key, cdecl; the cell's node
    /// for it, thiscall: key, 3).
    const EFFECTS_KEY: usize = 0x004C_9BE0;
    const CELL_NODE: usize = 0x0044_1800;
    /// How long the game makes the direction it gives a summoned creature's arrival (units).
    const ARRIVAL_FACING: f32 = 64.0;
    /// Sounds: the sound manager (`OSGlobals` +0x24) making one from its form id (thiscall: form
    /// id, flags, 1), its place, its volume, starting it and letting it go (thiscall), the handle
    /// freed after.
    const SOUND_MANAGER: usize = 0x24;
    const MAKE_SOUND: usize = 0x006A_E0A0;
    const SOUND_3D: u32 = 0x102;
    const SOUND_AT: usize = 0x006B_7360;
    const SOUND_VOLUME: usize = 0x006B_7280;
    const SOUND_START: usize = 0x006B_7190;
    const SOUND_RELEASE: usize = 0x006B_73E0;
    const FREE: usize = 0x0040_1F20;

    unsafe fn set_string(at: usize, text: &str) {
        let s = std::ffi::CString::new(text).unwrap_or_default();
        type Set = unsafe extern "thiscall" fn(usize, *const i8, u32);
        unsafe { std::mem::transmute::<usize, Set>(STRING_SET)(at, s.as_ptr(), 0) };
    }

    unsafe fn write<T>(at: usize, v: T) {
        unsafe { (at as *mut T).write_unaligned(v) };
    }

    /// The form with this id (`LookupFormByID`, cdecl), or 0.
    pub fn lookup(id: u32) -> usize {
        const LOOKUP_FORM: usize = 0x0046_B250;
        unsafe { std::mem::transmute::<usize, unsafe extern "C" fn(u32) -> usize>(LOOKUP_FORM)(id) }
    }

    /// Gives the player `spell` (the game's own `AddSpell`): the spell, or 0 if they still haven't it.
    pub fn give(player: usize, spell: usize) -> usize {
        let id = mem::at::<u32>(spell + FORM_ID).unwrap_or(0);
        let given = crate::console::run_script(&format!("player.addspell {id:08X}")) && find(player) == spell;
        tracing::info!("mirror spell: {NAME} ({id:08X}) {}", if given { "given to the player" } else { "not given: the game wouldn't" });
        if given { spell } else { 0 }
    }

    /// Makes the mirror spell (a created form a save keeps): the spell, or 0.
    pub fn make() -> usize {
        let script_effect = combat::effect_setting(combat::SCRIPT_EFFECT);
        if script_effect == 0 || combat::effect_setting(LOOK) == 0 {
            return 0;
        }
        type Create = unsafe extern "C" fn(u8) -> usize;
        type Alloc = unsafe extern "C" fn(usize) -> usize;
        type ItemNew = unsafe extern "thiscall" fn(usize, usize) -> usize;
        type Add = unsafe extern "thiscall" fn(usize, usize) -> usize;
        let spell = unsafe { std::mem::transmute::<usize, Create>(CREATE_FORM)(FORM_SPELL) };
        let id = mem::at::<u32>(spell + FORM_ID).unwrap_or(0);
        // (Only a created form is kept by a save, in its created objects.)
        if spell == 0 || id >> 24 != 0xFF {
            tracing::warn!("mirror spell: the game made no created spell ({spell:#x}, {id:08X})");
            return 0;
        }
        unsafe {
            set_string(spell + SPELL_NAME, NAME);
            write(spell + SPELL_TYPE, LESSER_POWER);
            write(spell + SPELL_COST, 0u32);
            write(spell + SPELL_LEVEL, 0u32);
            write(spell + SPELL_FLAGS, MANUAL_COST);
            let effect = std::mem::transmute::<usize, Alloc>(ALLOC)(EFFECT_ITEM_SIZE);
            if effect == 0 {
                return 0;
            }
            std::mem::transmute::<usize, ItemNew>(EFFECT_ITEM_NEW)(effect, script_effect);
            let info = mem::deref(effect + EFFECT_SCRIPT_INFO);
            if info == 0 {
                tracing::warn!("mirror spell: its script effect has no info");
                return 0;
            }
            write(info + SCRIPT_ID, 0u32);
            write(info + SCRIPT_SCHOOL, CONJURATION);
            write(info + SCRIPT_VISUAL, LOOK);
            write(info + SCRIPT_HOSTILE, 0u8);
            set_string(info + SCRIPT_NAME, NAME);
            std::mem::transmute::<usize, Add>(EFFECT_LIST_ADD)(spell + SPELL_EFFECTS, effect);
            std::mem::transmute::<usize, Add>(ADD_TO_DATA_HANDLER)(mem::deref(DATA_HANDLER), spell);
            std::mem::transmute::<usize, Add>(ADD_CREATED)(mem::deref(CREATED_OBJECTS), spell);
        }
        tracing::info!("mirror spell: made {NAME} ({id:08X}), a lesser power");
        spell
    }

    /// The arrival at the mirror's glass (`m`, tracking space) for its summons: the game's summoning
    /// effect at its foot, facing the player, and its moment.
    pub fn arrive(view: &View, m: &MirrorView, player: usize) -> Arrival {
        let v = |a: [f32; 4]| Vec3::new(a[0], a[1], a[2]);
        let (center, right, up) = (v(m.center), v(m.right), v(m.up));
        let basis = camera::basis();
        let foot = basis.point_to_game(view.point(center - up));
        let normal = up.cross(right).normalize_or_zero();
        let facing = (basis.dir_to_game(view.dir(normal)) * Vec3::new(1.0, 1.0, 0.0)).normalize_or_zero() * ARRIVAL_FACING;
        let mut arrival = Arrival { count: m.summons, left: 0.0, at: basis.point_to_game(view.point(center)).to_array() };
        let setting = combat::effect_setting(ARRIVAL);
        let model = combat::read_str(mem::deref(setting + SETTING_MODEL + 4));
        let cell = mem::deref(player + PARENT_CELL);
        if setting == 0 || model.is_empty() || cell == 0 {
            tracing::warn!("mirror spell: no arrival effect ({setting:#x}, {model:?}, cell {cell:#x}): the glass comes at once");
            return arrival;
        }
        let path = std::ffi::CString::new(model.clone()).unwrap_or_default();
        type Key = unsafe extern "C" fn(usize) -> usize;
        type Node = unsafe extern "thiscall" fn(usize, usize, u32) -> usize;
        type Alloc = unsafe extern "C" fn(usize) -> usize;
        type New = unsafe extern "thiscall" fn(usize, usize, f32, usize, *const i8, f32, f32, f32, f32, f32, f32, f32, u32) -> usize;
        type Play = unsafe extern "thiscall" fn(usize, *const u8);
        type AddTemp = unsafe extern "thiscall" fn(usize, usize);
        type KeyTime = unsafe extern "C" fn(usize, *const u8, *const u8) -> f64;
        unsafe {
            let key = std::mem::transmute::<usize, Key>(EFFECTS_KEY)(player);
            let node = std::mem::transmute::<usize, Node>(CELL_NODE)(cell, key, 3);
            let fx = std::mem::transmute::<usize, Alloc>(ALLOC)(TEMP_PARTICLE_SIZE);
            if fx == 0 {
                return arrival;
            }
            let fx = std::mem::transmute::<usize, New>(TEMP_PARTICLE)(fx, cell, 1.0, node, path.as_ptr(), facing.x, facing.y, facing.z, foot.x, foot.y, foot.z, 1.0, 0);
            std::mem::transmute::<usize, Play>(PLAY_SEQUENCE)(fx, ARRIVAL_SEQUENCE.as_ptr());
            std::mem::transmute::<usize, AddTemp>(ADD_TEMP_EFFECT)(PROCESS_MANAGER, fx);
            arrival.left = std::mem::transmute::<usize, KeyTime>(KEY_TIME)(mem::deref(fx + TEMP_NODE), ARRIVAL_SEQUENCE.as_ptr(), ARRIVAL_KEY.as_ptr()) as f32;
        }
        tracing::info!("mirror spell: the game's arrival {model:?} at {foot:.0?} for summons {}, its moment in {:.2} s", m.summons, arrival.left);
        arrival
    }

    /// The hit sound of the magic effect `setting` at `at` (game units).
    pub fn sound(setting: usize, at: [f32; 3]) {
        let sound = mem::deref(setting + SETTING_HIT_SOUND);
        let manager = mem::deref(mem::deref(engine::OS_GLOBALS) + SOUND_MANAGER);
        let Some(id) = mem::at::<u32>(sound + FORM_ID).filter(|_| setting != 0 && sound != 0 && manager != 0) else { return };
        type Make = unsafe extern "thiscall" fn(usize, u32, u32, u32) -> usize;
        type At = unsafe extern "thiscall" fn(usize, f32, f32, f32) -> i32;
        type Volume = unsafe extern "thiscall" fn(usize, f32) -> i32;
        type Start = unsafe extern "thiscall" fn(usize, u32) -> i32;
        type Release = unsafe extern "thiscall" fn(usize) -> i32;
        type Free = unsafe extern "C" fn(usize);
        unsafe {
            let h = std::mem::transmute::<usize, Make>(MAKE_SOUND)(manager, id, SOUND_3D, 1);
            if h == 0 {
                return;
            }
            std::mem::transmute::<usize, At>(SOUND_AT)(h, at[0], at[1], at[2]);
            std::mem::transmute::<usize, Volume>(SOUND_VOLUME)(h, 1.0);
            std::mem::transmute::<usize, Start>(SOUND_START)(h, 0);
            std::mem::transmute::<usize, Release>(SOUND_RELEASE)(h);
            std::mem::transmute::<usize, Free>(FREE)(h);
        }
    }
}

#[cfg(not(target_arch = "x86"))]
mod game {
    use super::*;

    pub fn lookup(_id: u32) -> usize {
        0
    }

    pub fn give(_player: usize, _spell: usize) -> usize {
        0
    }

    pub fn make() -> usize {
        0
    }

    pub fn arrive(_view: &rustvr_adapter::View, m: &rustvr_protocol::MirrorView, _player: usize) -> Arrival {
        Arrival { count: m.summons, left: 0.0, at: [0.0; 3] }
    }

    pub fn sound(_setting: usize, _at: [f32; 3]) {}
}
