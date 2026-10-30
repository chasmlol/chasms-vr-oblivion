//! Oblivion's interface for the runtime to paint itself: its bitmap fonts (`fonts\*.fnt`, each
//! with its atlas `<name>_0_lod_a.tex`) and its menu art (`textures\menus\...`), for cards made of
//! the game's own pieces (`rustvr_protocol::cards`).
//!
//! A font file is a 296-byte header (its line height, a float, at 0; its atlas's name from 12)
//! and 256 glyphs of 14 floats each: the atlas page, the four corners' texture coordinates (top
//! left, top right, bottom left, bottom right), the glyph's width and height, the space before it
//! and after it along the line, and how far its top is above the baseline. The atlas is two u32
//! sizes and then RGBA (white, the glyph in its alpha).

use anyhow::{Result, bail};
use rustvr_asset::{FORMAT_VERSION, Font, Glyph, TexFormat, Texture};

const HEADER: usize = 0x128;
const GLYPH: usize = 56;

fn f32_at(b: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

/// A font from its `.fnt`, its atlas's texture key `atlas`.
pub fn font(data: &[u8], source: &str, atlas: &str) -> Result<Font> {
    if data.len() < HEADER + 256 * GLYPH {
        bail!("{source}: {} bytes, too short for a font", data.len());
    }
    let height = f32_at(data, 0);
    let glyphs = (0..256)
        .map(|c| {
            let g = |k: usize| f32_at(data, HEADER + c * GLYPH + 4 * k);
            Glyph { uv: [g(1), g(2), g(7), g(8)], width: g(9), height: g(10), before: g(11), after: g(12), top: g(13) }
        })
        .collect();
    Ok(Font { version: FORMAT_VERSION, source: source.to_owned(), height, atlas: atlas.to_owned(), glyphs })
}

/// A font's atlas (`.tex`) as a texture.
pub fn atlas(data: &[u8], source: &str) -> Result<Texture> {
    if data.len() < 8 {
        bail!("{source}: not a font atlas");
    }
    let (w, h) = (u32::from_le_bytes(data[0..4].try_into().unwrap()), u32::from_le_bytes(data[4..8].try_into().unwrap()));
    let n = (w as usize) * (h as usize) * 4;
    let Some(px) = data.get(8..8 + n) else { bail!("{source}: {w} x {h} but short") };
    Ok(Texture { version: FORMAT_VERSION, source: source.to_owned(), width: w, height: h, format: TexFormat::Rgba8, mips: vec![px.to_vec()], faces: 1 })
}

/// The atlas a font's file names: `<font>_0_lod_a.tex` beside it.
pub fn atlas_path(font: &str) -> String {
    format!("{}_0_lod_a.tex", font.trim_end_matches(".fnt"))
}
