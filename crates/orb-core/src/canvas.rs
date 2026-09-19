//! **Drawing one alpha-blended textured quad, through a game's own Direct3D 11.**
//!
//! What orb's overlay asks of a device is one thing: a quad with a colour per vertex and a texture, in the
//! game's own 640x480 output. `overlay::Frame`'s `fill`, `label`, `label_scaled`, `picture` and
//! `picture_in` are all `Frame::quad`, so a backend that can draw that can draw the whole of the overlay.
//! This is that backend for a game whose frames go through a DXGI swapchain — see `crate::dxgi`, which is
//! where the swapchain comes from, and `orb_api::d3d11`, which is the seam every call here goes through.
//!
//! **The decisions are here and not behind the seam**, which is what `orb_api::d3d11`'s own doc is about:
//! the shaders' source, the vertex layout, the blend mode, the topology and the viewport are all in this
//! file, because they are what an e2e test over the drawing is about.
//!
//! **Nothing here is th06c's.** What it needs told is a swapchain and how big the output is; which game
//! that is belongs to the `Game`.
//!
//! **And nothing here decides when it is made or what it draws**, both of which are
//! [`crate::overlay`]'s: an `Overlay` holds one of these where the game has no Direct3D 8 device, and
//! every quad reaching it is one of `Frame`'s.

use orb_api::d3d11::{self, Object};

use crate::log;

/// **The vertex the shaders below agree on**: where it goes in the output, in pixels, the colour it
/// carries, and where in the texture it reads from.
///
/// Pixels and not clip space, which is what the `POSITION` semantic would usually be: the overlay's
/// coordinates are the game's own 640x480 and the vertex shader is what turns them into clip space — see
/// [`SHADERS`]. So a quad's four corners are written the way `overlay::Frame::quad` already writes them.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    u: f32,
    v: f32,
    /// Straight, as the four floats a shader reads a colour as: orb's colours are one `0xAARRGGBB` word and
    /// turning one into floats is this file's own job rather than the shader's.
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

/// How many of them one quad is, as a triangle strip: the two triangles of a rectangle share an edge.
const CORNERS: usize = 4;

/// **The pair, compiled at runtime out of the game's own `d3dcompiler`** — see `orb_api::d3d11::compile`.
///
/// The vertex shader takes the output's size as two floats in a constant of its own... which it does not:
/// a constant buffer is a second object and a second binding, and the size is known when a vertex is
/// written. So the *pixels* are turned into clip space here, in this file's own arithmetic, and the shader
/// passes them through. What that costs is nothing and what it buys is one fewer object to make, bind and
/// release.
///
/// The pixel shader multiplies the texture by the colour, which is what the D3D8 overlay's modulate stage
/// does — so a solid fill is a one-pixel white texture and text is the font's own, exactly as there.
const SHADERS: &str = r#"
struct Vertex {
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
    float4 colour : COLOR0;
};

Vertex vertex_main(float2 position : POSITION, float2 uv : TEXCOORD0, float4 colour : COLOR0) {
    Vertex out_of_it;
    out_of_it.position = float4(position, 0.0, 1.0);
    out_of_it.uv = uv;
    out_of_it.colour = colour;
    return out_of_it;
}

Texture2D the_texture : register(t0);
SamplerState how_to_read_it : register(s0);

float4 pixel_main(Vertex it) : SV_TARGET {
    return the_texture.Sample(how_to_read_it, it.uv) * it.colour;
}
"#;

/// The semantics the layout names, as the NUL-terminated strings D3D11 wants. Held as constants because the
/// pointers handed to `input_layout` have to outlive the call and these outlive the process.
const POSITION: &[u8] = b"POSITION\0";
const TEXCOORD: &[u8] = b"TEXCOORD\0";
const COLOUR: &[u8] = b"COLOR\0";

