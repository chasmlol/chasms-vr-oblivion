//! Oblivion's models: NetImmerse/Gamebryo `.nif` files, version 20.0.0.4 or 20.0.0.5 (user version
//! 11), and the older 10.0.1.0 to 10.2.0.0 that many of the head's parts (hair, ears, eyes, mouth,
//! teeth, tongue) and some older models (Chorrol's doors, the Ayleid switches) are. Those versions
//! store no block sizes, so every block a file holds has to be read field by field: this reader
//! knows every block the game's models use (layouts from the NifTools project's format
//! description, `nif.xml`, with the fields each version has) and stops at the first one it
//! doesn't, keeping the blocks before it.
//!
//! What's kept: the node tree (names, local transforms, children), triangle shapes (vertices,
//! normals, UVs, vertex colours, triangles from lists or strips), their skinning (bones, bind
//! transforms, weights), their textures and materials, extra data, animations, and the Havok
//! collision (rigid bodies: mass, centre, shapes, the triangles of triangle shapes). Particle
//! systems, cameras and most controllers are read past.

use anyhow::{Result, bail};
use glam::{Mat3, Vec3};

#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub rotation: Mat3,
    pub translation: Vec3,
    pub scale: f32,
}

impl Transform {
    pub const IDENTITY: Transform = Transform { rotation: Mat3::IDENTITY, translation: Vec3::ZERO, scale: 1.0 };

    /// `self` after `parent` (child to parent's space).
    pub fn then(&self, parent: &Transform) -> Transform {
        Transform {
            rotation: parent.rotation * self.rotation,
            translation: parent.translation + parent.rotation * (self.translation * parent.scale),
            scale: self.scale * parent.scale,
        }
    }

    pub fn apply(&self, p: Vec3) -> Vec3 {
        self.translation + self.rotation * (p * self.scale)
    }

    pub fn inverse(&self) -> Transform {
        let r = self.rotation.transpose();
        let s = if self.scale != 0.0 { 1.0 / self.scale } else { 1.0 };
        Transform { rotation: r, translation: -(r * self.translation) * s, scale: s }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AvObject {
    pub name: String,
    pub flags: u16,
    pub transform: Transform,
    pub properties: Vec<i32>,
    pub extra: Vec<i32>,
    pub collision: i32,
}

#[derive(Clone, Debug)]
pub enum Block {
    Node { av: AvObject, children: Vec<i32> },
    Shape { av: AvObject, data: i32, skin: i32 },
    ShapeData(ShapeData),
    SkinInstance { data: i32, partition: i32, root: i32, bones: Vec<i32> },
    SkinData { transform: Transform, bones: Vec<BoneData> },
    SkinPartition,
    Texturing(Texturing),
    SourceTexture { file: String },
    Material(MaterialProps),
    Alpha { flags: u16, threshold: u8 },
    StringExtra { name: String, value: String },
    IntegerExtra { name: String, value: u32 },
    BinaryExtra { name: String, data: Vec<u8> },
    /// `NiSpecularProperty`: whether the shape has specular highlights.
    Specular { on: bool },
    CollisionObject { target: i32, body: i32 },
    RigidBody(RigidBody),
    BoxShape { dims: Vec3, radius: f32 },
    CapsuleShape { a: Vec3, b: Vec3, radius: f32 },
    SphereShape { radius: f32 },
    /// `bhkConvexVerticesShape`: its vertices, shell radius and face planes (each the outward
    /// normal and w, with n·p + w = 0 on the face), Havok's units.
    ConvexVertices { vertices: Vec<Vec3>, radius: f32, planes: Vec<[f32; 4]> },
    ListShape { shapes: Vec<i32> },
    ConvexTransform { shape: i32, transform: [f32; 16] },
    /// `bhkPackedNiTriStripsShape`: its parts (each: Havok filter's layer, vertex count, material) and its data block.
    PackedStrips { parts: Vec<PackedPart>, data: i32 },
    /// `hkPackedNiTriStripsData`: triangles (vertex indices) and vertices, in Havok's units.
    PackedStripsData { triangles: Vec<[u16; 3]>, vertices: Vec<Vec3> },
    /// `bhkMoppBvTreeShape`: the shape it's a tree over.
    Mopp { shape: i32 },
    /// `bhkNiTriStripsShape`: its `NiTriStripsData` blocks (each a `Block::ShapeData`). (Also
    /// 10.0.1.0's `bhkMeshShape`, the same.)
    TriStripsShape { strips: Vec<i32> },
    /// `bhkConvexListShape`: its shapes.
    ConvexList { shapes: Vec<i32> },
    /// `bhkMultiSphereShape`: its spheres (centre, radius), Havok's units.
    MultiSphere { spheres: Vec<(Vec3, f32)> },
    /// `NiGeomMorpherController`: shape `target`'s vertices morphed (a bow bending as it's drawn),
    /// by `data` ([`Block::MorphData`]), each morph weighted by one of `interpolators`.
    Morpher { target: i32, data: i32, interpolators: Vec<i32> },
    /// `NiMorphData`: each morph's name and its vertices' offsets from the shape's own (Oblivion's
    /// are all relative: the first, "Base", is all zero).
    MorphData { morphs: Vec<(String, Vec<Vec3>)> },
    /// `NiControllerManager`: the model's named animations (a door's "Open" and "Close", an
    /// activator's "Forward" and "Backward").
    ControllerManager { sequences: Vec<i32> },
    Sequence(Sequence),
    /// `NiStringPalette`: names, each ended by a NUL, that sequences point into by offset.
    StringPalette(String),
    /// `NiTextKeyExtraData`: (time, text) the animation marks (`Hold`, `Sound: ...`).
    TextKeys(Vec<(f32, String)>),
    /// `NiTransformInterpolator`: a node's pose where it has no keys (rotation `[x, y, z, w]`), and
    /// its keys.
    TransformInterpolator { translation: Vec3, rotation: [f32; 4], scale: f32, data: i32 },
    Keyframes(Keyframes),
    /// `NiFloatInterpolator`: its value when it has no keys, and its keys' block.
    FloatInterpolator { value: f32, data: i32 },
    /// `NiFloatData`: its keys, (time, value).
    FloatKeys(Vec<(f32, f32)>),
    /// Read past: of no use here (stencil, vertex colour, z-buffer, wireframe, dither, shade and fog
    /// properties, constraints, particle systems, cameras, furniture markers, most controllers).
    Other(String),
}

/// A part of a `bhkPackedNiTriStripsShape`: its vertices' count (the parts' vertices follow each
/// other in the data block), its Havok filter's layer and its Havok material.
#[derive(Clone, Copy, Debug, Default)]
pub struct PackedPart {
    pub layer: u8,
    pub vertices: u32,
    pub material: u32,
}

/// One of a model's named animations (`NiControllerSequence`): the nodes it moves, each through an
/// interpolator, between its start and stop times.
#[derive(Clone, Debug, Default)]
pub struct Sequence {
    pub name: String,
    /// What it animates: each one's interpolator, and the offsets in `palette` of the node's name,
    /// the controller's type, the controller's id and the interpolator's id.
    pub blocks: Vec<(i32, [u32; 4])>,
    pub palette: i32,
    /// Files before 10.2 name what a sequence animates in the sequence itself: those names, each
    /// ended by a NUL, that `blocks`' offsets point into (`palette` is then -1).
    pub names: String,
    pub start: f32,
    pub stop: f32,
    pub text_keys: i32,
}

/// A node's keys (`NiTransformData`): rotations (quaternions `[x, y, z, w]`, or each axis's angle
/// apart, `NiKeyframeData`'s XYZ rotation) and translations, each `(time, value)`.
#[derive(Clone, Debug, Default)]
pub struct Keyframes {
    pub rotations: Vec<(f32, [f32; 4])>,
    pub xyz: [Vec<(f32, f32)>; 3],
    pub translations: Vec<(f32, Vec3)>,
}

impl Keyframes {
    /// The rotation at `t` (between keys, the shorter way; held before the first and after the
    /// last), or None without rotation keys.
    pub fn rotation_at(&self, t: f32) -> Option<glam::Quat> {
        if !self.rotations.is_empty() {
            let keys: Vec<(f32, glam::Quat)> = self.rotations.iter().map(|&(t, v)| (t, glam::Quat::from_array(v).normalize())).collect();
            return Some(lerp_keys(&keys, t, |a, b, s| a.slerp(b, s)));
        }
        if self.xyz.iter().all(Vec::is_empty) {
            return None;
        }
        let angle = |keys: &Vec<(f32, f32)>| if keys.is_empty() { 0.0 } else { lerp_keys(keys, t, |a, b, s| a + (b - a) * s) };
        let [x, y, z] = [angle(&self.xyz[0]), angle(&self.xyz[1]), angle(&self.xyz[2])];
        Some(glam::Quat::from_rotation_z(z) * glam::Quat::from_rotation_y(y) * glam::Quat::from_rotation_x(x))
    }

    pub fn translation_at(&self, t: f32) -> Option<Vec3> {
        (!self.translations.is_empty()).then(|| lerp_keys(&self.translations, t, |a, b, s| a.lerp(b, s)))
    }

    /// The first and last key's times.
    pub fn span(&self) -> Option<(f32, f32)> {
        let times = self.rotations.iter().map(|k| k.0).chain(self.xyz.iter().flatten().map(|k| k.0)).chain(self.translations.iter().map(|k| k.0));
        times.fold(None, |s, t| Some(s.map_or((t, t), |(a, b): (f32, f32)| (a.min(t), b.max(t)))))
    }
}

/// The name at `offset` in names each ended by a NUL.
fn name_at(names: &str, offset: u32) -> Option<String> {
    let rest = names.as_bytes().get(offset as usize..)?;
    let end = rest.iter().position(|&c| c == 0).unwrap_or(rest.len());
    Some(String::from_utf8_lossy(&rest[..end]).into_owned())
}

/// Keys' value at `t`, blended linearly between the two either side.
fn lerp_keys<T: Copy>(keys: &[(f32, T)], t: f32, mix: impl Fn(T, T, f32) -> T) -> T {
    let i = keys.partition_point(|k| k.0 <= t);
    if i == 0 {
        return keys[0].1;
    }
    if i == keys.len() {
        return keys[keys.len() - 1].1;
    }
    let (a, b) = (keys[i - 1], keys[i]);
    let span = (b.0 - a.0).max(1e-6);
    mix(a.1, b.1, ((t - a.0) / span).clamp(0.0, 1.0))
}

#[derive(Clone, Debug, Default)]
pub struct ShapeData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub colors: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub triangles: Vec<[u16; 3]>,
}

#[derive(Clone, Debug, Default)]
pub struct BoneData {
    pub transform: Transform,
    pub weights: Vec<(u16, f32)>,
}

#[derive(Clone, Debug, Default)]
pub struct Texturing {
    pub base: i32,
    pub glow: i32,
    pub bump: i32,
    pub detail: i32,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialProps {
    pub name: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub emissive: [f32; 3],
    pub glossiness: f32,
    pub alpha: f32,
}

/// A Havok rigid body, in Havok's units (game units / 7).
#[derive(Clone, Debug, Default)]
pub struct RigidBody {
    pub shape: i32,
    pub layer: u8,
    pub translation: [f32; 4],
    /// As Havok stores it: `[x, y, z, w]`.
    pub rotation: [f32; 4],
    pub center: [f32; 4],
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    /// `bhkRigidBodyT`: translation and rotation apply.
    pub transformed: bool,
    /// Havok's motion type (`hkMotionType`: 6 keyframed, moved by the node it's on, as a door's
    /// leaf is by its animation; 7 fixed; the rest simulated).
    pub motion: u8,
}

/// Game units per Havok unit in Oblivion.
pub const HAVOK_SCALE: f32 = 7.0;

pub struct Nif {
    pub version: u32,
    pub blocks: Vec<Block>,
    pub kinds: Vec<String>,
    pub roots: Vec<i32>,
    /// Where reading stopped short, if it did: the block type it didn't know.
    pub stopped_at: Option<String>,
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    /// The file's version (`0x14000004` is 20.0.0.4).
    v: u32,
    /// Bethesda's stream version (from its stream header; 0 without one).
    bs: u32,
}

/// Versions as `nif.xml` writes them.
const V10_0_1_0: u32 = 0x0A00_0100;
const V10_0_1_2: u32 = 0x0A00_0102;
const V10_0_1_3: u32 = 0x0A00_0103;
const V10_1_0_0: u32 = 0x0A01_0000;
const V10_1_0_101: u32 = 0x0A01_0065;
const V10_1_0_103: u32 = 0x0A01_0067;
const V10_1_0_104: u32 = 0x0A01_0068;
const V10_1_0_106: u32 = 0x0A01_006A;
const V10_1_0_108: u32 = 0x0A01_006C;
const V10_1_0_109: u32 = 0x0A01_006D;
const V10_1_0_110: u32 = 0x0A01_006E;
const V10_1_0_112: u32 = 0x0A01_0070;
const V10_1_0_113: u32 = 0x0A01_0071;
const V10_1_0_114: u32 = 0x0A01_0072;
const V10_2_0_0: u32 = 0x0A02_0000;
/// The first without a texture's PS2 fields and a sequence's phase.
const V10_4_0_1: u32 = 0x0A04_0001;
const V20_0_0_2: u32 = 0x1400_0002;
const V20_0_0_4: u32 = 0x1400_0004;
const V20_0_0_5: u32 = 0x1400_0005;

impl Reader<'_> {
    /// The file has the fields added in `version`.
    fn since(&self, version: u32) -> bool {
        self.v >= version
    }
    /// The file still has the fields last in `version`.
    fn until(&self, version: u32) -> bool {
        self.v <= version
    }
    /// Bethesda's stream header comes next: a small version, then three strings (author, process
    /// and export script), each a length byte and ended by a NUL. (Without one, a count of block
    /// types and the first type's name's length come next: never as small.)
    fn stream_header_follows(&self) -> bool {
        let b = &self.b[self.at.min(self.b.len())..];
        if b.len() < 4 || u32::from_le_bytes(b[..4].try_into().unwrap()) > 0xFF {
            return false;
        }
        let mut at = 4;
        for _ in 0..3 {
            let Some(&n) = b.get(at) else { return false };
            if b.get(at + n as usize) != Some(&0) {
                return false;
            }
            at += 1 + n as usize;
        }
        true
    }

    fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.at + n > self.b.len() {
            bail!("read past the end at {:#x} (+{n})", self.at);
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn skip(&mut self, n: usize) -> Result<()> {
        self.take(n).map(|_| ())
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn i32(&mut self) -> Result<i32> {
        Ok(self.u32()? as i32)
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn bool(&mut self) -> Result<bool> {
        Ok(self.u8()? != 0)
    }
    fn vec3(&mut self) -> Result<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    fn vec4(&mut self) -> Result<[f32; 4]> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }
    /// A `Matrix33` as stored: row-major, as Gamebryo uses it (rotating column vectors).
    fn mat3(&mut self) -> Result<Mat3> {
        let mut m = [0f32; 9];
        for v in &mut m {
            *v = self.f32()?;
        }
        Ok(Mat3::from_cols_array(&m).transpose())
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u32()? as usize;
        if n > 0x10000 {
            bail!("string of {n} bytes at {:#x}", self.at);
        }
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }
    fn refs(&mut self) -> Result<Vec<i32>> {
        let n = self.u32()? as usize;
        if n > 0x10000 {
            bail!("{n} refs at {:#x}", self.at);
        }
        (0..n).map(|_| self.i32()).collect()
    }
    fn transform(&mut self) -> Result<Transform> {
        let rotation = self.mat3()?;
        let translation = self.vec3()?;
        let scale = self.f32()?;
        Ok(Transform { rotation, translation, scale })
    }
}

fn object_net(r: &mut Reader) -> Result<(String, Vec<i32>)> {
    let name = r.string()?;
    let extra = r.refs()?;
    let _controller = r.i32()?;
    Ok((name, extra))
}

fn av_object(r: &mut Reader) -> Result<AvObject> {
    let (name, extra) = object_net(r)?;
    let flags = r.u16()?;
    let translation = r.vec3()?;
    let rotation = r.mat3()?;
    let scale = r.f32()?;
    let properties = r.refs()?;
    let collision = r.i32()?;
    Ok(AvObject { name, flags, transform: Transform { rotation, translation, scale }, properties, extra, collision })
}

/// A `HavokMaterial` (up to 10.0.1.2 after an unknown int).
fn havok_material(r: &mut Reader) -> Result<u32> {
    if r.until(V10_0_1_2) {
        r.skip(4)?;
    }
    r.u32()
}

fn tex_desc(r: &mut Reader) -> Result<i32> {
    let source = r.i32()?;
    r.skip(4 + 4 + 4)?; // clamp, filter, UV set
    if !r.since(V10_4_0_1) {
        r.skip(4)?; // PS2 L, K
    }
    if r.since(V10_1_0_0) && r.bool()? {
        r.skip(8 + 8 + 4 + 4 + 8)?; // translation, scale, rotation, method, centre
    }
    Ok(source)
}

/// `NiGeometryData`'s fields, and its vertex count (a particle system's data has no vertices but
/// arrays as long).
fn geometry_data(r: &mut Reader) -> Result<(ShapeData, usize)> {
    if r.since(V10_1_0_114) {
        let _group = r.i32()?;
    }
    let n = r.u16()? as usize;
    if r.since(V10_1_0_0) {
        r.skip(2)?; // keep, compress
    }
    let mut d = ShapeData::default();
    if r.bool()? {
        d.vertices = (0..n).map(|_| r.vec3()).collect::<Result<_>>()?;
    }
    let flags = r.u16()?;
    if r.bool()? {
        d.normals = (0..n).map(|_| r.vec3()).collect::<Result<_>>()?;
        if flags & 4096 != 0 && r.since(V10_1_0_0) {
            r.skip(n * 24)?; // tangents, bitangents
        }
    }
    r.skip(16)?; // bound
    if r.bool()? {
        d.colors = (0..n).map(|_| r.vec4()).collect::<Result<_>>()?;
    }
    let uv_sets = (flags & 63) as usize;
    for set in 0..uv_sets {
        let uv: Vec<[f32; 2]> = (0..n).map(|_| Ok([r.f32()?, r.f32()?])).collect::<Result<_>>()?;
        if set == 0 {
            d.uvs = uv;
        }
    }
    r.skip(2)?; // consistency
    if r.since(V20_0_0_4) {
        let _additional = r.i32()?;
    }
    Ok((d, n))
}

fn tri_shape_data(r: &mut Reader) -> Result<ShapeData> {
    let (mut d, _) = geometry_data(r)?;
    let tris = r.u16()? as usize;
    let _points = r.u32()?;
    if !r.since(V10_1_0_0) || r.bool()? {
        d.triangles = (0..tris).map(|_| Ok([r.u16()?, r.u16()?, r.u16()?])).collect::<Result<_>>()?;
    }
    let groups = r.u16()? as usize;
    for _ in 0..groups {
        let n = r.u16()? as usize;
        r.skip(n * 2)?;
    }
    Ok(d)
}

fn tri_strips_data(r: &mut Reader) -> Result<ShapeData> {
    let (mut d, _) = geometry_data(r)?;
    let _tris = r.u16()?;
    let strips = r.u16()? as usize;
    let lengths: Vec<usize> = (0..strips).map(|_| Ok(r.u16()? as usize)).collect::<Result<_>>()?;
    if !r.since(V10_0_1_3) || r.bool()? {
        for len in lengths {
            let s: Vec<u16> = (0..len).map(|_| r.u16()).collect::<Result<_>>()?;
            for i in 2..len {
                let t = if i % 2 == 0 { [s[i - 2], s[i - 1], s[i]] } else { [s[i - 2], s[i], s[i - 1]] };
                if t[0] != t[1] && t[1] != t[2] && t[0] != t[2] {
                    d.triangles.push(t);
                }
            }
        }
    }
    Ok(d)
}

fn skin_partition(r: &mut Reader) -> Result<()> {
    let n = r.u32()?;
    for _ in 0..n {
        let verts = r.u16()? as usize;
        let tris = r.u16()? as usize;
        let bones = r.u16()? as usize;
        let strips = r.u16()? as usize;
        let per_vertex = r.u16()? as usize;
        r.skip(bones * 2)?;
        // (Before 10.1 the vertex map, weights and faces are always there.)
        let always = !r.since(V10_1_0_0);
        if always || r.bool()? {
            r.skip(verts * 2)?;
        }
        if always || r.bool()? {
            r.skip(verts * per_vertex * 4)?;
        }
        let lengths: Vec<usize> = (0..strips).map(|_| Ok(r.u16()? as usize)).collect::<Result<_>>()?;
        if always || r.bool()? {
            if strips != 0 {
                r.skip(lengths.iter().sum::<usize>() * 2)?;
            } else {
                r.skip(tris * 6)?;
            }
        }
        if r.bool()? {
            r.skip(verts * per_vertex)?;
        }
    }
    Ok(())
}

/// `NiTimeController`'s own fields: next, flags, frequency, phase, start, stop, target.
fn time_controller(r: &mut Reader) -> Result<()> {
    r.skip(4 + 2 + 4 * 4 + 4)
}

/// `NiInterpController`'s: `NiTimeController`'s and (10.1.0.104 to 10.1.0.108) whether a sequence
/// manages it.
fn interp_controller(r: &mut Reader) -> Result<()> {
    time_controller(r)?;
    if r.since(V10_1_0_104) && r.until(V10_1_0_108) {
        r.skip(1)?;
    }
    Ok(())
}

/// `NiSingleInterpController`'s: those and its interpolator (from 10.1.0.104; before, the type's
/// own fields end with its keys' block: [`old_keys`]).
fn single_interp_controller(r: &mut Reader) -> Result<()> {
    interp_controller(r)?;
    if r.since(V10_1_0_104) {
        r.skip(4)?;
    }
    Ok(())
}

/// A controller's keys' block, which it held itself up to 10.1.0.103.
fn old_keys(r: &mut Reader) -> Result<()> {
    if r.until(V10_1_0_103) {
        r.skip(4)?;
    }
    Ok(())
}

/// `NiPSysModifier`'s: name, order, particle system, active.
fn psys_modifier(r: &mut Reader) -> Result<()> {
    r.string()?;
    r.skip(4 + 4 + 1)
}

/// `NiPSysEmitter`'s: speed, declination and planar angle (each with its variation), colour, radius
/// (and from 10.4.0.1 its variation), life span and its variation; and a volume emitter's node
/// (from 10.1).
fn psys_emitter(r: &mut Reader, volume: bool) -> Result<()> {
    psys_modifier(r)?;
    r.skip(24 + 16 + 4)?;
    if r.since(V10_4_0_1) {
        r.skip(4)?;
    }
    r.skip(8)?;
    if volume && r.since(V10_1_0_0) {
        r.skip(4)?;
    }
    Ok(())
}

/// `NiParticlesData`' fields (after `NiGeometryData`'s) for `n` particles: their sizes and
/// rotations.
fn particles_data(r: &mut Reader, n: usize) -> Result<()> {
    if r.until(V10_0_1_0) {
        r.skip(4)?; // radius
    }
    if r.since(V10_1_0_0) && r.bool()? {
        r.skip(n * 4)?; // radii
    }
    r.skip(2)?; // active
    if r.bool()? {
        r.skip(n * 4)?; // sizes
    }
    if r.bool()? {
        r.skip(n * 16)?; // rotations
    }
    if r.since(V20_0_0_4) {
        if r.bool()? {
            r.skip(n * 4)?; // rotation angles
        }
        if r.bool()? {
            r.skip(n * 12)?; // rotation axes
        }
    }
    Ok(())
}

/// `NiBlendInterpolator`'s fields (a blend of a sequence's interpolators).
fn blend_interpolator(r: &mut Reader) -> Result<()> {
    if r.since(V10_1_0_112) {
        let flags = r.u8()?;
        let items = r.u8()? as usize;
        r.skip(4)?; // weight threshold
        if flags & 1 == 0 {
            r.skip(4 + 4 * 4)?; // count, single index, priorities; single time, weight sums, spinner
            r.skip(items * 17)?; // interpolator, weights, priority, spinner
        }
    } else {
        // (The older layout, 10.1.0.106's here: shorts for the counts, an int for each priority.)
        let items = r.u16()? as usize;
        r.skip(2)?; // grow by
        r.skip(items * 20)?; // interpolator, weights, priority, spinner
        r.skip(1 + 4 + 1 + 2 + 2 + 4 + 4)?; // managed, weight threshold, only the highest, count, single index, priorities
    }
    Ok(())
}

/// A `KeyGroup` of `width` floats per value.
fn key_group(r: &mut Reader, width: usize) -> Result<()> {
    let n = r.u32()? as usize;
    if n == 0 {
        return Ok(());
    }
    let interpolation = r.u32()?;
    let each = 4 + width * 4 * match interpolation {
        2 => 3,
        _ => 1,
    } + if interpolation == 3 { 12 } else { 0 };
    r.skip(n * each)
}

/// A `KeyGroup` of `width` floats per value, kept: each key's time and value (its tangents or TBC
/// read past).
fn keys(r: &mut Reader, width: usize) -> Result<Vec<(f32, [f32; 3])>> {
    let n = r.u32()? as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n > 0x10000 {
        bail!("{n} keys at {:#x}", r.at);
    }
    let interpolation = r.u32()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let t = r.f32()?;
        let mut v = [0f32; 3];
        for x in v.iter_mut().take(width) {
            *x = r.f32()?;
        }
        match interpolation {
            2 => r.skip(width * 8)?,
            3 => r.skip(12)?,
            _ => {}
        }
        out.push((t, v));
    }
    Ok(out)
}

/// `NiKeyframeData`'s keys.
fn keyframes(r: &mut Reader) -> Result<Keyframes> {
    let mut k = Keyframes::default();
    let n = r.u32()? as usize;
    if n != 0 {
        let kind = r.u32()?;
        if kind == 4 {
            if r.until(V10_1_0_0) {
                r.skip(4)?; // order
            }
            for axis in &mut k.xyz {
                *axis = keys(r, 1)?.into_iter().map(|(t, v)| (t, v[0])).collect();
            }
        } else {
            if n > 0x10000 {
                bail!("{n} rotation keys at {:#x}", r.at);
            }
            for _ in 0..n {
                let t = r.f32()?;
                let [w, x, y, z] = r.vec4()?;
                if kind == 3 {
                    r.skip(12)?;
                }
                k.rotations.push((t, [x, y, z, w]));
            }
        }
    }
    k.translations = keys(r, 3)?.into_iter().map(|(t, v)| (t, Vec3::from(v))).collect();
    key_group(r, 1)?; // scales
    Ok(k)
}

fn block(r: &mut Reader, kind: &str) -> Result<Option<Block>> {
    Ok(Some(match kind {
        "NiNode" | "BSFadeNode" | "AvoidNode" | "RootCollisionNode" | "NiBSAnimationNode" | "NiBSParticleNode" | "NiBillboardNode" => {
            let av = av_object(r)?;
            let children = r.refs()?;
            let _effects = r.refs()?;
            if kind == "NiBillboardNode" && r.since(V10_1_0_0) {
                r.skip(2)?; // billboard mode (before 10.1, in the flags)
            }
            Block::Node { av, children }
        }
        "NiTriShape" | "NiTriStrips" => {
            let av = av_object(r)?;
            let data = r.i32()?;
            let skin = r.i32()?;
            if r.bool()? {
                r.string()?;
                r.skip(4)?;
            }
            Block::Shape { av, data, skin }
        }
        "NiTriShapeData" => Block::ShapeData(tri_shape_data(r)?),
        "NiTriStripsData" => Block::ShapeData(tri_strips_data(r)?),
        "NiSkinInstance" => {
            let data = r.i32()?;
            let partition = if r.since(V10_1_0_101) { r.i32()? } else { -1 };
            let root = r.i32()?;
            let bones = r.refs()?;
            Block::SkinInstance { data, partition, root, bones }
        }
        "NiSkinData" => {
            let transform = r.transform()?;
            let n = r.u32()? as usize;
            if !r.since(V10_1_0_0) {
                let _partition = r.i32()?;
            }
            let has_weights = r.bool()?;
            let mut bones = Vec::with_capacity(n);
            for _ in 0..n {
                let transform = r.transform()?;
                r.skip(16)?; // bound
                let count = r.u16()? as usize;
                let weights = if has_weights { (0..count).map(|_| Ok((r.u16()?, r.f32()?))).collect::<Result<_>>()? } else { Vec::new() };
                bones.push(BoneData { transform, weights });
            }
            Block::SkinData { transform, bones }
        }
        "NiSkinPartition" => {
            skin_partition(r)?;
            Block::SkinPartition
        }
        "NiTexturingProperty" => {
            object_net(r)?;
            if !r.since(V10_0_1_3) {
                r.skip(2)?; // flags
            }
            r.skip(4)?; // apply mode
            let count = r.u32()?;
            let mut t = Texturing { base: -1, glow: -1, bump: -1, detail: -1 };
            if r.bool()? {
                t.base = tex_desc(r)?;
            }
            if r.bool()? {
                tex_desc(r)?; // dark
            }
            if r.bool()? {
                t.detail = tex_desc(r)?;
            }
            if r.bool()? {
                tex_desc(r)?; // gloss
            }
            if r.bool()? {
                t.glow = tex_desc(r)?;
            }
            if count > 5 && r.bool()? {
                t.bump = tex_desc(r)?;
                r.skip(4 + 4 + 16)?;
            }
            for i in 6..count.min(10) {
                let _ = i;
                if r.bool()? {
                    tex_desc(r)?; // decal
                }
            }
            let shader = r.u32()?;
            for _ in 0..shader {
                if r.bool()? {
                    tex_desc(r)?;
                    r.skip(4)?;
                }
            }
            Block::Texturing(t)
        }
        "NiSourceTexture" => {
            object_net(r)?;
            let external = r.u8()?;
            let file = if external == 1 {
                let f = r.string()?;
                if r.since(V10_1_0_0) {
                    r.skip(4)?; // unknown ref
                }
                f
            } else {
                if r.since(V10_1_0_0) {
                    r.string()?;
                } else if r.u8()? == 0 {
                    // (Not internal either: no pixel data follows.)
                    r.skip(12 + 1)?;
                    return Ok(Some(Block::SourceTexture { file: String::new() }));
                }
                r.skip(4)?; // pixel data
                String::new()
            };
            r.skip(12 + 1)?; // format prefs, static
            if r.since(V10_1_0_103) {
                r.skip(1)?; // direct render
            }
            Block::SourceTexture { file }
        }
        "NiMaterialProperty" => {
            let (name, _) = object_net(r)?;
            if !r.since(V10_0_1_3) {
                r.skip(2)?; // flags
            }
            let c = |r: &mut Reader| -> Result<[f32; 3]> { Ok([r.f32()?, r.f32()?, r.f32()?]) };
            Block::Material(MaterialProps { name, ambient: c(r)?, diffuse: c(r)?, specular: c(r)?, emissive: c(r)?, glossiness: r.f32()?, alpha: r.f32()? })
        }
        "NiAlphaProperty" => {
            object_net(r)?;
            let flags = r.u16()?;
            let threshold = r.u8()?;
            Block::Alpha { flags, threshold }
        }
        "NiStencilProperty" => {
            object_net(r)?;
            if r.until(V10_0_1_2) {
                r.skip(2)?; // flags
            }
            r.skip(1 + 4 + 4 + 4 + 4 + 4 + 4 + 4)?;
            Block::Other(kind.into())
        }
        "NiFogProperty" => {
            object_net(r)?;
            r.skip(2 + 4 + 12)?; // flags, depth, colour
            Block::Other(kind.into())
        }
        "NiVertexColorProperty" => {
            object_net(r)?;
            r.skip(2 + 4 + 4)?;
            Block::Other(kind.into())
        }
        "NiZBufferProperty" => {
            object_net(r)?;
            r.skip(2 + 4)?;
            Block::Other(kind.into())
        }
        "NiSpecularProperty" => {
            object_net(r)?;
            Block::Specular { on: r.u16()? & 1 != 0 }
        }
        "NiWireframeProperty" | "NiDitherProperty" | "NiShadeProperty" => {
            object_net(r)?;
            r.skip(2)?;
            Block::Other(kind.into())
        }
        "NiDirectionalLight" | "NiAmbientLight" | "NiPointLight" => {
            // A light exported with the model (the head's parts have one): read past.
            av_object(r)?;
            if r.since(0x0A01_006A) {
                r.skip(1)?; // switch state
            }
            if r.since(V10_1_0_0) {
                r.refs()?; // affected nodes
            }
            r.skip(4 + 36)?; // dimmer, ambient, diffuse, specular
            if kind == "NiPointLight" {
                r.skip(12)?; // attenuation
            }
            Block::Other(kind.into())
        }
        "NiBooleanExtraData" => {
            r.string()?;
            r.skip(1)?;
            Block::Other(kind.into())
        }
        "NiStringExtraData" => {
            let name = r.string()?;
            let value = r.string()?;
            Block::StringExtra { name, value }
        }
        "BSXFlags" | "NiIntegerExtraData" => {
            let name = r.string()?;
            let value = r.u32()?;
            Block::IntegerExtra { name, value }
        }
        "NiBinaryExtraData" => {
            let name = r.string()?;
            let n = r.u32()? as usize;
            let data = r.take(n)?.to_vec();
            Block::BinaryExtra { name, data }
        }
        "BSBound" => {
            r.string()?;
            r.skip(24)?;
            Block::Other(kind.into())
        }
        "BSFurnitureMarker" => {
            r.string()?;
            let n = r.u32()? as usize;
            r.skip(n * 16)?; // positions: offset, orientation, the marker models they're for
            Block::Other(kind.into())
        }
        "NiCamera" => {
            av_object(r)?;
            if r.since(V10_1_0_0) {
                r.skip(2)?; // flags
            }
            r.skip(24)?; // frustum
            if r.since(V10_1_0_0) {
                r.skip(1)?; // orthographic
            }
            r.skip(16 + 4 + 4 + 4 + 4)?; // viewport, LOD adjust, scene, screen polygons and textures
            Block::Other(kind.into())
        }
        "bhkCollisionObject" | "bhkSPCollisionObject" => {
            let target = r.i32()?;
            r.skip(2)?; // flags
            let body = r.i32()?;
            Block::CollisionObject { target, body }
        }
        "bhkRigidBody" | "bhkRigidBodyT" => {
            let shape = r.i32()?;
            if r.until(V10_0_1_2) {
                r.skip(4)?; // unknown
            }
            let filter = r.u32()?;
            r.skip(20 + 4)?; // world object info, entity info
            if r.since(V10_1_0_0) {
                r.skip(16)?; // the filter and entity info again
            }
            r.skip(4)?; // unused
            let translation = r.vec4()?;
            let rotation = r.vec4()?;
            r.skip(16 + 16 + 48)?; // linear and angular velocity, inertia
            let center = r.vec4()?;
            let mass = r.f32()?;
            r.skip(8)?; // damping
            let friction = r.f32()?;
            let restitution = r.f32()?;
            if r.since(V10_1_0_0) {
                r.skip(12)?; // fastest linear and angular velocity, penetration depth
            }
            let motion = r.u8()?;
            r.skip(3 + 12)?; // deactivator, solver deactivation, quality; unused
            let body = RigidBody { shape, layer: filter as u8, translation, rotation, center, mass, friction, restitution, transformed: kind == "bhkRigidBodyT", motion };
            let _constraints = r.refs()?;
            r.skip(4)?; // body flags
            Block::RigidBody(body)
        }
        "bhkBoxShape" => {
            havok_material(r)?;
            let radius = r.f32()?;
            r.skip(8)?;
            let dims = r.vec3()?;
            r.skip(4)?;
            Block::BoxShape { dims, radius }
        }
        "bhkCapsuleShape" => {
            havok_material(r)?;
            let radius = r.f32()?;
            r.skip(8)?;
            let a = r.vec3()?;
            r.skip(4)?;
            let b = r.vec3()?;
            r.skip(4)?;
            Block::CapsuleShape { a, b, radius }
        }
        "bhkSphereShape" => {
            havok_material(r)?;
            Block::SphereShape { radius: r.f32()? }
        }
        "bhkMultiSphereShape" => {
            havok_material(r)?;
            r.skip(12)?; // shape property
            let n = r.u32()? as usize;
            let spheres = (0..n).map(|_| Ok((r.vec3()?, r.f32()?))).collect::<Result<_>>()?;
            Block::MultiSphere { spheres }
        }
        "bhkConvexVerticesShape" => {
            havok_material(r)?;
            let radius = r.f32()?;
            r.skip(24)?;
            let n = r.u32()? as usize;
            let vertices = (0..n).map(|_| r.vec4().map(|v| Vec3::new(v[0], v[1], v[2]))).collect::<Result<_>>()?;
            let n = r.u32()? as usize;
            let planes = (0..n).map(|_| r.vec4()).collect::<Result<_>>()?;
            Block::ConvexVertices { vertices, radius, planes }
        }
        "bhkListShape" => {
            let shapes = r.refs()?;
            havok_material(r)?;
            r.skip(24)?;
            let filters = r.u32()? as usize;
            r.skip(filters * 4)?;
            Block::ListShape { shapes }
        }
        "bhkConvexListShape" => {
            let shapes = r.refs()?;
            havok_material(r)?;
            r.skip(4 + 8 + 12 + 1 + 4)?; // radius, unknown, child shape property, cached bounds, closest point distance
            Block::ConvexList { shapes }
        }
        "bhkConvexTransformShape" | "bhkTransformShape" => {
            let shape = r.i32()?;
            havok_material(r)?;
            r.skip(4 + 8)?;
            let mut transform = [0f32; 16];
            for v in &mut transform {
                *v = r.f32()?;
            }
            Block::ConvexTransform { shape, transform }
        }
        "bhkConvexSweepShape" => {
            // (10.0.1.0's.)
            r.skip(4)?; // shape
            havok_material(r)?;
            r.skip(4 + 12)?; // radius, unknown
            Block::Other(kind.into())
        }
        "bhkMeshShape" => {
            // (10.0.1.0's `bhkNiTriStripsShape`: as it, triangles in `NiTriStripsData` blocks.)
            r.skip(8 + 4 + 8 + 16)?; // unknown, radius, unknown, scale (1/7)
            let n = r.u32()? as usize;
            r.skip(n * 12 + 12)?; // shape properties, unknown
            let strips = if r.until(V10_0_1_0) { r.refs()? } else { Vec::new() };
            Block::TriStripsShape { strips }
        }
        // Collision made of triangles (much of the clutter and all the architecture).
        "bhkNiTriStripsShape" => {
            havok_material(r)?;
            r.skip(4 + 20 + 4)?; // radius, unused, grow by
            if r.since(V10_1_0_0) {
                r.skip(16)?; // scale
            }
            let strips = r.refs()?;
            let filters = r.u32()? as usize;
            r.skip(filters * 4)?;
            Block::TriStripsShape { strips }
        }
        "hkPackedNiTriStripsData" => {
            let n = r.u32()? as usize;
            let mut triangles = Vec::with_capacity(n.min(0x10000));
            for _ in 0..n {
                triangles.push([r.u16()?, r.u16()?, r.u16()?]);
                r.skip(2 + 12)?; // welding, normal
            }
            let n = r.u32()? as usize;
            let vertices = (0..n).map(|_| r.vec3()).collect::<Result<_>>()?;
            Block::PackedStripsData { triangles, vertices }
        }
        "bhkPackedNiTriStripsShape" => {
            let n = r.u16()? as usize;
            let parts = (0..n)
                .map(|_| {
                    let filter = r.u32()?;
                    let vertices = r.u32()?;
                    Ok(PackedPart { layer: filter as u8, vertices, material: havok_material(r)? })
                })
                .collect::<Result<_>>()?;
            r.skip(4 + 4 + 4 + 4 + 16 + 4 + 16)?; // user data, unused, radius, unused, scale, copies
            let data = r.i32()?;
            Block::PackedStrips { parts, data }
        }
        "bhkMoppBvTreeShape" => {
            let shape = r.i32()?;
            r.skip(12 + 4)?; // unused, scale
            let n = r.u32()? as usize;
            if r.since(V10_1_0_0) {
                r.skip(16)?; // offset
            }
            r.skip(n)?; // the code
            Block::Mopp { shape }
        }
        "bhkSimpleShapePhantom" => {
            r.skip(4)?; // shape
            if r.until(V10_0_1_2) {
                r.skip(4)?; // unknown
            }
            r.skip(4 + 20 + 8 + 64)?; // filter, world object info, unused, transform
            Block::Other(kind.into())
        }
        "NiBSBoneLODController" => {
            time_controller(r)?;
            r.skip(4)?; // LOD
            let lods = r.u32()?;
            let _groups = r.u32()?;
            for _ in 0..lods {
                let _nodes = r.refs()?;
            }
            Block::Other(kind.into())
        }
        "bhkBlendController" => {
            time_controller(r)?;
            r.skip(4)?;
            Block::Other(kind.into())
        }
        "NiTransformController" | "NiKeyframeController" | "NiVisController" | "NiAlphaController" => {
            single_interp_controller(r)?;
            old_keys(r)?;
            Block::Other(kind.into())
        }
        "BSKeyframeController" => {
            single_interp_controller(r)?;
            old_keys(r)?;
            r.skip(4)?; // more keys
            Block::Other(kind.into())
        }
        "NiMaterialColorController" => {
            single_interp_controller(r)?;
            if r.since(V10_1_0_0) {
                r.skip(2)?; // which colour
            }
            old_keys(r)?;
            Block::Other(kind.into())
        }
        "NiTextureTransformController" => {
            single_interp_controller(r)?;
            r.skip(1 + 4 + 4)?; // shader map, texture slot, what it moves
            old_keys(r)?;
            Block::Other(kind.into())
        }
        "NiFlipController" => {
            single_interp_controller(r)?;
            r.skip(4)?; // texture slot
            if r.until(V10_1_0_103) {
                r.skip(4 + 4)?; // time so far, time between
            }
            let _textures = r.refs()?;
            Block::Other(kind.into())
        }
        // Particle systems: read past.
        "NiPSysEmitterCtlr" => {
            single_interp_controller(r)?;
            r.string()?; // modifier
            old_keys(r)?;
            if r.since(V10_1_0_104) {
                r.skip(4)?; // visibility interpolator
            }
            Block::Other(kind.into())
        }
        "NiPSysModifierActiveCtlr" | "NiPSysEmitterDeclinationCtlr" | "NiPSysEmitterDeclinationVarCtlr" | "NiPSysEmitterInitialRadiusCtlr" | "NiPSysEmitterLifeSpanCtlr" | "NiPSysEmitterSpeedCtlr" | "NiPSysGravityStrengthCtlr" => {
            single_interp_controller(r)?;
            r.string()?; // modifier
            old_keys(r)?;
            Block::Other(kind.into())
        }
        "NiPSysUpdateCtlr" | "NiPSysResetOnLoopCtlr" => {
            time_controller(r)?;
            Block::Other(kind.into())
        }
        "NiParticleSystem" | "NiMeshParticleSystem" => {
            av_object(r)?;
            r.skip(4 + 4)?; // data, skin
            if r.bool()? {
                r.string()?; // shader
                r.skip(4)?;
            }
            if r.since(V10_1_0_0) {
                r.skip(1)?; // world space
                let _modifiers = r.refs()?;
            }
            Block::Other(kind.into())
        }
        "NiPSysData" | "NiMeshPSysData" => {
            let (_, n) = geometry_data(r)?;
            particles_data(r, n)?;
            r.skip(n * if r.until(V10_4_0_1) { 40 } else { 28 })?; // each particle's state
            if r.since(V20_0_0_2) && r.bool()? {
                r.skip(n * 4)?; // rotation speeds
            }
            r.skip(2 + 2)?; // added particles, base
            if kind == "NiMeshPSysData" {
                if r.since(V10_2_0_0) {
                    r.skip(4 + 1)?; // pool size, fill pools
                    let n = r.u32()? as usize;
                    r.skip(n * 4)?; // generations
                }
                r.skip(4)?; // particle meshes
            }
            Block::Other(kind.into())
        }
        "NiPSysAgeDeathModifier" | "NiPSysBombModifier" | "NiPSysBoundUpdateModifier" | "NiPSysColliderManager" | "NiPSysColorModifier" | "NiPSysDragModifier" | "NiPSysGravityModifier" | "NiPSysGrowFadeModifier" | "NiPSysPositionModifier" | "NiPSysSpawnModifier" | "BSParentVelocityModifier" | "BSWindModifier" => {
            psys_modifier(r)?;
            r.skip(match kind {
                "NiPSysAgeDeathModifier" => 1 + 4,                     // spawn on death, spawner
                "NiPSysBombModifier" => 4 + 12 + 4 + 4 + 4 + 4,        // node, axis, decay, acceleration, decay and symmetry types
                "NiPSysBoundUpdateModifier" => 2,                      // update skip
                "NiPSysDragModifier" => 4 + 12 + 4 + 4 + 4,            // node, axis, percentage, range, falloff
                "NiPSysGravityModifier" => 4 + 12 + 4 + 4 + 4 + 4 + 4, // node, axis, decay, strength, force type, turbulence and its scale
                "NiPSysGrowFadeModifier" => 4 + 2 + 4 + 2,             // grow time and generation, fade time and generation
                "NiPSysSpawnModifier" => 2 + 4 + 2 + 2 + 4 * 4,        // generations, share spawned, least and most, speed and direction variation, life span and its variation
                "NiPSysPositionModifier" => 0,
                _ => 4, // collider, colour keys, damping, strength
            })?;
            Block::Other(kind.into())
        }
        "NiPSysRotationModifier" => {
            psys_modifier(r)?;
            r.skip(4)?; // speed
            if r.since(V20_0_0_2) {
                r.skip(4 + 4 + 4 + 1)?; // speed variation, angle and its variation, random sign
            }
            r.skip(1 + 12)?; // random axis, axis
            Block::Other(kind.into())
        }
        "NiPSysBoxEmitter" | "NiPSysCylinderEmitter" | "NiPSysSphereEmitter" | "BSPSysArrayEmitter" => {
            psys_emitter(r, true)?;
            r.skip(match kind {
                "NiPSysBoxEmitter" => 12,     // width, height, depth
                "NiPSysCylinderEmitter" => 8, // radius, height
                "NiPSysSphereEmitter" => 4,   // radius
                _ => 0,
            })?;
            Block::Other(kind.into())
        }
        "NiPSysMeshUpdateModifier" => {
            psys_modifier(r)?;
            let _meshes = r.refs()?;
            Block::Other(kind.into())
        }
        "NiPSysMeshEmitter" => {
            psys_emitter(r, false)?;
            let _meshes = r.refs()?;
            r.skip(4 + 4 + 12)?; // velocity type, where from, axis
            Block::Other(kind.into())
        }
        "NiPSysPlanarCollider" => {
            r.skip(4 + 1 + 1 + 4 + 4 + 4 + 4)?; // bounce, spawn and die on collision, spawner, manager, next, node
            r.skip(4 + 4 + 12 + 12)?; // width, height, axes
            Block::Other(kind.into())
        }
        "NiTransformInterpolator" => {
            let translation = r.vec3()?;
            let [w, x, y, z] = r.vec4()?;
            let scale = r.f32()?;
            if r.until(V10_1_0_109) {
                r.skip(3)?; // which of those are set
            }
            let data = r.i32()?;
            Block::TransformInterpolator { translation, rotation: [x, y, z, w], scale, data }
        }
        "NiFloatInterpolator" => {
            let value = r.f32()?;
            let data = r.i32()?;
            Block::FloatInterpolator { value, data }
        }
        "NiBoolInterpolator" | "NiBoolTimelineInterpolator" => {
            r.skip(1 + 4)?; // value, data
            Block::Other(kind.into())
        }
        "NiPathInterpolator" => {
            r.skip(2 + 4 + 4 + 4 + 2 + 4 + 4)?; // flags, bank direction, steepest bank, smoothing, follow axis, path keys, percent keys
            Block::Other(kind.into())
        }
        // A blend of interpolators (an older model's sequences go through one), and its value.
        "NiBlendTransformInterpolator" | "NiBlendFloatInterpolator" | "NiBlendBoolInterpolator" | "NiBlendPoint3Interpolator" => {
            blend_interpolator(r)?;
            match kind {
                "NiBlendFloatInterpolator" => r.skip(4)?,
                "NiBlendBoolInterpolator" => r.skip(1)?,
                "NiBlendPoint3Interpolator" => r.skip(12)?,
                _ if r.until(V10_1_0_109) => r.skip(12 + 16 + 4 + 3)?, // translation, rotation, scale, which are set
                _ => {}
            }
            Block::Other(kind.into())
        }
        "NiPoint3Interpolator" => {
            r.skip(12 + 4)?;
            Block::Other(kind.into())
        }
        "NiTransformData" | "NiKeyframeData" => Block::Keyframes(keyframes(r)?),
        "NiFloatData" => Block::FloatKeys(keys(r, 1)?.into_iter().map(|(t, v)| (t, v[0])).collect()),
        "NiPosData" => {
            key_group(r, 3)?;
            Block::Other(kind.into())
        }
        "NiColorData" => {
            key_group(r, 4)?;
            Block::Other(kind.into())
        }
        "NiBoolData" => {
            // (Bytes, one a key; constant keys.)
            let n = r.u32()? as usize;
            if n != 0 {
                let interpolation = r.u32()?;
                r.skip(n * (4 + 1 + if interpolation == 2 { 2 } else if interpolation == 3 { 12 } else { 0 }))?;
            }
            Block::Other(kind.into())
        }
        "NiTextKeyExtraData" => {
            r.string()?;
            let n = r.u32()?;
            Block::TextKeys((0..n).map(|_| Ok((r.f32()?, r.string()?))).collect::<Result<_>>()?)
        }
        "NiControllerManager" => {
            time_controller(r)?;
            r.skip(1)?; // cumulative
            let sequences = r.refs()?;
            r.skip(4)?; // object palette
            Block::ControllerManager { sequences }
        }
        "NiControllerSequence" => {
            let mut s = Sequence { name: r.string()?, palette: -1, text_keys: -1, ..Default::default() };
            if r.until(V10_1_0_103) {
                r.string()?; // accumulation root
                s.text_keys = r.i32()?;
            }
            let n = r.u32()? as usize;
            if n > 0x1000 {
                bail!("{n} controlled blocks at {:#x}", r.at);
            }
            if r.since(V10_1_0_106) {
                r.skip(4)?; // array grow by
            }
            // (A name kept in the sequence's own: its offset there.)
            let name = |r: &mut Reader, names: &mut String| -> Result<u32> {
                let at = names.len() as u32;
                names.push_str(&r.string()?);
                names.push('\0');
                Ok(at)
            };
            for _ in 0..n {
                // What it animates: the interpolator, and the node's name, the controller's type, its
                // id and the interpolator's id (from 10.2 offsets in a string palette, before the
                // names themselves; up to 10.1.0.103 the node's name and its controller alone).
                if r.until(V10_1_0_103) {
                    let node = name(r, &mut s.names)?;
                    r.skip(4)?; // controller
                    let none = s.names.len() as u32;
                    s.names.push('\0');
                    s.blocks.push((-1, [node, none, none, none]));
                    continue;
                }
                let interpolator = r.i32()?;
                r.skip(4)?; // controller
                if r.until(V10_1_0_110) {
                    r.skip(4 + 2)?; // blend interpolator, its index
                }
                if r.bs > 0 {
                    r.skip(1)?; // priority
                }
                let offsets = if r.since(V10_2_0_0) {
                    let _palette = r.i32()?;
                    let node = r.u32()?;
                    r.skip(4)?; // property type
                    [node, r.u32()?, r.u32()?, r.u32()?]
                } else {
                    let node = name(r, &mut s.names)?;
                    r.string()?; // property type
                    [node, name(r, &mut s.names)?, name(r, &mut s.names)?, name(r, &mut s.names)?]
                };
                s.blocks.push((interpolator, offsets));
            }
            if r.since(V10_1_0_106) {
                r.skip(4)?; // weight
                s.text_keys = r.i32()?;
                r.skip(4 + 4)?; // cycle type, frequency
                if !r.since(V10_4_0_1) {
                    r.skip(4)?; // phase
                }
                s.start = r.f32()?;
                s.stop = r.f32()?;
                if r.v == V10_1_0_106 {
                    r.skip(1)?; // play backwards
                }
                r.skip(4)?; // manager
                r.string()?; // accumulation root
                if r.since(V10_1_0_113) {
                    s.palette = r.i32()?;
                }
            }
            Block::Sequence(s)
        }
        "NiStringPalette" => {
            let s = r.string()?;
            r.skip(4)?;
            Block::StringPalette(s)
        }
        "NiDefaultAVObjectPalette" => {
            r.skip(4)?; // scene
            let n = r.u32()?;
            for _ in 0..n {
                r.string()?;
                r.skip(4)?;
            }
            Block::Other(kind.into())
        }
        "NiMultiTargetTransformController" => {
            interp_controller(r)?;
            let n = r.u16()? as usize;
            r.skip(n * 4)?;
            Block::Other(kind.into())
        }
        "NiGeomMorpherController" => {
            // `NiTimeController`'s fields, to its target.
            r.skip(4 + 2 + 4 * 4)?;
            let target = r.i32()?;
            if r.since(V10_1_0_104) && r.until(V10_1_0_108) {
                r.skip(1)?; // managed
            }
            if r.since(V10_0_1_2) {
                r.skip(2)?; // morpher flags
            }
            let data = r.i32()?;
            r.skip(1)?; // always update
            let interpolators = if r.since(V10_1_0_106) { r.refs()? } else { Vec::new() };
            // (Bethesda's unknown ints, from 10.2 with a stream version past 9.)
            if r.since(V10_2_0_0) && r.bs > 9 {
                let unknown = r.u32()? as usize;
                r.skip(unknown * 4)?;
            }
            Block::Morpher { target, data, interpolators }
        }
        "NiMorphData" => {
            let n = r.u32()?;
            let vertices = r.u32()? as usize;
            r.skip(1)?; // relative targets (always 1 in the game's files)
            let mut morphs = Vec::new();
            for _ in 0..n {
                let name = if r.since(V10_1_0_106) { r.string()? } else { String::new() };
                if r.until(V10_1_0_0) {
                    // (Its own float keys, up to 10.1.)
                    let keys = r.u32()? as usize;
                    let interpolation = r.u32()?;
                    r.skip(keys * (8 + if interpolation == 2 { 8 } else if interpolation == 3 { 12 } else { 0 }))?;
                }
                if r.since(V10_1_0_104) && r.bs < 10 {
                    r.skip(4)?; // weight
                }
                let offsets = (0..vertices).map(|_| r.vec3()).collect::<Result<Vec<_>>>()?;
                morphs.push((name, offsets));
            }
            Block::MorphData { morphs }
        }
        "bhkBlendCollisionObject" => {
            let target = r.i32()?;
            r.skip(2)?;
            let body = r.i32()?;
            r.skip(8)?; // heir and velocity gain
            if r.bs < 9 {
                r.skip(8)?; // unknown
            }
            Block::CollisionObject { target, body }
        }
        "bhkHingeConstraint" => {
            r.skip(16 + 80)?;
            Block::Other(kind.into())
        }
        "bhkMalleableConstraint" => {
            r.skip(16)?;
            let sub = r.u32()?;
            r.skip(16)?;
            r.skip(match sub {
                0 => 32,
                1 => 80,
                2 => 124,
                6 => 140,
                7 => 120,
                8 => 36,
                _ => bail!("malleable constraint of type {sub}"),
            })?;
            r.skip(8)?; // tau, damping
            Block::Other(kind.into())
        }
        "bhkLimitedHingeConstraint" => {
            r.skip(16 + 124)?;
            Block::Other(kind.into())
        }
        "bhkPrismaticConstraint" => {
            r.skip(16 + 140)?;
            Block::Other(kind.into())
        }
        "bhkRagdollConstraint" => {
            r.skip(16 + 120)?;
            Block::Other(kind.into())
        }
        "bhkStiffSpringConstraint" => {
            r.skip(16 + 36)?;
            Block::Other(kind.into())
        }
        _ => return Ok(None),
    }))
}

impl Nif {
    pub fn parse(data: &[u8]) -> Result<Nif> {
        let Some(nl) = data.iter().position(|&c| c == b'\n') else { bail!("no header line") };
        let mut r = Reader { b: data, at: nl + 1, v: 0, bs: 0 };
        let version = r.u32()?;
        if ![V10_0_1_0, V10_0_1_2, V10_1_0_0, V10_1_0_101, V10_1_0_106, V10_2_0_0, V20_0_0_4, V20_0_0_5].contains(&version) {
            bail!("NIF version {version:#x} (only 10.0.1.0 to 10.2.0.0 and 20.0.0.4/5 are read)");
        }
        r.v = version;
        if r.since(0x1400_0003) {
            let _endian = r.u8()?;
        }
        let user = if r.since(0x0A00_0108) { r.u32()? } else { 0 };
        let count = r.u32()? as usize;
        // Bethesda's stream header (`nif.xml`'s `BSSTREAMHEADER`): its version, author and scripts.
        // 10.0.1.2 files have one though they've no user version to say so: it's known by its look.
        let stream_header = if version == V10_0_1_2 { r.stream_header_follows() } else { user >= 3 && (version == V20_0_0_5 || (r.since(V10_1_0_0) && version <= V20_0_0_4 && user <= 11)) };
        if stream_header {
            r.bs = r.u32()?;
            for _ in 0..3 {
                let n = r.u8()? as usize;
                r.skip(n)?;
            }
        }
        let ntypes = r.u16()? as usize;
        let types: Vec<String> = (0..ntypes).map(|_| r.string()).collect::<Result<_>>()?;
        let kinds: Vec<String> = (0..count).map(|_| Ok(types.get(r.u16()? as usize).cloned().unwrap_or_default())).collect::<Result<_>>()?;
        let _unknown = r.u32()?;
        let mut blocks = Vec::with_capacity(count);
        let mut stopped_at = None;
        for kind in &kinds {
            // (Up to 10.1.0.106 every block starts with a zero, but Bethesda's Havok blocks.)
            if version <= V10_1_0_106 && !kind.starts_with("bhk") && !kind.starts_with("hk") && r.u32()? != 0 {
                bail!("block {} ({kind}) doesn't start with a zero", blocks.len());
            }
            match block(&mut r, kind) {
                Ok(Some(b)) => blocks.push(b),
                Ok(None) => {
                    stopped_at = Some(kind.clone());
                    break;
                }
                Err(e) => bail!("block {} ({kind}): {e:#}", blocks.len()),
            }
        }
        let mut roots = Vec::new();
        if stopped_at.is_none() {
            let footer = r.at;
            let n = r.u32()? as usize;
            roots = (0..n).map(|_| r.i32()).collect::<Result<_>>()?;
            // (Some of the distant land's files were written over an older, longer one and keep its
            // end: the rest of its last block, then its footer, the same as this one.)
            let stale_end = data.ends_with(&data[footer..r.at]);
            if r.at != data.len() && !stale_end {
                bail!("{} bytes left after the footer: a block was misread", data.len() - r.at);
            }
        }
        Ok(Nif { version, blocks, kinds, roots, stopped_at })
    }

