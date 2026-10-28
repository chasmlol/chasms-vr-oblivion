//! Inline detours and the safety wrapper every hook body runs inside.
//!
//! Two rules:
//!
//! * **Never hook a target twice.** Installing a second detour over our own trampoline corrupts
//!   the first. [`install`] refuses a target it already hooked and returns `None`.
//! * **A panic must never unwind into the game.** [`guard`] runs a hook body inside
//!   `catch_unwind`; if it panics we log and fall back, and the caller still invokes the original.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

#[cfg(windows)]
use retour::RawDetour;

#[cfg(windows)]
#[allow(dead_code)] // held only to keep the detour (and its trampoline) alive for the process.
struct SendDetour(RawDetour);
// Detours are installed once and then only kept alive; the pointers inside are process-global.
#[cfg(windows)]
unsafe impl Send for SendDetour {}

#[cfg(windows)]
struct Registry {
    detours: Vec<SendDetour>,
    /// Targets already hooked (a handful, so a plain Vec is cheaper than a HashSet and `const`).
    hooked: Vec<usize>,
}

#[cfg(windows)]
static REGISTRY: Mutex<Registry> = Mutex::new(Registry { detours: Vec::new(), hooked: Vec::new() });

/// Installs a detour on `target`, returning a pointer to the trampoline that calls the original, or
/// `None` if `target` is null, already hooked, or the detour could not be built/enabled.
///
/// The returned pointer is only valid while the process lives (the detour is kept alive forever).
#[cfg(windows)]
pub fn install(target: *const (), detour: *const (), name: &str) -> Option<*const ()> {
    if target.is_null() {
        tracing::error!("hook {name}: null target");
        return None;
    }
    let mut reg = REGISTRY.lock().unwrap();
    if reg.hooked.contains(&(target as usize)) {
        tracing::warn!("hook {name}: target {target:p} already hooked, refusing to hook twice");
        return None;
    }
    reg.hooked.push(target as usize);
    let det = match unsafe { RawDetour::new(target, detour) } {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("hook {name}: create failed: {e}");
            reg.hooked.pop();
            return None;
        }
    };
    if let Err(e) = unsafe { det.enable() } {
        tracing::error!("hook {name}: enable failed: {e}");
        reg.hooked.pop();
        return None;
    }
    let tramp = det.trampoline() as *const () as *const ();
    reg.detours.push(SendDetour(det));
    tracing::info!("hook {name}: installed at {target:p}");
    Some(tramp)
}

#[cfg(not(windows))]
pub fn install(_target: *const (), _detour: *const (), _name: &str) -> Option<*const ()> {
    None
}

/// [`install`] for targets other threads may be calling while it's installed: the trampoline is
/// stored in `original` before the detour goes live, so the detour never finds it unset. True when
/// installed.
#[cfg(windows)]
pub fn install_into(target: *const (), detour: *const (), name: &str, original: &std::sync::atomic::AtomicUsize) -> bool {
    use std::sync::atomic::Ordering;
    if target.is_null() {
        tracing::error!("hook {name}: null target");
        return false;
    }
    let mut reg = REGISTRY.lock().unwrap();
    if reg.hooked.contains(&(target as usize)) {
        tracing::warn!("hook {name}: target {target:p} already hooked, refusing to hook twice");
        return false;
    }
    let det = match unsafe { RawDetour::new(target, detour) } {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("hook {name}: create failed: {e}");
            return false;
        }
    };
    original.store(det.trampoline() as *const () as usize, Ordering::Release);
    if let Err(e) = unsafe { det.enable() } {
        tracing::error!("hook {name}: enable failed: {e}");
        return false;
    }
    reg.hooked.push(target as usize);
    reg.detours.push(SendDetour(det));
    tracing::info!("hook {name}: installed at {target:p}");
    true
}

#[cfg(not(windows))]
pub fn install_into(_target: *const (), _detour: *const (), _name: &str, _original: &std::sync::atomic::AtomicUsize) -> bool {
    false
}

/// Runs a hook body, swallowing any panic so it can never unwind into game code. Returns `fallback`
/// if the body panicked.
pub fn guard<R>(name: &str, fallback: R, body: impl FnOnce() -> R) -> R {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(r) => r,
        Err(_) => {
            tracing::error!("hook {name}: panic caught, falling back");
            fallback
        }
    }
}

/// Convenience for hooks whose body returns nothing and whose fallback is a no-op.
pub fn guard_void(name: &str, body: impl FnOnce()) {
    guard(name, (), body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_returns_body_value() {
        assert_eq!(guard("t", -1, || 7), 7);
    }

    #[test]
    fn guard_catches_panic_and_falls_back() {
        let r = guard("t", 42, || panic!("boom"));
        assert_eq!(r, 42);
    }
}
