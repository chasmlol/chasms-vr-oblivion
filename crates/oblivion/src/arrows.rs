//! The game's arrows and spells in flight meet what's drawn (`docs/design/hitboxes.md`): through the
//! gaps the game's simplified collision fills (between a chair's legs, through a fence's bars, past
//! a crude box's corner), and into the drawn surface where it is, with the game's own impact (its
//! damage, its sticking, its breaking on stone, its sounds; a spell's effect, its explosion).
//!
//! A spell's bolt or ball flies the same way, its listener (`bhkCharacterListenerSpell`, 0xA75218)
//! sharing the arrow's constraints callback; its own `contactPointAddedCallback` (0x68F450,
//! thiscall: the listener, the contact point) finds the projectile (the listener's +0x1E0) and the
//! reference met and hands the hit to the projectile. A drawn shape met on a spell's way is that
//! callback, given the contact there.
//!
//! An arrow (`ArrowProjectile`) flies as a Havok character proxy whose listener
//! (`bhkCharacterListenerArrow`) the proxy tells of each new contact (`contactPointAddedCallback`,
//! 0x60DBB0: the game's impact, 0x60D950, on what it met) and of its constraints each step
//! (`processConstraintsCallback`, 0x60DBF0: the arrow's own sets every surface's aside, so a proxy
//! never slides or stops: the impact is all there is). So:
//!
//! * a contact with a fixed body a drawn shape stands in for ([`crate::collision::stands_in`])
//!   isn't passed on: the arrow flies on through the game's shape;
//! * each step, where the arrow goes in it (its proxy's place and velocity for the step, as Havok
//!   hands them the listener: [`STEP_MAX`] at most) is held up to the drawn shapes
//!   ([`crate::collision::drawn_hit`]); one it meets is the game's impact, on that reference's body,
//!   at the drawn point. The game sticks an arrow where its line enters the
//!   body's own shape (it casts along it); one that sticks is put where the drawn surface is
//!   instead, as far in as the game puts one (12 units).
//!
//! Contacts with everything else (the land, trees, creatures, things lying about, what no drawn
//! shape stands in for) are the game's as ever. The game's own walking and physics never see any
//! of it.
//!
//! The listener the proxy calls is at +0x1F0 in its object, the arrow at +0x1E0 from it; an arrow's
//! state +0x60 (0 in flight), its form flags +8 (0x20 deleted). The step (`hkSimplexSolverInput`):
//! the place +0 and the velocity +0x10 (Havok's units), the time +0x40. The impact (cdecl: arrow, the
//! collidable met, the point in Havok's units, the normal: `hkVector4`s); a body's collidable at
//! `hkRigidBody` +0x14, its type +0x18 (1 an entity). `TESObjectREFR::SetPos` 0x4D8A30,
//! `NiAVObject::Update` 0x707370.

/// Game units an arrow goes in a step at most (faster: a jump, not a flight).
pub const STEP_MAX: f32 = 400.0;
/// How far into the surface the game puts an arrow it sticks (game units, along it).
pub const STICK_IN: f32 = 12.0;

/// Collision materials (Bethesda's, 0..31) an arrow breaks on rather than sticks in (stone, glass,
/// metal and their heavy and stair kinds), as the game's impact has them.
pub fn breaks_on(material: u32) -> bool {
    matches!(material, 0 | 3 | 5 | 10 | 11 | 13 | 15 | 18 | 20 | 25 | 26 | 28 | 30)
}

