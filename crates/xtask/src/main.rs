//! The commands that need a target named, run through the `xtask` alias in `.cargo/config.toml`.
//!
//! **The target lives here and not in `[build] target`**, which is what this crate is for. A
//! `build.target` applies to every package in the workspace, this one included, and a task runner built
//! for a 32-bit Windows target is an executable that cannot spawn the `cargo` that built it wherever the
//! host is not Windows. So the target moved to where the builds are asked for — and to *one* place rather
//! than to the hook, the workflow and the README each keeping a copy of a triple to hold in step.
//!
//! What is not here is `cargo fmt --check`: it needs no target, so the hook and the workflow run it
//! themselves and it stays readable where it is asked for.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// What everything but this crate is built for. The game is a 32-bit process, so both halves target it.
const TARGET: &str = "i686-pc-windows-gnu";

/// The target the coverage run is measured on, which is **not** the one everything else is built for.
///
/// rustup ships `profiler_builtins` for this target and not for `i686-pc-windows-gnu`, so
/// `-C instrument-coverage` has no runtime to link against on the one that ships. The ABI, the CRT and
/// every `cfg` are the same — the linker and the unwinder are what differ — so what a run measures is
/// the code that ships. See
/// [docs/todo/what-the-scenarios-never-enter.md](../../../docs/todo/what-the-scenarios-never-enter.md).
const COVERAGE_TARGET: &str = "i686-pc-windows-gnullvm";

/// Where llvm-mingw is, as the environment names it. Not a default and not a search: where a toolchain
/// is installed is one machine's business, which is the rule for everything else in this repository that
/// varies.
const LLVM_MINGW: &str = "LLVM_MINGW";

/// Which package's tests the coverage run drives, and why it is only one.
///
/// The scenarios are `orb-sim`'s tests, and what a scenario covers is the question — a line only a unit
/// test reaches is a line no scenario stands behind. `orb-launcher` is left out for a harder reason: its
/// artifact dependency pins `orb`'s cdylib to `i686-pc-windows-gnu`, which has no profiler runtime, so a
/// run that includes it does not compile.
const COVERED: &str = "orb-sim";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let task = args.next();
    let rest: Vec<String> = args.collect();
    match task.as_deref() {
        Some("clippy") => plain(["clippy", "--all-targets"], ["-D", "warnings"], rest),
        Some("test") => plain(["test"], [], rest),
        Some("build") => plain(["build"], [], rest),
        Some("coverage") => coverage(rest),
        Some(task) => {
            eprintln!("xtask: no task called {task}");
            usage()
        }
        None => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: cargo xtask <task> [<args>]\n\
         \n\
         clippy    every target, {TARGET}, warnings denied\n\
         test      the suite, {TARGET}\n\
         build     {TARGET} — `build --release` is what installs\n\
         coverage  runs {COVERED}'s tests instrumented and reports what they cover;\n\
         \x20         --missing lists the line ranges nothing ran, which is what a\n\
         \x20         worklist is read off, and --html writes the browsable report\n\
         \n\
         `cargo fmt --check` is not a task here: it needs no target.\n\
         \n\
         coverage wants {LLVM_MINGW} naming an llvm-mingw installation, its target being\n\
         {COVERAGE_TARGET}, whose linker is that toolchain's i686-w64-mingw32-clang.\n\
         Releases are at https://github.com/mstorsjo/llvm-mingw/releases — the msvcrt\n\
         build, to match the CRT the shipping target uses."
    );
    ExitCode::FAILURE
}

