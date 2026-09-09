//! **`cargo xtask real <copy of the install> [<case>...]`: the real game played to `orb-real`'s cases, and
//! each trace checked.**
//!
//! What an e2e test cannot say is that a laid-out game is the game — see `orb-real`'s own doc — and what says
//! so is the real game driven to the same points. This is the driving: the workspace built, `orb.exe` copied
//! into the copy of the install, and for each case a launch that plays itself to the recipe, is waited on
//! until the game has gone, and leaves a trace for `orb-real` to check at the end.
//!
//! **A copy of the install, never the install** — a played game writes its records beside itself, and a bot
//! in the directory somebody plays in writes over them. The directory is the caller's to name; nothing here
//! guesses one.
//!
//! **Nothing in that directory is deleted.** Every launch keeps orb's own files — the run it leaves, its
//! replay, the trace — in a directory of this session's under `orb-real/`, through `--keep-in`, made fresh
//! for each case that does not continue the one before it, so that a run left by an earlier session answers
//! no question of this one. The settings stay the copy's own `orb.yaml`.
//!
//! **And the game's own records are moved aside so that a case starts from the same ones every time** — see
//! [`Records`]: the score file, the configuration beside it and the file orb forks the first into are the
//! copy's, are read and written by every launch, and are what decides which difficulty a select comes up on,
//! which item the title menu is on and what the game says has been unlocked. They are taken into this
//! session's own directory and put back when it ends, and what a launch writes is taken aside in turn before
//! the next case that starts from nothing.
//!
//! **The game is waited on by its process and not by orb's**: `orb.exe` returns as soon as it has injected,
//! so what makes one launch one run is waiting for the game's exe to appear in the task list and then to
//! leave it. Counted by the rows of `tasklist.exe`'s CSV rather than by name, and asked twice before a nought
//! is believed — the two things the sessions driving this by hand learnt, written down in
//! `.claude/skills/driving-orb-with-autoplay` and now here.
//!
//! **A game that has stopped getting anywhere is not killed.** Every recipe bounds its own waits with
//! `give_up_after`, so a launch ends on its own; where one does not inside the ceiling its case carries —
//! `orb_real::WITHIN`, and an hour and a half for the case that plays a whole game — the session stops and
//! says so, and closing the game is the caller's: a run that looks stuck may be the one somebody is
//! watching, and what it was doing is in `orb.log`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant, SystemTime};

use crate::{TARGET, cargo_command, status};

/// How long a launch is given to bring the game up before the session gives up on it.
const UP_WITHIN: Duration = Duration::from_secs(60);

/// One game `orb-real` knows, as its `list` prints it.
struct Game {
    name: String,
    exe: String,
    /// The records it keeps of its own, by the names `orb_real::Game::records` gives them.
    records: Vec<String>,
    cases: Vec<Case>,
}

/// **The game's own records taken out of the copy, so that a case starts from the same ones every time.**
///
/// `--keep-in` covers the files orb writes and not the game's: its score file, the configuration beside it
/// and the file orb forks the first into stay in the copy, and every launch reads and writes them. Left
/// alone, a case therefore starts from whatever the case before it left — which difficulty a select comes up
/// on, which item the title menu is on, and what the game says has been unlocked are all that — and two
/// recipes were written against one copy's leftovers before this existed.
///
/// **Moved and never deleted, and put back when the session ends.** A copy somebody plays in holds records
/// nothing can regenerate, so what a session does with them is take them aside into its own directory and
/// give them back: [`Drop`] is what gives them back, so a session that stops early on a failed launch gives
/// them back too, and one that is killed leaves them under the path the session printed.
struct Records {
    /// The copy, and where this session keeps what it took out of it.
    copy: PathBuf,
    aside: PathBuf,
    /// The names, which are the game's.
    names: Vec<String>,
}

impl Records {
    /// Takes the copy's own records aside, and says where they went.
    fn taken_aside(copy: &Path, aside: PathBuf, names: &[String]) -> Result<Self, String> {
        let held = Self {
            copy: copy.to_path_buf(),
            aside,
            names: names.to_vec(),
        };
        held.moved(&held.copy, &held.aside)?;
        println!(
            "xtask: the game's own records are aside under {} and go back when the session ends",
            held.aside.display()
        );
        Ok(held)
    }

    /// And the ones a launch wrote taken aside in turn, which is what makes the next case's launch start
    /// from none: a case that continues the one before keeps them instead, the run it picks up and the
    /// unlock it needs being what that case is about.
    fn cleared(&self, into: &Path) -> Result<(), String> {
        self.moved(&self.copy, into)
    }

    /// Every name that is there moved from one directory to the other, the directory made where anything
    /// has to go into it.
    fn moved(&self, from: &Path, to: &Path) -> Result<(), String> {
        for name in &self.names {
            let source = from.join(name);
            if !source.is_file() {
                continue;
            }
            std::fs::create_dir_all(to)
                .map_err(|why| format!("{} cannot be made: {why}", to.display()))?;
            let destination = to.join(name);
            std::fs::rename(&source, &destination).map_err(|why| {
                format!(
                    "{} cannot be moved to {}: {why}",
                    source.display(),
                    destination.display()
                )
            })?;
        }
        Ok(())
    }
}

