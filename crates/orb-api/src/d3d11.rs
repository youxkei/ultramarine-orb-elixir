//! The Direct3D 11 objects a game's swapchain leads to, and the drawing done through them.
//!
//! **A mirror of the slots and not an abstraction over them**, the way [`crate::d3d8`] is. Every function
//! here takes what the vtable slot behind it takes. What must not cross is a *decision* — the shaders' own
//! source, the vertex layout, the blend mode, the topology and the viewport are all above this line,
//! because they are what an e2e test over the drawing is about. A seam that said *draw this text here*
//! would take all of that into [`crate::real::d3d11`], where no e2e test reaches.
//!
//! **The vtables are declared here because nothing else declares them.** `windows-sys` carries no COM
//! interfaces at all — its `Win32::Graphics` holds Dwm, Gdi, GdiPlus, Hlsl, OpenGL and Printing, and no
//! `Direct3D11` — so this is the same position `d3d8.rs` is in for Direct3D 8 and it is answered the same
//! way. Only the slots that get called are typed, the rest being pointer-sized padding, and every typed
//! slot's index is asserted at compile time against the one it has in the header mingw-w64 ships:
//!
//! ```sh
//! grep -A200 'typedef struct ID3D11DeviceVtbl' /usr/share/mingw-w64/include/d3d11.h
//! ```
//!
//! A wrong index is a call into an unrelated method with the wrong signature, which is why they are
//! asserted rather than read once: `GetImmediateContext` is slot 40 and not the 39 that counting the
//! interface's declared methods by hand gives.
//!
//! **And the shaders are compiled at runtime, out of the game's own compiler.** A `d3dcompiler` is already
//! loaded in a process that draws this way — th06c names `D3DCompiler_33` through `_47` in its `.rdata` —
//! so [`compile`] asks the loaded one for `D3DCompile` and hands it HLSL. The alternative is bytecode
//! built by `fxc`, a Windows tool this repository is not built with: checked-in bytecode nobody here can
//! regenerate is worse than a compile that can fail and say so.
//!
//! **Under a simulated host these go to the double instead**, which is `orb_sim::Recording` — the same one
//! the Direct3D 8 drawing goes to, so a quad drawn through a canvas comes back as the quad an e2e test
//! already knows how to read. **Including the state a quad is drawn under**, which that host kept none of
//! until the drawing had to put it back: see [`Set`], which has what keeping none of it cost.

use std::ffi::c_void;
use std::mem::offset_of;

use crate::Hresult;

/// An `ID3D11Device`, as the address it is at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device(pub usize);
/// Its `ID3D11DeviceContext`, which every per-frame call goes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context(pub usize);
/// An `ID3D11Buffer`, which here is only ever one quad's vertices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buffer(pub usize);
/// An `ID3D11Texture2D` — the swapchain's own back buffer, or one made from pixels orb baked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture(pub usize);
/// An `ID3D11ShaderResourceView` over one of those, which is how a shader reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceView(pub usize);
/// An `ID3D11RenderTargetView` over the back buffer, which is what the drawing lands in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexShader(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelShader(pub usize);
/// An `ID3D11InputLayout`, which is the vertex layout as the device was told it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blend(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sampler(pub usize);
/// An `ID3D11DepthStencilState` and an `ID3D11RasterizerState`. **Orb makes neither**: what it sets for
/// its own draw is the device's default, which is what a nought one of these means — see
/// [`set_depth_stencil`] and [`set_rasterizer`]. They are named because the *game*'s two have to be read
/// back and put back, which is what `orb_core::canvas::Saved` is for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DepthStencil(pub usize);
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rasterizer(pub usize);
/// An `ID3D11DepthStencilView`, which is bound with the render targets rather than beside them. Orb binds
/// none — nothing it draws is depth-tested — but the game's has to be put back with its targets, which is
/// why it is named.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DepthView(pub usize);

/// Anything above that a release is asked of, so that one function closes them all: `Release` is
/// `IUnknown`'s and sits at the same slot in every one of these vtables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Object {
    Device(Device),
    Context(Context),
    Buffer(Buffer),
    Texture(Texture),
    ResourceView(ResourceView),
    Target(Target),
    VertexShader(VertexShader),
    PixelShader(PixelShader),
    Layout(Layout),
    Blend(Blend),
    Sampler(Sampler),
    DepthStencil(DepthStencil),
    Rasterizer(Rasterizer),
    DepthView(DepthView),
}

impl Object {
    /// The address it is at, which is all a release needs.
    pub fn at(self) -> usize {
        match self {
            Self::Device(it) => it.0,
            Self::Context(it) => it.0,
            Self::Buffer(it) => it.0,
            Self::Texture(it) => it.0,
            Self::ResourceView(it) => it.0,
            Self::Target(it) => it.0,
            Self::VertexShader(it) => it.0,
            Self::PixelShader(it) => it.0,
            Self::Layout(it) => it.0,
            Self::Blend(it) => it.0,
            Self::Sampler(it) => it.0,
            Self::DepthStencil(it) => it.0,
            Self::Rasterizer(it) => it.0,
            Self::DepthView(it) => it.0,
        }
    }
}

// ---- the numbers `d3d11.h` and `dxgiformat.h` fix ----

