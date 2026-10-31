//! The game's vertex buffers in system memory, and D3D9On12. Direct3D 9's drivers read a buffer of
//! `D3DPOOL_SYSTEMMEM` whole; D3D9On12 copies it to the GPU for each draw, and only the vertices a
//! `DrawIndexedPrimitive`'s hints name (`MinVertexIndex + NumVertices` from `BaseVertexIndex`;
//! `InputAssembly::UploadDeferredInputBufferData`, github.com/microsoft/D3D9On12). A draw whose
//! indices reach past its hints, which the drivers never minded, reads stale upload memory past
//! them: triangles stretched to wherever that points, like the streaks across the trees
//! (`docs/research/visuals.md`). Its loop over the other streams also steps twice a pass, so a
//! system memory buffer on stream 2, 4 or 6 is never copied.
//!
//! So the buffers the game makes in system memory are known, and so is which is bound to which
//! stream. An indexed draw from one has its hints widened to the whole buffer (what Direct3D 9
//! read), up to [`MAX_WIDEN`] vertices. What was seen is summed up in the log every 10 s.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The most vertices a draw's hints are widened to (a bigger buffer is left as the game draws it).
pub const MAX_WIDEN: u32 = 1 << 18;
const STREAMS: usize = 16;

/// A draw's vertex hints, widened to the whole of the buffers it reads: (min index, vertices), or
/// `None` if they already cover them (or the buffers are too big to copy each draw). `vertices` is
/// how many vertices each bound system memory stream has from its offset.
pub fn widen(base: i32, min: u32, count: u32, vertices: &[u32]) -> Option<(u32, u32)> {
    let have = *vertices.iter().min()?;
    let base = u32::try_from(base).ok()?;
    let all = have.checked_sub(base)?;
    if min + count >= all || all > MAX_WIDEN {
        return None;
    }
    Some((0, all))
}

#[derive(Default)]
struct Seen {
    created: [u64; 4],
    draws: u64,
    short: u64,
    widened: u64,
    even_streams: u64,
    largest_short: u32,
}

struct State {
    /// The game's system memory vertex buffers and their sizes in bytes.
    buffers: HashMap<usize, u32>,
    /// Each stream's bound system memory buffer: its vertices from the offset.
    streams: [Option<u32>; STREAMS],
    seen: Seen,
    since: Option<Instant>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
/// A system memory buffer is bound to some stream (a quick look before the lock).
static BOUND: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut s = STATE.lock().unwrap();
    f(s.get_or_insert_with(|| State { buffers: HashMap::new(), streams: [None; STREAMS], seen: Seen::default(), since: None }))
}

/// Testing, `RUSTVR_OBLIVION_DRAWS_AS_GAME=1`: the draws' hints as the game gives them.
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("RUSTVR_OBLIVION_DRAWS_AS_GAME").is_ok_and(|v| v == "1"))
}

/// The game made a vertex buffer of `length` bytes in `pool` (`D3DPOOL`: 0 default, 1 managed, 2
/// system memory, 3 scratch).
pub fn created(buffer: usize, length: u32, pool: u32) {
    with(|s| {
        if let Some(n) = s.seen.created.get_mut(pool as usize) {
            *n += 1;
        }
        if pool == 2 {
            s.buffers.insert(buffer, length);
        }
    });
}

/// The game binds `buffer` to `stream` from `offset` bytes, `stride` bytes a vertex.
pub fn bind(stream: u32, buffer: usize, offset: u32, stride: u32) {
    with(|s| {
        let Some(slot) = s.streams.get_mut(stream as usize) else { return };
        *slot = s.buffers.get(&buffer).filter(|_| stride > 0).map(|&len| len.saturating_sub(offset) / stride);
        BOUND.store(s.streams.iter().any(Option::is_some), Ordering::Release);
    });
}

pub fn bound() -> bool {
    BOUND.load(Ordering::Acquire)
}

/// An indexed draw while a system memory buffer is bound: its hints to use instead, if widened.
pub fn draw(base: i32, min: u32, count: u32) -> Option<(u32, u32)> {
    with(|s| {
        let vertices: Vec<u32> = s.streams.iter().flatten().copied().collect();
        s.seen.draws += 1;
        if s.streams.iter().enumerate().any(|(i, v)| v.is_some() && i > 0 && i % 2 == 0) {
            s.seen.even_streams += 1;
        }
        let short = vertices.iter().min().zip(u32::try_from(base).ok()).and_then(|(&have, base)| have.checked_sub(base)).filter(|&all| min + count < all);
        if let Some(all) = short {
            s.seen.short += 1;
            s.seen.largest_short = s.seen.largest_short.max(all - (min + count));
        }
        let wide = widen(base, min, count, &vertices);
        if wide.is_some() {
            s.seen.widened += 1;
        }
        wide
    })
}

/// Once a frame: what was seen, in the log every 10 s.
pub fn tick() {
    with(|s| {
        let now = Instant::now();
        let since = *s.since.get_or_insert(now);
        if now - since > Duration::from_secs(10) {
            let c = s.seen.created;
            tracing::info!(
                "sysmem: vertex buffers made so far: {} default, {} managed, {} system memory, {} scratch; {} draws from system memory, {} with hints short of the buffer (by up to {} vertices; {} widened), {} with one on stream 2, 4 or 6",
                c[0], c[1], c[2], c[3], s.seen.draws, s.seen.short, s.seen.largest_short, s.seen.widened, s.seen.even_streams
            );
            s.seen = Seen { created: c, ..Seen::default() };
            s.since = Some(now);
        }
    });
}
