//! Soul gems by hand (`docs/research/interact.md`, "Soul gems and enchanting"):
//!
//! * **The soul flies to the gem.** When the game traps a soul (its Soul Trap effect on a creature
//!   that dies, 0x6A4DF0: the gem filled by the game's own rules, its "You have captured a soul!",
//!   its own art at the body), the soul is seen to go: the Soul Trap effect's own model, playing its
//!   `SpecialIdle_Projectile` art, flies from the body to a soul gem in the player's hand (else to
//!   their pack, at the small of the back) over [`FLIGHT`], and the hand feels it arrive.
//! * **Filled gems look filled.** Oblivion draws a gem the same filled or empty; a filled one in the
//!   hand wears the Soul Trap effect's own enchanting shimmer (its effect shader), brighter as a soul
//!   has just arrived (the shader's own fade in).
//! * **Recharging by touch.** A filled gem held to the enchanted weapon in the other hand for
//!   [`RECHARGE_HOLD`] (felt building) recharges it: the game's own recharge with that gem (its
//!   `RechargeMenu` for a soul gem, made for a moment, the weapon's row clicked): the soul's charge
//!   added up to the weapon's maximum, the gem used up (Azura's Star emptied), its sound; in combat
//!   only if the game's setting allows it.

use glam::Vec3;

/// How long a soul takes to fly to the gem (seconds).
pub const FLIGHT: f64 = 1.2;
/// How long a filled gem is held to a weapon to recharge it (seconds), and how near (metres).
pub const RECHARGE_HOLD: f64 = 0.6;
pub const TOUCH: f32 = 0.04;

