//! The flat game's crosshair, gone in VR. Oblivion's reticle (`menus\main\hud_reticle.xml`) is a
//! rect (`hudreticle_reticle`) holding two images: the crosshair (`hudreticle_reticle_crosshair`,
//! `menus\misc\hud_reticle.dds`), shown while the reticle's icon type (its `user5`) is 0, and the
//! icon of what it can activate (`hudreticle_reticle_activate`: a hand, a door, a book; red when
//! taking it is stealing), shown otherwise. In VR the hands aim, and a dot fixed in the middle of
//! the view is only in the way, so the crosshair is hidden; the activate icon stays, as it's the
//! game's only sign of a theft before it's done.
//!
//! The game's own Gameplay setting (`bCrossHair`) isn't used: it hides the whole rect, the icons
//! with it. Instead, before each picture, the crosshair image's `visible` is set to false through
//! the game's own update of a tile's value (`Tile::UpdateFloat`, as the HUD's own code sets the
//! reticle's), whenever the game has made it true again (as the icon type comes back to 0).
//!
//! Tiles (xOBSE's `GameTiles.h`): a tile's name at +8 (`BSStringT`: the characters, their count),
//! its values at +0x14 and its children at +0x30 (lists: +4 the first node; a node is next,
//! previous, the data); a value's number at +4 and its id at +0x18 (u16). `visible` is 0xFA1 (1 no,
//! 2 yes). The HUD's reticle code keeps the `hudreticle_reticle` rect at 0xB3B350 (its child with
//! id 1, found as the HUD loads).

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::mem;

    /// The HUD reticle's `hudreticle_reticle` rect (a `Tile*`).
    const RETICLE_RECT: usize = 0x00B3_B350;
    /// `Tile::UpdateFloat(this, id, value)` (thiscall): sets a value and updates what's drawn from it.
    const UPDATE_FLOAT: usize = 0x0058_CEB0;
    const NAME: usize = 0x08;
    const VALUES: usize = 0x14;
    const CHILDREN: usize = 0x30;
    const LIST_FIRST: usize = 0x4;
    const VALUE_NUM: usize = 0x4;
    const VALUE_ID: usize = 0x18;
    const VISIBLE: u16 = 0xFA1;
    const FALSE: f32 = 1.0;
    const CROSSHAIR: &str = "hudreticle_reticle_crosshair";

    fn list(at: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = mem::deref(at + LIST_FIRST);
        while node != 0 && out.len() < 64 {
            // (A node is next, previous, the data: read at once.)
            let Some([next, _, data]) = mem::at::<[u32; 3]>(node) else { break };
            if data != 0 {
                out.push(data as usize);
            }
            node = next as usize;
        }
        out
    }

    fn name(tile: usize) -> String {
        let (p, n) = (mem::deref(tile + NAME), mem::at::<u16>(tile + NAME + 4).unwrap_or(0) as usize);
        let n = n.min(64);
        if p == 0 || n == 0 || !mem::is_readable(p as *const u8, n) {
            return String::new();
        }
        let bytes = unsafe { std::slice::from_raw_parts(p as *const u8, n) };
        String::from_utf8_lossy(bytes.split(|&b| b == 0).next().unwrap_or(&[])).into_owned()
    }

    fn visible(tile: usize) -> Option<f32> {
        list(tile + VALUES).into_iter().find(|&v| mem::at::<u16>(v + VALUE_ID) == Some(VISIBLE)).and_then(|v| mem::at::<f32>(v + VALUE_NUM))
    }

    /// Before a picture in VR: the game's crosshair hidden, if the game has shown it again.
    pub fn hide_crosshair() {
        let rect = mem::deref(RETICLE_RECT);
        if rect == 0 {
            return;
        }
        let Some(crosshair) = list(rect + CHILDREN).into_iter().find(|&t| name(t) == CROSSHAIR) else { return };
        if visible(crosshair).is_some_and(|v| v != FALSE) {
            type Update = unsafe extern "thiscall" fn(usize, u32, f32);
            unsafe { std::mem::transmute::<usize, Update>(UPDATE_FLOAT)(crosshair, VISIBLE as u32, FALSE) };
            static SAID: std::sync::Once = std::sync::Once::new();
            SAID.call_once(|| tracing::info!("reticle: the game's crosshair hidden (its activate icons stay)"));
        }
    }
}
