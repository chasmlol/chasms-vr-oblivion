//! Repairing with the hammer (`docs/research/interact.md`, "Repairing with the hammer"): a repair
//! hammer drawn from the body's slots is the game's own model in the hand, and its head struck on
//! something the player holds or wears repairs it a strike at a time: the weapon in the other hand,
//! the shield on the arm, the armour worn where it's worn (the helmet at the head, the cuirass on the
//! chest, the greaves on the legs, the boots at the feet, the gauntlets on the other hand).
//!
//! Each strike is one click of the game's own `RepairMenu`, in its hammer mode, on the thing struck:
//! the menu made for that moment and gone again in the same frame (as alchemy's menu is), so the
//! world never stops. Everything else is the game's: what a strike restores (the Armorer skill), the
//! cap (full health, or a quarter past it for an Expert), the journeyman rule for enchanted things,
//! the hammer breaking and being used up, the skill's use, its sounds and messages, and no repairing
//! in combat (its own words) unless its setting allows it.
//!
//! A strike is the hammer's head arriving at the thing at [`STRIKE_SPEED`] or more (its fastest in
//! the last [`PEAK_TIME`]: a held blade stops the hammer, so it's slow by the time it's there); the
//! next needs the head lifted [`LIFT`] off it again.

use glam::{Quat, Vec3};

/// How fast (m/s) the head must have been coming to strike.
pub const STRIKE_SPEED: f32 = 1.2;
/// How far back (seconds) its fastest is looked for.
pub const PEAK_TIME: f64 = 0.12;
/// The head within this (metres) of a thing is on it.
pub const REACH: f32 = 0.03;
/// How far off the thing the head is lifted before it can strike again.
pub const LIFT: f32 = 0.08;

/// The head of a hammer held in a hand (`pos`, `rot`: its model's pose; `center`, `half`: its
/// model's box in its own space; `grip`: the hand's grip): a line across the far end of its box from
/// the hand (its longest side's), along its next longest side, a little in from the end (a hammer's
/// head is a block across its handle's end).
pub fn head_of(pos: Vec3, rot: Quat, center: Vec3, half: Vec3, grip: Vec3) -> (Vec3, Vec3) {
    let axes = [Vec3::X * half.x, Vec3::Y * half.y, Vec3::Z * half.z];
    let mut order = [0usize, 1, 2];
    order.sort_by(|&a, &b| axes[b].length().total_cmp(&axes[a].length()));
    let (long, across) = (axes[order[0]], axes[order[1]]);
    // (Its head's own thickness: as deep as its next longest side's third, at most a sixth of its length.)
    let inset = (across.length() * 0.33).min(long.length() / 3.0);
    let end = |s: f32| pos + rot * (center + long * s - long.normalize_or_zero() * inset * s);
    let far = if end(1.0).distance(grip) >= end(-1.0).distance(grip) { 1.0 } else { -1.0 };
    let mid = end(far);
    let side = rot * across;
    (mid - side, mid + side)
}

/// The point of the line `a`..`b` nearest `p`.
pub fn nearest_on_line(p: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = b - a;
    let t = if ab.length_squared() > 1e-12 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
    a + ab * t
}

/// How far the line `a`..`b` comes from the line `p`..`q` (sampled along the first: a hammer's head
/// is short, so a few points find it).
pub fn line_to_line(a: Vec3, b: Vec3, p: Vec3, q: Vec3) -> f32 {
    (0..=8).map(|k| a.lerp(b, k as f32 / 8.0)).map(|s| s.distance(nearest_on_line(s, p, q))).fold(f32::MAX, f32::min)
}

/// How far the line `a`..`b` comes from a box (`center`, `half`) in the space `rot` turns and `pos`
/// moves it to.
pub fn line_to_box(a: Vec3, b: Vec3, pos: Vec3, rot: Quat, center: Vec3, half: Vec3) -> f32 {
    let inv = rot.inverse();
    let local = |p: Vec3| inv * (p - pos) - center;
    let (a, b) = (local(a), local(b));
    (0..=8)
        .map(|k| a.lerp(b, k as f32 / 8.0))
        .map(|s| (s.abs() - half).max(Vec3::ZERO).length())
        .fold(f32::MAX, f32::min)
}

