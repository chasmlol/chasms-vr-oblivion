//! Oblivion's models: NetImmerse/Gamebryo `.nif` files, version 20.0.0.4 or 20.0.0.5 (user version
//! 11). Those versions store no block sizes, so every block a file holds has to be read field by
//! field: this reader knows the blocks Oblivion's weapons, armour, clothes, bodies and hands use
//! (layouts from the NifTools project's format description, `nif.xml`) and stops at the first one
//! it doesn't, keeping the blocks before it.
//!
//! What's kept: the node tree (names, local transforms, children), triangle shapes (vertices,
//! normals, UVs, vertex colours, triangles from lists or strips), their skinning (bones, bind
//! transforms, weights), their textures and materials, extra data, and the Havok collision (rigid
//! bodies: mass, centre, shapes).

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
    BinaryExtra { name: String },
    CollisionObject { target: i32, body: i32 },
    RigidBody(RigidBody),
    BoxShape { dims: Vec3, radius: f32 },
    CapsuleShape { a: Vec3, b: Vec3, radius: f32 },
    SphereShape { radius: f32 },
    ConvexVertices { vertices: Vec<Vec3>, radius: f32 },
    ListShape { shapes: Vec<i32> },
    ConvexTransform { shape: i32, transform: [f32; 16] },
    /// Read past: of no use here (stencil, vertex colour, z-buffer, specular properties,
    /// constraints).
    Other(String),
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
    pub rotation: [f32; 4],
    pub center: [f32; 4],
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    /// `bhkRigidBodyT`: translation and rotation apply.
    pub transformed: bool,
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
}

impl Reader<'_> {
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

fn tex_desc(r: &mut Reader) -> Result<i32> {
    let source = r.i32()?;
    r.skip(4 + 4 + 4)?; // clamp, filter, UV set
    if r.bool()? {
        r.skip(8 + 8 + 4 + 4 + 8)?; // translation, scale, rotation, method, centre
    }
    Ok(source)
}

