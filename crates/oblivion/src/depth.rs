//! The game's depth, for the runtime: Oblivion's depth buffer (a standard Direct3D projection's
//! depth, 24 bits in the low bits of each 32-bit word, between the camera's near and far planes;
//! it reaches this pass as the bytes of a `B8G8R8A8` texture, the lowest in blue)
//! as the linear depth along the view, in metres, which the runtime hides what it draws behind
//! (`rustvr_avatar::DepthInput`): the body, what the hands hold. 0 where nothing was drawn (the
//! sky).

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use rustvr_gpu::Gpu;
    use rustvr_gpu::pipeline::{ConstantBuffer, viewport};
    use rustvr_gpu::shader::Program;
    use std::sync::Mutex;
    use windows::Win32::Graphics::Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
    use windows::Win32::Graphics::Direct3D11::ID3D11ShaderResourceView;

    const SOURCE: &str = r#"
cbuffer C : register(b0) { float near_; float far_; float units_per_metre; float pad_; };
Texture2D<float4> depth : register(t0);
float4 VSMain(uint id : SV_VertexID) : SV_Position {
    float2 uv = float2((id << 1) & 2, id & 2);
    return float4(uv * float2(2, -2) + float2(-1, 1), 0, 1);
}
float PSMain(float4 pos : SV_Position) : SV_Target {
    uint3 b = (uint3)round(depth.Load(int3(pos.xy, 0)).bgr * 255.0);
    float d = (b.x | (b.y << 8) | (b.z << 16)) / 16777215.0;
    if (d >= 0.9999999) return 0;
    return near_ * far_ / (far_ - d * (far_ - near_)) / units_per_metre;
}
"#;

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct Cb {
        near: f32,
        far: f32,
        units_per_metre: f32,
        _pad: f32,
    }

    struct Pass {
        program: Program,
        cb: ConstantBuffer<Cb>,
    }
    unsafe impl Send for Pass {}
    static PASS: Mutex<Option<Pass>> = Mutex::new(None);

    /// Writes the linear depth (metres) of `src` (the game's depth words as `B8G8R8A8` bytes) into `dst`
    /// (`R32_FLOAT`, the runtime's), for a projection between `near` and `far` (game units).
    pub fn convert(gpu: &Gpu, src: &ID3D11ShaderResourceView, dst: &rustvr_gpu::texture::Texture, near: f32, far: f32, units_per_metre: f32) -> bool {
        let Some(rtv) = dst.rtv.as_ref() else { return false };
        let mut pass = PASS.lock().unwrap();
        if pass.is_none() {
            match Program::new(gpu, SOURCE, "oblivion depth", &[]).and_then(|program| Ok(Pass { program, cb: ConstantBuffer::new(gpu)? })) {
                Ok(p) => *pass = Some(p),
                Err(e) => {
                    tracing::error!("depth: the pass: {e:#}");
                    return false;
                }
            }
        }
        let Some(p) = pass.as_ref() else { return false };
        let ctx = &gpu.ctx;
        p.cb.update(ctx, &Cb { near, far, units_per_metre, _pad: 0.0 });
        unsafe {
            ctx.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            ctx.OMSetBlendState(None, None, 0xFFFF_FFFF);
            ctx.OMSetDepthStencilState(None, 0);
            ctx.RSSetState(None);
            viewport(ctx, dst.width, dst.height);
            p.program.bind(ctx);
            ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            ctx.PSSetConstantBuffers(0, Some(&[Some(p.cb.buf.clone())]));
            ctx.PSSetShaderResources(0, Some(&[Some(src.clone())]));
            ctx.Draw(3, 0);
            ctx.PSSetShaderResources(0, Some(&[None]));
            ctx.OMSetRenderTargets(None, None);
        }
        true
    }
}
