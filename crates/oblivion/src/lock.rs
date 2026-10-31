//! Oblivion's lock, picked at the lock itself (`docs/research/interact.md`, "Lockpicking at the
//! lock"). A lockpick drawn from the body's slots is the game's own model in the hand; its tip
//! touching a locked door or container is the player's own `Activate` of it, so the game decides
//! (its key, its picks, no picking while a crime alert is on) and opens its lock (`LockPickMenu`)
//! where the tip touched: the keyhole. It's Oblivion's own lock throughout: its tumblers (as many as
//! the lock's level), their speeds from the level, the Security skill's windows, picks broken and
//! used up, the crime on an owned lock, auto-attempt, its sounds; the game decides everything, and
//! the pick only works its menu:
//!
//! * **Under a tumbler**: the tip moved across the lock, a tumbler every [`SPACING`] either side of
//!   the keyhole (the game's own pick slides under it, with its scrape), as the game's cursor would
//!   put it there.
//! * **Pushed up**: the tip moved up ([`FLICK`]) pushes the tumbler up, as the mouse moved up does
//!   on a flat screen (the game's lift, at its speed for the lock).
//! * **Set**: the pick hand's trigger while the tumbler hangs at the top, as the mouse's click (the
//!   game's window, from the Security skill).
//! * **Felt**: the tumbler riding up on the pick ticks in the hand, its reaching the top pulses, a
//!   set tumbler clicks, a broken pick jolts it.
//! * **Seen**: the menu's flat picture isn't shown. Its own tumbler models stand in a row at the
//!   keyhole, a little proud of the surface, drawn by the game in its world, each held where the
//!   menu has it in its `UP` animation. What the menu says (the lock's level, the picks left, the
//!   player's Security) floats beside them ([`crate::hud`]).
//! * **Pulled out**: the tip taken a hand's width from the keyhole shuts the lock, as B does.
//!
//! A lock opened another way (the trigger's Activate at the door, a door tugged by its latch) stands
//! where the eyes were on the thing, and the pick hand's grip is the tip.
//!
//! The menu's own layout isn't read: the game's cursor is moved along the row of tumblers until the
//! game puts its pick under the one the tip is at (`LockPickMenu` +0x168, the tumbler under the
//! cursor), and where each tumbler was found is kept for the next time. A is Auto Attempt (the
//! game's own click on it), B closes it (Menu Mode, as on any menu). The world stops while the lock
//! is picked, as the game has it (its menu only runs on top of the menu stack); the hands go on
//! under it (`HUDL_HANDS`).

use glam::{Mat3, Quat, Vec3};


/// How far apart (metres) the tip's places for the five tumblers are: the row of tumblers is as
/// wide as this times four, centred on the keyhole.
pub const SPACING: f32 = 0.025;
/// How fast (m/s) the tip moves up to push the pick, and the mouse's movement (counts a frame, up)
/// that is to the game (it lifts the pick past 3).
pub const FLICK: f32 = 0.3;
const LIFT: i32 = -12;
/// The trigger's pull that sets a tumbler.
const SET_PULL: f32 = 0.75;
/// The row of tumblers on the screen (0..1 from the top left), where the game's lock menu puts its
/// lock (centred).
pub const TUMBLERS: [f32; 2] = [0.48, 0.44];
/// How near (metres) the tip comes to a locked thing's surface to be at it.
pub const TOUCH: f32 = 0.02;
/// How far the tip goes from where it touched before touching again tries the lock again.
pub const REARM: f32 = 0.06;
/// How far the tip goes from the keyhole (metres) to have been pulled out, and for how long
/// (seconds).
pub const PULL_OUT: f32 = 0.12;
pub const PULL_OUT_TIME: f64 = 0.3;
/// How far in front of the surface the tumblers stand (metres), and how tall one is.
pub const PROUD: f32 = 0.012;
pub const TUMBLER_TALL: f32 = 0.08;
/// Locked doors and containers within this far of the tip (metres, their references' places) are
/// looked at.
pub const NEAR: f32 = 3.0;

/// The tumbler (0..4) the tip is under, `lateral` metres to the right of the keyhole.
pub fn tumbler(lateral: f32) -> usize {
    ((lateral / SPACING).round() + 2.0).clamp(0.0, 4.0) as usize
}

