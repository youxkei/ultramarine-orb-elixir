//! A picture of each chapter's first frame.
//!
//! Whether a chapter starts in a sensible place is a judgement about what is on
//! screen, so the tuning pass leaves behind an image per chapter to look at
//! instead of asking anyone to watch a run and remember.
//!
//! Taken from the game's back buffer rather than the screen: it is the game's own
//! 640x480 output, with no letterbox, no overlay and nothing else's windows in it.

use std::path::Path;

use crate::d3d8::{
    D3DBACKBUFFER_TYPE_MONO, D3DFMT_A8R8G8B8, D3DFMT_R5G6B5, D3DFMT_X8R8G8B8, D3DLOCK_READONLY,
    Device, LockedRect, Surface, SurfaceDesc,
};
use crate::log::log;

/// Writes the back buffer to `path` as a BMP.
///
/// # Safety
/// Must run on the device's thread and outside a scene the game is drawing, which
/// after its update chain has finished is the case.
pub unsafe fn capture(device: *mut Device, path: &Path) {
    if device.is_null() {
        return;
    }
    let mut surface = std::ptr::null_mut();
    let got = unsafe {
        ((*(*device).vtable).get_back_buffer)(device, 0, D3DBACKBUFFER_TYPE_MONO, &mut surface)
    };
    if got < 0 || surface.is_null() {
        return log!("screenshot: no back buffer ({got:#x})");
    }
    let image = unsafe { read(surface) };
    unsafe { ((*(*surface).vtable).release)(surface) };

    let Some((width, height, pixels)) = image else { return };
    if let Err(error) = write_bmp(path, width, height, &pixels) {
        log!("screenshot: cannot write {}: {error}", path.display());
    }
}

/// Reads the surface as rows of BGR triples, top row first.
unsafe fn read(surface: *mut Surface) -> Option<(u32, u32, Vec<u8>)> {
    let vtable = unsafe { &*(*surface).vtable };
    let mut desc: SurfaceDesc = unsafe { std::mem::zeroed() };
    if unsafe { (vtable.get_desc)(surface, &mut desc) } < 0 {
        return None;
    }
    let mut locked = LockedRect { pitch: 0, bits: std::ptr::null_mut() };
    if unsafe { (vtable.lock_rect)(surface, &mut locked, std::ptr::null(), D3DLOCK_READONLY) } < 0
        || locked.bits.is_null()
    {
        log!("screenshot: cannot lock the back buffer");
        return None;
    }

    let mut pixels = Vec::with_capacity((desc.width * desc.height * 3) as usize);
    for row in 0..desc.height {
        let start = unsafe { locked.bits.byte_add(row as usize * locked.pitch as usize) };
        for column in 0..desc.width {
            let (blue, green, red) = match desc.format {
                D3DFMT_X8R8G8B8 | D3DFMT_A8R8G8B8 => {
                    let pixel = unsafe { *(start as *const u32).add(column as usize) };
                    ((pixel & 0xff) as u8, (pixel >> 8 & 0xff) as u8, (pixel >> 16 & 0xff) as u8)
                }
                D3DFMT_R5G6B5 => {
                    let pixel = unsafe { *(start as *const u16).add(column as usize) };
                    // Widened so that full 5- and 6-bit values reach 255.
                    let five = |value: u16| ((value * 255 + 15) / 31) as u8;
                    let six = |value: u16| ((value * 255 + 31) / 63) as u8;
                    (five(pixel & 0x1f), six(pixel >> 5 & 0x3f), five(pixel >> 11 & 0x1f))
                }
                other => {
                    unsafe { (vtable.unlock_rect)(surface) };
                    log!("screenshot: back buffer format {other} is not one orb can read");
                    return None;
                }
            };
            pixels.extend_from_slice(&[blue, green, red]);
        }
    }
    unsafe { (vtable.unlock_rect)(surface) };
    Some((desc.width, desc.height, pixels))
}

/// A 24-bit BMP, which needs no compression and opens anywhere.
fn write_bmp(path: &Path, width: u32, height: u32, pixels: &[u8]) -> std::io::Result<()> {
    let stride = (width * 3).next_multiple_of(4) as usize;
    let padding = stride - (width * 3) as usize;
    let data = stride * height as usize;
    const HEADER: u32 = 14 + 40;

    let mut file = Vec::with_capacity(HEADER as usize + data);
    file.extend_from_slice(b"BM");
    file.extend_from_slice(&(HEADER + data as u32).to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&HEADER.to_le_bytes());
    file.extend_from_slice(&40u32.to_le_bytes());
    file.extend_from_slice(&(width as i32).to_le_bytes());
    file.extend_from_slice(&(height as i32).to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&24u16.to_le_bytes());
    file.extend_from_slice(&[0; 24]);

    // BMP rows run bottom-up.
    for row in (0..height as usize).rev() {
        let start = row * (width * 3) as usize;
        file.extend_from_slice(&pixels[start..start + (width * 3) as usize]);
        file.extend(std::iter::repeat_n(0u8, padding));
    }
    std::fs::write(path, &file)
}