/// Everything the drawing needs, made once for a swapchain and given back together.
pub struct Canvas {
    device: d3d11::Device,
    context: d3d11::Context,
    /// The swapchain itself, because the view below has to be made again from it every frame.
    swapchain: usize,
    /// **A view over whichever buffer the swapchain is presenting now**, remade at the top of each frame —
    /// see [`Canvas::takes_this_frames_buffer`], which has why one made once is a view of the wrong
    /// surface.
    ///
    /// A `Cell` because the remaking happens through a `&self`: `overlay::Overlay::frame` is what brackets
    /// a frame and it holds the overlay by reference, several of the runtime's other fields being borrowed
    /// beside it. What makes that sound is the thread: an `Overlay` lives in `runtime::RUNTIME`, which is a
    /// `MainThread`, so nothing else ever holds this.
    target: std::cell::Cell<d3d11::Target>,
    vertices: d3d11::Buffer,
    layout: d3d11::Layout,
    vertex_shader: d3d11::VertexShader,
    pixel_shader: d3d11::PixelShader,
    blend: d3d11::Blend,
    sampler: d3d11::Sampler,
    /// How big the output is, which is what a vertex's pixels are turned into clip space against.
    output: (f32, f32),
    /// **How big the surface that output is stretched onto is**, which is the viewport every frame is
    /// drawn through — the back buffer's own size, asked of the swapchain and refreshed with the buffer.
    ///
    /// **Two sizes and not one, because they are two different things**: [`Canvas::output`] is the
    /// coordinate space orb places everything in — `overlay`'s 640x480 — and this is the pixels that
    /// space is mapped onto. Setting the viewport to the coordinate space instead is what left orb's
    /// own menus drawn in the top-left corner of a `1280x720` window at a quarter of its area.
    viewport: std::cell::Cell<(f32, f32)>,
}

/// A texture and the view a shader reads it through, which are always made and released together.
pub struct Sheet {
    texture: d3d11::Texture,
    view: d3d11::ResourceView,
    /// How big it is, so that a caller drawing a piece of one can work out the coordinates.
    pub width: u32,
    pub height: u32,
}

impl Canvas {
    /// **Everything made, or nothing and a line saying what failed.** A launch whose drawer does not come
    /// up is a launch with no overlay, which is what th06c was before this: its questions are answered by a
    /// recipe or not at all.
    ///
    /// # Safety
    /// `swapchain` must be a live `IDXGISwapChain` of a D3D11 device, and this must run on the thread that
    /// made it.
    pub unsafe fn new(swapchain: usize, output: (u32, u32)) -> Option<Self> {
        let device = unsafe { d3d11::device_of(swapchain) }?;
        let context = unsafe { d3d11::immediate_context(device) }?;
        let target = unsafe { a_view_of_this_frames_buffer(device, swapchain) }?;

        let compiled = match d3d11::compile(SHADERS, "vertex_main", "vs_4_0") {
            Ok(bytecode) => bytecode,
            Err(why) => {
                log!("canvas: the vertex shader would not compile: {why}");
                return None;
            }
        };
        let vertex_shader = unsafe { d3d11::vertex_shader(device, &compiled) }?;
        let pixels = match d3d11::compile(SHADERS, "pixel_main", "ps_4_0") {
            Ok(bytecode) => bytecode,
            Err(why) => {
                log!("canvas: the pixel shader would not compile: {why}");
                return None;
            }
        };
        let pixel_shader = unsafe { d3d11::pixel_shader(device, &pixels) }?;

        // **The layout against the vertex shader's own signature**, which is what the compiled vertex
        // bytecode is handed for: a layout that does not match what the shader declares is refused here
        // rather than drawing wrong.
        let elements = [
            element(POSITION, d3d11::DXGI_FORMAT_R32G32_FLOAT, 0),
            element(TEXCOORD, d3d11::DXGI_FORMAT_R32G32_FLOAT, 8),
            element(COLOUR, d3d11::DXGI_FORMAT_R32G32B32A32_FLOAT, 16),
        ];
        let layout = unsafe { d3d11::input_layout(device, &elements, &compiled) }?;

        let vertices = unsafe {
            d3d11::buffer(
                device,
                d3d11::BufferDesc {
                    byte_width: (CORNERS * size_of::<Vertex>()) as u32,
                    usage: d3d11::D3D11_USAGE_DYNAMIC,
                    bind_flags: d3d11::D3D11_BIND_VERTEX_BUFFER,
                    cpu_access_flags: d3d11::D3D11_CPU_ACCESS_WRITE,
                    ..d3d11::BufferDesc::default()
                },
                None,
            )
        }?;

        // **Straight alpha over what is already there**, which is what an overlay is: the source's alpha
        // against one minus it, added, and the target's own alpha left at one so the swapchain stays opaque.
        let mut blending = d3d11::BlendDesc::default();
        blending.render_target[0] = d3d11::RenderTargetBlendDesc {
            blend_enable: 1,
            src_blend: d3d11::D3D11_BLEND_SRC_ALPHA,
            dest_blend: d3d11::D3D11_BLEND_INV_SRC_ALPHA,
            blend_op: d3d11::D3D11_BLEND_OP_ADD,
            src_blend_alpha: d3d11::D3D11_BLEND_ONE,
            dest_blend_alpha: d3d11::D3D11_BLEND_INV_SRC_ALPHA,
            blend_op_alpha: d3d11::D3D11_BLEND_OP_ADD,
            render_target_write_mask: d3d11::D3D11_COLOR_WRITE_ENABLE_ALL,
        };
        let blend = unsafe { d3d11::blend_state(device, blending) }?;

        let sampler = unsafe {
            d3d11::sampler_state(
                device,
                d3d11::SamplerDesc {
                    filter: d3d11::D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                    address_u: d3d11::D3D11_TEXTURE_ADDRESS_CLAMP,
                    address_v: d3d11::D3D11_TEXTURE_ADDRESS_CLAMP,
                    address_w: d3d11::D3D11_TEXTURE_ADDRESS_CLAMP,
                    comparison_func: d3d11::D3D11_COMPARISON_NEVER,
                    max_lod: d3d11::D3D11_FLOAT32_MAX,
                    ..d3d11::SamplerDesc::default()
                },
            )
        }?;

        // **Both sizes, because they are two things and a launch has to be able to tell them apart**:
        // the coordinate space everything is placed in, and the surface it is stretched onto. A line
        // that said only the first is a line that could not have shown the overlay drawn into the
        // corner of a larger window — see [`Canvas::viewport`].
        let viewport = unsafe { crate::dxgi::the_size_of(swapchain) };
        log!(
            "canvas: drawing through this game's own Direct3D 11, over {}x{} stretched onto {}",
            output.0,
            output.1,
            match viewport {
                Some((width, height)) => format!("{width}x{height}"),
                None => "a buffer of a size the swapchain will not say".to_owned(),
            },
        );
        Some(Self {
            device,
            context,
            swapchain,
            target: std::cell::Cell::new(target),
            vertices,
            layout,
            vertex_shader,
            pixel_shader,
            blend,
            sampler,
            output: (output.0 as f32, output.1 as f32),
            // **The back buffer's own size where the swapchain will say it, and the output's where it
            // will not**: a viewport of nought draws nothing at all, so the coordinate space is the
            // answer that at least puts the overlay on the screen.
            viewport: std::cell::Cell::new(
                viewport.map_or((output.0 as f32, output.1 as f32), |(width, height)| {
                    (width as f32, height as f32)
                }),
            ),
        })
    }