    /// The model as its morphs bend it `weight` of the way (a bow at full draw: 1): each morphed
    /// shape's vertices moved by its morphs past the first (the base) times `weight`. None when
    /// nothing in it morphs. (The game morphs positions only: the normals stay.)
    pub fn morphed(&self, weight: f32) -> Option<Nif> {
        let mut out = Nif { version: self.version, blocks: self.blocks.clone(), kinds: self.kinds.clone(), roots: self.roots.clone(), stopped_at: self.stopped_at.clone() };
        let mut any = false;
        for b in &self.blocks {
            let Block::Morpher { target, data, .. } = b else { continue };
            let (Some(Block::Shape { data: shape_data, .. }), Some(Block::MorphData { morphs })) = (self.get(*target), self.get(*data)) else { continue };
            let Some(Block::ShapeData(d)) = out.blocks.get_mut(*shape_data as usize) else { continue };
            let n = d.vertices.len();
            for (_, offsets) in morphs.iter().skip(1).filter(|(_, o)| o.len() == n) {
                for (v, o) in d.vertices.iter_mut().zip(offsets) {
                    *v += *o * weight;
                }
                any = true;
            }
        }
        any.then_some(out)
    }

    /// How a bow's limbs spring back as the game animates its shot (the player's `AttackBow`,
    /// this `.kf`): its morph `morph`'s weight from the text key `Hold` (full draw) on, as
    /// (seconds after the hold, the bend as a share of the bend at the hold).
    pub fn bow_spring(&self, morph: &str) -> Option<Vec<[f32; 2]>> {
        let s = self.sequences().into_iter().next()?;
        let at = |o: u32| self.sequence_name(s, o).unwrap_or_default();
        let hold = self.text_keys(s).iter().find(|(_, k)| k.eq_ignore_ascii_case("Hold"))?.0;
        let interpolator = s.blocks.iter().find(|(_, o)| at(o[1]) == "NiGeomMorpherController" && at(o[3]).eq_ignore_ascii_case(morph))?.0;
        let Some(Block::FloatInterpolator { data, .. }) = self.get(interpolator) else { return None };
        let Some(Block::FloatKeys(k)) = self.get(*data) else { return None };
        let full = k.iter().find(|(t, _)| (*t - hold).abs() < 1e-3).map(|(_, v)| *v).filter(|v| v.abs() > 1e-3)?;
        Some(k.iter().filter(|(t, _)| *t >= hold - 1e-3).map(|&(t, v)| [(t - hold).max(0.0), v / full]).collect())
    }