/// Where a soul is `u` (0..1) of the way from `from` to `to`: an arc rising over the way by a third
/// of its length, eased in and out.
pub fn arc(from: Vec3, to: Vec3, u: f32) -> Vec3 {
    let u = u.clamp(0.0, 1.0);
    let t = u * u * (3.0 - 2.0 * u);
    let lift = Vec3::Y * from.distance(to) / 3.0 + Vec3::Y * 0.3;
    let mid = (from + to) * 0.5 + lift;
    from * (1.0 - t) * (1.0 - t) + mid * 2.0 * t * (1.0 - t) + to * t * t
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem, scene};
    use glam::Quat;
    use rustvr_protocol::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// `SoulTrapEffect`'s end on its target (thiscall: the active effect): the game's soul trap. The
    /// active effect's target actor (thiscall: it; 0x6A1D70), its effect item (+0xC: the effect's
    /// setting at +0x1C, whose model is its `TESModel` at +0x18, the path from vtable +0x14).
    const SOUL_TRAP_END: usize = 0x006A_4DF0;
    const TARGET_OF: usize = 0x006A_1D70;
    const EFFECT_ITEM: usize = 0xC;
    const ITEM_SETTING: usize = 0x1C;
    const SETTING_MODEL: usize = 0x18;
    /// The player's "Souls Trapped" (its count goes up as the game traps one).
    const SOULS_TRAPPED: usize = 0x680;
    /// The art that flies, and the Soul Trap effect (`MGEF` STRP) whose enchanting shimmer a filled
    /// gem wears (`EffectSetting` +0x7C, as [`crate::magic`] reads an enchantment's).
    const FLYING: &str = "SpecialIdle_Projectile";
    const SOUL_TRAP: u32 = 0x0000_189F;
    const ENCHANT_SHADER: usize = 0x7C;
    const LOOKUP_FORM: usize = 0x0046_B250;
    /// `TESSoulGem`'s soul (+0x70); `ExtraSoul` (0x2F) and its soul (+0xC); `GetExtra` (thiscall).
    const GEM_SOUL: usize = 0x70;
    const EXTRA_SOUL: u32 = 0x2F;
    const SOUL: usize = 0xC;
    const GET_EXTRA: usize = 0x0041_E210;
    const FORM_TYPE: usize = 0x4;
    const FORM_ID: usize = 0xC;
    const SOUL_GEM: u8 = 0x26;
    const REFR_NODE: usize = 0x3C;
    /// The recharge menu (`kMenuType_Recharge`) made for a soul gem (cdecl: the gem's inventory
    /// entry, which it keeps: its `TileMenu`), and its `HandleClick` (thiscall: id, tile; its vtable's
    /// fourth); its rows (ids from 51) carry their place in the player's inventory (value 0xFB9).
    const RECHARGE: u32 = 1049;
    const OPEN_RECHARGE: usize = 0x005C_FB50;
    const RECHARGE_CLICK: usize = 0x005C_FD90;
    const FIRST_ROW: u32 = 51;
    const ROW_INDEX: u16 = 0xFB9;
    /// The player's inventory entry at a place, a copy (thiscall: the player, the place, 0), and its
    /// end before the game's free; an entry's extra data lists (+0) and form (+8).
    const ENTRY_AT: usize = 0x004D_88F0;
    const ENTRY_END: usize = 0x0048_4470;
    const FREE: usize = 0x0040_1F20;
    /// In combat (thiscall: the player, 0); `iAllowRechargeDuringCombat`'s value; "You cannot
    /// recharge enchanted items during combat!"; a notice (cdecl: text, 0, 1, seconds).
    const IN_COMBAT: usize = 0x0066_05A0;
    const RECHARGE_IN_COMBAT: usize = 0x00B3_8A88;
    const NO_RECHARGE_IN_COMBAT: usize = 0x00B3_8A70;
    const MESSAGE: usize = 0x0057_ACC0;
    /// The equipped weapon's entry (the player's high process +0x58, its +0xE4).
    const PROCESS: usize = 0x58;
    const EQUIPPED_WEAPON: usize = 0xE4;

    /// A soul on its way: from where, the art, its sequence and how far through it.
    struct Flight {
        from: Vec3,
        node: usize,
        seq: Option<scene::Sequence>,
        phase: f32,
        start: f64,
    }

    #[derive(Default)]
    struct State {
        /// Souls trapped this frame: the body's place (the protocol's world) and the art's model.
        trapped: Vec<(Vec3, String)>,
        flights: Vec<Flight>,
        /// When a soul last arrived in a hand's gem (its shimmer fades in from then).
        arrived: Option<f64>,
        /// A filled gem held to the weapon: since when (gem's object), and the recharge asked for.
        touching: Option<(u32, f64)>,
        next_tick: f64,
        recharge: Option<(usize, usize, usize)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    static REAL_END: AtomicUsize = AtomicUsize::new(0);

    pub fn install() {
        crate::hooks::install_into(SOUL_TRAP_END as *const (), soul_trap_end as *const (), "SoulTrapEffect's end", &REAL_END);
    }

    /// The game's soul trap, and whether it trapped a soul (its count of them went up): where from,
    /// for the soul to fly.
    unsafe extern "thiscall" fn soul_trap_end(effect: usize) -> i32 {
        type End = unsafe extern "thiscall" fn(usize) -> i32;
        type Target = unsafe extern "thiscall" fn(usize) -> usize;
        type Path = unsafe extern "thiscall" fn(usize) -> usize;
        let real: End = unsafe { std::mem::transmute(REAL_END.load(Ordering::Acquire)) };
        let player = engine::player();
        let before = mem::at::<u32>(player + SOULS_TRAPPED);
        let target = crate::hooks::guard("soul trap's target", 0, || unsafe { std::mem::transmute::<usize, Target>(TARGET_OF)(effect) });
        let r = unsafe { real(effect) };
        crate::hooks::guard_void("soul trapped", || {
            if before.is_none() || mem::at::<u32>(player + SOULS_TRAPPED) <= before || target == 0 {
                return;
            }
            let setting = mem::deref(mem::deref(effect + EFFECT_ITEM) + ITEM_SETTING);
            let model = if setting != 0 && mem::deref(setting + SETTING_MODEL) != 0 {
                crate::combat::read_str(unsafe { std::mem::transmute::<usize, Path>(mem::deref(mem::deref(setting + SETTING_MODEL) + 0x14))(setting + SETTING_MODEL) })
            } else {
                String::new()
            };
            let node = mem::deref(target + REFR_NODE);
            let at = mem::at::<[f32; 4]>(node + 0x20).map(|b| camera::basis().point_from_game(Vec3::new(b[0], b[1], b[2])));
            tracing::info!("soul: a soul trapped from {:08X} at {at:.2?} (its art {model:?})", mem::at::<u32>(target + FORM_ID).unwrap_or(0));
            if let Some(at) = at {
                with_state(|s| s.trapped.push((at, model)));
            }
        });
        r
    }

    fn extra(list: usize, kind: u32) -> usize {
        type Get = unsafe extern "thiscall" fn(usize, u32) -> usize;
        if list == 0 {
            return 0;
        }
        unsafe { std::mem::transmute::<usize, Get>(GET_EXTRA)(list, kind) }
    }

    /// The soul in a gem of `form` with the extra data `list` (0: none): its form's own (a gem sold
    /// filled), or the one the game put in it (`ExtraSoul`).
    pub fn soul_of(form: usize, list: usize) -> u8 {
        let own = mem::at::<u8>(form + GEM_SOUL).unwrap_or(0);
        let x = extra(list, EXTRA_SOUL);
        if x != 0 { mem::at::<u8>(x + SOUL).unwrap_or(0).max(own) } else { own }
    }

    fn is_gem(form: usize) -> bool {
        mem::at::<u8>(form + FORM_TYPE) == Some(SOUL_GEM)
    }

    fn pose_of(view: &rustvr_adapter::View, flags: u32, p: &Pose) -> (Vec3, Quat) {
        if flags & OS_TRACKING != 0 { view.pose(p) } else { (Vec3::from_array(p.pos), Quat::from_array(p.rot)) }
    }

    /// Before the eyes are drawn: souls flying to the gem, the filled gems' shimmer, a gem held to
    /// the weapon.
    pub fn place(view: &rustvr_adapter::View, _f: &RtFrame, now: f64) {
        let copies = crate::inventory::held_copies();
        let gems: Vec<_> = copies.iter().filter(|c| is_gem(c.form)).collect();
        let filled: Vec<_> = gems.iter().filter(|c| soul_of(c.form, c.list) > 0).collect();
        let body = crate::link::with_adapter(|a| a.read_body()).flatten();
        // Where a soul goes: into a gem in a hand, else into the pack at the small of the back.
        let gem_at = gems.first().map(|c| (pose_of(view, c.flags, &c.pose).0 + pose_of(view, c.flags, &c.pose).1 * c.center, Some(c.hand)));
        let pack = body.map(|b| {
            let hips = view.point(Vec3::from_array(b.joints[Joint::Hips as usize].pos));
            let chest = view.point(Vec3::from_array(b.joints[Joint::Chest as usize].pos));
            let back = view.rot(Quat::from_array(b.joints[Joint::Hips as usize].rot)) * Vec3::NEG_Z;
            ((hips + chest) * 0.5 + back.with_y(0.0).normalize_or_zero() * 0.15, None)
        });
        let target = gem_at.or(pack);
        with_state(|s| {
            for (from, model) in std::mem::take(&mut s.trapped) {
                let node = if model.is_empty() { 0 } else { scene::load(&model) };
                let seq = if node != 0 { scene::play(node, FLYING) } else { None };
                tracing::info!("soul: flying from {from:.2?} to {:?} ({})", target.map(|t| t.0), if node != 0 { "the effect's art" } else { "no art" });
                s.flights.push(Flight { from, node, seq, phase: 0.0, start: now });
            }
            let dt = scene::frame_time().max(1.0 / 90.0);
            s.flights.retain_mut(|fl| {
                let u = ((now - fl.start) / FLIGHT) as f32;
                let Some((to, hand)) = target else { return false };
                if u >= 1.0 {
                    if fl.node != 0 {
                        scene::unload(fl.node);
                    }
                    if let Some(h) = hand {
                        crate::link::with_adapter(|a| a.haptic(h as u32, 0.25, 70.0, 0.7));
                        s.arrived = Some(now);
                    }
                    tracing::info!("soul: arrived");
                    return false;
                }
                let at = arc(fl.from, to, u);
                let ahead = arc(fl.from, to, (u + 0.02).min(1.0)) - at;
                if fl.node != 0 {
                    if let Some(q) = fl.seq.as_ref() {
                        q.cycle(&mut fl.phase, 0.0, q.length, dt);
                    }
                    scene::place(fl.node, at, scene::facing(ahead, Vec3::Y), 1.0);
                    scene::animate(fl.node);
                }
                true
            });
            // A filled gem in a hand wears the Soul Trap effect's shimmer.
            let shader = soul_trap_shader();
            let age = s.arrived.map(|t| (now - t) as f32).filter(|a| *a < 10.0);
            let shimmer: Vec<Overlay> = filled.iter().filter_map(|c| crate::magic::shader_overlay(shader, c.id, age)).collect();
            crate::magic::set_held_overlays(if shader != 0 { shimmer } else { Vec::new() });
            // A filled gem held to the enchanted weapon in the other hand: the recharge, once held.
            let weapon = weapon_in_hand(view);
            let weapon_hand = weapon.map_or(1, |w| w.0);
            let at_weapon = filled.iter().find(|c| {
                weapon.is_some_and(|(hand, pos, rot, centre, half)| {
                    let (p, r) = pose_of(view, c.flags, &c.pose);
                    hand != c.hand && crate::repair::line_to_box(p + r * c.center, p + r * c.center, pos, rot, centre, half) < TOUCH
                })
            });
            match at_weapon {
                Some(c) => {
                    let since = match s.touching {
                        Some((id, t)) if id == c.id => t,
                        _ => {
                            s.touching = Some((c.id, now));
                            now
                        }
                    };
                    let held = now - since;
                    if held < RECHARGE_HOLD && now >= s.next_tick {
                        s.next_tick = now + 0.08;
                        let k = (held / RECHARGE_HOLD) as f32;
                        crate::link::with_adapter(|a| {
                            a.haptic(c.hand as u32, 0.02, 120.0 + 120.0 * k, 0.2 + 0.5 * k);
                        });
                    }
                    if held >= RECHARGE_HOLD && s.recharge.is_none() {
                        tracing::info!("soul: a filled gem held to the weapon: recharging");
                        s.recharge = Some((c.form, c.list, weapon_hand));
                        s.touching = Some((c.id, f64::MAX / 2.0));
                    }
                }
                None => s.touching = None,
            }
        });
    }

    /// The weapon in a hand, if it's enchanted: the hand, its pose and its model's box.
    fn weapon_in_hand(view: &rustvr_adapter::View) -> Option<(usize, Vec3, Quat, Vec3, Vec3)> {
        let e = crate::weapons::equipped().filter(|e| e.enchantment != 0)?;
        let mut states: Box<ObjectStates> = bytemuck::zeroed_box();
        if !crate::link::with_adapter(|a| a.read_object_states(&mut states)).unwrap_or(false) {
            return None;
        }
        let n = (states.count as usize).min(MAX_OBJECTS);
        let st = states.states[..n].iter().find(|st| st.id == crate::weapons::WEAPON_ID && st.flags & (OS_HELD_LEFT | OS_HELD_RIGHT) != 0)?;
        let (pos, rot) = pose_of(view, st.flags, &st.pose);
        let key = oblivion_assets::export::model_key(&e.model);
        let m = rustvr_asset::load_model(&rustvr_asset::default_root().join("oblivion"), &key).ok()?;
        let (lo, hi) = (Vec3::from_array(m.bounds_min), Vec3::from_array(m.bounds_max));
        Some((if st.flags & OS_HELD_LEFT != 0 { 0 } else { 1 }, pos, rot, (lo + hi) * 0.5, (hi - lo) * 0.5))
    }

    /// The Soul Trap effect's enchanting shader (`TESEffectShader`), or 0.
    fn soul_trap_shader() -> usize {
        static SHADER: AtomicUsize = AtomicUsize::new(usize::MAX);
        let s = SHADER.load(Ordering::Relaxed);
        if s != usize::MAX {
            return s;
        }
        type Lookup = unsafe extern "C" fn(u32) -> usize;
        let effect = unsafe { std::mem::transmute::<usize, Lookup>(LOOKUP_FORM)(SOUL_TRAP) };
        let shader = if effect != 0 { mem::deref(effect + ENCHANT_SHADER) } else { 0 };
        tracing::info!("soul: Soul Trap's effect {effect:#x}, its enchanting shader {shader:#x}");
        SHADER.store(shader, Ordering::Relaxed);
        shader
    }

    fn free(p: usize) {
        type Free = unsafe extern "C" fn(usize);
        if p != 0 {
            unsafe { std::mem::transmute::<usize, Free>(FREE)(p) };
        }
    }

    /// The player's inventory entry (a copy, the caller's to keep or free) of a thing of `form` with
    /// the extra data `list` (0: its first of that form).
    fn entry_of(form: usize, list: usize) -> Option<usize> {
        type At = unsafe extern "thiscall" fn(usize, i32, u8) -> usize;
        type End = unsafe extern "thiscall" fn(usize);
        let player = engine::player();
        for i in 0..512 {
            let e = unsafe { std::mem::transmute::<usize, At>(ENTRY_AT)(player, i, 0) };
            if e == 0 {
                return None;
            }
            if mem::deref(e + 8) == form && (list == 0 || crate::repair::lists_of(e).contains(&list)) {
                return Some(e);
            }
            unsafe { std::mem::transmute::<usize, End>(ENTRY_END)(e) };
            free(e);
        }
        None
    }

    /// `TESEnchantableForm` (the game's cast to it from `TESBoundObject`: its maximum charge at +8, a
    /// u16) and an extra data list's charge set (thiscall: it, the charge).
    const CAST: usize = 0x0098_32E6;
    const BOUND_OBJECT: usize = 0x00B0_3248;
    const ENCHANTABLE: usize = 0x00B0_3718;
    const MAX_CHARGE: usize = 0x8;
    const SET_CHARGE: usize = 0x0041_EE90;

    /// Tests (`Discharge <percent>`): the equipped weapon's enchantment at `percent` of its charge
    /// (the game's own charge on its worn extra data), to be recharged.
    pub fn discharge(percent: f32) -> bool {
        type Cast = unsafe extern "C" fn(usize, u32, usize, usize, u32) -> usize;
        type Set = unsafe extern "thiscall" fn(usize, f32);
        let entry = mem::deref(mem::deref(engine::player() + PROCESS) + EQUIPPED_WEAPON);
        let form = mem::deref(entry + 8);
        let list = crate::repair::worn_list_of(entry);
        let ench = if form != 0 { unsafe { std::mem::transmute::<usize, Cast>(CAST)(form, 0, BOUND_OBJECT, ENCHANTABLE, 0) } } else { 0 };
        let Some(max) = (ench != 0 && list != 0).then(|| mem::at::<u16>(ench + MAX_CHARGE)).flatten() else { return false };
        unsafe { std::mem::transmute::<usize, Set>(SET_CHARGE)(list, max as f32 * percent / 100.0) };
        tracing::info!("soul: the weapon's charge at {percent}% of {max}");
        true
    }

    /// Each game frame: a recharge asked for, the game's own with that gem.
    pub fn frame() {
        let Some((gem, list, hand)) = with_state(|s| s.recharge.take()) else { return };
        type InCombat = unsafe extern "thiscall" fn(usize, u8) -> u8;
        type Message = unsafe extern "C" fn(usize, u32, u32, f32);
        type Open = unsafe extern "C" fn(usize) -> usize;
        type Click = unsafe extern "thiscall" fn(usize, u32, usize);
        let player = engine::player();
        if player == 0 || crate::menus::open_menu(RECHARGE) != 0 {
            return;
        }
        if unsafe { std::mem::transmute::<usize, InCombat>(IN_COMBAT)(player, 0) } != 0 && mem::at::<u32>(RECHARGE_IN_COMBAT) == Some(0) {
            unsafe { std::mem::transmute::<usize, Message>(MESSAGE)(mem::deref(NO_RECHARGE_IN_COMBAT), 0, 1, -1.0) };
            return;
        }
        let weapon = mem::deref(mem::deref(player + PROCESS) + EQUIPPED_WEAPON);
        let weapon_form = mem::deref(weapon + 8);
        let weapon_list = crate::repair::worn_list_of(weapon);
        // (The gem's entry is the menu's: it keeps it.)
        let Some(entry) = entry_of(gem, list) else {
            tracing::info!("soul: the gem isn't in the inventory");
            return;
        };
        let open = || unsafe { std::mem::transmute::<usize, Open>(OPEN_RECHARGE)(entry) };
        let done = crate::menus::for_a_moment(RECHARGE, open, |menu, tile| {
            let row = crate::repair::row_for(tile, ROW_INDEX, weapon_form, weapon_list);
            match row {
                Some((row, _)) => {
                    unsafe { std::mem::transmute::<usize, Click>(RECHARGE_CLICK)(menu, FIRST_ROW, row) };
                    true
                }
                None => false,
            }
        });
        tracing::info!("soul: the weapon recharged: {done:?}");
        crate::link::with_adapter(|a| a.haptic(hand as u32, 0.3, 60.0, if done == Some(true) { 0.9 } else { 0.3 }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_soul_arcs_from_the_body_to_the_gem() {
        let (from, to) = (Vec3::new(0.0, 0.5, 6.0), Vec3::new(0.0, 1.2, 0.3));
        assert!(arc(from, to, 0.0).abs_diff_eq(from, 1e-5));
        assert!(arc(from, to, 1.0).abs_diff_eq(to, 1e-5));
        let mid = arc(from, to, 0.5);
        assert!(mid.y > from.y.max(to.y), "it rises on the way: {mid:?}");
    }
}
