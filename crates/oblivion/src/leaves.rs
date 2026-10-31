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
//! up in the log every 10 s, and a value far out of range is logged as it comes. (Playtest 7: all
//! sound. The streaks are the shadow pass's: `docs/research/visuals.md`.)
//!
//! The cards face the camera's `BillboardRight` and `BillboardUp`: square to the way it looks, and
//! leaning with it. A flat screen's camera never leans and turns little; a head leans and turns all
//! the time, and every leaf on every tree turned and leaned with it. So for each draw the cards are
//! given a right that's level and square to the line from the eye to the tree, and the game's up
//! without its lean toward that right ([`facing`]): they turn as you walk round a tree, not as your
//! head turns or leans. The eye in the tree's own space is where its model-view-projection takes
//! x, y and w to 0 ([`eye_in_model`]).

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
    pub mvp: (u32, u32),
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
    Some(Layout { billboard, rock: get("RockParams").unwrap_or_default(), rustle: get("RustleParams").unwrap_or_default(), wind, corners, mvp: get("ModelViewProj").unwrap_or_default() })
}

/// The eye in a model's own space: where its model-view-projection (`row_major`, the rows as set)
/// takes x, y and w to 0, which is the eye for any perspective projection, off-centre or not.
pub fn eye_in_model(mvp: &[f32; 16]) -> Option<[f32; 3]> {
    let row = |i: usize| [mvp[i * 4] as f64, mvp[i * 4 + 1] as f64, mvp[i * 4 + 2] as f64, mvp[i * 4 + 3] as f64];
    let (a, b, c) = (row(0), row(1), row(3));
    let det3 = |x: [f64; 3], y: [f64; 3], z: [f64; 3]| x[0] * (y[1] * z[2] - y[2] * z[1]) - x[1] * (y[0] * z[2] - y[2] * z[0]) + x[2] * (y[0] * z[1] - y[1] * z[0]);
    // (Cramer's rule on the three rows' x, y and z columns.)
    let (cx, cy, cz) = ([a[0], b[0], c[0]], [a[1], b[1], c[1]], [a[2], b[2], c[2]]);
    let rhs = [-a[3], -b[3], -c[3]];
    let det = det3(cx, cy, cz);
    if det.abs() < 1e-12 {
        return None;
    }
    let e = [det3(rhs, cy, cz) / det, det3(cx, rhs, cz) / det, det3(cx, cy, rhs) / det];
    e.iter().all(|v| v.is_finite()).then(|| [e[0] as f32, e[1] as f32, e[2] as f32])
}

/// The leaf cards' right and up for a tree seen from `eye` (its own space, its origin at the trunk,
/// up its +z): right level and square to the line from the eye to the tree, up the game's `up`
/// without its lean toward that right. Each as long as the game's and pointing its way, `w` as the
/// game set it.
pub fn facing(eye: [f32; 3], right: [f32; 4], up: [f32; 4]) -> Option<([f32; 4], [f32; 4])> {
    let len3 = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let (rl, ul) = (len3([right[0], right[1], right[2]]), len3([up[0], up[1], up[2]]));
    if rl < 1e-6 || ul < 1e-6 {
        return None;
    }
    let (dx, dy) = (-eye[0], -eye[1]);
    let d = (dx * dx + dy * dy).sqrt();
    let (mut rx, mut ry) = if d > 1e-3 {
        (-dy / d, dx / d)
    } else {
        // (Right over the trunk: the game's right, level.)
        let h = (right[0] * right[0] + right[1] * right[1]).sqrt();
        if h < 1e-6 {
            return None;
        }
        (right[0] / h, right[1] / h)
    };
    if rx * right[0] + ry * right[1] < 0.0 {
        (rx, ry) = (-rx, -ry);
    }
    let lean = up[0] * rx + up[1] * ry;
    let u = [up[0] - lean * rx, up[1] - lean * ry, up[2]];
    let l = len3(u);
    let u = if l < 1e-6 { [0.0, 0.0, 1.0] } else { [u[0] / l, u[1] / l, u[2] / l] };
    Some(([rx * rl, ry * rl, 0.0, right[3]], [u[0] * ul, u[1] * ul, u[2] * ul, up[3]]))
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
    /// Draws whose cards were turned to the eye, and those that couldn't be.
    pub faced: u64,
    pub unfaced: u64,
}

