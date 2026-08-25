//! **What a stage transition of th07 carries, and what only the start of a run puts in place.**
//!
//! `GameManager::AddedCallback` is one function reached two ways, and the whole of the difference is its own
//! first condition: 0x42e894 compares the scene the supervisor has been *asked for* against the rebuild scene
//! and jumps to 0x42eee6 where it is that one. What it jumps over is the block that fills a run's numbers in —
//! the lives at 0x42ea1c, the power at 0x42ea51 and the deaths at 0x42eb08 are all inside it — so a run's first
//! stage puts those in place and a transition carries whatever the run had. Which is what a run *is*: six stages
//! with one set of numbers walked through them.
//!
//! **Read out of the game's own code rather than off this tree**, because the laid-out game is both the writer
//! and the reader of the memory orb is driven through: a transition that put a stage's numbers back would be a
//! transition every e2e test here agreed with. See
//! [docs/adr/0008](../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
//!
//! And what orb does with it: a transition is **not** the run ending, and it is not a frame outside a run either
//! — the supervisor does the whole of it inside one update, so what changes for the length of that update is the
//! scene *asked for* and never the scene. The chapters, the retries and the file this run is written down in are
//! the same run's on the far side; the next stage takes a chapter of its own at its own start, and the file
//! follows the run into it.

use crate::fake::th07::{BOMBS, Fake, LIVES, POWER};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run moves on to, counted from zero the way everything above `Game` counts.
const NEXT: i32 = 1;

/// The frames the first stage runs before the transition: past the shortest a chapter may be, so the stage has a
/// chapter of its own to be told from the next stage's.
const APART: u32 = 120;

/// What a life spent leaves: one fewer than the run started with, which is the number a transition has to carry.
const AFTER_A_DEATH: i8 = LIVES as i8 - 1;

fn a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game
}

/// A run carries its lives, its bombs and its power across a stage boundary.
#[test]
fn a_run_of_th07_carries_its_lives_bombs_and_power_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = a_run("th07-a-transition");
        let log = game.log();

        // A life spent in the first stage, and the chapter it was lost in put back: what the second stage has to
        // begin with is what the *run* has, which after a death is one life fewer than it started with.
        game.frames(APART);
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        // A restore is not a way of spending a life — it puts the memory back — so the life goes after it, the
        // way a run really loses one: the player is hit and the chapter is not retried.
        game.dies();
        game.frames_until("the life spent", 8, || game.state().lives == AFTER_A_DEATH);
        let before = game.state();
        assert_eq!(before.stage, 0, "the run is not in its first stage");

        // The transition: the gameplay scene left for a frame, and the next stage built inside it.
        game.moves_on_to_the_stage(NEXT);
        game.frames_until("the next stage's own first chapter", 600, || {
            log.said("stage 2 chapter 1 (stage start)")
        });

        let after = game.state();
        assert_eq!(
            after.stage, NEXT,
            "the run did not move on to the next stage"
        );
        assert_eq!(
            (after.lives, after.bombs, after.power),
            (before.lives, before.bombs, before.power),
            "the next stage was handed the numbers a run *starts* with",
        );
        assert_eq!(
            after.lives, AFTER_A_DEATH,
            "the life spent in the first stage came back with the second",
        );
        assert_eq!(
            after.deaths, before.deaths,
            "the count of deaths is the run's and did not survive its own stage",
        );
        // And its clocks are the new stage's, which is the other half of what a transition is.
        assert!(
            after.stage_frames < APART,
            "the stage's own clock is the stage before it's: {after}",
        );
    });
}

