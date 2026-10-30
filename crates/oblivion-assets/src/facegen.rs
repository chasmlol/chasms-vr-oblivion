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
//!   controls, each a coefficient vector and an offset: the first, the age from the symmetric
//!   geometry.

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

/// FaceGen's age control from `si.ctl`: the coefficients over the symmetric geometry coordinates,
/// and the offset (the age of the mean face).
pub fn parse_age(data: &[u8]) -> Result<(Vec<f32>, f32)> {
    let mut r = Bytes { b: data, at: 0 };
    ensure!(r.take(8)? == b"FRCTL001", "not a FaceGen control file");
    let _versions = (r.u32()?, r.u32()?);
    let counts = [r.u32()? as usize, r.u32()? as usize, r.u32()? as usize, r.u32()? as usize];
    for n in counts {
        let controls = r.u32()? as usize;
        for _ in 0..controls {
            r.take(n * 4)?;
            let name = r.u32()? as usize;
            r.take(name)?;
        }
    }
    let gs = counts[0];
    let coefficients = (0..gs).map(|_| r.f32()).collect::<Result<Vec<_>>>()?;
    let offset = r.f32()?;
    if !(0.0..200.0).contains(&offset) {
        bail!("the age control's offset is {offset}: not the layout expected");
    }
    Ok((coefficients, offset))
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
