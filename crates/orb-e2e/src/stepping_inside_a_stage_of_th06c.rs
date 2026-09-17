//! **The stepping keys over th06c: one chapter at a time, inside the stage.**
//!
//! What `stepping_inside_a_stage.rs` asks of th06 and `stepping_inside_a_stage_of_th07.rs` of th07, asked of
//! this game. Both go through `Chapters::previous_start` and `next_start`, whose whole job is that a step
//! lands **strictly** either side of where the game is — so a press moves rather than staying put, and a
//! frame or two past a boundary steps back onto it.
//!
//! **A pass over this game watches its own replay**, which took a reading of its own to make possible:
//! `runtime::step` asks for `during_replay && state.replay && state.in_run` together, and `sight::in_a_run`
//! answered `!a_replay_is_playing()` — the demo and the watched replay at once — so those three were never
//! all true and no pass over th06c could step. It excludes the demo alone now, and `State::in_game` is the
//! reading a run of somebody's own answers. `th06c.rs`'s own replay test is the three answers written down.
//!
//! **And the boundaries are the fight's**, this game's midstage table being empty until a pass fills it:
//! the stage's own start, the midboss's arrival, the card it puts up and the attack after it. Which is what
//! a pass has to step between before there is a table at all — see `a_chapter_table_collected_for_th06c`.

use orb_config::LogLevel;

use crate::fake::th06c::{ATTACK_CHANGES, CARD_STARTS, Fake, MIDBOSS_ARRIVES};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;
/// And the one held with a step to leave the stage altogether, which is what `moving_between_a_replays_stages`
/// is about for the games that have it.
const ACROSS: u8 = orb_config::keys::SHIFT.0;

/// How long a stage lasts here: past the attack the card hands over to, and no further.
const STAGE_FRAMES: u32 = ATTACK_CHANGES + 200;

/// How many frames a wait here is given.
const FRAMES: u32 = STAGE_FRAMES * 3;

/// One press of a stepping key with nothing else held, and the frame it left the game on.
///
/// # Panics
/// Where the step wrote no line saying where it held the game, which is a press that did nothing.
fn steps(game: &Fake, key: u8) -> u32 {
    let held = game.log().written();
    game.press(key);
    let said = game.log().lines().split_off(held).join("\n  ");
    assert!(
        said.contains("step: held"),
        "the press of {key:#04x} held the game nowhere:\n  {said}",
    );
    game.state().stage_frames
}