/// **A stage is written down as it begins, and not only once its first chapter has been announced.**
///
/// A stage's first chapter waits for the stage to settle and for the music — see `Chapters::due` — and that
/// wait is a second of a run in which nothing is written down at all. A session interrupted there is picked up
/// at the stage *before*, which is a whole stage of a run thrown away, and where the stage before ends on a
/// chapter of its own it is worse than that: **the run can never get past the boundary.**
///
/// Which two launches of th07 measured. One played into the way into stage 2 and left the run for the title with
/// `resume: stage 2 begins with … power: 81, counted: 231` in its log and a file holding `stage 1 chapter 9`; the
/// next picked that up, played stage 1's boss again, reached the same window, and left a file holding the same
/// chapter — not one `written to` line between the landing and the end of it, stage 1's chapter 9 being its last.
/// Interrupt there twice and the second launch is the first.
///
/// **Nothing here is a guess about what the chapter model will say.** A stage's start is its first chapter at
/// frame 0 with no frames to play back, whatever the stage holds — the same answer, written before the settle
/// rather than after it.
#[test]
fn a_stage_of_th07_is_written_down_as_it_begins() {
    in_its_own_process(|| {
        let game = a_run("th07-a-stage-begun-is-written-down");
        let log = game.log();
        game.frames_until_a_thread("the first stage written down", || {
            log.said("stage 1 chapter 1 (MIDSTAGE 1) at frame")
        });

        let from = log.written();
        game.moves_on_to_the_stage(NEXT);
        game.frames_until_a_thread("the next stage written down", || {
            log.said_since(from, "stage 2 chapter 1 (MIDSTAGE 1) at frame")
        });
        // **Before its first chapter was announced**, which is the whole of it: the window a session is
        // interrupted in is the one between the stage beginning and that announcement, and a file written after
        // it is a file that window has nothing in.
        assert!(
            !log.said_since(from, "stage 2 chapter 1 (stage start)"),
            "the stage's own start was written down only once its first chapter had begun:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **A chapter inside a stage the run walked into is written down like any other.**
///
/// Which is what a session interrupted at stage 4's boss is picked up from, and the whole of what the file is for:
/// the furthest chapter reached, not the stage it is in. A rule that wrote a stage's start in place of its
/// chapters was built here once and taken out again — it made every interrupted stage cost the run that whole
/// stage, which is the thing orb exists to stop.
#[test]
fn a_chapter_of_th07_inside_a_stage_the_run_walked_into_is_written_down() {
    in_its_own_process(|| {
        let game = a_run("th07-a-chapter-of-a-walked-into-stage");
        let log = game.log();
        game.frames_until_a_thread("the first stage written down", || {
            log.said("stage 1 chapter 1 (MIDSTAGE 1) at frame")
        });

        let from = log.written();
        game.moves_on_to_the_stage(NEXT);
        game.frames_until_a_thread("the stage's own start written down", || {
            log.said_since(from, "stage 2 chapter 1 (MIDSTAGE 1) at frame")
        });

        // And a chapter inside it, which is the point the run would come back to.
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || {
            log.said_since(from, "stage 2 chapter 2")
        });
        game.frames_until_a_thread("the chapter written down", || {
            log.said_since(from, "stage 2 chapter 2 (MIDBOSS SPELL 1) at frame")
        });
    });
}

/// **What a stage begins with says the run's own bytes, both ways into it.**
///
/// Which is the instrument the parting is being read with: a chapter recorded in a stage the run walked into
/// does not replay, the two passes agreeing on every field a reproduction line carries and then collecting
/// different items — so the field that differs is one nobody named, and a dump of the whole object is what can
/// name it. Held against the same stage entered the other way, an offset falls out, and an offset is a thing
/// the disassembly can be asked about.
///
/// Said at a stage's start and nowhere else, that being where a run's numbers have just been put in place and
/// the one line per stage a log can afford to carry 0xc8 bytes on.
#[test]
fn what_a_stage_of_th07_begins_with_says_the_runs_own_bytes() {
    in_its_own_process(|| {
        let game = a_run("th07-what-a-stage-begins-with");
        let log = game.log();
        game.frames_until_a_thread("the first stage's own numbers", || {
            log.said("resume: stage 1 begins with")
        });

        let from = log.written();
        game.moves_on_to_the_stage(NEXT);
        game.frames_until_a_thread("the next stage's own numbers", || {
            log.said_since(from, "resume: stage 2 begins with")
        });
        let said = log
            .lines()
            .iter()
            .rev()
            .find(|line| line.contains("resume: stage 2 begins with"))
            .expect("the stage's start was never said")
            .clone();
        // The run's own object and the item manager's head beside it, each as its own bytes: what a comparison
        // needs is the whole of them, so a line carrying a fold or a field list would not do.
        let run = said
            .split("run=")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or_default();
        assert_eq!(
            run.len(),
            RUN_NUMBER_BYTES * 2,
            "the run's own bytes are not the whole of its object:\n  {said}"
        );
        assert!(
            said.contains("items="),
            "the item manager's head was not said with them:\n  {said}"
        );

        // **And on the way in a resume takes**, which is the half that makes it a comparison: the run given up
        // and picked up again says the same line for the same stage, so the two entries can be held against
        // each other.
        // Its first chapter begun before the run is given up, that being done by dying and a death being
        // announced in a chapter.
        game.frames_until("the stage's own first chapter", SETTLES, || {
            log.said("stage 2 chapter 1 (stage start)")
        });
        let from = log.written();
        game.gives_the_run_up();
        game.picks_the_run_up_at(NEXT);
        assert!(
            log.said_since(from, "resume: stage 2 begins with") && log.said_since(from, "run="),
            "the way a resume enters a stage said nothing of the run's own bytes:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// What the game's own allocation for a run's numbers is, which is what the dump has to be the whole of.
const RUN_NUMBER_BYTES: usize = 0xc8;

/// The frames a stage's settle and its music are given to leave the stage with a chapter running.
const SETTLES: u32 = 600;

/// **A run that walked through two stages is written down at the one it is in.**
///
/// Which is the half of the rule above that one transition cannot tell from another: a run left at stage 4's
/// midboss is picked up at stage 4's start and not at stage 3's, and what says so is the point being written again at
/// every boundary rather than once. A stage's start is the *newest* point the file holds, so the run comes back
/// where it was standing to within a stage.
///
/// The one thing that could go wrong and pass the test above is a start written once and never again — a rule
/// that refused every chapter *and* every start after the first would leave a session interrupted in stage 6
/// picked up in stage 2, which is five stages of a run thrown away and worse than what the rule was written to
/// stop.
#[test]
fn a_run_of_th07_that_walked_through_two_stages_is_written_down_at_the_one_it_is_in() {
    in_its_own_process(|| {
        let game = a_run("th07-walked-through-two-stages");
        let log = game.log();
        game.frames_until_a_thread("the first stage written down", || {
            log.said("stage 1 chapter 1 (MIDSTAGE 1) at frame")
        });

        game.moves_on_to_the_stage(NEXT);
        game.frames_until_a_thread("the second stage's start written down", || {
            log.said("stage 2 chapter 1 (MIDSTAGE 1) at frame")
        });

        // The stage after it, walked into from a stage that was itself walked into — which is the run reaching
        // its third stage, and the case a single transition says nothing about.
        let from = log.written();
        game.moves_on_to_the_stage(NEXT + 1);
        game.frames_until_a_thread("the third stage's start written down", || {
            log.said_since(from, "stage 3 chapter 1 (MIDSTAGE 1) at frame")
        });

        // **And it is the newest point in the file**, which is what a resume would offer: the last write the log
        // carries is that stage's, not the stage before it's.
        // The chapter file's own writes and not every file orb writes: the replay of this run is handed over
        // at every chapter too, and it is the *chapter* a resume would be offered out of.
        let last = log
            .lines()
            .iter()
            .rev()
            .find(|line| line.contains("resume: stage") && line.contains("written to"))
            .expect("nothing was written down at all")
            .clone();
        assert!(
            last.contains("stage 3 chapter 1 (MIDSTAGE 1)"),
            "the newest point in the file is not the stage the run is in:\n  {last}"
        );
    });
}

/// The card the midboss puts up in that test, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// And the run is the same run on the far side: the chapters, the retries it has spent and the file it is
/// written down in all follow it into the next stage.
#[test]
fn a_transition_of_th07_is_not_the_run_ending() {
    in_its_own_process(|| {
        let game = a_run("th07-a-transition-carries-on");
        let log = game.log();

        // A retry spent in the first stage, so that what the second stage's own menu says about it is a number
        // this run carried rather than one it started with.
        game.frames(APART);
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        game.frames_until_a_thread("the first stage written down", || {
            log.said("stage 1 chapter 1 (MIDSTAGE 1) at frame")
        });

        let from = log.written();
        game.moves_on_to_the_stage(NEXT);
        game.frames_until("the next stage's own first chapter", 600, || {
            log.said_since(from, "stage 2 chapter 1 (stage start)")
        });
        // Nothing said the run ended, which is the whole of what a transition must not look like: the line a run
        // ending writes is the one that counts its retries.
        assert!(
            !log.said_since(from, "run ended after"),
            "a stage transition was taken for the run ending:\n  {}",
            log.lines().join("\n  ")
        );
        // The file follows the run into the stage it moved on to, which is what a session closed here picks up.
        game.frames_until_a_thread("the next stage written down", || {
            log.said_since(from, "stage 2 chapter 1 (MIDSTAGE 1) at frame")
        });

        // And the retry the first stage cost is still spent: the run is one run, so the menu over the second
        // stage counts from where the first left off.
        game.dies();
        game.frames_until("the death seen", 8, || {
            log.said_since(from, "died in chapter 1")
        });
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("RETRY 1").len(),
            1,
            "the retries the run has spent were forgotten with the stage they were spent in",
        );
        // The numbers a run starts with are not what a stage of it starts with either, which is the same claim
        // the first of these makes and the reason this one can be read after a transition at all.
        let state = game.state();
        assert_eq!(
            (state.bombs, state.power),
            (BOMBS as i8, POWER as u16),
            "the bombs and the power are not the run's own",
        );
    });
}
