//! Glyph rasterisation through GDI.
//!
//! Rendering with the game's own bundled `font.ttf`, loaded process-private so
//! nothing is installed system-wide and Japanese text works without depending on
//! what fonts the machine happens to have.
//!
//! The result is a coverage mask: white pixels with the antialiased glyph shape
//! in the alpha channel. The colour is applied by the vertex colour at draw
//! time, so a label costs one bake no matter how many colours it is drawn in.
//!
//! **Nothing here says anything.** A failure comes back as `None` and the caller writes the line,
//! because the log is `orb-core`'s and this is under it — which is also why the face name is asked
//! for through the seam rather than logged here: what a substituted face means is the drawing's
//! business.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Graphics::Gdi::{
    ANTIALIASED_QUALITY, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLIP_DEFAULT_PRECIS,
    CreateCompatibleDC, CreateDIBSection, DEFAULT_PITCH, DIB_RGB_COLORS, DeleteDC, DeleteObject,
    FR_PRIVATE, HFONT, OPAQUE, OUT_TT_PRECIS, RemoveFontResourceExW, SHIFTJIS_CHARSET,
    SelectObject, SetBkColor, SetBkMode, SetTextColor,
};

use crate::real::gdi;
use crate::{Face, Mask};

const FW_NORMAL: i32 = 400;
const BLACK: u32 = 0x0000_0000;
const WHITE: u32 = 0x00ff_ffff;

/// The bundled font's family name, as it is asked for. What GDI really selected is
/// [`face_name`]'s answer, and a run says so in its log — a different `font.ttf` shows up there
/// rather than silently substituting whatever font Windows picks instead.
const FACE_NAME: &str = "Rounded M+ 2p regular";

/// What a [`Face`] is a pointer to: the face GDI made, and the path the add was made with so that
/// the drop can hand the same string back.
///
/// Boxed and leaked rather than kept in a table keyed by the handle: what has to be found again is
/// the pair, and a `Box` is the pair with nothing to keep in step.
struct Loaded {
    handle: HFONT,
    path: Vec<u16>,
}

pub fn load_face(path: &Path, height: i32) -> Option<Face> {
    let wide = wide(path);
    let added =
        unsafe { (gdi::text().add_font_resource)(wide.as_ptr(), FR_PRIVATE, std::ptr::null()) };
    if added == 0 {
        return None;
    }

    let mut description: windows_sys::Win32::Graphics::Gdi::LOGFONTW =
        unsafe { std::mem::zeroed() };
    description.lfHeight = -height;
    description.lfWeight = FW_NORMAL;
    description.lfCharSet = SHIFTJIS_CHARSET;
    description.lfOutPrecision = OUT_TT_PRECIS;
    description.lfClipPrecision = CLIP_DEFAULT_PRECIS;
    description.lfQuality = ANTIALIASED_QUALITY;
    description.lfPitchAndFamily = DEFAULT_PITCH;
    for (slot, unit) in description
        .lfFaceName
        .iter_mut()
        .zip(FACE_NAME.encode_utf16())
    {
        *slot = unit;
    }

    let handle = unsafe { (gdi::text().create_font_indirect)(&description) };
    if handle.is_null() {
        unsafe { RemoveFontResourceExW(wide.as_ptr(), FR_PRIVATE, std::ptr::null()) };
        return None;
    }
    Some(Face(
        Box::into_raw(Box::new(Loaded { handle, path: wide })) as usize
    ))
}

pub fn face_name(face: Face) -> Option<String> {
    let loaded = unsafe { &*(face.0 as *const Loaded) };
    let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    if dc.is_null() {
        return None;
    }
    let previous = unsafe { SelectObject(dc, loaded.handle as _) };
    let mut buffer = [0u16; 64];
    let length = unsafe { (gdi::text().text_face)(dc, buffer.len() as i32, buffer.as_mut_ptr()) };
    unsafe {
        SelectObject(dc, previous);
        DeleteDC(dc);
    }
    (length > 0).then(|| String::from_utf16_lossy(&buffer[..length as usize - 1]))
}

pub fn bake(face: Face, text: &str) -> Option<Mask> {
    if text.is_empty() {
        return None;
    }
    let loaded = unsafe { &*(face.0 as *const Loaded) };
    let wide: Vec<u16> = text.encode_utf16().collect();
    let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    if dc.is_null() {
        return None;
    }
    let previous_font = unsafe { SelectObject(dc, loaded.handle as _) };

    let mut extent = unsafe { std::mem::zeroed() };
    let measured =
        unsafe { (gdi::text().text_extent)(dc, wide.as_ptr(), wide.len() as i32, &mut extent) };
    // Antialiasing and italic overhang can reach a pixel past the extent.
    let width = if measured == 0 {
        0
    } else {
        extent.cx.max(0) as u32 + 2
    };
    let height = if measured == 0 {
        0
    } else {
        extent.cy.max(0) as u32 + 2
    };
    if width == 0 || height == 0 {
        unsafe {
            SelectObject(dc, previous_font);
            DeleteDC(dc);
        }
        return None;
    }

    let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        // Negative: rows top-down, matching how the texture is filled.
        biHeight: -(height as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        ..unsafe { std::mem::zeroed() }
    };
    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let bitmap = unsafe {
        CreateDIBSection(
            dc,
            &info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        )
    };
    let mask = if bitmap.is_null() || bits.is_null() {
        None
    } else {
        let previous_bitmap = unsafe { SelectObject(dc, bitmap as _) };
        unsafe {
            SetBkMode(dc, OPAQUE as i32);
            SetBkColor(dc, BLACK);
            SetTextColor(dc, WHITE);
            (gdi::text().text_out)(dc, 1, 1, wide.as_ptr(), wide.len() as i32);
        }
        let rendered =
            unsafe { std::slice::from_raw_parts(bits as *const u32, (width * height) as usize) };
        // White text on black, so any channel is the coverage; keep the
        // brightest so a subpixel-ish edge does not lose weight.
        let pixels = rendered
            .iter()
            .map(|pixel| {
                let coverage = (pixel >> 16 & 0xff)
                    .max(pixel >> 8 & 0xff)
                    .max(pixel & 0xff);
                coverage << 24 | WHITE
            })
            .collect();
        unsafe { SelectObject(dc, previous_bitmap) };
        Some(Mask {
            width,
            height,
            pixels,
        })
    };

    unsafe {
        if !bitmap.is_null() {
            DeleteObject(bitmap as _);
        }
        SelectObject(dc, previous_font);
        DeleteDC(dc);
    }
    mask
}

pub fn drop_face(face: Face) {
    let loaded = unsafe { Box::from_raw(face.0 as *mut Loaded) };
    unsafe {
        DeleteObject(loaded.handle as _);
        // The add taken back out, not only the handle deleted: GDI counts the adds, the
        // overlay makes two of them for the same file — one per size — and makes them
        // again every time the overlay is built.
        RemoveFontResourceExW(loaded.path.as_ptr(), FR_PRIVATE, std::ptr::null());
    }
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain([0]).collect()
}
