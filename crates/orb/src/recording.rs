//! A Direct3D 8 device that records what it is asked to draw instead of drawing it.
//!
//! No seam and no `#[cfg(test)]` branch anywhere in the drawing code, because none is needed: a
//! [`Device`] is a pointer to a vtable, so a vtable of Rust functions *is* a device as far as
//! everything that calls one is concerned. [`Overlay`](crate::overlay::Overlay) builds its state
//! block, uploads its textures and draws its quads through the same calls it makes against the
//! game's device, and they land here.
//!
//! What is kept is the request, not pixels — the quads with their rectangles and colours, in the
//! order they were drawn, and which texture each went through. Enough to say that the retry menu
//! put three items on the screen with the cursor on the second, or that the mark over the lives
//! covers the row and nothing beside it. A rasteriser would answer the same questions at the cost
//! of being one, and the one thing here that is genuinely about pixels — the brush stroke's
//! coverage — is baked by `build.rs` and tested there.

use std::ffi::c_void;
use std::sync::Mutex;

use crate::d3d8::{Device, DeviceVtable, Hresult, LockedRect, Texture, TextureVtable, Viewport};

/// One quad, as the vertices handed to `DrawPrimitiveUP` describe it.
///
/// The half-pixel shift the drawing applies is undone, so these are the coordinates the caller
/// asked for rather than the ones Direct3D wants.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: u32,
    /// Which texture was bound when it was drawn. Solid fills go through the overlay's one white
    /// texel, so this tells a filled rectangle from a glyph or a picture without holding either.
    pub texture: usize,
}

impl Quad {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Whether this quad covers the whole of `other`, which is how "the mark covers the row" is
    /// asked.
    pub fn covers(&self, other: &Quad) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }

    /// Whether the two overlap at all, for asking that something is *not* drawn over its neighbour.
    pub fn overlaps(&self, other: &Quad) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// What the device was asked to do, in order.
#[derive(Default)]
pub struct Drawn {
    pub quads: Vec<Quad>,
    /// Rectangles the device was asked to clear, with the colour, for the wash a chapter boundary
    /// puts over the play field.
    pub clears: Vec<u32>,
    pub scenes: u32,
    /// Viewports set, so that a frame drawn to the whole output rather than the play field is
    /// visible as such.
    pub viewports: Vec<Viewport>,
}

impl Drawn {
    /// The quads that went through a texture other than the white texel, which are the ones with a
    /// picture or a glyph in them.
    pub fn pictured(&self) -> Vec<Quad> {
        let white = self.quads.first().map(|quad| quad.texture);
        self.quads
            .iter()
            .copied()
            .filter(|quad| Some(quad.texture) != white)
            .collect()
    }
}

/// The vertex `Overlay` writes. Declared again here rather than shared, because what this has to
/// agree with is the bytes on the wire: a change to the drawing's own struct that this did not
/// follow is a change this should stop decoding.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    rhw: f32,
    color: u32,
    u: f32,
    v: f32,
}

/// A device and the textures it has handed out, with a record of everything asked of it.
///
/// Pinned in place by the `Box`es: the game is given raw pointers to these, so they must not move
/// while it holds them.
pub struct Recording {
    device: Box<DeviceHead>,
    /// Declared last so it is dropped last: everything above is torn down while this thread still
    /// holds the recording, and only then is the next test let in. Two values — a recording and a
    /// guard beside it — got this wrong, because a guard dropped first leaves the next test clearing
    /// the textures this one is still releasing.
    _held: std::sync::MutexGuard<'static, ()>,
}

/// A `Device` with its vtable behind it, so one allocation holds both and the pointer the game gets
/// stays valid for as long as this does.
#[repr(C)]
struct DeviceHead {
    device: Device,
    vtable: DeviceVtable,
}

#[repr(C)]
struct TextureHead {
    texture: Texture,
    vtable: TextureVtable,
    /// Storage `LockRect` hands out. As wide as the widest texture asked for, so that a lock of any
    /// of them has somewhere to write.
    pixels: Vec<u32>,
    width: u32,
}

/// Where the calls are recorded. A static rather than something reached through the device, because
/// the vtable's functions are plain `extern "system"` and have nowhere to carry a context: what they
/// are handed is the device pointer the game holds, and threading anything else through would mean
/// changing the ABI they exist to match.
///
/// One test at a time, then — which is what [`Recording::new`] takes the lock for.
static DRAWN: Mutex<Option<Drawn>> = Mutex::new(None);

