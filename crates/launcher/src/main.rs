//! Starts a game orb knows with `orb.dll` loaded before the game's entry point runs.
//!
//! Which games those are is `orb_core::game::KNOWN`, read here as well as in the DLL so that the two
//! cannot disagree about which games exist — see `docs/adr/0004`.
//!
//! The DLL is carried inside this exe and written out to be loaded, so installing orb is one
//! file.

// **Not a console program.** Windows makes a console for one that was not started from a shell, and
// that window is on the screen for as long as this process lives — the whole of the settings dialog,
// and a flash of it otherwise — in front of the game somebody asked for. Hiding it once it is there,
// `ShowWindow` on `GetConsoleWindow`, is that window appearing and then going away again, which is
// the whole of what there is to avoid. What this prints goes to the console of whatever started the
// launch instead, which `attach_to_the_console` asks for.
#![windows_subsystem = "windows"]

mod inject;
mod pad;
mod settings;
mod thcrap;

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use orb_config::args::Options;
use orb_config::{Config, Language};
use orb_core::game::{self, Known};
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Console::{
    ATTACH_PARENT_PROCESS, AttachConsole, GetStdHandle, STD_ERROR_HANDLE, STD_HANDLE,
    STD_OUTPUT_HANDLE, SetStdHandle,
};
use windows_sys::Win32::System::Threading::{
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, TIMER_ALL_ACCESS,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONERROR, MB_OK, MB_SETFOREGROUND, MessageBoxW,
};

/// Starts the game orb finds where it was pointed, and hands the options below on to it.
///
/// Every option but the launcher's own is read twice, here and again inside the game off the
/// command line this writes, so nothing about which pass a launch is has to be written down for
/// the two halves to agree.
#[derive(Parser, Debug)]
// Capped at the width the prose below is written to be read at, so that a wide terminal gets
// paragraphs rather than one line per option running off the side of it.
#[command(name = "orb", max_term_width = 100, after_help = orb_config::args::AFTER_HELP)]
struct Launch {
    #[command(flatten)]
    options: Options,

    /// where the game's exe is, if it is not beside orb.exe
    #[arg(long, require_equals = true, value_name = "PATH", help_heading = MINE)]
    game_dir: Option<PathBuf>,

    /// an orb.dll to load instead of the one orb.exe carries inside itself
    #[arg(long, require_equals = true, value_name = "PATH", help_heading = MINE)]
    orb_dll: Option<PathBuf>,

    /// the orb.yaml to read, instead of the one beside orb.exe
    #[arg(long, require_equals = true, value_name = "PATH", help_heading = MINE)]
    config: Option<PathBuf>,

    /// ask for the settings — the screen, the ending, the wash a chapter gets — however
    /// orb.yaml has "ask at startup" set, and write down what is answered
    #[arg(long, help_heading = MINE)]
    settings: bool,
}

/// The four the launcher answers itself. The DLL is inside the game already, so where the game
/// and the DLL are is a question it never has to ask, and a game that has started is past the
/// moment the settings could be asked for.
const MINE: &str = "Read here and not handed on to the game";

/// What a host that cannot make the timer orb's frame loop waits on is told.
///
/// Its own words rather than the DLL's, because the two are different situations: the DLL's message is
/// a game that is being stopped, and this one is a game that was never started.
fn no_timer_text(language: Language) -> &'static str {
    match language {
        Language::Japanese => {
            "orb がゲームのフレームを刻むのに使う高分解能タイマーを、このマシンでは作れません。orb は\
             ほかの方法でフレームを刻みません。\n\nWindows 10 バージョン 1803 以降が必要です。ゲームは\
             起動していません。"
        }
        Language::English => {
            "This host cannot create the high-resolution timer that orb paces the game's frames on, \
             and orb does not pace them any other way.\n\nWindows 10 version 1803 or later is \
             needed. The game has not been started."
        }
    }
}

/// And the line for a terminal, which stays English whatever the machine reads: it is one line of what
/// a run says of itself, and `orb.log` is English throughout — see [`Refused`].
const NO_TIMER_LINE: &str = "this host cannot create a high-resolution waitable timer, which orb \
     paces the game's frames on — Windows 10 1803 or later is needed, and the game was not started";

