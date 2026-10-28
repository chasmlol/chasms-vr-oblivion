//! Oblivion's textures: DirectDraw Surface files, mostly DXT1/3/5 (kept compressed, as BC1/2/3) and
//! a few uncompressed 32- or 24-bit ones (made RGBA).

use anyhow::{Result, bail};
use rustvr_asset::{FORMAT_VERSION, TexFormat, Texture};

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

pub fn parse(data: &[u8], source: &str) -> Result<Texture> {
    if data.len() < 128 || &data[0..4] != b"DDS " {
        bail!("{source}: not a DDS file");
    }
    let height = u32_at(data, 12);
    let width = u32_at(data, 16);
    let mips = u32_at(data, 28).max(1);
    let pf_flags = u32_at(data, 80);
    let fourcc = &data[84..88];
    let bits = u32_at(data, 88);
    let masks = [u32_at(data, 92), u32_at(data, 96), u32_at(data, 100), u32_at(data, 104)];
    let caps2 = u32_at(data, 112);
    let faces = if caps2 & 0x200 != 0 { 6 } else { 1 };
    const FOURCC: u32 = 0x4;
    let (format, block) = if pf_flags & FOURCC != 0 {
        match fourcc {
            b"DXT1" => (TexFormat::Bc1, 8),
            b"DXT2" | b"DXT3" => (TexFormat::Bc2, 16),
            b"DXT4" | b"DXT5" => (TexFormat::Bc3, 16),
            _ => bail!("{source}: DDS format {:?}", String::from_utf8_lossy(fourcc)),
        }
    } else if bits == 32 || bits == 24 {
        (TexFormat::Rgba8, 0)
    } else {
        bail!("{source}: {bits}-bit uncompressed DDS");
    };
    let mut at = 128;
    let mut out = Vec::new();
    for _face in 0..faces {
        let (mut w, mut h) = (width, height);
        for _ in 0..mips {
            let level = if block > 0 {
                let n = (w.div_ceil(4).max(1) * h.div_ceil(4).max(1) * block) as usize;
                let Some(bytes) = data.get(at..at + n) else { break };
                at += n;
                bytes.to_vec()
            } else {
                let px = (bits / 8) as usize;
                let n = (w * h) as usize * px;
                let Some(bytes) = data.get(at..at + n) else { break };
                at += n;
                let mut rgba = Vec::with_capacity((w * h * 4) as usize);
                for p in bytes.chunks_exact(px) {
                    let v = p.iter().rev().fold(0u32, |a, &b| (a << 8) | b as u32);
                    let ch = |m: u32| if m == 0 { 255 } else { ((v & m) >> m.trailing_zeros()) as u8 };
                    rgba.extend_from_slice(&[ch(masks[0]), ch(masks[1]), ch(masks[2]), if masks[3] == 0 { 255 } else { ch(masks[3]) }]);
                }
                rgba
            };
            out.push(level);
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
    }
    if out.is_empty() {
        bail!("{source}: no image data");
    }
    Ok(Texture { version: FORMAT_VERSION, source: source.to_owned(), width, height, format, mips: out, faces })
}
