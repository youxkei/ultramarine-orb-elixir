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

mod real;

/// What everything but this crate is built for. The game is a 32-bit process, so both halves target it.
const TARGET: &str = "i686-pc-windows-gnu";

/// The host with no Windows on it that the crates above the seam are checked against, which is what makes
/// the boundary self-enforcing rather than kept by hand.
///
/// The rule is that **the code an e2e test drives cannot reach Windows except through the seam**.
/// A host that has no Windows is the only thing that can say so: a grep for `windows-sys` passes a file
/// that is calling a COM vtable, and a build for [`TARGET`] passes anything at all.
///
/// **A 32-bit target and not a 64-bit one**, which is the part that would otherwise be read as this
/// boundary being wrong. Measured:
///
/// ```text
/// $ cargo check -p orb-core --target x86_64-unknown-linux-gnu
/// error[E0570]: "thiscall" is not a supported ABI for the current target
///     --> crates/orb-core/src/game/th06/mod.rs:1977:32
/// ```
///
/// What raises it are the `handed_over!` calls, which transmute to `extern "thiscall"` — a convention MSVC6
/// compiled the game's own methods with, and one that exists on x86 and nowhere else. So a 64-bit check
/// would fail on arrival over the ABI while saying nothing about the host.
const SEAM_TARGET: &str = "i686-unknown-linux-gnu";

/// Which packages that check covers: orb's logic, and the simulated Windows it is tested against.
///
/// The two the rule is about. `orb` is the injection and may name Windows in every file of it; `orb-e2e`
/// is a game playing the game's part, which is allowed to be a real object where a laid-out one will not
/// do. What is left is the code under test and the host it is tested against, and neither may reach past
/// `orb-api`.
const ABOVE_THE_SEAM: [&str; 2] = ["orb-core", "orb-sim"];

/// The target the coverage run is measured on, which is **not** the one everything else is built for.
///
/// rustup ships `profiler_builtins` for this target and not for `i686-pc-windows-gnu`, so
/// `-C instrument-coverage` has no runtime to link against on the one that ships. The ABI, the CRT and
/// every `cfg` are the same — the linker and the unwinder are what differ — so what a run measures is
/// the code that ships. `.cargo/config.toml` carries the `crt-static` this one needs and says why.
///
/// **Measuring on [`TARGET`] itself was tried and does not work**, which is here rather than in a
/// document because this constant is the thing that tempts somebody back to it.
/// `-Z build-std=std,panic_abort,profiler_builtins` does build the runtime from compiler-rt's sources,
/// and the binaries then link and run — but the profile is unreadable. compiler-rt takes the bounds of
/// its data, names and counters from one-element sentinel variables in `$A`/`$Z` subsections, which
/// assumes MSVC's chunk layout where the real data begins immediately after the sentinel. With mingw's
/// gcc and lld every boundary is padded instead: measured **names +4, counters +8 and data +0x40**,
/// against the `+1` and `+sizeof` the runtime computes. `llvm-profdata` rejects every file — `symbol
/// name is empty`, then `counter offset is negative` — and `__attribute__((aligned(1)))` on the
/// sentinels removes none of it. Two of the three paddings can be justified from first principles and
/// the third cannot, which is where that road ends: numbers nobody can vouch for are worse than no
/// numbers.
///
/// And with GNU ld rather than lld it does not get that far at all — `.lcovfun` and `.lcovmap` land at
/// VMA 0, below the image base, and the PE will not load. Ubuntu's mingw gcc cannot be made to use lld:
/// it resolves `/usr/bin/i686-w64-mingw32-ld` absolutely, and `-B`, `COMPILER_PATH` and `PATH` change
/// none of it — `-B` does not even redirect `as`.
///
const COVERAGE_TARGET: &str = "i686-pc-windows-gnullvm";

