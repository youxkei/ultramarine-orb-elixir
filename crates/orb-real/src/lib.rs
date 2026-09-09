//! **The cases the real game is tested with**: a recipe orb plays a launch to, and the check the trace that
//! launch writes has to pass.
//!
//! An e2e test over a laid-out game can say everything about orb but one thing — that the layout is the
//! game's. A run of the real game is the measurement that says so, and until now it was read by a person off
//! `orb.log`, which is a measurement nobody repeats. `--autoplay` plays the run without anybody at the
//! keyboard and `--trace` writes down what every frame held; this crate is the list of what to play and what
//! the trace then has to show, so that `cargo xtask real <copy of the install>` is the measurement.
//!
//! **A check reads the trace's fields and not the log's sentences wherever a field says it.** The chapter, the
//! retries, the question up and the game's own state are fields; a line is read only for what no field
//! carries — which item answered a question, and how the run was left — and then by its opening words, which
//! are the log's own names for those things.
//!
//! **The same cases are played against the laid-out games in `orb-e2e`**, which is what keeps a recipe
//! honest before it costs six minutes of the real game: a step that never reaches its point, or a check that
//! asks for something no launch shows, fails there first.
//!
//! **One case can carry on from the one before it.** A pick-up needs a run left unfinished, and the launch
//! that leaves one is a case of its own; [`Case::continues`] says the two share the directory orb keeps its
//! files in **and the game's own records**, so the second finds both what the first left for it and what the
//! game says that run unlocked. Everything else starts from neither — a directory of its own, and the
//! records taken aside — so that which difficulty a select comes up on and which item a title menu is on are
//! the same every time a case is played. Nothing in the copy is deleted for it; see `cargo xtask real`.
//!
//! **A case plays the game the way somebody does**, which is `orb_bot`'s own rule and not this list's: the
//! word goes in where a run's own recorded words go, the game's screens are pressed at rather than written
//! past, and nothing makes the player unhittable. What follows here is the one thing that is this list's —
//! **no case asks for `--clear`**, which suppresses the very records a case reads: it refuses the replay
//! write, turns the resume off, and leaves an ending nobody played to. What reaches an ending is
//! [`checks::a_run_played_to_its_clear`], the run played out.

use orb_core::trace::Record;

/// Which game a case is written for: the recipes name the same buttons and questions in both, and the front
/// end each walks is its own game's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Game {
    Th06,
    Th07,
}

impl Game {
    /// The directory a game's recipes are under, and the name `cargo xtask real` takes for it.
    pub fn name(self) -> &'static str {
        match self {
            Game::Th06 => "th06",
            Game::Th07 => "th07",
        }
    }

    /// The exe the game runs as, which is what says a copy of the install is this game's, and what a launch
    /// is waited on by. The same names as `orb_core::game::KNOWN` carries, which is where they are read off
    /// the exe; here because a check has no game in memory to ask.
    pub fn exe(self) -> &'static str {
        match self {
            Game::Th06 => "東方紅魔郷.exe",
            Game::Th07 => "th07.exe",
        }
    }

    /// **The records the game keeps of its own, by the names it keeps them under**: its score file, the
    /// configuration beside it, and the file orb forks the first into while it is in pointdevice mode.
    ///
    /// What reads them is `cargo xtask real`, which takes them out of the copy before a case that starts
    /// from nothing — so that which difficulty a select comes up on, which item the title menu is on, and
    /// what the game says has been unlocked are the same for that case every time it is played, rather than
    /// whatever the case before it left. The same names as `orb_core::game::KNOWN` and `orb_core::score`
    /// carry, here for the reason [`Game::exe`] is: the session that moves them has no game in memory to
    /// ask.
    pub fn records(self) -> [&'static str; 3] {
        [
            "score.dat",
            match self {
                Game::Th06 => "東方紅魔郷.cfg",
                Game::Th07 => "th07.cfg",
            },
            "pointdevice_score.dat",
        ]
    }

    pub const ALL: [Game; 2] = [Game::Th06, Game::Th07];
}

/// **How long an ordinary case's launch may take**, in seconds, before `cargo xtask real` gives up on it: a
/// case is a stage or two at sixty frames a second with every wait of its own bounded, so a launch still
/// running past this is one whose step never gave up.
pub const WITHIN: u32 = 20 * 60;

/// **And what a run played to its clear may take**, which is the one case that plays a whole game: six
/// stages by the rules with every death retried, and then the ending and the staff roll. Its own number
/// rather than a ceiling raised for every case, because a ceiling is what says a launch has stopped getting
/// anywhere — raised to this everywhere, a stuck stage would sit for an hour and a half before the session
/// said so.
pub const A_WHOLE_GAME: u32 = 90 * 60;

