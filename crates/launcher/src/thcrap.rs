//! The translation patch installed beside the game, put into the game orb starts.
//!
//! **Why orb does this at all.** 紅魔郷 is played in English through thcrap, and thcrap patches a game by
//! injecting into it. Two launchers for one game is one too many: somebody who installs both has to start
//! the game through thcrap and edit its `games.js` to name `orb.exe`, or start it through orb and lose the
//! translation. So orb looks for a thcrap installed beside the game and hands the game it has just created
//! to it — one exe to start, whichever of the two the player thinks of as the launcher.
//!
//! **Both sides are reached through what each publishes about itself**, and nothing here reproduces a
//! step of the other's:
//!
//! - Where thcrap is comes out of the launcher its own configuration tool writes into the game's
//!   directory: `th06 (en).exe` and its like, whose string resources 0, 1 and 2 hold the directory its
//!   `bin` is in, the command line it runs, and the exe it runs. Resource 2 naming `thcrap_loader.exe` is
//!   what says a file is one of those launchers.
//! - Getting thcrap into the process is `thcrap_inject_into_running`, which thcrap exports for this: it
//!   takes the process and the run configuration, and does the loading and the handover itself. Its own
//!   README says a program may use thcrap "with as little effort as possible by calling `LoadLibrary`,
//!   `GetProcAddress`, and executing the function", and this is that.
//!
//! **A failure here does not stop the launch.** The game is orb's business and the words in it are
//! thcrap's: a patch that cannot be found or cannot be injected is a game somebody plays in Japanese,
//! which is the game they had before installing either. Every step says what it did in the line the
//! launcher prints.

use std::ffi::{CString, c_void};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows_sys::Win32::Foundation::{FreeLibrary, HANDLE, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_AS_DATAFILE, LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::LoadStringW;

/// What the configuration tool's own launcher is called, in resource 2 of every one it writes. A file
/// whose resource 2 is this is a thcrap launcher; a file with no resources at all is every other exe in
/// the directory, the game's own included.
const LOADER: &str = "thcrap_loader.exe";

/// The two functions thcrap exports for a program that has a process of its own to patch, in the order
/// its own loader calls them.
///
/// **The first is not optional and is why this is two calls and not one.** A process created suspended has
/// not run the loader initialisation Windows does before its entry point, and thcrap loaded into one in
/// that state leaves the game standing: measured, a game with thcrap's DLLs in it that never created its
/// window. `WaitUntilEntryPoint` is thcrap's own answer — it runs the process as far as its entry point
/// and stops it there, which is after Windows' own initialisation and before any of the game's code — and
/// it is exported for exactly this, under a comment in that list reading "Yes, these are necessary for
/// injection chaining...".
const WAIT: &[u8] = b"WaitUntilEntryPoint\0";
const INJECT: &[u8] = b"thcrap_inject_into_running\0";

/// And the ones its updater is made of, which orb calls in the order and with the filters thcrap's own
/// loader does — see [`update`].
const RUNCONFIG_FROM_FILE: &[u8] = b"runconfig_load_from_file\0";
const RUNCONFIG_FREE: &[u8] = b"runconfig_free\0";
const STACK_UPDATE: &[u8] = b"stack_update_wrapper\0";
const FILTER_GLOBAL: &[u8] = b"update_filter_global_wrapper\0";
const FILTER_GAMES: &[u8] = b"update_filter_games_wrapper\0";
const GLOBAL_BOOLEAN: &[u8] = b"globalconfig_get_boolean\0";
/// Where thcrap is installed, which its updater is told before it resolves anything: `repos` is under it,
/// and without this an update finds no repository and fetches nothing — which is what it did.
const THCRAP_DIR: &[u8] = b"runconfig_thcrap_dir_set\0";

/// The setting thcrap keeps beside the box its own updater window offers.
const AT_EXIT: &[u8] = b"update_at_exit\0";

/// `LOAD_WITH_ALTERED_SEARCH_PATH`, under a name that says what it is for here.
const ALTERED_SEARCH_PATH: u32 = LOAD_WITH_ALTERED_SEARCH_PATH;

/// A thcrap installed beside the game: the DLL to load, the run configuration to hand it, and the rest of
/// what its own launcher was written with.
pub struct Found {
    /// The launcher this was read out of, for the line that says where it came from.
    pub wrapper: PathBuf,
    pub dll: PathBuf,
    pub run_config: PathBuf,
    /// The directory thcrap is installed in, which is where its own loader sets the working directory
    /// before it does anything: `repos` and `config` are resolved against it.
    pub root: PathBuf,
    /// What thcrap calls this game — `th06` — out of the same command line. Its updater is told which
    /// game's files to fetch by this and would otherwise fetch every game's.
    pub game_id: String,
}

/// The thcrap installed beside the game, or `None` where there is none to find.
///
/// Every exe in the directory is asked, since what the configuration tool calls its launcher is the game
/// and the patch stack — `th06 (en).exe` — and orb has no more idea than the player which stack that is.
pub fn beside(game_dir: &Path) -> Option<Found> {
    let mut launchers: Vec<PathBuf> = std::fs::read_dir(game_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        })
        .collect();
    // In the order a directory reads the same way twice, since a directory with two patch stacks in it has
    // no other reason to prefer one — and a launch that picked a different one each time would be a
    // translation that changed by itself.
    launchers.sort();
    launchers.iter().find_map(|path| read(path))
}