pub const DXGI_FORMAT_R32G32B32A32_FLOAT: u32 = 0x2;
pub const DXGI_FORMAT_R32G32_FLOAT: u32 = 0x10;
pub const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 0x1c;
/// The one a texture of `0xAARRGGBB` words is: `orb_api::text` bakes a line of glyphs as those, and on a
/// little-endian machine one of them is the bytes `BB GG RR AA` — which is this format's own order and not
/// `R8G8B8A8`'s. The two differ only in which channel a byte is, so the wrong one of them draws text with
/// its red and its blue swapped rather than failing.
pub const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 0x57;

pub const D3D11_USAGE_DEFAULT: u32 = 0;
pub const D3D11_USAGE_DYNAMIC: u32 = 2;
pub const D3D11_BIND_VERTEX_BUFFER: u32 = 0x1;
pub const D3D11_BIND_SHADER_RESOURCE: u32 = 0x8;
pub const D3D11_CPU_ACCESS_WRITE: u32 = 0x10000;
pub const D3D11_MAP_WRITE_DISCARD: u32 = 4;
pub const D3D11_INPUT_PER_VERTEX_DATA: u32 = 0;
pub const D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP: u32 = 5;

pub const D3D11_BLEND_ONE: u32 = 2;
pub const D3D11_BLEND_SRC_ALPHA: u32 = 5;
pub const D3D11_BLEND_INV_SRC_ALPHA: u32 = 6;
pub const D3D11_BLEND_OP_ADD: u32 = 1;
pub const D3D11_COLOR_WRITE_ENABLE_ALL: u8 = 0xf;

pub const D3D11_FILTER_MIN_MAG_MIP_LINEAR: u32 = 0x15;
pub const D3D11_TEXTURE_ADDRESS_CLAMP: u32 = 3;
pub const D3D11_COMPARISON_NEVER: u32 = 1;
/// `D3D11_FLOAT32_MAX`, which is what a sampler's `MaxLOD` is left at.
pub const D3D11_FLOAT32_MAX: f32 = 3.402_823_5e38;

// ---- the structures those calls take, field for field ----

/// `D3D11_BUFFER_DESC`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferDesc {
    pub byte_width: u32,
    pub usage: u32,
    pub bind_flags: u32,
    pub cpu_access_flags: u32,
    pub misc_flags: u32,
    pub structure_byte_stride: u32,
}

/// `DXGI_SAMPLE_DESC`, which a texture description carries one of.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SampleDesc {
    pub count: u32,
    pub quality: u32,
}

impl Default for SampleDesc {
    /// One sample and the default quality, which is a texture nobody is multisampling.
    fn default() -> Self {
        Self {
            count: 1,
            quality: 0,
        }
    }
}

/// `D3D11_TEXTURE2D_DESC`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Texture2dDesc {
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub array_size: u32,
    pub format: u32,
    pub sample_desc: SampleDesc,
    pub usage: u32,
    pub bind_flags: u32,
    pub cpu_access_flags: u32,
    pub misc_flags: u32,
}

/// `D3D11_SUBRESOURCE_DATA`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SubresourceData {
    pub sys_mem: *const c_void,
    pub sys_mem_pitch: u32,
    pub sys_mem_slice_pitch: u32,
}

/// `D3D11_INPUT_ELEMENT_DESC`. The semantic name is a C string the caller keeps alive across the call.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InputElementDesc {
    pub semantic_name: *const u8,
    pub semantic_index: u32,
    pub format: u32,
    pub input_slot: u32,
    pub aligned_byte_offset: u32,
    pub input_slot_class: u32,
    pub instance_data_step_rate: u32,
}

/// `D3D11_RENDER_TARGET_BLEND_DESC`. `WINBOOL` is a 32-bit int, so the enable is a `u32`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderTargetBlendDesc {
    pub blend_enable: u32,
    pub src_blend: u32,
    pub dest_blend: u32,
    pub blend_op: u32,
    pub src_blend_alpha: u32,
    pub dest_blend_alpha: u32,
    pub blend_op_alpha: u32,
    pub render_target_write_mask: u8,
}

/// `D3D11_BLEND_DESC`, whose eight render targets are one array and only the first of which is read where
/// independent blending is off.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlendDesc {
    pub alpha_to_coverage_enable: u32,
    pub independent_blend_enable: u32,
    pub render_target: [RenderTargetBlendDesc; 8],
}

/// `D3D11_SAMPLER_DESC`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SamplerDesc {
    pub filter: u32,
    pub address_u: u32,
    pub address_v: u32,
    pub address_w: u32,
    pub mip_lod_bias: f32,
    pub max_anisotropy: u32,
    pub comparison_func: u32,
    pub border_color: [f32; 4],
    pub min_lod: f32,
    pub max_lod: f32,
}

/// `D3D11_VIEWPORT`, which is floats where `D3DVIEWPORT8` is words.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Viewport {
    pub top_left_x: f32,
    pub top_left_y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

/// `D3D11_MAPPED_SUBRESOURCE`, as a map hands it back.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MappedSubresource {
    pub data: *mut c_void,
    pub row_pitch: u32,
    pub depth_pitch: u32,
}

// ---- the vtables, and the index of every slot that is called ----

