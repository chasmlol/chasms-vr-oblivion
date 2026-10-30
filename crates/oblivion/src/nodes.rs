//! Walking the game's scene graph: Gamebryo's nodes by name, and what class an object is (from its
//! vtable's RTTI, as the game's own dynamic casts read it).
//!
//! `NiObjectNET`: its name at +8, its first controller at +0xC. `NiAVObject`: its parent at +0x1C,
//! its flags at +0x18 (`APP_CULLED` bit 0). `NiNode`: its children, an `NiTArray` (data +0xB0, the
//! slots in use up to +0xB6).

use crate::engine::av;
use crate::mem;
use glam::Vec3;

pub const NAME: usize = 0x08;
pub const CONTROLLER: usize = 0x0C;
pub const PARENT: usize = 0x1C;
pub const CHILDREN: usize = 0xB0;
pub const CHILD_SLOTS: usize = 0xB6;

/// A node's name ("" for none).
pub fn name(obj: usize) -> String {
    let p = mem::deref(obj + NAME);
    if p == 0 { String::new() } else { crate::combat::read_str(p) }
}

/// The class of a C++ object of the game's (its RTTI type name, `NiNode`), or "".
pub fn class(obj: usize) -> String {
    // The vtable's slot -1 is the complete object locator; its type descriptor's name (+8) is
    // the decorated name (`.?AVNiNode@@`).
    let vt = mem::deref(obj);
    let col = if vt != 0 { mem::deref(vt - 4) } else { 0 };
    let td = if col != 0 { mem::deref(col + 0x0C) } else { 0 };
    if td == 0 {
        return String::new();
    }
    let s = crate::combat::read_str(td + 8);
    s.strip_prefix(".?AV").and_then(|s| s.strip_suffix("@@")).unwrap_or(&s).to_owned()
}

/// A node's children (none for what isn't a node).
pub fn children(node: usize) -> Vec<usize> {
    let data = mem::deref(node + CHILDREN);
    let n = mem::at::<u16>(node + CHILD_SLOTS).unwrap_or(0).min(256) as usize;
    let c = class(node);
    if data == 0 || !(c.contains("Node") || c == "SceneGraph") {
        return Vec::new();
    }
    (0..n).map(|i| mem::deref(data + i * 4)).filter(|&c| c != 0).collect()
}

/// The first node under `root` (or `root`) named `name` (case-insensitive), breadth first.
pub fn find(root: usize, name_: &str) -> Option<usize> {
    let mut queue = std::collections::VecDeque::from([(root, 0u32)]);
    let mut seen = 0;
    while let Some((n, depth)) = queue.pop_front() {
        seen += 1;
        if seen > 4000 {
            break;
        }
        if name(n).eq_ignore_ascii_case(name_) {
            return Some(n);
        }
        if depth < 40 {
            queue.extend(children(n).into_iter().map(|c| (c, depth + 1)));
        }
    }
    None
}

/// A node's world position (game units).
pub fn world_pos(obj: usize) -> Option<Vec3> {
    mem::at::<[f32; 3]>(obj + av::WORLD_TRANSLATE).map(Vec3::from)
}

/// The tree under `root`, `depth` levels deep, one line a node: its class, name, flags, where it
/// is and its first controller's class (for the log, finding the game's facts).
pub fn dump(root: usize, depth: u32) -> String {
    fn walk(n: usize, d: u32, max: u32, out: &mut String, budget: &mut u32) {
        if *budget == 0 {
            return;
        }
        *budget -= 1;
        let ctl = mem::deref(n + CONTROLLER);
        let at = world_pos(n).unwrap_or(Vec3::NAN);
        out.push_str(&format!(
            "\n{:indent$}{} '{}' {:#x} flags {:#06x} at ({:.0}, {:.0}, {:.0}){}",
            "",
            class(n),
            name(n),
            n,
            mem::at::<u16>(n + av::FLAGS).unwrap_or(0),
            at.x,
            at.y,
            at.z,
            if ctl != 0 { format!(" ctl {}", class(ctl)) } else { String::new() },
            indent = (d * 2) as usize
        ));
        if d < max {
            for c in children(n) {
                walk(c, d + 1, max, out, budget);
            }
        }
    }
    let mut out = String::new();
    let mut budget = 400;
    walk(root, 0, depth, &mut out, &mut budget);
    out
}