/// A refusal that has words of its own for the dialog, beside the line it prints.
///
/// **Two wordings because they are read by two people.** The line goes to whoever ran orb from a shell
/// and is in the English every log line is in, so that two reports of one fault read the same whichever
/// machines they came off. The dialog is read by whoever double-clicked the exe and has nothing else to
/// go on, so it is in the language their machine is in — and in sentences, a printed line being terse
/// where a paragraph is what a dialog holds.
///
/// Only the refusals somebody playing can actually meet are here: no game where orb was pointed, and an
/// exe that is no build orb knows. A file that cannot be read or a process that will not start says
/// whatever Windows said, in a dialog that gets the line as it stands — which is what every refusal got
/// before any of them had words.
#[derive(Debug)]
struct Refused {
    line: String,
    text: String,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.line)
    }
}

impl Error for Refused {}

/// Whether this host can create the timer at all, which is the whole of what orb needs of it that a
/// host may not have.
///
/// The flag is Windows 10 1803's, and a creation that fails is not a configuration to carry code for:
/// it is a machine orb does not run on. Made and given straight back — what is being asked is whether
/// the host will, and the one the frame loop waits on is the DLL's own, made inside the game.
fn can_make_the_timer() -> bool {
    let timer = unsafe {
        CreateWaitableTimerExW(
            std::ptr::null(),
            std::ptr::null(),
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
            TIMER_ALL_ACCESS,
        )
    };
    if timer.is_null() {
        return false;
    }
    unsafe { CloseHandle(timer) };
    true
}

/// Says so where a launch from Explorer will see it, which is nowhere a printed line reaches.
fn cannot_run(text: &str) {
    let title: Vec<u16> = "Ultramarine Orb Elixir".encode_utf16().chain([0]).collect();
    let text: Vec<u16> = text.encode_utf16().chain([0]).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
        )
    };
}

/// Takes the console of whatever started this launch, and answers whether a line printed here
/// reaches anybody at all.
///
/// A shell hands its console to a console program and this is not one, so the console of the process
/// that started this one is asked for outright. `ATTACH_PARENT_PROCESS` is that ask, and it fails
/// where that process has no console to lend: a launch from Explorer, from a shortcut, from anything
/// that is not a shell. Which is the answer, together with a redirection: what a launch was given as
/// its standard output before it ran is something reading the output too.
///
/// **What the parent handed over is put back**, because attaching takes the standard handles as well
/// as the console. Measured on Windows 11, an exe like this one printing a line on either side of the
/// attach and run from cmd as `exe > file`: the file holds the line from before it and not the one
/// after, which went to cmd's console instead.
fn attach_to_the_console() -> bool {
    let handed_over = |id: STD_HANDLE| match unsafe { GetStdHandle(id) } {
        handle if handle.is_null() || handle == INVALID_HANDLE_VALUE => None,
        handle => Some(handle),
    };

    let (out, error) = (
        handed_over(STD_OUTPUT_HANDLE),
        handed_over(STD_ERROR_HANDLE),
    );
    let attached = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0;
    for (id, handle) in [(STD_OUTPUT_HANDLE, out), (STD_ERROR_HANDLE, error)] {
        if let Some(handle) = handle {
            unsafe { SetStdHandle(id, handle) };
        }
    }
    attached || out.is_some() || error.is_some()
}

