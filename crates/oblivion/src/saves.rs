//! A save's picture, read back. The game keeps a picture of the view with each save (256 wide, as
//! tall as the screen's shape makes it) and reads it back for the save and load menus and a load's
//! screen into a square as wide as it (0x45D7xx: the width, then the height, then that many rows
//! copied into `NiPixelData(width, width)`): a flat screen is never taller than wide. A headset's
//! eye is (4485 x 4830 here), so a save made in VR has a picture 256 x 275, and its copy ran 19 rows
//! past the square: the save menu crashed opening, as did a load's screen showing it. Where the game
//! makes the square (its call at 0x45D88D), it's made as tall as the picture when that's taller.

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    /// The call making the picture's pixels (`NiPixelData::NiPixelData(width, height, format,
    /// mipmaps, faces)`, thiscall, given the width twice), and the constructor it calls.
    const MAKE_PIXELS: usize = 0x0045_D88D;
    const PIXEL_DATA: usize = 0x0070_E560;

    /// At [`MAKE_PIXELS`]: the height given is the picture's own (`edi` holds it there) when that's
    /// more than the width, then on to the game's constructor.
    #[unsafe(naked)]
    unsafe extern "C" fn pixels_for_picture() {
        core::arch::naked_asm!(
            "cmp edi, [esp + 8]",
            "jbe 2f",
            "mov [esp + 8], edi",
            "2:",
            "mov eax, {pixel_data}",
            "jmp eax",
            pixel_data = const PIXEL_DATA,
        )
    }

    pub fn install() {
        let expect = {
            let rel = (PIXEL_DATA as i32).wrapping_sub(MAKE_PIXELS as i32 + 5).to_le_bytes();
            [0xE8, rel[0], rel[1], rel[2], rel[3]]
        };
        let rel = (pixels_for_picture as *const () as i32).wrapping_sub(MAKE_PIXELS as i32 + 5).to_le_bytes();
        if crate::mem::patch_code(MAKE_PIXELS, &expect, &[0xE8, rel[0], rel[1], rel[2], rel[3]]) {
            tracing::info!("saves: a save's picture taller than wide (made in VR) is read back into room for it");
        } else {
            tracing::warn!("saves: the game's reading of a save's picture isn't as expected: left alone");
        }
    }
}
