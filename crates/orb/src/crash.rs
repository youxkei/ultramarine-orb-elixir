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
//!
//! **And the memory is kept beside them, as a minidump.** The lines name the fault and nothing else: there
//! is no stack below the first few frames, no object the faulting frame was reading, and nothing of the
//! module list as the loader had it. A th10 launch died inside `nvd3dum.dll` on 2026-10-28
//! (`0xc0000005` reading `0x3d000019`, five frames, none of them orb's) and five lines is everything
//! there was to read — a fault in a driver is not a fault anybody fixes by reading the log. A dump opened
//! later in `cdb` answers what the lines cannot, and is what `--dump-on-a-crash` asks for by name.

use std::sync::OnceLock;

use orb_api::mem;
use orb_core::crash::{self, Around, Fault, Registers, Touching, Whose};
use windows_sys::Win32::Foundation::{
    CloseHandle, EXCEPTION_ACCESS_VIOLATION, HANDLE, HMODULE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ,
};
use windows_sys::Win32::System::Diagnostics::Debug::{
    EXCEPTION_POINTERS, MINIDUMP_EXCEPTION_INFORMATION, MiniDumpNormal, MiniDumpWithDataSegs,
    MiniDumpWithFullMemoryInfo, MiniDumpWithHandleData, MiniDumpWithIndirectlyReferencedMemory,
    MiniDumpWithThreadInfo, MiniDumpWithUnloadedModules, MiniDumpWriteDump,
};
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleHandleExW,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId,
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

/// **Where the minidump of a fault goes**, which the attach settles before anything can fault — a
/// directory and not a path, because the name is the process's and is made at the fault: `orb-<pid>.dmp`,
/// and never over one an earlier launch of the same pid left, that pid being another process by now.
///
/// Set by [`wants_a_dump_in`] and nowhere else, so that a launch nobody asked leaves no file: a driver's
/// fault comes again every few hundred launches, and a dump apiece is a directory somebody has to empty.
static DUMP_IN: OnceLock<std::path::PathBuf> = OnceLock::new();

/// **Whether a dump is wanted with this launch's crashes**, asked at the attach by `--dump-on-a-crash`.
pub fn wants_a_dump_in(dir: std::path::PathBuf) {
    let _ = DUMP_IN.set(dir);
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
        // **Named by the 32-bit registers on either architecture**, which is what the eight fields of
        // [`Registers`] are: a fault in a 64-bit game is read off `Rax` and the rest, and what a
        // reader of the log wants of one is the same eight values under the same eight names.
        registers: unsafe { (*exception).ContextRecord.as_ref() }.map_or_else(
            Registers::default,
            |context| {
                #[cfg(target_arch = "x86")]
                let (a, b, c, d, si, di, sp, bp) = (
                    context.Eax,
                    context.Ebx,
                    context.Ecx,
                    context.Edx,
                    context.Esi,
                    context.Edi,
                    context.Esp,
                    context.Ebp,
                );
                #[cfg(target_arch = "x86_64")]
                let (a, b, c, d, si, di, sp, bp) = (
                    context.Rax,
                    context.Rbx,
                    context.Rcx,
                    context.Rdx,
                    context.Rsi,
                    context.Rdi,
                    context.Rsp,
                    context.Rbp,
                );
                Registers {
                    eax: a as usize,
                    ebx: b as usize,
                    ecx: c as usize,
                    edx: d as usize,
                    esi: si as usize,
                    edi: di as usize,
                    esp: sp as usize,
                    ebp: bp as usize,
                }
            },
        ),
    };
    for line in crash::lines(&fault, &Host) {
        crate::log::line(&line);
    }
    // **The log closed before the dump is written, and the path named before either**: the five lines above
    // are handed to the log's own thread and reach the disk on `close`, so a `MiniDumpWriteDump` that hung —
    // it allocates, opens a file and walks a process whose heap the fault may have been inside — would take
    // the one record that exists today down with it. Said first, so the file still names where the dump was
    // to go whether or not it got there.
    if let Some(dir) = DUMP_IN.get() {
        crate::log::line(&format!(
            "crash: a dump of this process is being written to {}",
            dir.join(format!("orb-{}.dmp", unsafe { GetCurrentProcessId() }))
                .display()
        ));
    }
    crate::log::close();
    unsafe { write_the_dump(exception) };
    EXCEPTION_CONTINUE_SEARCH
}