/// One launch of the real game and what its trace has to show.
pub struct Case {
    pub game: Game,
    /// The name the recipe file and the trace's directory carry.
    pub name: &'static str,
    /// The recipe itself, `orb-config`'s grammar.
    pub recipe: &'static str,
    /// Whether this launch picks up in the directory the case before it left orb's files in — the run it
    /// left, the score file the mode forked — rather than starting in an empty one.
    pub continues: bool,
    /// How long this launch may take, in seconds — [`WITHIN`] for every case but the one that plays a whole
    /// game.
    pub within: u32,
    /// What the trace of that launch has to show, and why not where it does not.
    pub check: fn(&Trace) -> Result<(), String>,
}

impl Case {
    /// **Whether the game's own records hold a clear where this launch starts**, which is what the front end
    /// offers its Extra out of: the case that plays the Extra follows the one that clears the game in the
    /// other mode, and every other starts from records `xtask real`'s own `Records` has taken aside.
    ///
    /// Said here rather than left to the chain because a laid-out game has no records carried across
    /// launches and has to be told — `orb-e2e`'s `the_cases_the_real_game_is_tested_with` is the reader, and
    /// a laid-out title that offered an Extra the real one does not is a title walked somewhere else.
    pub fn after_a_clear(&self) -> bool {
        self.name == "the_extra_a_cleared_run_unlocked"
    }
}

/// Every case, in the order they are played: a case that continues comes straight after the one it
/// continues.
pub fn cases() -> Vec<Case> {
    let mut all = Vec::new();
    for game in Game::ALL {
        all.extend(cases_of(game));
    }
    all
}

