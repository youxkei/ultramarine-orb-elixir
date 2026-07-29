//! Trampoline hooks over the game's code.
//!
//! Only functions whose first instructions are position-independent can be
//! hooked this way, so the expected bytes are passed in and checked: vpatch
//! also patches this exe, and a silent mismatch would mean relocating an
//! instruction that cannot be relocated.

use std::ffi::c_void;
use std::fmt;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
    VirtualAlloc, VirtualProtect,
};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

const JMP_REL32: u8 = 0xe9;
const JMP_LENGTH: usize = 5;
const NOP: u8 = 0x90;

#[derive(Debug)]
pub enum Error {
    PrologueTooShort,
    UnexpectedPrologue { expected: Vec<u8>, found: Vec<u8> },
    ImportNotFound { dll: String, function: String },
    OutOfMemory,
    Protect,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrologueTooShort => write!(f, "prologue is shorter than a jmp rel32"),
            Self::UnexpectedPrologue { expected, found } => {
                write!(f, "expected prologue {}, found {}", hex(expected), hex(found))
            }
            Self::ImportNotFound { dll, function } => write!(f, "{dll} does not import {function}"),
            Self::OutOfMemory => write!(f, "cannot allocate a trampoline"),
            Self::Protect => write!(f, "cannot make the target writable"),
        }
    }
}

/// Redirects `target` to `hook` and returns the address to call for the
/// original behaviour.
///
/// # Safety
/// `target` must point at the given `prologue`, and every one of those bytes
/// must be safe to execute from a different address. Callers must not let the
/// game execute `target` while this runs; during `DllMain` of a suspended
/// process that is automatic.
pub unsafe fn install(target: usize, prologue: &[u8], hook: usize) -> Result<usize, Error> {
    if prologue.len() < JMP_LENGTH {
        return Err(Error::PrologueTooShort);
    }
    let found = unsafe { std::slice::from_raw_parts(target as *const u8, prologue.len()) };
    if found != prologue {
        return Err(Error::UnexpectedPrologue {
            expected: prologue.to_vec(),
            found: found.to_vec(),
        });
    }

    let trampoline = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            prologue.len() + JMP_LENGTH,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };
    if trampoline.is_null() {
        return Err(Error::OutOfMemory);
    }
    let trampoline = trampoline as usize;
    unsafe {
        std::ptr::copy_nonoverlapping(prologue.as_ptr(), trampoline as *mut u8, prologue.len());
        write_jmp(trampoline + prologue.len(), target + prologue.len());
    }

    unsafe {
        let mut previous: PAGE_PROTECTION_FLAGS = 0;
        if VirtualProtect(target as *const c_void, prologue.len(), PAGE_EXECUTE_READWRITE, &mut previous)
            == FALSE
        {
            return Err(Error::Protect);
        }
        write_jmp(target, hook);
        for offset in JMP_LENGTH..prologue.len() {
            crate::mem::write(target + offset, NOP);
        }
        VirtualProtect(target as *const c_void, prologue.len(), previous, &mut previous);
        FlushInstructionCache(GetCurrentProcess(), target as *const c_void, prologue.len());
    }
    Ok(trampoline)
}

/// Points `module`'s import of `dll!function` at `replacement` and returns the
/// address that was there.
///
/// Redirecting the import table rather than the API itself keeps the game's
/// calls separate from d3d8's and dsound's, which reach the same functions
/// through their own imports.
///
/// # Safety
/// `module` must be a module mapped into this process, and `replacement` must
/// have the imported function's exact signature and calling convention.
pub unsafe fn install_import(
    module: usize,
    dll: &str,
    function: &str,
    replacement: usize,
) -> Result<usize, Error> {
    unsafe {
        let slot = crate::pe::import_slot(module, dll, function)
            .ok_or_else(|| Error::ImportNotFound { dll: dll.to_owned(), function: function.to_owned() })?;
        let mut previous: PAGE_PROTECTION_FLAGS = 0;
        if VirtualProtect(slot as *const c_void, size_of::<usize>(), PAGE_READWRITE, &mut previous)
            == FALSE
        {
            return Err(Error::Protect);
        }
        let original = crate::mem::read::<usize>(slot);
        crate::mem::write(slot, replacement);
        VirtualProtect(slot as *const c_void, size_of::<usize>(), previous, &mut previous);
        Ok(original)
    }
}

/// Swaps a function pointer, for vtables and other tables of them, and returns
/// what was there.
///
/// # Safety
/// `slot` must hold a function pointer, and `replacement` must have that
/// function's exact signature and calling convention.
pub unsafe fn replace_pointer(slot: usize, replacement: usize) -> Result<usize, Error> {
    unsafe {
        let mut previous: PAGE_PROTECTION_FLAGS = 0;
        if VirtualProtect(slot as *const c_void, size_of::<usize>(), PAGE_READWRITE, &mut previous)
            == FALSE
        {
            return Err(Error::Protect);
        }
        let original = crate::mem::read::<usize>(slot);
        crate::mem::write(slot, replacement);
        VirtualProtect(slot as *const c_void, size_of::<usize>(), previous, &mut previous);
        Ok(original)
    }
}

unsafe fn write_jmp(at: usize, to: usize) {
    unsafe {
        crate::mem::write(at, JMP_REL32);
        crate::mem::write(at + 1, (to.wrapping_sub(at + JMP_LENGTH)) as u32);
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x} ")).collect::<String>().trim_end().to_owned()
}