impl Default for Seen {
    fn default() -> Self {
        Seen { binds: 0, uploads: 0, corner_uploads: (u32::MAX, 0), largest: [(0.0, 0); 5], not_finite: 0, first_bad: None, faced: 0, unfaced: 0 }
    }
}

impl Seen {
    fn summary(&self) -> String {
        let largest: Vec<String> = NAMES.iter().zip(self.largest).map(|(n, (v, r))| format!("{n} {v:.3} (c{r})")).collect();
        let corners = if self.corner_uploads.1 == 0 { "none".to_string() } else { format!("{}..{}", self.corner_uploads.0, self.corner_uploads.1) };
        format!(
            "leaves: {} leaf shaders bound, {} uploads to them; corners uploaded {} at a time; largest: {}; {} values not finite (made 0); {} draws' cards turned to the eye, {} not",
            self.binds,
            self.uploads,
            corners,
            largest.join(", "),
            self.not_finite,
            self.faced,
            self.unfaced
        )
    }
}

/// Testing, `RUSTVR_OBLIVION_LEAVES_AS_GAME=1`: the leaves' constants as the game sets them, unchecked.
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("RUSTVR_OBLIVION_LEAVES_AS_GAME").is_ok_and(|v| v == "1"))
}

/// Testing, `RUSTVR_OBLIVION_GAME_LEAF_FACING=1`: the cards square to the way the head looks, leaning
/// with it, as the game has them.
pub fn faces_eye() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("RUSTVR_OBLIVION_GAME_LEAF_FACING").is_ok_and(|v| v == "1"))
}

struct State {
    shaders: HashMap<usize, Layout>,
    bound: Option<Layout>,
    seen: Seen,
    since: Option<Instant>,
    last_alarm: Option<Instant>,
    /// The game's own model-view-projection, right and up, as last set while a leaf shader was bound.
    mvp: [f32; 16],
    right: [f32; 4],
    up: [f32; 4],
    have: [bool; 3],
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
/// A leaf shader is bound (a quick look before the lock).
static BOUND: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut s = STATE.lock().unwrap();
    f(s.get_or_insert_with(|| State {
        shaders: HashMap::new(),
        bound: None,
        seen: Seen::default(),
        since: None,
        last_alarm: None,
        mvp: [0.0; 16],
        right: [0.0; 4],
        up: [0.0; 4],
        have: [false; 3],
    }))
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
        // The game's own values (the cards are turned at the draw, from these).
        let values = fixed.as_deref().unwrap_or(data);
        for (i, v) in values.chunks_exact(4).enumerate() {
            let reg = start + i as u32;
            if layout.mvp.1 >= 4 && reg >= layout.mvp.0 && reg < layout.mvp.0 + 4 {
                let row = (reg - layout.mvp.0) as usize;
                s.mvp[row * 4..row * 4 + 4].copy_from_slice(v);
                s.have[0] = true;
            } else if layout.billboard.1 >= 2 && reg == layout.billboard.0 {
                s.right.copy_from_slice(v);
                s.have[1] = true;
            } else if layout.billboard.1 >= 2 && reg == layout.billboard.0 + 1 {
                s.up.copy_from_slice(v);
                s.have[2] = true;
            }
        }
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

/// A leaf shader's draw: its cards' right and up turned to the eye, as (first register, the two
/// registers' values to set before it, the game's to put back after it: other shaders use them).
pub fn facing_now() -> Option<(u32, [f32; 8], [f32; 8])> {
    if !faces_eye() {
        return None;
    }
    with(|s| {
        let layout = s.bound?;
        let turned = (s.have == [true; 3] && layout.billboard.1 >= 2)
            .then(|| eye_in_model(&s.mvp))
            .flatten()
            .and_then(|eye| facing(eye, s.right, s.up));
        match turned {
            Some((r, u)) => {
                s.seen.faced += 1;
                let (gr, gu) = (s.right, s.up);
                Some((layout.billboard.0, [r[0], r[1], r[2], r[3], u[0], u[1], u[2], u[3]], [gr[0], gr[1], gr[2], gr[3], gu[0], gu[1], gu[2], gu[3]]))
            }
            None => {
                s.seen.unfaced += 1;
                None
            }
        }
    })
}

/// Testing, `RUSTVR_OBLIVION_GAME_LEAF_INDEX=1`: the vs_1_1 shaders' indices as the game wrote them.
pub fn floors_indices() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("RUSTVR_OBLIVION_GAME_LEAF_INDEX").is_ok_and(|v| v == "1"))
}