/// The cases of one game, in order.
pub fn cases_of(game: Game) -> Vec<Case> {
    macro_rules! recipe {
        ($game:expr, $name:literal) => {
            match $game {
                Game::Th06 => include_str!(concat!("../recipes/th06/", $name, ".yaml")),
                Game::Th07 => include_str!(concat!("../recipes/th07/", $name, ".yaml")),
            }
        };
    }
    vec![
        Case {
            game,
            name: "a_run_left_in_its_second_chapter",
            recipe: recipe!(game, "a_run_left_in_its_second_chapter"),
            continues: false,
            within: WITHIN,
            check: checks::a_run_left_in_its_second_chapter,
        },
        Case {
            game,
            name: "the_run_picked_up_where_it_was_left",
            recipe: recipe!(game, "the_run_picked_up_where_it_was_left"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_where_it_was_left,
        },
        Case {
            game,
            name: "a_death_answered_by_the_chapter_again",
            recipe: recipe!(game, "a_death_answered_by_the_chapter_again"),
            continues: false,
            within: WITHIN,
            check: checks::a_death_answered_by_the_chapter_again,
        },
        // The two that watch what that death left: the run it gave up is what はじめから is answered about,
        // and the replay kept there is the one with two attempts at the chapter it died in.
        Case {
            game,
            name: "the_run_started_over_and_its_replay_kept",
            recipe: recipe!(game, "the_run_started_over_and_its_replay_kept"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_started_over_and_its_replay_kept,
        },
        Case {
            game,
            name: "an_attempt_of_the_kept_replay_watched",
            recipe: recipe!(game, "an_attempt_of_the_kept_replay_watched"),
            continues: true,
            within: WITHIN,
            check: checks::an_attempt_of_the_kept_replay_watched,
        },
        Case {
            game,
            name: "a_chapter_taken_again_without_a_death",
            recipe: recipe!(game, "a_chapter_taken_again_without_a_death"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_taken_again_without_a_death,
        },
        Case {
            game,
            name: "a_chapter_further_back_taken_again",
            recipe: recipe!(game, "a_chapter_further_back_taken_again"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_further_back_taken_again,
        },
        Case {
            game,
            name: "the_stage_taken_again_from_its_start",
            recipe: recipe!(game, "the_stage_taken_again_from_its_start"),
            continues: false,
            within: WITHIN,
            check: checks::the_stage_taken_again_from_its_start,
        },
        Case {
            game,
            name: "the_run_given_up_at_the_retry",
            recipe: recipe!(game, "the_run_given_up_at_the_retry"),
            continues: false,
            within: WITHIN,
            check: checks::the_run_given_up_at_the_retry,
        },
        Case {
            game,
            name: "a_chapter_of_the_stage_after_the_first",
            recipe: recipe!(game, "a_chapter_of_the_stage_after_the_first"),
            continues: false,
            within: WITHIN,
            check: checks::a_chapter_of_the_stage_after_the_first,
        },
        // And the run that case left in the stage it walked into, picked up in that stage.
        Case {
            game,
            name: "the_run_picked_up_in_the_stage_it_walked_into",
            recipe: recipe!(game, "the_run_picked_up_in_the_stage_it_walked_into"),
            continues: true,
            within: WITHIN,
            check: checks::the_run_picked_up_in_the_stage_it_walked_into,
        },
        Case {
            game,
            name: "a_stage_practised_on_its_own",
            recipe: recipe!(game, "a_stage_practised_on_its_own"),
            continues: false,
            within: WITHIN,
            check: checks::a_stage_practised_on_its_own,
        },
        Case {
            game,
            name: "a_normal_mode_run_nothing_is_kept_of",
            recipe: recipe!(game, "a_normal_mode_run_nothing_is_kept_of"),
            continues: false,
            within: WITHIN,
            check: checks::a_normal_mode_run_nothing_is_kept_of,
        },
        // And the three that play whole games, last because they are the cases that cost an hour apiece: the
        // run's own end is reachable no other way, and the Extra behind them is reachable only from a clear.
        Case {
            game,
            name: "a_run_played_to_its_clear",
            recipe: recipe!(game, "a_run_played_to_its_clear"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_run_played_to_its_clear,
        },
        // **The other mode's own clear, which is what unlocks the Extra**: a pointdevice run's records go to
        // the mode's own file and the front end lights its items from the game's, so the clear that puts the
        // record where that read finds it is a run of the other mode — see `orb_core::score`.
        Case {
            game,
            name: "a_normal_mode_run_played_to_its_clear",
            recipe: recipe!(game, "a_normal_mode_run_played_to_its_clear"),
            continues: false,
            within: A_WHOLE_GAME,
            check: checks::a_normal_mode_run_played_to_its_clear,
        },
        // And the Extra that clear unlocked. **The one case that needs what the case before it left in the
        // game's own records** rather than in the directory orb keeps its files in — which is what
        // [`Case::continues`] keeps for it: a case that starts from nothing has the game's records taken
        // aside, and a game with no record of a clear does not offer the Extra at all.
        Case {
            game,
            name: "the_extra_a_cleared_run_unlocked",
            recipe: recipe!(game, "the_extra_a_cleared_run_unlocked"),
            continues: true,
            within: WITHIN,
            check: checks::the_extra_a_cleared_run_unlocked,
        },
    ]
}

/// The name of the file a launch's trace is written to, beside the settings file of the directory orb keeps
/// its files in for that launch.
pub const TRACE_FILE: &str = "trace.msgpack";

/// A trace read back, with the questions a check asks of one.
pub struct Trace {
    pub records: Vec<Record>,
}

impl Trace {
    pub fn read(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self {
            records: orb_core::trace::read(bytes)?,
        })
    }

    /// The index of the first record from `from` on whose lines hold `needle`.
    pub fn line_from(&self, from: usize, needle: &str) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| record.lines.iter().any(|line| line.contains(needle)))
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on under which `question` stands.
    pub fn question_from(&self, from: usize, question: &str) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| record.question.as_deref() == Some(question))
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on that is inside a run: the game's own state says so.
    pub fn in_run_from(&self, from: usize) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| record.state.is_some_and(|state| state.in_run))
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on that is at or past `chapter` of `stage`, the stage
    /// counted the way a recipe counts it — from one — and the chapter the way the log does.
    pub fn chapter_from(&self, from: usize, stage: i32, chapter: u32) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record.state.is_some_and(|state| {
                    state.in_run && (state.stage + 1, record.chapter) >= (stage, chapter)
                })
            })
            .map(|at| at + from)
    }

    /// The index of the first record from `from` on that is in a run at or past `stage`, the stage counted
    /// the way a recipe counts it — **and nothing said about the chapter**, for a run orb keeps none of:
    /// a run of the normal mode is at chapter nought on every frame of it.
    pub fn stage_from(&self, from: usize, stage: i32) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record
                    .state
                    .is_some_and(|state| state.in_run && state.stage + 1 >= stage)
            })
            .map(|at| at + from)
    }

    /// And the same at or past `chapter` of **whatever stage** the run is in, for a case whose stage is the
    /// game's own to choose: which stage a practice run is played in is whichever one its select came up on.
    pub fn a_chapter_from(&self, from: usize, chapter: u32) -> Option<usize> {
        self.records[from.min(self.records.len())..]
            .iter()
            .position(|record| {
                record.state.is_some_and(|state| state.in_run) && record.chapter >= chapter
            })
            .map(|at| at + from)
    }

    /// Every record between two indices, the second left out.
    pub fn between(&self, from: usize, to: usize) -> &[Record] {
        &self.records[from.min(self.records.len())..to.min(self.records.len())]
    }

    /// The lines of every record, with the frame each was said on, for a check that failed to say what the
    /// launch did instead.
    ///
    /// **Without the per-frame ones**, which say the same thing sixty times a second and would bury the line a
    /// reader is after: the frame a playback played, the state line, and a step still waiting. A trace keeps
    /// them; what this is for is reading, and the whole file is there for a check that wants one of them.
    /// **What a launch did that the game it was played on has no say in**, in the order it happened: the
    /// questions orb put up, the screens and items its front end was walked through, and the recipe's own
    /// steps with how each of them ended.
    ///
    /// What it leaves out is everything a laid-out game is free to differ in — how long a stage lasts, how
    /// many chapters it holds, what the generator handed out, what the sound did — so that a laid-out game's
    /// own held against the real game's says whether the same presses reached the same places, and not
    /// whether the two are the same program.
    pub fn shape(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut question = None;
        for record in &self.records {
            let now = record.question.as_deref();
            if now != question {
                if let Some(name) = now {
                    out.push(format!("orb asks {name}"));
                }
                question = now;
            }
            for line in &record.lines {
                // The front end's own line and not the one the verbose log writes beside it, which carries
                // the object's address: what is being compared is the screen and the item, and an address is
                // a thing no two launches of one game agree on either.
                let walked = line.starts_with("front end: screen");
                let stepped =
                    line.starts_with("autoplay: step") && !line.contains("is still waiting");
                if walked || stepped {
                    out.push(line.clone());
                }
            }
        }
        out
    }

    pub fn narrative(&self) -> String {
        const PER_FRAME: [&str; 3] = ["resume: frame ", "state: cur=", "autoplay: still waiting"];
        let mut out = String::new();
        for record in &self.records {
            for line in &record.lines {
                let per_frame = PER_FRAME.iter().any(|prefix| line.starts_with(prefix))
                    || line.starts_with('f') && line.contains(" scene=");
                if !per_frame {
                    out.push_str(&format!("f{} {line}\n", record.frame));
                }
            }
        }
        out
    }
}