/// Held for as long as a test is recording, so two tests cannot write each other's drawing.
static RECORDING: Mutex<()> = Mutex::new(());

fn record(with: impl FnOnce(&mut Drawn)) {
    if let Ok(mut drawn) = DRAWN.lock()
        && let Some(drawn) = drawn.as_mut()
    {
        with(drawn);
    }
}

unsafe extern "system" fn create_texture(
    _device: *mut Device,
    width: u32,
    height: u32,
    _levels: u32,
    _usage: u32,
    _format: u32,
    _pool: u32,
    out: *mut *mut Texture,
) -> Hresult {
    let mut head = Box::new(TextureHead {
        texture: Texture {
            vtable: std::ptr::null(),
        },
        vtable: TextureVtable {
            _query_interface: 0,
            _add_ref: 0,
            release: texture_release,
            _slot_3_to_15: [0; 13],
            lock_rect,
            unlock_rect,
        },
        pixels: vec![0; (width * height.max(1)) as usize],
        width,
    });
    head.texture.vtable = &raw const head.vtable;
    unsafe { *out = &raw mut head.texture };
    if let Ok(mut textures) = TEXTURES.lock() {
        textures.0.push(head);
    }
    0
}

/// The textures handed out, kept alive for as long as the recording is. Separate from
/// [`Recording`] for the same reason [`DRAWN`] is: `create_texture` has only the ABI's arguments.
static TEXTURES: Mutex<Textures> = Mutex::new(Textures(Vec::new()));

// The box is not redundant, whatever `vec_box` says about a `Vec` already being on the heap: each
// head holds a pointer to its own vtable and has handed another one out, so it must not move. A
// `Vec<TextureHead>` moves every element it holds the moment it grows, which would leave the
// overlay calling through a vtable pointer into freed memory.
#[allow(clippy::vec_box)]
struct Textures(Vec<Box<TextureHead>>);

// The pointer in each head is to that head's own vtable, so it travels with the box and is as valid
// on one thread as another; the pixels behind it are the box's too. Only one recording runs at a
// time, which is what `RECORDING` holds.
unsafe impl Send for Textures {}

unsafe extern "system" fn texture_release(_texture: *mut Texture) -> u32 {
    // Dropped with the recording rather than here: a release that freed the box would leave the
    // overlay's own pointer dangling if it released twice, and what is being tested is not the
    // reference counting.
    0
}

unsafe extern "system" fn lock_rect(
    texture: *mut Texture,
    _level: u32,
    locked: *mut LockedRect,
    _rect: *const c_void,
    _flags: u32,
) -> Hresult {
    let head = texture as *mut TextureHead;
    unsafe {
        (*locked).pitch = ((*head).width * size_of::<u32>() as u32) as i32;
        (*locked).bits = (*head).pixels.as_mut_ptr() as *mut c_void;
    }
    0
}

unsafe extern "system" fn unlock_rect(_texture: *mut Texture, _level: u32) -> Hresult {
    0
}

unsafe extern "system" fn begin_scene(_device: *mut Device) -> Hresult {
    record(|drawn| drawn.scenes += 1);
    0
}

unsafe extern "system" fn end_scene(_device: *mut Device) -> Hresult {
    0
}

unsafe extern "system" fn clear(
    _device: *mut Device,
    _count: u32,
    _rects: *const c_void,
    _flags: u32,
    color: u32,
    _z: f32,
    _stencil: u32,
) -> Hresult {
    record(|drawn| drawn.clears.push(color));
    0
}

unsafe extern "system" fn set_viewport(_device: *mut Device, viewport: *const Viewport) -> Hresult {
    let viewport = unsafe { *viewport };
    record(|drawn| drawn.viewports.push(viewport));
    0
}

unsafe extern "system" fn get_viewport(_device: *mut Device, viewport: *mut Viewport) -> Hresult {
    // The game's own, which the overlay puts back when its frame ends.
    unsafe {
        *viewport = Viewport {
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            min_z: 0.0,
            max_z: 1.0,
        }
    };
    0
}

unsafe extern "system" fn set_render_state(
    _device: *mut Device,
    _state: u32,
    _value: u32,
) -> Hresult {
    0
}

unsafe extern "system" fn state_block(_device: *mut Device, _token: u32) -> Hresult {
    0
}

