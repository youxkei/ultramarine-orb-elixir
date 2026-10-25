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
use windows_sys::Win32::System::Diagnostics::Debug::{
    CheckRemoteDebuggerPresent, WriteProcessMemory,
};
use windows_sys::Win32::System::Environment::SetEnvironmentVariableW;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows_sys::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessW, CreateRemoteThread, GetExitCodeThread, INFINITE,
    PROCESS_INFORMATION, ResumeThread, STARTUPINFOW, TerminateProcess, WaitForSingleObject,
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

/// Starts `exe` suspended, with `steam_app_id` in the environment it inherits where the game has one.
///
/// **The id is set here and not written beside the exe.** Steam's own client puts it in the environment
/// of a game it started and never writes a `steam_appid.txt`; a launch orb starts directly has nobody to
/// do that, so the game asks Steam to relaunch it and exits — see `orb_core::game::Build::steam_app_id`.
/// The child is created with no environment block of its own and inherits this process's, so setting it
/// here is setting it for the game.
///
/// **And only where Steam has not already said which app this is.** A launch Steam started carries
/// `SteamAppId` already — the app the client launched, 4924840, where the exe's own startup passes
/// 4771400 — and the client ties the session the game opens to that app: overwritten with the exe's
/// own, `SteamAPI_Init` asks the client for an app it did not launch, and it refuses. The id is what a
/// launch with no client behind it is missing.
pub fn spawn_suspended(
    exe: &Path,
    working_dir: &Path,
    options: &[String],
    steam_app_id: Option<u32>,
) -> io::Result<Process> {
    if let Some(app_id) = steam_app_id {
        if std::env::var_os("SteamAppId").is_some() {
            orb_core::log::noted("steam: the SteamAppId this launch inherited is kept");
        } else {
            set_environment("SteamAppId", &app_id.to_string())?;
            orb_core::log::noted(&format!("steam: SteamAppId {app_id} for this launch"));
        }
    }
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

    /// The process and its main thread, for the one thing orb hands to somebody else: the translation
    /// patch beside the game, which runs the process to its entry point and loads itself into it — see
    /// [`crate::thcrap`]. Borrowed rather than given away, both handles being this value's to close.
    pub fn handles(&self) -> (HANDLE, HANDLE) {
        (self.process, self.main_thread)
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

    /// **Waits, the game still suspended, until a debugger is attached to it or `within` has gone**, and says
    /// which of the two it was.
    pub fn waits_for_a_debugger(&self, within: std::time::Duration) -> io::Result<bool> {
        let started = std::time::Instant::now();
        while started.elapsed() < within {
            let mut attached = FALSE;
            if unsafe { CheckRemoteDebuggerPresent(self.process, &mut attached) } == FALSE {
                return Err(io::Error::last_os_error());
            }
            if attached != FALSE {
                return Ok(true);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(false)
    }

    pub fn resume(&mut self) -> io::Result<()> {
        if unsafe { ResumeThread(self.main_thread) } == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        self.resumed = true;
        Ok(())
    }

    /// Waits until the game ends, which a launch Steam started wants: the client counts the process it
    /// started as the game, so a launcher that exits as soon as the DLL is in is a game that lasted a
    /// second — playtime, the overlay and the running state all end there.
    pub fn waits_until_it_ends(&self) -> io::Result<()> {
        if unsafe { WaitForSingleObject(self.process, INFINITE) } != WAIT_OBJECT_0 {
            return Err(io::Error::last_os_error());
        }
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
        // Both types spelled out: what `GetProcAddress` hands back, and what it is being taken
        // for. A transmute with one end inferred is one whose other end can change under it.
        Some(address) => Ok(unsafe {
            std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(*mut c_void) -> u32,
            >(address)
        }),
        None => Err(io::Error::last_os_error()),
    }
}

fn wait_for_module(thread: HANDLE) -> io::Result<()> {
    if unsafe { WaitForSingleObject(thread, DLL_MAIN_TIMEOUT_MS) } != WAIT_OBJECT_0 {
        return Err(io::Error::other("DllMain did not return in time"));
    }
    // **The low 32 bits of the module handle and not the handle**, a thread's exit code being a `u32`: on a
    // 64-bit game a DLL mapped at a base whose low 32 bits are nought reads here as a failed load. Bases are
    // laid on 64 KiB, so that is one launch in 65,536 told it failed where it did not; asking the process for
    // its modules instead would settle it, and no launch has met it yet.
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

/// Sets one variable in this process's environment, which is what a child created with no environment
/// block of its own inherits.
fn set_environment(name: &str, value: &str) -> io::Result<()> {
    let name = wide(name);
    let value = wide(value);
    if unsafe { SetEnvironmentVariableW(name.as_ptr(), value.as_ptr()) } == FALSE {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::set_environment;

    /// The id lands in this process's environment, which is the one a game started with no block of
    /// its own inherits — and the environment is the whole of where the game's Steam startup looks for
    /// it when there is no `steam_appid.txt`.
    #[test]
    fn an_app_id_is_set_where_a_child_inherits_it() {
        set_environment("SteamAppId", "4771400").expect("set");
        assert_eq!(std::env::var("SteamAppId").as_deref(), Ok("4771400"));
    }
}