/// `Err` with the reason where a condition does not hold, so that a check reads as a list of what has to be
/// true and the reason travels with it.
fn expect(holds: bool, why_not: impl FnOnce() -> String) -> Result<(), String> {
    if holds { Ok(()) } else { Err(why_not()) }
}

/// The checks, one per case.
pub mod checks {
    use super::{Trace, expect};

    /// **Which difficulty the Extra is**, which is the same number in both games: th06's own
    /// `DIFFICULTY_EXTRA`, and the fifth of the six th07 keeps a record per. What reads it is the case that
    /// starts the Extra, the item being what writes it rather than a select.
    const EXTRA: i32 = 4;

    /// **The mode question stood, was answered for the rewindable mode, a run was started and played into its
    /// second chapter, and then left for the title** — which is what leaves a run for the next case to pick up.
    pub fn a_run_left_in_its_second_chapter(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        expect(
            trace.between(asked, started).iter().any(|record| {
                record
                    .lines
                    .iter()
                    .any(|line| line.starts_with("mode: answered"))
            }),
            || "the mode question was not answered before the run started".to_owned(),
        )?;
        let second = trace
            .chapter_from(started, 1, 2)
            .ok_or("the run never reached stage 1 chapter 2")?;
        expect(
            trace
                .between(started, second)
                .iter()
                .all(|record| record.retries == 0),
            || {
                "a chapter was retried on the way to the second, which a clean run does not do"
                    .to_owned()
            },
        )?;
        let left = trace
            .line_from(second, "retry: the run is given up")
            .ok_or("the run was never given up for the title")?;
        expect(
            trace.records[left..]
                .iter()
                .any(|record| record.state.is_some_and(|state| !state.in_run)),
            || "the game never left the run after it was given up".to_owned(),
        )
    }

    /// **The run the case before left is offered, taken up from where it stopped, and lands in the chapter it
    /// was left in** — then plays on into the next.
    pub fn the_run_picked_up_where_it_was_left(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to pick up")?;
        let taken = trace
            .line_from(asked, "resume: from where it stopped")
            .ok_or("the resume question was not answered with where it stopped")?;
        let landed = trace
            .chapter_from(taken, 1, 2)
            .ok_or("the run never came back up in stage 1 chapter 2, where it was left")?;
        expect(
            trace
                .between(taken, landed)
                .iter()
                .all(|record| record.chapter <= 2),
            || {
                "the run went past the chapter it was left in before landing there, which is a playback \
                 that overshot"
                    .to_owned()
            },
        )?;
        trace
            .chapter_from(landed, 1, 3)
            .map(|_| ())
            .ok_or_else(|| "the picked-up run never went on into stage 1 chapter 3".to_owned())
    }

    /// **The run left is declined for one of this launch's own, and the replay of it is kept**: はじめから
    /// is answered at the question the file raises, the question about that run's replay comes up behind it
    /// and is answered はい, and the run that starts is a fresh one — nothing played back into place.
    pub fn the_run_started_over_and_its_replay_kept(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to start over")?;
        let answered = trace
            .line_from(asked, "resume: from the beginning")
            .ok_or("the resume question was not answered with the beginning")?;
        let offered = trace
            .question_from(answered, "keep_replay")
            .ok_or("the replay of the run being written over was never asked about")?;
        let kept = trace
            .line_from(offered, "replay: kept")
            .ok_or("the question about the replay was not answered with keeping it")?;
        let started = trace
            .in_run_from(kept)
            .ok_or("no run was started after the replay was kept")?;
        expect(
            trace.between(kept, started).iter().all(|record| {
                !record
                    .lines
                    .iter()
                    .any(|line| line.contains("resume: the landing is"))
            }),
            || {
                "a chapter was played back into place, where a run from the beginning has none"
                    .to_owned()
            },
        )?;
        expect(trace.records[started].chapter <= 1, || {
            format!(
                "the run came up in chapter {}, where a run from the beginning starts at the stage's \
                 first",
                trace.records[started].chapter
            )
        })?;
        trace
            .chapter_from(started, 1, 2)
            .map(|_| ())
            .ok_or_else(|| {
                "the run started from the beginning never reached its second chapter".to_owned()
            })
    }