/// `IDXGISwapChain`'s, for the two slots a drawer needs of it. Its `Present` is `orb_core::dxgi`'s and is
/// not called from here.
#[repr(C)]
pub struct SwapChainVtable {
    pub _iunknown: [usize; 3],
    pub _dxgi_object: [usize; 4],
    pub get_device: unsafe extern "system" fn(*mut c_void, *const u8, *mut *mut c_void) -> Hresult,
    pub _present: usize,
    pub get_buffer:
        unsafe extern "system" fn(*mut c_void, u32, *const u8, *mut *mut c_void) -> Hresult,
}

const _: () = assert!(offset_of!(SwapChainVtable, get_device) == 7 * size_of::<usize>());
const _: () = assert!(offset_of!(SwapChainVtable, get_buffer) == 9 * size_of::<usize>());

/// `ID3D11Device`'s. Everything a drawer makes once is made through one of these.
#[repr(C)]
pub struct DeviceVtable {
    pub _iunknown: [usize; 3],
    pub create_buffer: unsafe extern "system" fn(
        *mut c_void,
        *const BufferDesc,
        *const SubresourceData,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_4: usize,
    pub create_texture_2d: unsafe extern "system" fn(
        *mut c_void,
        *const Texture2dDesc,
        *const SubresourceData,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_6: usize,
    pub create_shader_resource_view: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const c_void,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_8: usize,
    pub create_render_target_view: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const c_void,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_10: usize,
    pub create_input_layout: unsafe extern "system" fn(
        *mut c_void,
        *const InputElementDesc,
        u32,
        *const u8,
        usize,
        *mut *mut c_void,
    ) -> Hresult,
    pub create_vertex_shader: unsafe extern "system" fn(
        *mut c_void,
        *const u8,
        usize,
        *mut c_void,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_13_to_14: [usize; 2],
    pub create_pixel_shader: unsafe extern "system" fn(
        *mut c_void,
        *const u8,
        usize,
        *mut c_void,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_16_to_19: [usize; 4],
    pub create_blend_state:
        unsafe extern "system" fn(*mut c_void, *const BlendDesc, *mut *mut c_void) -> Hresult,
    pub _slot_21_to_22: [usize; 2],
    pub create_sampler_state:
        unsafe extern "system" fn(*mut c_void, *const SamplerDesc, *mut *mut c_void) -> Hresult,
    pub _slot_24_to_39: [usize; 16],
    pub get_immediate_context: unsafe extern "system" fn(*mut c_void, *mut *mut c_void),
}

const _: () = assert!(offset_of!(DeviceVtable, create_buffer) == 3 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, create_texture_2d) == 5 * size_of::<usize>());
const _: () =
    assert!(offset_of!(DeviceVtable, create_shader_resource_view) == 7 * size_of::<usize>());
const _: () =
    assert!(offset_of!(DeviceVtable, create_render_target_view) == 9 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, create_input_layout) == 11 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, create_vertex_shader) == 12 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, create_pixel_shader) == 15 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, create_blend_state) == 20 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, create_sampler_state) == 23 * size_of::<usize>());
const _: () = assert!(offset_of!(DeviceVtable, get_immediate_context) == 40 * size_of::<usize>());

/// `ID3D11DeviceContext`'s, for the fifteen slots one quad takes.
#[repr(C)]
pub struct ContextVtable {
    pub _iunknown: [usize; 3],
    pub _device_child: [usize; 4],
    pub _slot_7: usize,
    pub set_pixel_shader_resources:
        unsafe extern "system" fn(*mut c_void, u32, u32, *const *mut c_void),
    pub set_pixel_shader:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *const *mut c_void, u32),
    pub set_pixel_samplers: unsafe extern "system" fn(*mut c_void, u32, u32, *const *mut c_void),
    pub set_vertex_shader:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *const *mut c_void, u32),
    pub _slot_12: usize,
    pub draw: unsafe extern "system" fn(*mut c_void, u32, u32),
    pub map: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        u32,
        u32,
        u32,
        *mut MappedSubresource,
    ) -> Hresult,
    pub unmap: unsafe extern "system" fn(*mut c_void, *mut c_void, u32),
    pub _slot_16: usize,
    pub set_input_layout: unsafe extern "system" fn(*mut c_void, *mut c_void),
    pub set_vertex_buffers: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        *const *mut c_void,
        *const u32,
        *const u32,
    ),
    pub _slot_19_to_23: [usize; 5],
    pub set_primitive_topology: unsafe extern "system" fn(*mut c_void, u32),
    pub _slot_25_to_32: [usize; 8],
    pub set_render_targets:
        unsafe extern "system" fn(*mut c_void, u32, *const *mut c_void, *mut c_void),
    pub _slot_34: usize,
    pub set_blend_state: unsafe extern "system" fn(*mut c_void, *mut c_void, *const [f32; 4], u32),
    pub set_depth_stencil_state: unsafe extern "system" fn(*mut c_void, *mut c_void, u32),
    pub _slot_37_to_42: [usize; 6],
    pub set_rasterizer_state: unsafe extern "system" fn(*mut c_void, *mut c_void),
    pub set_viewports: unsafe extern "system" fn(*mut c_void, u32, *const Viewport),
    // **And the twelve that read the same state back**, which is what the drawing has to put back — see
    // `orb_core::canvas::Saved`. Direct3D 11 has no state block, so every piece is its own call, and each
    // of these adds a reference to whatever it hands back.
    pub _slot_45_to_72: [usize; 28],
    pub get_pixel_shader_resources:
        unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void),
    pub get_pixel_shader:
        unsafe extern "system" fn(*mut c_void, *mut *mut c_void, *mut *mut c_void, *mut u32),
    pub get_pixel_samplers: unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void),
    pub get_vertex_shader:
        unsafe extern "system" fn(*mut c_void, *mut *mut c_void, *mut *mut c_void, *mut u32),
    pub _slot_77: usize,
    pub get_input_layout: unsafe extern "system" fn(*mut c_void, *mut *mut c_void),
    pub get_vertex_buffers:
        unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void, *mut u32, *mut u32),
    pub _slot_80_to_82: [usize; 3],
    pub get_primitive_topology: unsafe extern "system" fn(*mut c_void, *mut u32),
    pub _slot_84_to_88: [usize; 5],
    pub get_render_targets:
        unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void, *mut *mut c_void),
    pub _slot_90: usize,
    pub get_blend_state:
        unsafe extern "system" fn(*mut c_void, *mut *mut c_void, *mut [f32; 4], *mut u32),
    pub get_depth_stencil_state: unsafe extern "system" fn(*mut c_void, *mut *mut c_void, *mut u32),
    pub _slot_93: usize,
    pub get_rasterizer_state: unsafe extern "system" fn(*mut c_void, *mut *mut c_void),
    pub get_viewports: unsafe extern "system" fn(*mut c_void, *mut u32, *mut Viewport),
}

