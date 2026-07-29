//! Glyph rasterisation through GDI.
//!
//! Rendering with the game's own bundled `font.ttf`, loaded process-private so
//! nothing is installed system-wide and Japanese text works without depending on
//! what fonts the machine happens to have.
//!
//! The result is a coverage mask: white pixels with the antialiased glyph shape
//! in the alpha channel. The colour is applied by the vertex colour at draw
//! time, so a label costs one bake no matter how many colours it is drawn in.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Graphics::Gdi::{
    ANTIALIASED_QUALITY, AddFontResourceExW, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLIP_DEFAULT_PRECIS,
    CreateCompatibleDC, CreateDIBSection, CreateFontIndirectW, DEFAULT_PITCH, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, FR_PRIVATE, GetTextExtentPoint32W, GetTextFaceW, HFONT, OPAQUE,
    OUT_TT_PRECIS, SHIFTJIS_CHARSET, SelectObject, SetBkColor, SetBkMode, SetTextColor, TextOutW,
};

use crate::log::log;

/// The bundled font's family name, as GDI reports it. Verified after loading, so
/// a different `font.ttf` shows up in the log rather than silently substituting
/// whatever font Windows picks instead.
const FACE_NAME: &str = "Rounded M+ 2p regular";

const FW_NORMAL: i32 = 400;
const BLACK: u32 = 0x0000_0000;
const WHITE: u32 = 0x00ff_ffff;

pub struct Font {
    handle: HFONT,
}

/// A glyph coverage mask: `0x00ffffff` with coverage in the alpha channel.
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Font {
    /// `path` is the `.ttf` to load; `height` is the em height in pixels of the
    /// game's 640x480 output.
    pub fn load(path: &Path, height: i32) -> Option<Self> {
        let wide = wide(path);
        if unsafe { AddFontResourceExW(wide.as_ptr(), FR_PRIVATE, std::ptr::null()) } == 0 {
            log!("overlay: cannot load {}", path.display());
            return None;
        }

        let mut description: windows_sys::Win32::Graphics::Gdi::LOGFONTW =
            unsafe { std::mem::zeroed() };
        description.lfHeight = -height;
        description.lfWeight = FW_NORMAL;
        description.lfCharSet = SHIFTJIS_CHARSET as u8;
        description.lfOutPrecision = OUT_TT_PRECIS as u8;
        description.lfClipPrecision = CLIP_DEFAULT_PRECIS as u8;
        description.lfQuality = ANTIALIASED_QUALITY as u8;
        description.lfPitchAndFamily = DEFAULT_PITCH as u8;
        for (slot, unit) in description.lfFaceName.iter_mut().zip(FACE_NAME.encode_utf16()) {
            *slot = unit;
        }

        let handle = unsafe { CreateFontIndirectW(&description) };
        if handle.is_null() {
            log!("overlay: cannot create a font");
            return None;
        }
        let font = Self { handle };
        log!("overlay: font.ttf loaded, GDI is using {:?}", font.face_name().as_deref());
        Some(font)
    }

    /// What GDI actually selected, which is `FACE_NAME` only if the font loaded.
    fn face_name(&self) -> Option<String> {
        let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        if dc.is_null() {
            return None;
        }
        let previous = unsafe { SelectObject(dc, self.handle as _) };
        let mut buffer = [0u16; 64];
        let length =
            unsafe { GetTextFaceW(dc, buffer.len() as i32, buffer.as_mut_ptr()) };
        unsafe {
            SelectObject(dc, previous);
            DeleteDC(dc);
        }
        (length > 0).then(|| String::from_utf16_lossy(&buffer[..length as usize - 1]))
    }

    pub fn render(&self, text: &str) -> Option<Mask> {
        if text.is_empty() {
            return None;
        }
        let wide: Vec<u16> = text.encode_utf16().collect();
        let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        if dc.is_null() {
            return None;
        }
        let previous_font = unsafe { SelectObject(dc, self.handle as _) };

        let mut extent = unsafe { std::mem::zeroed() };
        let measured = unsafe {
            GetTextExtentPoint32W(dc, wide.as_ptr(), wide.len() as i32, &mut extent)
        };
        // Antialiasing and italic overhang can reach a pixel past the extent.
        let width = if measured == 0 { 0 } else { extent.cx.max(0) as u32 + 2 };
        let height = if measured == 0 { 0 } else { extent.cy.max(0) as u32 + 2 };
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
            CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0)
        };
        let mask = if bitmap.is_null() || bits.is_null() {
            None
        } else {
            let previous_bitmap = unsafe { SelectObject(dc, bitmap as _) };
            unsafe {
                SetBkMode(dc, OPAQUE as i32);
                SetBkColor(dc, BLACK);
                SetTextColor(dc, WHITE);
                TextOutW(dc, 1, 1, wide.as_ptr(), wide.len() as i32);
            }
            let rendered =
                unsafe { std::slice::from_raw_parts(bits as *const u32, (width * height) as usize) };
            // White text on black, so any channel is the coverage; keep the
            // brightest so a subpixel-ish edge does not lose weight.
            let pixels = rendered
                .iter()
                .map(|pixel| {
                    let coverage =
                        (pixel >> 16 & 0xff).max(pixel >> 8 & 0xff).max(pixel & 0xff);
                    coverage << 24 | WHITE
                })
                .collect();
            unsafe { SelectObject(dc, previous_bitmap) };
            Some(Mask { width, height, pixels })
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
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.handle as _) };
    }
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain([0]).collect()
}