    pub fn get(&self, i: i32) -> Option<&Block> {
        if i < 0 { None } else { self.blocks.get(i as usize) }
    }

    pub fn av(&self, i: i32) -> Option<&AvObject> {
        match self.get(i)? {
            Block::Node { av, .. } | Block::Shape { av, .. } => Some(av),
            _ => None,
        }
    }

    /// Each block's parent node (-1 for the roots).
    pub fn parents(&self) -> Vec<i32> {
        let mut p = vec![-1; self.blocks.len()];
        for (i, b) in self.blocks.iter().enumerate() {
            if let Block::Node { children, .. } = b {
                for &c in children {
                    if c >= 0 && (c as usize) < p.len() {
                        p[c as usize] = i as i32;
                    }
                }
            }
        }
        p
    }

    /// A node's or shape's transform into the model's space (through all its parents).
    pub fn world(&self, i: i32, parents: &[i32]) -> Transform {
        let mut t = self.av(i).map(|a| a.transform.clone()).unwrap_or(Transform::IDENTITY);
        let mut p = parents.get(i as usize).copied().unwrap_or(-1);
        while p >= 0 {
            if let Some(a) = self.av(p) {
                t = t.then(&a.transform);
            }
            p = parents[p as usize];
        }
        t
    }

    /// The first node named `name` (case-insensitive).
    pub fn find(&self, name: &str) -> Option<i32> {
        self.blocks.iter().position(|b| matches!(b, Block::Node { av, .. } if av.name.eq_ignore_ascii_case(name))).map(|i| i as i32)
    }