    /// **A texture of `pixels`, and the view a shader reads it through.** The pixels are `0xAARRGGBB` as
    /// `orb_api::text` bakes a line of text, and the format says so.
    ///
    /// # Safety
    /// Must run on the thread the device was made on, and `pixels` must hold `width * height` of them.
    pub unsafe fn sheet(&self, width: u32, height: u32, pixels: &[u32]) -> Option<Sheet> {
        let texture = unsafe {
            d3d11::texture(
                self.device,
                d3d11::Texture2dDesc {
                    width,
                    height,
                    mip_levels: 1,
                    array_size: 1,
                    format: d3d11::DXGI_FORMAT_B8G8R8A8_UNORM,
                    usage: d3d11::D3D11_USAGE_DEFAULT,
                    bind_flags: d3d11::D3D11_BIND_SHADER_RESOURCE,
                    ..d3d11::Texture2dDesc::default()
                },
                pixels,
            )
        }?;
        match unsafe { d3d11::resource_view(self.device, texture) } {
            Some(view) => Some(Sheet {
                texture,
                view,
                width,
                height,
            }),
            None => {
                unsafe { d3d11::release(Object::Texture(texture)) };
                None
            }
        }
    }

    /// **The view over whichever buffer this frame is being presented from**, taken at the top of a frame.
    ///
    /// **Because a swapchain made by `CreateSwapChainForHwnd` presents from a different buffer each
    /// frame.** That call's swap effect is one of the flip ones, where `GetBuffer(0)` answers the surface
    /// of *this* frame and the one it answered last frame is queued or being scanned out. So a view made
    /// once in [`Canvas::new`] is a view of a surface nothing is showing after the first frame, and
    /// everything drawn through it goes somewhere that never reaches the screen.
    ///
    /// **Both symptoms of this session came out of that one view.** With the state left unrestored, the
    /// game drew the rest of its own frame into that stale surface too and presented an empty one — the
    /// black window. With the state restored, the game's picture came back and orb's own quads were the
    /// only thing still going into the wrong surface: the log said the mode question was up and the
    /// screen showed the title with nothing over it.
    ///
    /// Nothing at all where the buffer or the view cannot be had, which leaves the frame drawing into the
    /// view it already has rather than into none.
    ///
    /// # Safety
    /// Must run on the thread the device was made on, between frames.
    pub unsafe fn takes_this_frames_buffer(&self) {
        let Some(target) = (unsafe { a_view_of_this_frames_buffer(self.device, self.swapchain) })
        else {
            return;
        };
        let was = self.target.replace(target);
        unsafe { d3d11::release(Object::Target(was)) };
        // **And the size that buffer is**, which a resized window changes: the viewport is what the
        // output's coordinates are stretched onto, so a stale one draws the overlay at the size the
        // window used to be. Left as it was where the swapchain will not say.
        if let Some((width, height)) = unsafe { crate::dxgi::the_size_of(self.swapchain) } {
            self.viewport.set((width as f32, height as f32));
        }
    }

