//! The creatures' capsules fitted to what they wear ([`fitted`]): how thick each part is, and how
//! far its free ends reach (a helmet's crown, a gauntlet's fingers, a boot's heel and toes, a
//! cuirass over the hips), from the models the game draws on the actor: its skinned body, armour and
//! clothes, and the head, hair and eyes on its head bone. A hand meets the armour where it's drawn,
//! not the bare body inside it.
//!
//! Each vertex goes with the bone that moves it most (its skin's biggest weight), placed where that
//! bone has it now, and the bone with a part of [`combat::HUMANOID`] ([`part_of`]: a twist bone with
//! its arm, a finger with its hand, the spine and the collarbones with the torso). Against its
//! part's capsule (from bone to bone) a vertex is so far off the axis and so far along it. A part is
//! as thick as nine in ten of its vertices are off its axis; the torso as deep as its front and back
//! are (a capsule as wide as the shoulders would stand a hand off the chest, and the arms hang at
//! its sides). A free end reaches its furthest vertices. An actor is fitted when its 3D or what it
//! wears changes; until then, and where there's nothing to fit, the table's own thickness holds.
//!
//! Engine facts (`Oblivion.exe` 1.2.0.416; xOBSE's `NiGeometry.h`, and `NiSkinData`'s loader
//! 0x72F420): `NiGeometry` +0xB4 its data and +0xB8 its skin instance; `NiGeometryData` +8 its
//! vertex count (u16) and +0x1C its vertices; `NiSkinInstance` +8 its data and +0x14 its bones
//! (`NiAVObject*`s); `NiSkinData` +0x40 its bone count and +0x44 its bones, 0x4C each: the skin to
//! bone transform at 0 (rotation rows, translation, scale), its weights at +0x44 (a u16 vertex and
//! an f32 weight, 8 bytes each) and their count at +0x48 (u16).

#[cfg(all(windows, target_arch = "x86"))]
pub use game::*;

/// A part fitted to what's drawn on it: its radius, and how far its capsule reaches on past its
/// start bone and past its end bone (metres, along it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub radius: f32,
    pub start: f32,
    pub end: f32,
}

#[cfg(all(windows, target_arch = "x86"))]
mod game {
    use super::*;
    use crate::engine::av;
    use crate::{camera, combat, mem};
    use glam::{Mat3, Vec3};
    use rustvr_protocol::PART_TORSO;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// A part is as thick as this share of its vertices are off its axis (the rest stick out: a
    /// pauldron's rim, a nose).
    const THICK: f32 = 0.9;
    /// A free end reaches this share of its vertices along it (past it, a stray vertex).
    const REACH: f32 = 0.98;
    /// A free end's round cap covers its furthest vertices off the axis too: it stops this share of the
    /// radius short of them.
    const CAP: f32 = 0.7;
    /// Fewer vertices than this on a part fit nothing.
    const MIN_VERTICES: usize = 24;
    /// A part fitted thinner or thicker than these (metres), or reaching further than the last, was
    /// misread: the table's own holds.
    const RADIUS_MIN: f32 = 0.02;
    const RADIUS_MAX: f32 = 0.4;
    const END_MAX: f32 = 0.4;

    /// The part of [`combat::HUMANOID`] a bone's skin goes with: the part that starts at it, else the
    /// part it's a helper or a piece of.
    fn part_of(name: &str) -> Option<usize> {
        let owner = match name {
            "Bip01 Spine" | "Bip01 Spine1" | "Bip01 Spine2" | "Bip01 L Clavicle" | "Bip01 R Clavicle" => "Bip01 Pelvis",
            "Bip01 Neck1" | "Bip01 Head" => "Bip01 Neck",
            "Bip01 L UpperArmTwist" | "Bip01 L Shoulder Helper" => "Bip01 L UpperArm",
            "Bip01 R UpperArmTwist" | "Bip01 R Shoulder Helper" => "Bip01 R UpperArm",
            "Bip01 L ForearmTwist" | "Bip01 L Wrist Helper" => "Bip01 L Forearm",
            "Bip01 R ForearmTwist" | "Bip01 R Wrist Helper" => "Bip01 R Forearm",
            "Bip01 L Toe0" => "Bip01 L Foot",
            "Bip01 R Toe0" => "Bip01 R Foot",
            n if n.starts_with("Bip01 L Finger") => "Bip01 L Hand",
            n if n.starts_with("Bip01 R Finger") => "Bip01 R Hand",
            n => n,
        };
        combat::HUMANOID.iter().position(|e| e.0 == owner)
    }