/// The point of the triangle `t` nearest `p`.
pub fn closest_on_triangle(p: Vec3, t: &[Vec3; 3]) -> Vec3 {
    // (Ericson, Real-Time Collision Detection, 5.1.5: by the region of the triangle `p` is in.)
    let [a, b, c] = *t;
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    a + ab * (vb * denom) + ac * (vc * denom)
}

/// Where a lock is on a surface, in the protocol's world (left-handed, Y up): its keyhole, and the
/// lock's right (along its row of tumblers, as the player facing it sees it), up and out (towards
/// the player).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyhole {
    pub at: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub out: Vec3,
}

impl Keyhole {
    /// The keyhole at `at` on a surface facing `out` (towards whoever's at it). Up is the world's up
    /// along the surface; on a surface lying flat (a lid's top), the way from `towards` (where the
    /// player is) across it.
    pub fn new(at: Vec3, out: Vec3, towards: Vec3) -> Keyhole {
        let out = out.normalize_or(Vec3::NEG_Z);
        let along = |d: Vec3| (d - out * d.dot(out)).normalize_or_zero();
        let mut up = along(Vec3::Y);
        if out.dot(Vec3::Y).abs() > 0.9 || up == Vec3::ZERO {
            up = along(at - towards);
        }
        if up == Vec3::ZERO {
            up = out.any_orthonormal_vector();
        }
        // (Left-handed: the player looking along -out has +X right, so right = out x up.)
        Keyhole { at, right: out.cross(up).normalize(), up, out }
    }

    /// A point's place in the lock: across it (right), up it, and out from it.
    pub fn local(&self, p: Vec3) -> Vec3 {
        let d = p - self.at;
        Vec3::new(d.dot(self.right), d.dot(self.up), d.dot(self.out))
    }

    /// The rotation that turns a model's own right, up and out (in the protocol's axes) to the
    /// lock's.
    pub fn turn(&self, right: Vec3, up: Vec3, out: Vec3) -> Quat {
        let w = Mat3::from_cols(self.right, self.up, self.out);
        let m = Mat3::from_cols(right, up, out);
        Quat::from_mat3(&(w * m.transpose())).normalize()
    }
}

/// The tip of a thing held in a hand: of the two ends of its model's box along its longest side
/// (`center`, `half`: the box in its model's space; `pos`, `rot`: its model's pose), the one
/// farther from the hand's grip `grip`.
pub fn tip_of(pos: Vec3, rot: Quat, center: Vec3, half: Vec3, grip: Vec3) -> Vec3 {
    let long = if half.x >= half.y && half.x >= half.z {
        Vec3::X * half.x
    } else if half.y >= half.z {
        Vec3::Y * half.y
    } else {
        Vec3::Z * half.z
    };
    let ends = [pos + rot * (center + long), pos + rot * (center - long)];
    if ends[0].distance(grip) >= ends[1].distance(grip) { ends[0] } else { ends[1] }
}

