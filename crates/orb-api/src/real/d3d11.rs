//! The Direct3D 11 objects called through the vtables `crate::d3d11` declares.
//!
//! **Every slot here is one line**: read the vtable out of the object, call the slot, hand back what it
//! answered. Nothing decides anything — the shaders' source, the vertex layout, the blend mode, the
//! topology and the viewport are above the seam, where an e2e test reaches them.
//!
//! **The one thing this file does beyond calling** is find a shader compiler: `D3DCompile` out of whichever
//! `d3dcompiler` the process already has, asked for by name from the newest down. A game that draws this
//! way has one loaded — it compiles its own shaders with it — and `GetModuleHandleW` never loads a second
//! copy, so a process without one answers nothing rather than pulling a DLL in.

use std::ffi::c_void;

use crate::d3d11::{
    Blend, BlendDesc, BlobVtable, Buffer, BufferDesc, Context, ContextVtable,
    D3D11_MAP_WRITE_DISCARD, DepthStencil, DepthView, Device, DeviceVtable, IID_ID3D11DEVICE,
    IID_ID3D11TEXTURE2D, InputElementDesc, Layout, MappedSubresource, Object, PixelShader,
    Rasterizer, ResourceView, Sampler, SamplerDesc, SubresourceData, SwapChainVtable, Target,
    Texture, Texture2dDesc, UnknownVtable, VertexShader, Viewport,
};

/// The vtable of an object at an address: its first field is the pointer to it, which is what a COM object
/// is.
///
/// # Safety
/// `object` must be a live COM object whose vtable is laid out as `V` says.
unsafe fn vtable<V>(object: usize) -> &'static V {
    unsafe { &*(*(object as *const *const V)) }
}

fn object(address: usize) -> *mut c_void {
    address as *mut c_void
}

/// What a create call answered, as the newtype the caller wanted — `None` where it failed or handed back
/// nothing, which for these calls is the same thing to the caller.
fn made<T>(answered: i32, at: *mut c_void, into: impl FnOnce(usize) -> T) -> Option<T> {
    match answered < 0 || at.is_null() {
        true => None,
        false => Some(into(at as usize)),
    }
}

pub unsafe fn device_of(swapchain: usize) -> Option<Device> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<SwapChainVtable>(swapchain).get_device)(
            object(swapchain),
            IID_ID3D11DEVICE.as_ptr(),
            &mut at,
        )
    };
    made(answered, at, Device)
}

pub unsafe fn immediate_context(device: Device) -> Option<Context> {
    let mut at: *mut c_void = std::ptr::null_mut();
    unsafe {
        (vtable::<DeviceVtable>(device.0).get_immediate_context)(object(device.0), &mut at);
    }
    made(0, at, Context)
}

pub unsafe fn back_buffer(swapchain: usize) -> Option<Texture> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<SwapChainVtable>(swapchain).get_buffer)(
            object(swapchain),
            0,
            IID_ID3D11TEXTURE2D.as_ptr(),
            &mut at,
        )
    };
    made(answered, at, Texture)
}

pub unsafe fn render_target(device: Device, texture: Texture) -> Option<Target> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_render_target_view)(
            object(device.0),
            object(texture.0),
            std::ptr::null(),
            &mut at,
        )
    };
    made(answered, at, Target)
}

pub unsafe fn vertex_shader(device: Device, bytecode: &[u8]) -> Option<VertexShader> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_vertex_shader)(
            object(device.0),
            bytecode.as_ptr(),
            bytecode.len(),
            std::ptr::null_mut(),
            &mut at,
        )
    };
    made(answered, at, VertexShader)
}

pub unsafe fn pixel_shader(device: Device, bytecode: &[u8]) -> Option<PixelShader> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_pixel_shader)(
            object(device.0),
            bytecode.as_ptr(),
            bytecode.len(),
            std::ptr::null_mut(),
            &mut at,
        )
    };
    made(answered, at, PixelShader)
}