unsafe extern "system" fn create_state_block(
    _device: *mut Device,
    _kind: u32,
    token: *mut u32,
) -> Hresult {
    unsafe { *token = 1 };
    0
}

/// The texture bound to stage 0, which the next quad is drawn with.
static BOUND: Mutex<usize> = Mutex::new(0);

unsafe extern "system" fn set_texture(
    _device: *mut Device,
    _stage: u32,
    texture: *mut Texture,
) -> Hresult {
    if let Ok(mut bound) = BOUND.lock() {
        *bound = texture as usize;
    }
    0
}

unsafe extern "system" fn set_texture_stage_state(
    _device: *mut Device,
    _stage: u32,
    _kind: u32,
    _value: u32,
) -> Hresult {
    0
}

unsafe extern "system" fn draw_primitive_up(
    _device: *mut Device,
    _kind: u32,
    count: u32,
    vertices: *const c_void,
    stride: u32,
) -> Hresult {
    if stride as usize != size_of::<Vertex>() || count != 2 {
        // Not the two-triangle strip the overlay draws. Said rather than decoded as one, because a
        // silent misread here would be a test asserting about quads that were never drawn.
        return 0;
    }
    let vertices = unsafe { std::slice::from_raw_parts(vertices as *const Vertex, 4) };
    let texture = BOUND.lock().map(|bound| *bound).unwrap_or(0);
    // The drawing shifts by half a pixel so texel centres land on pixel centres; undone here so
    // the record is what the caller asked for.
    let quad = Quad {
        x: vertices[0].x + 0.5,
        y: vertices[0].y + 0.5,
        width: vertices[3].x - vertices[0].x,
        height: vertices[3].y - vertices[0].y,
        color: vertices[0].color,
        texture,
    };
    record(|drawn| drawn.quads.push(quad));
    0
}

unsafe extern "system" fn set_vertex_shader(_device: *mut Device, _shader: u32) -> Hresult {
    0
}

impl Recording {
    /// A device with nothing recorded yet, which no other test can be recording against at the same
    /// time.
    ///
    /// Waits for any other recording to be done with, since the record is a static and there is one
    /// of it.
    pub fn new() -> Self {
        let held = RECORDING.lock().unwrap_or_else(|held| held.into_inner());
        *DRAWN.lock().unwrap() = Some(Drawn::default());
        TEXTURES.lock().unwrap().0.clear();
        *BOUND.lock().unwrap() = 0;

        let mut device = Box::new(DeviceHead {
            device: Device {
                vtable: std::ptr::null(),
            },
            vtable: DeviceVtable {
                _iunknown: [0; 3],
                _slot_3_to_19: [0; 17],
                create_texture,
                _slot_21_to_33: [0; 13],
                begin_scene,
                end_scene,
                clear,
                _slot_37_to_39: [0; 3],
                set_viewport,
                get_viewport,
                _slot_42_to_49: [0; 8],
                set_render_state,
                _slot_51_to_53: [0; 3],
                apply_state_block: state_block,
                capture_state_block: state_block,
                delete_state_block: state_block,
                create_state_block,
                _slot_58_to_60: [0; 3],
                set_texture,
                _slot_62: [0; 1],
                set_texture_stage_state,
                _slot_64_to_71: [0; 8],
                draw_primitive_up,
                _slot_73_to_75: [0; 3],
                set_vertex_shader,
            },
        });
        device.device.vtable = &raw const device.vtable;
        Self {
            device,
            _held: held,
        }
    }

    /// The pointer to hand whatever is being asked to draw.
    pub fn device(&self) -> *mut Device {
        &raw const self.device.device as *mut Device
    }

    /// What has been asked of it since the last [`clear`](Self::clear).
    pub fn drawn(&self) -> Drawn {
        let mut held = DRAWN.lock().unwrap();
        let drawn = held.as_mut().expect("a recording is running");
        Drawn {
            quads: drawn.quads.clone(),
            clears: drawn.clears.clone(),
            scenes: drawn.scenes,
            viewports: drawn.viewports.clone(),
        }
    }

    /// Forgets what has been drawn, so that what a test asserts about is one frame rather than the
    /// textures the overlay uploaded while it was being built.
    pub fn clear(&self) {
        *DRAWN.lock().unwrap() = Some(Drawn::default());
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        *DRAWN.lock().unwrap() = None;
        TEXTURES.lock().unwrap().0.clear();
    }
}