// **The eight here is a slot index and not a count of bits**, which `manual_bits` cannot tell apart: every
// assert in this file is `<slot> * size_of::<usize>()`, and this is the one whose slot happens to be the
// number of bits in a byte. Written as the rest are, so that reading down the list gives the slot numbers.
#[allow(clippy::manual_bits)]
const _: () =
    assert!(offset_of!(ContextVtable, set_pixel_shader_resources) == 8 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_pixel_shader) == 9 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_pixel_samplers) == 10 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_vertex_shader) == 11 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, draw) == 13 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, map) == 14 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, unmap) == 15 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_input_layout) == 17 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_vertex_buffers) == 18 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_primitive_topology) == 24 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_render_targets) == 33 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_blend_state) == 35 * size_of::<usize>());
const _: () =
    assert!(offset_of!(ContextVtable, set_depth_stencil_state) == 36 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_rasterizer_state) == 43 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, set_viewports) == 44 * size_of::<usize>());
const _: () =
    assert!(offset_of!(ContextVtable, get_pixel_shader_resources) == 73 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_pixel_shader) == 74 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_pixel_samplers) == 75 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_vertex_shader) == 76 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_input_layout) == 78 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_vertex_buffers) == 79 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_primitive_topology) == 83 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_render_targets) == 89 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_blend_state) == 91 * size_of::<usize>());
const _: () =
    assert!(offset_of!(ContextVtable, get_depth_stencil_state) == 92 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_rasterizer_state) == 94 * size_of::<usize>());
const _: () = assert!(offset_of!(ContextVtable, get_viewports) == 95 * size_of::<usize>());

/// `IUnknown`'s three, for the release every one of these objects takes.
#[repr(C)]
pub struct UnknownVtable {
    pub _query_interface: usize,
    pub _add_ref: usize,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
}

const _: () = assert!(offset_of!(UnknownVtable, release) == 2 * size_of::<usize>());

/// `ID3DBlob`'s two, which is how a compiled shader's bytes are reached.
#[repr(C)]
pub struct BlobVtable {
    pub _iunknown: [usize; 3],
    pub get_buffer_pointer: unsafe extern "system" fn(*mut c_void) -> *mut c_void,
    pub get_buffer_size: unsafe extern "system" fn(*mut c_void) -> usize,
}

const _: () = assert!(offset_of!(BlobVtable, get_buffer_pointer) == 3 * size_of::<usize>());
const _: () = assert!(offset_of!(BlobVtable, get_buffer_size) == 4 * size_of::<usize>());

/// `IID_ID3D11Device`, `db6f6ddb-ac77-4e88-8253-819df9bbf140`, as the sixteen bytes a `GUID` is — its
/// first three fields little-endian and the last eight in the order they are written, the way
/// `orb_core::dxgi` spells the swapchain's.
pub const IID_ID3D11DEVICE: [u8; 16] = [
    0xdb, 0x6d, 0x6f, 0xdb, 0x77, 0xac, 0x88, 0x4e, 0x82, 0x53, 0x81, 0x9d, 0xf9, 0xbb, 0xf1, 0x40,
];
/// `IID_ID3D11Texture2D`, `6f15aaf2-d208-4e89-9ab4-489535d34f9c`.
pub const IID_ID3D11TEXTURE2D: [u8; 16] = [
    0xf2, 0xaa, 0x15, 0x6f, 0x08, 0xd2, 0x89, 0x4e, 0x9a, 0xb4, 0x48, 0x95, 0x35, 0xd3, 0x4f, 0x9c,
];

/// **The simulated host, where a test has one installed.** Written once here rather than at each of the
/// calls below, because the `sim` feature being off has to leave nothing behind for the optimiser to keep.
#[allow(clippy::unnecessary_wraps)]
fn sim() -> Option<std::sync::Arc<dyn crate::Win>> {
    #[cfg(feature = "sim")]
    return crate::installed();
    #[cfg(not(feature = "sim"))]
    None
}

// ---- the seam itself ----

