//! **One frame of a window, taken through Windows.Graphics.Capture and read back out of Direct3D 11.**
//!
//! The `windows-sys` this tree pins types no COM interface at all, and its copy carries no
//! `Direct3D11`, `Dxgi` or `WinRT` module either — so every interface and every entry point below is
//! declared here, the way `crate::d3d11` declares the Direct3D 11 vtables it calls. Only the slots this
//! file calls are typed, the rest is pointer-sized padding, and each typed slot's index is asserted
//! against its position here rather than counted by hand: a wrong index is a call into an unrelated
//! method with the wrong signature.
//!
//! **The frame pool is free-threaded rather than dispatcher-bound.** orb calls this from the game's own
//! thread, which runs no WinRT apartment's message loop, so an event-driven pool would never deliver
//! `FrameArrived` there. Polling `TryGetNextFrame` is the other half of the same decision, and it is
//! bounded: a wait with no end would hold up the frame orb is inside for as long as the game keeps
//! running.
//!
//! **A device of its own, and only for the length of the capture.** The game's device is reached through
//! a swapchain orb reads out of the game's memory, but a capture frame pool has to be handed an
//! `ID3D11Device` this process made; taking the game's would tie the capture to a device the game is
//! free to lose.

use std::ffi::c_void;
use std::mem::offset_of;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

use crate::d3d11::{
    ContextVtable, D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_USAGE_STAGING,
    DXGI_FORMAT_B8G8R8A8_UNORM, DeviceVtable, IID_ID3D11TEXTURE2D, MappedSubresource, SampleDesc,
    Texture2dDesc, UnknownVtable,
};
use crate::window::Picture;
use crate::{Hresult, Hwnd};

/// How long a capture waits for the compositor to hand over a frame. A capture of a window that is
/// minimised or on another desktop never gets one, and the caller has a frame to finish.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(2);
/// How long between two asks. The compositor composes on its own clock, so asking faster only spends
/// the thread.
const FRAME_POLL_INTERVAL: Duration = Duration::from_millis(10);
/// Two buffers, which is the least `CreateFreeThreaded` accepts.
const FRAME_POOL_BUFFERS: i32 = 2;

const RO_INIT_MULTITHREADED: i32 = 1;
/// `RPC_E_CHANGED_MODE`: the thread is already initialized in the other apartment, which is not a
/// reason to refuse a capture — only a reason not to uninitialize it afterwards.
const RPC_E_CHANGED_MODE: Hresult = 0x8001_0106u32 as Hresult;

const D3D_DRIVER_TYPE_HARDWARE: u32 = 1;
/// `D3D11_CREATE_DEVICE_BGRA_SUPPORT`. Windows.Graphics.Capture hands its frames over as
/// `B8G8R8A8`, and a device without this flag cannot make the texture the frame pool needs.
const D3D11_CREATE_DEVICE_BGRA_SUPPORT: u32 = 0x20;
/// `D3D11_SDK_VERSION`, the only value the call takes.
const D3D11_SDK_VERSION: u32 = 7;

/// `DirectXPixelFormat_B8G8R8A8UIntNormalized`, which is `DXGI_FORMAT_B8G8R8A8_UNORM`'s number.
const DIRECTX_PIXEL_FORMAT_B8G8R8A8_UNORM: i32 = 87;

const CLASS_GRAPHICS_CAPTURE_ITEM: &str = "Windows.Graphics.Capture.GraphicsCaptureItem";
const CLASS_FRAME_POOL: &str = "Windows.Graphics.Capture.Direct3D11CaptureFramePool";

// ---- the IIDs, as the sixteen bytes a GUID is ----
//
// The first three fields little-endian and the last eight in the order they are written, the way
// `crate::d3d11` spells the Direct3D ones.