/// What one exe says about itself, and `None` for one that is not a thcrap launcher.
fn read(exe: &Path) -> Option<Found> {
    let resources = Resources::of(exe)?;
    // Resource 2 is the exe it runs. Anything else is not one of these launchers — including the game's
    // own exe and orb's, which carry no string resources at all.
    if !resources.string(2)?.eq_ignore_ascii_case(LOADER) {
        return None;
    }
    let bin = resources.string(0)?;
    let command_line = resources.string(1)?;

    // Relative to the launcher's own directory, which is the game's, and absolute where thcrap was
    // installed somewhere a relative path could not reach.
    let bin = Path::new(&bin);
    let bin = if bin.is_absolute() {
        bin.to_owned()
    } else {
        exe.parent()?.join(bin)
    };
    let root = bin.parent()?;

    Some(Found {
        wrapper: exe.to_owned(),
        dll: bin.join("thcrap.dll"),
        root: root.to_owned(),
        game_id: game_id(&command_line)?,
        // thcrap resolves a relative run configuration against its own `config` directory, and this is
        // the same name out of the same command line — made absolute here because what it is relative to
        // on the far side is the working directory, which is the game's.
        run_config: root.join("config").join(run_config(&command_line)?),
    })
}

/// The run configuration out of the command line the launcher was written with —
/// `thcrap_loader.exe "en.js" th06`, whose second word is it.
///
/// The quotes are taken off where they are there: what a name with a space in it is written with is what
/// the far side is handed.
fn run_config(command_line: &str) -> Option<String> {
    let named = command_line
        .split('"')
        .nth(1)
        .filter(|quoted| quoted.ends_with(".js"))
        .map(str::to_owned);
    named.or_else(|| {
        command_line
            .split_whitespace()
            .find(|word| word.ends_with(".js"))
            .map(str::to_owned)
    })
}

/// What thcrap calls the game, out of the same command line: the word that is neither its own loader nor
/// the run configuration.
fn game_id(command_line: &str) -> Option<String> {
    command_line
        .split(['"', ' '])
        .map(str::trim)
        .filter(|word| !word.is_empty())
        .find(|word| !word.ends_with(".exe") && !word.ends_with(".js"))
        .map(str::to_owned)
}

/// A module opened for its resources alone, which is what `LOAD_LIBRARY_AS_DATAFILE` is: nothing in it is
/// initialised and no code of it runs, so an exe that is not a launcher costs a mapping and nothing else.
struct Resources(isize);

