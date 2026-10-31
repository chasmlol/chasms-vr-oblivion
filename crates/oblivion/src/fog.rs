//! Oblivion's fog by how far away a thing is.
//!
//! The game's vertex shaders don't fog a vertex by its distance. They fog it by the length of its
//! clip-space position: x, y and depth (`SLS2000.vso` takes the length `d` of the position's
//! `ModelViewProj` x, y and z, and fogs by `(FogParam.x - d) / FogParam.y`). That's the depth times
//! `sqrt(1 + ndc.x² + ndc.y²)`, so it grows towards the frustum's edges, and is the distance only in
//! a symmetric 90 degree frustum. A flat player's 75 degrees barely shows it. The headset's eyes are
//! off-centre, mirrored and wide, and wider again overscanned:
//!
//! * the two eyes fogged a far hill off to the side differently;
//! * a hill's haze changed as the head turned towards it or away;
//! * the overscan thinned the fog;
//! * and the land didn't meet the water, whose pixel shader fogs by the distance (`EyePos`).
//!
//! As the game makes each vertex shader (`crate::d3d9`'s `CreateVertexShader`), [`rewrite`] makes
//! its fog's distance the vertex's view-space distance. The clip position's x and y go back through
//! the projection with the depth (`w`, kept by one more `dp4` beside the clip position's rows):
//! `x = cx·(r−l)/2 + w·(r+l)/2`, `y = cy·(t−b)/2 + w·(t+b)/2`, and the distance is `|(x, y, w)|`.
//! The frustum's terms ([`terms`]) go in a constant register no game shader uses ([`REGISTER`]), set
//! for each camera the adapter renders with (`crate::render`).

use std::sync::atomic::{AtomicU32, Ordering};

static REWRITTEN: AtomicU32 = AtomicU32::new(0);

/// One more of the game's vertex shaders made with its fog by distance: how many now.
pub fn rewritten() -> u32 {
    REWRITTEN.fetch_add(1, Ordering::Relaxed) + 1
}

/// The game's shaders fog by distance (they're made so): the runtime needn't even out its eyes' fog.
pub fn by_distance() -> bool {
    REWRITTEN.load(Ordering::Relaxed) > 0
}

/// The constant register the frustum's terms go in: past every constant the game's shaders use (they
/// stop at `c95`, the most a `vs_1_1` shader can count on), the last of the 256 a device has.
pub const REGISTER: u32 = 255;
/// The temp the view-space position is made in: past every temp the game's shaders use (`r8`).
const TEMP: u32 = 11;

const END: u32 = 0x0000_FFFF;
const COMMENT: u32 = 0xFFFE;
const CTAB: u32 = u32::from_le_bytes(*b"CTAB");
const OP_ADD: u32 = 2;
const OP_MAD: u32 = 4;
const OP_MUL: u32 = 5;
const OP_RCP: u32 = 6;
const OP_RSQ: u32 = 7;
const OP_DP3: u32 = 8;
const OP_DP4: u32 = 9;
const OP_DCL: u32 = 31;
const OP_DEF: u32 = 81;
/// Flow control (`call` .. `label`, `rep` .. `breakc`): a shader with any isn't rewritten (its writes
/// aren't in order).
fn flow(op: u32) -> bool {
    (25..=30).contains(&op) || (38..=45).contains(&op)
}
const PREDICATED: u32 = 1 << 28;
const TYPE_TEMP: u32 = 0;
const TYPE_CONST: u32 = 2;
/// The swizzle that reads each component as itself (x in the low bits).
const XYZW: u32 = 0b11_10_01_00;
/// A source modifier's bits, and a register's relative addressing.
const MODIFIER: u32 = 0xF << 24;
const RELATIVE: u32 = 1 << 13;

/// A register parameter's token.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Param(u32);

