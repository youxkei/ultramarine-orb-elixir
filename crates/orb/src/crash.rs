//! Turns a crash into log lines.
//!
//! `orb` writes into the game's memory and calls into its objects, so a mistake
//! shows up as the game vanishing. Naming the faulting address and the module it
//! belongs to is the difference between knowing which call was wrong and
//! guessing.
//!
//! **What is said about a fault is `orb_core::crash`'s**, which is what makes it
//! testable: this side is the two things only Windows can answer — the exception's
//! own record and context, and which module an address is in — and the lines are
//! written from those.

use orb_core::crash::{self, Fault, Registers, Touching};
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
    let Some(record) = record else {
        return EXCEPTION_CONTINUE_SEARCH;
    };
    let touching = if record.ExceptionCode == EXCEPTION_ACCESS_VIOLATION {
        let touching = match record.ExceptionInformation[0] {
            0 => Touching::Reading,
            1 => Touching::Writing,
            8 => Touching::Executing,
            other => return continue_after(&format!("access violation, operation {other}")),
        };
        Some((touching, record.ExceptionInformation[1]))
    } else {
        None
    };
    let fault = Fault {
        code: record.ExceptionCode,
        at: record.ExceptionAddress as usize,
        touching,
        // The registers as the fault left them, out of the context the filter is handed: the whole reason
        // this runs in the process rather than being read out of a dump afterwards.
        registers: unsafe { (*exception).ContextRecord.as_ref() }.map_or_else(
            Registers::default,
            |context| Registers {
                eax: context.Eax as usize,
                ebx: context.Ebx as usize,
                ecx: context.Ecx as usize,
                edx: context.Edx as usize,
                esi: context.Esi as usize,
                edi: context.Edi as usize,
                esp: context.Esp as usize,
                ebp: context.Ebp as usize,
            },
        ),
    };
    for line in crash::lines(&fault, &|address| {
        module_of(address).unwrap_or_else(|| "an unknown module".to_owned())
    }) {
        crate::log::line(&line);
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
    let path = orb_api::module::path(Some(module as usize))?;
    let name = path.file_name()?.to_string_lossy().into_owned();
    Some(format!("{name}+{:#x}", address - module as usize))
}