    /// Whether a part's capsule reaches on past its start bone and past its end bone to what's drawn
    /// there: the torso down over the hips, the head up through the crown, a hand to its fingertips, a
    /// foot to its heel and its toes. (At a joint the next part takes over.)
    fn free_ends(from: &str, to: &str) -> (bool, bool) {
        (from == "Bip01 Pelvis" || from.ends_with(" Foot"), to == "Bip01 Head" || to.contains("Finger") || to.ends_with("Toe0"))
    }

    /// A part's vertices: how far along its axis from its start bone, and how far off it (metres).
    #[derive(Default)]
    struct Samples {
        along: Vec<f32>,
        off: Vec<f32>,
    }

    /// The value `q` of the way up `v` (sorted in place).
    fn percentile(v: &mut [f32], q: f32) -> f32 {
        v.sort_by(f32::total_cmp);
        v[((v.len() - 1) as f32 * q).round() as usize]
    }

    /// A part from bone to bone `len` apart fitted to its vertices, free at its start and its end as
    /// [`free_ends`] says.
    fn fit_part(s: &mut Samples, len: f32, (free_start, free_end): (bool, bool)) -> Option<Fit> {
        if s.along.len() < MIN_VERTICES {
            return None;
        }
        let radius = percentile(&mut s.off, THICK);
        if !(RADIUS_MIN..=RADIUS_MAX).contains(&radius) {
            return None;
        }
        let start = if free_start { -percentile(&mut s.along, 1.0 - REACH) - CAP * radius } else { 0.0 };
        let end = if free_end { percentile(&mut s.along, REACH) - len - CAP * radius } else { 0.0 };
        (start <= END_MAX && end <= END_MAX).then_some(Fit { radius, start: start.max(0.0), end: end.max(0.0) })
    }

    const NAME: usize = 0x08;
    const CHILDREN: usize = 0xB0;
    const CHILD_COUNT: usize = 0xB6;
    const VT_RTTI: usize = 0x04;
    const GEOM_DATA: usize = 0xB4;
    const GEOM_SKIN: usize = 0xB8;
    const DATA_VERTEX_COUNT: usize = 0x08;
    const DATA_VERTICES: usize = 0x1C;
    const SKIN_DATA: usize = 0x08;
    const SKIN_BONES: usize = 0x14;
    const SKIN_BONE_COUNT: usize = 0x40;
    const SKIN_BONE_DATA: usize = 0x44;
    const BONE_DATA_SIZE: usize = 0x4C;
    const BONE_WEIGHTS: usize = 0x44;
    const BONE_WEIGHT_COUNT: usize = 0x48;
    const CODE: std::ops::Range<usize> = 0x0040_1000..0x00A3_0000;
    /// A model whose vertices land further than this from their bones on average (metres) was
    /// misread: it's left out.
    const BONE_REACH: f32 = 0.6;
    /// What an actor wears is looked at again every this many frames (a helmet put on, armour
    /// taken off).
    const RECHECK: u32 = 30;