/// Where llvm-mingw is, as the environment names it. Not a default and not a search: where a toolchain
/// is installed is one machine's business, which is the rule for everything else in this repository that
/// varies.
///
/// **What it has to name is a directory with `bin/i686-w64-mingw32-clang` under it, out of the msvcrt
/// release** — the CRT [`COVERAGE_TARGET`] links against. Unpacking that release somewhere is the whole of
/// it: nothing is installed, nothing goes on `PATH`, and where it is unpacked is this machine's business
/// like everything else here that varies.
///
/// One host's way of doing it, as an example and not the way: with nix,
/// `nix-prefetch-url --unpack --print-path <the msvcrt release>` hands back a store path in that shape and
/// leaves nothing behind — and a store path is not a GC root, so it goes at the next collection and the
/// command is how it comes back. The trap there is worth one line, being the first thing anybody with nix
/// finds: nixpkgs' own `llvm-mingw`, in `pkgs/applications/emulators/wine/llvm-mingw.nix`, fetches the
/// *ucrt* release rather than the msvcrt one, and is a `let` binding inside wine's `packages.nix` rather
/// than an attribute, so there is nothing to build by name either. What it has that the release itself has
/// not is `autoPatchelfHook`, which a NixOS host needs to run a prebuilt binary at all and no other host
/// does.
const LLVM_MINGW: &str = "LLVM_MINGW";

/// Which package's tests the coverage run drives, and why it is only one.
///
/// That package holds the e2e tests, and what one of those covers is the question — a line only a unit
/// test reaches is a line no e2e test stands behind. Which is why it is not `orb-sim`: what is left in
/// that package's `tests/` is the handful no game drives — `log_writes`, `log_off_thread`,
/// `log_written_off_the_frame`, `log_refused` and the two `pacing_no_timer` halves — so a run there answers
/// a question about those. `orb-launcher` is left
/// out for a harder reason: its artifact dependency pins `orb`'s cdylib to `i686-pc-windows-gnu`, which
/// has no profiler runtime, so a run that includes it does not compile.
///
/// **Read the missed lines; ignore the percentage**, which is what a report of this is for. Coverage
/// counts what was *executed* and not what was *asserted*, and in this suite the two come apart in one
/// direction: `Th06::read_state` runs on every frame of every e2e test, so all of it reads as covered while
/// only some of its fields are asserted anywhere. A percentage driven upwards buys nothing. What a missed
/// line says is exact, though — nothing in the suite has ever run it, so no e2e test can be relying on it —
/// and `--missing` is how to find them. Where that flag hands back no ranges, `cargo llvm-cov report
/// --lcov` and the `DA:` records whose hit count is zero are the same answer.
///
/// **Every line and function a run reports as missed is accounted for beside the code it is in**, which is
/// where a reader meets it rather than in a list of its own. Four kinds of them:
///
/// - **const-evaluated in a static**, so what happens is not execution: `game::proposed` and `game::hand`
///   in the baked table, `sync::MainThread::new`, `resume::Record::new`, `runtime::FrameCall::none`.
/// - **the patched bytes**, which a laid-out game has none of: `Th06::hooks` and `Th06::frame_calls`, which
///   only `orb::attach` calls, and `joystick::install`, the one write over an import table entry.
/// - **what a simulated host answers with nothing**: `snapshot::overlaps` and `snapshot::hash`, which
///   `fingerprint_untracked` would call per private range and `orb_sim` reports none of; every arm of
///   `audio.rs` that gives up, `orb_sim::Sound` answering every call; and `REPORT_READS`, which wants a
///   second of wall clock.
/// - **what no game reaches at all**: `Pacing::no_timer`, on a host orb does not run on; `note_reentry`,
///   for a chain walk re-entering itself; `Judgement::Out`, whose key was taken away; and the six `Default`
///   impls that exist so `new` may.
///
/// Which is what a run is worth reading for: a missed line with no such note beside it is either work or a
/// note somebody has not written. `game/th07` is the one file to read differently — it is at 20.8% because
/// `orb-e2e`'s `th07` asserts orb does nothing to a game that declines everything, so raising it changes
/// what is measured rather than what is known.
const COVERED: &str = "orb-e2e";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let task = args.next();
    let rest: Vec<String> = args.collect();
    match task.as_deref() {
        Some("clippy") => plain(
            ["clippy", "--all-targets"],
            TARGET,
            ["-D", "warnings"],
            rest,
        ),
        Some("test") => test(rest),
        Some("build") => plain(["build"], TARGET, [], rest),
        Some("seam") => seam(rest),
        Some("coverage") => coverage(rest),
        Some("real") => real::real(rest),
        Some(task) => {
            eprintln!("xtask: no task called {task}");
            usage()
        }
        None => usage(),
    }
}

