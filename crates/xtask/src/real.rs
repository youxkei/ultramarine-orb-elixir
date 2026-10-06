//! **`cargo xtask real <copy of the install> [<case>...]`: the real game played to `orb-real`'s cases, and
//! each trace checked.**
//!
//! What an e2e test cannot say is that a laid-out game is the game — see `orb-real`'s own doc — and what says
//! so is the real game driven to the same points. This is the driving: the workspace built, `orb.exe` copied
//! into the copy of the install, and for each case a launch that plays itself to the recipe, is waited on
//! until the game has gone, and leaves a trace for `orb-real` to check at the end.
//!
//! **And built twice where the game is not this task's own architecture.** The checker is read on
//! [`TARGET`] whatever the game is — a trace is the same file either way — but the exe installed beside the
//! game has to be the architecture the game is: see `orb_real::Game::target`, and `orb_core::game::KNOWN`
//! for why a build of orb carries the games of one architecture and no others.
//!
//! **A copy of the install, never the install** — a played game writes its records beside itself, and a bot
//! in the directory somebody plays in writes over them. The directory is the caller's to name; nothing here
//! guesses one.
//!
//! **Nothing in that directory is deleted.** Every launch keeps orb's own files — the run it leaves, its
//! replay, the trace — in a directory of this session's under `orb-real/`, through `--keep-in`, made fresh
//! for each case, so that a run left by an earlier session or case answers no question of this one. The
//! settings stay the copy's own `orb.yaml`.
//!
//! **And the game's own records are moved aside so that a case starts from the same ones every time** — see
//! [`Records`]: the score file, the configuration beside it and the file orb forks the first into are the
//! copy's, are read and written by every launch, and are what decides which difficulty a select comes up on,
//! which item the title menu is on and what the game says has been unlocked. They are taken into this
//! session's own directory and put back when it ends, and what a case's launches write is taken aside in turn
//! before the next case, which starts from none or from the records `orb-real records` makes for it.
//!
//! **No case depends on another**: a case that needs a run left or a replay kept plays the launches that leave
//! one as its own first — `orb_real::Case::before` — so any case can be played alone.
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
    /// **The target the orb installed beside it has to be built for**, which is the architecture the game's
    /// own exe is — see `orb_real::Game::target`. [`TARGET`] for the 32-bit games and the 64-bit one for
    /// th06c, and a mismatch is an exe that injects into nothing.
    target: String,
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

    /// And the ones the launches before wrote taken aside in turn, which is what makes every case start from
    /// none, whichever cases were played before it.
    fn cleared(&self, into: &Path) -> Result<(), String> {
        self.moved(&self.copy, into)
    }

    /// Cleared, and the records `start` names laid in by `orb-real records` — see `orb_real::Start`: made
    /// with the game's own logic, never left by a launch of another case.
    fn started_from(
        &self,
        into: &Path,
        checker: &Path,
        game: &str,
        start: &str,
    ) -> Result<(), String> {
        self.cleared(into)?;
        if start == "nothing" {
            return Ok(());
        }
        let mut lay = Command::new(checker);
        lay.args([
            "records".to_owned(),
            game.to_owned(),
            start.to_owned(),
            windows_path(&self.copy),
        ]);
        match status(lay, "orb-real records") {
            true => Ok(()),
            false => Err(format!("{game}'s {start} records could not be laid")),
        }
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

/// **And the copy's own records go back only here**, so a session ended from outside — a `TaskStop`, a kill of
/// this process — leaves them aside under its `the-copys-own-records` with the case's in their place: th11_dev's
/// under `orb-real/1791676157` and th06nc_dev's under `orb-real/1791701034` (2026-10-11) were left so. A session
/// is ended by its own bound and not from outside it.
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
    /// The cases whose recipes it plays first, as launches of its own — see `orb_real::Case::before`.
    before: Vec<String>,
    /// The records its first launch starts from, as `orb-real records` takes them — see `orb_real::Start`.
    starts_from: String,
    /// The most this launch may take before the session gives up on it, which is the case's own: every wait
    /// a recipe makes is bounded, so a launch still running past it is one whose step never gave up — and
    /// the case that plays a whole game is an hour and a half where the others are twenty minutes.
    within: Duration,
    /// How many updates a frame the case asks to be played at, which is the case's own too — see
    /// `orb_real::Case::speed`, where the rule and the measurements behind it are.
    speed: u32,
    /// And how many times its own rate it asks to be paced to — see `orb_real::Case::pace`.
    pace: u32,
}

