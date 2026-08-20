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

use std::sync::OnceLock;

use orb_api::mem;
use orb_core::crash::{self, Around, Fault, Registers, Touching, Whose};
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

/// Orb's own module and the symbols of it, read at the attach.
///
/// **Read now and not at the fault.** A filter that had to open a file and build a table would be
/// allocating out of the heap that a fault inside a heap has just said cannot be trusted, and what it would
/// do about a second fault is nothing. So the work is done while the process is well and the filter only
/// looks things up.
static SYMBOLS: OnceLock<Own> = OnceLock::new();

/// Orb's own module: where it is, how far it runs, and the symbols of it.
struct Own {
    base: usize,
    bytes: usize,
    symbols: Vec<(usize, String)>,
}

pub fn install() {
    unsafe { SetUnhandledExceptionFilter(Some(report)) };
    let _ = SYMBOLS.set(own_symbols().unwrap_or(Own {
        base: 0,
        bytes: 0,
        symbols: Vec::new(),
    }));
}

/// Where orb's own module is and what its symbols are, out of the file it was unpacked into and is running
/// from: the symbol table is a file offset and no part of what the loader maps, so the file is the only
/// place it can be read.
fn own_symbols() -> Option<Own> {
    // An address inside orb itself, which is what asks the loader where orb is: a function of this file's
    // own, taken as a pointer rather than cast, a cast of a function item being a lint of its own.
    let inside: *const () = install as *const fn() as *const ();
    let (_, base, bytes) = module_of_base(inside as usize)?;
    let path = orb_api::module::path(Some(base))?;
    let image = orb_api::fs::read(&path).ok()?;
    Some(Own {
        base,
        bytes,
        symbols: crash::symbols(&image),
    })
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
    for line in crash::lines(&fault, &Host) {
        crate::log::line(&line);
    }
    crate::log::close();
    EXCEPTION_CONTINUE_SEARCH
}

/// The three things about a fault only Windows and orb's own lists can answer.
struct Host;

impl Around for Host {
    /// A frame of orb's own by the name of the function it is in, and everything else by its module.
    ///
    /// Orb's own because those are the frames a fault is worked out from and the ones a name is to be had
    /// for: the games ship no symbols, so `th07.exe+0x341fc` is all there is to say about theirs.
    fn name(&self, address: usize) -> String {
        if let Some(own) = SYMBOLS.get()
            && own.bytes != 0
            && (own.base..own.base + own.bytes).contains(&address)
            && let Some((name, offset)) = crash::nearest(&own.symbols, address - own.base)
        {
            return format!("{name}+{offset:#x}");
        }
        module_of(address).unwrap_or_else(|| "an unknown module".to_owned())
    }

    /// **Orb's own first, then the chapters', then the modules'**, which is the order they can be told
    /// apart in: a buffer of orb's own is inside no module and inside no range a chapter names, and a range
    /// a chapter names is the game's own memory, which a module holds where it is the exe's `.data`.
    fn whose(&self, address: usize) -> Option<Whose> {
        let named = [
            ("a buffer of orb's own", mem::orbs_own()),
            (
                "a region a chapter would put back",
                mem::what_a_restore_needs(),
            ),
            ("the process heap", mem::process_heap_regions()),
        ];
        for (what, ranges) in named {
            if let Some((base, len)) = ranges
                .into_iter()
                .find(|(base, len)| address >= *base && address < base + len)
            {
                return Some(Whose {
                    what: what.to_owned(),
                    base,
                    len,
                });
            }
        }
        // And a module, where the address is inside one: the exe's own `.data` is where most of either
        // game's state is, and a field of it is named by the module and the offset like any other address.
        module_of_base(address).map(|(name, base, len)| Whose {
            what: name,
            base,
            len,
        })
    }

    fn heaps(&self) -> Vec<(usize, bool)> {
        mem::heaps()
    }
}

fn continue_after(message: &str) -> i32 {
    crate::log::line(&format!("crash: {message}"));
    crate::log::close();
    EXCEPTION_CONTINUE_SEARCH
}

/// The module an address is inside, as its name and the extent of its image.
///
/// The extent out of the image's own headers — the PE offset at `+0x3c` of the base and `SizeOfImage` at
/// `+0x50` of that — because what a data address wants said is which module holds it *and how far in*, and
/// a module with no length would be a range nothing can be read against.
fn module_of_base(address: usize) -> Option<(String, usize, usize)> {
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
    let base = module as usize;
    let path = orb_api::module::path(Some(base))?;
    let name = path.file_name()?.to_string_lossy().into_owned();
    let headers = base + mem::read_committed::<u32>(base + 0x3c)? as usize;
    let bytes = mem::read_committed::<u32>(headers + 0x50)? as usize;
    Some((name, base, bytes))
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