/// Armour's slots (`TESBipedModelForm` +4) and where on the body each is struck: between two of
/// the body's joints, so far round the line between them (metres). The hands' are the other hand's
/// (the hammer's own isn't struck by itself).
pub const HEAD: u16 = 0x1;
pub const UPPER_BODY: u16 = 0x4;
pub const LOWER_BODY: u16 = 0x8;
pub const HANDS: u16 = 0x10;
pub const FEET: u16 = 0x20;
pub const SHIELD: u16 = 0x2000;

use rustvr_protocol::Joint;

pub const WORN_AT: [(u16, Joint, Joint, f32); 12] = [
    (HEAD, Joint::Neck, Joint::Head, 0.12),
    (UPPER_BODY, Joint::Spine, Joint::UpperChest, 0.16),
    (UPPER_BODY, Joint::LeftShoulder, Joint::LeftUpperArm, 0.07),
    (UPPER_BODY, Joint::RightShoulder, Joint::RightUpperArm, 0.07),
    (LOWER_BODY, Joint::LeftUpperLeg, Joint::LeftLowerLeg, 0.09),
    (LOWER_BODY, Joint::RightUpperLeg, Joint::RightLowerLeg, 0.09),
    (LOWER_BODY, Joint::LeftLowerLeg, Joint::LeftFoot, 0.07),
    (LOWER_BODY, Joint::RightLowerLeg, Joint::RightFoot, 0.07),
    (HANDS, Joint::LeftLowerArm, Joint::LeftHand, 0.06),
    (HANDS, Joint::RightLowerArm, Joint::RightHand, 0.06),
    (FEET, Joint::LeftFoot, Joint::LeftToes, 0.07),
    (FEET, Joint::RightFoot, Joint::RightToes, 0.07),
];

/// Whether a place on the body (`WORN_AT`'s) is the hammer hand's own (`hand`: 0 left, 1 right): its
/// gauntlet isn't struck by the hammer it holds.
pub fn own_hand(joint: Joint, hand: usize) -> bool {
    matches!((joint, hand), (Joint::LeftLowerArm, 0) | (Joint::RightLowerArm, 1))
}