/// `IGraphicsCaptureItemInterop`, `3628e81b-3cac-4c60-b7f4-23ce0e0c3356`.
const IID_IGRAPHICS_CAPTURE_ITEM_INTEROP: [u8; 16] = [
    0x1b, 0xe8, 0x28, 0x36, 0xac, 0x3c, 0x60, 0x4c, 0xb7, 0xf4, 0x23, 0xce, 0x0e, 0x0c, 0x33, 0x56,
];
/// `IGraphicsCaptureItem`, `79c3f95b-31f7-4ec2-a464-632ef5d30760`.
const IID_IGRAPHICS_CAPTURE_ITEM: [u8; 16] = [
    0x5b, 0xf9, 0xc3, 0x79, 0xf7, 0x31, 0xc2, 0x4e, 0xa4, 0x64, 0x63, 0x2e, 0xf5, 0xd3, 0x07, 0x60,
];
/// `IDirect3D11CaptureFramePoolStatics2`, `589b103f-6bbc-5df5-a991-02e28b3b66d5`.
const IID_IDIRECT3D11CAPTUREFRAMEPOOLSTATICS2: [u8; 16] = [
    0x3f, 0x10, 0x9b, 0x58, 0xbc, 0x6b, 0xf5, 0x5d, 0xa9, 0x91, 0x02, 0xe2, 0x8b, 0x3b, 0x66, 0xd5,
];
/// `IDXGIDevice`, `54ec77fa-1377-44e6-8c32-88fd5f44c84c`.
const IID_IDXGIDEVICE: [u8; 16] = [
    0xfa, 0x77, 0xec, 0x54, 0x77, 0x13, 0xe6, 0x44, 0x8c, 0x32, 0x88, 0xfd, 0x5f, 0x44, 0xc8, 0x4c,
];
/// `IDirect3DDevice`, `a37624ab-8d5f-4650-9d3e-9eae3d9bc670`.
const IID_IDIRECT3DDEVICE: [u8; 16] = [
    0xab, 0x24, 0x76, 0xa3, 0x5f, 0x8d, 0x50, 0x46, 0x9d, 0x3e, 0x9e, 0xae, 0x3d, 0x9b, 0xc6, 0x70,
];
/// `IDirect3DDxgiInterfaceAccess`, `a9b3d012-3df2-4ee3-b8d1-8695f457d3c1`.
const IID_IDIRECT3DDXGIINTERFACEACCESS: [u8; 16] = [
    0x12, 0xd0, 0xb3, 0xa9, 0xf2, 0x3d, 0xe3, 0x4e, 0xb8, 0xd1, 0x86, 0x95, 0xf4, 0x57, 0xd3, 0xc1,
];

// ---- the interfaces, slot for slot ----

/// `IGraphicsCaptureItemInterop`, which derives from `IUnknown` and not from `IInspectable` — its
/// `CreateForWindow` is the fourth slot, not the seventh.
#[repr(C)]
/// The three calls a frame's texture takes, the last of them the one `CopyResource` is held against — every
/// slot before `get_desc` walked past rather than left out, a COM vtable being addressed by position.
struct Texture2dVtable {
    _iunknown: [usize; 3],
    _device_child: [usize; 4],
    _resource: [usize; 3],
    get_desc: unsafe extern "system" fn(*mut c_void, *mut Texture2dDesc),
}
const _: () = assert!(offset_of!(Texture2dVtable, get_desc) == 10 * size_of::<usize>());

#[repr(C)]
struct CaptureItemInteropVtable {
    _iunknown: [usize; 3],
    create_for_window:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *const u8, *mut *mut c_void) -> Hresult,
}
const _: () =
    assert!(offset_of!(CaptureItemInteropVtable, create_for_window) == 3 * size_of::<usize>());

/// `IGraphicsCaptureItem`'s `Size`, which is the second method past `IInspectable`'s six.
#[repr(C)]
struct CaptureItemVtable {
    _iinspectable: [usize; 6],
    _display_name: usize,
    size: unsafe extern "system" fn(*mut c_void, *mut SizeInt32) -> Hresult,
}
const _: () = assert!(offset_of!(CaptureItemVtable, size) == 7 * size_of::<usize>());

/// `IDirect3D11CaptureFramePoolStatics2`'s `CreateFreeThreaded`, the only static this file asks for.
#[repr(C)]
struct FramePoolStatics2Vtable {
    _iinspectable: [usize; 6],
    create_free_threaded: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        i32,
        i32,
        SizeInt32,
        *mut *mut c_void,
    ) -> Hresult,
}
const _: () =
    assert!(offset_of!(FramePoolStatics2Vtable, create_free_threaded) == 6 * size_of::<usize>());

/// `IDirect3D11CaptureFramePool`: `TryGetNextFrame` is slot 7 and `CreateCaptureSession` slot 10.
#[repr(C)]
struct FramePoolVtable {
    _iinspectable: [usize; 6],
    _recreate: usize,
    try_get_next_frame: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> Hresult,
    _frame_arrived: usize,
    _remove_frame_arrived: usize,
    create_capture_session:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> Hresult,
}
const _: () = assert!(offset_of!(FramePoolVtable, try_get_next_frame) == 7 * size_of::<usize>());
const _: () =
    assert!(offset_of!(FramePoolVtable, create_capture_session) == 10 * size_of::<usize>());