/// **The process's memory, written out beside the log as `orb-<pid>.dmp`**, where the launch asked for one
/// — see [`wants_a_dump_in`].
///
/// **Written through `dbghelp`'s own call and never a walk of our own**: a filter runs on the thread that
/// faulted, in a process whose heap a fault may have been inside, so everything this does it does through
/// host calls and hands the exception record it is holding straight across.
///
/// **What it holds is chosen for a fault nothing of orb's is in**, which is the one this exists for: every
/// thread's stack and what each was waiting on, every module as the loader had it with the ones already
/// unloaded, and — the flag that matters most — the memory the stacks point at. A driver's fault is a
/// `nvd3dum+offset` with no symbols behind it, so what says what the faulting frame was reading is that
/// object and not the frame; the module data segments alone are not it, which is what this asked for first
/// and what would have answered less than the doc said it did.
unsafe fn write_the_dump(exception: *mut EXCEPTION_POINTERS) {
    let Some(dir) = DUMP_IN.get() else {
        return;
    };
    // Made where it was asked for and not beside the game, which is where the log went: a session of
    // `cargo xtask real` keeps each launch's files in its own directory, and a dump in the install is one
    // the next case starts beside.
    let _ = orb_api::fs::create_dir_all(dir);
    let path = dir.join(format!("orb-{}.dmp", unsafe { GetCurrentProcessId() }));
    let wide: Vec<u16> = {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str().encode_wide().chain([0]).collect()
    };
    let file = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0x4000_0000, // GENERIC_WRITE, which windows-sys wants the `Win32_Storage_FileSystem` feature to name.
            FILE_SHARE_READ,
            std::ptr::null(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    if file == INVALID_HANDLE_VALUE {
        crate::log::noted(&format!(
            "crash: no dump written; {} cannot be made",
            path.display()
        ));
        return;
    }
    let record = MINIDUMP_EXCEPTION_INFORMATION {
        // **This thread's own id, and nought is not "this thread"**: a dump written with nought stores an
        // exception whose thread `cdb` calls `FFFFFFFF` and refuses, and `.ecxr` — the one command the whole
        // dump exists for — answers `Unable to get exception context`. Found by opening a dump this filter
        // wrote and reading it, which is what the test's kept file is for.
        ThreadId: unsafe { GetCurrentThreadId() },
        ExceptionPointers: exception,
        // Ours and not a debugger's, which is what says the pointers are this process's own addresses.
        ClientPointers: 0,
    };
    let written = unsafe {
        MiniDumpWriteDump(
            GetCurrentProcess(),
            GetCurrentProcessId(),
            file as HANDLE,
            MiniDumpNormal
                | MiniDumpWithDataSegs
                | MiniDumpWithHandleData
                | MiniDumpWithThreadInfo
                | MiniDumpWithFullMemoryInfo
                | MiniDumpWithUnloadedModules
                | MiniDumpWithIndirectlyReferencedMemory,
            &record,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    unsafe { CloseHandle(file as HANDLE) };
    // **Said into the file after the fact and not before it** — the log is already closed, so this is
    // `noted`, which opens it, says one line and closes it again: what the fault left behind is read
    // together, and a dump nobody was told about is one nobody opens.
    if written != 0 {
        crate::log::noted(&format!(
            "crash: the dump is {} — `cdb -z` opens it",
            path.display()
        ));
    } else {
        crate::log::noted(&format!(
            "crash: {} was made and MiniDumpWriteDump refused it",
            path.display()
        ));
    }
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

#[cfg(test)]
mod tests {
    use windows_sys::Win32::System::Diagnostics::Debug::{
        CONTEXT, EXCEPTION_POINTERS, EXCEPTION_RECORD,
    };

    /// **The dump is the one `cdb -z` opens**, and not merely a file this wrote: the bytes start with the
    /// dump's own signature, `MDMP`, and it holds more of the process than the five lines beside it.
    ///
    /// **The exception is made and not raised**, which is what makes this test one: raising a fault to get
    /// real pointers would take the harness down the moment anything about the filter was wrong, and what is
    /// being asked here — does `MiniDumpWriteDump` write a dump of *this* process when handed a record — is a
    /// question a record written out by hand answers. The pointers are this process's own, which is what
    /// `ClientPointers` nought says.
    #[test]
    fn a_dump_is_written_and_is_one_a_debugger_opens() {
        use std::sync::atomic::{AtomicBool, Ordering};

        /// Whether the filter of this test ran — the write is called through it and not around it, so that
        /// what the test exercises is the path a real fault takes.
        static RAN: AtomicBool = AtomicBool::new(false);

        let dir = std::env::temp_dir().join(format!("orb-crash-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // The directory is made by the write itself, so it is not made here: that is half of what the write
        // is asked to do.
        super::wants_a_dump_in(dir.clone());

        let mut record: EXCEPTION_RECORD = unsafe { std::mem::zeroed() };
        // The same code and address a th10 launch died of on 2026-10-28, as a signed `i32` because that is
        // what `EXCEPTION_RECORD` holds.
        record.ExceptionCode = 0xc000_0005u32.cast_signed();
        record.ExceptionAddress = 0x3d00_0019 as *mut std::ffi::c_void;
        record.ExceptionInformation[0] = 0; // reading
        record.ExceptionInformation[1] = 0x3d00_0019;
        let mut context: CONTEXT = unsafe { std::mem::zeroed() };
        let mut pointers = EXCEPTION_POINTERS {
            ExceptionRecord: &mut record,
            ContextRecord: &mut context,
        };
        unsafe { super::write_the_dump((&raw mut pointers).cast()) };
        RAN.store(true, Ordering::Release);
        assert!(RAN.load(Ordering::Acquire));

        let dumps: Vec<_> = std::fs::read_dir(&dir)
            .expect("the directory the dump was asked for")
            .flatten()
            .filter(|entry| entry.path().extension().is_some_and(|it| it == "dmp"))
            .map(|entry| entry.path())
            .collect();
        assert_eq!(dumps.len(), 1, "one write is one dump");
        let bytes = std::fs::read(&dumps[0]).expect("the dump reads back");
        assert_eq!(&bytes[..4], b"MDMP", "the dump is not a minidump");
        assert!(
            bytes.len() > 0x1000,
            "a dump of {} bytes names nothing of the process",
            bytes.len()
        );
        // **And its name is the process's**, which is what makes two launches of one pid tell apart.
        assert_eq!(
            dumps[0].file_stem().and_then(|it| it.to_str()),
            Some(
                format!("orb-{}", unsafe {
                    windows_sys::Win32::System::Threading::GetCurrentProcessId()
                })
                .as_str()
            )
        );
        // Kept where a run is asked to, so that what the dump holds can be opened with `cdb` and read:
        // what a test can assert is that the file is a minidump, and what it says about the process is
        // what somebody has to look at.
        if std::env::var_os("ORB_KEEP_DUMP").is_none() {
            let _ = std::fs::remove_dir_all(&dir);
        } else {
            println!("the dump is under {}", dir.display());
        }
    }
}