    /// The model's animation named `name` (case-insensitive).
    pub fn sequence(&self, name: &str) -> Option<&Sequence> {
        self.blocks.iter().find_map(|b| match b {
            Block::Sequence(s) if s.name.eq_ignore_ascii_case(name) => Some(s),
            _ => None,
        })
    }

    /// Every animation's name.
    pub fn sequences(&self) -> Vec<&Sequence> {
        self.blocks.iter().filter_map(|b| if let Block::Sequence(s) = b { Some(s) } else { None }).collect()
    }

    /// The name at `offset` in a string palette.
    pub fn palette_name(&self, palette: i32, offset: u32) -> Option<String> {
        let Some(Block::StringPalette(p)) = self.get(palette) else { return None };
        name_at(p, offset)
    }

    /// The name at `offset` among a sequence's: in its string palette, or (files before 10.2) its own.
    pub fn sequence_name(&self, s: &Sequence, offset: u32) -> Option<String> {
        if s.palette < 0 { name_at(&s.names, offset) } else { self.palette_name(s.palette, offset) }
    }

    /// An animation's start and stop times (its keys' when it doesn't say).
    pub fn sequence_span(&self, s: &Sequence) -> (f32, f32) {
        let valid = |v: f32| v.is_finite() && v.abs() < 1e30;
        if valid(s.start) && valid(s.stop) && s.stop > s.start {
            return (s.start, s.stop);
        }
        let keys = s.blocks.iter().filter_map(|&(interp, _)| match self.get(interp) {
            Some(Block::TransformInterpolator { data, .. }) => match self.get(*data) {
                Some(Block::Keyframes(k)) => k.span(),
                _ => None,
            },
            _ => None,
        });
        keys.fold(None, |a: Option<(f32, f32)>, (lo, hi)| Some(a.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))))).unwrap_or((0.0, 0.0))
    }

    /// Where an animation has each node it moves at time `t`: (node name, local transform). A
    /// node's part the animation doesn't key stays as the node has it.
    pub fn sequence_pose(&self, s: &Sequence, t: f32) -> Vec<(String, Transform)> {
        let mut out = Vec::new();
        for &(interp, [offset, ..]) in &s.blocks {
            let Some(name) = self.sequence_name(s, offset) else { continue };
            let Some(Block::TransformInterpolator { translation, rotation, data, .. }) = self.get(interp) else { continue };
            let rest = self.find(&name).and_then(|i| self.av(i)).map(|a| a.transform.clone()).unwrap_or(Transform::IDENTITY);
            // (Unset parts of the interpolator's own pose are Gamebryo's "invalid": huge.)
            let valid = |v: f32| v.is_finite() && v.abs() < 1e30;
            let base_t = if [translation.x, translation.y, translation.z].iter().all(|&v| valid(v)) { *translation } else { rest.translation };
            let base_r = if rotation.iter().all(|&v| valid(v)) { Mat3::from_quat(glam::Quat::from_array(*rotation).normalize()) } else { rest.rotation };
            let keys = match self.get(*data) {
                Some(Block::Keyframes(k)) => Some(k),
                _ => None,
            };
            out.push((
                name,
                Transform {
                    rotation: keys.and_then(|k| k.rotation_at(t)).map(Mat3::from_quat).unwrap_or(base_r),
                    translation: keys.and_then(|k| k.translation_at(t)).unwrap_or(base_t),
                    scale: rest.scale,
                },
            ));
        }
        out
    }

    /// Where an animation has each node it moves at its start and at its end: (node name, local
    /// transform at the start, at the end).
    pub fn sequence_ends(&self, s: &Sequence) -> Vec<(String, Transform, Transform)> {
        let (t0, t1) = self.sequence_span(s);
        self.sequence_pose(s, t0).into_iter().zip(self.sequence_pose(s, t1)).map(|((n, a), (_, b))| (n, a, b)).collect()
    }

    /// An animation's text keys (its sounds: "Sound: DRSMetalOpen01" where it plays).
    pub fn text_keys(&self, s: &Sequence) -> &[(f32, String)] {
        match self.get(s.text_keys) {
            Some(Block::TextKeys(k)) => k,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_compose_and_invert() {
        let parent = Transform { rotation: Mat3::from_rotation_z(0.5), translation: Vec3::new(1.0, 2.0, 3.0), scale: 2.0 };
        let child = Transform { rotation: Mat3::from_rotation_x(0.3), translation: Vec3::new(0.5, 0.0, 0.0), scale: 1.0 };
        let p = Vec3::new(0.1, 0.2, 0.3);
        let both = child.then(&parent);
        assert!(both.apply(p).abs_diff_eq(parent.apply(child.apply(p)), 1e-5));
        assert!(both.inverse().apply(both.apply(p)).abs_diff_eq(p, 1e-5));
    }
}
