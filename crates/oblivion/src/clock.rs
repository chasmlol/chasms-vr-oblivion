//! The game's clock, to the microsecond. Oblivion times each frame by `GetTickCount` (its time
//! globals at 0xB33E90, updated by 0x47D170 at the start of every frame with the tick count): whole
//! milliseconds of a clock that only moves every 15.6 ms. At a headset's 90 to 120 frames a second
//! most frames are timed at nothing (the game makes it 1 ms) and every second or third at 15 or
//! 16 ms, and everything the game moves by its frame time (walking, physics, animation, the
//! actors) moves in those lumps: the world lurches by a step on one frame and stands still on the
//! next, and the player runs on after the stick is let go.
//!
//! So the frame time is the real time since the last frame, from the performance counter. The
//! rest is the game's: its paused clock, its fixed-step mode, its cap on a long frame (166 ms) and
//! its time scale (0xB06704); the milliseconds it counts for its timers stay as it counts them.
//!
//! Time globals (`TimeGlobal`): +0 paused (byte), +4 a fixed frame time (0: none), +0xC the frame's
//! time (seconds), +0x10 the milliseconds counted, +0x14 the tick count at the start.

/// The frame's time in seconds, from the performance counter's ticks between two frames.
pub fn frame_seconds(ticks: i64, freq: i64) -> f32 {
    if ticks <= 0 || freq <= 0 { 0.001 } else { (ticks as f64 / freq as f64).clamp(0.0001, MAX_FRAME) as f32 }
}

/// The longest frame the game takes as it is (its own cap: 166 ms).
pub const MAX_FRAME: f64 = 0.166;

#[cfg(all(windows, target_arch = "x86"))]
pub use win::install;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

    const UPDATE: usize = 0x0047_D170;
    const TIME_SCALE: usize = 0x00B0_6704;
    const PAUSED: usize = 0x0;
    const FIXED: usize = 0x4;
    const FRAME_TIME: usize = 0xC;

    static REAL: AtomicUsize = AtomicUsize::new(0);
    static LAST: AtomicI64 = AtomicI64::new(0);

    fn counter() -> (i64, i64) {
        use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
        let (mut now, mut freq) = (0i64, 0i64);
        unsafe {
            let _ = QueryPerformanceCounter(&mut now);
            let _ = QueryPerformanceFrequency(&mut freq);
        }
        (now, freq)
    }

    unsafe extern "thiscall" fn update(this: *mut u8, ticks: u32) {
        let real: unsafe extern "thiscall" fn(*mut u8, u32) = unsafe { std::mem::transmute(REAL.load(Ordering::Acquire)) };
        unsafe { real(this, ticks) };
        let (now, freq) = counter();
        let last = LAST.swap(now, Ordering::AcqRel);
        crate::hooks::guard_void("clock", || unsafe {
            let paused = this.add(PAUSED).read() != 0;
            let fixed = (this.add(FIXED) as *const f32).read_unaligned() != 0.0;
            if paused || fixed || last == 0 {
                return;
            }
            let scale = crate::mem::at::<f32>(TIME_SCALE).unwrap_or(1.0);
            (this.add(FRAME_TIME) as *mut f32).write_unaligned(super::frame_seconds(now - last, freq) * scale);
        });
    }

    /// Times the game's frames by the performance counter.
    pub fn install() {
        if crate::hooks::install_into(UPDATE as *const (), update as *const (), "TimeGlobal::Update", &REAL) {
            tracing::info!("clock: the game's frames are timed to the microsecond");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_its_real_time_within_the_games_cap() {
        assert!((frame_seconds(9_000, 1_000_000) - 0.009).abs() < 1e-7);
        assert!((frame_seconds(5_000_000, 1_000_000) - MAX_FRAME as f32).abs() < 1e-7, "a hitch is the game's longest frame");
        assert!(frame_seconds(0, 1_000_000) > 0.0, "never no time at all");
    }
}