/// A cargo command for [`TARGET`]: `before` is what goes in front of the target, `after` what goes past
/// the `--` cargo forwards, and `args` is whatever the caller added.
fn plain<const B: usize, const A: usize>(
    before: [&str; B],
    after: [&str; A],
    args: Vec<String>,
) -> ExitCode {
    let mut command = Command::new(cargo_bin());
    command
        .args(before)
        .args(["--target", TARGET])
        .args(&args)
        .args(if after.is_empty() { &[][..] } else { &["--"] })
        .args(after);
    if status(command, before[0]) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn coverage(args: Vec<String>) -> ExitCode {
    let Some(clang) = linker() else {
        return usage();
    };
    // How the report is asked for, apart from what is passed on to `cargo test`: a summary of every
    // file, the line ranges nothing ran — which is what a worklist is read off, `--summary-only` giving
    // only how much — or the browsable one.
    let (asked, rest): (Vec<String>, Vec<String>) = args
        .into_iter()
        .partition(|arg| matches!(arg.as_str(), "--html" | "--missing"));

    // Two invocations rather than one, because the report is wanted over the whole workspace while only
    // one package's tests are run: `cargo llvm-cov test -p orb-sim` would narrow the report to that
    // package as well, and orb-core — the thing the answer is about — would not be in it.
    let mut run = cargo(&clang);
    run.args([
        "llvm-cov",
        "--no-report",
        // `cfg(coverage)` is not something this tree reads, and a build that sets it is a different
        // build from the one being measured.
        "--no-cfg-coverage",
        "--no-cfg-coverage-nightly",
        "--target",
        COVERAGE_TARGET,
        "-p",
        COVERED,
        "test",
    ])
    .args(&rest);
    if !status(run, "the instrumented tests") {
        return ExitCode::FAILURE;
    }

    let mut report = cargo(&clang);
    report.args(["llvm-cov", "report", "--target", COVERAGE_TARGET]);
    let html = asked.iter().any(|arg| arg == "--html");
    match asked.iter().any(|arg| arg == "--missing") {
        // Every file's uncovered line ranges beside its numbers. `--summary-only` is what the flag
        // turns off, so the two cannot be asked for together.
        true => report.arg("--show-missing-lines"),
        false if html => report.arg("--html"),
        false => report.arg("--summary-only"),
    };
    if !status(report, "the report") {
        return ExitCode::FAILURE;
    }
    if html {
        println!("the report is in target/llvm-cov/html/index.html");
    }
    ExitCode::SUCCESS
}

/// The cargo that is running this, so a task uses the toolchain the caller did.
fn cargo_bin() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

/// `cargo`, with what the coverage build needs that no config file can carry.
fn cargo(clang: &Path) -> Command {
    let mut cargo = Command::new(cargo_bin());
    cargo.env("CARGO_TARGET_I686_PC_WINDOWS_GNULLVM_LINKER", clang);
    if let Some(wslenv) = wslenv() {
        cargo.env("WSLENV", wslenv);
    }
    cargo
}

/// What `WSLENV` has to say so that the test binaries can write their profiles, and `None` anywhere but
/// inside WSL.
///
/// **Only under WSL, and added to whatever is already there.** The test binaries are Windows executables
/// run through WSL interop and the profile path they are handed is a Linux one, which they cannot write —
/// without the translation they write nothing at all and the report says only that it found no profiles.
/// On a host that is not WSL there is no boundary to translate across, and a `WSLENV` set for something
/// else is somebody's working setup to leave alone.
fn wslenv() -> Option<std::ffi::OsString> {
    let inside_wsl =
        std::env::var_os("WSL_INTEROP").is_some() || std::env::var_os("WSL_DISTRO_NAME").is_some();
    if !inside_wsl {
        return None;
    }
    const OURS: &str = "LLVM_PROFILE_FILE/p";
    match std::env::var("WSLENV") {
        Ok(theirs) if theirs.split(':').any(|entry| entry == OURS) => Some(theirs.into()),
        Ok(theirs) if !theirs.is_empty() => Some(format!("{theirs}:{OURS}").into()),
        _ => Some(OURS.into()),
    }
}

/// The linker, out of [`LLVM_MINGW`], checked before it is used: a path that is not there is a usage
/// line and not a link error two minutes in.
///
/// Both names, because llvm-mingw's own releases carry the one and its Windows releases the other, and
/// which host this is running on is not something to make the caller say.
fn linker() -> Option<PathBuf> {
    let bin = PathBuf::from(std::env::var_os(LLVM_MINGW)?).join("bin");
    let names = ["i686-w64-mingw32-clang", "i686-w64-mingw32-clang.exe"];
    if let Some(clang) = names
        .iter()
        .map(|name| bin.join(name))
        .find(|at| at.is_file())
    {
        return Some(clang);
    }
    eprintln!(
        "xtask: neither {} nor {} is there",
        bin.join(names[0]).display(),
        names[1],
    );
    None
}

fn status(mut command: Command, what: &str) -> bool {
    match command.status() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("xtask: {what} failed: {status}");
            false
        }
        Err(error) => {
            eprintln!("xtask: {what} could not be run: {error}");
            false
        }
    }
}