impl Param {
    fn new(kind: u32, num: u32, bits: u32) -> Param {
        Param(0x8000_0000 | ((kind & 7) << 28) | ((kind & 0x18) << 8) | num | (bits << 16))
    }
    fn kind(self) -> u32 {
        ((self.0 >> 28) & 7) | ((self.0 >> 8) & 0x18)
    }
    fn num(self) -> u32 {
        self.0 & 0x7FF
    }
    /// A destination's write mask (x in bit 0).
    fn mask(self) -> u32 {
        (self.0 >> 16) & 0xF
    }
    /// A source's swizzle.
    fn swizzle(self) -> u32 {
        (self.0 >> 16) & 0xFF
    }
    /// A source that reads one component of a temp, plainly: which.
    fn temp_scalar(self) -> Option<(u32, u32)> {
        let s = self.swizzle();
        (self.kind() == TYPE_TEMP && self.0 & (MODIFIER | RELATIVE) == 0 && s == (s & 3) * 0b01_01_01_01).then_some((self.num(), s & 3))
    }
    /// A source that reads all of a register as it is.
    fn plain(self, kind: u32) -> Option<u32> {
        (self.kind() == kind && self.0 & (MODIFIER | RELATIVE) == 0 && self.swizzle() == XYZW).then_some(self.num())
    }
}

/// One instruction: where its token is, how many tokens follow it, its opcode.
#[derive(Clone, Copy, Debug)]
struct Ins {
    at: usize,
    len: usize,
    op: u32,
}

/// How many tokens follow each `vs_1_x` instruction (they don't say).
fn sm1_params(op: u32) -> Option<usize> {
    Some(match op {
        0 => 0,
        1 | 6 | 7 | 14 | 15 | 16 | 19 | 31 | 78 | 79 => 2,
        2 | 3 | 5 | 8..=13 | 17 | 20..=24 => 3,
        4 | 18 => 4,
        81 => 5,
        _ => return None,
    })
}

/// A vertex shader's instructions, and its length in tokens (with its end).
fn walk(read: impl Fn(usize) -> Option<u32>) -> Option<(Vec<Ins>, usize)> {
    let version = read(0)?;
    if version >> 16 != 0xFFFE {
        return None;
    }
    let major = (version >> 8) & 0xFF;
    let mut out = Vec::new();
    let mut i = 1;
    loop {
        let t = read(i)?;
        if t == END {
            return Some((out, i + 1));
        }
        let op = t & 0xFFFF;
        let len = if op == COMMENT {
            ((t >> 16) & 0x7FFF) as usize
        } else if major >= 2 {
            ((t >> 24) & 0xF) as usize
        } else {
            sm1_params(op)?
        };
        out.push(Ins { at: i, len, op });
        i += 1 + len;
    }
}

/// The length in tokens of the vertex shader at `code` (as the game hands it to
/// `CreateVertexShader`, which doesn't say), if it reads as one.
///
/// # Safety
/// `code` points at a shader's tokens.
pub unsafe fn length(code: *const u32) -> Option<usize> {
    // (No shader of the game's is near a quarter of this.)
    const MOST: usize = 1 << 16;
    walk(|i| (i < MOST).then(|| unsafe { code.add(i).read_unaligned() })).map(|(_, n)| n)
}

/// The float constants in a shader's constant table: name, first register, how many.
fn constants(code: &[u32], ins: &[Ins]) -> Vec<(String, u32, u32)> {
    let Some(c) = ins.iter().find(|i| i.op == COMMENT && i.len > 1 && code[i.at + 1] == CTAB) else { return Vec::new() };
    let bytes: Vec<u8> = code[c.at + 2..c.at + 1 + c.len].iter().flat_map(|t| t.to_le_bytes()).collect();
    let u32_at = |o: usize| bytes.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let u16_at = |o: usize| bytes.get(o..o + 2).map(|b| u16::from_le_bytes(b.try_into().unwrap()));
    let (Some(count), Some(info)) = (u32_at(12), u32_at(16)) else { return Vec::new() };
    let mut out = Vec::new();
    for k in 0..count as usize {
        let e = info as usize + k * 20;
        let (Some(name), Some(set), Some(index), Some(n)) = (u32_at(e), u16_at(e + 4), u16_at(e + 6), u16_at(e + 8)) else { break };
        // (Register set 2: float4.)
        if set != 2 {
            continue;
        }
        let name = bytes.get(name as usize..).map(|s| s.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect()).unwrap_or_default();
        out.push((name, index as u32, n as u32));
    }
    out
}