    /// **The replay kept, watched by stage, chapter and attempt**: the question over the title menu's replay
    /// item is answered for the mode whose replays are orb's own, each of the three lists behind it is
    /// answered in turn, and the attempt chosen is played back into the stage the run made it in — landing
    /// where the run was and then going on a frame of the attempt to a frame of the game.
    pub fn an_attempt_of_the_kept_replay_watched(trace: &Trace) -> Result<(), String> {
        trace
            .line_from(0, "menu: Replays is under the cursor")
            .ok_or("the question orb asked was not the one about which replays to look at")?;
        let listed = trace
            .line_from(0, "replay(s) to watch")
            .ok_or("the replays orb keeps were never listed")?;
        let chosen = trace
            .line_from(listed, "stage(s) in it")
            .ok_or("no replay was chosen out of the list")?;
        let of_a_stage = trace
            .line_from(chosen, "chapter(s) in it")
            .ok_or("no stage of that replay was chosen")?;
        let of_a_chapter = trace
            .line_from(of_a_stage, "attempt(s) at it")
            .ok_or("no chapter of that stage was chosen")?;
        let watching = trace
            .line_from(of_a_chapter, "replay: watching attempt")
            .ok_or("no attempt at that chapter was chosen")?;
        let landed = trace
            .line_from(watching, "resume: the landing is")
            .ok_or("the attempt was never played back into a stage")?;
        let over = trace
            .line_from(landed, "replay: the attempt is over")
            .ok_or("the attempt never ran out")?;
        // The stage's own clock over the frames it was being watched in, which is what says a frame of the
        // attempt went to a frame of the game rather than the run being left standing where it landed.
        let clock: Vec<u32> = trace
            .between(landed, over)
            .iter()
            .filter_map(|record| {
                record
                    .state
                    .filter(|state| state.in_run)
                    .map(|state| state.stage_frames)
            })
            .collect();
        expect(clock.windows(2).all(|pair| pair[1] >= pair[0]), || {
            "the stage's clock went back while the attempt was being watched".to_owned()
        })?;
        expect(
            matches!((clock.first(), clock.last()), (Some(first), Some(last)) if *last >= first + 60),
            || {
                format!(
                    "the attempt was watched over {} frame(s) of the stage, which is not a run being \
                     played on",
                    clock.last().unwrap_or(&0) - clock.first().unwrap_or(&0),
                )
            },
        )?;
        expect(
            trace.records[over..]
                .iter()
                .any(|record| record.state.is_some_and(|state| !state.in_run)),
            || "the run the attempt was watched in was never left".to_owned(),
        )?;
        trace
            .line_from(over, "replay: the attempts at that chapter again")
            .map(|_| ())
            .ok_or_else(|| "the list the attempt was chosen out of never came back up".to_owned())
    }

