//! Oblivion's picture, out of Direct3D 9.
//!
//! Oblivion makes its device through `Direct3DCreate9` (loaded by name from `d3d9.dll`). The hook
//! answers with a D3D9On12 device on a Direct3D 12 device and queue of our own: the same Direct3D 9
//! to the game (managed textures and all), but every surface is a Direct3D 12 resource underneath.
//!
//! An eye ([`capture`]): the back buffer is copied into a render target of ours (Direct3D 9), that
//! target's Direct3D 12 resource is copied on our queue into a texture shared with the adapter's
//! Direct3D 11 device, and a shared fence says when; [`deliver`] waits for the fence on the Direct3D
//! 11 side and copies into the runtime's texture.
//!
//! An eye's depth ([`capture_depth`]): the game's depth buffer after that eye (multisampled, as the
//! game's antialiasing has it, resolved to each pixel's nearest sample first): its depth plane
//! (24 bits in each 32) is copied into a buffer, laid out as Direct3D 12 says it copies, and from
//! there into a shared texture as its bytes, `B8G8R8A8` (Direct3D 11 opens neither a shared depth
//! format nor a 32-bit integer one); [`deliver_depth`] turns it into the runtime's linear depth
//! ([`crate::depth`]).

#[cfg(all(windows, target_arch = "x86"))]
pub use win::*;