/// The lockpick (`TESObjectMISC` 0x0000000A) and the Skeleton Key (0x0000000B): what picks a lock.
pub const LOCKPICK: u32 = 0x0000_000A;
pub const SKELETON_KEY: u32 = 0x0000_000B;

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn keyhole() -> Option<Keyhole> {
    None
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::{camera, engine, mem, scene};
    use rustvr_protocol::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// `kMenuType_LockPick`; the menus by type (`g_TileMenuArray`'s data); a menu tile's `Menu`
    /// (+0x44); the lock menu's vtable.
    const LOCKPICK_MENU: u32 = 0x3F6;
    const TILE_MENUS: usize = 0x00B1_3974;
    const TILE_MENU: usize = 0x44;
    const LOCKPICK_VTABLE: usize = 0x00A6_C73C;
    /// `LockPickMenu`: the reference being picked (+0x38); the picks left (+0x3C); its five
    /// tumblers (40 bytes each from +0x7C): height (+0, the time into its `UP` animation), when it
    /// reached the top (+4, -1 not there), moving (+0x18), set (+0x19); the top's height (+0x78); the
    /// tumbler under the cursor (+0x168, -1 none).
    const MENU_REF: usize = 0x38;
    const PICKS: usize = 0x3C;
    const TUMBLER: usize = 0x7C;
    const TUMBLER_SIZE: usize = 40;
    const HEIGHT: usize = 0;
    const AT_TOP: usize = 4;
    const MOVING: usize = 0x18;
    const SET: usize = 0x19;
    const TOP: usize = 0x78;
    const UNDER_CURSOR: usize = 0x168;
    /// `LockPickMenu::HandleClick(tile id, tile)` (thiscall, its vtable's third), its Auto Attempt
    /// button's id and its Close button's.
    const HANDLE_CLICK: usize = 0x005B_0A00;
    const AUTO_ATTEMPT: u32 = 5;
    const CLOSE: u32 = 2;
    /// How far the cursor moves a frame (of the screen's width) while it looks for a tumbler.
    const STEP: f32 = 0.004;
    /// The tumbler's model (the menu's own, in the game's model cache) and the sequence its height
    /// is a time into.
    const TUMBLER_MODEL: &str = "menus\\lockpicking\\tumbler02.nif";
    const UP: &str = "UP";
    /// Forms: type (+4), id (+0xC), flags (+8: deleted, disabled); a reference's base (+0x1C), 3D
    /// (+0x3C), extra data (+0x44).
    const FORM_TYPE: usize = 0x4;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DELETED: u32 = 0x20;
    const DISABLED: u32 = 0x800;
    const REFR_BASE: usize = 0x1C;
    const REFR_NODE: usize = 0x3C;
    const CONTAINER: u8 = 0x17;
    const DOOR: u8 = 0x18;
    /// `NiAVObject`: collision object, world bound (+0x20: centre, radius); `NiNode`: children.
    const NODE_COLLISION: usize = 0xA8;
    const NODE_BOUND: usize = 0x20;
    const NODE_CHILDREN: usize = 0xB0;
    const NODE_COUNT: usize = 0xB6;
    /// `bhkNiCollisionObject` -> `bhkRigidBody` (+0x10) -> `hkRigidBody` (+8).
    const COLLISION_BODY: usize = 0x10;
    const HK_OBJ: usize = 0x8;
    /// Seconds a thing's triangles are kept before they're read again.
    const TRIANGLES_FOR: f64 = 1.0;

    /// The tip touched a locked thing: the game's Activate to make in the next game frame.
    #[derive(Clone, Copy, Debug)]
    struct Touch {
        refr: usize,
    }

    /// The tumbler models at the keyhole: each one's node and its `UP` sequence; how the model
    /// stands (its own right, up and out in the protocol's axes, and its middle at the bottom of its
    /// travel, in its own space, game units); its scale.
    struct Models {
        tumblers: Vec<(usize, scene::Sequence)>,
        right: Vec3,
        up: Vec3,
        out: Vec3,
        bottom: Vec3,
        scale: f32,
    }

    #[derive(Default)]
    struct State {
        /// The lock open, where its keyhole is (for which reference), and which hand picks it.
        open: Option<(usize, Keyhole)>,
        hand: usize,
        /// The keyhole the tip last touched (the reference, the place): the game's lock opened there
        /// is at it.
        touched: Option<(usize, Keyhole, f64)>,
        pending: Option<Touch>,
        /// Not to try again until the tip has gone from here.
        spent: Option<Vec3>,
        /// The tip in the world now, the pick hand, when; how fast it rises along the lock's up.
        tip: Option<(Vec3, usize, f64)>,
        rise: f32,
        /// Since when the tip has been out of the lock.
        out_since: Option<f64>,
        /// Each locked thing's triangles near the tip (the protocol's world), and when read.
        triangles: HashMap<usize, (f64, Vec<[Vec3; 3]>)>,
        models: Option<Models>,
        /// The cursor (the screen, 0..1), and where each tumbler was found under it.
        cursor: [f32; 2],
        found: [Option<f32>; 5],
        picks: i32,
        set: [bool; 5],
        top: [bool; 5],
        next_tick: f64,
        a: bool,
    }
    static STATE: Mutex<Option<State>> = Mutex::new(None);

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        f(STATE.lock().unwrap().get_or_insert_with(State::default))
    }

    fn menu() -> usize {
        let data = mem::deref(TILE_MENUS);
        let tile = if data != 0 { mem::deref(data + 4 * (LOCKPICK_MENU - 0x3E9) as usize) } else { 0 };
        let menu = if tile != 0 { mem::deref(tile + TILE_MENU) } else { 0 };
        (menu != 0 && mem::deref(menu) == LOCKPICK_VTABLE).then_some(menu).unwrap_or(0)
    }

    fn haptic(hand: u32, seconds: f32, freq: f32, amplitude: f32) {
        crate::link::with_adapter(|a| a.haptic(hand, seconds, freq, amplitude));
    }

    /// The lock's keyhole while its menu is open (where its tumblers stand), for what floats beside
    /// it.
    pub fn keyhole() -> Option<Keyhole> {
        with_state(|s| s.open.map(|(_, k)| k))
    }

    /// A reference that's a locked door or container, there and drawn.
    pub(crate) fn locked_thing(refr: usize) -> bool {
        let flags = mem::at::<u32>(refr + FORM_FLAGS).unwrap_or(DELETED);
        let base = mem::deref(refr + REFR_BASE);
        flags & (DELETED | DISABLED) == 0
            && mem::deref(refr + REFR_NODE) != 0
            && base != 0
            && matches!(mem::at::<u8>(base + FORM_TYPE), Some(CONTAINER | DOOR))
            && crate::world::locked(refr)
    }

    /// The Havok bodies of a reference's 3D (its root's and its children's, a few levels down).
    fn bodies(node: usize, depth: u32, out: &mut Vec<usize>) {
        if node == 0 || depth > 3 || out.len() >= 8 {
            return;
        }
        let co = mem::deref(node + NODE_COLLISION);
        let hk = if co != 0 { mem::deref(mem::deref(co + COLLISION_BODY) + HK_OBJ) } else { 0 };
        if hk != 0 && mem::is_readable(hk as *const u8, 0xC4) && !out.contains(&hk) {
            out.push(hk);
        }
        let data = mem::deref(node + NODE_CHILDREN);
        let n = mem::at::<u16>(node + NODE_COUNT).unwrap_or(0).min(64) as usize;
        for i in 0..n {
            bodies(mem::deref(data + i * 4), depth + 1, out);
        }
    }

    /// A reference's collision as triangles in the protocol's world.
    pub(crate) fn triangles_of(refr: usize) -> Vec<[Vec3; 3]> {
        let mut found = Vec::new();
        bodies(mem::deref(refr + REFR_NODE), 0, &mut found);
        let b = camera::basis();
        found.into_iter().flat_map(crate::collision::body_triangles).map(|t| t.map(|v| b.point_from_game(v))).collect()
    }

    /// Where on `tris` is nearest `p`: the point, the surface's way out (towards `p`), how far.
    pub(crate) fn nearest(tris: &[[Vec3; 3]], p: Vec3) -> Option<(Vec3, Vec3, f32)> {
        let mut best: Option<(Vec3, Vec3, f32)> = None;
        for t in tris {
            let q = closest_on_triangle(p, t);
            let d = q.distance(p);
            if best.is_some_and(|b| b.2 <= d) {
                continue;
            }
            let n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
            if n == Vec3::ZERO {
                continue;
            }
            best = Some((q, if n.dot(p - q) < 0.0 { -n } else { n }, d));
        }
        best
    }

    /// Where `from` along `dir` first meets `tris`: the point and the surface's way out (towards
    /// `from`).
    fn hit(tris: &[[Vec3; 3]], from: Vec3, dir: Vec3) -> Option<(Vec3, Vec3)> {
        let mut best: Option<(f32, Vec3)> = None;
        for t in tris {
            if let Some(d) = crate::collision::ray_triangle(from, dir, t)
                && best.is_none_or(|b| d < b.0)
            {
                let n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
                best = Some((d, if n.dot(dir) > 0.0 { -n } else { n }));
            }
        }
        best.map(|(d, n)| (from + dir * d, n))
    }

    /// The pick in a hand: a lockpick or the Skeleton Key drawn from the body's slots (its tip, and
    /// the hand); else, with the lock open, the hand that picks it (its grip is the tip).
    fn pick(view: &rustvr_adapter::View, f: &RtFrame, open: bool, hand: usize) -> Option<(Vec3, usize)> {
        for c in crate::inventory::held_copies() {
            let id = mem::at::<u32>(c.form + FORM_ID).unwrap_or(0);
            if id != LOCKPICK && id != SKELETON_KEY {
                continue;
            }
            let (pos, rot) = if c.flags & OS_TRACKING != 0 { view.pose(&c.pose) } else { (Vec3::from_array(c.pose.pos), Quat::from_array(c.pose.rot)) };
            let grip = view.point(Vec3::from_array(f.hands[c.hand].grip.pos));
            return Some((tip_of(pos, rot, c.center, c.half, grip), c.hand));
        }
        open.then(|| (view.point(Vec3::from_array(f.hands[hand].grip.pos)), hand))
    }

    /// Before the eyes are drawn: where the pick's tip is; out of the lock, whether it's at a locked
    /// thing (the game's Activate asked for); in it, its tumblers at the keyhole.
    pub fn place(view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        with_state(|s| {
            let menu = if crate::menus::menu_open(LOCKPICK_MENU) { menu() } else { 0 };
            if menu == 0 {
                if s.open.take().is_some() {
                    tracing::info!("lock: closed");
                    unload(s);
                }
                s.tip = None;
                s.rise = 0.0;
                if !crate::menus::menu_mode() {
                    touch(s, view, f, now);
                }
                return;
            }
            if s.open.is_none() {
                let refr = mem::deref(menu + MENU_REF);
                let keyhole = match s.touched.filter(|t| t.0 == refr && now - t.2 < 2.0) {
                    Some((_, k, _)) => k,
                    None => looked_at(view, refr),
                };
                // The hand with the pick in it picks; else the hand the laser is on.
                let hand = pick(view, f, false, 1).map_or(1, |(_, h)| h);
                s.open = Some((refr, keyhole));
                s.hand = hand;
                s.touched = None;
                s.out_since = None;
                tracing::info!("lock: open on {:08X} ({} hand), its keyhole at {:.2?} facing {:.2?}", mem::at::<u32>(refr + FORM_ID).unwrap_or(0), if hand == 0 { "left" } else { "right" }, keyhole.at, keyhole.out);
            }
            let Some((_, keyhole)) = s.open else { return };
            if let Some((tip, hand)) = pick(view, f, true, s.hand) {
                if let Some((was, _, t)) = s.tip {
                    let dt = (now - t) as f32;
                    if dt > 1e-4 {
                        s.rise = (tip - was).dot(keyhole.up) / dt;
                    }
                }
                s.tip = Some((tip, hand, now));
            }
            stand(s, menu, keyhole);
        });
    }

    /// The tip at a locked thing: its keyhole, and the game's Activate of it asked for.
    fn touch(s: &mut State, view: &rustvr_adapter::View, f: &RtFrame, now: f64) {
        let Some((tip, _)) = pick(view, f, false, 1) else {
            s.spent = None;
            return;
        };
        if s.spent.is_some_and(|at| at.distance(tip) < REARM) || s.pending.is_some() {
            return;
        }
        s.spent = None;
        let b = camera::basis();
        let mut near: Vec<usize> = Vec::new();
        for cell in crate::items::cells() {
            let mut entry = cell + 0x48;
            let mut guard = 0;
            while entry != 0 && guard < 8192 {
                guard += 1;
                let r = mem::deref(entry);
                entry = mem::deref(entry + 4);
                let Some(p) = (r != 0).then(|| mem::at::<[f32; 3]>(r + engine::refr::POS)).flatten() else { continue };
                if b.point_from_game(Vec3::from(p)).distance(tip) < NEAR && locked_thing(r) {
                    near.push(r);
                }
            }
        }
        s.triangles.retain(|r, (t, _)| near.contains(r) && now - *t < TRIANGLES_FOR);
        for refr in near {
            let tris = &s.triangles.entry(refr).or_insert_with(|| (now, triangles_of(refr))).1;
            let Some((at, out, d)) = nearest(tris, tip) else { continue };
            if d > TOUCH {
                continue;
            }
            let keyhole = Keyhole::new(at, out, view.head_pos);
            tracing::info!("lock: the pick's tip at {:08X}, {:.3} m from it at {at:.2?}", mem::at::<u32>(refr + FORM_ID).unwrap_or(0), d);
            s.pending = Some(Touch { refr });
            s.spent = Some(tip);
            s.touched = Some((refr, keyhole, now));
            return;
        }
    }

    /// The keyhole of a lock opened another way: where the eyes were on the thing (else in front
    /// of them).
    fn looked_at(view: &rustvr_adapter::View, refr: usize) -> Keyhole {
        let look = (view.head_rot * Vec3::Z).normalize_or(Vec3::Z);
        match hit(&triangles_of(refr), view.head_pos, look) {
            Some((at, out)) => Keyhole::new(at, out, view.head_pos),
            None => Keyhole::new(view.head_pos + look * 0.5, -look, view.head_pos),
        }
    }

    /// The menu's tumblers at the keyhole, each at its height: the game's own model, in its world.
    fn stand(s: &mut State, menu: usize, keyhole: Keyhole) {
        if s.models.is_none() {
            s.models = load(keyhole);
        }
        let Some(m) = s.models.as_ref() else { return };
        let turn = keyhole.turn(m.right, m.up, m.out);
        let top = mem::at::<f32>(menu + TOP).filter(|t| *t > 0.0).unwrap_or(0.667);
        let b = camera::basis();
        for (k, (node, seq)) in m.tumblers.iter().enumerate() {
            let height = mem::at::<f32>(menu + TUMBLER + k * TUMBLER_SIZE + HEIGHT).unwrap_or(0.0).clamp(0.0, top);
            // Its bottom (its middle at the bottom of its travel) on the keyhole's level, across the
            // row, a little proud of the surface.
            let across = (k as f32 - 2.0) * SPACING;
            let want = keyhole.at + keyhole.right * across + keyhole.up * (TUMBLER_TALL * 0.5) + keyhole.out * PROUD;
            let pos = want - turn * b.point_from_game(m.bottom * m.scale);
            seq.hold(height);
            scene::place(*node, pos, turn, m.scale);
            scene::animate(*node);
        }
    }

    /// The five tumblers put up in the world, and how the model stands (measured on it: where its
    /// middle goes between the bottom and the top of its `UP` animation is its up).
    fn load(keyhole: Keyhole) -> Option<Models> {
        let mut tumblers = Vec::new();
        for _ in 0..5 {
            let node = scene::load(TUMBLER_MODEL);
            if node == 0 {
                break;
            }
            let Some(seq) = scene::play(node, UP) else {
                tracing::info!("lock: {TUMBLER_MODEL} has no '{UP}' sequence");
                scene::unload(node);
                break;
            };
            tumblers.push((node, seq));
        }
        if tumblers.len() < 5 {
            tracing::info!("lock: the tumblers couldn't be put up ({} of 5)", tumblers.len());
            for (n, _) in tumblers {
                scene::unload(n);
            }
            return None;
        }
        // Measured unturned, at its own size, where it stands: its bound's middle at the bottom and
        // at the top of its travel, from its root (game units, its own axes).
        let (node, seq) = tumblers[0];
        let middle = |phase: f32| -> Option<(Vec3, f32)> {
            seq.hold(phase);
            scene::place(node, keyhole.at, Quat::IDENTITY, 1.0);
            scene::animate(node);
            let bound: [f32; 4] = mem::at(node + NODE_BOUND)?;
            let root: [f32; 3] = mem::at(node + engine::av::WORLD_TRANSLATE)?;
            Some((Vec3::new(bound[0], bound[1], bound[2]) - Vec3::from(root), bound[3]))
        };
        let (bottom, radius) = middle(0.0).unwrap_or((Vec3::ZERO, 10.0));
        let (topmost, _) = middle(seq.length).unwrap_or((bottom, radius));
        let travel = topmost - bottom;
        // (Its up: the way its travel goes, or the game's own up; its right across that, the game's
        // X unless that's its up; its out from them.)
        let up_game = if travel.length() > 0.01 { travel.normalize() } else { Vec3::Z };
        let right_game = if up_game.x.abs() < 0.7 { (Vec3::X - up_game * up_game.x).normalize() } else { (Vec3::Y - up_game * up_game.y).normalize() };
        let out_game = right_game.cross(up_game);
        let b = camera::basis();
        let scale = (TUMBLER_TALL * camera::UNITS_PER_METER / (2.0 * radius.max(0.1))).clamp(0.01, 10.0);
        tracing::info!(
            "lock: the tumblers put up: radius {radius:.2}, middle {bottom:.2?} at the bottom, travel {travel:.2?} over {:.2} s (up {up_game:.2?}, right {right_game:.2?}), at {scale:.3} times its size",
            seq.length
        );
        Some(Models { tumblers, right: b.dir_from_game(right_game), up: b.dir_from_game(up_game), out: b.dir_from_game(out_game), bottom, scale })
    }

    fn unload(s: &mut State) {
        if let Some(m) = s.models.take() {
            for (n, _) in m.tumblers {
                scene::unload(n);
            }
        }
    }

    /// Each game frame, after the laser has put the game's cursor: the game's Activate of a locked
    /// thing the tip touched; with the lock open, the tip works the pick (the cursor's place along
    /// the lock, the mouse's movement and its button in `v`).
    pub fn frame(f: &RtFrame, v: &mut crate::input::Virtual, now: f64) {
        let mut st = STATE.lock().unwrap();
        let s = st.get_or_insert_with(State::default);
        if let Some(t) = s.pending.take() {
            let ok = crate::world::activate(t.refr);
            tracing::info!("lock: the pick at {:08X}: the game's Activate ({ok}), its lock {}", mem::at::<u32>(t.refr + FORM_ID).unwrap_or(0), if crate::menus::menu_open(LOCKPICK_MENU) { "open" } else { "not open (yet)" });
            let hand = s.tip.map_or(1, |t| t.1);
            haptic(hand as u32, 0.02, 120.0, 0.5);
        }
        let menu = if crate::menus::menu_open(LOCKPICK_MENU) { menu() } else { 0 };
        let Some((_, keyhole)) = s.open.filter(|_| menu != 0) else {
            s.cursor = TUMBLERS;
            s.found = [None; 5];
            s.picks = -1;
            return;
        };
        let hand = s.tip.map_or(s.hand, |t| t.1);
        let c = &f.hands[hand];
        let picks = mem::at::<i32>(menu + PICKS).unwrap_or(0);
        let field = |k: usize, at: usize| menu + TUMBLER + k * TUMBLER_SIZE + at;
        let tumblers: Vec<(bool, bool, bool)> = (0..5)
            .map(|k| (mem::at::<u8>(field(k, MOVING)).unwrap_or(0) != 0, mem::at::<u8>(field(k, SET)).unwrap_or(0) != 0, mem::at::<i32>(field(k, AT_TOP)).unwrap_or(-1) != -1))
            .collect();
        if s.picks < 0 {
            s.picks = picks;
            for (k, t) in tumblers.iter().enumerate() {
                s.set[k] = t.1;
                s.top[k] = t.2;
            }
            tracing::info!("lock: {picks} picks; the tip is the pick");
        }
        let tip = s.tip.map(|t| keyhole.local(t.0)).unwrap_or(Vec3::ZERO);
        // Pulled out: shut, as its Close button does.
        if tip.length() > PULL_OUT {
            let since = *s.out_since.get_or_insert(now);
            if now - since > PULL_OUT_TIME {
                s.out_since = None;
                tracing::info!("lock: the pick pulled out ({:.2} m from the keyhole): closed", tip.length());
                click(menu, CLOSE);
                return;
            }
        } else {
            s.out_since = None;
        }
        // Under which tumbler: the cursor to where the game put that one, or on along the lock
        // until it does.
        let want = tumbler(tip.x);
        let under = mem::at::<i32>(menu + UNDER_CURSOR).unwrap_or(-1);
        let moving = tumblers.iter().any(|t| t.0);
        if (0..5).contains(&under) {
            s.found[under as usize] = Some(s.cursor[0]);
        }
        if !moving && under != want as i32 {
            s.cursor[0] = match s.found[want] {
                Some(x) => x,
                None if under < 0 => s.cursor[0] + (0.5 - s.cursor[0]).clamp(-STEP, STEP),
                None => s.cursor[0] + if (under as usize) < want { STEP } else { -STEP },
            }
            .clamp(0.0, 1.0);
        }
        s.cursor[1] = TUMBLERS[1];
        crate::menus::point(s.cursor);
        // Pushed up: the mouse moved up, to the game. Set: its button. A: Auto Attempt.
        if s.rise > FLICK {
            v.mouse[1] = LIFT;
        }
        if c.trigger > SET_PULL {
            v.buttons |= 1;
        }
        let a = c.buttons & CTRL_A != 0;
        if a && !s.a {
            tracing::info!("lock: Auto Attempt");
            click(menu, AUTO_ATTEMPT);
        }
        s.a = a;
        // Felt: the tumbler riding the pick, at the top, set; a pick broken.
        if moving && now >= s.next_tick {
            s.next_tick = now + 0.1;
            haptic(hand as u32, 0.012, 200.0, 0.25);
        }
        for (k, t) in tumblers.iter().enumerate() {
            if t.2 && !s.top[k] && !t.1 {
                haptic(hand as u32, 0.03, 90.0, 0.6);
            }
            if t.1 && !s.set[k] {
                haptic(hand as u32, 0.04, 150.0, 0.8);
                tracing::info!("lock: tumbler {k} set");
            }
            s.set[k] = t.1;
            s.top[k] = t.2;
        }
        if picks < s.picks {
            haptic(hand as u32, 0.12, 60.0, 1.0);
            tracing::info!("lock: a pick broke, {picks} left");
        }
        s.picks = picks;
    }

    /// One of the menu's own buttons, clicked as the game's click does.
    fn click(menu: usize, id: u32) {
        type Click = unsafe extern "thiscall" fn(usize, u32, usize);
        unsafe { std::mem::transmute::<usize, Click>(HANDLE_CLICK)(menu, id, 0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_nearest_point_of_a_triangle() {
        let t = [Vec3::ZERO, Vec3::X, Vec3::Y];
        assert!(closest_on_triangle(Vec3::new(0.2, 0.2, 1.0), &t).abs_diff_eq(Vec3::new(0.2, 0.2, 0.0), 1e-6), "over its face");
        assert!(closest_on_triangle(Vec3::new(-1.0, -1.0, 0.0), &t).abs_diff_eq(Vec3::ZERO, 1e-6), "past a corner");
        assert!(closest_on_triangle(Vec3::new(0.5, -1.0, 0.0), &t).abs_diff_eq(Vec3::new(0.5, 0.0, 0.0), 1e-6), "past an edge");
        assert!(closest_on_triangle(Vec3::new(1.0, 1.0, 0.0), &t).abs_diff_eq(Vec3::new(0.5, 0.5, 0.0), 1e-6), "past the long edge");
    }

    #[test]
    fn a_keyhole_on_a_door_reads_as_the_player_facing_it_sees_it() {
        // A door ahead of a player looking along +Z: it faces back at them.
        let k = Keyhole::new(Vec3::new(0.0, 1.0, 1.0), Vec3::NEG_Z, Vec3::new(0.0, 1.6, 0.0));
        assert!(k.up.abs_diff_eq(Vec3::Y, 1e-6));
        assert!(k.right.abs_diff_eq(Vec3::X, 1e-6), "their right is +X: {:?}", k.right);
        assert!(k.local(Vec3::new(0.05, 1.02, 0.99)).abs_diff_eq(Vec3::new(0.05, 0.02, 0.01), 1e-5));
        // The tumbler under the tip: the middle one at the keyhole, one more every SPACING.
        assert_eq!(tumbler(0.0), 2);
        assert_eq!(tumbler(SPACING), 3);
        assert_eq!(tumbler(-5.0 * SPACING), 0);
    }

    #[test]
    fn a_keyhole_on_a_lid_has_its_up_away_from_the_player() {
        let k = Keyhole::new(Vec3::new(0.0, 0.6, 1.0), Vec3::Y, Vec3::new(0.0, 1.6, 0.0));
        assert!(k.up.abs_diff_eq(Vec3::Z, 1e-6), "up the lid, away from them: {:?}", k.up);
        assert!((k.right.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn a_model_turned_to_the_keyhole() {
        let k = Keyhole::new(Vec3::ZERO, Vec3::X, Vec3::new(2.0, 1.6, 0.0));
        // A model whose own up is +Y, right +X and out -Z, as the protocol's axes have them.
        let q = k.turn(Vec3::X, Vec3::Y, Vec3::NEG_Z);
        assert!((q * Vec3::Y).abs_diff_eq(k.up, 1e-5));
        assert!((q * Vec3::NEG_Z).abs_diff_eq(k.out, 1e-5));
        assert!((q * Vec3::X).abs_diff_eq(k.right, 1e-5));
    }

    #[test]
    fn the_tip_is_the_end_away_from_the_hand() {
        let (center, half) = (Vec3::ZERO, Vec3::new(0.005, 0.06, 0.003));
        let tip = tip_of(Vec3::new(1.0, 1.0, 1.0), Quat::IDENTITY, center, half, Vec3::new(1.0, 0.95, 1.0));
        assert!(tip.abs_diff_eq(Vec3::new(1.0, 1.06, 1.0), 1e-6));
    }
}