    /// One of a bone's weights in a skin: a vertex, and how much the bone moves it.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Weight {
        vertex: u16,
        _pad: u16,
        weight: f32,
    }

    /// A transform as Gamebryo keeps one (game units): rotation, translation, scale.
    #[derive(Clone, Copy)]
    struct Xf {
        r: Mat3,
        t: Vec3,
        s: f32,
    }

    impl Xf {
        /// The transform at `p` (rotation rows, translation, scale: an `NiTransform`).
        fn read(p: usize) -> Option<Xf> {
            let r: [f32; 9] = mem::at(p)?;
            let t: [f32; 3] = mem::at(p + 0x24)?;
            let s: f32 = mem::at(p + 0x30)?;
            let xf = Xf { r: crate::items::ni_matrix(r), t: Vec3::from(t), s };
            (xf.r.is_finite() && xf.t.is_finite() && s.is_finite()).then_some(xf)
        }

        /// A node's world transform.
        fn world(node: usize) -> Option<Xf> {
            // (An `NiAVObject`'s world transform is an `NiTransform` like any other.)
            Xf::read(node + av::WORLD_ROTATE)
        }

        fn apply(&self, p: Vec3) -> Vec3 {
            self.r * (p * self.s) + self.t
        }
    }

    /// `n` values of `T` at `addr` in the game's memory, read at once.
    fn slice<T: Copy>(addr: usize, n: usize) -> Option<Vec<T>> {
        let bytes = n.checked_mul(std::mem::size_of::<T>())?;
        if n == 0 || !mem::is_readable(addr as *const u8, bytes) {
            return None;
        }
        let mut v = Vec::<T>::with_capacity(n);
        unsafe {
            std::ptr::copy_nonoverlapping(addr as *const u8, v.as_mut_ptr() as *mut u8, bytes);
            v.set_len(n);
        }
        Some(v)
    }

    /// Whether `obj` is a `kind` (or one of its kinds), by its `NiRTTI` chain; known by vtable.
    fn is_a(obj: usize, kind: &'static str) -> bool {
        static KNOWN: Mutex<Option<HashMap<(usize, &'static str), bool>>> = Mutex::new(None);
        let vt = mem::deref(obj);
        if vt == 0 {
            return false;
        }
        let mut known = KNOWN.lock().unwrap();
        let known = known.get_or_insert_with(HashMap::new);
        if let Some(&k) = known.get(&(vt, kind)) {
            return k;
        }
        let f = mem::deref(vt + VT_RTTI);
        let mut yes = false;
        if CODE.contains(&f) {
            type Rtti = unsafe extern "thiscall" fn(usize) -> usize;
            let mut rtti = unsafe { std::mem::transmute::<usize, Rtti>(f)(obj) };
            for _ in 0..16 {
                if rtti == 0 {
                    break;
                }
                if combat::read_str(mem::deref(rtti)) == kind {
                    yes = true;
                    break;
                }
                rtti = mem::deref(rtti + 4);
            }
        }
        known.insert((vt, kind), yes);
        yes
    }

    /// The models drawn on an actor (its 3D `root`): the skinned ones, and the rigid ones on its
    /// head bone (its head, hair and eyes), each with that bone (0 for a skinned one). Hidden ones
    /// aren't drawn; particles and lines aren't solid.
    fn models(root: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut stack = vec![(root, 0usize, 0u32)];
        let mut seen = 0;
        while let Some((obj, head, depth)) = stack.pop() {
            seen += 1;
            if seen > 1500 || depth > 48 {
                break;
            }
            if mem::at::<u16>(obj + av::FLAGS).is_none_or(|f| f & av::APP_CULLED != 0) {
                continue;
            }
            if is_a(obj, "NiTriBasedGeom") {
                if mem::deref(obj + GEOM_SKIN) != 0 {
                    out.push((obj, 0));
                } else if head != 0 {
                    out.push((obj, head));
                }
                continue;
            }
            if !is_a(obj, "NiNode") {
                continue;
            }
            let head = if head == 0 && combat::read_str(mem::deref(obj + NAME)) == "Bip01 Head" { obj } else { head };
            let data = mem::deref(obj + CHILDREN);
            let n = mem::at::<u16>(obj + CHILD_COUNT).unwrap_or(0).min(256) as usize;
            for i in 0..n {
                let c = mem::deref(data + i * 4);
                if c != 0 {
                    stack.push((c, head, depth + 1));
                }
            }
        }
        out
    }

    /// Each part of [`combat::HUMANOID`] fitted to the `models` drawn on an actor with these `bones`
    /// (by name), and how many vertices they had.
    fn fit(bones: &HashMap<String, usize>, models: &[(usize, usize)]) -> (Vec<Option<Fit>>, usize) {
        let table = combat::HUMANOID;
        // Each part's axis now: its start, its direction and its length.
        let axes: Vec<Option<(Vec3, Vec3, f32)>> = table
            .iter()
            .map(|&(from, to, ..)| {
                let a = bones.get(from).and_then(|&n| combat::node_pos(n))?;
                let b = bones.get(to).and_then(|&n| combat::node_pos(n))?;
                let len = a.distance(b);
                (len > 1e-3).then(|| (a, (b - a) / len, len))
            })
            .collect();
        // The torso's front, square to its axis and to the line across its shoulders.
        let at = |name: &str| bones.get(name).and_then(|&n| combat::node_pos(n));
        let across = at("Bip01 R Clavicle").zip(at("Bip01 L Clavicle")).or_else(|| at("Bip01 R UpperArm").zip(at("Bip01 L UpperArm"))).map(|(r, l)| r - l);
        let torso = table.iter().position(|e| e.3 == PART_TORSO);
        let front = torso.and_then(|i| axes[i]).zip(across).and_then(|((_, dir, _), across)| dir.cross(across).try_normalize());
        let part_of_node: HashMap<usize, usize> = bones.iter().filter_map(|(name, &n)| Some((n, part_of(name)?))).collect();
        let mut samples: Vec<Samples> = table.iter().map(|_| Samples::default()).collect();
        let mut add = |part: usize, p: Vec3| {
            let Some((a, dir, _)) = axes[part] else { return };
            let rel = p - a;
            let along = rel.dot(dir);
            let perp = rel - dir * along;
            let off = match front.filter(|_| Some(part) == torso) {
                Some(f) => perp.dot(f).abs(),
                None => perp.length(),
            };
            samples[part].along.push(along);
            samples[part].off.push(off);
        };
        let basis = camera::basis();
        let mut vertices = 0;
        for &(geom, head) in models {
            let data = mem::deref(geom + GEOM_DATA);
            let n = mem::at::<u16>(data + DATA_VERTEX_COUNT).unwrap_or(0) as usize;
            let Some(points) = slice::<[f32; 3]>(mem::deref(data + DATA_VERTICES), n) else { continue };
            if head != 0 {
                // On the head bone: where the model has them.
                let (Some(xf), Some(&part)) = (Xf::world(geom), part_of_node.get(&head)) else { continue };
                for p in &points {
                    add(part, basis.point_from_game(xf.apply(Vec3::from(*p))));
                }
                vertices += n;
                continue;
            }
            let skin = mem::deref(geom + GEOM_SKIN);
            let skin_data = mem::deref(skin + SKIN_DATA);
            let count = mem::at::<u32>(skin_data + SKIN_BONE_COUNT).unwrap_or(0).min(256) as usize;
            let bone_data = mem::deref(skin_data + SKIN_BONE_DATA);
            let Some(nodes) = slice::<u32>(mem::deref(skin + SKIN_BONES), count) else { continue };
            // Each vertex's bone: the one that moves it most.
            let mut best = vec![(0.0f32, 0usize); n];
            for i in 0..count {
                let bone = bone_data + i * BONE_DATA_SIZE;
                let k = mem::at::<u16>(bone + BONE_WEIGHT_COUNT).unwrap_or(0) as usize;
                let Some(weights) = slice::<Weight>(mem::deref(bone + BONE_WEIGHTS), k) else { continue };
                for w in weights {
                    if let Some(b) = best.get_mut(w.vertex as usize)
                        && w.weight > b.0
                    {
                        *b = (w.weight, i);
                    }
                }
            }
            // Each bone where it is now, after the skin's own transform into it, and its part.
            let bones_now: Vec<Option<(Xf, Xf, Option<usize>)>> = (0..count)
                .map(|i| {
                    let node = nodes[i] as usize;
                    Some((Xf::read(bone_data + i * BONE_DATA_SIZE)?, Xf::world(node)?, part_of_node.get(&node).copied()))
                })
                .collect();
            let placed: Vec<(Vec3, f32, usize)> = points
                .iter()
                .zip(&best)
                .filter(|(_, b)| b.0 > 0.0)
                .filter_map(|(p, &(_, i))| {
                    let (to_bone, world, _) = bones_now[i]?;
                    let at = world.apply(to_bone.apply(Vec3::from(*p)));
                    Some((at, at.distance(world.t) / camera::UNITS_PER_METER, i))
                })
                .collect();
            // (Misread, another layout or another convention, the vertices land far from their
            // bones: that model's left out.)
            let reach = placed.iter().map(|x| x.1).sum::<f32>() / placed.len().max(1) as f32;
            if placed.is_empty() || reach > BONE_REACH {
                tracing::info!("shapes: a model ({geom:#x}, {n} vertices, {count} bones) left out: its vertices {reach:.2} m from their bones");
                continue;
            }
            for (at, _, i) in placed {
                if let Some((_, _, Some(part))) = bones_now[i] {
                    add(part, basis.point_from_game(at));
                }
            }
            vertices += n;
        }
        let fits = table
            .iter()
            .zip(&axes)
            .zip(samples.iter_mut())
            .map(|((&(from, to, ..), axis), s)| fit_part(s, axis.as_ref()?.2, free_ends(from, to)))
            .collect();
        (fits, vertices)
    }

    /// An actor's parts as fitted, and what they were fitted to.
    struct Fitted {
        root: usize,
        models: Vec<(usize, usize)>,
        calls: u32,
        parts: Vec<Option<Fit>>,
    }

    static FITTED: Mutex<Option<HashMap<u32, Fitted>>> = Mutex::new(None);

    /// Each part of [`combat::HUMANOID`] on actor `id` (its 3D `root`, its `bones` by name) fitted to
    /// what's drawn on it; `None` for a part there's nothing to fit to.
    pub fn fitted(id: u32, root: usize, bones: &HashMap<String, usize>) -> Vec<Option<Fit>> {
        let mut all = FITTED.lock().unwrap();
        let all = all.get_or_insert_with(HashMap::new);
        let mut drawn = None;
        if let Some(f) = all.get_mut(&id)
            && f.root == root
        {
            f.calls += 1;
            if f.calls % RECHECK != 0 {
                return f.parts.clone();
            }
            let now = models(root);
            if now == f.models {
                return f.parts.clone();
            }
            drawn = Some(now);
        }
        let models = drawn.unwrap_or_else(|| models(root));
        let (parts, vertices) = fit(bones, &models);
        let said: Vec<String> = combat::HUMANOID
            .iter()
            .zip(&parts)
            .filter_map(|(e, f)| f.map(|f| format!("{} {:.3} (+{:.2} +{:.2})", e.0.trim_start_matches("Bip01 "), f.radius, f.start, f.end)))
            .collect();
        tracing::info!("shapes: {id:#x} fitted to {} models ({vertices} vertices): {}", models.len(), said.join(", "));
        all.insert(id, Fitted { root, models, calls: 0, parts: parts.clone() });
        parts
    }

    /// Forgets the actors no longer near (`near`: the ones that are, by id).
    pub fn forget_others<V>(near: &HashMap<u32, V>) {
        if let Some(all) = FITTED.lock().unwrap().as_mut() {
            all.retain(|id, _| near.contains_key(id));
        }
    }
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn fitted(_id: u32, _root: usize, _bones: &std::collections::HashMap<String, usize>) -> Vec<Option<Fit>> {
    Vec::new()
}

#[cfg(not(all(windows, target_arch = "x86")))]
pub fn forget_others<V>(_near: &std::collections::HashMap<u32, V>) {}
