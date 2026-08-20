//! **The count 妖々夢 keeps for the whole of a run, and what a stage resumed without it is worth.**
//!
//! `[run+0x18]`, capped at 999999 by the compare at 0x43ebcb. One routine adds to it — 0x43eb90, which adds one
//! to the stage's own count at `[run+0x14]` beside it and takes 2000 points at 0x43ed12 — and two read it,
//! 0x43369a and 0x4338e3, each working an award out as `(count / 40) * 10 + 300`. So it is not a tally: it is
//! what the game charges an award against, and a stage played again with it back at nothing is a stage whose
//! awards are a first stage's.
//!
//! **The game's own replay restore carries it**, 0x44327f writing it into a stage's record and 0x443841 putting
//! it back, and the run's own build zeroes it at 0x42ea7c. So a run picked up out of a file starts from nothing
//! unless the resume puts it back — which is what a session measured: a run picked up in stage 2 reproduced 830
//! frames of itself word for word and then drew four numbers nobody had drawn and took 2000 points nobody had
//! scored, and went on taking them.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between the stage's own start and the card: past the shortest a chapter may be.
const APART: u32 = 120;

/// A count a stage into a run, and not a round number: what a first stage has is nothing, so a resume that
/// wrote nothing back would pass against nothing at all. And the cards captured by then, which is the other
/// count of the run's the game's own restore carries.
const COUNTED: i32 = 9_614;
const CAPTURED: i32 = 11;

#[test]
fn a_run_of_th07_is_put_back_with_the_count_its_awards_are_charged_against() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-count-a-run-carries",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The count as a run a stage into itself has it, and **then** the stage after it: the numbers a stage
        // starts with are read where that stage starts, so written into the stage already running they would be
        // numbers no chapter of it was ever written down with.
        game.image().the_run_has_counted(COUNTED);
        game.image().the_run_has_captured(CAPTURED);
        game.moves_on_to_the_stage(1);

        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 2"));
        let at = game.state().stage_frames;
        game.frames_until_a_thread("the chapter written down", || {
            log.said(&format!("at frame {at}, {at} frame(s) of buttons"))
        });

        // And the run picked up again, whose own build zeroes the count: what stands there afterwards is the
        // recorded run's only if the resume put it back.
        game.gives_the_run_up();
        game.picks_the_run_up_at(1);
        assert_eq!(
            game.image().the_count_the_run_carries(),
            COUNTED,
            "the run came back with its awards charged against another count:\n  {}",
            log.lines().join("\n  "),
        );
        assert_eq!(
            game.image().the_cards_the_run_has_captured(),
            CAPTURED,
            "the run came back having captured another run's cards",
        );
    });
}