/// The last instruction before `before` writing component `c` of temp `r`.
fn writer(code: &[u32], ins: &[Ins], before: usize, r: u32, c: u32) -> Option<usize> {
    (0..before).rev().find(|&k| {
        let i = ins[k];
        let d = Param(code[i.at + 1]);
        i.op != COMMENT && i.op != OP_DCL && i.op != OP_DEF && i.len > 0 && d.kind() == TYPE_TEMP && d.num() == r && d.mask() & (1 << c) != 0
    })
}

/// Every register parameter of the shader (not `def`'s numbers, nor `dcl`'s usage).
fn params<'a>(code: &'a [u32], ins: &'a [Ins]) -> impl Iterator<Item = Param> + 'a {
    ins.iter().filter(|i| i.op != COMMENT).flat_map(move |i| {
        let first = if i.op == OP_DCL { 1 } else { 0 };
        let last = if i.op == OP_DEF { 1 } else { i.len };
        (first..last).map(move |k| Param(code[i.at + 1 + k]))
    })
}

/// `code` (a vertex shader's tokens) with its fog by the vertex's distance: `Ok(None)` for a shader
/// without fog, an `Err` saying why for one whose fog isn't the game's usual.
pub fn rewrite(code: &[u32]) -> Result<Option<Vec<u32>>, &'static str> {
    let (ins, end) = walk(|i| code.get(i).copied()).ok_or("not a vertex shader")?;
    let table = constants(code, &ins);
    let Some(&(_, fog, _)) = table.iter().find(|(n, ..)| n == "FogParam") else { return Ok(None) };
    if ins.iter().any(|i| i.op != COMMENT && (flow(i.op) || code[i.at] & PREDICATED != 0)) {
        return Err("flow control");
    }
    let src = |k: usize, n: usize| Param(code[ins[k].at + 2 + n]);
    // The fog: far less the distance (`add r, -d, FogParam.x`), the first that reads it.
    let reads_fog = |k: usize| ins[k].op != COMMENT && ins[k].op != OP_DCL && ins[k].op != OP_DEF && (1..ins[k].len).any(|n| Param(code[ins[k].at + 1 + n]).kind() == TYPE_CONST && Param(code[ins[k].at + 1 + n]).num() == fog);
    let add = (0..ins.len()).find(|&k| reads_fog(k)).ok_or("no use of FogParam")?;
    if ins[add].op != OP_ADD || ins[add].len != 3 {
        return Err("FogParam not first taken from");
    }
    let (a, b) = (src(add, 0), src(add, 1));
    let distance = if b.kind() == TYPE_CONST { a } else { b };
    let far = if b.kind() == TYPE_CONST { b } else { a };
    if distance.0 & MODIFIER != 1 << 24 || far.num() != fog || far.swizzle() != 0 {
        return Err("not far less the distance");
    }
    let (r, c) = Param(distance.0 & !MODIFIER).temp_scalar().ok_or("the distance isn't a temp")?;
    // The distance: the square root of the clip position's length squared (`rcp`, `rsq`, `dp3`).
    let rcp = writer(code, &ins, add, r, c).filter(|&k| ins[k].op == OP_RCP).ok_or("the distance isn't an rcp")?;
    let (r, c) = src(rcp, 0).temp_scalar().ok_or("rcp of what")?;
    let rsq = writer(code, &ins, rcp, r, c).filter(|&k| ins[k].op == OP_RSQ).ok_or("no rsq")?;
    let (r, c) = src(rsq, 0).temp_scalar().ok_or("rsq of what")?;
    let dp3 = writer(code, &ins, rsq, r, c).filter(|&k| ins[k].op == OP_DP3).ok_or("no dp3")?;
    let clip = src(dp3, 0).plain(TYPE_TEMP).filter(|&n| src(dp3, 1).plain(TYPE_TEMP) == Some(n)).ok_or("dp3 not of a temp with itself")?;
    // Its rows: `dp4 clip.x/y/z, ModelViewProj[0/1/2], position`, the same position each.
    let mut rows = [(0usize, 0u32, 0usize); 3];
    for (comp, row) in rows.iter_mut().enumerate() {
        let k = writer(code, &ins, dp3, clip, comp as u32).filter(|&k| ins[k].op == OP_DP4 && ins[k].len == 3).ok_or("the clip position isn't dp4s")?;
        let which = (0..2).find(|&n| src(k, n).plain(TYPE_CONST).is_some()).ok_or("a dp4 row isn't a constant")?;
        *row = (k, src(k, which).num(), which);
    }
    let m = rows[0].1;
    let pos = |row: (usize, u32, usize)| src(row.0, 1 - row.2);
    if rows[1].1 != m + 1 || rows[2].1 != m + 2 || pos(rows[1]) != pos(rows[0]) || pos(rows[2]) != pos(rows[0]) {
        return Err("the clip position's rows aren't one matrix's");
    }
    // (The matrix's fourth row is its depth, as the shader's own `oPos.w` takes it.)
    let w_row = (0..ins.len()).any(|k| ins[k].op == OP_DP4 && ins[k].len == 3 && (0..2).any(|n| src(k, n).plain(TYPE_CONST) == Some(m + 3) && src(k, 1 - n) == pos(rows[0])));
    if !w_row {
        return Err("no depth row");
    }
    if params(code, &ins).any(|p| p.kind() == TYPE_TEMP && p.num() == TEMP) {
        return Err("its spare temp is taken");
    }
    let defined = ins.iter().any(|i| i.op == OP_DEF && Param(code[i.at + 1]).num() == REGISTER);
    if defined || table.iter().any(|&(_, at, n)| (at..at + n).contains(&REGISTER)) {
        return Err("its constant register is taken");
    }
    let major = (code[0] >> 8) & 0xFF;
    let op = |op: u32, len: u32| if major >= 2 { op | (len << 24) } else { op };
    let temp = |bits| Param::new(TYPE_TEMP, TEMP, bits).0;
    let terms = |bits| Param::new(TYPE_CONST, REGISTER, bits).0;
    let (z, xy, zzzz, zwzw) = (0b0100, 0b0011, 0b10_10_10_10, 0b11_10_11_10);
    let after_z = ins[rows[2].0].at + 1 + ins[rows[2].0].len;
    let (at, len) = (ins[dp3].at, ins[dp3].len);
    let mut out = Vec::with_capacity(end + 16);
    out.extend_from_slice(&code[..after_z]);
    // The depth beside the clip position: `dp4 r11.z, ModelViewProj[3], position`.
    let row = &code[ins[rows[2].0].at..after_z];
    let mut depth = [op(OP_DP4, 3), temp(z), row[2], row[3]];
    depth[2 + rows[2].2] = (row[2 + rows[2].2] & !0x7FF) | (m + 3);
    out.extend_from_slice(&depth);
    out.extend_from_slice(&code[after_z..at]);
    // The clip position's x and y back to view space, and the distance squared:
    // `mul r11.xy, clip, terms; mad r11.xy, r11.z, terms.zw, r11; dp3 d, r11, r11`.
    out.extend_from_slice(&[op(OP_MUL, 3), temp(xy), Param::new(TYPE_TEMP, clip, XYZW).0, terms(XYZW)]);
    out.extend_from_slice(&[op(OP_MAD, 4), temp(xy), temp(zzzz), terms(zwzw), temp(XYZW)]);
    out.extend_from_slice(&[op(OP_DP3, 3), code[at + 1], temp(XYZW), temp(XYZW)]);
    out.extend_from_slice(&code[at + 1 + len..end]);
    Ok(Some(out))
}