/// Says why no game started, in whichever of the two forms the launch that stopped can see: the line
/// always, and the dialog where nothing is going to read a line.
///
/// Both were said before there was any way to tell one launch from the other, which put a dialog in
/// front of a terminal that had the line already.
fn not_started(line: &str, text: &str, lines_are_read: bool) -> ExitCode {
    // Printed whether or not it is read, rather than guarded: a launch with no console and no
    // redirection has no standard error at all, and printing into that returns `Ok` — measured on
    // Windows 11, and worth knowing under `panic = "abort"`, where a print that failed instead would
    // be the launcher taken down by a line nobody was going to see.
    eprintln!("orb: {line}");
    if !lines_are_read {
        cannot_run(text);
    }
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    // Before anything is printed, which is everything below.
    let lines_are_read = attach_to_the_console();

    // Before anything is read or started, because nothing else matters on a host orb cannot pace
    // frames on. The DLL asks the same question of the same call and its answer is to stop the game
    // on its first frame; asking it here is that answer with the game never started, which is the
    // one an unsupported host should get — see `docs/adr/0006`.
    if !can_make_the_timer() {
        // Words of its own for the dialog, where a paragraph reads as one — and in the machine's own
        // language, no file having been read yet: this is before `orb.yaml`, which is where a language
        // asked for by name would be.
        return not_started(
            NO_TIMER_LINE,
            no_timer_text(Language::of_the_machine()),
            lines_are_read,
        );
    }

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        // The dialog's own words where the refusal has them, and the line in both where it has not —
        // see [`Refused`].
        Err(error) => {
            let line = error.to_string();
            let text = match error.downcast_ref::<Refused>() {
                Some(refused) => refused.text.clone(),
                None => line.clone(),
            };
            not_started(&line, &text, lines_are_read)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // Read here as well as in the DLL, so that anything unreadable is said before a game
    // starts rather than into a log inside one.
    let launch = Launch::parse();

    let exe = std::env::current_exe()?;
    // Settled before the settings are asked for, because the pad they can be answered with is
    // described by a file in the game's directory too.
    let path = config_path(&exe, launch.game_dir.as_deref(), launch.config.as_deref());
    let mut config = match &launch.config {
        Some(path) => Config::load(path)?,
        None => Config::load_beside(&path)?,
    };

    // `--config` names a file that may be anywhere, so where the game is has to be said again;
    // for the file orb found itself, the directory it was found in is already the game's.
    if let Some(game_dir) = launch.game_dir {
        config.game_dir = game_dir;
    }

    // Which language the two refusals below are read in, and the dialog with them. Settled once the
    // file has been read, that being where a language asked for by name is.
    let language = config.language.unwrap_or_else(Language::of_the_machine);

    // Which of the games orb knows this directory holds, before the settings are asked for: the pad
    // mapping the dialog answers through is read out of that game's own configuration file, so the
    // game has to be settled first.
    let known = game_in(&config.game_dir, language)?;

    // Before the options are applied, so that what is written back is the settings and not one
    // launch's arguments — and before the game starts, since the DLL reads the same file from
    // inside it.
    if config.ask_at_startup || launch.settings {
        let game_cfg = config.game_dir.join(known.cfg);
        // Said before the dialog, because a pad answering it the wrong way round is this mapping and
        // there is nowhere else it is written down in a form anybody can read.
        println!("orb: pad — {}", pad::Mapping::read(&game_cfg).describe());
        let asked = settings::ask(&config, &game_cfg)?;
        // Which hand answered, and what the pad did while the dialog was up. Both, because a pad
        // answering a dialog is orb's own doing: without the first there is no evidence it worked,
        // and without the second a pad that was never there cannot be told from one that was
        // pushed and ignored.
        let with = settings::answered_with();
        let saw = pad::report();
        let Some(answers) = asked else {
            println!("orb: no game started (answered on the {with}; {saw})");
            return Ok(());
        };
        answers.apply(&mut config);
        config.save(&path)?;
        println!(
            "orb: settings written to {} (answered on the {with}; {saw})",
            path.display()
        );
    }

    config.apply(&launch.options);
    let options = to_hand_on(std::env::args().skip(1));

    let game_exe = config.game_dir.join(known.exe);
    let build = verify_game_exe(&game_exe, known, language)?;

    // Written out before the game starts, because a `LoadLibrary` needs a path. Kept out
    // of the game's directory: nothing there is orb's to leave behind, and this file is a
    // detail of how the launcher carries its payload rather than something to install.
    let orb_dll = match launch.orb_dll {
        Some(path) => path,
        None => unpack_orb()?,
    };

    // The translation patch installed beside the game, if there is one and the settings want it — two
    // launchers for one game is one too many, and which of them somebody thinks of as the launcher is not
    // orb's business. See `thcrap`.
    //
    // Nothing about it stops a launch: a patch that cannot be found, brought up to date or handed over is
    // a game played in Japanese, which is the game somebody had before installing either of the two.
    let patch = config
        .thcrap
        .then(|| thcrap::beside(&config.game_dir))
        .flatten();
    if config.thcrap && patch.is_none() {
        println!("orb: thcrap — none installed where the game is");
    }
    // Before the game starts, which is where thcrap puts it unless its own setting says otherwise: a patch
    // updated after the launch is one this launch is not playing with.
    if let Some(found) = &patch {
        say_the_update(unsafe { thcrap::update(found, true) });
    }

    // Handed on to the game, whose command line is where the DLL reads them back from: the
    // game itself never looks at it. Nothing is written down and the two sides cannot
    // disagree about which pass this is.
    let mut process = inject::spawn_suspended(&game_exe, &config.game_dir, &options)?;
    // The patch first and orb's own DLL after it, which is the order that leaves both working: they rewrite
    // some of the same entries of the game's import table — `CreateWindowExA` is one, orb rewriting it to
    // size the window and the patch to read its title as UTF-8 — and whichever goes in second owns the
    // entry. orb's own goes second because orb's rewrite calls through to whatever was there and the
    // patch's does not: the other way round, the window is the one the game asked for and orb's letterbox
    // has no client to measure. Measured, as a launch with no `screen:` line and a game that came up in its
    // own size.
    //
    // Both before the resume, since what either patches is read at startup.
    if let Some(found) = &patch {
        match unsafe {
            let (handle, thread) = process.handles();
            thcrap::inject(found, &game_exe, handle, thread)
        } {
            Ok(()) => println!(
                "orb: thcrap — {} into the game, as {} asks for",
                found.run_config.display(),
                found.wrapper.display(),
            ),
            Err(refused) => println!("orb: thcrap — {refused}; the game is not patched"),
        }
    }
    load_library(&process, &orb_dll)?;
    process.resume()?;

    println!(
        "orb: started {} {} (pid {})",
        game_exe.display(),
        build.version,
        process.id()
    );

    // And the other side of thcrap's own setting: with *install updates after running the game* ticked,
    // this is the moment its updater would have fetched them, and the next launch is the one that plays
    // with them. Last, so that a download nobody is waiting for is not something the game waits for.
    if let Some(found) = &patch {
        say_the_update(unsafe { thcrap::update(found, false) });
    }
    Ok(())
}

/// What a pass over thcrap's patch stack came to, said by the pass that did it: the other one has nothing
/// to report, and a line from both would be one of them saying it did what it left alone.
fn say_the_update(answered: Result<thcrap::Update, String>) {
    match answered {
        Ok(thcrap::Update::Ran { at_exit: false }) => {
            println!("orb: thcrap — its patch stack brought up to date")
        }
        Ok(thcrap::Update::Ran { at_exit: true }) => println!(
            "orb: thcrap — its patch stack brought up to date, which the next launch plays with"
        ),
        Ok(thcrap::Update::Skipped) => {}
        Err(refused) => println!("orb: thcrap — {refused}; the patch is as it was on disk"),
    }
}

/// Which game orb knows is in the directory a launch was pointed at.
///
/// By the exe's own file name, which is what the DLL recognises the process by too — so the game the
/// launcher starts and the game orb attaches to are decided from one table by one name.
///
/// The table's order where a directory holds two of them, there being nothing in a directory to say
/// which was meant. Not an error: a directory with two games in it is somebody's own arrangement, and
/// refusing it would refuse a launch that has an obvious first answer.
fn game_in(dir: &Path, language: Language) -> Result<&'static Known, Box<dyn Error>> {
    game::KNOWN
        .iter()
        .find(|known| dir.join(known.exe).is_file())
        .ok_or_else(|| {
            let dir = dir.display();
            let known = game::known_named();
            // Into a `Box<dyn Error>` from the value and not from a box of it: `Box<Refused>` is itself
            // an `Error`, so boxing first is a box inside a box and the downcast in `main` finds the
            // outer one — which is a dialog quietly reading the line instead of its own words.
            Refused {
                line: format!("no game orb knows is in {dir}; it knows {known}"),
                text: match language {
                    Language::Japanese => format!(
                        "{dir} に orb が知っているゲームがありません。\n\norb が知っているのは \
                         {known} です。ゲームは起動していません。"
                    ),
                    Language::English => format!(
                        "There is no game orb knows in {dir}.\n\nWhat it knows is {known}. The game \
                         has not been started."
                    ),
                },
            }
            .into()
        })
}

