//! **The stepping keys over th10: one chapter at a time, inside the stage.**
//!
//! What `stepping_inside_a_stage_of_th08.rs` asks of that game, asked of this one. Both go through
//! `Chapters::previous_start` and `next_start`, whose whole job is that a step lands **strictly** either side
//! of where the game is — so a press moves rather than staying put, and a frame or two past a boundary steps
//! back onto it.
//!
//! **The boundaries are the stage's two fights**, stage 1 having no row of the midstage table: the stage's own
//! start, the midboss's two attacks and the waves after her, and the stage's own boss arriving, and the card she
//! puts up and the nonspell after it — see `fake::th10`'s `THE_MIDBOSS_ARRIVES_AT` and `THE_BOSS_FIGHTS_FOR`.
//! **And the wall at the far end is her going down**, which this game's stage does some frames before its
//! next stage is asked for (`AFTER_THE_BOSS`), so what a step forward comes to rest on is the stage won.
//!
//! **The run is a replay the game is playing**, the game object's `+0x5c` (0x41811d through 0x418121), which is
//! what the stepping keys watch: a pass judges a replay and not a run somebody is in the middle of.

use orb_config::LogLevel;

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The keys a pass steps with: the numbers, orb's own names for them being its own.
const HOLD: u8 = orb_config::keys::SPACE.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;
/// And the one held with a step to leave the stage altogether.
const ACROSS: u8 = orb_config::keys::SHIFT.0;

/// **How long a stage lasts here**: room for the midboss from frame 1200 before the boss, who arrives for the
/// stage's last 900 frames and goes 120 before its end.
const STAGE_FRAMES: u32 = 3_600;

/// How many frames a wait here is given.
const FRAMES: u32 = STAGE_FRAMES * 3;

/// The chapters of the stage up to the boss's arrival, as a pass's log says each one begins: the stage's start,
/// the midboss's nonspell and card, the waves after her and the boss's first nonspell.
const UP_TO_THE_BOSS: [&str; 5] = [
    "stage 1 chapter 1 (stage start)",
    "stage 1 chapter 2 at frame 1200 (script 1200): a midboss nonspell",
    "stage 1 chapter 3 at frame 1500 (script 1500): a midboss spellcard",
    "stage 1 chapter 4 at frame 1800 (script 1800): the midboss was beaten",
    "stage 1 chapter 5 at frame 2700 (script 2700): a boss nonspell",
];

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

/// **The launch a judging pass is**: the stepping keys on over a replay, placed rather than chosen off the
/// list, with nothing fired at the player, a replay being no run to be hit in.
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
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.image().the_run_is_a_replay_being_played();
    game.frames_until("the stage's own first chapter", FRAMES, || {
        game.log().said(UP_TO_THE_BOSS[0])
    });
    game
}

/// The boundaries of this stage up to the boss's arrival, each waited for and the frame it began on kept.
fn the_boundaries(game: &Fake) -> Vec<u32> {
    let log = game.log();
    let mut began = vec![game.state().stage_frames];
    for chapter in &UP_TO_THE_BOSS[1..] {
        game.frames_until(chapter, FRAMES, || log.said(chapter));
        began.push(game.state().stage_frames);
    }
    assert!(
        game.state().boss_present,
        "the chapters did not fall where the fights put them: {began:?}",
    );
    began
}

#[test]
fn a_step_inside_a_stage_of_th10_lands_on_the_chapter_start_either_side_of_where_the_game_is() {
    in_its_own_process(|| {
        let game = judging("th10-stepping-inside");
        let log = game.log();
        let began = the_boundaries(&game);

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
        for (presses, at) in began.iter().rev().enumerate() {
            assert_eq!(steps(&game, BACK), *at, "{} press(es) back", presses + 1);
        }

        // And the press after that, which has nowhere left to go inside the stage.
        game.press(BACK);
        assert!(
            log.said("step: held at the stage's start"),
            "the press below the first chapter did not say where it stopped:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.state().stage_frames,
            began[0],
            "the press below the first chapter left the stage",
        );

        // And forward again, over the same ones: what a step forward runs to is the next start where the
        // stage has one.
        for (presses, at) in began.iter().skip(1).enumerate() {
            assert_eq!(steps(&game, NEXT), *at, "{} press(es) forward", presses + 1);
        }
    });
}

/// **And a step forward does not leave the stage**: the stage won is a wall, and a step back from it still
/// goes where it always goes.
#[test]
fn a_step_forward_inside_a_stage_of_th10_does_not_leave_it() {
    in_its_own_process(|| {
        let game = judging("th10-stepping-the-wall");
        let began = the_boundaries(&game);
        let boss = began[began.len() - 1];
        let before_her = began[began.len() - 2];
        game.press(HOLD);

        // Forward over the rest of her fight first, which has two more starts: the card she puts up and the
        // nonspell after it.
        let card = steps(&game, NEXT);
        let after = steps(&game, NEXT);
        assert!(
            boss < card && card < after,
            "the steps forward over her fight did not each move on: {boss}, {card}, {after}",
        );

        let held = game.log().written();
        game.press(NEXT);
        assert!(
            !game.log().said_since(held, "stage 2 chapter 1"),
            "the step forward ran on into the stage after the one it was made in:\n  {}",
            game.log().lines().split_off(held).join("\n  ")
        );
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
        assert_eq!(steps(&game, BACK), boss, "three presses back from the wall");
        assert_eq!(
            steps(&game, BACK),
            before_her,
            "four presses back from the wall"
        );
    });
}

/// **And the way out of a stage is one this game declines**, which is what the across key finds: it says so
/// and the pass stays where it was. `Th10::jump_to_stage` answers false, no reading of how this game's own
/// playback is asked for another stage having been made.
#[test]
fn the_across_key_over_th10_says_the_replay_has_no_other_stage() {
    in_its_own_process(|| {
        let game = judging("th10-stepping-across");
        let log = game.log();
        let began = the_boundaries(&game);
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
        assert_eq!(
            steps(&game, BACK),
            began[began.len() - 1],
            "a step back after the refusal"
        );
        assert_eq!(
            steps(&game, BACK),
            began[began.len() - 2],
            "two presses back after it"
        );
    });
}
