//! Header walking over an already-mapped module: sections and imports.
//!
//! The snapshot needs the exe's `.data` bounds and the memory hooks need its
//! import table. Both are read from the loaded headers rather than hardcoded, so
//! a mismatch shows up as a missing section or a missing import instead of a
//! wrong address being used.

use std::mem::offset_of;
use std::ops::Range;

use windows_sys::Win32::System::Diagnostics::Debug::{
    IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_NT_HEADERS32, IMAGE_SECTION_HEADER,
};
use windows_sys::Win32::System::SystemServices::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE,
};

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

const ORDINAL_FLAG: u32 = 0x8000_0000;

/// # Safety
/// `module` must be the base address of a module mapped into this process.
pub unsafe fn section(module: usize, name: &[u8; 8]) -> Option<Range<usize>> {
    unsafe {
        let (nt_address, nt) = headers(module)?;
        let table = nt_address
            + offset_of!(IMAGE_NT_HEADERS32, OptionalHeader)
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
                // Bound imports leave the name array empty and only fill the IAT.
                let names = match (*descriptor).original_first_thunk {
                    0 => (*descriptor).first_thunk,
                    thunk => thunk,
                };
                let names = (module + names as usize) as *const u32;
                let slots = (module + (*descriptor).first_thunk as usize) as *const u32;
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

unsafe fn headers(module: usize) -> Option<(usize, &'static IMAGE_NT_HEADERS32)> {
    unsafe {
        let dos = &*(module as *const IMAGE_DOS_HEADER);
        if dos.e_magic != IMAGE_DOS_SIGNATURE {
            return None;
        }
        let nt_address = module + dos.e_lfanew as usize;
        let nt = &*(nt_address as *const IMAGE_NT_HEADERS32);
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
