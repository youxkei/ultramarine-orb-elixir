//! The modules really loaded into this process.

use std::ffi::{CString, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleW, GetProcAddress,
};

pub fn path(module: Option<usize>) -> Option<PathBuf> {
    let mut buffer = [0u16; 1024];
    let length = unsafe {
        GetModuleFileNameW(
            module.unwrap_or(0) as HMODULE,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
    };
    if length == 0 || length as usize >= buffer.len() {
        return None;
    }
    Some(PathBuf::from(OsString::from_wide(
        &buffer[..length as usize],
    )))
}

pub fn loaded(module: &str) -> bool {
    !handle(module).is_null()
}

/// `GetModuleHandleW` rather than `LoadLibraryW`: what is wanted is the module the *game* already
/// has open, and loading a second copy would answer about the wrong one — and would take the
/// loader lock somewhere orb is not allowed to.
fn handle(module: &str) -> HMODULE {
    let wide: Vec<u16> = module.encode_utf16().chain([0]).collect();
    unsafe { GetModuleHandleW(wide.as_ptr()) }
}

pub fn proc_address(module: &str, name: &str) -> Option<usize> {
    let handle = handle(module);
    if handle.is_null() {
        return None;
    }
    let name = CString::new(name).ok()?;
    let address = unsafe { GetProcAddress(handle, name.as_ptr().cast()) };
    address.map(|address| address as usize)
}
