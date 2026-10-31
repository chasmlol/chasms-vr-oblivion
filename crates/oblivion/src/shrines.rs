//! Shrines and offerings by hand (`docs/research/interact.md`, "Shrines and offerings"):
//!
//! * **A Daedric shrine's offering laid on its altar.** A thing held to a Daedric shrine (laid on its
//!   altar: held there [`OFFER_HOLD`]) that the shrine's own script names (one of the forms its
//!   script refers to: Azura's glow dust, Hircine's bear pelt...) is the player's activation of the
//!   shrine, so its script decides as ever (the hour, the player's level, the offering in the pack);
//!   its own question ("Do you wish to offer...?") is answered yes by the laying itself, and its
//!   script takes the offering and the Daedra speaks. Anything else it says (not the proper
//!   offering, too weak a champion) is left for the player to read.
//! * **A wayshrine or a chapel's altar**, touched with an open hand ([`TOUCH_HOLD`]) or knelt
//!   before ([`KNEEL_HOLD`]), is the player's activation of it: its own script's blessing (the god's
//!   spell, the pilgrimage, the cure at the chapel altars, "Seek not the Dragon's blessing,
//!   sinner!").
//! * **A Doomstone or a rune stone**, a palm on it ([`TOUCH_HOLD`]), is the player's activation of
//!   it (`docs/research/interact.md`, "Doomstones and runestones"): its own script's words and
//!   question ("Do you choose to steer by the stars of the Atronach?", "The stone stirs under your
//!   hand..."), which the player answers: a Doomstone's power takes the place of the one they have.

/// How long things are held to give them (seconds).
pub const OFFER_HOLD: f64 = 0.5;
pub const TOUCH_HOLD: f64 = 0.6;
pub const KNEEL_HOLD: f64 = 1.0;
/// How near (metres) a hand or a thing is to touch a shrine.
pub const TOUCH: f32 = 0.05;
/// Kneeling: the head lower than this share of the player's standing eye height, within
/// [`KNEEL_REACH`] of the shrine (metres, along the floor) and facing it.
pub const KNEEL: f32 = 0.72;
pub const KNEEL_REACH: f32 = 1.6;

/// Whether the head at `head` (the protocol's world, Y up) facing `look` kneels before a shrine at
/// `shrine`, the head `height` over the floor and the player's standing eye height `standing`.
pub fn kneels_before(head: glam::Vec3, look: glam::Vec3, shrine: glam::Vec3, height: f32, standing: f32) -> bool {
    let to = (shrine - head).with_y(0.0);
    let look = look.with_y(0.0).normalize_or_zero();
    standing > 0.5 && height < standing * KNEEL && to.length() < KNEEL_REACH && to.normalize_or_zero().dot(look) > 0.5
}

/// What kind of shrine a model makes an activator (its path under `meshes\`, any case).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Daedric,
    Wayshrine,
    Altar,
    /// A Doomstone or a rune stone (the standing stones of the ruins outside).
    Stone,
}

