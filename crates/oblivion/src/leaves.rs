//! The trees' leaves (SpeedTree's leaf cards, `STLEAF000..003.vso`), watched for the streaks seen
//! across the view at one place by Weye's lake (`docs/research/visuals.md`): thin lines of leaf
//! texture from trees on the far shore to the edge of the picture, in one eye or both.
//!
//! The shader puts each leaf corner at the leaf's position plus `LeafBase[cluster] * size`, turned
//! by the tree's rocking (`RockParams`, `RustleParams`) and laid along the camera's `BillboardRight`
//! and `BillboardUp`, then moves it toward `WindMatrices[n]` times that point as far as the leaf's
//! wind weight says. A corner thrown across the view needs one of those constants not finite or far
//! out of range, or a cluster index past the corners the game uploaded for the tree.
//!
//! So while a leaf shader is bound, the constants the game sets are checked. A value that isn't
//! finite is made 0: on Direct3D 9 a leaf with no wind weight multiplied a broken wind matrix away
//! (0 times anything is 0 there), where D3D9On12 follows IEEE and gets NaN. What they hold is summed
//! up in the log every 10 s, and a value far out of range is logged as it comes.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// What moves a leaf shader's corners: (first register, registers) of each.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    pub billboard: (u32, u32),
    pub rock: (u32, u32),
    pub rustle: (u32, u32),
    pub wind: (u32, u32),
    pub corners: (u32, u32),
}

const NAMES: [&str; 5] = ["billboard", "rocking", "rustle", "wind", "corners"];

impl Layout {
    fn ranges(&self) -> [(u32, u32); 5] {
        [self.billboard, self.rock, self.rustle, self.wind, self.corners]
    }

    /// Which of them register `r` is in.
    fn which(&self, r: u32) -> Option<usize> {
        self.ranges().iter().position(|&(start, n)| n > 0 && r >= start && r < start + n)
    }
}

/// A vertex shader's float constants (name, first register, registers) from its constant table: the
/// first comment after the version token, `CTAB` and a `D3DXSHADER_CONSTANTTABLE`.
pub fn constants(tokens: &[u32]) -> Vec<(String, u32, u32)> {
    let [_, comment, ..] = tokens else { return Vec::new() };
    let len = (comment >> 16) as usize;
    if comment & 0xFFFF != 0xFFFE || tokens.len() < 2 + len || len < 8 || tokens[2] != u32::from_le_bytes(*b"CTAB") {
        return Vec::new();
    }
    let table: Vec<u8> = tokens[3..2 + len].iter().flat_map(|t| t.to_le_bytes()).collect();
    let u32_at = |o: usize| table.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let u16_at = |o: usize| table.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let (Some(count), Some(info)) = (u32_at(12), u32_at(16)) else { return Vec::new() };
    let mut out = Vec::new();
    for i in 0..count.min(256) as usize {
        let at = info as usize + i * 20;
        let (Some(name), Some(set), Some(index), Some(n)) = (u32_at(at), u16_at(at + 4), u16_at(at + 6), u16_at(at + 8)) else { break };
        // D3DXRS_FLOAT4.
        if set != 2 {
            continue;
        }
        let name = table.get(name as usize..).unwrap_or_default();
        let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        out.push((String::from_utf8_lossy(&name[..end]).into_owned(), index as u32, n as u32));
    }
    out
}

/// The layout of a leaf shader (one with `LeafBase` and `WindMatrices`), from its code's tokens.
pub fn leaf_layout(tokens: &[u32]) -> Option<Layout> {
    let c = constants(tokens);
    let get = |name: &str| c.iter().find(|(n, _, _)| n == name).map(|&(_, r, n)| (r, n));
    let corners = get("LeafBase")?;
    let wind = get("WindMatrices")?;
    let (right, up) = (get("BillboardRight").unwrap_or_default(), get("BillboardUp").unwrap_or_default());
    // (Right and up are next to each other in every leaf shader; one range covers both.)
    let billboard = if up.0 == right.0 + right.1 { (right.0, right.1 + up.1) } else { right };
    Some(Layout { billboard, rock: get("RockParams").unwrap_or_default(), rustle: get("RustleParams").unwrap_or_default(), wind, corners })
}

/// A value far out of range for any of them: leaf sizes are tens of units, the rest angles,
/// directions and rotations.
const FAR: f32 = 1.0e5;