pub unsafe fn input_layout(
    device: Device,
    elements: &[InputElementDesc],
    bytecode: &[u8],
) -> Option<Layout> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_input_layout)(
            object(device.0),
            elements.as_ptr(),
            elements.len() as u32,
            bytecode.as_ptr(),
            bytecode.len(),
            &mut at,
        )
    };
    made(answered, at, Layout)
}

pub unsafe fn buffer(device: Device, desc: BufferDesc, bytes: Option<&[u8]>) -> Option<Buffer> {
    let initial = bytes.map(|bytes| SubresourceData {
        sys_mem: bytes.as_ptr().cast(),
        sys_mem_pitch: 0,
        sys_mem_slice_pitch: 0,
    });
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_buffer)(
            object(device.0),
            &desc,
            initial.as_ref().map_or(std::ptr::null(), |it| it),
            &mut at,
        )
    };
    made(answered, at, Buffer)
}

pub unsafe fn texture(device: Device, desc: Texture2dDesc, pixels: &[u32]) -> Option<Texture> {
    let initial = SubresourceData {
        sys_mem: pixels.as_ptr().cast(),
        // The stride of one row in bytes, which is what a texture's upload is described by.
        sys_mem_pitch: desc.width * size_of::<u32>() as u32,
        sys_mem_slice_pitch: 0,
    };
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_texture_2d)(
            object(device.0),
            &desc,
            &initial,
            &mut at,
        )
    };
    made(answered, at, Texture)
}

pub unsafe fn resource_view(device: Device, texture: Texture) -> Option<ResourceView> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_shader_resource_view)(
            object(device.0),
            object(texture.0),
            std::ptr::null(),
            &mut at,
        )
    };
    made(answered, at, ResourceView)
}

pub unsafe fn blend_state(device: Device, desc: BlendDesc) -> Option<Blend> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_blend_state)(object(device.0), &desc, &mut at)
    };
    made(answered, at, Blend)
}

pub unsafe fn sampler_state(device: Device, desc: SamplerDesc) -> Option<Sampler> {
    let mut at: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.0).create_sampler_state)(object(device.0), &desc, &mut at)
    };
    made(answered, at, Sampler)
}

pub unsafe fn write(context: Context, buffer: Buffer, bytes: &[u8]) -> bool {
    let mut mapped = MappedSubresource {
        data: std::ptr::null_mut(),
        row_pitch: 0,
        depth_pitch: 0,
    };
    let answered = unsafe {
        (vtable::<ContextVtable>(context.0).map)(
            object(context.0),
            object(buffer.0),
            0,
            D3D11_MAP_WRITE_DISCARD,
            0,
            &mut mapped,
        )
    };
    if answered < 0 || mapped.data.is_null() {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.data.cast::<u8>(), bytes.len());
        (vtable::<ContextVtable>(context.0).unmap)(object(context.0), object(buffer.0), 0);
    }
    true
}

pub unsafe fn set_render_targets(context: Context, targets: &[Target], depth: DepthView) {
    let targets: Vec<*mut c_void> = targets.iter().map(|target| object(target.0)).collect();
    unsafe {
        (vtable::<ContextVtable>(context.0).set_render_targets)(
            object(context.0),
            targets.len() as u32,
            targets.as_ptr(),
            object(depth.0),
        );
    }
}

pub unsafe fn set_viewports(context: Context, viewports: &[Viewport]) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_viewports)(
            object(context.0),
            viewports.len() as u32,
            viewports.as_ptr(),
        );
    }
}

pub unsafe fn set_blend_state(context: Context, blend: Blend, factor: [f32; 4], mask: u32) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_blend_state)(
            object(context.0),
            object(blend.0),
            &factor,
            mask,
        );
    }
}

pub unsafe fn set_depth_stencil(context: Context, state: DepthStencil, reference: u32) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_depth_stencil_state)(
            object(context.0),
            object(state.0),
            reference,
        );
    }
}

pub unsafe fn set_rasterizer(context: Context, state: Rasterizer) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_rasterizer_state)(
            object(context.0),
            object(state.0),
        );
    }
}

// ---- and the twelve reads that put it back ----------------------------------------------------------
//
// **Each one a slot and nothing more**, which is this file's whole job. What the counts asked for are, and
// which of these a drawing needs, is `orb_core::canvas::Saved`'s.