/// Which `orb.yaml` a launch reads.
///
/// The one beside the game, because that is the one the DLL reads: the DLL is inside the game and
/// both sides call `load_beside`, so nothing carries a path across to it. `--game-dir` therefore
/// names the file as well as the game — read beside orb.exe instead, every answer the settings
/// dialog wrote went into a file the game never opens, and a size and an ending answered here
/// left it starting on the defaults.
///
/// `--config` names one outright, wherever it is, which is what it is for.
fn config_path(exe: &Path, game_dir: Option<&Path>, named: Option<&Path>) -> PathBuf {
    if let Some(named) = named {
        return named.to_owned();
    }
    game_dir
        .unwrap_or_else(|| exe.parent().unwrap_or(Path::new(".")))
        .join(orb_config::FILE_NAME)
}

/// `orb.dll`, carried inside this exe and written out where the game can be told to load
/// it from.
///
/// Cargo builds the cdylib first and passes its path in, so the two halves are always the
/// same build and there is one file to install rather than two.
const ORB_DLL: &[u8] = include_bytes!(env!("CARGO_CDYLIB_FILE_ORB"));

/// Writes the carried `orb.dll` somewhere the game can load it from, and clears out what
/// earlier runs left.
///
/// Named for this build rather than reused, because a mapped image cannot be replaced while
/// it is loaded — a game still running would otherwise stop the next launch from writing.
/// The leftovers go on the next launch that finds them unloaded.
fn unpack_orb() -> Result<PathBuf, Box<dyn Error>> {
    let dir = std::env::temp_dir().join("orb");
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    for stale in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        // Whatever is still loaded by a running game refuses to go; that is expected.
        let _ = std::fs::remove_file(stale.path());
    }

    let path = dir.join(format!("orb-{}.dll", checksum(ORB_DLL)));
    if path.is_file() && std::fs::read(&path).is_ok_and(|found| found == ORB_DLL) {
        return Ok(path);
    }
    std::fs::write(&path, ORB_DLL)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(path)
}

