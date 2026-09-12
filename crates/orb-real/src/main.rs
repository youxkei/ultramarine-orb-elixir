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
             orb-real show <trace> [<needle>]"
        );
        return ExitCode::FAILURE;
    };
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
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                case.game.name(),
                case.game.exe(),
                case.game.records().join(","),
                case.game.target(),
                case.name,
                u8::from(case.continues),
                case.within,
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
fn shown(path: &str, needle: Option<&str>) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|why| format!("{path} cannot be read: {why}"))?;
    let trace = Trace::read(&bytes)?;
    for record in &trace.records {
        let lines: Vec<&String> = record
            .lines
            .iter()
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
