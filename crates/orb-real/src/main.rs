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
//! orb-real show <trace> [<needle>]
//! ```
//!
//! `<session-dir>/<case>/trace.msgpack` is where each case's trace is looked for; a case with no trace there
//! is reported as not run, which is not a failure: `cargo xtask real` plays the cases of the one game the
//! copy holds, and a session stopped early has traces for the cases before the stop.
//!
//! `list` prints the cases for `cargo xtask real` to drive — one line each, `game`, the game's exe, the
//! records it keeps of its own as one comma-separated field, the target orb has to be built for to attach
//! to it, the case, whether it continues the one before and how long its launch may take, tab-separated —
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
             orb-real show <trace> [<needle>]\n       orb-real watching <trace>\n       \
             orb-real picture <trace> <out.bmp> [<frame>]"
        );
        return ExitCode::FAILURE;
    };
    if session == "watching" {
        let Some(path) = args.next() else {
            eprintln!("usage: orb-real watching <trace>");
            return ExitCode::FAILURE;
        };
        return match watching(&path) {
            Ok(()) => ExitCode::SUCCESS,
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
    if session == "list" {
        for case in orb_real::cases() {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                case.game.name(),
                case.game.exe(),
                case.game.records().join(","),
                case.game.target(),
                case.name,
                u8::from(case.continues),
                case.within,
                case.speed(),
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
            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
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
            let trace = match Trace::read(&bytes) {
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
fn watching(path: &str) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|why| format!("{path} cannot be read: {why}"))?;
    let trace = Trace::read(&bytes)?;
    let Some(newest) = trace.records.last() else {
        return Err(format!("{path} holds no frame yet"));
    };
    let said = |among: &[&str]| {
        trace
            .records
            .iter()
            .rev()
            .flat_map(|record| record.lines.iter().rev().map(move |line| (record, line)))
            .find(|(_, line)| among.iter().any(|it| line.contains(it)))
            .map(|(record, line)| format!("f{} {line}", record.frame))
    };
    let step = said(&["autoplay: step"]);
    // **Each needle as long as it has to be to match one line and not another.** `chapter` alone finds
    // the `chaptering=` of every `perf:` line, which is said once every 600 frames and so is always the
    // newest of anything: the trailing space is what tells a boundary's own line from that field.
    let event = said(&[
        "died in chapter",
        "autoplay: hit at",
        "chapter ",
        "run ended",
        "retry: ",
        "front end: screen",
    ]);
    let pictures = trace
        .records
        .iter()
        .filter(|record| record.picture.is_some())
        .count();
    match &newest.state {
        Some(state) => println!("f{} {}ms {state}", newest.frame, newest.ms),
        None => println!("f{} {}ms nothing read yet", newest.frame, newest.ms),
    }
    println!("  step: {}", step.as_deref().unwrap_or("no step said yet"));
    println!("  event: {}", event.as_deref().unwrap_or("nothing yet"));
    println!("  {} frame(s), {pictures} picture(s)", trace.records.len());
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
    let bytes = std::fs::read(path).map_err(|why| format!("{path} cannot be read: {why}"))?;
    let trace = Trace::read(&bytes)?;
    let mut kept = trace
        .records
        .iter()
        .filter(|record| record.picture.is_some());
    let found = match at {
        Some(at) => kept.find(|record| record.frame >= at),
        None => kept.next_back(),
    };
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
    let bytes = std::fs::read(path).map_err(|why| format!("{path} cannot be read: {why}"))?;
    let trace = Trace::read(&bytes)?;
    for record in &trace.records {
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
            continue;
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
    }
    Ok(())
}