/// **The `ID3D11Device` a swapchain belongs to.** `None` where the object answers no such interface,
/// which is a swapchain of some other API's device.
///
/// # Safety
/// `swapchain` must be a live `IDXGISwapChain`.
pub unsafe fn device_of(swapchain: usize) -> Option<Device> {
    if let Some(win) = sim() {
        return win.d3d11_device_of(swapchain);
    }
    unsafe { host::device_of(swapchain) }
}

/// Its immediate context, which every per-frame call goes through.
///
/// # Safety
/// `device` must be a live `ID3D11Device`.
pub unsafe fn immediate_context(device: Device) -> Option<Context> {
    if let Some(win) = sim() {
        return win.d3d11_immediate_context(device);
    }
    unsafe { host::immediate_context(device) }
}

/// The swapchain's own back buffer, which is the texture a frame is drawn into.
///
/// # Safety
/// `swapchain` must be a live `IDXGISwapChain`.
pub unsafe fn back_buffer(swapchain: usize) -> Option<Texture> {
    if let Some(win) = sim() {
        return win.d3d11_back_buffer(swapchain);
    }
    unsafe { host::back_buffer(swapchain) }
}

/// A render target view over a texture, with the description left to the device — which for a back buffer
/// is the view of the whole of it in its own format.
///
/// # Safety
/// Both must be live objects of the same device.
pub unsafe fn render_target(device: Device, texture: Texture) -> Option<Target> {
    if let Some(win) = sim() {
        return win.d3d11_render_target(device, texture);
    }
    unsafe { host::render_target(device, texture) }
}

/// **HLSL compiled by whichever `d3dcompiler` the process already has.** The error is what the compiler
/// said, or that none is loaded.
pub fn compile(source: &str, entry: &str, profile: &str) -> Result<Vec<u8>, String> {
    if let Some(win) = sim() {
        return win.d3d11_compile(source, entry, profile);
    }
    host::compile(source, entry, profile)
}

/// # Safety
/// `device` must be live, and `bytecode` a compiled vertex shader of a profile it supports.
pub unsafe fn vertex_shader(device: Device, bytecode: &[u8]) -> Option<VertexShader> {
    if let Some(win) = sim() {
        return win.d3d11_vertex_shader(device, bytecode);
    }
    unsafe { host::vertex_shader(device, bytecode) }
}

/// # Safety
/// As [`vertex_shader`], for a pixel shader.
pub unsafe fn pixel_shader(device: Device, bytecode: &[u8]) -> Option<PixelShader> {
    if let Some(win) = sim() {
        return win.d3d11_pixel_shader(device, bytecode);
    }
    unsafe { host::pixel_shader(device, bytecode) }
}

/// The vertex layout as the device is told it, checked against the vertex shader's own input signature.
///
/// # Safety
/// `device` must be live, `elements` must name semantics the shader declares, and every `semantic_name`
/// must point at a NUL-terminated string that outlives the call.
pub unsafe fn input_layout(
    device: Device,
    elements: &[InputElementDesc],
    bytecode: &[u8],
) -> Option<Layout> {
    if let Some(win) = sim() {
        return win.d3d11_input_layout(device, elements, bytecode);
    }
    unsafe { host::input_layout(device, elements, bytecode) }
}

/// A buffer of `desc`, with `bytes` as its initial contents where it has any.
///
/// # Safety
/// `device` must be live, and `bytes` — where given — must be `desc.byte_width` long.
pub unsafe fn buffer(device: Device, desc: BufferDesc, bytes: Option<&[u8]>) -> Option<Buffer> {
    if let Some(win) = sim() {
        return win.d3d11_buffer(device, desc, bytes);
    }
    unsafe { host::buffer(device, desc, bytes) }
}

/// A texture of `desc` filled with `pixels`, whose rows are `desc.width` long.
///
/// # Safety
/// `device` must be live, and `pixels` must hold `desc.width * desc.height` of them.
pub unsafe fn texture(device: Device, desc: Texture2dDesc, pixels: &[u32]) -> Option<Texture> {
    if let Some(win) = sim() {
        return win.d3d11_texture(device, desc, pixels);
    }
    unsafe { host::texture(device, desc, pixels) }
}

/// A shader's view of a texture, with the description left to the device.
///
/// # Safety
/// Both must be live objects of the same device.
pub unsafe fn resource_view(device: Device, texture: Texture) -> Option<ResourceView> {
    if let Some(win) = sim() {
        return win.d3d11_resource_view(device, texture);
    }
    unsafe { host::resource_view(device, texture) }
}

/// # Safety
/// `device` must be live.
pub unsafe fn blend_state(device: Device, desc: BlendDesc) -> Option<Blend> {
    if let Some(win) = sim() {
        return win.d3d11_blend_state(device, desc);
    }
    unsafe { host::blend_state(device, desc) }
}

/// # Safety
/// `device` must be live.
pub unsafe fn sampler_state(device: Device, desc: SamplerDesc) -> Option<Sampler> {
    if let Some(win) = sim() {
        return win.d3d11_sampler_state(device, desc);
    }
    unsafe { host::sampler_state(device, desc) }
}

/// **A dynamic buffer's contents replaced**, which is `Map` with `WRITE_DISCARD` and `Unmap` — the pair,
/// because a map whose unmap never came is a buffer the GPU cannot read. `false` where the map failed.
///
/// # Safety
/// `context` and `buffer` must be live objects of the same device, `buffer` must have been made `DYNAMIC`
/// with `CPU_ACCESS_WRITE`, and `bytes` must fit it.
pub unsafe fn write(context: Context, buffer: Buffer, bytes: &[u8]) -> bool {
    if let Some(win) = sim() {
        return win.d3d11_write(context, buffer, bytes);
    }
    unsafe { host::write(context, buffer, bytes) }
}

