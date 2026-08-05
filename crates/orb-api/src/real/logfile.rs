//! The real file the log is appended to.
//!
//! Writes go straight to `WriteFile` rather than through `std::fs`, because the first lines are
//! emitted from `DllMain` while the loader lock is held.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CreateFileW, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ,
    FILE_SHARE_WRITE, GetFileAttributesExW, GetFileExInfoStandard, OPEN_ALWAYS,
    WIN32_FILE_ATTRIBUTE_DATA, WriteFile,
};

use crate::LogFile;

pub fn open(path: &Path, max_bytes: u64) -> Option<LogFile> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let disposition = if size_of_file(&wide) > max_bytes {
        CREATE_ALWAYS
    } else {
        OPEN_ALWAYS
    };
    let file = unsafe {
        CreateFileW(
            wide.as_ptr(),
            FILE_APPEND_DATA,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            disposition,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    (file != INVALID_HANDLE_VALUE).then_some(LogFile(file as usize))
}

fn size_of_file(path: &[u16]) -> u64 {
    let mut attributes: WIN32_FILE_ATTRIBUTE_DATA = unsafe { std::mem::zeroed() };
    let read = unsafe {
        GetFileAttributesExW(
            path.as_ptr(),
            GetFileExInfoStandard,
            (&raw mut attributes).cast(),
        )
    };
    if read == 0 {
        return 0;
    }
    u64::from(attributes.nFileSizeHigh) << 32 | u64::from(attributes.nFileSizeLow)
}

pub fn write(file: LogFile, bytes: &[u8]) {
    let mut written = 0u32;
    unsafe {
        WriteFile(
            file.0 as HANDLE,
            bytes.as_ptr(),
            bytes.len() as u32,
            &mut written,
            std::ptr::null_mut(),
        );
    }
}

pub fn close(file: LogFile) {
    unsafe { CloseHandle(file.0 as HANDLE) };
}