pub fn kind_of(model: &str) -> Option<Kind> {
    let m = model.to_ascii_lowercase();
    if m.contains("daedricstatues\\daedricshrine") {
        Some(Kind::Daedric)
    } else if m.contains("wayshrine\\wayshrinealtar") {
        Some(Kind::Wayshrine)
    } else if m.contains("cathedral\\cathedralstenintaltar") {
        Some(Kind::Altar)
    } else if m.contains("ruinstonesexterior\\runestone") {
        Some(Kind::Stone)
    } else {
        None
    }
}

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::av;
    use crate::{camera, engine, mem};
    use glam::{Quat, Vec3};
    use rustvr_protocol::*;
    use std::sync::Mutex;

    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DISABLED: u32 = 0x800;
    const ACTIVATOR: u8 = 0x12;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    /// An activator's model (`TESModel` +0x30, its path +4) and script (`TESScriptableForm` +0x48,
    /// its script +4); a script's referred-to forms (`refList` +0x40: `{var, next}`, a variable's
    /// form +8).
    const ACTI_MODEL: usize = 0x34;
    const ACTI_SCRIPT: usize = 0x4C;
    const SCRIPT_REFS: usize = 0x40;
    /// Shrines within this of the player (metres) are looked at.
    const NEAR: f32 = 3.0;
    /// Seconds after a shrine's activation its question may come.
    const ASKS_WITHIN: f64 = 2.0;
    /// Seconds before the same shrine is given to again.
    const AGAIN: f64 = 3.0;

    struct Shrine {
        refr: usize,
        kind: Kind,
        at: Vec3,
        /// What its script names (the forms an offering may be).
        names: Vec<usize>,
        triangles: Vec<[Vec3; 3]>,
    }

    /// What's being given at a shrine (since when), and how.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Giving {
        /// The offering's form, and its reference if it was lying about (the game takes it into the
        /// pack first: the shrine's script asks for it there).
        Offering(usize, usize),
        Touch(usize),
        Kneel,
    }

    #[derive(Default)]
    struct State {
        shrines: Vec<Shrine>,
        next_scan: f64,
        giving: Option<(usize, Giving, f64)>,
        next_tick: f64,
        /// The last given to (not again until it's let be: the hand off it, standing up).
        given: Option<(usize, f64)>,
        activate: Option<(usize, bool)>,
        /// An offering's question to answer yes, until when; an offering lying about, to take into the
        /// pack before the shrine is activated.
        answer_until: Option<f64>,
        take: usize,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    /// The forms a script refers to.
    fn script_names(script: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = script + SCRIPT_REFS;
        while node != 0 && out.len() < 128 {
            let var = mem::deref(node);
            let form = if var != 0 { mem::deref(var + 8) } else { 0 };
            if form != 0 {
                out.push(form);
            }
            node = mem::deref(node + 4);
        }
        out
    }

    fn scan(s: &mut State) {
        let b = camera::basis();
        let Some(me) = mem::at::<[f32; 3]>(engine::player() + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { return };
        let mut found = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let base = if r != 0 { mem::deref(r + REFR_BASE) } else { 0 };
                if base == 0 || mem::at::<u8>(base + FORM_TYPE) != Some(ACTIVATOR) || mem::at::<u32>(r + FORM_FLAGS).is_none_or(|f| f & DISABLED != 0) || mem::deref(r + REFR_NODE) == 0 {
                    continue;
                }
                let Some(kind) = kind_of(&crate::combat::read_str(mem::deref(base + ACTI_MODEL))) else { continue };
                // (A standing stone that's only a stone, with no script, has nothing to answer.)
                if kind == Kind::Stone && mem::deref(base + ACTI_SCRIPT) == 0 {
                    continue;
                }
                let Some(at) = mem::at::<[f32; 3]>(r + engine::refr::POS).map(|p| b.point_from_game(Vec3::from(p))) else { continue };
                if at.distance(me) > NEAR {
                    continue;
                }
                let script = mem::deref(base + ACTI_SCRIPT);
                let names = if kind == Kind::Daedric && script != 0 { script_names(script) } else { Vec::new() };
                found.push(Shrine { refr: r, kind, at, names, triangles: crate::lock::triangles_of(r) });
            }
        }
        if found.len() != s.shrines.len() {
            tracing::info!("shrines: {} near: {:?}", found.len(), found.iter().map(|f| (f.kind, mem::at::<u32>(f.refr + FORM_ID).unwrap_or(0), f.names.len())).collect::<Vec<_>>());
        }
        s.shrines = found;
    }

    /// What the hands hold: each thing's form, its place in the world (a held copy's model's middle,
    /// a held reference's 3D), the hand, and the reference if it's one lying about (0: the
    /// player's already).
    fn held_things(view: &rustvr_adapter::View) -> Vec<(usize, Vec3, usize, usize)> {
        let mut out = Vec::new();
        for c in crate::inventory::held_copies() {
            let (p, r) = if c.flags & OS_TRACKING != 0 { view.pose(&c.pose) } else { (Vec3::from_array(c.pose.pos), Quat::from_array(c.pose.rot)) };
            out.push((c.form, p + r * c.center, c.hand, 0));
        }
        let b = camera::basis();
        for h in crate::items::held() {
            let node = mem::deref(h.refr + REFR_NODE);
            if let Some(t) = mem::at::<[f32; 3]>(node + av::WORLD_TRANSLATE) {
                out.push((mem::deref(h.refr + REFR_BASE), b.point_from_game(Vec3::from(t)), h.hand, h.refr));
            }
        }
        out
    }

    /// Before the eyes are drawn: an offering held to a Daedric shrine, an open hand on a wayshrine
    /// or an altar, or the player kneeling before one; held long enough, the shrine's activation.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        if crate::menus::menu_mode() {
            return;
        }
        let things = held_things(view);
        let body = crate::link::with_adapter(|a| a.read_body()).flatten();
        with_state(|s| {
            if now >= s.next_scan {
                s.next_scan = now + 0.5;
                scan(s);
            }
            let near = |tris: &[[Vec3; 3]], p: Vec3| crate::lock::nearest(tris, p).is_some_and(|n| n.2 < TOUCH);
            let head = view.head_pos;
            let look = view.head_rot * Vec3::Z;
            let mut now_giving: Option<(usize, Giving, usize)> = None;
            for sh in &s.shrines {
                match sh.kind {
                    Kind::Daedric => {
                        if let Some(&(form, _, hand, lying)) = things.iter().find(|(form, p, _, _)| sh.names.contains(form) && near(&sh.triangles, *p)) {
                            now_giving = Some((sh.refr, Giving::Offering(form, lying), hand));
                        }
                    }
                    Kind::Wayshrine | Kind::Altar | Kind::Stone => {
                        let empty = |i: usize| body.is_none_or(|b| b.hands[i].flags & HAND_HOLDING == 0);
                        if let Some(i) = (0..2).find(|&i| empty(i) && near(&sh.triangles, view.point(Vec3::from_array(f.hands[i].grip.pos)))) {
                            now_giving = Some((sh.refr, Giving::Touch(i), i));
                        } else if sh.kind != Kind::Stone && body.is_some_and(|b| kneels_before(head, look, sh.at, f.hmd.pos[1], b.eye_height)) {
                            now_giving = Some((sh.refr, Giving::Kneel, 2));
                        }
                    }
                }
                if now_giving.is_some() {
                    break;
                }
            }
            // (Given to already: not again until it's let be a moment.)
            if let Some((r, t)) = s.given {
                if now_giving.is_some_and(|g| g.0 == r) || now - t < AGAIN {
                    if now_giving.is_some_and(|g| g.0 == r) {
                        s.given = Some((r, now));
                    }
                    s.giving = None;
                    return;
                }
                s.given = None;
            }
            let Some((refr, how, hand)) = now_giving else {
                s.giving = None;
                return;
            };
            let since = match s.giving {
                Some((r, g, t)) if r == refr && g == how => t,
                _ => now,
            };
            s.giving = Some((refr, how, since));
            let hold = match how {
                Giving::Offering(..) => OFFER_HOLD,
                Giving::Touch(_) => TOUCH_HOLD,
                Giving::Kneel => KNEEL_HOLD,
            };
            let held = now - since;
            let hands: Vec<u32> = if hand < 2 { vec![hand as u32] } else { vec![0, 1] };
            if held < hold {
                if now >= s.next_tick {
                    s.next_tick = now + 0.1;
                    let k = (held / hold) as f32;
                    crate::link::with_adapter(|a| hands.iter().for_each(|&h| { a.haptic(h, 0.03, 70.0 + 80.0 * k, 0.1 + 0.4 * k); }));
                }
                return;
            }
            tracing::info!("shrines: {how:?} at {:08X}", mem::at::<u32>(refr + FORM_ID).unwrap_or(0));
            crate::link::with_adapter(|a| hands.iter().for_each(|&h| { a.haptic(h, 0.25, 55.0, 0.8); }));
            if let Giving::Offering(_, lying) = how {
                s.take = lying;
            }
            s.activate = Some((refr, matches!(how, Giving::Offering(..))));
            s.given = Some((refr, now));
            s.giving = None;
        });
    }

    /// Each game frame: a shrine given to (its own activation), and an offering's question answered
    /// yes by the laying.
    pub fn frame(now: f64) {
        let (answer, take) = with_state(|s| (s.answer_until, std::mem::take(&mut s.take)));
        if take != 0 {
            let ok = crate::items::pick_up(take);
            tracing::info!("shrines: the offering taken into the pack first ({ok})");
            return;
        }
        if let Some((refr, offering)) = with_state(|s| s.activate.take()) {
            let ok = crate::world::activate(refr);
            tracing::info!("shrines: the shrine's activation ({ok})");
            if offering {
                with_state(|s| s.answer_until = Some(now + ASKS_WITHIN));
            }
            return;
        }
        let Some(until) = answer else { return };
        if now > until {
            with_state(|s| s.answer_until = None);
            return;
        }
        let Some((menu, buttons)) = crate::menus::message_box() else { return };
        with_state(|s| s.answer_until = None);
        // A question (two buttons or more): its yes, the first. A message alone is the player's to read.
        if buttons >= 2 {
            tracing::info!("shrines: the offering's question answered yes by the laying");
            crate::menus::click_message(menu, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn shrines_by_their_models() {
        assert_eq!(kind_of("Architecture\\DaedricStatues\\DaedricShrineAzura01.NIF"), Some(Kind::Daedric));
        assert_eq!(kind_of("Architecture\\Wayshrine\\WayshrineAltar01.NIF"), Some(Kind::Wayshrine));
        assert_eq!(kind_of("Architecture\\Cathedral\\CathedralStenIntAltar02.NIF"), Some(Kind::Altar));
        assert_eq!(kind_of("Dungeons\\FortRuins\\Exterior\\RuinStonesExterior\\RuneStoneExt01Glyph02.NIF"), Some(Kind::Stone));
        assert_eq!(kind_of("Clutter\\LecternWorkstation1.NIF"), None);
    }

    #[test]
    fn kneeling_before_a_shrine() {
        let shrine = Vec3::new(0.0, 0.0, 1.0);
        let head = Vec3::new(0.0, 1.0, 0.0);
        assert!(kneels_before(head, Vec3::Z, shrine, 1.0, 1.6));
        assert!(!kneels_before(head, Vec3::Z, shrine, 1.5, 1.6), "standing");
        assert!(!kneels_before(head, Vec3::NEG_Z, shrine, 1.0, 1.6), "facing away");
    }
}
