//! Trampoline hooks over the game's code.
//!
//! Only functions whose first instructions are position-independent can be
//! hooked this way, so the expected bytes are passed in and checked. A silent
//! mismatch would mean relocating an instruction that cannot be relocated, and
//! anything else patching the same exe would cause one.

use std::ffi::c_void;
use std::fmt;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
    PAGE_READWRITE, VirtualAlloc, VirtualFree, VirtualProtect,
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
                write!(
                    f,
                    "expected prologue {}, found {}",
                    hex(expected),
                    hex(found)
                )
            }
            Self::ImportNotFound { dll, function } => write!(f, "{dll} does not import {function}"),
            Self::OutOfMemory => write!(f, "cannot allocate a trampoline"),
            Self::Protect => write!(f, "cannot make the target writable"),
        }
    }
}

/// The address of one of orb's own functions, which is what every hook here is given as a number.
///
/// Through a pointer rather than straight to an integer, and named rather than written out at each
/// of the twenty-odd places that needs it. A function *item* is a type of its own with no address
/// until it is coerced to a pointer, so `function as usize` asks the compiler for the address of a
/// thing that does not have one yet and gets it by an implied coercion — which is what
/// `function_casts_as_integer` is about. The call reads `address(replacement as _)`, the `as _`
/// being the coercion this is about doing on purpose.
pub fn address(function: *const ()) -> usize {
    function as usize
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
        if VirtualProtect(
            target as *const c_void,
            prologue.len(),
            PAGE_EXECUTE_READWRITE,
            &mut previous,
        ) == FALSE
        {
            // Given back rather than abandoned: `attach` answers a failed hook by leaving orb
            // doing nothing for the rest of the run, so nothing would ever come back for a page
            // of executable memory nobody holds the address of.
            VirtualFree(trampoline as *mut c_void, 0, MEM_RELEASE);
            return Err(Error::Protect);
        }
        write_jmp(target, hook);
        for offset in JMP_LENGTH..prologue.len() {
            orb_api::mem::write(target + offset, NOP);
        }
        VirtualProtect(
            target as *const c_void,
            prologue.len(),
            previous,
            &mut previous,
        );
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
        let slot =
            crate::pe::import_slot(module, dll, function).ok_or_else(|| Error::ImportNotFound {
                dll: dll.to_owned(),
                function: function.to_owned(),
            })?;
        let mut previous: PAGE_PROTECTION_FLAGS = 0;
        if VirtualProtect(
            slot as *const c_void,
            size_of::<usize>(),
            PAGE_READWRITE,
            &mut previous,
        ) == FALSE
        {
            return Err(Error::Protect);
        }
        let original = orb_api::mem::read::<usize>(slot);
        orb_api::mem::write(slot, replacement);
        VirtualProtect(
            slot as *const c_void,
            size_of::<usize>(),
            previous,
            &mut previous,
        );
        Ok(original)
    }
}

unsafe fn write_jmp(at: usize, to: usize) {
    unsafe {
        orb_api::mem::write(at, JMP_REL32);
        orb_api::mem::write(at + 1, (to.wrapping_sub(at + JMP_LENGTH)) as u32);
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x} "))
        .collect::<String>()
        .trim_end()
        .to_owned()
}

/// [`install_import`] over a module synthesized here, which is the only way to reach it at all.
///
/// **Not a scenario.** A laid-out game has no import table for that function to walk — which is why every
/// scenario turns the memory hooks off, `Fake::attach_declaring` saying so beside the setting — so what
/// stands behind it is a test over headers written out by hand. See
/// [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
///
/// The headers are the ones [`crate::pe::import_slot`] walks and no more: a DOS header pointing at the NT
/// headers, one data directory entry for the imports, one descriptor, and the two arrays a descriptor names —
/// the names the loader resolved *from* and the addresses it wrote *into*. What the test is about is that the
/// walk lands on the second of those and that the swap is reversible, which is the whole of what the memory
/// hooks ask of it.
#[cfg(test)]
mod tests {
    use windows_sys::Win32::System::Diagnostics::Debug::{
        IMAGE_DATA_DIRECTORY, IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_NT_HEADERS32,
    };
    use windows_sys::Win32::System::SystemServices::{
        IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE,
    };

    /// Where each piece of the synthesized module goes, as offsets from its base — which is what a
    /// relative virtual address is once a loader has mapped one.
    const NT: usize = 0x40;
    const IMPORTS: usize = 0x200;
    const NAMES: usize = 0x240;
    const ADDRESSES: usize = 0x250;
    const DLL: usize = 0x260;
    const FUNCTION: usize = 0x280;

    /// The import: the library the game's own table names it in, and the function orb replaces there.
    const IMPORTED_FROM: &[u8] = b"winmm.dll\0";
    const IMPORTED: &[u8] = b"joyGetPosEx\0";

