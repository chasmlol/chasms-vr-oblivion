//! Oblivion's say in the framework's melee (`rustvr_protocol::melee`, the
//! framework's `docs/MELEE.md`): the player's rules, and what the runtime's melee asks of the game.
//!
//! * **The rules** ([`publish`], with the creatures each frame, sent when they change): the hands as
//!   strong as the game's Strength makes the player (50 is the runtime's own, as for grabbing
//!   people), the swings' stamina the game's own Fatigue (`RULES_STAMINA`: the player chose the
//!   stamina meter), climbing by the hands on (the player chose it), and slowing time off (the player
//!   didn't). Telekinesis stays off: the gravity gloves pull things. The buttons stay the game's (no
//!   kick on the movement stick, which is the game's own).
//! * **What the swings spent** ([`apply`], on the game's thread before its frame): the runtime's
//!   running total of the Fatigue the swings spent; what it went up by comes off the player's own
//!   Fatigue (`DamageActorValue`), so a tired player's arms weaken (the runtime's side), and the
//!   game's own Fatigue does what it does (its damage, its power attacks, falling over at none).
//! * **Climbing** ([`climb`]): while the hands hold the world and climb (`RTM_CLIMBING`), the step the
//!   game's `Actor::Move` makes is the arms' (`crate::movement`), as swimming's is the strokes'.
//!
//! Engine facts (`Oblivion.exe` 1.2.0.416): `Actor` vtable +0x284 `GetActorValue`, +0x2A4
//! `DamageActorValue` (thiscall: the value's code, minus the amount, who did it); Strength is actor
//! value 0, Fatigue 10.

use rustvr_protocol::*;
use std::sync::Mutex;

const AV_STRENGTH: u32 = 0;
const AV_FATIGUE: u32 = 10;
/// The game's Strength the runtime's own hand strength is (an average person's, as for grabbing).
const STRENGTH_ONE: f32 = 50.0;

struct State {
    /// The rules as last sent.
    sent: Option<MeleeRules>,
    /// The runtime's running total of the stamina spent, as last taken off.
    spent: f32,
    /// The arms' climbing now: how they move the player (the game's units a second, the game's axes).
    climb: Option<[f32; 3]>,
}

static STATE: Mutex<State> = Mutex::new(State { sent: None, spent: f32::NAN, climb: None });

/// The player's rules now, all of the game's say in one table (the adapter's only writer of
/// [`MeleeRules`]: anything else of the game's that has a say, telekinesis say, fills its part here).
pub fn rules() -> Option<MeleeRules> {
    let player = crate::engine::player();
    if player == 0 {
        return None;
    }
    let mut r: MeleeRules = bytemuck::Zeroable::zeroed();
    r.flags = RULES_STAMINA | RULES_SAYS_SLOW | RULES_SAYS_CLIMB | RULES_CLIMB | RULES_SAYS_TELEKINESIS;
    r.strength = (crate::combat::actor_value(player, AV_STRENGTH) as f32 / STRENGTH_ONE).clamp(0.2, 3.0);
    r.stamina = crate::combat::actor_value(player, AV_FATIGUE) as f32;
    r.stamina_max = crate::combat::base_actor_value(player, AV_FATIGUE).max(1) as f32;
    Some(r)
}

/// Once a game frame (in the world): the player's rules, when any of them changes.
pub fn publish(a: &rustvr_adapter::Adapter) {
    let Some(r) = rules() else { return };
    let mut s = STATE.lock().unwrap();
    if s.sent.is_some_and(|was| bytemuck::bytes_of(&was) == bytemuck::bytes_of(&r)) {
        return;
    }
    a.write_melee_rules(&r);
    if s.sent.is_none_or(|was| was.flags != r.flags || was.strength != r.strength) {
        tracing::info!("melee: the rules: flags {:#x}, strength {:.2}, Fatigue {} of {}", r.flags, r.strength, r.stamina, r.stamina_max);
    }
    s.sent = Some(r);
}

/// Before the game's frame: the Fatigue the swings spent since the last, off the player's; and the
/// arms' climbing for the player's step.
pub fn apply() {
    let Some(rt) = crate::link::with_adapter(|a| {
        let mut rt: RtMelee = bytemuck::Zeroable::zeroed();
        a.read_rt_melee(&mut rt).then_some(rt)
    })
    .flatten() else {
        return;
    };
    let player = crate::engine::player();
    let mut s = STATE.lock().unwrap();
    // (The first total read is where it stands: nothing of it is new.)
    let spent = rt.stamina_spent - if s.spent.is_nan() { rt.stamina_spent } else { s.spent };
    s.spent = rt.stamina_spent;
    if player != 0 && spent > 0.0 && spent.is_finite() {
        damage_fatigue(player, spent);
    }
    let climbing = rt.flags & RTM_CLIMBING != 0;
    // (The world's units are the protocol's metres: the game's, as the strokes' are.)
    let climb = climbing.then(|| crate::camera::basis().point_to_game(glam::Vec3::from_array(rt.climb)).to_array());
    if climb.is_some() != s.climb.is_some() {
        tracing::info!("melee: {} climbing by the hands", if climbing { "now" } else { "no longer" });
    }
    s.climb = climb;
}

/// While the hands climb: how they move the player (the game's units a second, the game's axes).
pub fn climb() -> Option<[f32; 3]> {
    STATE.lock().unwrap().climb
}

#[cfg(target_arch = "x86")]
fn damage_fatigue(player: usize, amount: f32) {
    const VT_DAMAGE_AV: usize = 0x2A4;
    type F = unsafe extern "thiscall" fn(usize, u32, f32, usize);
    let f = crate::mem::deref(crate::mem::deref(player) + VT_DAMAGE_AV);
    if crate::engine::CODE.contains(&f) {
        unsafe { std::mem::transmute::<usize, F>(f)(player, AV_FATIGUE, -amount, 0) };
    }
}

#[cfg(not(target_arch = "x86"))]
fn damage_fatigue(_player: usize, _amount: f32) {}