/// A vs_1_1 shader's code with each `mov a0.x, v#.c` (an index into the constants, straight from a
/// vertex's data) made to load the whole part of it, or `None` if it has none. The leaves' shaders
/// index their corners by `v3.z`, the cluster's number with a shade in its fraction (`expp r6.y,
/// v3.z` reads it back), and their wind matrices by `v3.y`: written for the drivers of their day,
/// which loaded `a0` rounding down. Direct3D 9's own reference rounds to the nearest, and D3D9On12
/// does: half the leaves took the next cluster's corner, and the last cluster's read past the
/// corners into what another draw had left there, a corner thrown across the view, differently for
/// each eye (the streaks, `docs/research/visuals.md`). So the index is made whole first: `expp
/// r11.y, v#.c` (its fraction), `add r11.x, v#.c, -r11.y`, `mov a0.x, r11.x`. The tokens are the
/// version's, then instructions (the shader's first unused temporary must be r11 or below).
pub fn floor_indices(tokens: &[u32]) -> Option<Vec<u32>> {
    const VS_1_1: u32 = 0xFFFE_0101;
    const END: u32 = 0x0000_FFFF;
    const COMMENT: u32 = 0xFFFE;
    const DEF: u32 = 0x51;
    const MOV: u32 = 0x01;
    const ADD: u32 = 0x02;
    const EXPP: u32 = 0x4E;
    const ADDR: u32 = 3;
    const INPUT: u32 = 1;
    const TEMP: u32 = 0;
    let kind = |p: u32| ((p >> 28) & 7) | (((p >> 11) & 3) << 3);
    if tokens.first() != Some(&VS_1_1) {
        return None;
    }
    // The instructions: (start, end) of each, and the temporaries they use.
    let mut instructions = Vec::new();
    let mut highest_temp = 0;
    let mut i = 1;
    loop {
        let t = *tokens.get(i)?;
        if t == END {
            break;
        }
        let op = t & 0xFFFF;
        let end = if op == COMMENT {
            i + 1 + ((t >> 16) & 0x7FFF) as usize
        } else if op == DEF {
            i + 6
        } else {
            let mut j = i + 1;
            while tokens.get(j).is_some_and(|p| p & 0x8000_0000 != 0) {
                j += 1;
            }
            for &p in &tokens[i + 1..j] {
                if kind(p) == TEMP {
                    highest_temp = highest_temp.max(p & 0x7FF);
                }
            }
            j
        };
        instructions.push((i, end, op));
        i = end;
    }
    let temp = 11;
    if highest_temp >= temp {
        return None;
    }
    let mut out = vec![tokens[0]];
    let mut changed = false;
    for &(start, end, op) in &instructions {
        let params = &tokens[start + 1..end];
        if op == MOV && params.len() == 2 && kind(params[0]) == ADDR && kind(params[1]) == INPUT {
            let src = params[1];
            let reg = |mask: u32| 0x8000_0000 | (mask << 16) | temp;
            out.extend([EXPP, reg(0b0010), src]);
            // (-r11.yyyy: the negate modifier, the swizzle y in each of the four.)
            out.extend([ADD, reg(0b0001), src, 0x8000_0000 | (1 << 24) | (0x55 << 16) | temp]);
            out.extend([MOV, params[0], 0x8000_0000 | temp]);
            changed = true;
        } else {
            out.extend_from_slice(&tokens[start..end]);
        }
    }
    out.push(END);
    changed.then_some(out)
}