fn geometry_data(r: &mut Reader) -> Result<ShapeData> {
    let _group = r.i32()?;
    let n = r.u16()? as usize;
    r.skip(2)?; // keep, compress
    let mut d = ShapeData::default();
    if r.bool()? {
        d.vertices = (0..n).map(|_| r.vec3()).collect::<Result<_>>()?;
    }
    let flags = r.u16()?;
    if r.bool()? {
        d.normals = (0..n).map(|_| r.vec3()).collect::<Result<_>>()?;
        if flags & 4096 != 0 {
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
    let _additional = r.i32()?;
    Ok(d)
}

fn tri_shape_data(r: &mut Reader) -> Result<ShapeData> {
    let mut d = geometry_data(r)?;
    let tris = r.u16()? as usize;
    let _points = r.u32()?;
    if r.bool()? {
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
    let mut d = geometry_data(r)?;
    let _tris = r.u16()?;
    let strips = r.u16()? as usize;
    let lengths: Vec<usize> = (0..strips).map(|_| Ok(r.u16()? as usize)).collect::<Result<_>>()?;
    if r.bool()? {
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
        if r.bool()? {
            r.skip(verts * 2)?;
        }
        if r.bool()? {
            r.skip(verts * per_vertex * 4)?;
        }
        let lengths: Vec<usize> = (0..strips).map(|_| Ok(r.u16()? as usize)).collect::<Result<_>>()?;
        if r.bool()? {
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

fn havok_material(r: &mut Reader) -> Result<()> {
    r.skip(4)
}

fn block(r: &mut Reader, kind: &str) -> Result<Option<Block>> {
    Ok(Some(match kind {
        "NiNode" | "BSFadeNode" | "AvoidNode" | "RootCollisionNode" | "NiBSAnimationNode" | "NiBSParticleNode" | "NiBillboardNode" => {
            let av = av_object(r)?;
            let children = r.refs()?;
            let _effects = r.refs()?;
            if kind == "NiBillboardNode" {
                r.skip(2)?; // billboard mode
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
            let partition = r.i32()?;
            let root = r.i32()?;
            let bones = r.refs()?;
            Block::SkinInstance { data, partition, root, bones }
        }
        "NiSkinData" => {
            let transform = r.transform()?;
            let n = r.u32()? as usize;
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
                r.skip(4)?; // unknown ref
                f
            } else {
                r.string()?;
                r.skip(4)?; // pixel data
                String::new()
            };
            r.skip(12 + 1 + 1)?; // format prefs, static, direct render
            Block::SourceTexture { file }
        }
        "NiMaterialProperty" => {
            let (name, _) = object_net(r)?;
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
            r.skip(1 + 4 + 4 + 4 + 4 + 4 + 4 + 4)?;
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
        "NiSpecularProperty" | "NiWireframeProperty" | "NiDitherProperty" | "NiShadeProperty" => {
            object_net(r)?;
            r.skip(2)?;
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
            r.skip(n)?;
            Block::BinaryExtra { name }
        }
        "BSBound" => {
            r.string()?;
            r.skip(24)?;
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
            let filter = r.u32()?;
            r.skip(20 + 4)?; // world object info, entity info
            let info = r.take(196)?.to_vec();
            let f = |at: usize| f32::from_le_bytes(info[at..at + 4].try_into().unwrap());
            let v4 = |at: usize| [f(at), f(at + 4), f(at + 8), f(at + 12)];
            let body = RigidBody {
                shape,
                layer: filter as u8,
                translation: v4(20),
                rotation: v4(36),
                center: v4(132),
                mass: f(148),
                friction: f(160),
                restitution: f(164),
                transformed: kind == "bhkRigidBodyT",
            };
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
        "bhkConvexVerticesShape" => {
            havok_material(r)?;
            let radius = r.f32()?;
            r.skip(24)?;
            let n = r.u32()? as usize;
            let vertices = (0..n).map(|_| r.vec4().map(|v| Vec3::new(v[0], v[1], v[2]))).collect::<Result<_>>()?;
            let normals = r.u32()? as usize;
            r.skip(normals * 16)?;
            Block::ConvexVertices { vertices, radius }
        }
        "bhkListShape" => {
            let shapes = r.refs()?;
            havok_material(r)?;
            r.skip(24)?;
            let filters = r.u32()? as usize;
            r.skip(filters * 4)?;
            Block::ListShape { shapes }
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
        "NiTransformController" | "NiVisController" | "NiAlphaController" => {
            time_controller(r)?;
            r.skip(4)?; // interpolator
            Block::Other(kind.into())
        }
        "NiTransformInterpolator" => {
            r.skip(32 + 4)?;
            Block::Other(kind.into())
        }
        "NiFloatInterpolator" => {
            r.skip(4 + 4)?;
            Block::Other(kind.into())
        }
        "NiTransformData" | "NiKeyframeData" => {
            let n = r.u32()?;
            if n != 0 {
                let kind = r.u32()?;
                if kind == 4 {
                    for _ in 0..3 {
                        key_group(r, 1)?;
                    }
                } else {
                    let each = match kind {
                        3 => 4 + 16 + 12,
                        _ => 4 + 16,
                    };
                    r.skip(n as usize * each)?;
                }
            }
            key_group(r, 3)?;
            key_group(r, 1)?;
            Block::Other(kind.into())
        }
        "NiFloatData" => {
            key_group(r, 1)?;
            Block::Other(kind.into())
        }
        "NiTextKeyExtraData" => {
            r.string()?;
            let n = r.u32()?;
            for _ in 0..n {
                r.skip(4)?;
                r.string()?;
            }
            Block::Other(kind.into())
        }
        "bhkBlendCollisionObject" => {
            let target = r.i32()?;
            r.skip(2)?;
            let body = r.i32()?;
            r.skip(8)?;
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
        _ => return Ok(None),
    }))
}

impl Nif {
    pub fn parse(data: &[u8]) -> Result<Nif> {
        let Some(nl) = data.iter().position(|&c| c == b'\n') else { bail!("no header line") };
        let mut r = Reader { b: data, at: nl + 1 };
        let version = r.u32()?;
        if version != 0x1400_0004 && version != 0x1400_0005 {
            bail!("NIF version {version:#x} (only 20.0.0.4/5 are read)");
        }
        let _endian = r.u8()?;
        let _user = r.u32()?;
        let count = r.u32()? as usize;
        let _user2 = r.u32()?;
        for _ in 0..3 {
            let n = r.u8()? as usize;
            r.skip(n)?;
        }
        let ntypes = r.u16()? as usize;
        let types: Vec<String> = (0..ntypes).map(|_| r.string()).collect::<Result<_>>()?;
        let kinds: Vec<String> = (0..count).map(|_| Ok(types.get(r.u16()? as usize).cloned().unwrap_or_default())).collect::<Result<_>>()?;
        let _unknown = r.u32()?;
        let mut blocks = Vec::with_capacity(count);
        let mut stopped_at = None;
        for kind in &kinds {
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
            let n = r.u32()? as usize;
            roots = (0..n).map(|_| r.i32()).collect::<Result<_>>()?;
            if r.at != data.len() {
                bail!("{} bytes left after the footer: a block was misread", data.len() - r.at);
            }
        }
        Ok(Nif { version, blocks, kinds, roots, stopped_at })
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
