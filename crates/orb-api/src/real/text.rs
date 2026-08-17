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

/// The two weights orb asks for, which are Windows' own numbers: 400 and 700, the second being the one
/// 妖々夢's `CreateFontA` asks for as `0x2bc`.
const FW_NORMAL: i32 = 400;
const FW_BOLD: i32 = 700;
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

    // Negated here, because the em height is what this way of asking takes: a font loaded out of a file is
    // asked for at the size orb draws it, and the game whose file it is names no height of its own.
    let handle = made(-height, FACE_NAME);
    if handle.is_null() {
        unsafe { RemoveFontResourceExW(wide.as_ptr(), FR_PRIVATE, std::ptr::null()) };
        return None;
    }
    Some(Face(
        Box::into_raw(Box::new(Loaded { handle, path: wide })) as usize
    ))
}

/// A face with no font resource behind it, which is what a machine's own is: `family` by name where the
/// caller has one — the family the game itself asks GDI for — and nothing where it has not, in which case
/// GDI answers with whatever it maps the charset to.
///
/// The path it is remembered with is empty, which is what the drop reads as nothing to hand back — a
/// resource that was never added must not be removed, and removing one by an empty name would be a call
/// about somebody else's font.
pub fn machine_face(height: i32, family: &str, bold: bool) -> Option<Face> {
    let handle = made_weighing(height, family, bold);
    if handle.is_null() {
        return None;
    }
    Some(Face(Box::into_raw(Box::new(Loaded {
        handle,
        path: Vec::new(),
    })) as usize))
}

/// The face itself, out of the one `LOGFONTW` both ways of asking fill in: the height, the Shift-JIS
/// charset — this is a Japanese game and its text is Japanese — and a name, which is a family for the
/// game's own font and empty where GDI is to answer by charset alone.
fn made(height: i32, name: &str) -> HFONT {
    made_weighing(height, name, false)
}

/// And the same with the weight asked for, which is what matching a game that asks for a bold family means:
/// 妖々夢 asks for `ＭＳ ゴシック` at 700, and a face of it at 400 is the same glyphs in a lighter hand.
///
/// **`height` is GDI's own number and goes in as it comes**: negative for the em height, positive for the whole
/// cell. Which of the two a game asks for is the game's — see `Game::own_face` — so negating it here would be
/// this function deciding it.
fn made_weighing(height: i32, name: &str, bold: bool) -> HFONT {
    let mut description: windows_sys::Win32::Graphics::Gdi::LOGFONTW =
        unsafe { std::mem::zeroed() };
    description.lfHeight = height;
    description.lfWeight = if bold { FW_BOLD } else { FW_NORMAL };
    description.lfCharSet = SHIFTJIS_CHARSET;
    description.lfOutPrecision = OUT_TT_PRECIS;
    description.lfClipPrecision = CLIP_DEFAULT_PRECIS;
    description.lfQuality = ANTIALIASED_QUALITY;
    description.lfPitchAndFamily = DEFAULT_PITCH;
    for (slot, unit) in description.lfFaceName.iter_mut().zip(name.encode_utf16()) {
        *slot = unit;
    }
    unsafe { (gdi::text().create_font_indirect)(&description) }
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
    unsafe { DeleteObject(loaded.handle as _) };
    // The add taken back out, not only the handle deleted: GDI counts the adds, the
    // overlay makes two of them for the same file — one per size — and makes them
    // again every time the overlay is built.
    //
    // And nothing to take out for a face of the machine's, which had no add: its path is empty, and a
    // remove by an empty name would be a call about a font that is not orb's.
    if !loaded.path.is_empty() {
        unsafe { RemoveFontResourceExW(loaded.path.as_ptr(), FR_PRIVATE, std::ptr::null()) };
    }
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain([0]).collect()
}
