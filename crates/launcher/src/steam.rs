//! The Steam client, for a launch of a game sold through it.
//!
//! **A Steam game started directly asks Steam to start it and exits**, and the process that carries on
//! is one the client made — with no orb in it. orb starts the game itself, with the app id in its
//! environment (`game::Build::steam_app_id`), so nothing makes that request any more; what is left is
//! the half the game would have done, and this is it: the client is started where it is not running,
//! and waited for before the game is.
//!
//! **Which is only for a game Steam knows** — a build whose `steam_app_id` is some — and not for a
//! launch that has taken Steam out of the run entirely: `without_steam` answers the game's Steam calls
//! itself, and needs no client at all.
//!
//! **Where the client is, and whether it is running, are both its own registry answers.**
//! `HKCU\Software\Valve\Steam\SteamExe` names the exe, and
//! `HKCU\Software\Valve\Steam\ActiveProcess\pid` is the client's own process id while it runs and
//! nought while it does not. Started, the client is up when `steamwebhelper.exe` is among the
//! processes — the signal the autoplay skill waits on, and not merely that `steam.exe` was launched —
//! **and `ActiveUser` is not nought**: a client nobody has logged in to cannot serve a game, and a
//! launch started the moment the helper appeared was measured failing at `SteamAPI_Init` for it.
//!
//! **A failure here does not stop the launch**, as thcrap's does not: the game is started either way,
//! and a game with no client is one whose own `SteamAPI_Init` says so and exits with its `log.txt` at
//! the 122 bytes of its header.

use std::ffi::{OsStr, c_void};
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_SUCCESS, INVALID_HANDLE_VALUE, STILL_ACTIVE,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegGetValueW,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// Where the client names its own exe, and where it keeps the pid it runs under.
const STEAM: &str = r"Software\Valve\Steam";
const ACTIVE_PROCESS: &str = r"Software\Valve\Steam\ActiveProcess";

/// The process the client runs its window in, which is what says it is up rather than merely started.
const HELPER: &str = "steamwebhelper.exe";

/// How long a client this started is given to come up, in all: a cold start verifies the client's
/// files and logs in before it can serve a game, and the one measured here took a little over a
/// minute on a machine that was not busy.
const WITHIN: Duration = Duration::from_secs(180);

/// How often it is asked.
const INTERVAL: Duration = Duration::from_millis(250);

/// Makes sure the client is running before the game is started, starting it where it is not, and says
/// what it did as a clause for the launcher's own line — or nothing, where the client was already up
/// and there was nothing to do.
pub fn make_sure_it_runs() -> Option<String> {
    if the_client_is_up() {
        return None;
    }
    if let Err(why) = start() {
        return Some(format!("{why}; the game is started without it"));
    }
    let started = Instant::now();
    while started.elapsed() < WITHIN {
        if the_client_is_up() {
            return Some(format!(
                "started the client, and it was up in {:.1}s",
                started.elapsed().as_secs_f32()
            ));
        }
        std::thread::sleep(INTERVAL);
    }
    Some(format!(
        "started the client, and it was not up after {}s; the game is started anyway",
        WITHIN.as_secs()
    ))
}

/// Whether the client runs and has finished starting: its own pid is live, its window's process is
/// there, and somebody is logged in.
///
/// **The account as well as the process**, because a game's `SteamAPI_Init` has nothing to open until
/// one is: measured, a launch started the moment `steamwebhelper.exe` appeared was one whose Init
/// failed, where the same launch a few seconds later, with `ActiveUser` set, went on to its dialog.
fn the_client_is_up() -> bool {
    let Some(pid) = read_dword(ACTIVE_PROCESS, "pid") else {
        return false;
    };
    let logged_in = read_dword(ACTIVE_PROCESS, "ActiveUser").is_some_and(|user| user != 0);
    process_is_alive(pid) && logged_in && any_process_named(HELPER)
}

/// Starts the client from the exe it names, with nothing of this launch attached to it: the client
/// outlives orb, and its own window is its own.
fn start() -> Result<(), String> {
    let exe = steam_exe()
        .ok_or_else(|| "the client's own exe is not named in the registry".to_owned())?;
    Command::new(&exe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|why| format!("could not start {}: {why}", exe.display()))
}

/// The client's exe, out of the registry: `SteamExe` names it outright, and `SteamPath` is the older
/// answer the same client keeps, a directory with the exe beside it.
fn steam_exe() -> Option<PathBuf> {
    read_string(STEAM, "SteamExe")
        .map(PathBuf::from)
        .or_else(|| read_string(STEAM, "SteamPath").map(|dir| PathBuf::from(dir).join("steam.exe")))
}

/// Whether a process of that id runs, which for nought is nobody.
fn process_is_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return false;
    }
    let mut code = 0u32;
    let read = unsafe { GetExitCodeProcess(process, &mut code) } != 0;
    unsafe { CloseHandle(process) };
    read && code == STILL_ACTIVE as u32
}

/// Whether the process of that exe name is among the ones running.
fn any_process_named(name: &str) -> bool {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }
    let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    let mut found = false;
    while more {
        let ended = entry
            .szExeFile
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(entry.szExeFile.len());
        if String::from_utf16_lossy(&entry.szExeFile[..ended]).eq_ignore_ascii_case(name) {
            found = true;
            break;
        }
        more = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    found
}

/// One value of this user's registry, as a string.
fn read_string(key: &str, value: &str) -> Option<String> {
    let key = wide(key);
    let value = wide(value);
    // A client's paths are far shorter than this, and a value longer than the buffer comes back as
    // `ERROR_MORE_DATA`, which is the `None` below.
    let mut buffer = [0u16; 512];
    let mut bytes = (buffer.len() * size_of::<u16>()) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast::<c_void>(),
            &mut bytes,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    // The size the call wrote counts the terminating nought.
    let ended = (bytes as usize / size_of::<u16>()).saturating_sub(1);
    Some(String::from_utf16_lossy(&buffer[..ended]))
}

/// One value of this user's registry, as a number.
fn read_dword(key: &str, value: &str) -> Option<u32> {
    let key = wide(key);
    let value = wide(value);
    let mut data = 0u32;
    let mut bytes = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut data as *mut u32).cast::<c_void>(),
            &mut bytes,
        )
    };
    (status == ERROR_SUCCESS).then_some(data)
}

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::process_is_alive;

    /// This process is alive, which is what the pid the client keeps is asked for.
    #[test]
    fn a_living_pid_is_one_that_runs() {
        assert!(process_is_alive(std::process::id()));
    }

    /// And nought is nobody, which is the answer the client's registry carries while it is closed.
    #[test]
    fn nought_is_no_process_at_all() {
        assert!(!process_is_alive(0));
    }
}
