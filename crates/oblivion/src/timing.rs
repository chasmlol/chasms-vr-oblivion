//! Where the game's frames go: the time of each part of a frame (the runtime's frame and what the
//! adapter does with it before the eyes, each eye's render, the picture and depth copies, and the
//! rest of the game's frame between renders), averaged and the worst, logged every few seconds
//! with how much of its 32-bit address space the game has left.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The parts of a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Prepare,
    Combat,
    Collision,
    Lighting,
    Eye,
    Capture,
    Depth,
    /// From one render's end to the next one's start: the game's update, its menus, Present.
    Between,
}

const PARTS: [(Part, &str); 8] = [
    (Part::Prepare, "prepare"),
    (Part::Combat, "combat"),
    (Part::Collision, "collision"),
    (Part::Lighting, "lighting"),
    (Part::Eye, "eye renders"),
    (Part::Capture, "picture copies"),
    (Part::Depth, "depth copies"),
    (Part::Between, "between renders"),
];

/// How often the times are logged.
const EVERY: Duration = Duration::from_secs(5);

struct Times {
    since: Instant,
    frames: u32,
    /// Per part: this frame's, the period's total and its worst frame.
    now: [f64; 8],
    total: [f64; 8],
    worst: [f64; 8],
    worst_frame: f64,
    frame_start: Option<Instant>,
    render_end: Option<Instant>,
}

static TIMES: Mutex<Option<Times>> = Mutex::new(None);

fn index(p: Part) -> usize {
    PARTS.iter().position(|(q, _)| *q == p).unwrap_or(0)
}

/// Adds `d` to this frame's `part`.
pub fn add(part: Part, d: Duration) {
    if let Some(t) = TIMES.lock().unwrap().as_mut() {
        t.now[index(part)] += d.as_secs_f64() * 1000.0;
    }
}

/// Times `f` as `part`.
pub fn time<R>(part: Part, f: impl FnOnce() -> R) -> R {
    let t0 = Instant::now();
    let r = f();
    add(part, t0.elapsed());
    r
}

/// A render starts: the time since the last one ended is the rest of the game's frame.
pub fn render_start() {
    // (A new frame: the memory regions found readable are asked about again.)
    crate::mem::forget();
    let mut g = TIMES.lock().unwrap();
    let t = g.get_or_insert_with(|| Times { since: Instant::now(), frames: 0, now: [0.0; 8], total: [0.0; 8], worst: [0.0; 8], worst_frame: 0.0, frame_start: None, render_end: None });
    let now = Instant::now();
    if let Some(end) = t.render_end {
        t.now[index(Part::Between)] += (now - end).as_secs_f64() * 1000.0;
    }
    // The frame before this one is complete: into the period.
    if let Some(start) = t.frame_start {
        let frame = (now - start).as_secs_f64() * 1000.0;
        t.worst_frame = t.worst_frame.max(frame);
        for i in 0..PARTS.len() {
            t.total[i] += t.now[i];
            t.worst[i] = t.worst[i].max(t.now[i]);
        }
        t.frames += 1;
    }
    t.now = [0.0; 8];
    t.frame_start = Some(now);
    if t.since.elapsed() >= EVERY && t.frames > 0 {
        let secs = t.since.elapsed().as_secs_f64();
        let n = t.frames as f64;
        let parts: Vec<String> = PARTS.iter().enumerate().map(|(i, (_, name))| format!("{name} {:.1} (worst {:.1})", t.total[i] / n, t.worst[i])).collect();
        tracing::info!("timing: {:.1} frames/s, worst frame {:.0} ms; ms per frame: {}; {}", n / secs, t.worst_frame, parts.join(", "), memory());
        t.since = Instant::now();
        t.frames = 0;
        t.total = [0.0; 8];
        t.worst = [0.0; 8];
        t.worst_frame = 0.0;
    }
}

/// The game's address space: how much is left of it.
#[cfg(windows)]
fn memory() -> String {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut m = MEMORYSTATUSEX { dwLength: size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    if unsafe { GlobalMemoryStatusEx(&mut m) }.is_err() {
        return String::new();
    }
    let mb = |b: u64| b / (1024 * 1024);
    format!("address space {} of {} MB free", mb(m.ullAvailVirtual), mb(m.ullTotalVirtual))
}

#[cfg(not(windows))]
fn memory() -> String {
    String::new()
}

/// A render ends.
pub fn render_end() {
    if let Some(t) = TIMES.lock().unwrap().as_mut() {
        t.render_end = Some(Instant::now());
    }
}