    /// **A hit taken on purpose raises the retry question, the chapter is taken again, and the run goes on
    /// past it**: the deaths climb, the retries climb, the stage's clock goes back, and the chapter after the
    /// one died in is reached all the same.
    pub fn a_death_answered_by_the_chapter_again(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let retry = trace
            .question_from(started, "retry")
            .ok_or("the retry question never stood, so nothing was hit")?;
        let before = trace.records[retry]
            .state
            .ok_or("the retry stood over no state")?;
        let died_in = trace.records[retry].chapter;
        expect(before.deaths >= 1, || {
            format!(
                "the retry stood with the game counting {} deaths",
                before.deaths
            )
        })?;
        let answered = trace
            .line_from(retry, "retry: the chapter again")
            .ok_or("the retry question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == died_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {died_in}, which it died in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        trace
            .chapter_from(back, 1, died_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the run never got past chapter {died_in} after retrying it"))
    }

    /// **The run asked to start over raises the same question a death does, and the chapter is taken again
    /// with nothing lost**: the chapter comes back, the stage's clock goes back, the retries climb — and the
    /// deaths do not, which is what tells this retry from the one a death asks for.
    pub fn a_chapter_taken_again_without_a_death(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(after.deaths == before.deaths, || {
            format!(
                "the run died on the way to the chapter again: {} death(s) when it was asked to \
                 start over and {} when it came back",
                before.deaths, after.deaths
            )
        })?;
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked \
                 to start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        expect(trace.records[back].retries >= 1, || {
            "the retry was not counted".to_owned()
        })?;
        trace
            .chapter_from(back, 1, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| {
                format!("the run never got past chapter {asked_in} after taking it again")
            })
    }

    /// **The chapters behind the one the run was in are a list of their own, and one of them puts the run
    /// back there**: the list comes up, a chapter is chosen out of it and confirmed, and the run comes back
    /// in a chapter earlier than the one it was asked to start over in — then plays through to it again.
    pub fn a_chapter_further_back_taken_again(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        trace
            .line_from(up, "retry: asking which chapter further back")
            .ok_or("the chapters behind the one the run was in were never listed")?;
        let answered = trace
            .line_from(up, "retry: a chapter further back")
            .ok_or("no chapter behind the one the run was in was chosen")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the chapter behind was chosen")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        let came_back_in = trace.records[back].chapter;
        expect(came_back_in < asked_in, || {
            format!(
                "the run came back in chapter {came_back_in} rather than behind chapter {asked_in}, \
                 which it was asked to start over in"
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 1, asked_in)
            .map(|_| ())
            .ok_or_else(|| {
                format!(
                    "the run never played back up to chapter {asked_in} from the chapter behind it"
                )
            })
    }

    /// **The stage's own start is a way on of its own, and it asks before it acts**: the question is
    /// answered with the stage, はい is answered at the confirmation behind it, and the run comes back at
    /// the stage's first chapter with its clock at the start of the stage.
    pub fn the_stage_taken_again_from_its_start(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        trace
            .line_from(up, "retry: asking about the stage again")
            .ok_or("the stage was taken again without being asked about first")?;
        let answered = trace
            .line_from(up, "retry: the stage again")
            .ok_or("the question was not answered with the stage again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the stage was taken again")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == 1, || {
            format!(
                "the run came back in chapter {} rather than in the stage's first, having been asked \
                 to start over in chapter {asked_in}",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 1, 2)
            .map(|_| ())
            .ok_or_else(|| "the run never played out of the stage's first chapter again".to_owned())
    }

    /// **The run given up at that question, which is the other way a session is interrupted**: the item
    /// asks first, はい is answered at it, and the game leaves the run for the title.
    pub fn the_run_given_up_at_the_retry(trace: &Trace) -> Result<(), String> {
        let started = trace.in_run_from(0).ok_or("no run was started")?;
        let asked = trace
            .line_from(started, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        trace
            .line_from(up, "retry: asking about the run given up")
            .ok_or("the run was given up without being asked about first")?;
        let answered = trace
            .line_from(up, "retry: the run given up")
            .ok_or("the question was not answered with the run given up")?;
        let left = trace
            .line_from(answered, "retry: the run is given up")
            .ok_or("nothing gave the run up, the answer notwithstanding")?;
        expect(
            trace.records[left..]
                .iter()
                .any(|record| record.state.is_some_and(|state| !state.in_run)),
            || "the game never left the run after it was given up".to_owned(),
        )
    }

    /// **The run played into the stage after its first, and a chapter of that one taken again**: the second
    /// stage begins with its own clock and the numbers the run carried into it, and the chapter it is asked
    /// to start over in comes back the way a chapter of the first stage does.
    pub fn a_chapter_of_the_stage_after_the_first(trace: &Trace) -> Result<(), String> {
        let first = trace
            .chapter_from(0, 1, 1)
            .ok_or("the run never reached the first stage's first chapter")?;
        let carried = trace.records[first]
            .state
            .expect("chapter_from answers a record with a state");
        let second = trace
            .chapter_from(first, 2, 1)
            .ok_or("the run never reached the stage after the first")?;
        let begun = trace.records[second]
            .state
            .expect("chapter_from answers a record with a state");
        // Where the first stage had got to, which is what the second beginning is held against: its own
        // first chapter is taken as many frames in as the first stage's was, so the two chapters say
        // nothing about a clock starting again and the frame before the transition says it.
        let ended = trace
            .between(first, second)
            .iter()
            .rev()
            .find_map(|record| {
                record
                    .state
                    .filter(|state| state.in_run && state.stage == carried.stage)
            })
            .ok_or("the run was never in the first stage")?;
        expect(begun.stage_frames < ended.stage_frames, || {
            format!(
                "the second stage's clock is at {} where the first stage's had reached {}, so nothing \
                 began again",
                begun.stage_frames, ended.stage_frames
            )
        })?;
        // And what only the start of a run puts in place, held against what a stage walked into carries:
        // the deaths are the run's own count, so a stage that had reset them would be one built as a run's
        // first rather than out of the stage before it.
        expect(begun.deaths >= ended.deaths, || {
            format!(
                "the run reached the second stage with {} death(s) against the {} the first stage ended \
                 with",
                begun.deaths, ended.deaths
            )
        })?;
        let asked = trace
            .line_from(second, "retry: the run was asked to start over")
            .ok_or("the run was never asked to start over in the second stage")?;
        let up = trace
            .question_from(asked, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(after.stage == before.stage, || {
            format!(
                "the run came back in stage {} rather than in stage {}, which it was retried in",
                after.stage + 1,
                before.stage + 1
            )
        })?;
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked to \
                 start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 2, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the run never got past chapter {asked_in} of the second stage"))
    }

    /// **The run left in the stage it walked into is picked up in that stage**: the build a resume answers
    /// for is pointed at the stage the file names, so the run comes up there rather than at a run's first —
    /// then lands in the chapter it was left in and plays on.
    pub fn the_run_picked_up_in_the_stage_it_walked_into(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "resume")
            .ok_or("the resume question never stood, so nothing was left to pick up")?;
        let taken = trace
            .line_from(asked, "resume: from where it stopped")
            .ok_or("the resume question was not answered with where it stopped")?;
        let landed = trace
            .chapter_from(taken, 2, 2)
            .ok_or("the run never came back up in a chapter of the second stage")?;
        // **And it came up in that stage rather than at a run's own first**, which is the other way this can
        // go wrong: the build a resume answers for is pointed at the stage the file names, and a first stage
        // coming up here would be a run started from the beginning with a chapter of the second to put back.
        let at_the_first = trace.between(taken, landed).iter().find(|record| {
            record
                .state
                .is_some_and(|state| state.in_run && state.stage == 0)
        });
        expect(at_the_first.is_none(), || {
            format!(
                "frame {} of the pick-up is in the first stage, where the file named the second",
                at_the_first.expect("a record in the first stage").frame
            )
        })?;
        trace.chapter_from(landed, 2, 3).map(|_| ()).ok_or_else(|| {
            "the run picked up in the second stage never went on into its next chapter".to_owned()
        })
    }

    /// **One stage practised on its own, which orb rewinds and writes nothing down for**: the run says it is
    /// a practice one, its chapters are a stage's like any other's and one of them is taken again — and
    /// nothing of it was looked for on the disk or left there, a run of this kind having no name.
    pub fn a_stage_practised_on_its_own(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that practises a stage")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        let state = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        expect(state.practice, || {
            "the run that started reads as a full run rather than as one stage practised".to_owned()
        })?;
        let in_a_chapter = trace
            .a_chapter_from(started, 1)
            .ok_or("the practice run never reached a chapter of its own")?;
        trace
            .line_from(
                started,
                "of a run orb is not keeping; nothing of it is written down",
            )
            .ok_or("nothing said the practice run was one orb writes nothing down for")?;
        expect(
            trace.line_from(0, "resume: nothing was left of").is_none(),
            || {
                "a run left unfinished was looked for, where a practice run has no name to look one \
                 up by"
                    .to_owned()
            },
        )?;
        // And rewound all the same, which is the other half of a practice run being a run.
        let asked_over = trace
            .line_from(in_a_chapter, "retry: the run was asked to start over")
            .ok_or("the practice run was never asked to start over")?;
        let up = trace
            .question_from(asked_over, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the practice run never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked \
                 to start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })
    }

    /// **A run played from the title to its clear, which is the whole of what the mode is for**: it goes
    /// through every stage and is never given up, the ending runs out to where its staff roll begins — which
    /// is what says the game was cleared rather than left — and the replay of a run nobody will add to again
    /// is offered at the screen that run ends at and kept.
    pub fn a_run_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question")?;
        // **The fifth stage and not the last**, because how many stages a run has is the game's: th06's Easy
        // ends after the fifth, where every other difficulty of either game has a sixth. What says the run
        // was played *out* is the ending below and not a count of stages.
        let last = trace
            .chapter_from(started, 5, 1)
            .ok_or("the run never reached a fifth stage")?;
        expect(
            trace
                .line_from(started, "retry: the run is given up")
                .is_none(),
            || "the run was given up rather than played out".to_owned(),
        )?;
        trace
            .line_from(last, "run ended after")
            .ok_or("the run never ended, so it was neither cleared nor left")?;
        // **And the ending is what says it was cleared rather than left**: a run given up reaches none.
        //
        // **Read off the log's own words and not off the scene**, which is where this check was wrong first:
        // orb runs an ending out inside the frame it came up on, so a launch that skips one has no frame
        // whose state is in it at all — a cleared run of th06 said `ending skipped, 25101 frames run` and
        // every record of that trace read `in_ending` false. Which of the two lines a run gets is the
        // ending's own — the run-out stops at a staff roll where there is one — and a launch told not to
        // skip plays the ending instead, which is the frames the scene is in it for.
        let ending = trace.records[last..]
            .iter()
            .position(|record| {
                record.lines.iter().any(|line| {
                    line.starts_with("ending run out in") || line.starts_with("ending skipped")
                }) || record.state.is_some_and(|state| state.in_ending)
            })
            .map(|at| at + last)
            .ok_or("nothing says an ending ran, so the run was not cleared")?;
        let offered = trace
            .question_from(ending, "keep_replay")
            .ok_or("the replay of the run that was finished with was never asked about")?;
        trace
            .line_from(offered, "replay: kept")
            .map(|_| ())
            .ok_or_else(|| {
                "the question about that replay was not answered with keeping it".to_owned()
            })
    }

    /// **A run of the other mode played to its clear, which is the one thing that unlocks the Extra**: the
    /// question is answered for the normal mode, no chapter is counted in any frame of it, the run goes
    /// through the stages on the lives the game gave it, and an ending comes — the record of which lands in
    /// the game's own score file, where the front end reads the items it offers from.
    pub fn a_normal_mode_run_played_to_its_clear(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let answered = trace
            .line_from(asked, "mode: normal")
            .ok_or("the mode question was not answered for the normal mode")?;
        let started = trace
            .in_run_from(answered)
            .ok_or("no run was started after the mode question")?;
        // The fifth stage and by the stage alone: a run of this mode is at chapter nought on every frame,
        // orb keeping none of it, so there is no chapter to wait for.
        let last = trace
            .stage_from(started, 5)
            .ok_or("the run never reached a fifth stage")?;
        let kept = trace.records[started..]
            .iter()
            .find(|record| record.chapter != 0);
        expect(kept.is_none(), || {
            let record = kept.expect("a record with a chapter of its own");
            format!(
                "frame {} of the run is at chapter {}, where a run of this mode has none",
                record.frame, record.chapter
            )
        })?;
        // **And the ending off the log's own words as well as off the scene** — for the reason
        // [`a_run_played_to_its_clear`] gives, which holds of a run of either mode: orb runs an ending out
        // inside the frame it came up on, and the run cleared here said `ending skipped, 7993 frames run,
        // scene 10 -> 7` with every record of its trace reading `in_ending` false.
        trace.records[last..]
            .iter()
            .position(|record| {
                record.lines.iter().any(|line| {
                    line.starts_with("ending run out in") || line.starts_with("ending skipped")
                }) || record.state.is_some_and(|state| state.in_ending)
            })
            .map(|_| ())
            .ok_or_else(|| "nothing says an ending ran, so the run was not cleared".to_owned())
    }

    /// **The Extra, which the run cleared in the other mode unlocked**: the run that starts is at the
    /// Extra's own difficulty, its chapters are a stage's like any other's, and one of them is taken again.
    ///
    /// What says the unlock happened is that there was a run to start at all: the item is on the title menu
    /// only where the game's own score file holds the record, and the case before this one is what put it
    /// there.
    pub fn the_extra_a_cleared_run_unlocked(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood over the item that starts the Extra")?;
        let started = trace
            .in_run_from(asked)
            .ok_or("no run was started after the mode question, so the item offered none")?;
        let state = trace.records[started]
            .state
            .expect("in_run_from answers a record with a state");
        expect(state.difficulty == EXTRA, || {
            format!(
                "the run that started is at difficulty {}, where the Extra is {EXTRA}",
                state.difficulty
            )
        })?;
        let in_a_chapter = trace
            .chapter_from(started, 7, 1)
            .ok_or("the Extra never reached a chapter of its own")?;
        let asked_over = trace
            .line_from(in_a_chapter, "retry: the run was asked to start over")
            .ok_or("the Extra was never asked to start over")?;
        let up = trace
            .question_from(asked_over, "retry")
            .ok_or("the ask raised no question of where to start again")?;
        let before = trace.records[up]
            .state
            .ok_or("the question stood over no state")?;
        let asked_in = trace.records[up].chapter;
        let answered = trace
            .line_from(up, "retry: the chapter again")
            .ok_or("the question was not answered with the chapter again")?;
        let back = trace
            .in_run_from(answered + 1)
            .ok_or("the Extra never came back after the retry")?;
        let after = trace.records[back]
            .state
            .expect("in_run_from answers a record with a state");
        expect(trace.records[back].chapter == asked_in, || {
            format!(
                "the run came back in chapter {} rather than chapter {asked_in}, which it was asked to \
                 start over in",
                trace.records[back].chapter
            )
        })?;
        expect(after.stage_frames < before.stage_frames, || {
            format!(
                "the stage's clock did not go back: {} before the retry, {} after",
                before.stage_frames, after.stage_frames
            )
        })?;
        trace
            .chapter_from(back, 7, asked_in + 1)
            .map(|_| ())
            .ok_or_else(|| format!("the Extra never got past chapter {asked_in} after retrying it"))
    }

    /// **A run of the other mode is the game as it comes**: the question is answered for the normal mode,
    /// the run is played, and orb keeps nothing of it — no chapter counted, no retry, and the hook that
    /// writes a chapter down saying it wrote nothing.
    pub fn a_normal_mode_run_nothing_is_kept_of(trace: &Trace) -> Result<(), String> {
        let asked = trace
            .question_from(0, "mode")
            .ok_or("the mode question never stood")?;
        let answered = trace
            .line_from(asked, "mode: normal")
            .ok_or("the mode question was not answered for the normal mode")?;
        let started = trace
            .in_run_from(answered)
            .ok_or("no run was started after the mode question")?;
        trace
            .line_from(
                answered,
                "of a run orb is not keeping; nothing of it is written down",
            )
            .ok_or("nothing said the run was one orb keeps nothing of")?;
        let kept = trace.records[started..]
            .iter()
            .find(|record| record.chapter != 0 || record.retries != 0);
        expect(kept.is_none(), || {
            let record = kept.expect("a record with a chapter of its own");
            format!(
                "frame {} of the run is at chapter {} with {} retr(ies), where a run of this mode \
                 has neither",
                record.frame, record.chapter, record.retries
            )
        })?;
        let over = trace.records[started..]
            .iter()
            .find(|record| record.question.is_some());
        expect(over.is_none(), || {
            let record = over.expect("a record with a question over it");
            format!(
                "orb put its {} question up on frame {} of a run of this mode",
                record.question.as_deref().unwrap_or_default(),
                record.frame
            )
        })
    }
}
