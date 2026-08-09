//! The real machine's code page.

use windows_sys::Win32::Globalization::{CP_ACP, MultiByteToWideChar};

/// Measured first and then converted, rather than converted into a buffer of a size chosen here: how
/// many UTF-16 units a run of bytes comes to is the code page's business, and a Japanese one answers
/// fewer than there were bytes where a single-byte one answers exactly as many.
///
/// A zero-length ask is answered without the call, `MultiByteToWideChar` reporting a zero length as
/// the failure it reports every other failure as.
pub fn text(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let units = unsafe {
        MultiByteToWideChar(
            CP_ACP,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            std::ptr::null_mut(),
            0,
        )
    };
    if units <= 0 {
        // Not a code page's string at all, which is a device answering something winmm did not
        // promise. The bytes read as UTF-8 are then the best that can be said of them, and saying
        // nothing would lose which device it was.
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut wide = vec![0u16; units as usize];
    let written = unsafe {
        MultiByteToWideChar(
            CP_ACP,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            wide.as_mut_ptr(),
            units,
        )
    };
    String::from_utf16_lossy(&wide[..written.max(0) as usize])
}
