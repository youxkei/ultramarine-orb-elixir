//! The modules really loaded into this process.

use std::ffi::{CString, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW, GetModuleHandleW, GetProcAddress, LoadLibraryW,
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

/// The reference count is left alone, because a handle this hands back is used to name the module and
/// never to keep it loaded: taking a reference here would be one nothing releases.
pub fn holding(address: usize) -> Option<(PathBuf, usize)> {
    let mut module: HMODULE = std::ptr::null_mut();
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            address as *const u16,
            &mut module,
        )
    };
    if found == 0 || module.is_null() {
        return None;
    }
    Some((path(Some(module as usize))?, module as usize))
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

/// `LoadLibraryW`, which is the one place here that does load one — see `crate::module::brings_in` for
/// when that is the right act and when `handle` is.
///
/// The reference it takes is never given back: what is brought in this way is a library orb calls for the
/// life of the process, and a `FreeLibrary` would be orb unloading something the game may since have
/// opened for itself.
pub fn brings_in(module: &str) -> bool {
    if !handle(module).is_null() {
        return true;
    }
    let wide: Vec<u16> = module.encode_utf16().chain([0]).collect();
    !unsafe { LoadLibraryW(wide.as_ptr()) }.is_null()
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