    /// **The view over this frame's buffer given back, once the frame is drawn over** — the next frame
    /// takes its own in [`Canvas::takes_this_frames_buffer`].
    ///
    /// **Because a view held between frames is a reference to the swapchain's buffers**, and a swapchain
    /// whose buffers are referenced refuses `ResizeBuffers`: th06nc makes its buffers the window's size
    /// again from its own window procedure, between two frames, and a refusal there is a game that goes
    /// on drawing for the size it asked while its buffers stay the size they were.
    ///
    /// # Safety
    /// Must run on the thread the device was made on, after the frame's last quad.
    pub unsafe fn lets_go_of_this_frames_buffer(&self) {
        unsafe { d3d11::release(Object::Target(self.target.replace(d3d11::Target(0)))) };
    }

    /// **One quad, in the output's own pixels.** `uv` is the piece of the sheet to read, as
    /// `[u0, v0, u1, v1]`; `colour` is `0xAARRGGBB` and multiplies it.
    ///
    /// # Safety
    /// Must run on the thread the device was made on, inside the frame, with `sheet` a live one of this
    /// canvas's.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn quad(
        &self,
        sheet: &Sheet,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        colour: u32,
        uv: [f32; 4],
    ) {
        // The half-pixel shift the D3D8 overlay makes, for the same reason: a texel's centre lands on a
        // pixel's centre rather than between two.
        let (left, top) = (x - 0.5, y - 0.5);
        let (right, bottom) = (left + width, top + height);
        let [u0, v0, u1, v1] = uv;
        let (a, r, g, b) = (
            ((colour >> 24) & 0xff) as f32 / 255.0,
            ((colour >> 16) & 0xff) as f32 / 255.0,
            ((colour >> 8) & 0xff) as f32 / 255.0,
            (colour & 0xff) as f32 / 255.0,
        );
        // Pixels into clip space: across is twice the fraction less one, and down is the same negated,
        // clip space having its origin in the middle and its y going up.
        let across = |pixels: f32| pixels / self.output.0 * 2.0 - 1.0;
        let down = |pixels: f32| 1.0 - pixels / self.output.1 * 2.0;
        let corner = |x: f32, y: f32, u: f32, v: f32| Vertex {
            x: across(x),
            y: down(y),
            u,
            v,
            r,
            g,
            b,
            a,
        };
        let quad = [
            corner(left, top, u0, v0),
            corner(right, top, u1, v0),
            corner(left, bottom, u0, v1),
            corner(right, bottom, u1, v1),
        ];
        let bytes = unsafe {
            std::slice::from_raw_parts(quad.as_ptr().cast::<u8>(), std::mem::size_of_val(&quad))
        };
        if !unsafe { d3d11::write(self.context, self.vertices, bytes) } {
            return;
        }
        // **The game's own state read before any of orb's is set, and put back after the draw** — see
        // [`Saved`], and the black window that came of not doing it.
        let saved = unsafe { Saved::of(self.context) };
        unsafe {
            d3d11::set_render_targets(self.context, &[self.target.get()], d3d11::DepthView(0));
            // **The whole of the surface and not the coordinate space** — see [`Canvas::viewport`],
            // which is the back buffer's own size: the vertices are in [`Canvas::output`]'s pixels and
            // the viewport is what maps them onto whatever the window measures.
            let (width, height) = self.viewport.get();
            d3d11::set_viewports(
                self.context,
                &[d3d11::Viewport {
                    width,
                    height,
                    max_depth: 1.0,
                    ..d3d11::Viewport::default()
                }],
            );
            // The device's own defaults, which is what a nought state means: nothing orb draws is
            // depth-tested, scissored or culled.
            d3d11::set_rasterizer(self.context, d3d11::Rasterizer(0));
            d3d11::set_depth_stencil(self.context, d3d11::DepthStencil(0), 0);
            d3d11::set_blend_state(self.context, self.blend, [0.0; 4], u32::MAX);
            d3d11::set_input_layout(self.context, self.layout);
            d3d11::set_vertex_buffer(self.context, self.vertices, size_of::<Vertex>() as u32, 0);
            d3d11::set_topology(self.context, d3d11::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            d3d11::set_vertex_shader(self.context, self.vertex_shader);
            d3d11::set_pixel_shader(self.context, self.pixel_shader);
            d3d11::set_resource_view(self.context, sheet.view);
            d3d11::set_sampler(self.context, self.sampler);
            d3d11::draw(self.context, CORNERS as u32, 0);
        }
        unsafe { saved.put_back(self.context) };
    }
}

