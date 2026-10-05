//! Header walking over an already-mapped module: sections and imports.
//!
//! The snapshot needs the exe's `.data` bounds and the memory hooks need its
//! import table. Both are read from the loaded headers rather than hardcoded, so
//! a mismatch shows up as a missing section or a missing import instead of a
//! wrong address being used.

use std::mem::offset_of;
use std::ops::Range;

use windows_sys::Win32::System::Diagnostics::Debug::{
    IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_SECTION_HEADER,
};
use windows_sys::Win32::System::SystemServices::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE,
};

/// **The headers of the architecture orb was built for**, which is the architecture of the module it is
/// walking: a 32-bit process cannot load a 64-bit module or the other way about, so the exe orb is inside
/// is always its own kind.
///
/// **The two differ where it matters.** `FileHeader` is the same either way and so is where
/// `OptionalHeader` begins, which is what [`section`] reads — but the optional header itself is not, so
/// the data directory [`import_slot`] needs sits at a different offset in each. Read as a PE32 in a
/// PE32+, `DataDirectory` answers with whatever is beyond the array's end: th06c's import table was
/// reported as *not importing* `DialogBoxIndirectParamW`, `GetProcAddress` and every other name it does
/// import, so the dialog stood and the keyboard read stayed the game's.
#[cfg(target_arch = "x86")]
use windows_sys::Win32::System::Diagnostics::Debug::IMAGE_NT_HEADERS32 as NtHeaders;
#[cfg(target_arch = "x86_64")]
use windows_sys::Win32::System::Diagnostics::Debug::IMAGE_NT_HEADERS64 as NtHeaders;

/// One entry of either of the import table's two arrays — the names and the addresses the loader writes
/// over them — which is a pointer's width in the image being walked.
#[cfg(target_arch = "x86")]
type Thunk = u32;
#[cfg(target_arch = "x86_64")]
type Thunk = u64;

/// `IMAGE_IMPORT_DESCRIPTOR`, spelled out because the Windows definition wraps
/// its first field in a union that says nothing useful here.
#[repr(C)]
struct ImportDescriptor {
    original_first_thunk: u32,
    time_date_stamp: u32,
    forwarder_chain: u32,
    name: u32,
    first_thunk: u32,
}

/// The top bit of a thunk, which says the entry names an ordinal rather than pointing at a name.
const ORDINAL_FLAG: Thunk = 1 << (Thunk::BITS - 1);

/// # Safety
/// `module` must be the base address of a module mapped into this process.
pub unsafe fn section(module: usize, name: &[u8; 8]) -> Option<Range<usize>> {
    unsafe {
        let (nt_address, nt) = headers(module)?;
        let table = nt_address
            + offset_of!(NtHeaders, OptionalHeader)
            + nt.FileHeader.SizeOfOptionalHeader as usize;
        let table = table as *const IMAGE_SECTION_HEADER;
        (0..nt.FileHeader.NumberOfSections as usize).find_map(|index| {
            let section = &*table.add(index);
            (&section.Name == name).then(|| {
                let start = module + section.VirtualAddress as usize;
                start..start + section.Misc.VirtualSize as usize
            })
        })
    }
}

/// Address of `module`'s import-address-table slot for `dll!function`.
///
/// # Safety
/// `module` must be the base address of a module mapped into this process.
pub unsafe fn import_slot(module: usize, dll: &str, function: &str) -> Option<usize> {
    unsafe {
        let (_, nt) = headers(module)?;
        let directory = nt.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
        if directory.VirtualAddress == 0 {
            return None;
        }

        let mut descriptor =
            (module + directory.VirtualAddress as usize) as *const ImportDescriptor;
        while (*descriptor).name != 0 {
            let imported = cstr(module + (*descriptor).name as usize);
            if imported.eq_ignore_ascii_case(dll.as_bytes()) {
                // **No names, no slot**: a descriptor with no `OriginalFirstThunk` has only the IAT, and once the
                // loader has bound it the IAT holds addresses and not names — read as names, they are wild
                // memory, read from inside `DllMain`. None of the exes orb runs has such a descriptor.
                let names = match (*descriptor).original_first_thunk {
                    0 => return None,
                    thunk => thunk,
                };
                let names = (module + names as usize) as *const Thunk;
                let slots = (module + (*descriptor).first_thunk as usize) as *const Thunk;
                for index in 0.. {
                    let name = *names.add(index);
                    if name == 0 {
                        break;
                    }
                    if name & ORDINAL_FLAG != 0 {
                        continue;
                    }
                    // IMAGE_IMPORT_BY_NAME: a u16 hint, then the name.
                    if cstr(module + name as usize + 2) == function.as_bytes() {
                        return Some(slots.add(index) as usize);
                    }
                }
            }
            descriptor = descriptor.add(1);
        }
        None
    }
}

unsafe fn headers(module: usize) -> Option<(usize, &'static NtHeaders)> {
    unsafe {
        let dos = &*(module as *const IMAGE_DOS_HEADER);
        if dos.e_magic != IMAGE_DOS_SIGNATURE {
            return None;
        }
        let nt_address = module + dos.e_lfanew as usize;
        let nt = &*(nt_address as *const NtHeaders);
        (nt.Signature == IMAGE_NT_SIGNATURE).then_some((nt_address, nt))
    }
}

unsafe fn cstr(address: usize) -> &'static [u8] {
    unsafe {
        let bytes = address as *const u8;
        let mut length = 0;
        while *bytes.add(length) != 0 {
            length += 1;
        }
        std::slice::from_raw_parts(bytes, length)
    }
}