/// The frustum's terms for [`REGISTER`] (`frustum`: an `NiFrustum`'s left, right, top and bottom
/// tangents): the clip position's x and y back to view space by its depth.
pub fn terms(frustum: &[f32; 6]) -> [f32; 4] {
    let [l, r, t, b, ..] = *frustum;
    [(r - l) / 2.0, (t - b) / 2.0, (r + l) / 2.0, (t + b) / 2.0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3, Vec4};

    /// The game's eyes in a playtest (its log's frustums: the headset's and 11 degrees of overscan).
    const LEFT: [f32; 6] = [-2.1376505, 1.2318016, 1.4244225, -2.2386353, 10.0, 283840.0];

    /// A Direct3D off-centre projection from a frustum, as Gamebryo's renderer makes it (row-major
    /// rows as the shaders' `ModelViewProj` rows take them, for a view-space point).
    fn projection(f: &[f32; 6]) -> Mat4 {
        let [l, r, t, b, n, far] = *f;
        Mat4::from_cols(
            Vec4::new(2.0 / (r - l), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / (t - b), 0.0, 0.0),
            Vec4::new(-(r + l) / (r - l), -(t + b) / (t - b), far / (far - n), 1.0),
            Vec4::new(0.0, 0.0, -n * far / (far - n), 0.0),
        )
    }

    #[test]
    fn the_terms_give_back_the_distance_from_the_clip_position() {
        for f in [LEFT, [-1.2318016, 2.1376505, 1.4244225, -2.2386353, 10.0, 283840.0], [-0.767, 0.767, 0.575, -0.575, 5.0, 1000.0]] {
            let k = terms(&f);
            for v in [Vec3::new(0.0, 0.0, 9000.0), Vec3::new(-14000.0, 3000.0, 9000.0), Vec3::new(8000.0, -12000.0, 6000.0)] {
                let c = projection(&f) * v.extend(1.0);
                let x = c.x * k[0] + c.w * k[2];
                let y = c.y * k[1] + c.w * k[3];
                let d = Vec3::new(x, y, c.w).length();
                assert!((d - v.length()).abs() < v.length() * 1e-4, "{d} for {}", v.length());
            }
        }
    }

    #[test]
    fn a_shader_without_fog_is_left_alone() {
        // `vs_2_0`: `mov oPos, v0`.
        let code = [0xFFFE_0200, 0x0200_0001, Param::new(4, 0, 0xF).0, Param::new(1, 0, XYZW).0, END];
        assert_eq!(rewrite(&code), Ok(None));
    }

    /// Oblivion's own shaders (its package for the player's path, `RendererInfo.txt`'s package 4),
    /// if the game's here: every one with fog rewritten, each the same but for the four
    /// instructions, and Direct3D 9 taking them all.
    #[test]
    fn every_fogged_shader_of_the_games_is_rewritten() {
        let Some(game) = oblivion_assets::install::game_folder() else {
            eprintln!("no Oblivion here: skipped");
            return;
        };
        let path = game.join(r"Data\Shaders\shaderpackage004.sdp");
        let Ok(b) = std::fs::read(&path) else {
            eprintln!("no {}: skipped", path.display());
            return;
        };
        let count = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        let mut off = 12;
        let (mut fogged, mut vertex) = (Vec::new(), 0);
        for _ in 0..count {
            let name: String = b[off..off + 256].iter().take_while(|&&c| c != 0).map(|&c| c as char).collect();
            let size = u32::from_le_bytes(b[off + 256..off + 260].try_into().unwrap()) as usize;
            let code: Vec<u32> = b[off + 260..off + 260 + size].as_chunks::<4>().0.iter().map(|c| u32::from_le_bytes(*c)).collect();
            off += 260 + size;
            if !name.ends_with(".vso") {
                continue;
            }
            vertex += 1;
            assert_eq!(unsafe { length(code.as_ptr()) }, Some(code.len()), "{name}'s length");
            match rewrite(&code) {
                Ok(None) => {}
                Ok(Some(new)) => fogged.push((name, code, new)),
                Err(e) => panic!("{name}: {e}"),
            }
        }
        assert_eq!((vertex, fogged.len()), (329, 112), "the package's vertex shaders, and those with fog");
        let disassemble = disassembler();
        let device = d3d9_device();
        for (name, old, new) in &fogged {
            let (a, b) = (disassemble(old), disassemble(new));
            let (a, b): (Vec<&str>, Vec<&str>) = (a.lines().filter(|l| !l.starts_with("//")).collect(), b.lines().filter(|l| !l.starts_with("//")).collect());
            // The same but for the depth's row added and the dp3 made three.
            assert_eq!(b.len(), a.len() + 3, "{name}");
            let added: Vec<&&str> = b.iter().filter(|l| l.contains("r11")).collect();
            assert_eq!(added.len(), 4, "{name}: {added:?}");
            assert!(added[0].trim().starts_with("dp4 r11.z, c"), "{name}: {added:?}");
            assert!(added[1].trim().starts_with("mul r11.xy, r") && added[1].trim().ends_with(", c255"), "{name}: {added:?}");
            assert_eq!(added[2].trim(), "mad r11.xy, r11.z, c255.zwzw, r11", "{name}");
            assert!(added[3].trim().starts_with("dp3 r") && added[3].trim().ends_with(".x, r11, r11"), "{name}: {added:?}");
            // The rest as it was, in order, but for the fog's dp3.
            let kept: Vec<&str> = b.iter().filter(|l| !l.contains("r11")).copied().collect();
            let gone = (0..a.len()).find(|&i| kept.get(i) != Some(&a[i])).expect("a line goes");
            assert!(a[gone].trim().starts_with("dp3 "), "{name}: {} goes", a[gone]);
            assert_eq!(kept, [&a[..gone], &a[gone + 1..]].concat(), "{name}: only the fog's dp3 goes");
            if let Some(dev) = &device {
                unsafe { dev.CreateVertexShader(new.as_ptr()) }.unwrap_or_else(|e| panic!("{name}: Direct3D 9 refuses it: {e}"));
            }
        }
        assert!(device.is_some(), "no Direct3D 9 device to check the shaders on");
    }

    /// `D3DDisassemble` (from `d3dcompiler_47.dll`).
    fn disassembler() -> impl Fn(&[u32]) -> String {
        use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
        use windows::core::{s, w};
        type Disassemble = unsafe extern "system" fn(*const std::ffi::c_void, usize, u32, *const u8, *mut *mut std::ffi::c_void) -> windows::core::HRESULT;
        let module = unsafe { LoadLibraryW(w!("d3dcompiler_47.dll")) }.expect("d3dcompiler_47.dll");
        let f: Disassemble = unsafe { std::mem::transmute(GetProcAddress(module, s!("D3DDisassemble")).expect("D3DDisassemble")) };
        move |code: &[u32]| {
            let mut blob = std::ptr::null_mut();
            unsafe { f(code.as_ptr().cast(), code.len() * 4, 0, std::ptr::null(), &mut blob) }.ok().expect("disassembles");
            let blob = unsafe { <windows::Win32::Graphics::Direct3D::ID3DBlob as windows::core::Interface>::from_raw(blob) };
            let text = unsafe { std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize()) };
            String::from_utf8_lossy(text).trim_end_matches('\0').to_owned()
        }
    }

    /// A Direct3D 9 device (Ex, windowed on the desktop window), to make the shaders on.
    fn d3d9_device() -> Option<windows::Win32::Graphics::Direct3D9::IDirect3DDevice9Ex> {
        use windows::Win32::Graphics::Direct3D9::*;
        use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
        let d3d = unsafe { Direct3DCreate9Ex(D3D_SDK_VERSION) }.ok()?;
        let wnd = unsafe { GetDesktopWindow() };
        let mut p = D3DPRESENT_PARAMETERS { BackBufferWidth: 8, BackBufferHeight: 8, BackBufferCount: 1, SwapEffect: D3DSWAPEFFECT_DISCARD, hDeviceWindow: wnd, Windowed: true.into(), ..Default::default() };
        let mut dev = None;
        unsafe { d3d.CreateDeviceEx(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, wnd, D3DCREATE_HARDWARE_VERTEXPROCESSING as u32, &mut p, std::ptr::null_mut(), &mut dev) }.ok()?;
        dev
    }
}