impl Sheet {
    /// Given back, which is the view and then the texture: the view holds a reference to it.
    ///
    /// # Safety
    /// Must run on the thread the device was made on, and this must be a reference the caller holds.
    pub unsafe fn release(self) {
        unsafe {
            d3d11::release(Object::ResourceView(self.view));
            d3d11::release(Object::Texture(self.texture));
        }
    }
}

/// **Everything it made, given back.**
///
/// The device and the context among them, because both were *asked for* rather than made: `GetDevice` and
/// `GetImmediateContext` each add a reference of their own, and a launch that dropped a canvas without
/// releasing them would leave the game's own device alive past its renderer.
///
/// **A drop and not a method, unlike [`Sheet::release`]**, because nothing but the end of the overlay ever
/// gives a canvas back — where a sheet is given back every time a `Label` is re-baked. What makes it safe
/// is where the overlay is dropped: the frame loop's own thread, which is the one the device was made on.
impl Drop for Canvas {
    fn drop(&mut self) {
        for object in [
            Object::Sampler(self.sampler),
            Object::Blend(self.blend),
            Object::PixelShader(self.pixel_shader),
            Object::VertexShader(self.vertex_shader),
            Object::Layout(self.layout),
            Object::Buffer(self.vertices),
            Object::Target(self.target.get()),
            Object::Context(self.context),
            Object::Device(self.device),
        ] {
            unsafe { d3d11::release(object) };
        }
    }
}

/// **The context state one of orb's quads changes, as the game had it.**
///
/// **Because Direct3D 11 has no state block.** The Direct3D 8 overlay brackets every frame with
/// `CreateStateBlock`, `CaptureStateBlock` and `ApplyStateBlock` — see `overlay::Overlay::frame` — and
/// there is no counterpart here, so what a draw changes has to be read first and written back after, piece
/// by piece.
///
/// **What it cost to leave out**: a launch of the real th06c with the overlay drawing showed a black
/// window, the run advancing behind it the whole time — the log's own frame counter climbing while the
/// screen held nothing. The comment that used to stand where this does claimed the game's renderer set its
/// state again on its next draw, which was never measured and is not true: this game binds its render
/// target and leaves it bound, so orb's own target stayed on it and the game drew the rest of its frame
/// somewhere it never composited from.
///
/// **Every field is one the drawing sets**, and nothing else: a save of state a draw does not touch is a
/// restore that could undo something the game did between two of orb's frames.
struct Saved {
    targets: Vec<d3d11::Target>,
    depth_view: d3d11::DepthView,
    blend: d3d11::Blend,
    blend_factor: [f32; 4],
    blend_mask: u32,
    depth_stencil: d3d11::DepthStencil,
    stencil_reference: u32,
    rasterizer: d3d11::Rasterizer,
    viewports: Vec<d3d11::Viewport>,
    layout: d3d11::Layout,
    vertices: d3d11::Buffer,
    stride: u32,
    offset: u32,
    topology: u32,
    vertex_shader: d3d11::VertexShader,
    pixel_shader: d3d11::PixelShader,
    resource_view: d3d11::ResourceView,
    sampler: d3d11::Sampler,
}

/// **How many render targets and viewports are read back.** Direct3D 11's own maxima:
/// `D3D11_SIMULTANEOUS_RENDER_TARGET_COUNT` is eight and
/// `D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE` sixteen. Asked for in full rather than for
/// the one orb sets, because what is being put back is the game's own however many it had.
const TARGETS: usize = 8;
const VIEWPORTS: usize = 16;