#[cfg(all(windows, target_arch = "x86"))]
pub use win::install;

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn install() {}

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::engine::av;
    use crate::mem;
    use glam::Vec3;
    use std::collections::HashSet;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const CONTACT: usize = 0x0060_DBB0;
    const CONSTRAINTS: usize = 0x0060_DBF0;
    const IMPACT: usize = 0x0060_D950;
    const SPELL_CONTACT: usize = 0x0068_F450;
    /// The listeners' vtables as the proxy calls them (the spells' shares the constraints callback).
    const ARROW_LISTENER: usize = 0x00A6_FA38;
    const SPELL_LISTENER: usize = 0x00A7_5218;
    const LISTENER_ARROW: usize = 0x1E0;
    const ARROW_STATE: usize = 0x60;
    const FORM_FLAGS: usize = 0x8;
    const FORM_ID: usize = 0xC;
    const DELETED: u32 = 0x20;
    const REFR_NODE: usize = 0x3C;
    /// `hkRootCdPoint`: the collidable met at +0x28. A collidable's type +0x18 (1: an entity), its
    /// entity 0x14 before it.
    const POINT_COLLIDABLE: usize = 0x28;
    const COLLIDABLE_TYPE: usize = 0x18;
    const ENTITY_COLLIDABLE: usize = 0x14;
    const STEP_PLACE: usize = 0x0;
    const STEP_VELOCITY: usize = 0x10;
    const STEP_TIME: usize = 0x40;
    const SET_POS: usize = 0x004D_8A30;
    const UPDATE_NODE: usize = 0x0070_7370;

    static REAL_CONTACT: AtomicUsize = AtomicUsize::new(0);
    static REAL_CONSTRAINTS: AtomicUsize = AtomicUsize::new(0);
    static REAL_SPELL_CONTACT: AtomicUsize = AtomicUsize::new(0);

    /// The (arrow, body) the game's impact was asked for and didn't take: not asked again.
    static TRIED: Mutex<Option<HashSet<(usize, usize)>>> = Mutex::new(None);

    #[repr(C, align(16))]
    struct V4([f32; 4]);

    /// A contact as Havok hands one to a listener (`hkRootCdPoint`): the point and the normal (its
    /// `w` the distance), each collidable and its shape key.
    #[repr(C, align(16))]
    struct CdPoint {
        point: [f32; 4],
        normal: [f32; 4],
        a: u32,
        key_a: u32,
        b: u32,
        key_b: u32,
    }

    /// The fixed body a contact is with, if it's one a drawn shape stands in for.
    fn stood_in(point: usize) -> bool {
        let c = mem::deref(point + POINT_COLLIDABLE);
        c != 0 && mem::at::<u8>(c + COLLIDABLE_TYPE) == Some(1) && crate::collision::stands_in(c - ENTITY_COLLIDABLE)
    }

    unsafe extern "thiscall" fn contact(this: usize, point: usize) -> u32 {
        let real: unsafe extern "thiscall" fn(usize, usize) -> u32 = unsafe { std::mem::transmute(REAL_CONTACT.load(Ordering::Acquire)) };
        // (Through the game's shape: the drawn one is met on the arrow's way, below.)
        if crate::hooks::guard("arrow contact", false, || stood_in(point)) {
            return 0;
        }
        unsafe { real(this, point) }
    }

    unsafe extern "thiscall" fn spell_contact(this: usize, point: usize) -> u32 {
        let real: unsafe extern "thiscall" fn(usize, usize) -> u32 = unsafe { std::mem::transmute(REAL_SPELL_CONTACT.load(Ordering::Acquire)) };
        if crate::hooks::guard("spell contact", false, || stood_in(point)) {
            return 0;
        }
        unsafe { real(this, point) }
    }

    unsafe extern "thiscall" fn constraints(this: usize, manifold: usize, input: usize) -> u32 {
        let real: unsafe extern "thiscall" fn(usize, usize, usize) -> u32 = unsafe { std::mem::transmute(REAL_CONSTRAINTS.load(Ordering::Acquire)) };
        let r = unsafe { real(this, manifold, input) };
        crate::hooks::guard_void("arrow path", || step(this, input));
        r
    }

    /// An arrow's or a spell's step: where it goes in it, held up to the drawn shapes.
    fn step(listener: usize, input: usize) {
        let spell = match mem::deref(listener) {
            ARROW_LISTENER => false,
            SPELL_LISTENER => true,
            _ => return,
        };
        let arrow = mem::deref(listener + LISTENER_ARROW);
        let alive = arrow != 0 && mem::at::<u32>(arrow + FORM_FLAGS).is_some_and(|f| f & DELETED == 0);
        // (A spell's own callback asks the projectile whether it's still flying.)
        let flying = alive && (spell || mem::at::<u32>(arrow + ARROW_STATE) == Some(0));
        let (Some(place), Some(velocity), Some(time)) = (mem::at::<[f32; 4]>(input + STEP_PLACE), mem::at::<[f32; 4]>(input + STEP_VELOCITY), mem::at::<f32>(input + STEP_TIME)) else { return };
        if !flying || !(time > 0.0 && time < 1.0) {
            return;
        }
        let scale = crate::collision::HAVOK_SCALE;
        let from = Vec3::new(place[0], place[1], place[2]) * scale;
        let to = from + Vec3::new(velocity[0], velocity[1], velocity[2]) * scale * time;
        if !(from.is_finite() && to.is_finite()) || from.distance(to) > super::STEP_MAX || from.distance(to) < 0.01 {
            return;
        }
        let Some((hit, normal, body)) = crate::collision::drawn_hit(from, to) else { return };
        {
            let mut tried = TRIED.lock().unwrap();
            let tried = tried.get_or_insert_with(HashSet::new);
            if tried.len() > 512 {
                tried.clear();
            }
            if !tried.insert((arrow, body)) {
                return;
            }
        }
        if spell {
            // The spell's own contact callback, given the contact where it's drawn.
            let cd = CdPoint {
                point: [hit.x / scale, hit.y / scale, hit.z / scale, 0.0],
                normal: [normal.x, normal.y, normal.z, 0.0],
                a: (body + ENTITY_COLLIDABLE) as u32,
                key_a: u32::MAX,
                b: (body + ENTITY_COLLIDABLE) as u32,
                key_b: u32::MAX,
            };
            let real: unsafe extern "thiscall" fn(usize, usize) -> u32 = unsafe { std::mem::transmute(REAL_SPELL_CONTACT.load(Ordering::Acquire)) };
            unsafe { real(listener, &cd as *const CdPoint as usize) };
            let id = mem::at::<u32>(arrow + FORM_ID).unwrap_or(0);
            tracing::info!("arrows: spell {id:08X} met a drawn shape at {hit:.0?} (body {body:#x}): the game's hit");
            return;
        }
        // The game's impact on that body, where it's drawn.
        let point = V4([hit.x / scale, hit.y / scale, hit.z / scale, 0.0]);
        let n = V4([normal.x, normal.y, normal.z, 0.0]);
        type Impact = unsafe extern "C" fn(usize, usize, *const V4, *const V4) -> u32;
        unsafe { std::mem::transmute::<usize, Impact>(IMPACT)(arrow, body + ENTITY_COLLIDABLE, &point, &n) };
        let state = mem::at::<u32>(arrow + ARROW_STATE).unwrap_or(0);
        let material = crate::collision::body_material(body);
        let id = mem::at::<u32>(arrow + FORM_ID).unwrap_or(0);
        tracing::info!("arrows: {id:08X} met a drawn shape at {hit:.0?} (body {body:#x}, material {material:?}): the game's impact, the arrow's state now {state}");
        // Stuck (not broken on stone or metal): where the drawn surface is, as far in as the game
        // puts one.
        if state == 0 || material.is_some_and(super::breaks_on) {
            return;
        }
        let Some(dir) = (to - from).try_normalize() else { return };
        put(arrow, hit + dir * super::STICK_IN);
    }

    /// An arrow put at `to` (game units), its 3D with it.
    fn put(arrow: usize, to: Vec3) {
        type SetPos = unsafe extern "thiscall" fn(usize, f32, f32, f32);
        type Update = unsafe extern "thiscall" fn(usize, f32, u32) -> u32;
        unsafe { std::mem::transmute::<usize, SetPos>(SET_POS)(arrow, to.x, to.y, to.z) };
        let node = mem::deref(arrow + REFR_NODE);
        // (A stuck arrow's 3D stands as itself in the cell's, whose place is the world's: its place
        // is its own.)
        let parent = if node != 0 { mem::deref(node + crate::nodes::PARENT) } else { 0 };
        let at_origin = parent == 0
            || (mem::at::<[f32; 3]>(parent + av::WORLD_TRANSLATE).is_some_and(|t| Vec3::from(t).length() < 0.01)
                && mem::at::<[f32; 9]>(parent + av::WORLD_ROTATE).is_some_and(|r| r.iter().zip([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).all(|(a, b)| (a - b).abs() < 1e-4)));
        if node != 0 && at_origin && mem::is_readable((node + av::LOCAL_TRANSLATE) as *const u8, 12) {
            unsafe {
                ((node + av::LOCAL_TRANSLATE) as *mut [f32; 3]).write_unaligned(to.to_array());
                std::mem::transmute::<usize, Update>(UPDATE_NODE)(node, 0.0, 0);
            }
        }
    }

    /// The game's arrows and spells through the gaps its collision fills, onto the drawn shapes.
    pub fn install() {
        let a = crate::hooks::install_into(CONTACT as *const (), contact as *const (), "the arrow's contact", &REAL_CONTACT);
        let b = crate::hooks::install_into(CONSTRAINTS as *const (), constraints as *const (), "the arrow's step", &REAL_CONSTRAINTS);
        let c = crate::hooks::install_into(SPELL_CONTACT as *const (), spell_contact as *const (), "the spell's contact", &REAL_SPELL_CONTACT);
        if a && b && c {
            tracing::info!("arrows: the game's arrows and spells meet the drawn shapes");
        }
    }
}