fn usage() -> ExitCode {
    let above_the_seam = ABOVE_THE_SEAM.join(" and ");
    let also = ALSO.0;
    eprintln!(
        "usage: cargo xtask <task> [<args>]\n\
         \n\
         clippy    every target, {TARGET}, warnings denied\n\
         test      the suite, {TARGET}, on half the threads — and {also} first, which\n\
         \x20         is where the game built for it has its own tests\n\
         build     {TARGET} — `build --release` is what installs\n\
         seam      {above_the_seam} checked for {SEAM_TARGET}, a host with no Windows\n\
         \x20         on it, which is what says nothing above the seam reaches past it\n\
         coverage  runs {COVERED}'s tests instrumented and reports what they cover;\n\
         \x20         --missing lists the line ranges nothing ran, which is what a\n\
         \x20         worklist is read off, and --html writes the browsable report\n\
         real      the real game in a copy of the install played to orb-real's cases, one\n\
         \x20         launch each, and every trace checked: `real <copy> [<case>...]`\n\
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

/// A cargo command for one target: `before` is what goes in front of the target, `after` what goes past
/// the `--` cargo forwards, and `args` is whatever the caller added.
fn plain<const B: usize, const A: usize>(
    before: [&str; B],
    target: &str,
    after: [&str; A],
    args: Vec<String>,
) -> ExitCode {
    let mut command = cargo_command();
    command
        .args(before)
        .args(["--target", target])
        .args(&args)
        .args(if after.is_empty() { &[][..] } else { &["--"] })
        .args(after);
    if status(command, before[0]) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The suite, on **half the machine's threads** rather than one per core.
///
/// Every e2e test starts a process of its own — see `orb-e2e`'s `in_its_own_process` — and that child runs
/// the test while the parent waits on it, so a slot costs two processes rather than one thread. The
/// harness's own default of one per core therefore asks a sixteen-core machine for sixteen launches and
/// thirty-two processes, which is the machine gone for as long as the run takes.
///
/// **Half rather than fewer, because that is where the measurement puts the knee.** On the sixteen-core host
/// this was written on, the whole suite takes 47s on sixteen threads, 50s on eight and 73s on four: half the
/// load for three seconds, and a quarter of it for another twenty-three. The wall clock is set by the slowest
/// launches — a pacing e2e test runs thousands of simulated frames whatever else is happening — until there
/// are too few slots to keep them overlapping, which is what four does.
///
/// Half, and never less than one. `cargo xtask test -- --test-threads=N` asks for another number: an
/// argument already there is left alone, and the harness would read the last one anyway.
///
/// **Said as an argument and not as `RUST_TEST_THREADS`, because on WSL that variable never arrives.** The
/// tests are Windows binaries and cargo is a Linux one, so every test binary is started through WSL's
/// interop — and interop passes a Linux process's environment to a Win32 one only for the variables named in
/// `WSLENV`. `WSLENV` is empty here, so a `RUST_TEST_THREADS` set on this side reaches nothing: measured with
/// `RUST_TEST_THREADS=4 cmd.exe /c 'echo [%RUST_TEST_THREADS%]'`, which prints the name back unexpanded. The
/// setting was in this function as an environment variable first and did nothing at all — sixteen threads and
/// thirty-two processes every run — which is the whole reason this note is here.
/// **The other target the suite is run for, and the one package that has anything there.**
///
/// `TARGET` is 32-bit and th06c is a 64-bit build, so its module is gated to `x86_64` and everything it
/// asserts about that exe — the addresses, the keyboard, the scale each of its bullet states divides a step
/// by — is asserted by tests nothing was running. Only `orb-core` carries a game, so only `orb-core` is run
/// again: `orb-e2e` lays out th06 and th07, which are this `TARGET`'s, and a second run of it would be the
/// same suite twice.
///
/// **And it is `orb-core`'s own test modules that made this impossible until now.** Five of them name
/// `game::th06` or `game::th07` outright, which a 64-bit build has neither of, so the run did not compile
/// at all; each is gated to `target_arch = "x86"` with a line pointing here.
const ALSO: (&str, &str) = ("x86_64-pc-windows-gnu", "orb-core");

fn test(args: Vec<String>) -> ExitCode {
    let mut also = cargo_command();
    also.args(["test", "--target", ALSO.0, "-p", ALSO.1]);
    if !status(also, "test") {
        return ExitCode::FAILURE;
    }
    let mut command = cargo_command();
    command.args(["test", "--target", TARGET]).args(&args);
    if !args.iter().any(|arg| arg.starts_with("--test-threads")) {
        let threads =
            std::thread::available_parallelism().map_or(1, |cores| (cores.get() / 2).max(1));
        // The separator only where the caller has not already opened one: cargo takes it once, and a second
        // `--` would be an argument the harness reads as a filter.
        if !args.iter().any(|arg| arg == "--") {
            command.arg("--");
        }
        command.arg(format!("--test-threads={threads}"));
    }
    if status(command, "test") {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// [`ABOVE_THE_SEAM`] checked for [`SEAM_TARGET`], which is the boundary asked about rather than
/// remembered.
///
/// A `check` and not a `build`: what is being asked is whether the source names anything the seam does
/// not carry, and there is nothing to run at the end of it. The two packages in one invocation because
/// what fails is a name and not a link, so the first error is the answer wherever it is.
fn seam(args: Vec<String>) -> ExitCode {
    let mut command = cargo_command();
    command
        .arg("check")
        .args(ABOVE_THE_SEAM.iter().flat_map(|package| ["-p", package]))
        .args(["--target", SEAM_TARGET])
        .args(&args);
    if status(command, "the check above the seam") {
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
    // one package's tests are run: `cargo llvm-cov test -p orb-e2e` would narrow the report to that
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

    // **A file whose line count moved without its source moving is `cargo llvm-cov clean --workspace`
    // away**, and that is the whole of how this shows itself: the report is read off every object under
    // `target/llvm-cov-target`, and one built before the last change to a `Cargo.toml` is still an object.
    // Two builds of one crate then both land in the profile, and llvm-cov groups them by *source path* —
    // so a file's own functions are counted twice under two crate disambiguators, in **one row**, with its
    // total and its missed inflated together.
    //
    // Which is why it is worth saying: it looks like a file that got worse rather than like a fault.
    // Watched here — `orb-config/sim` was added to `orb-core`'s features, which changed orb-core's metadata
    // hash, and `resume.rs` read **559 lines with 64 missed** against the 405 and 47 a clean run gives.
    // A path renamed out from under a build does it too; this tree's directory was `ultramarine_orb_elixir`
    // once.
    //
    // Not cleaned here, because the clean throws away the whole instrumented build and every run of this
    // would then pay for one. What to do instead is clean once after a manifest changes.
    let mut report = cargo(&clang);
    report.args(["llvm-cov", "report", "--target", COVERAGE_TARGET]);
    let html = asked.iter().any(|arg| arg == "--html");
    match asked.iter().any(|arg| arg == "--missing") {
        // Every file's uncovered line ranges beside its numbers. `--summary-only` is what the flag
        // turns off, so the two cannot be asked for together.
        //
        // **And it may print no ranges at all**: on the `cargo-llvm-cov` the last run was made with, the
        // report came out with exactly the columns it has without this flag. `cargo llvm-cov report
        // --lcov` and the `DA:` records whose hit count is zero are the same answer and one step more, so
        // a range that will not come out here is not a reason to go looking for a version of anything.
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

/// The environment variable the commit goes in, which `orb` reads with `option_env!` as it compiles.
const BUILT_FROM: &str = "ORB_BUILT_FROM";

/// That cargo with the commit the build is being made from in its environment — see [`built_from`].
fn cargo_command() -> Command {
    let mut cargo = Command::new(cargo_bin());
    if let Some(commit) = built_from() {
        cargo.env(BUILT_FROM, commit);
    }
    cargo
}

/// **Which commit a build is being made from, so that a launch can say which build it is.**
///
/// The version cannot: `0.11.0-dev` is what every build between two releases says, and a launch driven
/// against an exe deployed before a fix said exactly the same line as one driven against the exe with it —
/// which is a measurement of the build nobody meant to measure, and there was nothing in the log to tell the
/// two apart.
///
/// **jj's working-copy commit, whose id changes with the tree.** `@` *is* the working copy here, so its id
/// covers work that is not committed anywhere: two trees that differ by an edit are two commits, and the id
/// therefore answers "is this the build with the fix" rather than "which release is this". Colocated with
/// git, so what it prints is the git hash of that commit.
///
/// **The read snapshots the working copy**, which is what makes the answer the tree being built rather than
/// the tree as of somebody's last jj command. That is a side effect of asking, and asking is what a build is
/// allowed to do: `--ignore-working-copy` would answer faster and stale, which is the failure this exists to
/// stop.
///
/// `None` where jj cannot answer — a tree with no repository around it, or no jj on the machine — and a
/// launch of such a build says so rather than naming a commit it cannot vouch for.
fn built_from() -> Option<String> {
    let asked = Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", "commit_id.short(12)"])
        .output()
        .ok()?;
    asked.status.success().then_some(())?;
    the_commit_in(&asked.stdout)
}

/// The commit out of what `jj log` printed, or `None` for anything that is not one.
fn the_commit_in(printed: &[u8]) -> Option<String> {
    let text = String::from_utf8(printed.to_vec()).ok()?;
    let commit = text.trim();
    let hexadecimal = !commit.is_empty() && commit.chars().all(|digit| digit.is_ascii_hexdigit());
    hexadecimal.then(|| commit.to_owned())
}

/// `cargo`, with what the coverage build needs that no config file can carry.
fn cargo(clang: &Path) -> Command {
    let mut cargo = cargo_command();
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

#[cfg(test)]
mod tests {
    use super::the_commit_in;

    /// The commit is taken out of what jj printed, and anything that is not one leaves the build unstamped.
    ///
    /// **Unstamped rather than stamped with whatever came back**, which is the point of reading it at all: a
    /// launch that names a commit is a launch somebody will hold a tree against, so a line that names an
    /// error message or half a hash is worse than a line saying there is no commit to name.
    #[test]
    fn the_commit_is_what_jj_printed_and_nothing_else_is() {
        assert_eq!(
            the_commit_in(b"7c4f1a2b3c4d\n").as_deref(),
            Some("7c4f1a2b3c4d"),
        );
        // What `--no-graph` leaves is the template's own output, which carries no newline of its own where
        // the template does not add one.
        assert_eq!(
            the_commit_in(b"7c4f1a2b3c4d").as_deref(),
            Some("7c4f1a2b3c4d")
        );
        assert_eq!(
            the_commit_in(b"  7c4f1a2b3c4d \n").as_deref(),
            Some("7c4f1a2b3c4d")
        );
        assert_eq!(the_commit_in(b"").as_deref(), None);
        assert_eq!(the_commit_in(b"\n").as_deref(), None);
        // A message where a hash was asked for: jj says what it cannot do on stdout as readily as on stderr,
        // and `Error: ...` is not a commit.
        assert_eq!(
            the_commit_in(b"Error: There is no jj repo in \".\"\n").as_deref(),
            None
        );
        // And bytes that are not text at all, which is what a pipe carrying something else looks like.
        assert_eq!(the_commit_in(&[0xff, 0xfe, 0x00]).as_deref(), None);
    }
}