#[cfg(all(windows, target_arch = "x86"))]
mod win {
    use crate::hooks;
    use anyhow::Context;
    use rustvr_gpu::Gpu;
    use std::cell::Cell;
    use std::ffi::c_void;
    use std::mem::ManuallyDrop;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use windows::Win32::Foundation::{GENERIC_ALL, HANDLE, HWND, TRUE};
    use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
    use windows::Win32::Graphics::Direct3D9::*;
    use windows::Win32::Graphics::Direct3D9on12::{D3D9ON12_ARGS, Direct3DCreate9On12, IDirect3DDevice9On12};
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC_0, D3D11_TEX2D_SRV, ID3D11Device1, ID3D11Device5, ID3D11DeviceContext4, ID3D11Fence,
        ID3D11ShaderResourceView, ID3D11Texture2D,
    };
    use windows::Win32::Graphics::Direct3D12::*;
    use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
    use windows::core::{IUnknown, Interface, PCWSTR, s};

    /// Copies: the left eye, the right eye, the HUD (or the whole picture outside the world), the
    /// mirror's world (`crate::render`).
    pub const KINDS: usize = 4;
    pub const HUD: usize = 2;
    pub const MIRROR: usize = 3;
    /// Depths: the left eye's, the right eye's, the mirror's world's.
    const DEPTHS: usize = 3;

    static REAL_CREATE9: AtomicUsize = AtomicUsize::new(0);
    static REAL_CREATE_DEVICE: AtomicUsize = AtomicUsize::new(0);
    static REAL_PRESENT: AtomicUsize = AtomicUsize::new(0);
    static REAL_TCL: AtomicUsize = AtomicUsize::new(0);
    static LAST_TCL: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
    /// The device's answers when lost: not to be reset yet, and to be reset.
    const DEVICE_LOST: i32 = 0x8876_0868_u32 as i32;
    const DEVICE_NOT_RESET: i32 = 0x8876_0869_u32 as i32;
    /// The game is in its render (`Main::Render`).
    static RENDERING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    /// Says whether the game is in its render (where a lost device isn't to be reset).
    pub fn rendering(now: bool) {
        RENDERING.store(now, Ordering::Release);
    }

    thread_local! {
        static MAKING: Cell<bool> = const { Cell::new(false) };
    }

    struct D3d12 {
        device: ID3D12Device,
        queue: ID3D12CommandQueue,
    }
    unsafe impl Send for D3d12 {}
    static D3D12: Mutex<Option<D3d12>> = Mutex::new(None);

    /// Our copies of one size: a Direct3D 9 render target per kind, its twin shared between Direct3D
    /// 12 and 11, and the fence that orders them.
    struct Bridge {
        size: (u32, u32),
        tex9: Vec<IDirect3DTexture9>,
        shared12: Vec<ID3D12Resource>,
        shared11: Vec<ID3D11Texture2D>,
        allocators: Vec<ID3D12CommandAllocator>,
        lists: Vec<ID3D12GraphicsCommandList>,
        /// The fence value each kind's allocator was last used up to.
        used: Vec<u64>,
        fence12: ID3D12Fence,
        fence11: ID3D11Fence,
        value: u64,
        /// The fence value each kind's latest copy is done at.
        ready: [u64; KINDS],
        event: HANDLE,
        /// The eyes' depth, once the game's depth buffer has been seen.
        depth: Option<Depth>,
    }

    /// Each eye's depth: the game's depth buffer as it was after that eye, in a twin shared between
    /// Direct3D 12 and 11, with its own command lists (on the bridge's fence).
    struct Depth {
        size: (u32, u32),
        /// How the depth plane copies into a buffer, and the buffers.
        footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
        buffers: Vec<ID3D12Resource>,
        /// A multisampled depth buffer is resolved into these first (each pixel's nearest sample).
        resolved: Vec<ID3D12Resource>,
        shared12: Vec<ID3D12Resource>,
        srv: Vec<ID3D11ShaderResourceView>,
        allocators: Vec<ID3D12CommandAllocator>,
        lists: Vec<ID3D12GraphicsCommandList>,
        used: [u64; DEPTHS],
        ready: [u64; DEPTHS],
    }
    unsafe impl Send for Bridge {}
    static BRIDGE: Mutex<Option<Bridge>> = Mutex::new(None);

    /// Hooks `d3d9.dll`'s `Direct3DCreate9` (before the game calls it).
    pub fn install() {
        let mut path: Vec<u16> = system_dir();
        path.extend("\\d3d9.dll\0".encode_utf16());
        let Ok(module) = (unsafe { LoadLibraryW(PCWSTR(path.as_ptr())) }) else {
            tracing::error!("d3d9: d3d9.dll didn't load");
            return;
        };
        let Some(f) = (unsafe { GetProcAddress(module, s!("Direct3DCreate9")) }) else {
            tracing::error!("d3d9: no Direct3DCreate9");
            return;
        };
        hooks::install_into(f as *const (), create9 as *const (), "Direct3DCreate9", &REAL_CREATE9);
    }

    fn system_dir() -> Vec<u16> {
        let mut buf = [0u16; 260];
        let n = unsafe { windows::Win32::System::SystemInformation::GetSystemDirectoryW(Some(&mut buf)) } as usize;
        buf[..n].to_vec()
    }

    type Create9Fn = unsafe extern "system" fn(u32) -> *mut c_void;
    type CreateDeviceFn = unsafe extern "system" fn(*mut c_void, u32, D3DDEVTYPE, HWND, u32, *mut D3DPRESENT_PARAMETERS, *mut *mut c_void) -> windows::core::HRESULT;
    type PresentFn = unsafe extern "system" fn(*mut c_void, *const c_void, *const c_void, HWND, *const c_void) -> windows::core::HRESULT;

    unsafe extern "system" fn create9(sdk: u32) -> *mut c_void {
        let real: Create9Fn = unsafe { std::mem::transmute(REAL_CREATE9.load(Ordering::Acquire)) };
        // D3D9On12 may make its own Direct3D 9 the ordinary way: that one is the real one.
        if MAKING.with(|m| m.get()) || !crate::settings::get().d3d9on12 {
            return unsafe { real(sdk) };
        }
        MAKING.with(|m| m.set(true));
        let made = hooks::guard("Direct3DCreate9", None, || make_on12(sdk));
        MAKING.with(|m| m.set(false));
        match made {
            Some(d3d) => d3d,
            None => {
                tracing::error!("d3d9: D3D9On12 failed: the game gets plain Direct3D 9 and VR is off");
                unsafe { real(sdk) }
            }
        }
    }

    fn make_on12(sdk: u32) -> Option<*mut c_void> {
        let mut device: Option<ID3D12Device> = None;
        if let Err(e) = unsafe { D3D12CreateDevice(None, D3D_FEATURE_LEVEL_11_0, &mut device) } {
            tracing::error!("d3d9: Direct3D 12 device: {e}");
            return None;
        }
        let device = device?;
        let queue: ID3D12CommandQueue = match unsafe { device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC { Type: D3D12_COMMAND_LIST_TYPE_DIRECT, ..Default::default() }) } {
            Ok(q) => q,
            Err(e) => {
                tracing::error!("d3d9: Direct3D 12 queue: {e}");
                return None;
            }
        };
        let mut args = D3D9ON12_ARGS {
            Enable9On12: TRUE,
            pD3D12Device: ManuallyDrop::new(Some(device.cast::<IUnknown>().ok()?)),
            ppD3D12Queues: [ManuallyDrop::new(Some(queue.cast::<IUnknown>().ok()?)), ManuallyDrop::new(None)],
            NumQueues: 1,
            NodeMask: 0,
        };
        let d3d = unsafe { Direct3DCreate9On12(sdk, &mut args, 1) };
        unsafe {
            ManuallyDrop::drop(&mut args.pD3D12Device);
            ManuallyDrop::drop(&mut args.ppD3D12Queues[0]);
        }
        let d3d = d3d?;
        tracing::info!("d3d9: the game's Direct3D 9 is on D3D9On12");
        *D3D12.lock().unwrap() = Some(D3d12 { device, queue });
        // IDirect3D9::CreateDevice (vtable slot 16).
        let raw = d3d.into_raw();
        let create_device = unsafe { *(*(raw as *const *const usize)).add(16) };
        hooks::install_into(create_device as *const (), create_device_hook as *const (), "IDirect3D9::CreateDevice", &REAL_CREATE_DEVICE);
        Some(raw)
    }

    unsafe extern "system" fn create_device_hook(this: *mut c_void, adapter: u32, kind: D3DDEVTYPE, focus: HWND, flags: u32, params: *mut D3DPRESENT_PARAMETERS, out: *mut *mut c_void) -> windows::core::HRESULT {
        let real: CreateDeviceFn = unsafe { std::mem::transmute(REAL_CREATE_DEVICE.load(Ordering::Acquire)) };
        if !params.is_null() {
            let p = unsafe { &mut *params };
            tracing::info!("d3d9: the game asks for {}x{} format {:?}, windowed {}, interval {:#x}", p.BackBufferWidth, p.BackBufferHeight, p.BackBufferFormat, p.Windowed.0, p.PresentationInterval);
            // Windowed, and never held to the monitor's refresh: the headset paces the game.
            p.Windowed = TRUE;
            p.FullScreen_RefreshRateInHz = 0;
            p.PresentationInterval = D3DPRESENT_INTERVAL_IMMEDIATE as u32;
        }
        let hr = unsafe { real(this, adapter, kind, focus, flags, params, out) };
        hooks::guard_void("CreateDevice", || {
            if hr.is_err() || out.is_null() || unsafe { (*out).is_null() } {
                tracing::error!("d3d9: CreateDevice failed: {hr:?}");
                return;
            }
            let dev = unsafe { *out };
            tracing::info!("d3d9: the game's device {dev:p}");
            // The game is running (Steam's DRM is done with its import table).
            crate::window::install();
            // IDirect3DDevice9::Present (vtable slot 17).
            let present = unsafe { *(*(dev as *const *const usize)).add(17) };
            hooks::install_into(present as *const (), present_hook as *const (), "IDirect3DDevice9::Present", &REAL_PRESENT);
            // IDirect3DDevice9::TestCooperativeLevel (slot 3): the game skips its frames while it fails.
            let tcl = unsafe { *(*(dev as *const *const usize)).add(3) };
            hooks::install_into(tcl as *const (), test_cooperative_level as *const (), "IDirect3DDevice9::TestCooperativeLevel", &REAL_TCL);
            match Gpu::new() {
                Ok(gpu) => {
                    let (w, h) = if params.is_null() { (0, 0) } else { unsafe { ((*params).BackBufferWidth, (*params).BackBufferHeight) } };
                    tracing::info!("d3d9: our Direct3D 11 device is on {}", gpu.adapter_name());
                    crate::link::init(gpu, w, h);
                }
                Err(e) => tracing::error!("d3d9: our Direct3D 11 device: {e:#}"),
            }
        });
        hr
    }

    unsafe extern "system" fn present_hook(dev: *mut c_void, src: *const c_void, dst: *const c_void, wnd: HWND, dirty: *const c_void) -> windows::core::HRESULT {
        let real: PresentFn = unsafe { std::mem::transmute(REAL_PRESENT.load(Ordering::Acquire)) };
        hooks::guard_void("Present", || crate::render::on_present(dev));
        unsafe { real(dev, src, dst, wnd, dirty) }
    }

    unsafe extern "system" fn test_cooperative_level(dev: *mut c_void) -> windows::core::HRESULT {
        let real: unsafe extern "system" fn(*mut c_void) -> windows::core::HRESULT = unsafe { std::mem::transmute(REAL_TCL.load(Ordering::Acquire)) };
        let mut hr = unsafe { real(dev) };
        // The game's renderer resets its device as soon as it hears it wants resetting, wherever it
        // asks: in the middle of its render too, as it starts drawing the water's reflection, and
        // the reset's callback lets go of the reflection's camera under the drawing (a crash). There
        // the device is only lost (its renderer draws nothing more until it's back): the game's own
        // loop resets it before its next frame, as it does whenever it finds it lost.
        if hr.0 == DEVICE_NOT_RESET && RENDERING.load(Ordering::Acquire) {
            hr = windows::core::HRESULT(DEVICE_LOST);
        }
        if LAST_TCL.swap(hr.0, Ordering::AcqRel) != hr.0 {
            tracing::info!("d3d9: TestCooperativeLevel {:#x}", hr.0);
        }
        // Lost (the desktop's display changed: a headset's streamer connecting), the device is reset
        // by the game, which it can only do once nothing else has a texture of its own on it: our
        // copies go, and are made again on its next frame.
        if hr.is_err() {
            hooks::guard_void("TestCooperativeLevel", release_bridge);
        }
        hr
    }

    /// Lets go of our copies of the game's picture (once the copies in flight are done).
    fn release_bridge() {
        let Some(b) = BRIDGE.lock().unwrap().take() else { return };
        if unsafe { b.fence12.GetCompletedValue() } < b.value && unsafe { b.fence12.SetEventOnCompletion(b.value, b.event) }.is_ok() {
            unsafe { WaitForSingleObject(b.event, 1000) };
        }
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(b.event);
        }
        tracing::info!("d3d9: the device is lost: our copies of its picture let go, for the game to reset it");
    }

    /// The game's device, borrowed.
    pub fn device(raw: *mut c_void) -> Option<IDirect3DDevice9> {
        unsafe { IDirect3DDevice9::from_raw_borrowed(&raw) }.cloned()
    }

    /// The back buffer's size.
    pub fn backbuffer_size(dev: &IDirect3DDevice9) -> Option<(u32, u32)> {
        let bb = unsafe { dev.GetBackBuffer(0, 0, D3DBACKBUFFER_TYPE_MONO) }.ok()?;
        let mut desc = D3DSURFACE_DESC::default();
        unsafe { bb.GetDesc(&mut desc) }.ok()?;
        Some((desc.Width, desc.Height))
    }

    fn make_bridge(dev: &IDirect3DDevice9, d12: &D3d12, gpu: &Gpu, w: u32, h: u32) -> anyhow::Result<Bridge> {
        let mut tex9 = Vec::new();
        let mut shared12 = Vec::new();
        let mut shared11 = Vec::new();
        let mut allocators = Vec::new();
        let mut lists = Vec::new();
        let dev11: ID3D11Device1 = gpu.device.cast()?;
        for _ in 0..KINDS {
            let mut t: Option<IDirect3DTexture9> = None;
            unsafe { dev.CreateTexture(w, h, 1, D3DUSAGE_RENDERTARGET as u32, D3DFMT_A8R8G8B8, D3DPOOL_DEFAULT, &mut t, std::ptr::null_mut()) }?;
            tex9.push(t.ok_or_else(|| anyhow::anyhow!("no texture"))?);
            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Width: w as u64,
                Height: h,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
                ..Default::default()
            };
            let heap = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() };
            let mut r: Option<ID3D12Resource> = None;
            unsafe { d12.device.CreateCommittedResource(&heap, D3D12_HEAP_FLAG_SHARED, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut r) }?;
            let r = r.ok_or_else(|| anyhow::anyhow!("no resource"))?;
            let handle = unsafe { d12.device.CreateSharedHandle(&r, None, GENERIC_ALL.0, PCWSTR::null()) }?;
            let t11: ID3D11Texture2D = unsafe { dev11.OpenSharedResource1(handle) }?;
            unsafe { windows::Win32::Foundation::CloseHandle(handle) }?;
            shared12.push(r);
            shared11.push(t11);
            let a: ID3D12CommandAllocator = unsafe { d12.device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }?;
            let l: ID3D12GraphicsCommandList = unsafe { d12.device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &a, None) }?;
            unsafe { l.Close() }?;
            allocators.push(a);
            lists.push(l);
        }
        let fence12: ID3D12Fence = unsafe { d12.device.CreateFence(0, D3D12_FENCE_FLAG_SHARED) }?;
        let handle = unsafe { d12.device.CreateSharedHandle(&fence12, None, GENERIC_ALL.0, PCWSTR::null()) }?;
        let dev5: ID3D11Device5 = gpu.device.cast()?;
        let mut fence11: Option<ID3D11Fence> = None;
        unsafe { dev5.OpenSharedFence(handle, &mut fence11) }?;
        unsafe { windows::Win32::Foundation::CloseHandle(handle) }?;
        let event = unsafe { CreateEventW(None, false, false, PCWSTR::null()) }?;
        tracing::info!("d3d9: bridge ready ({w}x{h})");
        Ok(Bridge {
            size: (w, h),
            tex9,
            shared12,
            shared11,
            allocators,
            lists,
            used: vec![0; KINDS],
            fence12,
            fence11: fence11.ok_or_else(|| anyhow::anyhow!("no fence"))?,
            value: 0,
            ready: [0; KINDS],
            event,
            depth: None,
        })
    }

    /// A texture of `format` shared between our Direct3D 12 device and the adapter's Direct3D 11.
    fn shared_texture(d12: &D3d12, dev11: &ID3D11Device1, w: u32, h: u32, format: DXGI_FORMAT, flags: D3D12_RESOURCE_FLAGS) -> anyhow::Result<(ID3D12Resource, ID3D11Texture2D)> {
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: w as u64,
            Height: h,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Flags: flags,
            ..Default::default()
        };
        let heap = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() };
        let mut r: Option<ID3D12Resource> = None;
        unsafe { d12.device.CreateCommittedResource(&heap, D3D12_HEAP_FLAG_SHARED, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut r) }.context("made")?;
        let r = r.ok_or_else(|| anyhow::anyhow!("no resource"))?;
        let handle = unsafe { d12.device.CreateSharedHandle(&r, None, GENERIC_ALL.0, PCWSTR::null()) }.context("shared")?;
        let t11 = unsafe { dev11.OpenSharedResource1::<ID3D11Texture2D>(handle) }.context("opened");
        unsafe { windows::Win32::Foundation::CloseHandle(handle) }?;
        Ok((r, t11?))
    }

    fn make_depth(d12: &D3d12, gpu: &Gpu, src: &D3D12_RESOURCE_DESC) -> anyhow::Result<Depth> {
        let (w, h) = (src.Width as u32, src.Height);
        let multisampled = src.SampleDesc.Count > 1;
        // (A multisampled depth plane doesn't copy: it's resolved into one of these first.)
        let single = D3D12_RESOURCE_DESC { SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 }, Alignment: 0, ..*src };
        let src = &single;
        // The depth plane (the first subresource), as it copies into a buffer. (The shared texture
        // holds the words as bytes, B8G8R8A8: Direct3D 11 opens neither a 32-bit integer texture nor
        // one that isn't a render target.)
        let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
        let mut bytes = 0u64;
        unsafe { d12.device.GetCopyableFootprints(src, 0, 1, 0, Some(&mut footprint), None, None, Some(&mut bytes)) };
        tracing::info!("d3d9: the depth plane copies as {:?}, {} bytes a row ({bytes} bytes)", footprint.Footprint.Format, footprint.Footprint.RowPitch);
        anyhow::ensure!(footprint.Footprint.Format != DXGI_FORMAT_UNKNOWN && bytes > 0 && bytes != u64::MAX, "the depth plane doesn't copy");
        let dev11: ID3D11Device1 = gpu.device.cast()?;
        let (mut buffers, mut resolved, mut shared12, mut srv, mut allocators, mut lists) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for _ in 0..DEPTHS {
            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Width: bytes,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                ..Default::default()
            };
            let heap = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() };
            let mut buf: Option<ID3D12Resource> = None;
            unsafe { d12.device.CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut buf) }.context("the buffer")?;
            buffers.push(buf.ok_or_else(|| anyhow::anyhow!("no buffer"))?);
            let (r, t11) = shared_texture(d12, &dev11, w, h, DXGI_FORMAT_B8G8R8A8_UNORM, D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET).context("the shared texture")?;
            let desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                ViewDimension: windows::Win32::Graphics::Direct3D::D3D11_SRV_DIMENSION_TEXTURE2D,
                Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 { Texture2D: D3D11_TEX2D_SRV { MostDetailedMip: 0, MipLevels: 1 } },
            };
            let mut v = None;
            unsafe { gpu.device.CreateShaderResourceView(&t11, Some(&desc), Some(&mut v)) }.context("the view")?;
            shared12.push(r);
            srv.push(v.ok_or_else(|| anyhow::anyhow!("no view"))?);
            let a: ID3D12CommandAllocator = unsafe { d12.device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }?;
            let l: ID3D12GraphicsCommandList = unsafe { d12.device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &a, None) }?;
            unsafe { l.Close() }?;
            allocators.push(a);
            lists.push(l);
            if multisampled {
                let heap = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() };
                let mut r: Option<ID3D12Resource> = None;
                unsafe { d12.device.CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, src, D3D12_RESOURCE_STATE_COMMON, None, &mut r) }.context("the resolved depth")?;
                resolved.push(r.ok_or_else(|| anyhow::anyhow!("no resource"))?);
            }
        }
        tracing::info!("d3d9: depth bridge ready ({w}x{h}{})", if multisampled { ", resolved from its samples" } else { "" });
        Ok(Depth { size: (w, h), footprint, buffers, resolved, shared12, srv, allocators, lists, used: [0; DEPTHS], ready: [0; DEPTHS] })
    }

    /// Copies the game's depth buffer as it is now (after `eye`'s render) into that eye's shared
    /// depth. False if there's no bridge or no depth buffer.
    pub fn capture_depth(dev: &IDirect3DDevice9, gpu: &Gpu, eye: usize) -> bool {
        let d12 = D3D12.lock().unwrap();
        let Some(d12) = d12.as_ref() else { return false };
        let mut bridge = BRIDGE.lock().unwrap();
        let Some(b) = bridge.as_mut() else { return false };
        match copy_depth(dev, d12, gpu, b, eye) {
            Ok(()) => true,
            Err(e) => {
                // (No depth buffer bound is no news: the menus, loading.)
                let msg = format!("{e:#}");
                static SAID: Mutex<Vec<String>> = Mutex::new(Vec::new());
                let mut said = SAID.lock().unwrap();
                if !said.contains(&msg) && said.len() < 8 {
                    tracing::warn!("d3d9: depth {eye}: {msg}");
                    said.push(msg);
                }
                false
            }
        }
    }

    fn copy_depth(dev: &IDirect3DDevice9, d12: &D3d12, gpu: &Gpu, b: &mut Bridge, eye: usize) -> anyhow::Result<()> {
        let ds = unsafe { dev.GetDepthStencilSurface() }.context("no depth buffer")?;
        let on12: IDirect3DDevice9On12 = dev.cast()?;
        // (A surface is a resource, but doesn't answer for one when asked: borrowed as one.)
        let raw_ds = ds.as_raw();
        let Some(res9) = (unsafe { IDirect3DResource9::from_raw_borrowed(&raw_ds) }) else { return Ok(()) };
        let mut raw: *mut c_void = std::ptr::null_mut();
        unsafe { on12.UnwrapUnderlyingResource(res9, &d12.queue, &ID3D12Resource::IID, &mut raw) }.context("unwrapped")?;
        let src = unsafe { ID3D12Resource::from_raw(raw) };
        let sd = unsafe { src.GetDesc() };
        if b.depth.as_ref().is_none_or(|d| d.size != (sd.Width as u32, sd.Height)) {
            tracing::info!("d3d9: the game's depth buffer: {}x{} {:?} x{} samples, flags {:?}", sd.Width, sd.Height, sd.Format, sd.SampleDesc.Count, sd.Flags);
            match make_depth(d12, gpu, &sd) {
                Ok(d) => b.depth = Some(d),
                Err(e) => {
                    drop(src);
                    unsafe { on12.ReturnUnderlyingResource(res9, 0, std::ptr::null_mut(), std::ptr::null_mut()) }?;
                    return Err(e.context("the depth bridge"));
                }
            }
        }
        let Some(depth) = b.depth.as_mut() else { return Ok(()) };
        if unsafe { b.fence12.GetCompletedValue() } < depth.used[eye] {
            unsafe { b.fence12.SetEventOnCompletion(depth.used[eye], b.event) }?;
            unsafe { WaitForSingleObject(b.event, INFINITE) };
        }
        let (alloc, list, buf, dst) = (&depth.allocators[eye], &depth.lists[eye], &depth.buffers[eye], &depth.shared12[eye]);
        // The depth plane into the buffer, laid out as it copies; the same bytes into the shared
        // texture, as bytes.
        let into_buffer = D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(buf) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: depth.footprint },
        };
        let mut as_words = depth.footprint;
        as_words.Footprint.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
        let from_buffer = D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(buf) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: as_words },
        };
        let resolved = depth.resolved.get(eye);
        let plane = resolved.unwrap_or(&src);
        let from_depth = D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(plane) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
        };
        let into_shared = D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(dst) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
        };
        unsafe {
            alloc.Reset().context("allocator")?;
            list.Reset(alloc, None).context("list")?;
            if let Some(r) = resolved {
                // Each pixel's nearest sample.
                let list1: ID3D12GraphicsCommandList1 = list.cast()?;
                list.ResourceBarrier(&[
                    transition(&src, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_RESOLVE_SOURCE),
                    transition(r, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_RESOLVE_DEST),
                ]);
                list1.ResolveSubresourceRegion(r, 0, 0, 0, &src, 0, None, sd.Format, D3D12_RESOLVE_MODE_MIN);
                list.ResourceBarrier(&[
                    transition(&src, D3D12_RESOURCE_STATE_RESOLVE_SOURCE, D3D12_RESOURCE_STATE_COMMON),
                    transition(r, D3D12_RESOURCE_STATE_RESOLVE_DEST, D3D12_RESOURCE_STATE_COMMON),
                ]);
            }
            list.ResourceBarrier(&[
                transition(plane, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_COPY_SOURCE),
                transition(buf, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_COPY_DEST),
                transition(dst, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_COPY_DEST),
            ]);
            list.CopyTextureRegion(&into_buffer, 0, 0, 0, &from_depth, None);
            list.ResourceBarrier(&[transition(buf, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE)]);
            list.CopyTextureRegion(&into_shared, 0, 0, 0, &from_buffer, None);
            list.ResourceBarrier(&[
                transition(plane, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COMMON),
                transition(buf, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COMMON),
                transition(dst, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COMMON),
            ]);
            list.Close().context("recorded")?;
            d12.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
        }
        b.value += 1;
        unsafe { d12.queue.Signal(&b.fence12, b.value) }.context("signalled")?;
        depth.used[eye] = b.value;
        depth.ready[eye] = b.value;
        drop(src);
        unsafe { on12.ReturnUnderlyingResource(res9, 0, std::ptr::null_mut(), std::ptr::null_mut()) }.context("returned")?;
        Ok(())
    }

    /// Writes `eye`'s latest depth into `dst` (the runtime's, linear metres), for the projection
    /// between `near` and `far` (game units). False if there's none.
    pub fn deliver_depth(gpu: &Gpu, eye: usize, dst: &rustvr_gpu::texture::Texture, near: f32, far: f32) -> bool {
        let bridge = BRIDGE.lock().unwrap();
        let Some(depth) = bridge.as_ref().and_then(|b| b.depth.as_ref().map(|d| (b, d))) else { return false };
        let (b, depth) = depth;
        if depth.ready[eye] == 0 || (dst.width, dst.height) != depth.size {
            return false;
        }
        let Ok(ctx4) = gpu.ctx.cast::<ID3D11DeviceContext4>() else { return false };
        if unsafe { ctx4.Wait(&b.fence11, depth.ready[eye]) }.is_err() {
            return false;
        }
        crate::depth::convert(gpu, &depth.srv[eye], dst, near, far, crate::camera::UNITS_PER_METER)
    }

    /// Copies the game's back buffer as it is now into `kind`'s shared texture. False if the
    /// bridge isn't there (yet).
    pub fn capture(dev: &IDirect3DDevice9, gpu: &Gpu, kind: usize) -> bool {
        // (Not on a lost device: a bridge made on it would keep it from being reset.)
        if LAST_TCL.load(Ordering::Acquire) < 0 {
            return false;
        }
        let Some((w, h)) = backbuffer_size(dev) else { return false };
        let d12 = D3D12.lock().unwrap();
        let Some(d12) = d12.as_ref() else { return false };
        let mut bridge = BRIDGE.lock().unwrap();
        if bridge.as_ref().is_none_or(|b| b.size != (w, h)) {
            *bridge = match make_bridge(dev, d12, gpu, w, h) {
                Ok(b) => Some(b),
                Err(e) => {
                    tracing::error!("d3d9: bridge {w}x{h}: {e:#}");
                    None
                }
            };
        }
        let Some(b) = bridge.as_mut() else { return false };
        match copy(dev, d12, b, kind) {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("d3d9: copy {kind}: {e:#}");
                false
            }
        }
    }

    fn copy(dev: &IDirect3DDevice9, d12: &D3d12, b: &mut Bridge, kind: usize) -> anyhow::Result<()> {
        let bb = unsafe { dev.GetBackBuffer(0, 0, D3DBACKBUFFER_TYPE_MONO) }?;
        let ours = unsafe { b.tex9[kind].GetSurfaceLevel(0) }?;
        unsafe { dev.StretchRect(&bb, std::ptr::null(), &ours, std::ptr::null(), D3DTEXF_NONE) }?;
        let on12: IDirect3DDevice9On12 = dev.cast()?;
        let mut raw: *mut c_void = std::ptr::null_mut();
        let res9: IDirect3DResource9 = b.tex9[kind].cast()?;
        unsafe { on12.UnwrapUnderlyingResource(&res9, &d12.queue, &ID3D12Resource::IID, &mut raw) }?;
        let src = unsafe { ID3D12Resource::from_raw(raw) };
        // The allocator again only once the GPU is done with its last copy.
        if unsafe { b.fence12.GetCompletedValue() } < b.used[kind] {
            unsafe { b.fence12.SetEventOnCompletion(b.used[kind], b.event) }?;
            unsafe { WaitForSingleObject(b.event, INFINITE) };
        }
        let (alloc, list) = (&b.allocators[kind], &b.lists[kind]);
        unsafe {
            alloc.Reset()?;
            list.Reset(alloc, None)?;
            list.ResourceBarrier(&[transition(&b.shared12[kind], D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_COPY_DEST)]);
            list.CopyResource(&b.shared12[kind], &src);
            list.ResourceBarrier(&[transition(&b.shared12[kind], D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COMMON)]);
            list.Close()?;
            d12.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
        }
        b.value += 1;
        unsafe { d12.queue.Signal(&b.fence12, b.value) }?;
        b.used[kind] = b.value;
        b.ready[kind] = b.value;
        drop(src);
        unsafe { on12.ReturnUnderlyingResource(&res9, 0, std::ptr::null_mut(), std::ptr::null_mut()) }?;
        Ok(())
    }

    fn transition(r: &ID3D12Resource, before: D3D12_RESOURCE_STATES, after: D3D12_RESOURCE_STATES) -> D3D12_RESOURCE_BARRIER {
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: unsafe { std::mem::transmute_copy(r) },
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    StateBefore: before,
                    StateAfter: after,
                }),
            },
        }
    }

    /// Copies `kind`'s latest capture into `dst` on the adapter's device (after the copy into the
    /// shared texture is done). False if there's none or the sizes differ.
    pub fn deliver(gpu: &Gpu, kind: usize, dst: &ID3D11Texture2D) -> bool {
        let bridge = BRIDGE.lock().unwrap();
        let Some(b) = bridge.as_ref() else { return false };
        if b.ready[kind] == 0 {
            return false;
        }
        let Ok(ctx4) = gpu.ctx.cast::<ID3D11DeviceContext4>() else { return false };
        unsafe {
            if ctx4.Wait(&b.fence11, b.ready[kind]).is_err() {
                return false;
            }
            gpu.ctx.CopyResource(dst, &b.shared11[kind]);
        }
        true
    }
}