pub unsafe fn render_targets(context: Context, most: usize) -> (Vec<Target>, DepthView) {
    let mut targets: Vec<*mut c_void> = vec![std::ptr::null_mut(); most];
    let mut depth: *mut c_void = std::ptr::null_mut();
    unsafe {
        (vtable::<ContextVtable>(context.0).get_render_targets)(
            object(context.0),
            most as u32,
            targets.as_mut_ptr(),
            &mut depth,
        );
    }
    let targets = targets
        .into_iter()
        .map(|at| Target(at as usize))
        .collect::<Vec<_>>();
    (targets, DepthView(depth as usize))
}

pub unsafe fn blend_state_set(context: Context) -> (Blend, [f32; 4], u32) {
    let mut blend: *mut c_void = std::ptr::null_mut();
    let mut factor = [0.0f32; 4];
    let mut mask = 0u32;
    unsafe {
        (vtable::<ContextVtable>(context.0).get_blend_state)(
            object(context.0),
            &mut blend,
            &mut factor,
            &mut mask,
        );
    }
    (Blend(blend as usize), factor, mask)
}

pub unsafe fn depth_stencil_set(context: Context) -> (DepthStencil, u32) {
    let mut state: *mut c_void = std::ptr::null_mut();
    let mut reference = 0u32;
    unsafe {
        (vtable::<ContextVtable>(context.0).get_depth_stencil_state)(
            object(context.0),
            &mut state,
            &mut reference,
        );
    }
    (DepthStencil(state as usize), reference)
}

pub unsafe fn rasterizer_set(context: Context) -> Rasterizer {
    let mut state: *mut c_void = std::ptr::null_mut();
    unsafe {
        (vtable::<ContextVtable>(context.0).get_rasterizer_state)(object(context.0), &mut state);
    }
    Rasterizer(state as usize)
}

/// **The count asked for is a count answered as well**, which is what the first argument of
/// `RSGetViewports` is: told how many there is room for, it writes how many there were. So the answer is
/// cut to that, and a context with none set answers nought of them.
pub unsafe fn viewports_set(context: Context, most: usize) -> Vec<Viewport> {
    let mut how_many = most as u32;
    let mut viewports = vec![Viewport::default(); most];
    unsafe {
        (vtable::<ContextVtable>(context.0).get_viewports)(
            object(context.0),
            &mut how_many,
            viewports.as_mut_ptr(),
        );
    }
    viewports.truncate((how_many as usize).min(most));
    viewports
}

pub unsafe fn input_layout_set(context: Context) -> Layout {
    let mut at: *mut c_void = std::ptr::null_mut();
    unsafe {
        (vtable::<ContextVtable>(context.0).get_input_layout)(object(context.0), &mut at);
    }
    Layout(at as usize)
}

pub unsafe fn vertex_buffer_set(context: Context) -> (Buffer, u32, u32) {
    let mut at: *mut c_void = std::ptr::null_mut();
    let mut stride = 0u32;
    let mut offset = 0u32;
    unsafe {
        (vtable::<ContextVtable>(context.0).get_vertex_buffers)(
            object(context.0),
            0,
            1,
            &mut at,
            &mut stride,
            &mut offset,
        );
    }
    (Buffer(at as usize), stride, offset)
}

pub unsafe fn topology_set(context: Context) -> u32 {
    let mut topology = 0u32;
    unsafe {
        (vtable::<ContextVtable>(context.0).get_primitive_topology)(
            object(context.0),
            &mut topology,
        );
    }
    topology
}

/// **The shader alone, and none of the class instances the slot also answers.** Told nought of them it
/// writes none, which is what orb wants: nothing it draws uses one.
pub unsafe fn vertex_shader_set(context: Context) -> VertexShader {
    let mut at: *mut c_void = std::ptr::null_mut();
    let mut how_many = 0u32;
    unsafe {
        (vtable::<ContextVtable>(context.0).get_vertex_shader)(
            object(context.0),
            &mut at,
            std::ptr::null_mut(),
            &mut how_many,
        );
    }
    VertexShader(at as usize)
}

