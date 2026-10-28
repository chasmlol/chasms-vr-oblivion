//! A converted model as a picture, to check a conversion by eye: three orthographic views (from
//! the front, +Z; the right side, +X; above, +Y) side by side, front faces grey and lit, back
//! faces red (a model drawn red where it should be grey has its triangles wound the wrong way),
//! its bones as blue lines to their parents.

use anyhow::Result;
use glam::{Mat4, Vec2, Vec3};
use rustvr_asset::Model;

const SIZE: usize = 420;

struct View {
    right: Vec3,
    up: Vec3,
    look: Vec3,
}

const VIEWS: [View; 3] = [
    View { right: Vec3::NEG_X, up: Vec3::Y, look: Vec3::NEG_Z },
    View { right: Vec3::Z, up: Vec3::Y, look: Vec3::NEG_X },
    View { right: Vec3::X, up: Vec3::Z, look: Vec3::NEG_Y },
];

pub fn render(model: &Model) -> Vec<u8> {
    let (lo, hi) = bounds(model);
    let centre = (lo + hi) * 0.5;
    let extent = (hi - lo).max_element().max(1e-3) * 1.1;
    let scale = SIZE as f32 / extent;
    let width = SIZE * VIEWS.len();
    let mut rgb = vec![30u8; width * SIZE * 3];
    let light = Vec3::new(0.4, 0.8, 0.6).normalize();
    for (k, view) in VIEWS.iter().enumerate() {
        let mut depth = vec![f32::MAX; SIZE * SIZE];
        let to_screen = |p: Vec3| {
            let d = p - centre;
            (Vec2::new(d.dot(view.right) * scale + SIZE as f32 * 0.5, SIZE as f32 * 0.5 - d.dot(view.up) * scale), d.dot(view.look))
        };
        for mesh in &model.meshes {
            for t in mesh.indices.as_chunks::<3>().0 {
                let v = [0, 1, 2].map(|j| &mesh.vertices[t[j] as usize]);
                let s = v.map(|v| to_screen(Vec3::from_array(v.pos)));
                // Pixel y runs down: a triangle counter-clockwise on screen has a negative area here.
                let area = (s[1].0 - s[0].0).perp_dot(s[2].0 - s[0].0);
                if area.abs() < 1e-6 {
                    continue;
                }
                let front = area < 0.0;
                let n = (Vec3::from_array(v[0].normal) + Vec3::from_array(v[1].normal) + Vec3::from_array(v[2].normal)).normalize_or_zero();
                let lit = 0.25 + 0.75 * n.dot(light).max(0.0);
                let colour = if front { [(200.0 * lit) as u8; 3] } else { [(220.0 * lit) as u8, 40, 40] };
                let (min, max) = (s[0].0.min(s[1].0).min(s[2].0), s[0].0.max(s[1].0).max(s[2].0));
                let (x0, x1) = (min.x.floor().max(0.0) as usize, (max.x.ceil() as usize).min(SIZE - 1));
                let (y0, y1) = (min.y.floor().max(0.0) as usize, (max.y.ceil() as usize).min(SIZE - 1));
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        let w0 = (s[2].0 - s[1].0).perp_dot(p - s[1].0) / area;
                        let w1 = (s[0].0 - s[2].0).perp_dot(p - s[2].0) / area;
                        let w2 = 1.0 - w0 - w1;
                        if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                            continue;
                        }
                        let z = w0 * s[0].1 + w1 * s[1].1 + w2 * s[2].1;
                        let at = y * SIZE + x;
                        if z < depth[at] {
                            depth[at] = z;
                            let o = (y * width + k * SIZE + x) * 3;
                            rgb[o..o + 3].copy_from_slice(&colour);
                        }
                    }
                }
            }
        }
        for b in &model.bones {
            let Some(parent) = usize::try_from(b.parent).ok().and_then(|p| model.bones.get(p)) else { continue };
            let a = to_screen(Mat4::from_cols_array(&b.model_matrix).w_axis.truncate()).0;
            let c = to_screen(Mat4::from_cols_array(&parent.model_matrix).w_axis.truncate()).0;
            let steps = (a - c).abs().max_element().ceil().max(1.0) as usize;
            for i in 0..=steps {
                let p = c.lerp(a, i as f32 / steps as f32);
                if p.x >= 0.0 && p.y >= 0.0 && (p.x as usize) < SIZE && (p.y as usize) < SIZE {
                    let o = (p.y as usize * width + k * SIZE + p.x as usize) * 3;
                    rgb[o..o + 3].copy_from_slice(&[60, 140, 255]);
                }
            }
        }
    }
    rgb
}

fn bounds(model: &Model) -> (Vec3, Vec3) {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let points = model.meshes.iter().flat_map(|m| m.vertices.iter().map(|v| Vec3::from_array(v.pos)));
    let bones = model.bones.iter().map(|b| Mat4::from_cols_array(&b.model_matrix).w_axis.truncate());
    for p in points.chain(bones) {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    if lo.x > hi.x { (Vec3::ZERO, Vec3::ONE) } else { (lo, hi) }
}

pub fn save(model: &Model, path: &std::path::Path) -> Result<()> {
    let rgb = render(model);
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, (SIZE * VIEWS.len()) as u32, SIZE as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&rgb)?;
    Ok(())
}
