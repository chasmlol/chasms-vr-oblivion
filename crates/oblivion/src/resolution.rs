//! The size the game renders each eye at: the headset's eye size as the runtime asks for it (its
//! recommended size times its resolution scale, `RtFrame::eye_size`), or the `eye_width` and
//! `eye_height` settings when they're given.
//!
//! The game reads its screen size (`iSize W`/`iSize H`, 0xB06C5C/0xB06C64) into `nWidth` and
//! `nHeight` (0xB06C4C/0xB06C50), makes its window, then makes its renderer at that size (0x498340).
//! [`apply`] runs as the window is made: it waits for the runtime (it may be starting alongside the
//! game) and puts the eye size in all four, so the device, the back buffer and every render target
//! the game makes are that size. The size the runtime gave is kept ([`SAVED`], beside the game), for
//! a game started before the headset is on: the runtime waits for the headset before it knows its
//! size, and the game can't wait for it.

use crate::{mem, settings};
use std::time::{Duration, Instant};

/// The last eye size the runtime gave (`<width> <height>`), beside the game.
pub const SAVED: &str = "rustvr-eye-size.txt";
/// How long the game waits for the runtime when it has a size kept from before.
const WAIT_SAVED: Duration = Duration::from_secs(4);
const SETTING_W: usize = 0x00B0_6C5C;
const SETTING_H: usize = 0x00B0_6C64;
const SCREEN_W: usize = 0x00B0_6C4C;
const SCREEN_H: usize = 0x00B0_6C50;
/// How long the game waits for the runtime's eye size before keeping its own.
const WAIT: Duration = Duration::from_secs(20);

/// The runtime's eye size, waiting up to `wait` for the runtime to publish it.
pub fn runtime_eye_size(wait: Duration) -> Option<(u32, u32)> {
    let t0 = Instant::now();
    loop {
        if let Ok(link) = rustvr_protocol::link::Link::open(rustvr_protocol::DEFAULT_MAPPING)
            && let Some(f) = link.read_rt_frame()
            && f.eye_size[0] >= 16
            && f.eye_size[1] >= 16
        {
            return Some((f.eye_size[0], f.eye_size[1]));
        }
        if t0.elapsed() > wait {
            return None;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// `<width> <height>`, if it's an eye size.
pub fn parse_saved(text: &str) -> Option<(u32, u32)> {
    let mut it = text.split_whitespace().map(|v| v.parse::<u32>().ok());
    let (w, h) = (it.next()??, it.next()??);
    (16..=16384).contains(&w).then_some(())?;
    (16..=16384).contains(&h).then_some((w, h))
}

fn saved_path() -> Option<std::path::PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.join(SAVED))
}

/// The size each eye is rendered at.
pub fn eye_size() -> Option<(u32, u32)> {
    let s = settings::get();
    if s.eye_width > 0 && s.eye_height > 0 {
        return Some((s.eye_width, s.eye_height));
    }
    let saved = saved_path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| parse_saved(&t));
    match runtime_eye_size(if saved.is_some() { WAIT_SAVED } else { WAIT }) {
        Some((w, h)) => {
            // (Tests' emulated headsets aren't the player's: theirs isn't kept.)
            if saved != Some((w, h))
                && !s.background
                && let Some(p) = saved_path()
                && let Err(e) = std::fs::write(&p, format!("{w} {h}\n"))
            {
                tracing::warn!("resolution: keeping the eye size in {}: {e}", p.display());
            }
            Some((w, h))
        }
        None => {
            if let Some((w, h)) = saved {
                tracing::info!("resolution: no eye size from the runtime yet (the headset isn't on?): the last one, {w}x{h}");
            }
            saved
        }
    }
}

/// The game's screen becomes the eye size (as its window is made).
pub fn apply() {
    let before = (mem::at::<u32>(SCREEN_W).unwrap_or(0), mem::at::<u32>(SCREEN_H).unwrap_or(0));
    let Some((w, h)) = eye_size() else {
        tracing::warn!("resolution: no eye size from the runtime; the game renders at its own {}x{}", before.0, before.1);
        return;
    };
    for (at, v) in [(SETTING_W, w), (SETTING_H, h), (SCREEN_W, w), (SCREEN_H, h)] {
        if mem::is_readable(at as *const u8, 4) {
            unsafe { (at as *mut u32).write(v) };
        }
    }
    tracing::info!("resolution: the game renders each eye at {w}x{h} (its screen was {}x{})", before.0, before.1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kept_size_reads_back() {
        assert_eq!(parse_saved("4147 4406\n"), Some((4147, 4406)));
        assert_eq!(parse_saved("4147"), None);
        assert_eq!(parse_saved("0 4406"), None);
        assert_eq!(parse_saved("wide tall"), None);
    }
}
