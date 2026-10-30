//! Oblivion's textures: DirectDraw Surface files, mostly DXT1/3/5 (kept compressed, as BC1/2/3) and
//! a few uncompressed 32-, 24- or 16-bit ones (made RGBA).

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
    } else if bits == 32 || bits == 24 || bits == 16 {
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
                    // Each channel scaled up to 8 bits (a 16-bit file's are 4, 5 or 6).
                    let ch = |m: u32| if m == 0 { 255 } else { (((v & m) >> m.trailing_zeros()) * 255 / (m >> m.trailing_zeros())) as u8 };
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

/// A texture's largest level as 8-bit RGBA, decoded from its blocks (DXT1/3/5) if it has them: for
/// the interface art the runtime paints itself (its cards), which takes RGBA only.
pub fn to_rgba(t: &Texture) -> Result<Texture> {
    let Some(top) = t.mips.first() else { bail!("{}: no image data", t.source) };
    let (w, h) = (t.width as usize, t.height as usize);
    let pixels = match t.format {
        TexFormat::Rgba8 => top.clone(),
        TexFormat::Bc1 | TexFormat::Bc2 | TexFormat::Bc3 => {
            let block = if t.format == TexFormat::Bc1 { 8 } else { 16 };
            let (bw, bh) = (w.div_ceil(4).max(1), h.div_ceil(4).max(1));
            if top.len() < bw * bh * block {
                bail!("{}: short image data", t.source);
            }
            let mut out = vec![0u8; w * h * 4];
            for by in 0..bh {
                for bx in 0..bw {
                    let b = &top[(by * bw + bx) * block..][..block];
                    let (alpha, colour) = if block == 16 { (Some(&b[..8]), &b[8..]) } else { (None, b) };
                    let px = decode_block(t.format, alpha, colour);
                    for (k, p) in px.iter().enumerate() {
                        let (x, y) = (bx * 4 + k % 4, by * 4 + k / 4);
                        if x < w && y < h {
                            out[(y * w + x) * 4..][..4].copy_from_slice(p);
                        }
                    }
                }
            }
            out
        }
        other => bail!("{}: can't decode {other:?}", t.source),
    };
    Ok(Texture { version: FORMAT_VERSION, source: t.source.clone(), width: t.width, height: t.height, format: TexFormat::Rgba8, mips: vec![pixels], faces: 1 })
}

/// One 4 x 4 block's pixels (row by row): its colours (BC1's two ends and the two between, or
/// one between and transparent black), and BC2's explicit or BC3's interpolated alpha.
fn decode_block(format: TexFormat, alpha: Option<&[u8]>, c: &[u8]) -> [[u8; 4]; 16] {
    let c0 = u16::from_le_bytes([c[0], c[1]]);
    let c1 = u16::from_le_bytes([c[2], c[3]]);
    let rgb = |v: u16| [((v >> 11) & 31) as u32 * 255 / 31, ((v >> 5) & 63) as u32 * 255 / 63, (v & 31) as u32 * 255 / 31];
    let (a, b) = (rgb(c0), rgb(c1));
    let mix = |p: u32, q: u32, ka: u32, kb: u32| (p * ka + q * kb) / (ka + kb);
    let four = format != TexFormat::Bc1 || c0 > c1;
    let palette: [[u8; 4]; 4] = [
        [a[0] as u8, a[1] as u8, a[2] as u8, 255],
        [b[0] as u8, b[1] as u8, b[2] as u8, 255],
        if four { [mix(a[0], b[0], 2, 1) as u8, mix(a[1], b[1], 2, 1) as u8, mix(a[2], b[2], 2, 1) as u8, 255] } else { [mix(a[0], b[0], 1, 1) as u8, mix(a[1], b[1], 1, 1) as u8, mix(a[2], b[2], 1, 1) as u8, 255] },
        if four { [mix(a[0], b[0], 1, 2) as u8, mix(a[1], b[1], 1, 2) as u8, mix(a[2], b[2], 1, 2) as u8, 255] } else { [0, 0, 0, 0] },
    ];
    let bits = u32::from_le_bytes([c[4], c[5], c[6], c[7]]);
    let mut out = [[0u8; 4]; 16];
    for (k, p) in out.iter_mut().enumerate() {
        *p = palette[((bits >> (2 * k)) & 3) as usize];
    }
    match (format, alpha) {
        (TexFormat::Bc2, Some(al)) => {
            let v = u64::from_le_bytes(al.try_into().unwrap());
            for (k, p) in out.iter_mut().enumerate() {
                p[3] = (((v >> (4 * k)) & 15) * 17) as u8;
            }
        }
        (TexFormat::Bc3, Some(al)) => {
            let (a0, a1) = (al[0] as u32, al[1] as u32);
            let mut levels = [a0, a1, 0, 0, 0, 0, 0, 0];
            for i in 1..7u32 {
                levels[i as usize + 1] = if a0 > a1 { ((7 - i) * a0 + i * a1) / 7 } else if i < 5 { ((5 - i) * a0 + i * a1) / 5 } else if i == 5 { 0 } else { 255 };
            }
            let v = al[2..8].iter().rev().fold(0u64, |acc, &x| (acc << 8) | x as u64);
            for (k, p) in out.iter_mut().enumerate() {
                p[3] = levels[((v >> (3 * k)) & 7) as usize] as u8;
            }
        }
        _ => {}
    }
    out
}
