//! **The check over what `cargo xtask real` left behind**: one directory per case, each holding the trace its
//! launch wrote, and a pass or a reason for each.
//!
//! A binary of its own rather than a task of `xtask`'s because a trace is read through `orb-core`, which is
//! built for the target the game runs on and for no other — so this runs where orb runs, and `xtask` starts
//! it the way it starts every other binary of that target.
//!
//! ```text
//! orb-real <session-dir> [<game>]
//! orb-real list
//! orb-real records <game> <start> <dir>
//! orb-real show <trace> [<needle>]
//! orb-real spawns <trace>
//! orb-real paths <trace> [<every>]
//! ```
//!
//! `<session-dir>/<case>/trace.msgpack` is where each case's trace is looked for; a case with no trace there
//! is reported as not run, which is not a failure: `cargo xtask real` plays the cases of the one game the
//! copy holds, and a session stopped early has traces for the cases before the stop.
//!
//! `list` prints the cases for `cargo xtask real` to drive — one line each, `game`, the game's exe, the
//! records it keeps of its own as one comma-separated field, the target orb has to be built for to attach
//! to it, the case, the cases whose recipes it plays first, the records it starts from and how long its launch may
//! take, tab-separated —
//! so that which games and cases there are is said in one place, this crate, and `xtask` carries no copy
//! of it.
//!
//! **`show` prints what one trace holds**, frame by frame: the state the game's memory was in and the log's
//! lines that frame, and with a `<needle>` only the frames whose lines hold it. Without it a trace is
//! readable by the checks alone, and the launch's own `orb.log` is not the same evidence — one file across
//! launches, rotated when it grows, so the launch a question is about is the one whose log has gone. That is
//! not a hypothetical: a title menu walked to the wrong item was asked about after the next case had started,
//! and the log holding the walk had been rotated away while its trace sat under the case all along.

use std::path::Path;
use std::process::ExitCode;