/// **What one of the ten state-setting calls was given**, for a simulated host to keep.
///
/// **Not an abstraction over the slots**: each of the ten is still its own function taking what its own
/// slot takes, and this is only the shape a *record* of them has — one variant per call, so that
/// [`crate::Win`] takes one method rather than ten for a thing no caller above the seam ever builds.
///
/// **And kept where it used to be dropped.** These were the ten a simulated host did not keep, on the
/// grounds that what they set is the state a quad is drawn under and the record holds the quad. That was
/// wrong the moment the drawing had to *put the state back*: which state the context is in after a frame
/// is then a question an e2e test asks, and a host keeping none of it cannot be asked. What it cost was a
/// launch of the real game whose screen went black.
#[derive(Debug, Clone, PartialEq)]
pub enum Set {
    RenderTargets(Vec<Target>, DepthView),
    BlendState(Blend, [f32; 4], u32),
    DepthStencil(DepthStencil, u32),
    Rasterizer(Rasterizer),
    InputLayout(Layout),
    VertexBuffer(Buffer, u32, u32),
    Topology(u32),
    VertexShader(VertexShader),
    PixelShader(PixelShader),
    Sampler(Sampler),
}

/// # Safety
/// Every object named must be live and the context's own.
pub unsafe fn set_render_targets(context: Context, targets: &[Target], depth: DepthView) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::RenderTargets(targets.to_vec(), depth));
    }
    unsafe { host::set_render_targets(context, targets, depth) }
}

/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_viewports(context: Context, viewports: &[Viewport]) {
    if let Some(win) = sim() {
        return win.d3d11_set_viewports(context, viewports);
    }
    unsafe { host::set_viewports(context, viewports) }
}

/// The blend state, with the factor and the sample mask the caller chooses.
///
/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_blend_state(context: Context, blend: Blend, factor: [f32; 4], mask: u32) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::BlendState(blend, factor, mask));
    }
    unsafe { host::set_blend_state(context, blend, factor, mask) }
}

/// The depth-stencil state and the stencil reference with it. A nought state is the device's own default,
/// which is what orb's own draw sets and what the game's is put back over.
///
/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_depth_stencil(context: Context, state: DepthStencil, reference: u32) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::DepthStencil(state, reference));
    }
    unsafe { host::set_depth_stencil(context, state, reference) }
}

/// And the rasterizer state, a nought one being the default the same way.
///
/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_rasterizer(context: Context, state: Rasterizer) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::Rasterizer(state));
    }
    unsafe { host::set_rasterizer(context, state) }
}

/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_input_layout(context: Context, layout: Layout) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::InputLayout(layout));
    }
    unsafe { host::set_input_layout(context, layout) }
}

/// One vertex buffer in slot zero, with the stride and offset the caller chooses.
///
/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_vertex_buffer(context: Context, buffer: Buffer, stride: u32, offset: u32) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::VertexBuffer(buffer, stride, offset));
    }
    unsafe { host::set_vertex_buffer(context, buffer, stride, offset) }
}

/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_topology(context: Context, topology: u32) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::Topology(topology));
    }
    unsafe { host::set_topology(context, topology) }
}

/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_vertex_shader(context: Context, shader: VertexShader) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::VertexShader(shader));
    }
    unsafe { host::set_vertex_shader(context, shader) }
}

/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_pixel_shader(context: Context, shader: PixelShader) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::PixelShader(shader));
    }
    unsafe { host::set_pixel_shader(context, shader) }
}

/// The texture a pixel shader reads, in slot zero.
///
/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_resource_view(context: Context, view: ResourceView) {
    if let Some(win) = sim() {
        return win.d3d11_set_resource_view(context, view);
    }
    unsafe { host::set_resource_view(context, view) }
}

/// And how it reads it, in slot zero.
///
/// # Safety
/// As [`set_render_targets`].
pub unsafe fn set_sampler(context: Context, sampler: Sampler) {
    if let Some(win) = sim() {
        return win.d3d11_sets(context, Set::Sampler(sampler));
    }
    unsafe { host::set_sampler(context, sampler) }
}

/// `vertices` of them from `first`, as whatever topology is set.
///
/// # Safety
/// As [`set_render_target`], with a vertex buffer, a layout and both shaders set.
pub unsafe fn draw(context: Context, vertices: u32, first: u32) {
    if let Some(win) = sim() {
        return win.d3d11_draw(context, vertices, first);
    }
    unsafe { host::draw(context, vertices, first) }
}

// ---- and the same state read back, which is what a drawing has to put back ------------------------
//
// **One read per `set_` above, and no struct holding them.** Direct3D 11 has no state block — the Direct3D
// 8 device's `CreateStateBlock` has no counterpart at all — so state is put back piece by piece, and every
// piece has to be read first. Which pieces those are is a decision and stays above this line:
// `orb_core::canvas::Saved` is the set, chosen to be exactly what its own draw changes.
//
// **Every object handed back carries a reference of its own**, `Get*` adding one. Whatever reads them has
// to release them, and a frame that does not is a frame leaking a dozen COM objects.