impl Resources {
    fn of(exe: &Path) -> Option<Self> {
        let wide: Vec<u16> = exe
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let module = unsafe {
            LoadLibraryExW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_AS_DATAFILE,
            )
        };
        (!module.is_null()).then(|| Self(module as isize))
    }

    /// One string resource, or `None` for an id this file has nothing at.
    ///
    /// Asked for with a length of zero, which is what hands back a pointer into the resource itself
    /// rather than a copy: these strings are not terminated in the file, so the length is the whole of
    /// what says where each ends.
    fn string(&self, id: u32) -> Option<String> {
        let mut at: *mut u16 = std::ptr::null_mut();
        let units =
            unsafe { LoadStringW(self.0 as _, id, &mut at as *mut *mut u16 as *mut u16, 0) };
        if units <= 0 || at.is_null() {
            return None;
        }
        let text = unsafe { std::slice::from_raw_parts(at, units as usize) };
        Some(String::from_utf16_lossy(text))
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0 as _) };
    }
}

/// thcrap.dll, loaded into orb's own process for the calls above and below.
///
/// **`LOAD_WITH_ALTERED_SEARCH_PATH`**, which is what puts the DLL's own directory in front of orb's for
/// the DLLs *it* imports: thcrap.dll is one of a dozen files in that `bin`, and loaded any other way it is
/// the one that is found and its neighbours are not — measured, as `os error 126` on a file that is there.
///
/// Loaded again on a second call rather than kept: Windows counts the loads and hands back the same module,
/// and a launch makes at most two of these calls.
///
/// # Safety
/// Runs whatever `DllMain` that file has.
unsafe fn load(found: &Found) -> Result<HMODULE, String> {
    let wide: Vec<u16> = found
        .dll
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let thcrap =
        unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), ALTERED_SEARCH_PATH) };
    if thcrap.is_null() {
        return Err(format!(
            "cannot load {}: {}",
            found.dll.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(thcrap)
}

/// Another of thcrap's own DLLs out of the same directory, for the ones it loads by name at run time.
///
/// # Safety
/// Runs whatever `DllMain` that file has.
unsafe fn load_beside(found: &Found, name: &str) -> Result<HMODULE, String> {
    let path = found.dll.with_file_name(name);
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let module =
        unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), ALTERED_SEARCH_PATH) };
    if module.is_null() {
        return Err(format!(
            "cannot load {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(module)
}

/// One of thcrap's exports, by name, or a refusal naming the one that is not there.
///
/// A thcrap too old to have one of them is a thcrap orb leaves the game to: what is missing is named so
/// that the line says which, rather than "it did not work".
fn export(thcrap: HMODULE, name: &[u8]) -> Result<usize, String> {
    match unsafe { GetProcAddress(thcrap, name.as_ptr()) } {
        Some(at) => Ok(at as usize),
        None => Err(format!(
            "this thcrap exports no {}",
            String::from_utf8_lossy(&name[..name.len() - 1])
        )),
    }
}

/// Runs the game to its entry point and hands the process to thcrap, which loads itself into it and
/// applies the run configuration.
///
/// The game is stopped at its entry point rather than resumed, so orb's own DLL goes in after this and
/// the game's first instruction runs with both inside it: what thcrap patches is read at startup, and a
/// game already past that has read its files before there was anything to answer them.
///
/// # Safety
/// `process` and `thread` must be a process orb created suspended and its main thread, neither resumed.
pub unsafe fn inject(
    found: &Found,
    game_exe: &Path,
    process: HANDLE,
    thread: HANDLE,
) -> Result<(), String> {
    let thcrap = unsafe { load(found) }?;
    let wait = export(thcrap, WAIT)?;
    let entry = export(thcrap, INJECT)?;

    // **`extern "C"` and not `"system"`**, which on this target is the difference between cdecl and
    // stdcall: thcrap declares both of these as plain C functions — `THCRAP_API` is `__declspec(dllexport)`
    // and names no convention — so the caller cleans the stack. Called as stdcall instead, the stack comes
    // back short by the arguments and orb dies a few instructions later with the game left standing at its
    // entry point, which is what it did.
    //
    // `int (HANDLE, HANDLE, const char *)`, nonzero being the failure — which is how thcrap's own caller
    // reads it. The exe is named in the machine's code page, that argument being a `const char *`.
    let wait: unsafe extern "C" fn(HANDLE, HANDLE, *const u8) -> i32 =
        unsafe { std::mem::transmute(wait) };
    let exe = CString::new(game_exe.display().to_string()).map_err(|error| error.to_string())?;
    let stopped = unsafe { wait(process, thread, exe.as_ptr() as *const u8) };
    if stopped != 0 {
        return Err(format!(
            "thcrap could not run {} to its entry point: {stopped}",
            game_exe.display()
        ));
    }
    // `int (HANDLE, const char *)`, and cdecl for the reason above. The run configuration goes over as
    // bytes of the machine's code page rather than as UTF-16: it is a `const char *` on that side.
    let inject: unsafe extern "C" fn(HANDLE, *const u8) -> i32 =
        unsafe { std::mem::transmute(entry) };
    let run_config = found.run_config.display().to_string();
    let run_config = CString::new(run_config).map_err(|error| error.to_string())?;
    let answered = unsafe { inject(process, run_config.as_ptr() as *const u8) };
    if answered != 0 {
        return Err(format!(
            "thcrap refused the process it was handed: {answered}"
        ));
    }
    Ok(())
}

/// The files thcrap's own patch stack is made of, brought up to date the way its loader does.
///
/// **Its own settings decide when, not orb's.** thcrap keeps `update_at_exit` beside the checkbox its
/// updater window offers — *Install updates after running the game* — so a launch through orb does what a
/// launch through thcrap would: the stack is brought up to date before the game starts unless that box is
/// ticked, in which case this is asked again once the game is running and the next launch is the one that
/// gets the new files.
///
/// **No window of its own and nothing watching it either.** thcrap's updater has one because a download
/// can take a minute; orb has nothing to put a progress bar in and no business putting a second window in
/// front of somebody who asked for a game.
///
/// **Which is why nothing is passed where its progress callback goes.** Reporting is not free here: a
/// `stack_update` given one corrupted the process's heap — first exception `0xc0000374`, then the same
/// access violation 477 times over and a stack overflow to finish, on two runs out of two with files to
/// fetch. The same two calls with nothing where the callback goes fetched the same thirty files and came
/// back. So this reports nothing, and what a launch says about the update is that it ran.
pub enum Update {
    /// The pass that brought the stack up to date, and whether thcrap's own setting had it happen with the
    /// game already running — which is what makes the files the next launch's and not this one's.
    Ran { at_exit: bool },
    /// The other pass's turn by that same setting, so this one did nothing.
    Skipped,
}

/// # Safety
/// `found` must be a thcrap this launch read out of one of its own launchers, and nothing else may be
/// using the process's working directory while this runs.
pub unsafe fn update(found: &Found, now: bool) -> Result<Update, String> {
    let thcrap = unsafe { load(found) }?;
    // **The updater's own DLL, loaded by orb before thcrap asks for it by name.** Every update call is a
    // wrapper in thcrap.dll that loads `thcrap_update.dll` and, where that fails, answers with a fallback
    // that does nothing at all — which is thcrap's own answer for an installation with the updater removed.
    // A `LoadLibrary` by name from orb searches orb's directory and not thcrap's, so the wrapper found
    // nothing and updated nothing: measured, a launch that fetched none of two files deleted by hand. Loaded
    // here with the same altered search path, the module is already in the process under that name when the
    // wrapper looks.
    unsafe { load_beside(found, "thcrap_update.dll") }?;
    // thcrap's own loader sets the working directory to the installation before it reads anything: its
    // `repos` and `config` are resolved against it, and a relative run configuration is too. Put back
    // afterwards, since orb's own paths are the game's and the launcher is not finished with them.
    let ours = std::env::current_dir().map_err(|error| error.to_string())?;
    std::env::set_current_dir(&found.root).map_err(|error| error.to_string())?;
    let answered = unsafe { update_inside(thcrap, found, now) };
    std::env::set_current_dir(ours).map_err(|error| error.to_string())?;
    answered
}

/// # Safety
/// `thcrap` must be a loaded thcrap.dll and the working directory its installation.
unsafe fn update_inside(thcrap: HMODULE, found: &Found, now: bool) -> Result<Update, String> {
    let setting: unsafe extern "C" fn(*const u8, i32) -> i32 =
        unsafe { std::mem::transmute(export(thcrap, GLOBAL_BOOLEAN)?) };
    // Answers nothing, so a run configuration thcrap cannot read is not something this can be told about:
    // its own declaration is `void runconfig_load_from_file(const char *path)`, and read as anything else
    // it is a register nobody wrote that decides whether orb carries on.
    let load_run_cfg: unsafe extern "C" fn(*const u8) =
        unsafe { std::mem::transmute(export(thcrap, RUNCONFIG_FROM_FILE)?) };
    let free_run_cfg: unsafe extern "C" fn() =
        unsafe { std::mem::transmute(export(thcrap, RUNCONFIG_FREE)?) };
    let stack_update: unsafe extern "C" fn(usize, *const c_void, usize, *const c_void) =
        unsafe { std::mem::transmute(export(thcrap, STACK_UPDATE)?) };
    let global_filter = export(thcrap, FILTER_GLOBAL)?;
    let games_filter = export(thcrap, FILTER_GAMES)?;

    // `update_at_exit` is thcrap's own name for the box its updater window offers, and false is what it
    // defaults to there.
    let at_exit = unsafe { setting(AT_EXIT.as_ptr(), 0) } != 0;
    if at_exit != !now {
        return Ok(Update::Skipped);
    }

    let set_thcrap_dir: unsafe extern "C" fn(*const u8) =
        unsafe { std::mem::transmute(export(thcrap, THCRAP_DIR)?) };
    let root = CString::new(found.root.display().to_string()).map_err(|error| error.to_string())?;
    unsafe { set_thcrap_dir(root.as_ptr() as *const u8) };

    let run_config =
        CString::new(found.run_config.display().to_string()).map_err(|error| error.to_string())?;
    unsafe { load_run_cfg(run_config.as_ptr() as *const u8) };

    // The game-independent files first and this game's after, which is the order and the two filters
    // thcrap's own updater uses — and the games filter is what keeps a machine with one game from
    // fetching the translations of thirty. Nothing where the progress callback goes, for the reason
    // [`Update`] gives.
    unsafe {
        stack_update(global_filter, std::ptr::null(), 0, std::ptr::null());
        let id = CString::new(found.game_id.clone()).map_err(|error| error.to_string())?;
        let games: [*const u8; 2] = [id.as_ptr() as *const u8, std::ptr::null()];
        stack_update(
            games_filter,
            games.as_ptr() as *const c_void,
            0,
            std::ptr::null(),
        );
        free_run_cfg();
    }
    Ok(Update::Ran { at_exit })
}

#[cfg(test)]
mod tests {
    use super::run_config;

    /// The run configuration out of the command line thcrap's own configuration tool writes, which is the
    /// one shape this has to read: `thcrap_loader.exe "en.js" th06`.
    #[test]
    fn the_run_configuration_is_the_quoted_js_of_the_command_line() {
        assert_eq!(
            run_config("thcrap_loader.exe \"en.js\" th06").as_deref(),
            Some("en.js")
        );
        // A name with a space in it is why the quotes are there, and why the quoted word is read first.
        assert_eq!(
            run_config("thcrap_loader.exe \"english and jp.js\" th06").as_deref(),
            Some("english and jp.js")
        );
        // And one written without them is still read, a command line being somebody's to edit.
        assert_eq!(
            run_config("thcrap_loader.exe en.js th06").as_deref(),
            Some("en.js")
        );
    }

    /// A command line with no run configuration in it is not one to guess at: the game id alone would
    /// leave thcrap with nothing to apply, and a patch orb made up is worse than none.
    #[test]
    fn a_command_line_with_no_configuration_answers_nothing() {
        assert_eq!(run_config("thcrap_loader.exe th06"), None);
        assert_eq!(run_config(""), None);
        assert_eq!(run_config("\"th06\""), None);
    }
}