pub unsafe fn pixel_shader_set(context: Context) -> PixelShader {
    let mut at: *mut c_void = std::ptr::null_mut();
    let mut how_many = 0u32;
    unsafe {
        (vtable::<ContextVtable>(context.0).get_pixel_shader)(
            object(context.0),
            &mut at,
            std::ptr::null_mut(),
            &mut how_many,
        );
    }
    PixelShader(at as usize)
}

pub unsafe fn resource_view_set(context: Context) -> ResourceView {
    let mut at: *mut c_void = std::ptr::null_mut();
    unsafe {
        (vtable::<ContextVtable>(context.0).get_pixel_shader_resources)(
            object(context.0),
            0,
            1,
            &mut at,
        );
    }
    ResourceView(at as usize)
}

pub unsafe fn sampler_set(context: Context) -> Sampler {
    let mut at: *mut c_void = std::ptr::null_mut();
    unsafe {
        (vtable::<ContextVtable>(context.0).get_pixel_samplers)(object(context.0), 0, 1, &mut at);
    }
    Sampler(at as usize)
}

pub unsafe fn set_input_layout(context: Context, layout: Layout) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_input_layout)(object(context.0), object(layout.0));
    }
}

pub unsafe fn set_vertex_buffer(context: Context, buffer: Buffer, stride: u32, offset: u32) {
    let buffers = [object(buffer.0)];
    unsafe {
        (vtable::<ContextVtable>(context.0).set_vertex_buffers)(
            object(context.0),
            0,
            1,
            buffers.as_ptr(),
            &stride,
            &offset,
        );
    }
}

pub unsafe fn set_topology(context: Context, topology: u32) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_primitive_topology)(object(context.0), topology);
    }
}

pub unsafe fn set_vertex_shader(context: Context, shader: VertexShader) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_vertex_shader)(
            object(context.0),
            object(shader.0),
            std::ptr::null(),
            0,
        );
    }
}

pub unsafe fn set_pixel_shader(context: Context, shader: PixelShader) {
    unsafe {
        (vtable::<ContextVtable>(context.0).set_pixel_shader)(
            object(context.0),
            object(shader.0),
            std::ptr::null(),
            0,
        );
    }
}

pub unsafe fn set_resource_view(context: Context, view: ResourceView) {
    let views = [object(view.0)];
    unsafe {
        (vtable::<ContextVtable>(context.0).set_pixel_shader_resources)(
            object(context.0),
            0,
            1,
            views.as_ptr(),
        );
    }
}

pub unsafe fn set_sampler(context: Context, sampler: Sampler) {
    let samplers = [object(sampler.0)];
    unsafe {
        (vtable::<ContextVtable>(context.0).set_pixel_samplers)(
            object(context.0),
            0,
            1,
            samplers.as_ptr(),
        );
    }
}

pub unsafe fn draw(context: Context, vertices: u32, first: u32) {
    unsafe {
        (vtable::<ContextVtable>(context.0).draw)(object(context.0), vertices, first);
    }
}

pub unsafe fn release(object_of: Object) -> u32 {
    let at = object_of.at();
    if at == 0 {
        return 0;
    }
    unsafe { (vtable::<UnknownVtable>(at).release)(object(at)) }
}

/// `D3DCompile`, as `d3dcompiler` declares it: the source and its length, a name for it in the messages, no
/// macros and no include handler, the entry point and the target profile, two sets of flags, and two blobs
/// out — the code and whatever the compiler had to say.
type D3dCompile = unsafe extern "system" fn(
    *const c_void,
    usize,
    *const u8,
    *const c_void,
    *const c_void,
    *const u8,
    *const u8,
    u32,
    u32,
    *mut *mut c_void,
    *mut *mut c_void,
) -> i32;

