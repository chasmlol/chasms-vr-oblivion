//! What the game picks to use, along a hand. Each frame the game's interface picks what the player
//! can use (its crosshair: what Activate uses, what its card names, the reticle's icon) along a
//! ray from the eye the way the player faces, as far as its own reach (`iActivatePickLength`). In
//! VR, while a free hand is put up in front ([`rustvr_protocol::INT_POINT_LEFT`]: the runtime
//! says which), that ray is the hand's pointer instead: the game's own pick, its own reach, along
//! the hand. A gate the hand points at within reach is what its trigger opens, and the game's card
//! of it floats over the hand ([`crate::hud`], [`rustvr_protocol::HUD_POINTED`]); with no hand
//! put up the game picks along the eyes as ever (a person looked at, a foe's health), but the card
//! isn't shown there.
//!
//! The pick (0x5806D0, the interface's, every frame) lays its ray out on its stack before it casts
//! it: the eye at +0x20 and the direction at +0x34 (the game's units and axes), and at 0x58080C it
//! goes on to the reach. There, hooked, the ray is the hand's.

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use glam::{Quat, Vec3};
    use rustvr_protocol::{INT_POINT_LEFT, INT_POINT_RIGHT};
    use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

    /// In the interface's pick, just after its ray is laid out (before the reach is read).
    const RAY_LAID_OUT: usize = 0x0058_080C;

    /// The game's continuation (its own instructions there, then back into the pick).
    static REAL: AtomicUsize = AtomicUsize::new(0);
    /// The hand the last pick was along: 0 none (the eyes), 1 the left, 2 the right.
    static ALONG: AtomicU32 = AtomicU32::new(0);

    /// The hand the game's pick is along now ([`rustvr_protocol::HAND_LEFT`] or
    /// [`rustvr_protocol::HAND_RIGHT`]), or None: the eyes'.
    pub fn hand() -> Option<u32> {
        ALONG.load(Ordering::Relaxed).checked_sub(1)
    }

    /// The pick's ray (the game's eye and its direction, on its stack): the pointing hand's.
    extern "C" fn ray(origin: *mut [f32; 3], dir: *mut [f32; 3]) {
        crate::hooks::guard_void("pointing", || {
            let along = crate::link::last().and_then(|s| {
                let view = s.view?;
                let flags = s.frame.intents.flags;
                let h = if flags & INT_POINT_LEFT != 0 {
                    0
                } else if flags & INT_POINT_RIGHT != 0 {
                    1
                } else {
                    return None;
                };
                let aim = s.frame.hands[h].aim;
                let q = Quat::from_array(aim.rot);
                if aim.valid == 0 || q.length_squared() < 1e-12 {
                    return None;
                }
                let basis = crate::camera::basis();
                let o = basis.point_to_game(view.tracking_to_world.transform_point3(Vec3::from_array(aim.pos)));
                let d = basis.dir_to_game(view.tracking_to_world.transform_vector3(q.normalize() * Vec3::Z)).normalize_or_zero();
                (o.is_finite() && d != Vec3::ZERO).then_some((h, o, d))
            });
            match along {
                Some((h, o, d)) => {
                    unsafe {
                        *origin = o.to_array();
                        *dir = d.to_array();
                    }
                    if ALONG.swap(h as u32 + 1, Ordering::Relaxed) != h as u32 + 1 {
                        tracing::info!("pointing: the game picks what to use along the {} hand", if h == 0 { "left" } else { "right" });
                    }
                }
                None => {
                    if ALONG.swap(0, Ordering::Relaxed) != 0 {
                        tracing::info!("pointing: the game picks what to use along the eyes (no hand put up)");
                    }
                }
            }
        });
    }

    /// At [`RAY_LAID_OUT`]: the ray given to [`ray`] (the eye at +0x20 of the pick's stack, the
    /// direction at +0x34, past the 36 bytes saved here), every register and flag kept.
    #[unsafe(naked)]
    unsafe extern "C" fn at_ray() {
        core::arch::naked_asm!(
            "pushad",
            "pushfd",
            "lea eax, [esp + 36 + 0x20]",
            "lea ecx, [esp + 36 + 0x34]",
            "push ecx",
            "push eax",
            "call {ray}",
            "add esp, 8",
            "popfd",
            "popad",
            "jmp dword ptr [{real}]",
            ray = sym ray,
            real = sym REAL,
        )
    }

    pub fn install() {
        if crate::hooks::install_into(RAY_LAID_OUT as *const (), at_ray as *const (), "the interface's pick (its ray)", &REAL) {
            tracing::info!("pointing: what the game picks to use is along a hand put up in front");
        }
    }
}