/// The game's repair hammer (`TESObjectMISC` 0x0000000C).
pub const HAMMER: u32 = 0x0000_000C;

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{engine, mem, tiles};
    use rustvr_protocol::*;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Mutex;

    /// `kMenuType_Repair`; the repair menu made (cdecl: its mode, 0, 0, 0; its `TileMenu`), the
    /// hammer's mode; its `HandleClick` (thiscall: id, tile) and the first id of its list's rows;
    /// the row's place in the player's inventory (value 0xFB9).
    const REPAIR: u32 = 1035;
    const OPEN: usize = 0x005D_2070;
    const HAMMER_MODE: u32 = 1;
    const HANDLE_CLICK: usize = 0x005D_22C0;
    const FIRST_ROW: u32 = 51;
    const ROW_INDEX: u16 = 0xFB9;
    /// The player's inventory entry at a place, a copy (thiscall: the player, the place, 0); an
    /// entry's health in percent (thiscall: it, 1); and its end (thiscall: it) before the game's free.
    const ENTRY_AT: usize = 0x004D_88F0;
    const HEALTH: usize = 0x0048_4850;
    const ENTRY_END: usize = 0x0048_4470;
    /// The game's free (cdecl).
    const FREE: usize = 0x0040_1F20;
    /// In combat (thiscall: the player, 0); `iAllowRepairDuringCombat`'s value; "You cannot repair
    /// items during combat!" (`sCanNotEquipRepairHammer`); a notice (cdecl: text, 0, 1, seconds).
    const IN_COMBAT: usize = 0x0066_05A0;
    const REPAIR_IN_COMBAT: usize = 0x00B3_8A90;
    const NO_REPAIR_IN_COMBAT: usize = 0x00B3_8A78;
    const MESSAGE: usize = 0x0057_ACC0;
    /// The player's high process (+0x58): the equipped weapon's entry (+0xE4) and the shield's
    /// (+0xF0); an entry's extra data lists (+0), form (+8). Forms: type (+4), id (+0xC); an
    /// armour's slots (+0x64 +4).
    const PROCESS: usize = 0x58;
    const EQUIPPED_WEAPON: usize = 0xE4;
    const EQUIPPED_SHIELD: usize = 0xF0;
    const FORM_TYPE: usize = 0x4;
    const FORM_ID: usize = 0xC;
    const ARMOR: u8 = 0x14;
    const ARMOR_SLOTS: usize = 0x64 + 4;
    /// The player's `ExtraContainerChanges` (in its extra data, +0x44) and an extra data list's
    /// extras: worn, worn on the left.
    const EXTRA_CONTAINER_CHANGES: u32 = 0x15;
    const EXTRA_WORN: u32 = 0x1B;
    const EXTRA_WORN_LEFT: u32 = 0x1C;
    const GET_EXTRA: usize = 0x0041_E210;

    /// What a strike is on: a thing of the player's (its form, and the extra data list that's the
    /// one of it: the worn one, or the one in the hand), for its card.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Thing {
        form: usize,
        list: usize,
        hand: Option<usize>,
    }

    #[derive(Default)]
    struct State {
        /// The head's middle lately (when, where).
        trail: VecDeque<(f64, Vec3)>,
        /// Struck and not lifted off since: the thing, where the head was.
        on: Option<(usize, Vec3)>,
        strikes: Vec<(Thing, usize)>,
        /// The weapons' and shields' boxes by model (middle, half extents; the protocol's axes).
        boxes: HashMap<String, Option<(Vec3, Vec3)>>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn extra(list: usize, kind: u32) -> usize {
        type Get = unsafe extern "thiscall" fn(usize, u32) -> usize;
        if list == 0 {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, Get>(GET_EXTRA)(list, kind) }
    }

    /// A game list's items (`{data, next}` nodes from its first, inline).
    fn list(first: usize, max: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = first;
        while node != 0 && out.len() < max {
            let data = mem::deref(node);
            if data != 0 {
                out.push(data);
            }
            node = mem::deref(node + 4);
        }
        out
    }

    /// An inventory entry's extra data lists.
    pub fn lists_of(entry: usize) -> Vec<usize> {
        list(mem::deref(entry), 64)
    }

    /// An entry's extra data list that's worn, else its first.
    pub fn worn_list_of(entry: usize) -> usize {
        let lists = lists_of(entry);
        lists.iter().copied().find(|&l| extra(l, EXTRA_WORN) != 0 || extra(l, EXTRA_WORN_LEFT) != 0).or(lists.first().copied()).unwrap_or(0)
    }

    /// The armour the player wears: its form, its worn extra data list and the slots it covers.
    fn worn_armour() -> Vec<(usize, usize, u16)> {
        let changes = extra(engine::player() + 0x44, EXTRA_CONTAINER_CHANGES);
        let data = mem::deref(changes + 0x0C);
        let mut out = Vec::new();
        for entry in list(mem::deref(data), 512) {
            let form = mem::deref(entry + 8);
            if mem::at::<u8>(form + FORM_TYPE) != Some(ARMOR) {
                continue;
            }
            let worn = list(mem::deref(entry), 64).into_iter().find(|&l| extra(l, EXTRA_WORN) != 0 || extra(l, EXTRA_WORN_LEFT) != 0);
            if let Some(l) = worn {
                out.push((form, l, mem::at::<u16>(form + ARMOR_SLOTS).unwrap_or(0)));
            }
        }
        out
    }

    /// The equipped weapon's or shield's thing.
    fn equipped(at: usize) -> Option<Thing> {
        let entry = mem::deref(mem::deref(engine::player() + PROCESS) + at);
        let form = mem::deref(entry + 8);
        (entry != 0 && form != 0).then(|| Thing { form, list: worn_list_of(entry), hand: None })
    }

    /// A model's box (its middle and half extents, the protocol's axes), as the runtime draws it.
    fn model_box(boxes: &mut HashMap<String, Option<(Vec3, Vec3)>>, path: &str) -> Option<(Vec3, Vec3)> {
        let key = oblivion_assets::export::model_key(path);
        *boxes.entry(key.clone()).or_insert_with(|| match rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key) {
            Ok(m) if !m.meshes.is_empty() => {
                let (lo, hi) = (Vec3::from_array(m.bounds_min), Vec3::from_array(m.bounds_max));
                Some(((lo + hi) * 0.5, (hi - lo) * 0.5))
            }
            _ => None,
        })
    }

    /// Before the eyes are drawn: the hammer's head, and whether it has just struck something the
    /// player holds or wears (the game's strike asked for).
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        let hammer = crate::inventory::held_copies().into_iter().find(|c| mem::at::<u32>(c.form + FORM_ID) == Some(HAMMER));
        with_state(|s| {
            let Some(c) = hammer else {
                s.trail.clear();
                s.on = None;
                return;
            };
            let (pos, rot) = if c.flags & OS_TRACKING != 0 { view.pose(&c.pose) } else { (Vec3::from_array(c.pose.pos), Quat::from_array(c.pose.rot)) };
            let grip = view.point(Vec3::from_array(f.hands[c.hand].grip.pos));
            let (a, b) = head_of(pos, rot, c.center, c.half, grip);
            let mid = (a + b) * 0.5;
            s.trail.push_back((now, mid));
            while s.trail.front().is_some_and(|t| now - t.0 > PEAK_TIME) {
                s.trail.pop_front();
            }
            let speed = s.trail.iter().zip(s.trail.iter().skip(1)).filter(|(p, q)| q.0 > p.0).map(|(p, q)| p.1.distance(q.1) / (q.0 - p.0) as f32).fold(0.0, f32::max);
            if let Some((_, at)) = s.on {
                if at.distance(mid) > LIFT {
                    s.on = None;
                }
                return;
            }
            let Some((thing, near, what)) = struck(s, view, a, b, c.hand) else { return };
            if speed < STRIKE_SPEED {
                return;
            }
            tracing::info!("repair: the hammer struck {what} ({:08X}) at {speed:.1} m/s, {near:.3} m from it", mem::at::<u32>(thing.form + FORM_ID).unwrap_or(0));
            s.on = Some((thing.form, mid));
            s.strikes.push((thing, c.hand));
        });
    }

    /// What the head (`a`..`b`) is on, of what the player holds or wears: the thing, how far, what.
    fn struck(s: &mut State, view: &rustvr_adapter::View, a: Vec3, b: Vec3, hand: usize) -> Option<(Thing, f32, &'static str)> {
        let mut best: Option<(Thing, f32, &'static str)> = None;
        let mut consider = |t: Thing, d: f32, what: &'static str| {
            if d <= REACH && best.is_none_or(|b| d < b.1) {
                best = Some((t, d, what));
            }
        };
        // The weapon and the shield, in a hand (the other one).
        let mut states: Box<ObjectStates> = bytemuck::zeroed_box();
        if crate::link::with_adapter(|ad| ad.read_object_states(&mut states)).unwrap_or(false) {
            let n = (states.count as usize).min(MAX_OBJECTS);
            for st in &states.states[..n] {
                let (at, path, what) = match st.id {
                    crate::weapons::WEAPON_ID => (EQUIPPED_WEAPON, crate::weapons::equipped().map(|e| e.model), "the weapon"),
                    crate::weapons::SHIELD_ID => (EQUIPPED_SHIELD, crate::weapons::equipped_shield().map(|e| e.1), "the shield"),
                    _ => continue,
                };
                let held = st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT);
                let in_hand = if st.flags & OS_HELD_LEFT != 0 { 0 } else { 1 };
                if held == 0 || in_hand == hand {
                    continue;
                }
                let (Some(thing), Some((centre, half))) = (equipped(at), path.and_then(|p| model_box(&mut s.boxes, &p))) else { continue };
                let (pos, rot) = if st.flags & OS_TRACKING != 0 { view.pose(&st.pose) } else { (Vec3::from_array(st.pose.pos), Quat::from_array(st.pose.rot)) };
                consider(Thing { hand: Some(in_hand), ..thing }, line_to_box(a, b, pos, rot, centre, half), what);
            }
        }
        // The armour worn, where it's worn.
        if let Some(body) = crate::link::with_adapter(|ad| ad.read_body()).flatten() {
            let joint = |j: Joint| view.point(Vec3::from_array(body.joints[j as usize].pos));
            for (form, list, slots) in worn_armour() {
                for (slot, p, q, r) in WORN_AT {
                    if slots & slot == 0 || own_hand(p, hand) {
                        continue;
                    }
                    let what = match slot {
                        HEAD => "the helmet",
                        UPPER_BODY => "the cuirass",
                        LOWER_BODY => "the greaves",
                        HANDS => "the gauntlets",
                        _ => "the boots",
                    };
                    consider(Thing { form, list, hand: None }, (line_to_line(a, b, joint(p), joint(q)) - r).max(0.0), what);
                }
            }
        }
        best
    }

    /// The game's own cast between a form's classes (cdecl: the object, 0, from, to, 0), and the
    /// classes: `TESBoundObject`, `TESHealthForm` (its full health: vtable +0x10, thiscall); an
    /// extra data list's health set (thiscall: it, the health).
    const CAST: usize = 0x0098_32E6;
    const BOUND_OBJECT: usize = 0x00B0_3248;
    const HEALTH_FORM: usize = 0x00B0_5CF4;
    const SET_HEALTH: usize = 0x0041_ECB0;
    const WEAPON: u8 = 0x21;

    /// Tests (`Wear <percent>`): every weapon and armour the player has on, at `percent` of its
    /// full health (the game's own health on its worn extra data), to be repaired.
    pub fn wear(percent: f32) -> bool {
        type Cast = unsafe extern "C" fn(usize, u32, usize, usize, u32) -> usize;
        type Full = unsafe extern "thiscall" fn(usize) -> u32;
        type Set = unsafe extern "thiscall" fn(usize, f32);
        let changes = extra(engine::player() + 0x44, EXTRA_CONTAINER_CHANGES);
        let data = mem::deref(changes + 0x0C);
        let mut n = 0;
        for entry in list(mem::deref(data), 512) {
            let form = mem::deref(entry + 8);
            if !matches!(mem::at::<u8>(form + FORM_TYPE), Some(ARMOR | WEAPON)) {
                continue;
            }
            let health = unsafe { std::mem::transmute::<usize, Cast>(CAST)(form, 0, BOUND_OBJECT, HEALTH_FORM, 0) };
            if health == 0 {
                continue;
            }
            let full = unsafe { std::mem::transmute::<usize, Full>(mem::deref(mem::deref(health) + 0x10))(health) };
            for l in list(mem::deref(entry), 64).into_iter().filter(|&l| extra(l, EXTRA_WORN) != 0 || extra(l, EXTRA_WORN_LEFT) != 0) {
                unsafe { std::mem::transmute::<usize, Set>(SET_HEALTH)(l, full as f32 * percent / 100.0) };
                n += 1;
            }
        }
        tracing::info!("repair: {n} thing(s) worn at {percent}% of their health");
        n > 0
    }

    /// Each game frame: the strikes made since, each the game's own repair of the thing struck.
    pub fn frame() {
        let strikes = with_state(|s| std::mem::take(&mut s.strikes));
        for (thing, hand) in strikes {
            let felt = strike(thing);
            crate::link::with_adapter(|a| {
                a.haptic(hand as u32, 0.035, 90.0, if felt { 1.0 } else { 0.5 });
                if let Some(h) = thing.hand {
                    a.haptic(h as u32, 0.03, 60.0, 0.6);
                }
            });
        }
    }

    fn free(p: usize) {
        type Free = unsafe extern "C" fn(usize);
        if p != 0 {
            unsafe { std::mem::transmute::<usize, Free>(FREE)(p) };
        }
    }

    /// The rows of an open list of the player's things (the repair, recharge, enchanting lists): each
    /// one's tile and its place in the player's inventory (its value `index`).
    fn rows(tile: usize, index: u16, out: &mut Vec<(usize, i32)>, depth: u32) {
        if depth > 12 || out.len() > 400 {
            return;
        }
        for c in tiles::children(tile) {
            if let Some(i) = tiles::value(c, index) {
                out.push((c, i as i32));
            }
            rows(c, index, out, depth + 1);
        }
    }

    fn row_of(tile: usize, thing: Thing) -> Option<(usize, f32)> {
        row_for(tile, ROW_INDEX, thing.form, thing.list)
    }

    /// The row (its tile) of a list of the player's things under `tile` that's the thing of `form`
    /// with the extra data `list` (rows carry their place in the inventory as their value `index`),
    /// and its health now (percent; -1 for what has none).
    pub fn row_for(tile: usize, index: u16, form: usize, list: usize) -> Option<(usize, f32)> {
        type At = unsafe extern "thiscall" fn(usize, i32, u8) -> usize;
        type Health = unsafe extern "thiscall" fn(usize, u8) -> f64;
        type End = unsafe extern "thiscall" fn(usize);
        let player = engine::player();
        let mut found = Vec::new();
        rows(tile, index, &mut found, 0);
        let mut by_form = Vec::new();
        for (row, at) in found {
            let entry = unsafe { std::mem::transmute::<usize, At>(ENTRY_AT)(player, at, 0) };
            if entry == 0 {
                continue;
            }
            let same_form = mem::deref(entry + 8) == form;
            let same = same_form && lists_of(entry).contains(&list);
            let health = unsafe { std::mem::transmute::<usize, Health>(HEALTH)(entry, 1) } as f32;
            unsafe { std::mem::transmute::<usize, End>(ENTRY_END)(entry) };
            free(entry);
            if same {
                return Some((row, health));
            }
            if same_form {
                by_form.push((row, health));
            }
        }
        // (The one row of its kind, if its extra data isn't the one the row has.)
        (by_form.len() == 1).then(|| by_form[0])
    }

    /// One strike: the game's own repair of the thing, as its hammer's menu would click it. False
    /// when the game didn't (in combat, its own menu open, the thing not there to repair).
    fn strike(thing: Thing) -> bool {
        type InCombat = unsafe extern "thiscall" fn(usize, u8) -> u8;
        type Message = unsafe extern "C" fn(usize, u32, u32, f32);
        type Open = unsafe extern "C" fn(u32, u32, u32, u32) -> usize;
        type Click = unsafe extern "thiscall" fn(usize, u32, usize);
        let player = engine::player();
        if player == 0 {
            return false;
        }
        // No repairing in combat, unless the game's setting allows it: its own words.
        if unsafe { std::mem::transmute::<usize, InCombat>(IN_COMBAT)(player, 0) } != 0 && mem::at::<u32>(REPAIR_IN_COMBAT) == Some(0) {
            unsafe { std::mem::transmute::<usize, Message>(MESSAGE)(mem::deref(NO_REPAIR_IN_COMBAT), 0, 1, -1.0) };
            tracing::info!("repair: not in combat");
            return false;
        }
        // (The player's own repair menu open: it's theirs, and making another would end it.)
        if crate::menus::open_menu(REPAIR) != 0 {
            return false;
        }
        let open = || unsafe { std::mem::transmute::<usize, Open>(OPEN)(HAMMER_MODE, 0, 0, 0) };
        crate::menus::for_a_moment(REPAIR, open, |menu, tile| match row_of(tile, thing) {
            Some((row, before)) => {
                unsafe { std::mem::transmute::<usize, Click>(HANDLE_CLICK)(menu, FIRST_ROW, row) };
                let after = row_of(tile, thing).map(|r| r.1);
                tracing::info!("repair: struck, its health {before:.1}% -> {after:.1?}%");
                true
            }
            None => {
                tracing::info!("repair: {:08X} has nothing to repair (not in the game's list)", mem::at::<u32>(thing.form + FORM_ID).unwrap_or(0));
                false
            }
        })
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hammers_head_is_across_its_far_end() {
        // The game's repair hammer: its handle down -Z (0.5 m), its head across X at the top.
        let (center, half) = (Vec3::new(0.0, 0.0, -0.235), Vec3::new(0.11, 0.035, 0.27));
        let (a, b) = head_of(Vec3::ZERO, Quat::IDENTITY, center, half, Vec3::new(0.0, 0.0, -0.45));
        let mid = (a + b) * 0.5;
        assert!(mid.z > -0.02 && mid.z < 0.035, "near the top: {mid:?}");
        assert!((a - b).abs().abs_diff_eq(Vec3::new(0.22, 0.0, 0.0), 1e-5), "across X: {a:?} {b:?}");
    }

    #[test]
    fn lines_and_boxes() {
        let d = line_to_box(Vec3::new(-1.0, 0.5, 0.0), Vec3::new(1.0, 0.5, 0.0), Vec3::ZERO, Quat::IDENTITY, Vec3::ZERO, Vec3::new(0.2, 0.4, 0.2));
        assert!((d - 0.1).abs() < 1e-5, "{d}");
        let d = line_to_line(Vec3::new(0.0, 1.0, -1.0), Vec3::new(0.0, 1.0, 1.0), Vec3::ZERO, Vec3::new(0.0, 2.0, 0.0));
        assert!(d < 0.13, "crossing it: {d}");
        assert!(own_hand(Joint::RightLowerArm, 1) && !own_hand(Joint::LeftLowerArm, 1));
    }
}
