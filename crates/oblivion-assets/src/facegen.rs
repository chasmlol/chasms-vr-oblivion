//! FaceGen's files, as Oblivion's heads have them (docs/research/mirror.md): a head part's geometry
//! morphs (`.egm`, `FREGM002`), the head's texture morphs (`.egt`, `FREGT003`) and the face's
//! controls (`FaceGen\si.ctl`, `FRCTL001`), whose first linear control is the face's age.
//!
//! * `.egm`: a 64-byte header (the magic, the vertex count, the symmetric and asymmetric morph
//!   counts, the basis version), then each morph: its scale (f32) and an x, y, z (i16) per vertex.
//!   The vertices are the model's, in its order, then those its `.tri`'s expressions add.
//! * `.egt`: a 64-byte header (the magic, the image's rows and columns, the symmetric and
//!   asymmetric morph counts, the basis version), then each morph: its scale (f32) and the red,
//!   green and blue planes (i8, rows × columns each).
//! * `.ctl`: the header (the magic, two basis versions, the four morph counts), four lists of named
//!   linear controls (a count, then each control's coefficients and its name), then FaceGen's own
//!   controls, each a coefficient vector and an offset: the age and the gender, over the symmetric
//!   geometry and over the symmetric texture.

use anyhow::{Result, bail, ensure};

/// A part's geometry morphs: each morph's scale and its integer move of every vertex.
pub struct Egm {
    pub vertices: usize,
    pub symmetric: Vec<(f32, Vec<[i16; 3]>)>,
    pub asymmetric: Vec<(f32, Vec<[i16; 3]>)>,
}

/// A head's texture morphs.
pub struct Egt {
    pub rows: usize,
    pub cols: usize,
    /// Each symmetric morph's scale and its planes (r, g, b, each rows × columns, as stored).
    pub symmetric: Vec<(f32, Vec<i8>)>,
}

struct Bytes<'a> {
    b: &'a [u8],
    at: usize,
}

impl Bytes<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        ensure!(self.at + n <= self.b.len(), "read past the end at {:#x} (+{n})", self.at);
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into()?))
    }
}

pub fn parse_egm(data: &[u8]) -> Result<Egm> {
    let mut r = Bytes { b: data, at: 0 };
    ensure!(r.take(8)? == b"FREGM002", "not a FaceGen geometry morph file");
    let (vertices, nsym, nasym) = (r.u32()? as usize, r.u32()? as usize, r.u32()? as usize);
    r.at = 64;
    let mut read = |n: usize| -> Result<Vec<(f32, Vec<[i16; 3]>)>> {
        (0..n)
            .map(|_| {
                let scale = r.f32()?;
                let raw = r.take(vertices * 6)?;
                let d = raw.chunks_exact(6).map(|c| [0, 2, 4].map(|k| i16::from_le_bytes([c[k], c[k + 1]]))).collect();
                Ok((scale, d))
            })
            .collect()
    };
    let symmetric = read(nsym)?;
    let asymmetric = read(nasym)?;
    Ok(Egm { vertices, symmetric, asymmetric })
}

pub fn parse_egt(data: &[u8]) -> Result<Egt> {
    let mut r = Bytes { b: data, at: 0 };
    ensure!(r.take(8)? == b"FREGT003", "not a FaceGen texture morph file");
    let (rows, cols, nsym) = (r.u32()? as usize, r.u32()? as usize, r.u32()? as usize);
    r.at = 64;
    let symmetric = (0..nsym)
        .map(|_| {
            let scale = r.f32()?;
            let planes = r.take(rows * cols * 3)?.iter().map(|&b| b as i8).collect();
            Ok((scale, planes))
        })
        .collect::<Result<_>>()?;
    Ok(Egt { rows, cols, symmetric })
}

/// FaceGen's controls from `si.ctl`: its named controls over the symmetric geometry (the first of
/// the four lists: the race menu's face sliders, each named and a unit direction), then its own
/// linear controls, each the coefficients over a set of coordinates and an offset: the age and the
/// gender, over the symmetric geometry and then the symmetric texture (in that order: age geometry,
/// age texture, gender geometry, gender texture, as the game's 0x6EDD30 indexes them).
pub fn parse_controls(data: &[u8]) -> Result<rustvr_asset::FaceControls> {
    let mut r = Bytes { b: data, at: 0 };
    ensure!(r.take(8)? == b"FRCTL001", "not a FaceGen control file");
    let _versions = (r.u32()?, r.u32()?);
    let counts = [r.u32()? as usize, r.u32()? as usize, r.u32()? as usize, r.u32()? as usize];
    let mut named = Vec::new();
    for (list, n) in counts.into_iter().enumerate() {
        let controls = r.u32()? as usize;
        for _ in 0..controls {
            let coefficients = (0..n).map(|_| r.f32()).collect::<Result<Vec<_>>>()?;
            let len = r.u32()? as usize;
            let name = String::from_utf8_lossy(r.take(len)?).trim_end_matches('\0').to_owned();
            if list == 0 {
                named.push(rustvr_asset::FaceControl { name, coefficients });
            }
        }
    }
    let (gs, ts) = (counts[0], counts[2]);
    let mut linear = |n: usize| -> Result<(Vec<f32>, f32)> { Ok(((0..n).map(|_| r.f32()).collect::<Result<Vec<_>>>()?, r.f32()?)) };
    let (age, age_offset) = linear(gs)?;
    let _age_texture = linear(ts)?;
    let (gender, gender_offset) = linear(gs)?;
    if !(0.0..200.0).contains(&age_offset) || !(-10.0..10.0).contains(&gender_offset) {
        bail!("the age and gender controls' offsets are {age_offset} and {gender_offset}: not the layout expected");
    }
    Ok(rustvr_asset::FaceControls { age, age_offset, gender, gender_offset, controls: named })
}

/// An `.egt` morph's planes with their rows from the top of the texture: FaceGen stores its images
/// from the bottom row up (its morphs' features line up with the head texture's only so: the
/// nostrils, eyes and lips).
pub fn top_first(planes: &[i8], rows: usize, cols: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(planes.len());
    for plane in planes.chunks_exact(rows * cols) {
        for row in plane.chunks_exact(cols).rev() {
            out.extend_from_slice(row);
        }
    }
    out
}