/// `IGraphicsCaptureSession`'s `StartCapture`.
#[repr(C)]
struct SessionVtable {
    _iinspectable: [usize; 6],
    start_capture: unsafe extern "system" fn(*mut c_void) -> Hresult,
}
const _: () = assert!(offset_of!(SessionVtable, start_capture) == 6 * size_of::<usize>());

/// `IDirect3D11CaptureFrame`'s `Surface`, the first method past `IInspectable`.
#[repr(C)]
struct FrameVtable {
    _iinspectable: [usize; 6],
    surface: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> Hresult,
    _system_relative_time: usize,
    _content_size: usize,
}
const _: () = assert!(offset_of!(FrameVtable, surface) == 6 * size_of::<usize>());

/// `IDirect3DDxgiInterfaceAccess`, `IUnknown`'s three and then `GetInterface`.
#[repr(C)]
struct DxgiInterfaceAccessVtable {
    _iunknown: [usize; 3],
    get_interface: unsafe extern "system" fn(*mut c_void, *const u8, *mut *mut c_void) -> Hresult,
}
const _: () =
    assert!(offset_of!(DxgiInterfaceAccessVtable, get_interface) == 3 * size_of::<usize>());

/// `IUnknown` with `QueryInterface` typed. `crate::d3d11::UnknownVtable` leaves it a `usize` because
/// the drawing never casts, and this file casts both the device and the frame's surface.
#[repr(C)]
struct UnknownQueryVtable {
    query_interface: unsafe extern "system" fn(*mut c_void, *const u8, *mut *mut c_void) -> Hresult,
    _add_ref: usize,
    _release: usize,
}
const _: () = assert!(offset_of!(UnknownQueryVtable, query_interface) == 0);

/// `ID3D11DeviceContext`'s `CopyResource`, which `crate::d3d11::ContextVtable` does not type: slot 47,
/// straight after the `CopySubresourceRegion` that vtable stops at.
#[repr(C)]
struct ContextCopyVtable {
    _through_scissor_rects: [usize; 47],
    copy_resource: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void),
}
const _: () = assert!(offset_of!(ContextCopyVtable, copy_resource) == 47 * size_of::<usize>());

/// `Windows.Graphics.SizeInt32`.
#[repr(C)]
#[derive(Clone, Copy)]
struct SizeInt32 {
    width: i32,
    height: i32,
}

// ---- the calls the pinned `windows-sys` has no module for ----
//
// `#[link]` puts the import library on the linker's command line; the Microsoft toolchain finds both
// by name. The WinRT activation and string functions live in `combase.dll`, whose import library the
// Windows SDK ships only through the `WindowsApp` umbrella — there is no `combase.lib` to name — and
// the two Direct3D 11 entry points in `d3d11.dll`.

#[link(name = "d3d11")]
unsafe extern "system" {
    fn D3D11CreateDevice(
        adapter: *mut c_void,
        driver_type: u32,
        software: *mut c_void,
        flags: u32,
        feature_levels: *const u32,
        feature_levels_count: u32,
        sdk_version: u32,
        device: *mut *mut c_void,
        feature_level: *mut u32,
        context: *mut *mut c_void,
    ) -> Hresult;
    fn CreateDirect3D11DeviceFromDXGIDevice(
        dxgi_device: *mut c_void,
        graphics_device: *mut *mut c_void,
    ) -> Hresult;
}

#[link(name = "WindowsApp")]
unsafe extern "system" {
    fn RoInitialize(init_type: i32) -> Hresult;
    fn RoUninitialize();
    fn RoGetActivationFactory(
        class_id: *mut c_void,
        interface: *const u8,
        factory: *mut *mut c_void,
    ) -> Hresult;
    fn WindowsCreateString(source: *const u16, length: u32, string: *mut *mut c_void) -> Hresult;
    fn WindowsDeleteString(string: *mut c_void) -> Hresult;
}

// ---- ownership of what the calls hand back ----

/// One COM object this file holds a reference to, released when it goes out of scope.
///
/// Every interface here starts with `IUnknown`, so one guard serves the Direct3D and the WinRT objects
/// alike. Releasing by hand on each return is what this replaces: there are six early returns below,
/// and a reference leaked on any of them keeps a capture session alive past the frame it was made for.
struct Com(*mut c_void);

impl Com {
    const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    fn is_null(&self) -> bool {
        self.0.is_null()
    }