/// **The `d3dcompiler` versions asked for, newest first.** th06c names these in its `.rdata` and walks
/// them the same way, so whichever it found is the one already loaded.
///
/// **And where none of them is, one is brought in.** Measured on the real game: th06c names all fifteen
/// and had loaded none of them by the frame orb's overlay is built on — `canvas: the vertex shader would
/// not compile: none of 15 is loaded with a D3DCompile in it`, every launch — so a canvas that only asked
/// after the game's own was a canvas that never came up. `d3dcompiler_47.dll` ships with Windows, so the
/// newest of the list is usually the first to answer; the walk is kept for a machine where it does not.
const COMPILERS: [&str; 15] = [
    "d3dcompiler_47.dll",
    "d3dcompiler_46.dll",
    "d3dcompiler_45.dll",
    "d3dcompiler_44.dll",
    "d3dcompiler_43.dll",
    "d3dcompiler_42.dll",
    "d3dcompiler_41.dll",
    "d3dcompiler_40.dll",
    "d3dcompiler_39.dll",
    "d3dcompiler_38.dll",
    "d3dcompiler_37.dll",
    "d3dcompiler_36.dll",
    "d3dcompiler_35.dll",
    "d3dcompiler_34.dll",
    "d3dcompiler_33.dll",
];

pub fn compile(source: &str, entry: &str, profile: &str) -> Result<Vec<u8>, String> {
    // Whichever the game already has, and then whichever the machine will bring in: the one it has is
    // preferred so that orb compiles through the same compiler the game's own shaders went through.
    let loaded = COMPILERS
        .iter()
        .find_map(|name| super::module::proc_address(name, "D3DCompile"));
    let Some(at) = loaded.or_else(|| {
        COMPILERS.iter().find_map(|name| {
            super::module::brings_in(name)
                .then(|| super::module::proc_address(name, "D3DCompile"))
                .flatten()
        })
    }) else {
        return Err(format!(
            "none of {} is loaded or can be, with a D3DCompile in it",
            COMPILERS.len()
        ));
    };
    let compile: D3dCompile = unsafe { std::mem::transmute(at) };
    let name = c"orb's own overlay";
    let entry = std::ffi::CString::new(entry).map_err(|why| format!("the entry point: {why}"))?;
    let profile = std::ffi::CString::new(profile).map_err(|why| format!("the profile: {why}"))?;
    let mut code: *mut c_void = std::ptr::null_mut();
    let mut errors: *mut c_void = std::ptr::null_mut();
    let answered = unsafe {
        compile(
            source.as_ptr().cast(),
            source.len(),
            name.as_ptr().cast(),
            std::ptr::null(),
            std::ptr::null(),
            entry.as_ptr().cast(),
            profile.as_ptr().cast(),
            0,
            0,
            &mut code,
            &mut errors,
        )
    };
    // What it had to say, taken before anything is given back: a compile that worked can still have said
    // something, and one that failed says only this.
    let said = match errors.is_null() {
        true => String::new(),
        false => {
            let bytes = unsafe { blob(errors) };
            unsafe { release_blob(errors) };
            String::from_utf8_lossy(&bytes)
                .trim_end_matches('\0')
                .trim()
                .to_owned()
        }
    };
    if answered < 0 || code.is_null() {
        return Err(match said.is_empty() {
            true => format!("D3DCompile answered {answered:#x} and said nothing"),
            false => said,
        });
    }
    let bytecode = unsafe { blob(code) };
    unsafe { release_blob(code) };
    Ok(bytecode)
}

/// A blob's bytes copied out of it, so that the caller owns them and the blob can go.
///
/// # Safety
/// `at` must be a live `ID3DBlob`.
unsafe fn blob(at: *mut c_void) -> Vec<u8> {
    let table = unsafe { vtable::<BlobVtable>(at as usize) };
    let bytes = unsafe { (table.get_buffer_pointer)(at) };
    let len = unsafe { (table.get_buffer_size)(at) };
    match bytes.is_null() || len == 0 {
        true => Vec::new(),
        false => unsafe { std::slice::from_raw_parts(bytes.cast::<u8>(), len) }.to_vec(),
    }
}

/// # Safety
/// `at` must be a live `ID3DBlob` and this must be a reference the caller holds.
unsafe fn release_blob(at: *mut c_void) {
    unsafe { (vtable::<UnknownVtable>(at as usize).release)(at) };
}