    /// What the loader had written into the slot, which is what a swap has to hand back.
    const RESOLVED: usize = 0x7654_3210;

    /// Whatever orb is putting there. Any address; nothing calls it.
    const REPLACEMENT: usize = 0x1234_5678;

    /// A module of `u64`s so that its base is aligned the way a mapped image's is: the headers are read as
    /// structs through that base, and a `Vec<u8>` promises nothing about where it starts.
    struct Module(Vec<u64>);

    impl Module {
        fn synthesized() -> Self {
            let module = Self(vec![0; 0x400 / size_of::<u64>()]);
            let mut dos: IMAGE_DOS_HEADER = unsafe { std::mem::zeroed() };
            dos.e_magic = IMAGE_DOS_SIGNATURE;
            dos.e_lfanew = NT as i32;
            module.put(0, dos);

            let mut nt: IMAGE_NT_HEADERS32 = unsafe { std::mem::zeroed() };
            nt.Signature = IMAGE_NT_SIGNATURE;
            nt.FileHeader.SizeOfOptionalHeader = size_of_val(&nt.OptionalHeader) as u16;
            nt.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize] =
                IMAGE_DATA_DIRECTORY {
                    VirtualAddress: IMPORTS as u32,
                    Size: size_of::<[u32; 5]>() as u32 * 2,
                };
            module.put(NT, nt);

            // One descriptor and the zeroed one that ends the list, which the `while` walks to.
            module.put(IMPORTS, [NAMES as u32, 0, 0, DLL as u32, ADDRESSES as u32]);
            // The names the loader resolved from, and the addresses it resolved into. Both end in a zero.
            module.put(NAMES, [FUNCTION as u32, 0]);
            module.put(ADDRESSES, [RESOLVED as u32, 0]);
            module.bytes(DLL, IMPORTED_FROM);
            // `IMAGE_IMPORT_BY_NAME`: a u16 hint nothing reads, then the name.
            module.put(FUNCTION, 0u16);
            module.bytes(FUNCTION + size_of::<u16>(), IMPORTED);
            module
        }

        fn base(&self) -> usize {
            self.0.as_ptr() as usize
        }

        fn put<T>(&self, at: usize, value: T) {
            unsafe { ((self.base() + at) as *mut T).write_unaligned(value) };
        }

        fn bytes(&self, at: usize, bytes: &[u8]) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    (self.base() + at) as *mut u8,
                    bytes.len(),
                )
            };
        }

        fn slot(&self) -> usize {
            self.base() + ADDRESSES
        }

        fn in_the_slot(&self) -> usize {
            unsafe { ((self.base() + ADDRESSES) as *const usize).read_unaligned() }
        }
    }

    /// The swap lands on the slot the loader wrote and hands back what was in it.
    #[test]
    fn an_imports_slot_is_swapped_and_what_was_there_comes_back() {
        let module = Module::synthesized();
        let slot = unsafe {
            crate::pe::import_slot(
                module.base(),
                std::str::from_utf8(&IMPORTED_FROM[..IMPORTED_FROM.len() - 1]).unwrap(),
                std::str::from_utf8(&IMPORTED[..IMPORTED.len() - 1]).unwrap(),
            )
        };
        assert_eq!(
            slot,
            Some(module.slot()),
            "the walk did not land on the slot the loader wrote its address into",
        );

        let original = unsafe {
            super::install_import(module.base(), "WINMM.dll", "joyGetPosEx", REPLACEMENT)
        }
        .expect("the import this module has");
        // The library's name matched whatever the case: a table naming it `WINMM.dll` and a call asking for
        // `winmm.dll` are the same import, and `eq_ignore_ascii_case` is what says so.
        assert_eq!(
            original, RESOLVED,
            "the swap handed back something other than the address the loader had resolved",
        );
        assert_eq!(
            module.in_the_slot(),
            REPLACEMENT,
            "the slot does not hold what the swap was asked to put there",
        );
    }

    /// And an import the module has not got is named rather than guessed at.
    #[test]
    fn an_import_a_module_has_not_got_is_named() {
        let module = Module::synthesized();
        let missing = unsafe {
            super::install_import(module.base(), "winmm.dll", "joyGetDevCapsA", REPLACEMENT)
        };
        match missing {
            Err(super::Error::ImportNotFound { dll, function }) => {
                assert_eq!(
                    (dll.as_str(), function.as_str()),
                    ("winmm.dll", "joyGetDevCapsA")
                );
            }
            other => panic!("an import that is not there answered {other:?}"),
        }
        assert_eq!(
            module.in_the_slot(),
            RESOLVED,
            "a swap that found nothing wrote over the slot it did find",
        );
    }
}