/// Enough of a fingerprint to tell one build's payload from another's in a file name.
fn checksum(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};
    Md5::digest(bytes)
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The arguments as written, less the launcher's own four: the DLL knows none of them and
/// would refuse the line for carrying a name it cannot read.
///
/// What is handed on is what was typed rather than the parsed options written back out, so that
/// the line the DLL reads is the line somebody wrote and there is no second spelling of an
/// option to keep in step with the first. One form each is enough to drop, since every option
/// takes its value onto itself and any other spelling was refused above — which is also what
/// keeps a path with a space in it out of a line the DLL splits on whitespace.
fn to_hand_on(arguments: impl Iterator<Item = String>) -> Vec<String> {
    const MINE: [&str; 4] = ["--game-dir=", "--orb-dll=", "--config=", "--settings"];
    arguments
        .filter(|argument| !MINE.iter().any(|name| argument.starts_with(name)))
        .collect()
}

/// Which of the builds orb has addresses for this exe is.
///
/// `orb` reads the game's state through absolute addresses, so an exe that is none of them would have
/// it writing into unrelated memory rather than failing — and which one it *is* is worth having, the
/// line a launch prints being where a report of a fault says what was played.
///
/// The refusal names every game and build orb knows rather than the ones this entry holds, because an
/// exe that is none of them is as likely to be another game's under a name orb reads as this game's
/// own next release.
fn verify_game_exe(
    path: &Path,
    known: &Known,
    language: Language,
) -> Result<&'static game::Build, Box<dyn Error>> {
    use md5::{Digest, Md5};

    let bytes =
        std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let digest = Md5::digest(&bytes);
    let digest: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    known
        .builds
        .iter()
        .find(|build| build.md5 == digest)
        .ok_or_else(|| {
            let path = path.display();
            let known = game::known_named();
            Refused {
                line: format!(
                    "{path} is md5 {digest}, which is no build orb knows: it knows {known}"
                ),
                text: match language {
                    Language::Japanese => format!(
                        "{path} の md5 は {digest} で、orb が知っているどのビルドでもありません。\
                         \n\norb が知っているのは {known} です。ゲームは起動していません。"
                    ),
                    Language::English => format!(
                        "{path} is md5 {digest}, which is no build orb knows.\n\nWhat it knows is \
                         {known}. The game has not been started."
                    ),
                },
            }
            .into()
        })
}