/// The values of one upload from `start`, checked against `layout`: a copy with each value that
/// isn't finite made 0 if there is one, and what was seen.
pub fn check(layout: &Layout, start: u32, data: &[f32], seen: &mut Seen) -> Option<Vec<f32>> {
    let mut fixed: Option<Vec<f32>> = None;
    for (i, &v) in data.iter().enumerate() {
        let reg = start + (i / 4) as u32;
        let Some(which) = layout.which(reg) else { continue };
        if !v.is_finite() {
            fixed.get_or_insert_with(|| data.to_vec())[i] = 0.0;
            seen.not_finite += 1;
            seen.first_bad.get_or_insert((which, reg, v));
        } else if v.abs() > seen.largest[which].0 {
            seen.largest[which] = (v.abs(), reg);
        }
    }
    if start == layout.corners.0 && data.len() >= 4 {
        let n = (data.len() / 4) as u32;
        seen.corner_uploads = (seen.corner_uploads.0.min(n), seen.corner_uploads.1.max(n));
    }
    fixed
}

/// What the leaf constants held since the last summary.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    pub binds: u64,
    pub uploads: u64,
    /// The fewest and most corners (`LeafBase` registers) uploaded at a time.
    pub corner_uploads: (u32, u32),
    /// The largest value of each (billboard, rocking, rustle, wind, corners) and its register.
    pub largest: [(f32, u32); 5],
    pub not_finite: u64,
    pub first_bad: Option<(usize, u32, f32)>,
}

impl Default for Seen {
    fn default() -> Self {
        Seen { binds: 0, uploads: 0, corner_uploads: (u32::MAX, 0), largest: [(0.0, 0); 5], not_finite: 0, first_bad: None }
    }
}

impl Seen {
    fn summary(&self) -> String {
        let largest: Vec<String> = NAMES.iter().zip(self.largest).map(|(n, (v, r))| format!("{n} {v:.3} (c{r})")).collect();
        let corners = if self.corner_uploads.1 == 0 { "none".to_string() } else { format!("{}..{}", self.corner_uploads.0, self.corner_uploads.1) };
        format!(
            "leaves: {} leaf shaders bound, {} uploads to them; corners uploaded {} at a time; largest: {}; {} values not finite (made 0)",
            self.binds,
            self.uploads,
            corners,
            largest.join(", "),
            self.not_finite
        )
    }
}

/// Testing, `RUSTVR_OBLIVION_LEAVES_AS_GAME=1`: the leaves' constants as the game sets them, unchecked.
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("RUSTVR_OBLIVION_LEAVES_AS_GAME").is_ok_and(|v| v == "1"))
}

struct State {
    shaders: HashMap<usize, Layout>,
    bound: Option<Layout>,
    seen: Seen,
    since: Option<Instant>,
    last_alarm: Option<Instant>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
/// A leaf shader is bound (a quick look before the lock).
static BOUND: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut s = STATE.lock().unwrap();
    f(s.get_or_insert_with(|| State { shaders: HashMap::new(), bound: None, seen: Seen::default(), since: None, last_alarm: None }))
}

/// The game made a vertex shader (`shader`) from `tokens`: one of the leaves' is remembered.
pub fn created(shader: usize, tokens: &[u32]) {
    if let Some(layout) = leaf_layout(tokens) {
        with(|s| s.shaders.insert(shader, layout));
    }
}

/// The game binds a vertex shader.
pub fn bind(shader: usize) {
    with(|s| {
        s.bound = s.shaders.get(&shader).copied();
        if s.bound.is_some() {
            s.seen.binds += 1;
        }
        BOUND.store(s.bound.is_some(), Ordering::Release);
    });
}

pub fn bound() -> bool {
    BOUND.load(Ordering::Acquire)
}

/// The game sets constants from `start` while a leaf shader is bound: the copy to set instead, if
/// one wasn't finite.
pub fn set(start: u32, data: &[f32]) -> Option<Vec<f32>> {
    with(|s| {
        let layout = s.bound?;
        s.seen.uploads += 1;
        let now = Instant::now();
        let before = s.seen.first_bad;
        let far_before = s.seen.largest;
        let fixed = check(&layout, start, data, &mut s.seen);
        // As they come, at most once a second: a value that isn't finite, or one far out of range.
        let alarm = s.last_alarm.is_none_or(|t| now - t > Duration::from_secs(1));
        if alarm {
            if s.seen.first_bad != before
                && let Some((which, reg, v)) = s.seen.first_bad
            {
                tracing::warn!("leaves: {} c{reg} is {v} (not finite): made 0", NAMES[which]);
                s.last_alarm = Some(now);
            } else if let Some(i) = (0..5).find(|&i| s.seen.largest[i].0 > FAR && far_before[i].0 <= FAR) {
                let (v, reg) = s.seen.largest[i];
                tracing::warn!("leaves: {} c{reg} is {v} (far out of range)", NAMES[i]);
                s.last_alarm = Some(now);
            }
        }
        let since = *s.since.get_or_insert(now);
        if now - since > Duration::from_secs(10) {
            tracing::info!("{}", s.seen.summary());
            s.seen = Seen::default();
            s.since = Some(now);
        }
        fixed
    })
}