/// The launch a judging pass is: the stepping keys on over a replay, with nothing fired at the player — a
/// death would end the run this is stepping around, and a replay is no run to offer a retry for anyway.
fn judging(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_stepping = true;
            config.during_replay = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.watches_a_replay_of_the_stage(0);
    game.frames_until("the stage's own first chapter", FRAMES, || {
        game.log().said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The four boundaries of this stage's fight, each waited for and the frame it began on kept.
fn the_boundaries(game: &Fake) -> (u32, u32, u32, u32) {
    let log = game.log();
    let the_stages_own_start = game.state().stage_frames;
    game.frames_until("the midboss's chapter", FRAMES, || log.said("chapter 2"));
    let the_midboss = game.state().stage_frames;
    game.frames_until("the card's chapter", FRAMES, || log.said("chapter 3"));
    let the_card = game.state().stage_frames;
    game.frames_until("the chapter after the card", FRAMES, || {
        log.said("chapter 4")
    });
    let after_the_card = game.state().stage_frames;
    // The frames the fight's own script puts them at, which is what says the waits above landed where this
    // game divides the stage rather than wherever the waiting stopped.
    assert!(
        the_midboss >= MIDBOSS_ARRIVES && the_card >= CARD_STARTS,
        "the fight's chapters did not fall where its script puts them: {the_midboss}, {the_card}",
    );
    (the_stages_own_start, the_midboss, the_card, after_the_card)
}

#[test]
fn a_step_inside_a_stage_of_th06c_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = judging("th06c-stepping-inside");
        let log = game.log();
        let (start, midboss, card, after) = the_boundaries(&game);

        // Held a little way past the last of them: a step back from inside a chapter goes to that chapter's
        // own start, which is what makes a press the way onto the boundary just passed.
        game.frames(60);
        let past = game.state().stage_frames;
        game.press(HOLD);
        assert_eq!(
            game.state().stage_frames,
            past,
            "the hold let the game past the frame it was aimed at",
        );

        // Back through them, one press each. Strictly either side, so the second press does not stay on the
        // frame the first landed on.
        assert_eq!(steps(&game, BACK), after, "one press back");
        assert_eq!(steps(&game, BACK), card, "two presses back");
        assert_eq!(steps(&game, BACK), midboss, "three presses back");
        assert_eq!(steps(&game, BACK), start, "four presses back");

        // And the press after that, which has nowhere left to go inside the stage.
        game.press(BACK);
        assert!(
            log.said(&format!("step: held at the stage's start (script {start})")),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            start,
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same three: what a step forward runs to is the next start where the
        // stage has one.
        assert_eq!(steps(&game, NEXT), midboss, "one press forward");
        assert_eq!(steps(&game, NEXT), card, "two presses forward");
        assert_eq!(steps(&game, NEXT), after, "three presses forward");
    });
}

/// **And a step forward does not leave the stage**: the last boundary is a wall, and a step back from it
/// still goes where it always goes.
#[test]
fn a_step_forward_inside_a_stage_of_th06c_does_not_leave_it() {
    in_its_own_process(|| {
        let game = judging("th06c-stepping-the-wall");
        let (_, _, card, after) = the_boundaries(&game);
        game.press(HOLD);

        // Forward from the last of them, which has no boundary ahead of it inside the stage: what stops it
        // is the stage's own end, and this game's end leaves the gameplay scene for the frame its next
        // stage is built in — which is the frame `runtime::step` comes to rest on. th06's own stage does
        // the same and th07's never leaves that scene at all, so its wall is the fight being won instead.
        let held = game.log().written();
        game.press(NEXT);
        assert!(
            !game.log().said_since(held, "stage 2 chapter 1"),
            "the step forward ran on into the stage after the one it was made in:\n  {}",
            game.log().lines().split_off(held).join("\n  ")
        );
        assert!(
            game.log().said_since(held, "step: held in stage"),
            "the step forward came to rest nowhere:\n  {}",
            game.log().lines().split_off(held).join("\n  ")
        );
        // And the wall says which stage has ended and how to leave it — the stage the chapters are of,
        // which is not the one the frame says: this frame is in no stage at all.
        assert!(
            game.log().said_since(
                held,
                "step: stage 1 has ended; the across key with next or back leaves it"
            ),
            "the wall did not say which stage had ended:\n  {}",
            game.log().lines().split_off(held).join("\n  ")
        );
        // And back from that wall, which is what makes it a place to be rather than one a pass loses the
        // stage from: every chapter of the stage is still a snapshot orb holds.
        assert_eq!(steps(&game, BACK), after, "a step back from the wall");
        assert_eq!(steps(&game, BACK), card, "two presses back from the wall");
    });
}

/// **And the way out of a stage is one this game declines**, which is what the across key finds: it says so
/// and the pass stays where it was.
///
/// `Th06c::jump_to_stage` answers false, no reading of how this game's own playback is asked for another
/// stage having been made — where th06 and th07 both have one, which is what
/// `moving_between_a_replays_stages.rs` and its th07 twin are about. What matters here is that the refusal
/// is a refusal and not a pass that loses its stage trying: the boundaries of the stage are still there to
/// step between afterwards.
#[test]
fn the_across_key_over_th06c_says_the_replay_has_no_other_stage() {
    in_its_own_process(|| {
        let game = judging("th06c-stepping-across");
        let log = game.log();
        let (_, _, card, after) = the_boundaries(&game);
        // A little way past the last boundary, so that a step back afterwards has that boundary to land on:
        // from the frame one began, a step back goes to the one before it.
        game.frames(60);
        game.press(HOLD);

        let from = log.written();
        game.keyboard().set(ACROSS, true);
        game.press(NEXT);
        game.keyboard().set(ACROSS, false);
        assert!(
            log.said_since(from, "step: the replay has no stage"),
            "the across key was taken as a move this game can make:\n  {}",
            log.lines().split_off(from).join("\n  ")
        );

        // And the stage is still the one the pass was in, with its own boundaries to step between.
        assert_eq!(steps(&game, BACK), after, "a step back after the refusal");
        assert_eq!(steps(&game, BACK), card, "two presses back after it");
    });
}
