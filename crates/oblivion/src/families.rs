//! Testing, `RUSTVR_OBLIVION_LEAVE_OUT=1` (with `RUSTVR_OBLIVION_EYE_PNG`): which of the game's kinds
//! of draw a fault in the picture is in. Every 5 s one frame's eyes are rendered with one kind left
//! out and saved, named after it (`<n>-<kind>-left.png`), round the kinds in turn; a fault gone from
//! the pictures without one kind and there in the rest is that kind's. Each kind is known by its
//! vertex shader's constants (its constant table, when the game makes it): the far land's
//! (`LODLandFlags`), the grass's (`ScaleMask`), the leaves' (`LeafBase`), the trees' branches and
//! trunks (`WindMatrices`), the distant trees' cards (`EyePos` with `InstanceData`), the shadows'
//! receivers (`ShadowProj`), skinned bodies (`Bones`), the rest, and draws with no vertex shader.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

pub const KINDS: [&str; 10] = ["nothing", "far-land", "grass", "leaves", "branches", "far-trees", "shadowed", "skinned", "other", "fixed-function"];
const NONE: u8 = 0;
const OTHER: u8 = 8;
const FIXED: u8 = 9;

pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTVR_OBLIVION_LEAVE_OUT").is_ok_and(|v| v == "1"))
}

/// The kind of a vertex shader's draws, from its constants' names.
pub fn kind_of(constants: &[(String, u32, u32)]) -> u8 {
    let has = |name: &str| constants.iter().any(|(n, _, _)| n == name);
    if has("LODLandFlags") || has("HighDetailRange") {
        1
    } else if has("ScaleMask") {
        2
    } else if has("LeafBase") {
        3
    } else if has("WindMatrices") {
        4
    } else if has("InstanceData") && has("EyePos") {
        5
    } else if has("ShadowProj") {
        6
    } else if has("Bones") {
        7
    } else {
        OTHER
    }
}

static SHADERS: Mutex<Option<HashMap<usize, u8>>> = Mutex::new(None);
/// The bound vertex shader's kind, and the kind left out of this frame (0: none).
static BOUND: AtomicU8 = AtomicU8::new(FIXED);
static LEFT_OUT: AtomicU8 = AtomicU8::new(NONE);

/// The game made a vertex shader (`shader`) from `tokens`.
pub fn created(shader: usize, tokens: &[u32]) {
    let kind = kind_of(&crate::leaves::constants(tokens));
    SHADERS.lock().unwrap().get_or_insert_with(HashMap::new).insert(shader, kind);
}

/// The game binds a vertex shader (0: none, the fixed function).
pub fn bind(shader: usize) {
    let kind = if shader == 0 { FIXED } else { SHADERS.lock().unwrap().as_ref().and_then(|m| m.get(&shader).copied()).unwrap_or(OTHER) };
    BOUND.store(kind, Ordering::Release);
}

/// This frame's renders leave out `kind` (an index into [`KINDS`]; 0 none).
pub fn leave_out(kind: u8) {
    LEFT_OUT.store(kind, Ordering::Release);
}

/// The draw about to be made is of the kind left out.
pub fn skip() -> bool {
    let left = LEFT_OUT.load(Ordering::Acquire);
    left != NONE && left == BOUND.load(Ordering::Acquire)
}
