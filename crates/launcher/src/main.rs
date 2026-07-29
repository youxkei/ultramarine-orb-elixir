//! Starts 東方紅魔郷 with `vpatch_th06.dll` and `orb.dll` loaded before the
//! game's entry point runs.

mod inject;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use orb_config::Config;

/// Only 1.02h is supported: every address `orb` uses was read off this build.
const GAME_EXE: &str = "東方紅魔郷.exe";
const GAME_EXE_MD5: &str = "fa3d64768b1bfc50703dedc2db92f7fa";

const USAGE: &str = "\
usage: orb-launcher [--config <path>]

  --config <path>  orb.yaml to use (default: beside orb-launcher.exe)
";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("orb-launcher: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let Some(config_path) = config_path()? else {
        print!("{USAGE}");
        return Ok(());
    };
    let config = Config::load(&config_path)?;

    let game_exe = config.game_dir.join(GAME_EXE);
    verify_game_exe(&game_exe)?;

    // Written out before the game starts, because a `LoadLibrary` needs a path. Kept out
    // of the game's directory: nothing there is orb's to leave behind, and this file is a
    // detail of how the launcher carries its payload rather than something to install.
    let orb_dll = match &config.orb_dll {
        Some(path) => path.clone(),
        None => unpack_orb()?,
    };

    let process = inject::spawn_suspended(&game_exe, &config.game_dir)?;
    if let Some(vpatch_dll) = &config.vpatch_dll {
        load_library(&process, vpatch_dll)?;
    }
    load_library(&process, &orb_dll)?;
    process.resume()?;

    println!("orb-launcher: started {} (pid {})", game_exe.display(), process.id());
    Ok(())
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
    Md5::digest(bytes).iter().take(8).map(|byte| format!("{byte:02x}")).collect()
}

fn config_path() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let mut path = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--config") => {
                path = Some(PathBuf::from(
                    args.next().ok_or("--config needs a path")?,
                ));
            }
            Some("--help" | "-h") => return Ok(None),
            _ => return Err(format!("unexpected argument {arg:?}\n\n{USAGE}").into()),
        }
    }
    match path {
        Some(path) => Ok(Some(path)),
        None => {
            let exe = std::env::current_exe()?;
            let dir = exe.parent().ok_or("cannot locate orb-launcher.exe directory")?;
            Ok(Some(dir.join(orb_config::FILE_NAME)))
        }
    }
}

/// `orb` reads the game's state through absolute addresses, so a different
/// build would have it writing into unrelated memory rather than failing.
fn verify_game_exe(path: &Path) -> Result<(), Box<dyn Error>> {
    use md5::{Digest, Md5};

    let bytes = std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
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