/// The render targets the context has set, and the depth view with them — as many as it was given, which
/// is what `OMGetRenderTargets` answers for the count asked of it.
///
/// # Safety
/// `context` must be live and this must run on its own thread.
pub unsafe fn render_targets(context: Context, most: usize) -> (Vec<Target>, DepthView) {
    if let Some(win) = sim() {
        return win.d3d11_render_targets(context, most);
    }
    unsafe { host::render_targets(context, most) }
}

/// The blend state, with the factor and the sample mask it was set with.
///
/// # Safety
/// As [`render_targets`].
pub unsafe fn blend_state_set(context: Context) -> (Blend, [f32; 4], u32) {
    if let Some(win) = sim() {
        return win.d3d11_blend_state_set(context);
    }
    unsafe { host::blend_state_set(context) }
}

/// The depth-stencil state and the stencil reference with it.
///
/// # Safety
/// As [`render_targets`].
pub unsafe fn depth_stencil_set(context: Context) -> (DepthStencil, u32) {
    if let Some(win) = sim() {
        return win.d3d11_depth_stencil_set(context);
    }
    unsafe { host::depth_stencil_set(context) }
}

/// # Safety
/// As [`render_targets`].
pub unsafe fn rasterizer_set(context: Context) -> Rasterizer {
    if let Some(win) = sim() {
        return win.d3d11_rasterizer_set(context);
    }
    unsafe { host::rasterizer_set(context) }
}

/// The viewports the context has set, however many it was given.
///
/// # Safety
/// As [`render_targets`].
pub unsafe fn viewports_set(context: Context, most: usize) -> Vec<Viewport> {
    if let Some(win) = sim() {
        return win.d3d11_viewports_set(context, most);
    }
    unsafe { host::viewports_set(context, most) }
}

/// # Safety
/// As [`render_targets`].
pub unsafe fn input_layout_set(context: Context) -> Layout {
    if let Some(win) = sim() {
        return win.d3d11_input_layout_set(context);
    }
    unsafe { host::input_layout_set(context) }
}

/// The vertex buffer in slot zero, with its stride and offset.
///
/// # Safety
/// As [`render_targets`].
pub unsafe fn vertex_buffer_set(context: Context) -> (Buffer, u32, u32) {
    if let Some(win) = sim() {
        return win.d3d11_vertex_buffer_set(context);
    }
    unsafe { host::vertex_buffer_set(context) }
}

/// # Safety
/// As [`render_targets`].
pub unsafe fn topology_set(context: Context) -> u32 {
    if let Some(win) = sim() {
        return win.d3d11_topology_set(context);
    }
    unsafe { host::topology_set(context) }
}

/// # Safety
/// As [`render_targets`].
pub unsafe fn vertex_shader_set(context: Context) -> VertexShader {
    if let Some(win) = sim() {
        return win.d3d11_vertex_shader_set(context);
    }
    unsafe { host::vertex_shader_set(context) }
}

/// # Safety
/// As [`render_targets`].
pub unsafe fn pixel_shader_set(context: Context) -> PixelShader {
    if let Some(win) = sim() {
        return win.d3d11_pixel_shader_set(context);
    }
    unsafe { host::pixel_shader_set(context) }
}

/// The texture a pixel shader reads in slot zero.
///
/// # Safety
/// As [`render_targets`].
pub unsafe fn resource_view_set(context: Context) -> ResourceView {
    if let Some(win) = sim() {
        return win.d3d11_resource_view_set(context);
    }
    unsafe { host::resource_view_set(context) }
}

/// And how it reads it, in slot zero.
///
/// # Safety
/// As [`render_targets`].
pub unsafe fn sampler_set(context: Context) -> Sampler {
    if let Some(win) = sim() {
        return win.d3d11_sampler_set(context);
    }
    unsafe { host::sampler_set(context) }
}

/// One reference given back, and what the object's count came to.
///
/// # Safety
/// The object must be live and this must be a reference the caller holds.
pub unsafe fn release(object: Object) -> u32 {
    if let Some(win) = sim() {
        return win.d3d11_release(object);
    }
    unsafe { host::release(object) }
}

#[cfg(windows)]
use crate::real::d3d11 as host;

#[cfg(not(windows))]
mod host {
    use super::{
        Blend, BlendDesc, Buffer, BufferDesc, Context, DepthStencil, DepthView, Device,
        InputElementDesc, Layout, Object, PixelShader, Rasterizer, ResourceView, Sampler,
        SamplerDesc, Target, Texture, Texture2dDesc, VertexShader, Viewport,
    };
    use crate::no_windows;

