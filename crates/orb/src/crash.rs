//! Turns a crash into a log line.
//!
//! `orb` writes into the game's memory and calls into its objects, so a mistake
//! shows up as the game vanishing. Naming the faulting address and the module it
//! belongs to is the difference between knowing which call was wrong and
//! guessing.

use std::ffi::c_void;

use windows_sys::Win32::Foundation::{EXCEPTION_ACCESS_VIOLATION, HMODULE};
use windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleHandleExW,
};

/// Hands the crash on to whatever would have handled it, so the failure still
/// looks like a crash rather than being swallowed.
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;

type Filter = unsafe extern "system" fn(*mut EXCEPTION_POINTERS) -> i32;

// Not in windows-sys.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetUnhandledExceptionFilter(filter: Option<Filter>) -> Option<Filter>;
}

pub fn install() {
    unsafe { SetUnhandledExceptionFilter(Some(report)) };
}

unsafe extern "system" fn report(exception: *mut EXCEPTION_POINTERS) -> i32 {
    let record = unsafe { (*exception).ExceptionRecord.as_ref() };
    let Some(record) = record else { return EXCEPTION_CONTINUE_SEARCH };
    let address = record.ExceptionAddress as usize;
    crate::log::line(&format!(
        "crash: code {:#010x} at {address:#010x} in {}",
        record.ExceptionCode,
        module_of(address).unwrap_or_else(|| "an unknown module".to_owned()),
    ));
    if record.ExceptionCode == EXCEPTION_ACCESS_VIOLATION {
        let kind = match record.ExceptionInformation[0] {
            0 => "reading",
            1 => "writing",
            8 => "executing",
            other => return continue_after(&format!("access violation, operation {other}")),
        };
        crate::log::line(&format!(
            "crash: {kind} {:#010x}",
            record.ExceptionInformation[1]
        ));
    }
    crate::log::close();
    EXCEPTION_CONTINUE_SEARCH
}

fn continue_after(message: &str) -> i32 {
    crate::log::line(&format!("crash: {message}"));
    crate::log::close();
    EXCEPTION_CONTINUE_SEARCH
}

fn module_of(address: usize) -> Option<String> {
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
    let path = crate::log::module_path(module as *mut c_void)?;
    let name = path.file_name()?.to_string_lossy().into_owned();
    Some(format!("{name}+{:#x}", address - module as usize))
}