pub fn real(args: Vec<String>) -> ExitCode {
    let mut args = args.into_iter();
    let Some(dir) = args.next() else {
        eprintln!(
            "usage: cargo xtask real <copy of the install> [--speed=N] [<case>...]\n\
             \x20      cargo xtask real <copy of the install> --again=<recipe> --again-times=N\n\
             \x20                       --again-records=<start> [--again-from=<kept dir>] [--again-pace=N] [--again-within=S]\n\
             \n\
             The directory holds one of the games orb runs and is a copy of the install, never the\n\
             install: the run writes the game's own records beside it. Cases are those of `orb-real`;\n\
             none named plays them all, in order, each at the speed it declares — four updates a frame\n\
             for a case that plays a whole game and the game's own for the rest. `--speed=N` overrides\n\
             that with one speed for every launch, which is a sweep and not what a session wants.\n\
             \n\
             `--again` is a measurement and not a case: the recipe — a file under `crates/orb-real/\n\
             recipes/<game>/` — launched N times from the records `orb-real records <game> <start>`\n\
             makes, and each chapter's deaths over its attempts said after each launch and over all of\n\
             them. `--again-from` starts every launch's `--keep-in` as a copy of that directory. Nothing\n\
             is checked."
        );
        return ExitCode::FAILURE;
    };
    let dir = PathBuf::from(dir);
    // **`--speed=N` among the cases is one speed for every launch, and every case otherwise carries its
    // own** — see `orb_real::Case::speed`, which answers four for a case that plays a whole game and the
    // game's own one for the rest. So a session with no flag is both green and quick: the long cases are
    // watched in minutes rather than the better part of an hour, and the short ones keep the speed their
    // trace check needs.
    //
    // The flag stays for a sweep, where what is wanted is every case at the one speed whatever it costs the
    // checks.
    let (speed, rest): (Vec<String>, Vec<String>) =
        args.partition(|arg| arg.starts_with("--speed="));
    let (again, only): (Vec<String>, Vec<String>) =
        rest.into_iter().partition(|arg| arg.starts_with("--again"));
    let again = match Again::from_flags(&again) {
        Ok(again) => again,
        Err(why) => {
            eprintln!("xtask: {why}");
            return ExitCode::FAILURE;
        }
    };

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

    // **And orb built again for the game's own architecture where that is not this task's.** The checker
    // above is read on [`TARGET`] whatever the game is — a trace is the same file either way — but the exe
    // installed beside the game has to be the architecture the game is, a 32-bit orb injecting into a
    // 64-bit process not at all. See `orb_real::Game::target`.
    let installed = if game.target == TARGET {
        built.clone()
    } else {
        if !status(
            {
                let mut build = cargo_command();
                build.args([
                    "build",
                    "--release",
                    "--target",
                    &game.target,
                    "-p",
                    "orb-launcher",
                ]);
                build
            },
            "build",
        ) {
            return ExitCode::FAILURE;
        }
        target_dir().join(&game.target).join("release")
    };
    if let Err(why) = std::fs::copy(installed.join("orb.exe"), dir.join("orb.exe")) {
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

    if let Some(again) = again {
        return match again.measured(&dir, game, &checker, &session, &records, &speed) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("xtask: {why}");
                ExitCode::FAILURE
            }
        };
    }

    for case in &game.cases {
        if !only.is_empty() && !only.iter().any(|name| name == &case.name) {
            continue;
        }
        let case_dir = session.join(&case.name);
        let base_dir = case_dir.join("base");
        // **The records the launches before wrote out of the way, and the ones this case starts from laid in.**
        // See `orb_real::Start`: they are made with the game's own logic by `orb-real records`, never left by a
        // launch of another case.
        if let Err(why) = records.started_from(
            &case_dir.join("the-records-before-it"),
            &checker,
            &game.name,
            &case.starts_from,
        ) {
            eprintln!("xtask: {why}");
            return ExitCode::FAILURE;
        }
        // **The cases it plays first, as launches of its own**, each in a directory of its own under this case's
        // and all in this case's `--keep-in`, so that the run one leaves and the replay one keeps are the next's.
        for (place, before) in case.before.iter().enumerate() {
            let Some(earlier) = game.cases.iter().find(|known| &known.name == before) else {
                eprintln!(
                    "xtask: {} {} plays {before} first, which is no case of {}",
                    game.name, case.name, game.name
                );
                return ExitCode::FAILURE;
            };
            let launch_dir = case_dir.join("before").join(format!("{place}-{before}"));
            if let Err(why) = launched(
                &dir,
                game,
                earlier,
                &recipe_of(&game.name, &earlier.name),
                &launch_dir,
                &base_dir,
                &speed,
                &format!("{} (before {})", earlier.name, case.name),
            ) {
                eprintln!("xtask: {why}");
                return ExitCode::FAILURE;
            }
        }
        if let Err(why) = launched(
            &dir,
            game,
            case,
            &recipe_of(&game.name, &case.name),
            &case_dir,
            &base_dir,
            &speed,
            &case.name,
        ) {
            eprintln!("xtask: {why}");
            return ExitCode::FAILURE;
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

/// **One launch of `recipe` as `case` declares it played**, the recipe, trace and kept hits in `launch_dir` and
/// orb's own files in `base_dir`, waited out; `said` is what the lines about it call it.
#[allow(clippy::too_many_arguments)] // Each is a separate thing a launch is made of, and a struct of them would be one named for this call alone.
fn launched(
    dir: &Path,
    game: &Game,
    case: &Case,
    recipe: &Path,
    launch_dir: &Path,
    base_dir: &Path,
    speed: &[String],
    said: &str,
) -> Result<(), String> {
    lay_out(recipe, launch_dir, base_dir)?;
    println!("xtask: {} {said}: launching", game.name);
    let mut launch = Command::new(dir.join("orb.exe"));
    launch.current_dir(dir).args([
        format!("--game-dir={}", windows_path(dir)),
        format!("--keep-in={}", windows_path(base_dir)),
        format!(
            "--autoplay={}",
            windows_path(&launch_dir.join("recipe.yaml"))
        ),
        format!(
            "--trace={}",
            windows_path(&launch_dir.join("trace.msgpack"))
        ),
        // **And every chapter the run was hit in kept beside the trace**, because a hit is what a way is
        // measured against and the run that took it is the one place it happens: a chapter picked up again
        // plays the same way every time, so only the kept one says whether another way gets through it.
        // th06c's stage 6 hit a no-hit case twice in one session and in none of fifteen attempts after it.
        format!("--keep-hits-in={}", windows_path(&launch_dir.join("hits"))),
        "--log=verbose".to_owned(),
        // **Pressed through the virtual pad**, so that a case plays the game the way somebody at a pad does:
        // through the game's own read of one and through orb's menus' — and a game that takes no word as a
        // pad has its word handed over as before, and its log says so.
        "--through-a-pad".to_owned(),
    ]);
    // The flag where one was given, and the case's own otherwise.
    match speed.is_empty() {
        true => launch.arg(format!("--speed={}", case.speed)),
        false => launch.args(speed),
    };
    if case.pace > 1 {
        launch.arg(format!("--pace={}", case.pace));
    }
    if !status(launch, "orb.exe") {
        return Err(format!("{} {said}: orb.exe failed", game.name));
    }
    let took =
        wait_out(&game.exe, case.within).map_err(|why| format!("{} {said}: {why}", game.name))?;
    println!(
        "xtask: {} {said}: the run is over after {}s",
        game.name,
        took.as_secs()
    );
    Ok(())
}

/// The launch's directory, with the recipe copied out of the tree, and the directory orb keeps its files in.
fn lay_out(recipe: &Path, case_dir: &Path, base_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(case_dir)
        .map_err(|why| format!("{} cannot be made: {why}", case_dir.display()))?;
    std::fs::create_dir_all(base_dir)
        .map_err(|why| format!("{} cannot be made: {why}", base_dir.display()))?;
    std::fs::copy(recipe, case_dir.join("recipe.yaml"))
        .map_err(|why| format!("{} cannot be copied: {why}", recipe.display()))?;
    Ok(())
}

/// Where a recipe of `game`'s called `name` is in the tree.
fn recipe_of(game: &str, name: &str) -> PathBuf {
    repo_root()
        .join("crates/orb-real/recipes")
        .join(game)
        .join(format!("{name}.yaml"))
}

/// **`--again`: one recipe launched again and again, and each chapter's attempts and deaths counted over all of
/// them**, which is how a way is measured — a chapter's own hit rate, never a whole run's hits over a whole run.
///
/// **Over launches that differ, which a kept chapter picked up again is not**: it plays with the buttons and the
/// generator it was played with, and orb's rules are a function of what they read, so every pick-up of one kept
/// hit is the same attempt — th06nc's chapter 8 from `s7-c8-f8750` hit at 376.0,432.0 fifteen frames in on each
/// of two (orb-real 1791701034). A launch from the start draws the generator from the clock, so it is the
/// launches from nothing that count; `--again-from` is for a kept directory a run is to be played *on from*.
struct Again {
    /// The recipe's name under `crates/orb-real/recipes/<game>/`.
    recipe: String,
    /// The `--keep-in` each launch starts from a copy of, where there is one.
    from: Option<PathBuf>,
    times: u32,
    /// The records each launch starts from, as `orb-real records` takes them.
    records: String,
    /// How many times its own rate each launch is paced to — `--again-pace`, one where it is not given.
    pace: u32,
    /// How long one launch is given — `--again-within`, in seconds, [`AN_ATTEMPT_WITHIN`] where it is not given.
    within: Duration,
}

impl Again {
    /// The flags, `--again-from` the one that may be left out.
    fn from_flags(flags: &[String]) -> Result<Option<Self>, String> {
        if flags.is_empty() {
            return Ok(None);
        }
        let given = |name: &str| {
            flags
                .iter()
                .find_map(|flag| flag.strip_prefix(&format!("{name}=")))
                .map(str::to_owned)
        };
        let value = |name: &str| {
            given(name).ok_or(format!(
                "--again needs {name}=, with --again-times= and --again-records="
            ))
        };
        let times = value("--again-times")?;
        let pace = given("--again-pace").unwrap_or_else(|| "1".to_owned());
        let within = match given("--again-within") {
            Some(seconds) => Duration::from_secs(seconds.parse().map_err(|why| {
                format!("--again-within={seconds} is no count of seconds: {why}")
            })?),
            None => AN_ATTEMPT_WITHIN,
        };
        Ok(Some(Self {
            within,
            pace: pace
                .parse()
                .map_err(|why| format!("--again-pace={pace} is no pace: {why}"))?,
            recipe: value("--again")?,
            from: given("--again-from").map(PathBuf::from),
            times: times
                .parse()
                .map_err(|why| format!("--again-times={times} is no count: {why}"))?,
            records: value("--again-records")?,
        }))
    }

    /// Every launch played and what it died in said after it.
    fn measured(
        &self,
        dir: &Path,
        game: &Game,
        checker: &Path,
        session: &Path,
        records: &Records,
        speed: &[String],
    ) -> Result<(), String> {
        let recipe = recipe_of(&game.name, &self.recipe);
        if !recipe.is_file() {
            return Err(format!("{} is no recipe", recipe.display()));
        }
        // **At the game's own speed and at the pace asked for**: a pace is the same updates in less wall time, where a
        // speed is more updates a frame and plays another game — see `orb_real::Case::pace`.
        let played = Case {
            name: self.recipe.clone(),
            before: Vec::new(),
            starts_from: self.records.clone(),
            within: self.within,
            speed: 1,
            pace: self.pace,
        };
        let mut chapters = Chapters::default();
        let mut lost = 0;
        for time in 1..=self.times {
            let launch_dir = session.join(format!("{}-{time}", self.recipe));
            let base_dir = launch_dir.join("base");
            records.started_from(
                &launch_dir.join("the-records-before-it"),
                checker,
                &game.name,
                &self.records,
            )?;
            if let Some(from) = &self.from {
                copied(from, &base_dir)?;
            }
            // **Ten seconds after the launch before**: a launch of th11_dev straight after one played to its end
            // read its scoreth11.dat back empty and wrote it so, on 2026-10-08 and again on 2026-10-10.
            std::thread::sleep(BETWEEN_LAUNCHES);
            // **A launch that does not end is counted and passed over**, where a case's stops the session: what is
            // measured here is a rate, and a launch the game hung or died in is one of them — th11's title stood
            // spinning at f215 on 0x44b5c0 in one launch in seven (orb-real 1791748820), and the session stopping
            // there left the three launches after it unplayed.
            if let Err(why) = launched(
                dir,
                game,
                &played,
                &recipe,
                &launch_dir,
                &base_dir,
                speed,
                &format!("{} {time} of {}", self.recipe, self.times),
            ) {
                lost += 1;
                println!(
                    "xtask: {} {time} of {}: lost — {why}",
                    self.recipe, self.times
                );
                closed(&game.exe)?;
                continue;
            }
            let launch = Chapters::of_the_last_launch(&dir.join("orb.log"));
            println!(
                "xtask: {} {time} of {}: {}",
                self.recipe,
                self.times,
                launch.said()
            );
            chapters.add(&launch);
        }
        println!(
            "xtask: {} over {} launches: {}; {lost} lost",
            self.recipe,
            self.times,
            chapters.said()
        );
        Ok(())
    }
}

/// **Each chapter's attempts and deaths**, by its number in the stage — `(stage, chapter)` keyed.
///
/// An attempt is a chapter begun: its own `stage S chapter C … at frame` line where the run walked into it, and
/// once more for every death in it the run carried on from, Pointdevice mode putting that same chapter back.
#[derive(Default)]
struct Chapters(std::collections::BTreeMap<(u32, u32), (u32, u32)>);

impl Chapters {
    /// The launch `orb.log` holds last, which is the one just waited out.
    fn of_the_last_launch(log: &Path) -> Self {
        let mut chapters = Self::default();
        let Ok(bytes) = std::fs::read(log) else {
            return chapters;
        };
        let text = String::from_utf8_lossy(&bytes);
        let last = text.rfind("---- new run ----").unwrap_or(0);
        let mut stage = 0;
        for line in text[last..].lines() {
            // **The line's own words after its time**, so that a line quoting another — the configuration
            // carries the recipe, comments and all — is never read as what it quotes.
            let said = line.split_once("] ").map_or(line, |(_, said)| said);
            if let Some((walked_into, chapter)) = Self::walked_into(said) {
                stage = walked_into;
                chapters.0.entry((stage, chapter)).or_default().0 += 1;
            } else if let Some(chapter) = said
                .strip_prefix("died in chapter ")
                .and_then(|number| number.trim().parse().ok())
            {
                let counts = chapters.0.entry((stage, chapter)).or_default();
                counts.1 += 1;
                counts.0 += 1;
            }
        }
        chapters
    }

    /// `stage S chapter C at frame F (script …)` or `… (stage start) at frame …`, the line a chapter begun says —
    /// and not the lines that say more about one, `stage 7 chapter 4 at frame 5014: the attack it began has a
    /// name after all` among them, nor `resume:`'s, which are what a chapter put back says.
    fn walked_into(said: &str) -> Option<(u32, u32)> {
        let rest = said.strip_prefix("stage ")?;
        let (stage, rest) = rest.split_once(" chapter ")?;
        let chapter: String = rest.chars().take_while(char::is_ascii_digit).collect();
        (rest.contains(" at frame ")
            && (rest.contains("(script ") || rest.contains("(stage start)")))
        .then(|| Some((stage.parse().ok()?, chapter.parse().ok()?)))
        .flatten()
    }

    fn add(&mut self, other: &Self) {
        for (chapter, (attempts, deaths)) in &other.0 {
            let counts = self.0.entry(*chapter).or_default();
            counts.0 += attempts;
            counts.1 += deaths;
        }
    }

    /// Every chapter that was died in, as `s7 c8: 2 of 5`, and how many chapters were attempted besides.
    fn said(&self) -> String {
        let died: Vec<String> = self
            .0
            .iter()
            .filter(|(_, (_, deaths))| *deaths > 0)
            .map(|((stage, chapter), (attempts, deaths))| {
                format!("s{stage} c{chapter}: {deaths} of {attempts}")
            })
            .collect();
        let clean = self.0.len() - died.len();
        match died.is_empty() {
            true => format!("no death in {clean} chapter(s)"),
            false => format!("{}; no death in {clean} chapter(s)", died.join(", ")),
        }
    }
}

/// A launch of `--again` given as long as a case that plays a stage through.
const AN_ATTEMPT_WITHIN: Duration = Duration::from_secs(1800);
const BETWEEN_LAUNCHES: Duration = Duration::from_secs(10);

/// `from` copied into `to`, every file and directory under it, `to` made fresh: a kept hit is picked up from a copy
/// so that the next launch finds it as it was kept.
fn copied(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|why| format!("{} cannot be made: {why}", to.display()))?;
    let entries = std::fs::read_dir(from)
        .map_err(|why| format!("{} cannot be read: {why}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|why| format!("{} cannot be read: {why}", from.display()))?;
        let source = entry.path();
        let destination = to.join(entry.file_name());
        match source.is_dir() {
            true => copied(&source, &destination)?,
            false => {
                std::fs::copy(&source, &destination)
                    .map_err(|why| format!("{} cannot be copied: {why}", source.display()))?;
            }
        }
    }
    Ok(())
}

/// What `orb-real list` prints, read back: one line per case, `game exe records target case before starts_from
/// within speed pace`, tab-separated, the records and the cases before comma-separated among them.
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
        let [
            name,
            exe,
            records,
            target,
            case,
            before,
            starts_from,
            within,
            speed,
            pace,
        ] = fields[..]
        else {
            return Err(format!(
                "orb-real list printed a line of another shape: {line:?}"
            ));
        };
        let case = Case {
            name: case.to_owned(),
            before: match before {
                "-" => Vec::new(),
                before => before.split(',').map(str::to_owned).collect(),
            },
            starts_from: starts_from.to_owned(),
            within: Duration::from_secs(within.parse().map_err(|why| {
                format!("orb-real list printed {within:?} as a count of seconds: {why}")
            })?),
            speed: speed.parse().map_err(|why| {
                format!("orb-real list printed {speed:?} as a count of updates a frame: {why}")
            })?,
            pace: pace
                .parse()
                .map_err(|why| format!("orb-real list printed {pace:?} as a pace: {why}"))?,
        };
        match games.iter_mut().find(|game| game.name == name) {
            Some(game) => game.cases.push(case),
            None => games.push(Game {
                name: name.to_owned(),
                exe: exe.to_owned(),
                records: records.split(',').map(str::to_owned).collect(),
                target: target.to_owned(),
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

/// **The game closed where it is still up**, through `scripts/game.sh` and not a `taskkill.exe` of this file's
/// own: that script is the one place the code-page traps of closing a game by its name are known.
fn closed(exe: &str) -> Result<(), String> {
    if running(exe) == 0 {
        return Ok(());
    }
    let status = Command::new("bash")
        .arg(repo_root().join("scripts/game.sh"))
        .args(["close", exe])
        .status()
        .map_err(|why| format!("scripts/game.sh could not be run: {why}"))?;
    if !status.success() || !gone(exe) {
        return Err(format!("{exe} would not close; the session stops here"));
    }
    Ok(())
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
