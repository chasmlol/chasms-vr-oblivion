//! The distant land's trees in a headset. Past the loaded cells the game draws its trees as cards,
//! many to a draw (`DISTLOD2003.vso` in its shader packages, instanced by `InstanceData`), each
//! turned square to the horizontal part of `EyeDir`: the camera's look direction, the same for
//! every tree. A flat screen's camera turns little and seldom; a head never stops, and every turn
//! of it turned every far tree, which reads as the whole far forest shimmering
//! (`docs/design/visuals.md`). The game's own near billboards face the eye's position instead.
//!
//! So when the game makes that shader ([`crate::d3d9`], `CreateVertexShader`) it's given this one:
//! the game's own worked the same way (its registers, its lighting, fog and fade), but each card
//! turned to the direction from the eye to its tree. A tree straight ahead stands as the
//! game stood it, and none of them turns as the head does.

/// The game's distant-tree vertex shader, with each card facing the eye (`EyeDir` only for a tree
/// the eye stands right over).
pub const SOURCE: &str = r#"
float4 ModelViewProj[4] : register(c0);
float4 DiffuseDir : register(c4);
float4 DiffuseColor : register(c5);
float4 EyePos : register(c9);
float4 FogColor : register(c10);
float4 FogParam : register(c11);
float4 AlphaParam : register(c12);
float4 AmbientColor : register(c13);
float4 EyeDir : register(c14);
float4 InstanceData[236] : register(c20);

struct VIn { float4 pos : POSITION; float4 color : COLOR0; float2 uv : TEXCOORD0; float4 inst : TEXCOORD1; };
struct VOut { float4 pos : POSITION; float4 fog : COLOR0; float2 uv : TEXCOORD0; float4 light : TEXCOORD4; float4 fade : TEXCOORD5; };

VOut main(VIn v) {
    VOut o;
    // The tree: its position (xyz, the fractions its normal) and its size (w, the fraction its
    // shade), by the whole part of the vertex's instance number.
    float4 inst = InstanceData[(int)(v.inst.x - frac(v.inst.x))];
    float2 to = inst.xy - EyePos.xy;
    float len2 = dot(to, to);
    float2 d = len2 > 1e-4 ? to * rsqrt(len2) : normalize(EyeDir.xy);
    float3 local = float3(d.y * v.pos.x - d.x * v.pos.y, -d.x * v.pos.x - d.y * v.pos.y, v.pos.z);
    float4 p = float4(local * (0.01 * inst.w) + inst.xyz, v.pos.w);
    float4 f = frac(inst);
    float3 n = (f.xyz - 0.5) * 2;
    o.light = float4(dot(DiffuseDir.xyz, n) * v.color.rgb * f.w * DiffuseColor.rgb + AmbientColor.rgb, 1);
    float4 c = float4(dot(ModelViewProj[0], p), dot(ModelViewProj[1], p), dot(ModelViewProj[2], p), dot(ModelViewProj[3], p));
    o.pos = c;
    o.fog = float4(FogColor.rgb, 1 - saturate((FogParam.x - length(c.xyz)) / FogParam.y));
    float e = length(EyePos.xyz - inst.xyz);
    float fade = saturate(1 - (e - AlphaParam.x) / AlphaParam.y);
    o.fade = float4(0, 0, 0, AlphaParam.x < e ? fade : 1);
    o.uv = v.uv;
    return o;
}
"#;

/// The constants only the game's distant-tree vertex shader has all of.
const NAMES: [&[u8]; 4] = [b"EyeDir\0", b"InstanceData\0", b"AlphaParam\0", b"DiffuseDir\0"];

/// Whether a Direct3D 9 vertex shader's code (its tokens: the version, then its constant table as a
/// comment) is the game's distant-tree shader.
pub fn is_distant_trees(tokens: &[u32]) -> bool {
    // vs_2_0, then a comment (0xFFFE, its length in tokens above) holding the constant table.
    let [version, comment, ..] = tokens else { return false };
    if *version != 0xFFFE_0200 || comment & 0xFFFF != 0xFFFE {
        return false;
    }
    let len = (comment >> 16) as usize;
    let Some(table) = tokens.get(2..2 + len) else { return false };
    let bytes: Vec<u8> = table.iter().flat_map(|t| t.to_le_bytes()).collect();
    NAMES.iter().all(|n| bytes.windows(n.len()).any(|w| w == *n))
}

/// The replacement, compiled once (None if it didn't compile: the game's own is kept).
pub fn replacement() -> Option<&'static [u32]> {
    static CODE: std::sync::OnceLock<Option<Vec<u32>>> = std::sync::OnceLock::new();
    CODE.get_or_init(|| match rustvr_gpu::shader::compile(SOURCE, "oblivion distant trees", "main", "vs_2_0") {
        Ok(bytes) => Some(bytes.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()),
        Err(e) => {
            tracing::error!("distant trees: {e:#}");
            None
        }
    })
    .as_deref()
}

/// Testing, `RUSTVR_OBLIVION_GAME_DISTANT_TREES=1`: the game's own shader, the cards turning with
/// the head.
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("RUSTVR_OBLIVION_GAME_DISTANT_TREES").is_ok_and(|v| v == "1"))
}