    fn ptr(&self) -> *mut c_void {
        self.0
    }

    fn out(&mut self) -> *mut *mut c_void {
        &mut self.0
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { release(self.0) };
        }
    }
}

/// A WinRT string, freed by `WindowsDeleteString` rather than `IUnknown::Release`: it is not a COM
/// object with a vtable, and handing one to `release` would read the characters as a pointer.
struct Hstring(*mut c_void);

impl Drop for Hstring {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { WindowsDeleteString(self.0) };
        }
    }
}

/// `RoInitialize` held for the length of one capture and undone on the way out when this thread is the
/// one that initialized it.
struct WinRt(bool);

impl WinRt {
    fn acquire() -> Option<Self> {
        let answered = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
        // `S_FALSE` and `RPC_E_CHANGED_MODE` both say the thread was initialized already, by something
        // whose uninitialization it is. Only `S_OK` is this capture's to undo.
        (answered >= 0 || answered == RPC_E_CHANGED_MODE).then_some(Self(answered == 0))
    }
}

impl Drop for WinRt {
    fn drop(&mut self) {
        if self.0 {
            unsafe { RoUninitialize() };
        }
    }
}

// ---- the capture ----

/// **Why the last capture answered nothing**, in the words of the step that gave up — see [`capture`].
///
/// The seam has no log of its own, so the reason is kept here for the caller above it to say: a capture
/// that fails is one line in a launch's own record instead of a picture nobody can account for.
static WHY: Mutex<&'static str> = Mutex::new("");

/// What the failing call answered, where it answered an `Hresult` — a refusal that says only which call it
/// was is half a reading, and `CreateForWindow`'s own codes are the half this needed.
static ANSWERED: AtomicI64 = AtomicI64::new(0);

pub fn why_the_last_capture_failed() -> String {
    let why = WHY.lock().map_or("nothing was said", |why| *why).to_owned();
    let answered = ANSWERED.load(Ordering::Relaxed);
    match answered {
        0 => why,
        answered => format!("{why} (answered {answered:#010x})"),
    }
}

/// Captures one frame of `window` and reads its pixels out of the surface as BGRA rows.
pub fn capture(window: Hwnd) -> Option<Picture> {
    match captured(window) {
        Ok(picture) => Some(picture),
        Err(why) => {
            if let Ok(mut last) = WHY.lock() {
                *last = why;
            }
            None
        }
    }
}