/// An overlay drawing onto a recording device: what a test needs to say what is on the screen.
///
/// Here rather than in each test module because it is three traps in one place, and each of them
/// cost a session to find:
///
/// - **The overlay must be torn down before the device.** Its own drop calls `DeleteStateBlock` and
///   releases the texture it fills solid quads with, so a device freed first leaves those calls
///   going through a vtable that is gone. Fields drop in declaration order, which is why the
///   overlay is declared first.
/// - **One device per test, and every frame of that test on it.** A `Label` keeps the texture it
///   baked and hands it to the next frame that asks for the same text, so a second device would be
///   drawing the first one's textures.
/// - **The font is Windows' own.** The game's `font.ttf` is a file beside a game no test has, and
///   `AddFontResourceExW` takes any path — `Font::load` already survives GDI substituting a face.
///   What it costs is the glyph metrics, so a test may ask where the drawing put something and not
///   how wide it came out.
pub struct Screen {
    overlay: crate::overlay::Overlay,
    recording: Recording,
}

impl Screen {
    /// # Panics
    /// If Windows has no Arial, which it has had since 3.1.
    pub fn new() -> Self {
        let recording = Recording::new();
        let overlay =
            unsafe { crate::overlay::Overlay::new(recording.device(), &Self::font(), 12, 25) }
                .expect("an overlay on a font every Windows has");
        // The textures the overlay uploaded building itself are not a frame.
        recording.clear();
        Self { overlay, recording }
    }

    fn font() -> std::path::PathBuf {
        let mut buffer = vec![0u16; 260];
        let written = unsafe {
            windows_sys::Win32::System::SystemInformation::GetWindowsDirectoryW(
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            )
        };
        assert!(written > 0, "no Windows directory");
        buffer.truncate(written as usize);
        std::path::PathBuf::from(String::from_utf16_lossy(&buffer))
            .join("Fonts")
            .join("arial.ttf")
    }

    /// Draws one frame and answers everything it was asked for.
    pub fn drawn(&self, with: impl FnOnce(&crate::overlay::Overlay)) -> Drawn {
        with(&self.overlay);
        let drawn = self.recording.drawn();
        self.recording.clear();
        drawn
    }

    /// The quads of one frame, which is what most of these ask about.
    pub fn frame(&self, with: impl FnOnce(&crate::overlay::Overlay)) -> Vec<Quad> {
        self.drawn(with).quads
    }
}

#[cfg(test)]
mod tests {
    use super::{Quad, Recording};
    use crate::d3d8::{D3DFMT_A8R8G8B8, D3DPOOL_MANAGED, D3DPT_TRIANGLESTRIP, D3DSBT_ALL};
    use std::ffi::c_void;

