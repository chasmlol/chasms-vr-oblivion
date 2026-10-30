//! Enchanting at the altar (`docs/research/interact.md`, "Soul gems and enchanting"): a filled soul
//! gem held to the Arcane University's Altar of Enchanting (its book rest is too steep for anything
//! to lie on: held there is laid there), with the weapon or the shield in the other hand held to it
//! too, or alone, for [`LAY_HOLD`], is the player's activation of the altar (its own script:
//! `ShowEnchantment`), with the gem, and the weapon or shield, chosen in its menu as its own lists
//! choose them (the `RepairMenu` in its enchanting modes, made for a moment, their rows clicked). The
//! rest is the game's menu, beside the altar: the effects, magnitudes, the name and Create (the
//! enchanted thing made, the gem used, the item taken), worked with the laser; with the gem alone,
//! its item list picks the thing to enchant (rings, clothes, armour carried).

/// How long the gem is held to the altar (seconds), and how near (metres).
pub const LAY_HOLD: f64 = 0.5;
pub const TOUCH: f32 = 0.05;

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn altar() -> usize {
    0
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem};
    use glam::{Quat, Vec3};
    use rustvr_protocol::*;
    use std::sync::Mutex;

    /// The altar's script (`EnchantingStationScript`, Oblivion.esm's 0x1DFD9: `ShowEnchantment` on
    /// the player's activation); an activator's script (`TESScriptableForm` +0x48, its script +4).
    const ALTAR_SCRIPT: u32 = 0x0001_DFD9;
    const ACTIVATOR: u8 = 0x12;
    const ACTI_SCRIPT: usize = 0x4C;
    /// `kMenuType_Enchantment`; the repair menu (cdecl: mode, 0, 0, 0) in its modes that choose the
    /// enchanting's item (4) and soul gem (5), its `HandleClick` (id, tile), its rows (from id 51,
    /// their place in the inventory as value 0xFB9).
    const ENCHANTMENT: u32 = 0x412;
    const REPAIR: u32 = 1035;
    const OPEN_LIST: usize = 0x005D_2070;
    const ITEM_MODE: u32 = 4;
    const GEM_MODE: u32 = 5;
    const LIST_CLICK: usize = 0x005D_22C0;
    const FIRST_ROW: u32 = 51;
    const ROW_INDEX: u16 = 0xFB9;
    /// The player's high process (+0x58): the equipped weapon's entry (+0xE4), the shield's (+0xF0).
    const PROCESS: usize = 0x58;
    const EQUIPPED_WEAPON: usize = 0xE4;
    const EQUIPPED_SHIELD: usize = 0xF0;
    const FORM_TYPE: usize = 0x4;
    const FORM_ID: usize = 0xC;
    const FORM_FLAGS: usize = 0x8;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    /// Altars within this of the player (metres) are looked at.
    const NEAR: f32 = 2.5;

    /// The things laid on an altar: the gem's form and extra data, the item's (none: chosen in the
    /// game's own list).
    #[derive(Clone, Copy, Debug)]
    struct Laid {
        altar: usize,
        gem: (usize, usize),
        item: Option<(usize, usize)>,
    }

    #[derive(Default)]
    struct State {
        /// Since when the gem has been on the altar (its object).
        on: Option<(u32, f64)>,
        next_tick: f64,
        /// Laid: the altar to activate (next game frame), then the choices to make in its menu.
        activate: Option<Laid>,
        choose: Option<(Laid, f64)>,
        /// The altar whose menu is open (what it floats beside).
        open: usize,
        triangles: Option<(usize, f64, Vec<[Vec3; 3]>)>,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    /// The altar whose enchanting is open (laid on), for its menu to float beside; 0: none.
    pub fn altar() -> usize {
        with_state(|s| if crate::menus::menu_open(ENCHANTMENT) { s.open } else { 0 })
    }

    /// The Altar of Enchanting nearest the player, within reach.
    fn near_altar() -> usize {
        let b = camera::basis();
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { return 0 };
        let mut best = (f32::MAX, 0usize);
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let base = if r != 0 { mem::deref(r + REFR_BASE) } else { 0 };
                if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(ACTIVATOR) || mem::at::<u32>(r + FORM_FLAGS).is_none_or(|f| f & DISABLED != 0) {
                    continue;
                }
                let script = mem::deref(base + ACTI_SCRIPT);
                if script == 0 || mem::at::<u32>(script + FORM_ID) != Some(ALTAR_SCRIPT) || mem::deref(r + REFR_NODE) == 0 {
                    continue;
                }
                let Some(p) = mem::at::<[f32; 3]>(r + engine::refr::POS) else { continue };
                let d = b.point_from_game(Vec3::from(p)).distance(me);
                if d < NEAR && d < best.0 {
                    best = (d, r);
                }
            }
        }
        best.1
    }

    /// An equipped weapon's or shield's form and extra data.
    fn equipped(at: usize) -> Option<(usize, usize)> {
        let entry = mem::deref(mem::deref(engine::player() + PROCESS) + at);
        let form = mem::deref(entry + 8);
        (entry != 0 && form != 0).then(|| (form, crate::repair::worn_list_of(entry)))
    }

    /// Before the eyes are drawn: a filled gem held to the altar (and the weapon or shield in the
    /// other hand with it), laid there once held long enough.
    pub fn place(view: &rustvr_adapter::View, now: f64) {
        if crate::menus::menu_mode() {
            return;
        }
        let altar = near_altar();
        with_state(|s| {
            if altar == 0 {
                s.on = None;
                s.triangles = None;
                return;
            }
            if s.triangles.as_ref().is_none_or(|t| t.0 != altar || now - t.1 > 2.0) {
                s.triangles = Some((altar, now, crate::lock::triangles_of(altar)));
            }
            let tris = &s.triangles.as_ref().unwrap().2;
            let at_altar = |p: Vec3| crate::lock::nearest(tris, p).is_some_and(|n| n.2 < TOUCH);
            let pose = |flags: u32, p: &Pose| if flags & OS_TRACKING != 0 { view.pose(p) } else { (Vec3::from_array(p.pos), Quat::from_array(p.rot)) };
            let gem = crate::inventory::held_copies().into_iter().find(|c| {
                mem::at::<u8>(c.form + FORM_TYPE) == Some(0x26) && crate::soul::soul_of(c.form, c.list) > 0 && {
                    let (p, r) = pose(c.flags, &c.pose);
                    at_altar(p + r * c.center)
                }
            });
            let Some(gem) = gem else {
                s.on = None;
                return;
            };
            let since = match s.on {
                Some((id, t)) if id == gem.id => t,
                _ => {
                    s.on = Some((gem.id, now));
                    now
                }
            };
            let held = now - since;
            if held < LAY_HOLD {
                if now >= s.next_tick {
                    s.next_tick = now + 0.08;
                    let k = (held / LAY_HOLD) as f32;
                    crate::link::with_adapter(|a| a.haptic(gem.hand as u32, 0.02, 110.0 + 100.0 * k, 0.2 + 0.4 * k));
                }
                return;
            }
            if s.activate.is_some() || s.choose.is_some() {
                return;
            }
            // The weapon or the shield in the other hand, held to the altar too.
            let item = other_hand_item(view, gem.hand, &at_altar);
            tracing::info!("enchant: a filled gem laid on the altar {:08X}, with {item:?}", mem::at::<u32>(altar + FORM_ID).unwrap_or(0));
            s.activate = Some(Laid { altar, gem: (gem.form, gem.list), item });
            s.on = Some((gem.id, f64::MAX / 2.0));
            crate::link::with_adapter(|a| a.haptic(gem.hand as u32, 0.15, 80.0, 0.8));
        });
    }

    /// The weapon or shield the other hand holds, if it's at the altar: its form and extra data.
    fn other_hand_item(view: &rustvr_adapter::View, gem_hand: usize, at_altar: &dyn Fn(Vec3) -> bool) -> Option<(usize, usize)> {
        let mut states: Box<ObjectStates> = bytemuck::zeroed_box();
        if !crate::link::with_adapter(|a| a.read_object_states(&mut states)).unwrap_or(false) {
            return None;
        }
        let n = (states.count as usize).min(MAX_OBJECTS);
        for st in &states.states[..n] {
            let which = match st.id {
                crate::weapons::WEAPON_ID => EQUIPPED_WEAPON,
                crate::weapons::SHIELD_ID => EQUIPPED_SHIELD,
                _ => continue,
            };
            let hand = if st.flags & OS_HELD_LEFT != 0 { 0 } else if st.flags & OS_HELD_RIGHT != 0 { 1 } else { continue };
            if hand == gem_hand {
                continue;
            }
            // (Anywhere along it: its middle, and its ends along the hand's grip's way.)
            let (p, r) = if st.flags & OS_TRACKING != 0 { view.pose(&st.pose) } else { (Vec3::from_array(st.pose.pos), Quat::from_array(st.pose.rot)) };
            let along = [-0.4f32, -0.2, 0.0, 0.2, 0.4, 0.6, 0.8];
            if along.iter().any(|&k| at_altar(p + r * (Vec3::Z * k))) || at_altar(p) {
                return equipped(which);
            }
        }
        None
    }

    /// Each game frame: the altar activated (its script opens its enchanting), then the gem and the
    /// item chosen in its menu, as its own lists choose them.
    pub fn frame(now: f64) {
        let (activate, choose) = with_state(|s| (s.activate.take(), s.choose));
        if let Some(l) = activate {
            let ok = crate::world::activate(l.altar);
            tracing::info!("enchant: the altar activated ({ok})");
            with_state(|s| {
                s.choose = Some((l, now));
                s.open = l.altar;
            });
            return;
        }
        let Some((l, since)) = choose else { return };
        if !crate::menus::menu_open(ENCHANTMENT) {
            // (Its script refused, or it hasn't opened yet: a moment's wait.)
            if now - since > 1.0 {
                tracing::info!("enchant: the altar's enchanting didn't open");
                with_state(|s| s.choose = None);
            }
            return;
        }
        with_state(|s| s.choose = None);
        let gem = choose_in(GEM_MODE, l.gem);
        let item = l.item.map(|i| choose_in(ITEM_MODE, i));
        tracing::info!("enchant: chosen in the altar's menu: the gem {gem}, the item {item:?}");
    }

    /// Chooses the thing of (form, extra data) in the enchanting menu's own list (`mode`: its item
    /// or its soul gem): the list made for a moment and its row clicked, as the player's click would.
    fn choose_in(mode: u32, (form, list): (usize, usize)) -> bool {
        type Open = unsafe extern "C" fn(u32, u32, u32, u32) -> usize;
        type Click = unsafe extern "thiscall" fn(usize, u32, usize);
        if crate::menus::open_menu(REPAIR) != 0 {
            return false;
        }
        let open = || unsafe { std::mem::transmute::<usize, Open>(OPEN_LIST)(mode, 0, 0, 0) };
        crate::menus::for_a_moment(REPAIR, open, |menu, tile| match crate::repair::row_for(tile, ROW_INDEX, form, list) {
            Some((row, _)) => {
                unsafe { std::mem::transmute::<usize, Click>(LIST_CLICK)(menu, FIRST_ROW, row) };
                true
            }
            None => false,
        })
        .unwrap_or(false)
    }
}