fn captured(window: Hwnd) -> Result<Picture, &'static str> {
    let _winrt = WinRt::acquire().ok_or("RoInitialize was refused")?;

    let mut device = Com::null();
    let mut context = Com::null();
    let answered = unsafe {
        D3D11CreateDevice(
            std::ptr::null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            std::ptr::null_mut(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            std::ptr::null(),
            0,
            D3D11_SDK_VERSION,
            device.out(),
            std::ptr::null_mut(),
            context.out(),
        )
    };
    if answered < 0 || device.is_null() || context.is_null() {
        return Err("the capture's own D3D11 device was not made");
    }

    let mut dxgi_device = Com::null();
    if !query_interface(&device, &IID_IDXGIDEVICE, &mut dxgi_device) {
        return Err("the capture's device has no IDXGIDevice");
    }
    let mut inspectable = Com::null();
    let answered =
        unsafe { CreateDirect3D11DeviceFromDXGIDevice(dxgi_device.ptr(), inspectable.out()) };
    if answered < 0 || inspectable.is_null() {
        return Err("the device's WinRT form was not made");
    }
    // The call answers an `IInspectable`; what the frame pool takes is the runtime class's own
    // `IDirect3DDevice`, so the cast is asked for the way COM asks for every other interface.
    let mut graphics_device = Com::null();
    if !query_interface(&inspectable, &IID_IDIRECT3DDEVICE, &mut graphics_device) {
        return Err("the WinRT device is no IDirect3DDevice");
    }

    let interop = activation_factory(
        CLASS_GRAPHICS_CAPTURE_ITEM,
        &IID_IGRAPHICS_CAPTURE_ITEM_INTEROP,
    )
    .ok_or("Windows.Graphics.Capture's item factory is not there")?;
    let mut item = Com::null();
    let answered = unsafe {
        (vtable::<CaptureItemInteropVtable>(interop.ptr()).create_for_window)(
            interop.ptr(),
            window.0 as *mut c_void,
            IID_IGRAPHICS_CAPTURE_ITEM.as_ptr(),
            item.out(),
        )
    };
    if answered < 0 || item.is_null() {
        ANSWERED.store(i64::from(answered), Ordering::Relaxed);
        return Err("the window is not one Windows.Graphics.Capture makes an item of");
    }

    let mut size = SizeInt32 {
        width: 0,
        height: 0,
    };
    let answered = unsafe { (vtable::<CaptureItemVtable>(item.ptr()).size)(item.ptr(), &mut size) };
    if answered < 0 || size.width <= 0 || size.height <= 0 {
        return Err("the window's item is of no size");
    }

    let statics = activation_factory(CLASS_FRAME_POOL, &IID_IDIRECT3D11CAPTUREFRAMEPOOLSTATICS2)
        .ok_or("Windows.Graphics.Capture's frame pool is not there")?;
    let mut pool = Com::null();
    let answered = unsafe {
        (vtable::<FramePoolStatics2Vtable>(statics.ptr()).create_free_threaded)(
            statics.ptr(),
            graphics_device.ptr(),
            DIRECTX_PIXEL_FORMAT_B8G8R8A8_UNORM,
            FRAME_POOL_BUFFERS,
            size,
            pool.out(),
        )
    };
    drop(statics);
    if answered < 0 || pool.is_null() {
        return Err("the frame pool would not be made for the window's size");
    }

    let mut session = Com::null();
    let answered = unsafe {
        (vtable::<FramePoolVtable>(pool.ptr()).create_capture_session)(
            pool.ptr(),
            item.ptr(),
            session.out(),
        )
    };
    if answered < 0 || session.is_null() {
        return Err("the capture session would not be made");
    }
    let answered = unsafe { (vtable::<SessionVtable>(session.ptr()).start_capture)(session.ptr()) };
    if answered < 0 {
        return Err("the capture session would not start");
    }

    let frame = wait_for_frame(&pool)?;

    let mut surface = Com::null();
    let answered =
        unsafe { (vtable::<FrameVtable>(frame.ptr()).surface)(frame.ptr(), surface.out()) };
    if answered < 0 || surface.is_null() {
        return Err("the frame has no surface");
    }

    let mut access = Com::null();
    if !query_interface(&surface, &IID_IDIRECT3DDXGIINTERFACEACCESS, &mut access) {
        return Err("the frame's surface has no DXGI interface access");
    }
    let mut texture = Com::null();
    let answered = unsafe {
        (vtable::<DxgiInterfaceAccessVtable>(access.ptr()).get_interface)(
            access.ptr(),
            IID_ID3D11TEXTURE2D.as_ptr(),
            texture.out(),
        )
    };
    if answered < 0 || texture.is_null() {
        return Err("the frame's surface is no ID3D11Texture2D");
    }

    read_picture(&device, &context, &texture, size)
}

/// Asks the pool for a frame until it has one or [`CAPTURE_TIMEOUT`] runs out.
fn wait_for_frame(pool: &Com) -> Result<Com, &'static str> {
    let started = Instant::now();
    loop {
        let mut frame = Com::null();
        let answered = unsafe {
            (vtable::<FramePoolVtable>(pool.ptr()).try_get_next_frame)(pool.ptr(), frame.out())
        };
        if answered >= 0 && !frame.is_null() {
            return Ok(frame);
        }
        if started.elapsed() >= CAPTURE_TIMEOUT {
            return Err("the compositor handed no frame over in two seconds");
        }
        std::thread::sleep(FRAME_POLL_INTERVAL);
    }
}