    pub unsafe fn device_of(_swapchain: usize) -> Option<Device> {
        no_windows("d3d11::device_of")
    }
    pub unsafe fn immediate_context(_device: Device) -> Option<Context> {
        no_windows("d3d11::immediate_context")
    }
    pub unsafe fn back_buffer(_swapchain: usize) -> Option<Texture> {
        no_windows("d3d11::back_buffer")
    }
    pub unsafe fn render_target(_device: Device, _texture: Texture) -> Option<Target> {
        no_windows("d3d11::render_target")
    }
    pub fn compile(_source: &str, _entry: &str, _profile: &str) -> Result<Vec<u8>, String> {
        no_windows("d3d11::compile")
    }
    pub unsafe fn vertex_shader(_device: Device, _bytecode: &[u8]) -> Option<VertexShader> {
        no_windows("d3d11::vertex_shader")
    }
    pub unsafe fn pixel_shader(_device: Device, _bytecode: &[u8]) -> Option<PixelShader> {
        no_windows("d3d11::pixel_shader")
    }
    pub unsafe fn input_layout(
        _device: Device,
        _elements: &[InputElementDesc],
        _bytecode: &[u8],
    ) -> Option<Layout> {
        no_windows("d3d11::input_layout")
    }
    pub unsafe fn buffer(
        _device: Device,
        _desc: BufferDesc,
        _bytes: Option<&[u8]>,
    ) -> Option<Buffer> {
        no_windows("d3d11::buffer")
    }
    pub unsafe fn texture(
        _device: Device,
        _desc: Texture2dDesc,
        _pixels: &[u32],
    ) -> Option<Texture> {
        no_windows("d3d11::texture")
    }
    pub unsafe fn resource_view(_device: Device, _texture: Texture) -> Option<ResourceView> {
        no_windows("d3d11::resource_view")
    }
    pub unsafe fn blend_state(_device: Device, _desc: BlendDesc) -> Option<Blend> {
        no_windows("d3d11::blend_state")
    }
    pub unsafe fn sampler_state(_device: Device, _desc: SamplerDesc) -> Option<Sampler> {
        no_windows("d3d11::sampler_state")
    }
    pub unsafe fn write(_context: Context, _buffer: Buffer, _bytes: &[u8]) -> bool {
        no_windows("d3d11::write")
    }
    pub unsafe fn set_render_targets(_context: Context, _targets: &[Target], _depth: DepthView) {
        no_windows("d3d11::set_render_targets")
    }
    pub unsafe fn set_viewports(_context: Context, _viewports: &[Viewport]) {
        no_windows("d3d11::set_viewports")
    }
    pub unsafe fn set_blend_state(_context: Context, _blend: Blend, _factor: [f32; 4], _mask: u32) {
        no_windows("d3d11::set_blend_state")
    }
    pub unsafe fn set_depth_stencil(_context: Context, _state: DepthStencil, _reference: u32) {
        no_windows("d3d11::set_depth_stencil")
    }
    pub unsafe fn set_rasterizer(_context: Context, _state: Rasterizer) {
        no_windows("d3d11::set_rasterizer")
    }
    pub unsafe fn render_targets(_context: Context, _most: usize) -> (Vec<Target>, DepthView) {
        no_windows("d3d11::render_targets")
    }
    pub unsafe fn blend_state_set(_context: Context) -> (Blend, [f32; 4], u32) {
        no_windows("d3d11::blend_state_set")
    }
    pub unsafe fn depth_stencil_set(_context: Context) -> (DepthStencil, u32) {
        no_windows("d3d11::depth_stencil_set")
    }
    pub unsafe fn rasterizer_set(_context: Context) -> Rasterizer {
        no_windows("d3d11::rasterizer_set")
    }
    pub unsafe fn viewports_set(_context: Context, _most: usize) -> Vec<Viewport> {
        no_windows("d3d11::viewports_set")
    }
    pub unsafe fn input_layout_set(_context: Context) -> Layout {
        no_windows("d3d11::input_layout_set")
    }
    pub unsafe fn vertex_buffer_set(_context: Context) -> (Buffer, u32, u32) {
        no_windows("d3d11::vertex_buffer_set")
    }
    pub unsafe fn topology_set(_context: Context) -> u32 {
        no_windows("d3d11::topology_set")
    }
    pub unsafe fn vertex_shader_set(_context: Context) -> VertexShader {
        no_windows("d3d11::vertex_shader_set")
    }
    pub unsafe fn pixel_shader_set(_context: Context) -> PixelShader {
        no_windows("d3d11::pixel_shader_set")
    }
    pub unsafe fn resource_view_set(_context: Context) -> ResourceView {
        no_windows("d3d11::resource_view_set")
    }
    pub unsafe fn sampler_set(_context: Context) -> Sampler {
        no_windows("d3d11::sampler_set")
    }
    pub unsafe fn set_input_layout(_context: Context, _layout: Layout) {
        no_windows("d3d11::set_input_layout")
    }
    pub unsafe fn set_vertex_buffer(
        _context: Context,
        _buffer: Buffer,
        _stride: u32,
        _offset: u32,
    ) {
        no_windows("d3d11::set_vertex_buffer")
    }
    pub unsafe fn set_topology(_context: Context, _topology: u32) {
        no_windows("d3d11::set_topology")
    }
    pub unsafe fn set_vertex_shader(_context: Context, _shader: VertexShader) {
        no_windows("d3d11::set_vertex_shader")
    }
    pub unsafe fn set_pixel_shader(_context: Context, _shader: PixelShader) {
        no_windows("d3d11::set_pixel_shader")
    }
    pub unsafe fn set_resource_view(_context: Context, _view: ResourceView) {
        no_windows("d3d11::set_resource_view")
    }
    pub unsafe fn set_sampler(_context: Context, _sampler: Sampler) {
        no_windows("d3d11::set_sampler")
    }
    pub unsafe fn draw(_context: Context, _vertices: u32, _first: u32) {
        no_windows("d3d11::draw")
    }
    pub unsafe fn release(_object: Object) -> u32 {
        no_windows("d3d11::release")
    }
}