    /// The vertex the overlay writes, written again here as the overlay writes it: four corners of
    /// a strip, the colour in each, and the half-pixel shift.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vertex {
        x: f32,
        y: f32,
        z: f32,
        rhw: f32,
        color: u32,
        u: f32,
        v: f32,
    }

    fn strip(x: f32, y: f32, width: f32, height: f32, color: u32) -> [Vertex; 4] {
        let (left, top) = (x - 0.5, y - 0.5);
        let (right, bottom) = (left + width, top + height);
        let corner = |x: f32, y: f32| Vertex {
            x,
            y,
            z: 0.0,
            rhw: 1.0,
            color,
            u: 0.0,
            v: 0.0,
        };
        [
            corner(left, top),
            corner(right, top),
            corner(left, bottom),
            corner(right, bottom),
        ]
    }

    /// What the recording is for: a quad drawn through the device comes back as the rectangle and
    /// the colour that were asked for, with the half-pixel shift undone.
    #[test]
    fn a_quad_drawn_comes_back_as_the_rectangle_that_was_asked_for() {
        let recording = Recording::new();
        let device = recording.device();
        let vtable = unsafe { &*(*device).vtable };

        let vertices = strip(496.0, 122.0, 144.0, 16.0, 0xff00_ff00);
        unsafe {
            (vtable.draw_primitive_up)(
                device,
                D3DPT_TRIANGLESTRIP,
                2,
                vertices.as_ptr() as *const c_void,
                size_of::<Vertex>() as u32,
            )
        };

        let drawn = recording.drawn();
        assert_eq!(drawn.quads.len(), 1);
        let quad = drawn.quads[0];
        assert_eq!((quad.x, quad.y), (496.0, 122.0));
        assert_eq!((quad.width, quad.height), (144.0, 16.0));
        assert_eq!(quad.color, 0xff00_ff00);
    }

    /// Which texture a quad went through is kept, because that is what tells a solid fill from a
    /// glyph: the overlay binds its one white texel for the first and a real texture for the second.
    #[test]
    fn a_quad_remembers_which_texture_it_was_drawn_with() {
        let recording = Recording::new();
        let device = recording.device();
        let vtable = unsafe { &*(*device).vtable };

        let mut white = std::ptr::null_mut();
        let mut picture = std::ptr::null_mut();
        unsafe {
            (vtable.create_texture)(
                device,
                1,
                1,
                1,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &mut white,
            );
            (vtable.create_texture)(
                device,
                64,
                32,
                1,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &mut picture,
            );
        }
        assert!(!white.is_null() && !picture.is_null());
        assert_ne!(white as usize, picture as usize);

        let vertices = strip(0.0, 0.0, 8.0, 8.0, 0xffff_ffff);
        let draw = |texture| unsafe {
            (vtable.set_texture)(device, 0, texture);
            (vtable.draw_primitive_up)(
                device,
                D3DPT_TRIANGLESTRIP,
                2,
                vertices.as_ptr() as *const c_void,
                size_of::<Vertex>() as u32,
            );
        };
        draw(white);
        draw(picture);

        let drawn = recording.drawn();
        assert_eq!(drawn.quads.len(), 2);
        assert_eq!(drawn.quads[0].texture, white as usize);
        // The white one is drawn first, which is how `pictured` tells the rest from it.
        assert_eq!(drawn.pictured().len(), 1);
        assert_eq!(drawn.pictured()[0].texture, picture as usize);
    }

    /// A texture the drawing locks has somewhere to write, and the pitch it is told is the pitch it
    /// gets: an upload that walked off the end of a row would be writing over the row below.
    #[test]
    fn a_locked_texture_hands_out_a_row_of_its_own_width() {
        let recording = Recording::new();
        let device = recording.device();
        let vtable = unsafe { &*(*device).vtable };

        let mut texture = std::ptr::null_mut();
        unsafe {
            (vtable.create_texture)(
                device,
                64,
                32,
                1,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &mut texture,
            )
        };
        let mut locked = crate::d3d8::LockedRect {
            pitch: 0,
            bits: std::ptr::null_mut(),
        };
        let result = unsafe {
            ((*(*texture).vtable).lock_rect)(texture, 0, &mut locked, std::ptr::null(), 0)
        };
        assert_eq!(result, 0);
        assert_eq!(locked.pitch, 64 * 4);
        assert!(!locked.bits.is_null());
        unsafe { ((*(*texture).vtable).unlock_rect)(texture, 0) };
    }

    /// The state block the overlay takes out and puts back, and the scene it draws inside. Neither
    /// does anything here; what a test needs of them is that they answer, since the overlay gives up
    /// on a device whose `CreateStateBlock` fails.
    #[test]
    fn a_state_block_and_a_scene_are_answered() {
        let recording = Recording::new();
        let device = recording.device();
        let vtable = unsafe { &*(*device).vtable };

        let mut token = 0;
        assert_eq!(
            unsafe { (vtable.create_state_block)(device, D3DSBT_ALL, &mut token) },
            0,
        );
        assert_ne!(token, 0, "a block the overlay can apply and delete");
        unsafe {
            (vtable.begin_scene)(device);
            (vtable.end_scene)(device);
        }
        assert_eq!(recording.drawn().scenes, 1);
    }

    /// A quad covers the row it is meant to and misses the rows either side, which is the shape of
    /// every assertion the mark over the lives wants to make.
    #[test]
    fn a_quad_says_what_it_covers_and_what_it_misses() {
        let row = Quad {
            x: 496.0,
            y: 122.0,
            width: 144.0,
            height: 16.0,
            color: 0,
            texture: 0,
        };
        let over = Quad {
            x: 496.0,
            y: 106.0,
            width: 144.0,
            height: 40.0,
            ..row
        };
        let bombs = Quad {
            x: 496.0,
            y: 146.0,
            width: 144.0,
            height: 16.0,
            ..row
        };
        assert!(over.covers(&row));
        assert!(!row.covers(&over));
        assert!(over.overlaps(&row));
        assert!(!over.overlaps(&bombs), "the row below is left alone");
    }
}