use orb_real::{Game, TRACE_FILE, Trace, cases_of};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(session) = args.next() else {
        eprintln!(
            "usage: orb-real <session-dir> [th06|th07]\n       orb-real list\n       \
             orb-real records <game> <start> <dir>\n       \
             orb-real show <trace> [<needle>]\n       orb-real watching <trace> [<kept>]\n       \
             orb-real picture <trace> <out.bmp> [<frame>]\n       orb-real spawns <trace>\n       \
             orb-real paths <trace> [<every>]"
        );
        return ExitCode::FAILURE;
    };
    if session == "watching" {
        let Some(path) = args.next() else {
            eprintln!("usage: orb-real watching <trace> [<kept>]");
            return ExitCode::FAILURE;
        };
        let kept = args.next();
        return match watching(&path, kept.as_deref()) {
            Ok(said) => {
                print!("{said}");
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "picture" {
        let (Some(path), Some(out)) = (args.next(), args.next()) else {
            eprintln!("usage: orb-real picture <trace> <out.bmp> [<frame>]");
            return ExitCode::FAILURE;
        };
        let at: Option<u32> = args.next().and_then(|it| it.parse().ok());
        return match picture(&path, &out, at) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "show" {
        let Some(path) = args.next() else {
            eprintln!("usage: orb-real show <trace> [<needle>]");
            return ExitCode::FAILURE;
        };
        return match shown(&path, args.next().as_deref()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "paths" {
        let Some(path) = args.next() else {
            eprintln!("usage: orb-real paths <trace> [<every>]");
            return ExitCode::FAILURE;
        };
        let every = match args.next().map(|it| it.parse::<i32>()) {
            None => A_PLACE_EVERY,
            Some(Ok(every)) if every > 0 => every,
            Some(_) => {
                eprintln!("usage: orb-real paths <trace> [<every>], <every> a count of frames");
                return ExitCode::FAILURE;
            }
        };
        return match paths(&path, every) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "world" {
        let (Some(path), Some(table)) = (args.next(), args.next()) else {
            eprintln!("usage: orb-real world <trace> <table>");
            return ExitCode::FAILURE;
        };
        return match world(&path, &table) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "spawns" {
        let Some(path) = args.next() else {
            eprintln!("usage: orb-real spawns <trace>");
            return ExitCode::FAILURE;
        };
        return match spawns(&path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "records" {
        let (Some(game), Some(start), Some(dir)) = (args.next(), args.next(), args.next()) else {
            eprintln!("usage: orb-real records <game> <start> <dir>");
            return ExitCode::FAILURE;
        };
        return match records_laid(&game, &start, Path::new(&dir)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("orb-real: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if session == "list" {
        for case in orb_real::cases() {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                case.game.name(),
                case.game.exe(),
                case.game.records().join(","),
                case.game.target(),
                case.name,
                match case.before {
                    [] => "-".to_owned(),
                    before => before.join(","),
                },
                start_name(case.starts_from),
                case.within,
                case.speed(),
                case.pace(),
            );
        }
        return ExitCode::SUCCESS;
    }
    let games: Vec<Game> = match args.next().as_deref() {
        None => Game::ALL.to_vec(),
        Some(name) => match Game::ALL.iter().find(|game| game.name() == name) {
            Some(game) => vec![*game],
            None => {
                eprintln!("orb-real: no game called {name}");
                return ExitCode::FAILURE;
            }
        },
    };

    let mut failed = false;
    let mut ran = 0;
    for game in games {
        for case in cases_of(game) {
            let path = Path::new(&session).join(case.name).join(TRACE_FILE);
            let file = match std::fs::File::open(&path) {
                Ok(file) => file,
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                    println!("{} {}: not run", game.name(), case.name);
                    continue;
                }
                Err(why) => {
                    println!(
                        "{} {}: FAILED, {} cannot be read: {why}",
                        game.name(),
                        case.name,
                        path.display()
                    );
                    failed = true;
                    continue;
                }
            };
            ran += 1;
            // **Its pictures let go of as each record is read**, no check looking at one: a trace kept with
            // them goes past what an i686 process can hold — see `orb_core::trace::each`.
            let mut records = Vec::new();
            let read = orb_core::trace::each(file, |mut record| {
                record.picture = None;
                records.push(record);
            });
            let trace = match read.map(|_| Trace { records }) {
                Ok(trace) => trace,
                Err(why) => {
                    println!(
                        "{} {}: FAILED, the trace cannot be read: {why}",
                        game.name(),
                        case.name
                    );
                    failed = true;
                    continue;
                }
            };
            match (case.check)(&trace) {
                Ok(()) => println!(
                    "{} {}: ok over {} frames",
                    game.name(),
                    case.name,
                    trace.records.len()
                ),
                Err(why) => {
                    println!("{} {}: FAILED, {why}", game.name(), case.name);
                    println!("  what the launch said:");
                    for line in trace.narrative().lines() {
                        println!("    {line}");
                    }
                    failed = true;
                }
            }
        }
    }
    if ran == 0 {
        println!("no case has a trace under {session}");
        return ExitCode::FAILURE;
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// The name `list` prints a case's [`orb_real::Start`] by and `records` takes it by.
fn start_name(start: orb_real::Start) -> &'static str {
    match start {
        orb_real::Start::Nothing => "nothing",
        orb_real::Start::ANormalClear => "a-normal-clear",
        orb_real::Start::TheExtraOffered => "the-extra-offered",
    }
}

/// **The game's records a case starts from, written into `dir`** — `Game::the_records_of`, by the names the game
/// keeps them under. Here and not in `xtask` because the records are made with `orb-core`'s reading of the game,
/// which is built for the target the game runs on, as every other read of this binary's is.
fn records_laid(game: &str, start: &str, dir: &Path) -> Result<(), String> {
    let game = Game::ALL
        .into_iter()
        .find(|known| known.name() == game)
        .ok_or_else(|| format!("no game called {game}"))?;
    let start = [
        orb_real::Start::Nothing,
        orb_real::Start::ANormalClear,
        orb_real::Start::TheExtraOffered,
    ]
    .into_iter()
    .find(|known| start_name(*known) == start)
    .ok_or_else(|| format!("no start called {start}"))?;
    for (name, bytes) in game.the_records_of(start) {
        let path = dir.join(name);
        std::fs::write(&path, bytes)
            .map_err(|why| format!("{} cannot be written: {why}", path.display()))?;
    }
    Ok(())
}

/// What one trace holds, printed: for every frame that said something, the state the game's memory was in,
/// which question of orb's own stood over it, and the lines the log said that frame — or, given a `needle`,
/// only the frames whose lines hold it.
///
/// **The frames that said nothing are left out**, a trace holding one record per frame and a launch running
/// to hundreds of thousands of them: what a reader is after is a line, and the state is worth printing where
/// it says what the game was doing as that line was written.
/// **What a launch being watched is asked every so often, out of its own trace**: the newest frame's state,
/// the recipe's newest step, and the newest of the things that happen to a run.
///
/// **Which is the same three a monitor reads the log for, and better from here.** A state line in the log
/// is said once every so many frames, so a monitor's `tail` finds whichever one is nearest; a record is
/// every frame, so the newest is the newest. And the step and the event come from the frames they were said
/// on, so what is printed is one moment of the launch rather than three greps of a file that may have
/// moved between them.
///
/// **The trace is seconds behind and that is what it is for**: records are held until a chunk of them is
/// worth a write — see `orb_core::trace`'s own `FLUSH_AT` — so this is the launch as of a moment ago and
/// never mid-record, whole records being what a chunk is made of.
///
/// **And with `kept`, read on from where the look before left off**: the file at `kept` holds the byte that look
/// read to and what it had found by then, and is written again with this look's — see
/// `orb_core::trace::each_from` for why a look of a long launch cannot start from the beginning. A trace shorter
/// than the byte kept, or another trace, is read from its start.
fn watching(path: &str, kept: Option<&str>) -> Result<String, String> {
    let mut seen = kept
        .and_then(|kept| std::fs::read_to_string(kept).ok())
        .and_then(|it| Watched::from_kept(&it, path))
        .unwrap_or_default();
    let read = seen.read_on(path);
    if let Some(kept) = kept {
        std::fs::write(kept, seen.to_kept(path))
            .map_err(|why| format!("{kept} cannot be written: {why}"))?;
    }
    read?;
    seen.said(path)
}

/// **What a watch has found of a trace**, up to the byte it read to.
#[derive(Debug, Default, PartialEq)]
struct Watched {
    read_to: u64,
    frames: usize,
    pictures: usize,
    newest: Option<String>,
    step: Option<String>,
    event: Option<String>,
}

impl Watched {
    /// The records past what has been found, taken in up to the last one whole.
    fn read_on(&mut self, path: &str) -> Result<(), String> {
        use std::io::Seek;
        // **Each needle as long as it has to be to match one line and not another.** `chapter` alone finds
        // the `chaptering=` of every `perf:` line, which is said once every 600 frames and so is always the
        // newest of anything: the trailing space is what tells a boundary's own line from that field.
        const EVENTS: [&str; 6] = [
            "died in chapter",
            "autoplay: hit at",
            "chapter ",
            "run ended",
            "retry: ",
            "front end: screen",
        ];
        let mut file =
            std::fs::File::open(path).map_err(|why| format!("{path} cannot be read: {why}"))?;
        let length = file
            .metadata()
            .map_err(|why| format!("{path} cannot be read: {why}"))?
            .len();
        if length < self.read_to {
            *self = Self::default();
        }
        file.seek(std::io::SeekFrom::Start(self.read_to))
            .map_err(|why| format!("{path} cannot be read: {why}"))?;
        orb_core::trace::each_from(file, self.read_to, |mut record, past| {
            let said = |among: &[&str]| {
                record
                    .lines
                    .iter()
                    .rev()
                    .find(|line| among.iter().any(|it| line.contains(it)))
                    .map(|line| format!("f{} {line}", record.frame))
            };
            self.step = said(&["autoplay: step"]).or(self.step.take());
            self.event = said(&EVENTS).or(self.event.take());
            self.pictures += usize::from(record.picture.take().is_some());
            self.frames += 1;
            self.newest = Some(match &record.state {
                Some(state) => format!("f{} {}ms {state}", record.frame, record.ms),
                None => format!("f{} {}ms nothing read yet", record.frame, record.ms),
            });
            self.read_to = past;
        })
        .map(|_| ())
        .map_err(|why| format!("{path}: {why}"))
    }

    /// What a look prints.
    fn said(&self, path: &str) -> Result<String, String> {
        let Some(newest) = &self.newest else {
            return Err(format!("{path} holds no frame yet"));
        };
        Ok(format!(
            "{newest}\n  step: {}\n  event: {}\n  {} frame(s), {} picture(s)\n",
            self.step.as_deref().unwrap_or("no step said yet"),
            self.event.as_deref().unwrap_or("nothing yet"),
            self.frames,
            self.pictures,
        ))
    }

    /// As a `kept` file holds it: the trace, the counts, then the three lines, each empty where nothing was found.
    fn to_kept(&self, path: &str) -> String {
        let line = |it: &Option<String>| it.as_deref().unwrap_or("").replace('\n', " ");
        format!(
            "{path}\n{} {} {}\n{}\n{}\n{}\n",
            self.read_to,
            self.frames,
            self.pictures,
            line(&self.newest),
            line(&self.step),
            line(&self.event),
        )
    }

    /// Out of a `kept` file, where it was written of `path`.
    fn from_kept(kept: &str, path: &str) -> Option<Self> {
        let mut lines = kept.lines();
        if lines.next()? != path {
            return None;
        }
        let mut counts = lines.next()?.split(' ');
        let mut count = || counts.next()?.parse::<u64>().ok();
        let (read_to, frames, pictures) = (count()?, count()?, count()?);
        let mut line = || {
            let it = lines.next()?;
            Some((!it.is_empty()).then(|| it.to_owned()))
        };
        Some(Self {
            read_to,
            frames: frames as usize,
            pictures: pictures as usize,
            newest: line()?,
            step: line()?,
            event: line()?,
        })
    }
}

/// **Every record of the trace at `path` handed to `on` as it is read**, and how many there were — see
/// `orb_core::trace::each` for why a file is not taken in whole.
fn each_record(path: &str, on: impl FnMut(orb_core::trace::Record)) -> Result<usize, String> {
    let file = std::fs::File::open(path).map_err(|why| format!("{path} cannot be read: {why}"))?;
    orb_core::trace::each(file, on).map_err(|why| format!("{path}: {why}"))
}

/// **One table of the world a trace was taken with `--trace-the-world`**, a line a row: the stage counted from one,
/// its frame, the row's place in the table, and the row's numbers as the game's `the_world` lays them out.
fn world(path: &str, table: &str) -> Result<(), String> {
    each_record(path, |record| {
        let (Some(state), Some(world)) = (&record.state, &record.the_world) else {
            return;
        };
        let Some((_, rows)) = world.tables.iter().find(|(name, _)| name == table) else {
            return;
        };
        for (at, row) in rows.iter().enumerate() {
            let numbers: Vec<String> = row.iter().map(|it| format!("{it}")).collect();
            println!(
                "s{} {} {at} {}",
                state.stage + 1,
                state.stage_frames,
                numbers.join(" ")
            );
        }
    })?;
    Ok(())
}

/// **The script frames enemies came on at**, a line apiece: the stage counted from one, the script frame,
/// and how many more enemies there were than the frame before.
///
/// **What holds one game's midstage table against another's clock.** A stage's waves come on when its
/// script says and leave when the run shoots them, so the frames the count rises on are the script's
/// alone and the same in any run of the stage, where a gap between waves — what `--tune` proposes a
/// boundary in — opens as fast as that run kills. th06nc's stages are th06c's waves on a clock shifted by
/// a different number of frames from stage to stage, and the rises of two runs laid side by side say by
/// how many where a row of the table stands. A frame whose script clock stands still — a boss holding it —
/// is left out, since every rise there has the one frame number.
fn spawns(path: &str) -> Result<(), String> {
    let mut before: Option<orb_core::game::State> = None;
    each_record(path, |record| {
        let Some(state) = record.state else {
            return;
        };
        if let Some(last) = &before
            && last.stage == state.stage
            && state.script_frames > last.script_frames
            && state.enemy_count > last.enemy_count
        {
            println!(
                "s{} {} +{}",
                state.stage + 1,
                state.script_frames,
                state.enemy_count - last.enemy_count,
            );
        }
        before = Some(state);
    })?;
    Ok(())
}

/// **How many frames apart the places `paths` writes are, where nothing says**: a focused step of `2.0` a frame
/// goes `8` between two, close enough that a path gone along between them is the one the player went.
const A_PLACE_EVERY: i32 = 4;

/// One card's frames on its first attempt, each with where the player was on it.
type ACardsPath = (u32, Vec<(i32, (f32, f32))>);

/// **The player's path through every card a trace holds, written as the recipe steps that go along it**: for
/// each card, its first attempt's frames — the card's own count, from the boss's attack — and the player's place
/// on every `every`th of them, as a `follow` step.
///
/// **What a replay is learnt from in one playing.** A replay played back with `--trace` puts the player's place
/// into every record, and a card whose fire is aimed at the player is gone through again by going where the player
/// went on each of its frames — th10's `土着神「宝永四年の赤蛙」`, whose doubles go at where the player is and lay their
/// rice on the way.
fn paths(path: &str, every: i32) -> Result<(), String> {
    // Each card's frames and the place on each, its first attempt alone: a card's count going back is a retry.
    let mut cards: Vec<ACardsPath> = Vec::new();
    let mut done: Vec<u32> = Vec::new();
    each_record(path, |record| {
        let (Some(state), Some(at)) = (&record.state, record.at) else {
            return;
        };
        let (Some(card), Some(frames)) = (state.spellcard, state.boss_attack_frames) else {
            return;
        };
        if done.contains(&card) {
            return;
        }
        match cards.iter_mut().find(|(it, _)| *it == card) {
            Some((_, places)) if places.last().is_some_and(|(last, _)| frames < *last) => {
                done.push(card)
            }
            Some((_, places)) => places.push((frames, at)),
            None => cards.push((card, vec![(frames, at)])),
        }
    })?;
    for (card, places) in cards {
        let Some(&(last, _)) = places.last() else {
            continue;
        };
        let mut kept = Vec::new();
        let mut at = places[0].1;
        let mut next = places.iter().peekable();
        for frame in (every..=last).step_by(usize::try_from(every).unwrap_or(1)) {
            while let Some((_, place)) = next.next_if(|(it, _)| *it <= frame) {
                at = *place;
            }
            kept.push(format!("[{:.1}, {:.1}]", at.0, at.1));
        }
        println!("- follow:");
        println!("    spell: {card}");
        println!("    every: {every}");
        println!("    path: [");
        for line in kept.chunks(8) {
            println!("      {},", line.join(", "));
        }
        println!("    ]");
    }
    Ok(())
}

/// **One of a trace's own pictures written out where it can be looked at**, the newest by default and
/// otherwise the first taken on or after the frame asked for.
///
/// **Because a picture in a trace is a picture nobody can see.** `--shot-every` puts the back buffer into
/// the trace every so many seconds and `orb-real watching` says how many are in there, which answers *that*
/// a screen was kept and nothing about what was on it — and what a run is asked about is usually the screen:
/// whether orb's own drawing is still up, which attack the boss is on, where the player stood.
///
/// A 32-bit bottom-up BMP, which is what `orb_core::shot` keeps — see its own doc for why nothing encodes
/// anything on the frame's own thread.
fn picture(path: &str, out: &str, at: Option<u32>) -> Result<(), String> {
    let mut found: Option<orb_core::trace::Record> = None;
    each_record(path, |record| {
        let wanted = match at {
            Some(at) => found.is_none() && record.frame >= at,
            None => true,
        };
        if wanted && record.picture.is_some() {
            found = Some(record);
        }
    })?;
    let record = found.ok_or_else(|| match at {
        Some(at) => format!("{path} holds no picture taken on frame {at} or after it"),
        None => format!("{path} holds no picture"),
    })?;
    let picture = record.picture.as_ref().expect("filtered to the kept ones");
    std::fs::write(out, picture).map_err(|why| format!("{out} cannot be written: {why}"))?;
    println!(
        "f{} {}ms: {} byte(s) written to {out}",
        record.frame,
        record.ms,
        picture.len(),
    );
    if let Some(state) = &record.state {
        println!("  {state}");
    }
    Ok(())
}

/// What stood over a hit, a line an enemy or a bullet, every one of them beginning `over a hit at` so that a
/// `show` can be asked for them alone.
fn over_a_hit(over: &orb_core::trace::OverAHit) -> Vec<String> {
    let at = format!("over a hit at {:.1},{:.1}:", over.at.0, over.at.1);
    let enemies = over.enemies.iter().map(|enemy| {
        format!(
            "{at} enemy slot {} at {:.1},{:.1} size {:.1},{:.1} flags {:#010x} familiar {:#x} behaviour {} \
             anchor {} angle {:.3}",
            enemy.slot,
            enemy.at.0,
            enemy.at.1,
            enemy.size.0,
            enemy.size.1,
            enemy.flags,
            enemy.familiar,
            enemy
                .behaviour
                .map_or_else(|| "none".to_owned(), |it| format!("{it:#x}")),
            enemy
                .anchor
                .map_or_else(|| "none".to_owned(), |(x, y)| format!("{x:.1},{y:.1}")),
            enemy.angle,
        )
    });
    let bullets = over.bullets.iter().map(|bullet| {
        format!(
            "{at} bullet slot {} at {:.1},{:.1} size {:.1},{:.1} kind {} hidden {}",
            bullet.slot,
            bullet.at.0,
            bullet.at.1,
            bullet.size.0,
            bullet.size.1,
            bullet.kind,
            bullet.hidden,
        )
    });
    enemies.chain(bullets).collect()
}

fn shown(path: &str, needle: Option<&str>) -> Result<(), String> {
    each_record(path, |record| {
        // **And what stood over a hit, as lines of their own** beside the log's — see
        // `orb_core::trace::OverAHit`, which a check reads as fields and a person reads here.
        let over: Vec<String> = record.over_a_hit.iter().flat_map(over_a_hit).collect();
        let lines: Vec<&String> = record
            .lines
            .iter()
            .chain(over.iter())
            .filter(|line| needle.is_none_or(|needle| line.contains(needle)))
            .collect();
        if lines.is_empty() {
            return;
        }
        let question = match &record.question {
            Some(question) => format!(" question={question}"),
            None => String::new(),
        };
        // The word the run was played with among them, which is what says whether a press reached the game
        // on the frame a screen moved: a walk that moved for no press of the recipe's is orb's own write.
        match &record.state {
            Some(state) => println!(
                "f{} {}ms{question} word={:#06x} {state}",
                record.frame, record.ms, record.word
            ),
            None => println!(
                "f{} {}ms{question} word={:#06x} nothing read yet",
                record.frame, record.ms, record.word
            ),
        }
        for line in lines {
            println!("  {line}");
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::watching;
    use orb_core::trace::Record;

    fn a_record(frame: u32, line: &str) -> Vec<u8> {
        rmp_serde::to_vec_named(&Record {
            frame,
            ms: frame * 16,
            state: None,
            chapter: 0,
            retries: 0,
            question: None,
            word: 0,
            lines: vec![line.to_owned()],
            picture: None,
            over_a_hit: None,
            the_field: None,
            at: None,
            the_world: None,
            spans: Vec::new(),
            phases: Vec::new(),
        })
        .unwrap()
    }

    /// **A look kept between looks reads on from where the one before left off, and says what one look of the
    /// whole trace says**: the bytes the first look read are written over with what no record is, and the second
    /// look, of the two records written since, still answers as a look of all five.
    #[test]
    fn a_watch_kept_between_looks_reads_on_from_the_look_before() {
        let dir = std::env::temp_dir().join(format!("orb-real-watching-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (trace, kept, whole) = (
            dir.join("trace.msgpack"),
            dir.join("kept"),
            dir.join("whole"),
        );
        let (trace, kept) = (trace.to_str().unwrap(), kept.to_str().unwrap());

        let mut first = a_record(1, "autoplay: step 1 of 3");
        first.extend(a_record(2, "stage 1 chapter 1 at frame 2"));
        first.extend(a_record(3, "nothing of note"));
        let later = [
            a_record(4, "autoplay: step 2 of 3"),
            a_record(5, "nothing of note"),
        ]
        .concat();
        std::fs::write(&whole, [first.as_slice(), &later].concat()).unwrap();
        let said_of_the_whole = watching(whole.to_str().unwrap(), None).unwrap();

        std::fs::write(trace, &first).unwrap();
        watching(trace, Some(kept)).unwrap();
        std::fs::write(trace, [vec![0xc1; first.len()], later].concat()).unwrap();
        let said = watching(trace, Some(kept)).unwrap();

        assert_eq!(said, said_of_the_whole);
        assert!(said.contains("5 frame(s)"), "{said}");
        assert!(said.contains("step: f4 autoplay: step 2 of 3"), "{said}");
        assert!(
            said.contains("event: f2 stage 1 chapter 1 at frame 2"),
            "{said}"
        );
    }
}