impl Drop for Records {
    fn drop(&mut self) {
        // **What the last launch wrote is moved aside and not deleted**, which is the same rule as the
        // rest: nothing here removes a file from the copy, and the records of the case that ran last are
        // the session's own evidence. Then the copy's own go back over a directory holding none.
        let written = self.aside.join("the-last-launchs-own");
        if let Err(why) = self.moved(&self.copy.clone(), &written) {
            eprintln!(
                "xtask: what the last launch wrote could not be moved aside: {why}\n\
                 xtask: the copy's own records are under {}",
                self.aside.display()
            );
            return;
        }
        if let Err(why) = self.moved(&self.aside.clone(), &self.copy.clone()) {
            eprintln!(
                "xtask: the game's own records could not be put back: {why}\n\
                 xtask: they are under {}",
                self.aside.display()
            );
            return;
        }
        println!(
            "xtask: the game's own records are back in {}",
            self.copy.display()
        );
    }
}

struct Case {
    name: String,
    continues: bool,
    /// The most this launch may take before the session gives up on it, which is the case's own: every wait
    /// a recipe makes is bounded, so a launch still running past it is one whose step never gave up — and
    /// the case that plays a whole game is an hour and a half where the others are twenty minutes.
    within: Duration,
}

pub fn real(args: Vec<String>) -> ExitCode {
    let mut args = args.into_iter();
    let Some(dir) = args.next() else {
        eprintln!(
            "usage: cargo xtask real <copy of the install> [<case>...]\n\
             \n\
             The directory holds one of the games orb runs and is a copy of the install, never the\n\
             install: the run writes the game's own records beside it. Cases are those of `orb-real`;\n\
             none named plays them all, in order."
        );
        return ExitCode::FAILURE;
    };
    let dir = PathBuf::from(dir);
    let only: Vec<String> = args.collect();

    if !status(
        {
            let mut build = cargo_command();
            build.args(["build", "--release", "--target", TARGET]);
            build
        },
        "build",
    ) {
        return ExitCode::FAILURE;
    }
    let built = target_dir().join(TARGET).join("release");
    let checker = built.join("orb-real.exe");
    let games = match games_known_to(&checker) {
        Ok(games) => games,
        Err(why) => {
            eprintln!("xtask: {why}");
            return ExitCode::FAILURE;
        }
    };
    let Some(game) = games.iter().find(|game| dir.join(&game.exe).is_file()) else {
        eprintln!(
            "xtask: {} holds none of the games orb-real has cases for: {}",
            dir.display(),
            games
                .iter()
                .map(|game| game.exe.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return ExitCode::FAILURE;
    };

    if let Err(why) = std::fs::copy(built.join("orb.exe"), dir.join("orb.exe")) {
        eprintln!(
            "xtask: orb.exe could not be copied into {}: {why}",
            dir.display()
        );
        return ExitCode::FAILURE;
    }

    if running(&game.exe) > 0 {
        eprintln!("xtask: {} is already running; close it first", game.exe);
        return ExitCode::FAILURE;
    }

    let stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let session = dir.join("orb-real").join(stamp.to_string());
    println!("xtask: the session's files go under {}", session.display());

    // The copy's own records out of the way for the whole session, and back at the end of it whichever way
    // the session goes — see [`Records`].
    let records =
        match Records::taken_aside(&dir, session.join("the-copys-own-records"), &game.records) {
            Ok(records) => records,
            Err(why) => {
                eprintln!("xtask: {why}");
                return ExitCode::FAILURE;
            }
        };

    let mut base: Option<PathBuf> = None;
    for case in &game.cases {
        if !only.is_empty() && !only.iter().any(|name| name == &case.name) {
            continue;
        }
        let case_dir = session.join(&case.name);
        let base_dir = match (&base, case.continues) {
            (Some(base), true) => base.clone(),
            _ => case_dir.join("base"),
        };
        if let Err(why) = lay_out(&game.name, case, &case_dir, &base_dir) {
            eprintln!("xtask: {why}");
            return ExitCode::FAILURE;
        }
        // **And the records the launch before wrote out of the way too, where this case starts from none.**
        // A case that continues the one before keeps them: the run it picks up is in orb's own directory,
        // and what the game says has been unlocked is in these.
        if !case.continues
            && let Err(why) = records.cleared(&case_dir.join("the-records-before-it"))
        {
            eprintln!("xtask: {why}");
            return ExitCode::FAILURE;
        }
        base = Some(base_dir.clone());

        println!("xtask: {} {}: launching", game.name, case.name);
        let mut launch = Command::new(dir.join("orb.exe"));
        launch.current_dir(&dir).args([
            format!("--game-dir={}", windows_path(&dir)),
            format!("--keep-in={}", windows_path(&base_dir)),
            format!("--autoplay={}", windows_path(&case_dir.join("recipe.yaml"))),
            format!("--trace={}", windows_path(&case_dir.join("trace.msgpack"))),
            "--log=verbose".to_owned(),
        ]);
        if !status(launch, "orb.exe") {
            return ExitCode::FAILURE;
        }
        match wait_out(&game.exe, case.within) {
            Ok(took) => println!(
                "xtask: {} {}: the run is over after {}s",
                game.name,
                case.name,
                took.as_secs()
            ),
            Err(why) => {
                eprintln!("xtask: {} {}: {why}", game.name, case.name);
                return ExitCode::FAILURE;
            }
        }
    }

    let mut check = Command::new(&checker);
    check.args([windows_path(&session), game.name.clone()]);
    if status(check, "orb-real") {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The case's directory, with the recipe copied out of the tree, and the directory orb keeps its files in.
fn lay_out(game: &str, case: &Case, case_dir: &Path, base_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(case_dir)
        .map_err(|why| format!("{} cannot be made: {why}", case_dir.display()))?;
    std::fs::create_dir_all(base_dir)
        .map_err(|why| format!("{} cannot be made: {why}", base_dir.display()))?;
    let recipe = repo_root()
        .join("crates/orb-real/recipes")
        .join(game)
        .join(format!("{}.yaml", case.name));
    std::fs::copy(&recipe, case_dir.join("recipe.yaml"))
        .map_err(|why| format!("{} cannot be copied: {why}", recipe.display()))?;
    Ok(())
}

/// What `orb-real list` prints, read back: one line per case, `game exe records case continues within`,
/// tab-separated, the records comma-separated among them.
fn games_known_to(checker: &Path) -> Result<Vec<Game>, String> {
    let listed = Command::new(checker)
        .arg("list")
        .output()
        .map_err(|why| format!("{} could not be run: {why}", checker.display()))?;
    if !listed.status.success() {
        return Err(format!(
            "{} list failed: {}",
            checker.display(),
            listed.status
        ));
    }
    let text = String::from_utf8_lossy(&listed.stdout);
    let mut games: Vec<Game> = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [name, exe, records, case, continues, within] = fields[..] else {
            return Err(format!(
                "orb-real list printed a line of another shape: {line:?}"
            ));
        };
        let case = Case {
            name: case.to_owned(),
            continues: continues == "1",
            within: Duration::from_secs(within.parse().map_err(|why| {
                format!("orb-real list printed {within:?} as a count of seconds: {why}")
            })?),
        };
        match games.iter_mut().find(|game| game.name == name) {
            Some(game) => game.cases.push(case),
            None => games.push(Game {
                name: name.to_owned(),
                exe: exe.to_owned(),
                records: records.split(',').map(str::to_owned).collect(),
                cases: vec![case],
            }),
        }
    }
    Ok(games)
}

/// How many processes of the game are up, by the rows of the task list and never by the name: the image name
/// comes back in the console code page, in which a grep for `東方紅魔郷.exe` finds nothing, where the filter
/// handed to `tasklist.exe` is Windows' own and matches. A row starts with a quote, which is the same byte in
/// every code page.
fn running(exe: &str) -> usize {
    let Ok(listed) = Command::new("tasklist.exe")
        .args(["/FI", &format!("IMAGENAME eq {exe}"), "/FO", "CSV", "/NH"])
        .output()
    else {
        return 0;
    };
    listed
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|row| row.first() == Some(&b'"'))
        .count()
}

/// Whether the game has gone — asked twice, two seconds apart, because the task list answered nothing once
/// while the game was up and playing.
fn gone(exe: &str) -> bool {
    if running(exe) > 0 {
        return false;
    }
    std::thread::sleep(Duration::from_secs(2));
    running(exe) == 0
}

/// Waits for the game to come up and then to go, and answers how long that took. `within` is the case's
/// own ceiling.
fn wait_out(exe: &str, within: Duration) -> Result<Duration, String> {
    let started = Instant::now();
    while running(exe) == 0 {
        if started.elapsed() > UP_WITHIN {
            return Err(format!(
                "{exe} never came up within {}s; what orb.exe said is above",
                UP_WITHIN.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    while !gone(exe) {
        if started.elapsed() > within {
            return Err(format!(
                "{exe} is still running after {}s, which is what this case is given; the session stops \
                 here and the game is left to be closed by hand, its log beside it saying what it was \
                 doing",
                within.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    Ok(started.elapsed())
}

/// A path as the Windows side spells it, where this is WSL — `orb.exe` is a Windows process and a path under
/// `/mnt/c` means nothing to it — and as it is everywhere else.
fn windows_path(path: &Path) -> String {
    let as_given = path.display().to_string();
    let Ok(converted) = Command::new("wslpath").args(["-w", &as_given]).output() else {
        return as_given;
    };
    if !converted.status.success() {
        return as_given;
    }
    String::from_utf8_lossy(&converted.stdout).trim().to_owned()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| repo_root().join("target"), PathBuf::from)
}
