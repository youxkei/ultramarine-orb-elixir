//! Suspended start plus remote `LoadLibraryW`.
//!
//! Injecting while the process is suspended is what lets `orb` hook the CRT's
//! very first allocation: its `DllMain` runs before the game's entry point.

use std::ffi::{OsStr, c_void};
use std::io;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows_sys::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessW, CreateRemoteThread, GetExitCodeThread, PROCESS_INFORMATION,
    ResumeThread, STARTUPINFOW, TerminateProcess, WaitForSingleObject,
};

/// A DLL whose `DllMain` blocks would otherwise hang the launcher with a
/// suspended game process left behind.
const DLL_MAIN_TIMEOUT_MS: u32 = 30_000;

pub struct Process {
    process: HANDLE,
    main_thread: HANDLE,
    id: u32,
    resumed: bool,
}

pub fn spawn_suspended(exe: &Path, working_dir: &Path, options: &[String]) -> io::Result<Process> {
    let application = wide(exe);
    // The options ride on the game's command line, which is where the injected DLL reads them
    // back from. The game never looks at it — `lpCmdLine` appears once in the whole of its
    // `WinMain`, as the parameter it ignores — so this carries orb's own arguments into the
    // process without a file or an environment variable in between.
    let mut command_line = wide(format!("\"{}\" {}", exe.display(), options.join(" ")));
    let working_dir = wide(working_dir);
    let mut startup: STARTUPINFOW = unsafe { zeroed() };
    startup.cb = size_of::<STARTUPINFOW>() as u32;
    let mut info: PROCESS_INFORMATION = unsafe { zeroed() };

    let started = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command_line.as_mut_ptr(),
            null(),
            null(),
            FALSE,
            CREATE_SUSPENDED,
            null(),
            working_dir.as_ptr(),
            &startup,
            &mut info,
        )
    };
    if started == FALSE {
        return Err(io::Error::last_os_error());
    }
    Ok(Process {
        process: info.hProcess,
        main_thread: info.hThread,
        id: info.dwProcessId,
        resumed: false,
    })
}

impl Process {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn load_library(&self, dll: &Path) -> io::Result<()> {
        let path = wide(dll);
        let bytes = size_of::<u16>() * path.len();
        let remote = unsafe {
            VirtualAllocEx(
                self.process,
                null(),
                bytes,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };
        if remote.is_null() {
            return Err(io::Error::last_os_error());
        }
        let result = self.write_path_and_load(remote, &path, bytes);
        unsafe { VirtualFreeEx(self.process, remote, 0, MEM_RELEASE) };
        result
    }

    pub fn resume(&mut self) -> io::Result<()> {
        if unsafe { ResumeThread(self.main_thread) } == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        self.resumed = true;
        Ok(())
    }

    fn write_path_and_load(
        &self,
        remote: *mut c_void,
        path: &[u16],
        bytes: usize,
    ) -> io::Result<()> {
        let written = unsafe {
            WriteProcessMemory(
                self.process,
                remote,
                path.as_ptr().cast(),
                bytes,
                null_mut(),
            )
        };
        if written == FALSE {
            return Err(io::Error::last_os_error());
        }

        let thread = unsafe {
            CreateRemoteThread(
                self.process,
                null(),
                0,
                Some(load_library_w()?),
                remote,
                0,
                null_mut(),
            )
        };
        if thread.is_null() {
            return Err(io::Error::last_os_error());
        }
        let result = wait_for_module(thread);
        unsafe { CloseHandle(thread) };
        result
    }
}

impl Drop for Process {
    /// A process that was never resumed is one nothing has happened in — the launcher gave up
    /// between starting the game and letting it run — and nothing will happen in it either: it
    /// is stopped before its entry point, so it draws no window, ends on nothing, and answers
    /// `tasklist` as a game that is running for as long as the machine is up.
    ///
    /// Killed here rather than on the one path that fails today, so that every way of giving up
    /// between the two, including any added later, leaves nothing behind.
    fn drop(&mut self) {
        unsafe {
            if !self.resumed {
                TerminateProcess(self.process, 1);
            }
            CloseHandle(self.main_thread);
            CloseHandle(self.process);
        }
    }
}

/// kernel32 is mapped at the same address in every process of a session, so the
/// launcher's own `LoadLibraryW` is also the game's.
fn load_library_w() -> io::Result<unsafe extern "system" fn(*mut c_void) -> u32> {
    let kernel32 = unsafe { GetModuleHandleW(wide("kernel32.dll").as_ptr()) };
    if kernel32.is_null() {
        return Err(io::Error::last_os_error());
    }
    let address = unsafe { GetProcAddress(kernel32, c"LoadLibraryW".as_ptr().cast()) };
    match address {
        Some(address) => {
            Ok(unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, _>(address) })
        }
        None => Err(io::Error::last_os_error()),
    }
}

fn wait_for_module(thread: HANDLE) -> io::Result<()> {
    if unsafe { WaitForSingleObject(thread, DLL_MAIN_TIMEOUT_MS) } != WAIT_OBJECT_0 {
        return Err(io::Error::other("DllMain did not return in time"));
    }
    let mut module = 0u32;
    if unsafe { GetExitCodeThread(thread, &mut module) } == FALSE {
        return Err(io::Error::last_os_error());
    }
    if module == 0 {
        return Err(io::Error::other(
            "LoadLibraryW returned NULL; the DLL or one of its dependencies failed to load",
        ));
    }
    Ok(())
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
