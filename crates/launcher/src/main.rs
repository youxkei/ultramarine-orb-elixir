//! Starts 東方紅魔郷 with `orb.dll` loaded before the game's entry point runs.
//!
//! The DLL is carried inside this exe and written out to be loaded, so installing orb is one
//! file.

mod inject;
mod pad;
mod settings;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use orb_config::Config;
use orb_config::args::Options;

/// Only 1.02h is supported: every address `orb` uses was read off this build.
const GAME_EXE: &str = "東方紅魔郷.exe";
const GAME_EXE_MD5: &str = "fa3d64768b1bfc50703dedc2db92f7fa";
/// What the game keeps its own configuration in, read for one thing only: which pad button it takes
/// as shoot and which as bomb, so that the settings dialog answers to the same two.
const GAME_CFG: &str = "東方紅魔郷.cfg";

/// Starts 東方紅魔郷 1.02h with orb loaded, and hands the options below on to it.
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

    /// where 東方紅魔郷.exe is, if it is not beside orb.exe
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("orb: {error}");
            ExitCode::FAILURE
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

    // Before the options are applied, so that what is written back is the settings and not one
    // launch's arguments — and before the game starts, since the DLL reads the same file from
    // inside it.
    if config.ask_at_startup || launch.settings {
        let game_cfg = config.game_dir.join(GAME_CFG);
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

    let game_exe = config.game_dir.join(GAME_EXE);
    verify_game_exe(&game_exe)?;

    // Written out before the game starts, because a `LoadLibrary` needs a path. Kept out
    // of the game's directory: nothing there is orb's to leave behind, and this file is a
    // detail of how the launcher carries its payload rather than something to install.
    let orb_dll = match launch.orb_dll {
        Some(path) => path,
        None => unpack_orb()?,
    };

    // Handed on to the game, whose command line is where the DLL reads them back from: the
    // game itself never looks at it. Nothing is written down and the two sides cannot
    // disagree about which pass this is.
    let mut process = inject::spawn_suspended(&game_exe, &config.game_dir, &options)?;
    load_library(&process, &orb_dll)?;
    process.resume()?;

    println!("orb: started {} (pid {})", game_exe.display(), process.id());
    Ok(())
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

/// `orb` reads the game's state through absolute addresses, so a different
/// build would have it writing into unrelated memory rather than failing.
fn verify_game_exe(path: &Path) -> Result<(), Box<dyn Error>> {
    use md5::{Digest, Md5};

    let bytes =
        std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let digest = Md5::digest(&bytes);
    let digest: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    if digest != GAME_EXE_MD5 {
        return Err(format!(
            "{} is md5 {digest}, but orb only supports 1.02h (md5 {GAME_EXE_MD5})",
            path.display()
        )
        .into());
    }
    Ok(())
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
    use super::config_path;
    use std::path::{Path, PathBuf};

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