/// Copies `source` into a CPU-readable texture, maps it, and takes the rows out of the mapping.
fn read_picture(
    device: &Com,
    context: &Com,
    source: &Com,
    size: SizeInt32,
) -> Result<Picture, &'static str> {
    let width = size.width as u32;
    let height = size.height as u32;
    let description = Texture2dDesc {
        width,
        height,
        mip_levels: 1,
        array_size: 1,
        format: DXGI_FORMAT_B8G8R8A8_UNORM,
        sample_desc: SampleDesc::default(),
        usage: D3D11_USAGE_STAGING,
        bind_flags: 0,
        cpu_access_flags: D3D11_CPU_ACCESS_READ,
        misc_flags: 0,
    };
    let mut staging = Com::null();
    let answered = unsafe {
        (vtable::<DeviceVtable>(device.ptr()).create_texture_2d)(
            device.ptr(),
            &description,
            std::ptr::null(),
            staging.out(),
        )
    };
    if answered < 0 || staging.is_null() {
        return Err("the picture's staging texture was not made");
    }
    // **And nothing copied out of a frame the staging texture is not the shape of.** `CopyResource` answers
    // nothing at all, so a copy that cannot happen is a staging texture left uninitialised and read as if it
    // had been — a picture nobody drew. The two are held against each other here instead.
    let mut from = Texture2dDesc::default();
    unsafe { (vtable::<Texture2dVtable>(source.ptr()).get_desc)(source.ptr(), &mut from) };
    if (from.width, from.height, from.format)
        != (description.width, description.height, description.format)
    {
        return Err("the frame is not the size the pool was made for");
    }

    unsafe {
        (vtable::<ContextCopyVtable>(context.ptr()).copy_resource)(
            context.ptr(),
            staging.ptr(),
            source.ptr(),
        );
    }

    let mut mapped = MappedSubresource {
        data: std::ptr::null_mut(),
        row_pitch: 0,
        depth_pitch: 0,
    };
    let answered = unsafe {
        (vtable::<ContextVtable>(context.ptr()).map)(
            context.ptr(),
            staging.ptr(),
            0,
            D3D11_MAP_READ,
            0,
            &mut mapped,
        )
    };
    if answered < 0 || mapped.data.is_null() {
        return Err("the frame's staging texture would not be mapped");
    }
    let rows = unsafe { copy_rows(&mapped, width, height) };
    unsafe {
        (vtable::<ContextVtable>(context.ptr()).unmap)(context.ptr(), staging.ptr(), 0);
    }
    rows.map(|rows| Picture {
        width,
        height,
        rows,
        row_pitch: mapped.row_pitch,
    })
    .ok_or("the frame's rows are shorter than the picture is wide")
}

/// Copies `height` rows of `width` BGRA pixels out of a mapped staging texture into one tight block.
///
/// # Safety
/// `mapped` must be a read mapping of a texture `width` pixels wide and `height` high in the format the
/// description asked for, and the mapping must stay live for the call.
unsafe fn copy_rows(mapped: &MappedSubresource, width: u32, height: u32) -> Option<Vec<u8>> {
    let row_bytes = width as usize * 4;
    let pitch = mapped.row_pitch as usize;
    // A pitch shorter than the row would read past the end of each row into the next, which is a
    // texture this file did not ask for. Refusing is better than a picture that is sheared.
    if pitch < row_bytes {
        return None;
    }
    let mut rows = vec![0u8; row_bytes * height as usize];
    for y in 0..height as usize {
        let source = unsafe { (mapped.data as *const u8).add(y * pitch) };
        let target = &mut rows[y * row_bytes..(y + 1) * row_bytes];
        unsafe { std::ptr::copy_nonoverlapping(source, target.as_mut_ptr(), row_bytes) };
    }
    Some(rows)
}

// ---- the small calls the flow is written in ----

/// The vtable of a COM object at an address: the object's first word is the pointer to it.
///
/// # Safety
/// `object` must point at a live COM object whose vtable is laid out as `V` says.
unsafe fn vtable<V>(object: *mut c_void) -> &'static V {
    unsafe { &*(*(object as *const *const V)) }
}

/// `IUnknown::Release`.
///
/// # Safety
/// `object` must be a live COM object this side owns a reference to.
unsafe fn release(object: *mut c_void) {
    unsafe { (vtable::<UnknownVtable>(object).release)(object) };
}

/// `IUnknown::QueryInterface`, answered as whether the object has the interface.
fn query_interface(object: &Com, iid: &[u8; 16], into: &mut Com) -> bool {
    let answered = unsafe {
        (vtable::<UnknownQueryVtable>(object.ptr()).query_interface)(
            object.ptr(),
            iid.as_ptr(),
            into.out(),
        )
    };
    answered >= 0 && !into.is_null()
}

/// The runtime class name as the `HSTRING` `RoGetActivationFactory` takes.
fn hstring(class: &str) -> Option<Hstring> {
    let wide: Vec<u16> = class.encode_utf16().collect();
    let mut string = std::ptr::null_mut();
    let answered = unsafe { WindowsCreateString(wide.as_ptr(), wide.len() as u32, &mut string) };
    (answered >= 0).then_some(Hstring(string))
}

/// The activation factory `class` exposes `iid` of.
fn activation_factory(class: &str, iid: &[u8; 16]) -> Option<Com> {
    let string = hstring(class)?;
    let mut factory = Com::null();
    let answered = unsafe { RoGetActivationFactory(string.0, iid.as_ptr(), factory.out()) };
    (answered >= 0 && !factory.is_null()).then_some(factory)
}