impl Saved {
    /// # Safety
    /// Must run on the context's own thread, before anything of orb's own is set.
    unsafe fn of(context: d3d11::Context) -> Self {
        let (targets, depth_view) = unsafe { d3d11::render_targets(context, TARGETS) };
        let (blend, blend_factor, blend_mask) = unsafe { d3d11::blend_state_set(context) };
        let (depth_stencil, stencil_reference) = unsafe { d3d11::depth_stencil_set(context) };
        let (vertices, stride, offset) = unsafe { d3d11::vertex_buffer_set(context) };
        Self {
            targets,
            depth_view,
            blend,
            blend_factor,
            blend_mask,
            depth_stencil,
            stencil_reference,
            rasterizer: unsafe { d3d11::rasterizer_set(context) },
            viewports: unsafe { d3d11::viewports_set(context, VIEWPORTS) },
            layout: unsafe { d3d11::input_layout_set(context) },
            vertices,
            stride,
            offset,
            topology: unsafe { d3d11::topology_set(context) },
            vertex_shader: unsafe { d3d11::vertex_shader_set(context) },
            pixel_shader: unsafe { d3d11::pixel_shader_set(context) },
            resource_view: unsafe { d3d11::resource_view_set(context) },
            sampler: unsafe { d3d11::sampler_set(context) },
        }
    }

    /// **Put back, and then the references these reads added given back.**
    ///
    /// The two in one call because they are one act: a restore that did not release would leak a dozen COM
    /// objects a frame, and a release without the restore would hand back objects the context still points
    /// at. Every `Get*` of Direct3D 11 adds a reference, which is the half of that API a state block used
    /// to hide.
    ///
    /// # Safety
    /// Must run on the context's own thread, with every object saved still live.
    unsafe fn put_back(self, context: d3d11::Context) {
        unsafe {
            d3d11::set_render_targets(context, &self.targets, self.depth_view);
            d3d11::set_blend_state(context, self.blend, self.blend_factor, self.blend_mask);
            d3d11::set_depth_stencil(context, self.depth_stencil, self.stencil_reference);
            d3d11::set_rasterizer(context, self.rasterizer);
            d3d11::set_viewports(context, &self.viewports);
            d3d11::set_input_layout(context, self.layout);
            d3d11::set_vertex_buffer(context, self.vertices, self.stride, self.offset);
            d3d11::set_topology(context, self.topology);
            d3d11::set_vertex_shader(context, self.vertex_shader);
            d3d11::set_pixel_shader(context, self.pixel_shader);
            d3d11::set_resource_view(context, self.resource_view);
            d3d11::set_sampler(context, self.sampler);
        }
        let mut objects: Vec<Object> = self.targets.iter().copied().map(Object::Target).collect();
        objects.extend([
            Object::DepthView(self.depth_view),
            Object::Blend(self.blend),
            Object::DepthStencil(self.depth_stencil),
            Object::Rasterizer(self.rasterizer),
            Object::Layout(self.layout),
            Object::Buffer(self.vertices),
            Object::VertexShader(self.vertex_shader),
            Object::PixelShader(self.pixel_shader),
            Object::ResourceView(self.resource_view),
            Object::Sampler(self.sampler),
        ]);
        for object in objects {
            // A nought is nothing the context had set, which every one of these reads answers for a slot
            // the game left empty — and a release of one would be a release of nothing.
            if object.at() != 0 {
                unsafe { d3d11::release(object) };
            }
        }
    }
}

/// **A render target view over the swapchain's current buffer.** The buffer is only wanted for the view,
/// which holds a reference of its own, so it is given back at once.
///
/// # Safety
/// Both must be live and this must run on the device's own thread.
unsafe fn a_view_of_this_frames_buffer(
    device: d3d11::Device,
    swapchain: usize,
) -> Option<d3d11::Target> {
    let buffer = unsafe { d3d11::back_buffer(swapchain) }?;
    let target = unsafe { d3d11::render_target(device, buffer) };
    unsafe { d3d11::release(Object::Texture(buffer)) };
    target
}

/// One row of the vertex layout: a semantic, the format its two or four floats are, and how far into a
/// vertex it starts.
fn element(semantic: &'static [u8], format: u32, at: u32) -> d3d11::InputElementDesc {
    d3d11::InputElementDesc {
        semantic_name: semantic.as_ptr(),
        semantic_index: 0,
        format,
        input_slot: 0,
        aligned_byte_offset: at,
        input_slot_class: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
        instance_data_step_rate: 0,
    }
}