fn load_library(process: &inject::Process, dll: &Path) -> Result<(), Box<dyn Error>> {
    if !dll.is_file() {
        return Err(format!("{} does not exist", dll.display()).into());
    }
    // A relative path would be resolved against the game's working directory by
    // the remote LoadLibraryW, which is not where orb.yaml resolved it from.
    let dll = std::path::absolute(dll)?;
    process
        .load_library(&dll)
        .map_err(|error| format!("cannot load {}: {error}", dll.display()).into())
}

#[cfg(test)]
mod tests {
    use super::{Refused, config_path, game_in, no_timer_text, verify_game_exe};
    use orb_config::Language;
    use orb_core::game;
    use std::path::{Path, PathBuf};

    /// The language the refusals below are read in where which one it is changes nothing they say
    /// about themselves.
    const LANGUAGE: Language = Language::English;

    /// Which game a directory holds is its exe being in it, and a directory holding none of them is
    /// refused by a message that names every game and build orb knows — the launcher and the DLL
    /// reading the one table is what makes those the same list.
    #[test]
    fn the_game_started_is_the_one_whose_exe_is_in_the_directory() {
        let dir = std::env::temp_dir().join(format!("orb-game-in-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("a scratch directory");

        let Err(refused) = game_in(&dir, LANGUAGE) else {
            panic!("a directory with no game in it named one");
        };
        let refused = refused.to_string();
        for known in game::KNOWN {
            for build in known.builds {
                assert!(
                    refused.contains(known.exe) && refused.contains(build.version),
                    "{refused:?} does not name {} {}",
                    known.exe,
                    build.version,
                );
            }
        }

        // The exe being there, whatever is in it: which build it is is the md5's to say, and a
        // directory with the file in it is the directory that game is installed in.
        for known in game::KNOWN {
            let exe = dir.join(known.exe);
            std::fs::write(&exe, b"not a game")
                .unwrap_or_else(|error| panic!("{}: {error}", exe.display()));
            let found = game_in(&dir, LANGUAGE).expect("the game whose exe is there");
            assert_eq!(found.exe, known.exe);
            // And the exe in it is none of that game's builds, which is what the launch is refused
            // by: the message names every build orb knows and not the one this entry holds.
            let refused = match verify_game_exe(&exe, found, LANGUAGE) {
                Err(refused) => refused.to_string(),
                Ok(build) => panic!("`not a game` was read as {}", build.version),
            };
            assert!(refused.contains(&game::known_named()), "{refused:?}");
            std::fs::remove_file(&exe).expect("the exe just written");
            assert!(game_in(&dir, LANGUAGE).is_err(), "the exe taken away again");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An exe is accepted whichever of its game's builds it is, and the launch names the one it is:
    /// a translation patch is a second exe the same addresses hold for, so the file's own name no
    /// longer says which of them was played and the line a report is read against has to.
    ///
    /// Driven through an entry made here rather than one of `game::KNOWN`, which holds one build
    /// apiece: what is being asked is that a second one is found, and none of orb's real builds is a
    /// file this test could write.
    #[test]
    fn any_build_of_a_game_is_accepted_and_the_launch_names_it() {
        /// The md5 of a file with nothing in it, which is what the exe here is. A build is picked out
        /// of an entry by the digest of the file and nothing else, so bytes nobody has to keep are
        /// enough to ask with.
        const EMPTY: &str = "d41d8cd98f00b204e9800998ecf8427e";
        let patched = game::Known {
            exe: "東方紅魔郷.exe",
            cfg: "東方紅魔郷.cfg",
            builds: &[
                game::Build {
                    md5: "fa3d64768b1bfc50703dedc2db92f7fa",
                    version: "1.02h",
                },
                game::Build {
                    md5: EMPTY,
                    version: "1.02h with its text swapped out",
                },
            ],
            game: &orb_core::game::th06::Th06,
        };

        let dir = std::env::temp_dir().join(format!("orb-build-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        let exe = dir.join(patched.exe);

        std::fs::write(&exe, b"").expect("an exe of no bytes at all");
        let found =
            verify_game_exe(&exe, &patched, LANGUAGE).expect("the second of the two builds");
        assert_eq!(found.md5, EMPTY);
        assert_eq!(found.version, "1.02h with its text swapped out");

        // And a file that is neither of them is refused, however much the entry it was asked about
        // holds more than one build.
        std::fs::write(&exe, b"not a game").expect("an exe that is no build at all");
        assert!(verify_game_exe(&exe, &patched, LANGUAGE).is_err());

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Every refusal somebody playing can meet says the same thing twice: the line in the English every
    /// log line is in, and the dialog in the language the machine is in.
    ///
    /// Which of the two languages is asserted by what the dialog is *not*: it is not the line, and the
    /// one in the other language is not this one. What each of them says is the wording itself, which
    /// nothing here can hold against anything — the whole of what a test can ask is that the two are
    /// two, and that the language chooses between them.
    #[test]
    fn a_refusal_somebody_playing_can_meet_says_it_in_their_own_language() {
        let dir = std::env::temp_dir().join(format!("orb-refused-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("a scratch directory");

        // The one refusal a directory alone produces, in each language.
        let refusals = |language| {
            let refused = game_in(&dir, language).err().expect("no game in it");
            let refused = refused
                .downcast_ref::<Refused>()
                .expect("a refusal with words of its own");
            (refused.line.clone(), refused.text.clone())
        };
        let (line, japanese) = refusals(Language::Japanese);
        let (again, english) = refusals(Language::English);

        assert_eq!(line, again, "the line changed with the language");
        assert_ne!(japanese, line, "the dialog is the line as it stands");
        assert_ne!(english, line, "the dialog is the line as it stands");
        assert_ne!(japanese, english, "one wording in both languages");
        // And each names the directory nothing was found in, which is the one thing somebody reading it
        // has to be told.
        for text in [&japanese, &english] {
            assert!(
                text.contains(&dir.display().to_string()),
                "{text:?} does not name {}",
                dir.display(),
            );
        }

        // And the timer, which is refused before any file has been read and so in the machine's own
        // language whatever a file says.
        assert_ne!(
            no_timer_text(Language::Japanese),
            no_timer_text(Language::English),
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The file a launch reads is the one the DLL will read, which is the one beside the game:
    /// answers written into any other file are answers the game never sees.
    #[test]
    fn the_config_read_is_the_one_beside_the_game() {
        let exe = PathBuf::from(r"C:\tools\orb.exe");
        assert_eq!(
            config_path(&exe, None, None),
            PathBuf::from(r"C:\tools\orb.yaml"),
        );
        assert_eq!(
            config_path(&exe, Some(Path::new(r"D:\th06")), None),
            PathBuf::from(r"D:\th06\orb.yaml"),
        );
        // And `--config` names one outright, wherever it is.
        assert_eq!(
            config_path(
                &exe,
                Some(Path::new(r"D:\th06")),
                Some(Path::new(r"E:\mine.yaml")),
            ),
            PathBuf::from(r"E:\mine.yaml"),
        );
    }
}
