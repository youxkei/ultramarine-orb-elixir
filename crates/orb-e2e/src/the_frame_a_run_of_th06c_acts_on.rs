//! **The keys a frame of th06c is played with reach that frame's own update, and the word beside them is the
//! frame before's.**
//!
//! th07's own answer is `the_frame_a_run_of_th07_acts_on.rs`, and there it is a fix: that game reads the keys
//! at chain priority 0, moves the player at 8 and only copies the read into the word the run acts on at 0x10,
//! so the player moves on the frame before's keys until orb hands the read to the run.
//!
//! **This game has no such gap to close**, and where that was read is `th06c/asm.md`'s *Where that word comes
//! from*: the job at `0x140058330` is the calc chain's priority nought, and inside one walk of it
//! `0x1400583b8` copies `0x140955390` into `0x140955394`, `0x1400583c4` calls `0x14000fa80` for the new word
//! and `0x1400583cf` writes it back — the word is built, the player moves on it and the bullets test against
//! where it moved, all in that same walk. So `Th06c` hands no read to the run and this file says what is left
//! to say: that nothing of orb's puts a frame between the keys and the update, and that the pair an edge is
//! taken from is still a pair.
//!
//! **Which the pair is for**: `0x14004a683` holds `0x140955390` against `0x140955394` where a bomb is
//! pressed, so a key held over two frames has to read as a press on the first and a hold on the second. A
//! word moved on without its predecessor would read as a hold on every frame and no bomb would ever go off.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below is played in, orb's own 0-based number for it.
const THE_FIRST_STAGE: i32 = 0;

/// The bit this game's own read leaves for up, which the walk at `0x14004ad29` tests first of the four.
/// Written out here rather than taken from `Th06c`'s own `button` module, which keeps the same number: a test
/// that read its bits off the code under test would agree with it whatever either said.
const UP: u16 = 0x0010;

/// And the bit it leaves for the focus, which the player's walk reads for the slower speeds.
const FOCUS: u16 = 0x0004;

/// The state a run is played in, and how many frames of a stage go by before the player is in it: the
/// walk-in and the invulnerability behind it, which `the_player_a_stage_of_th06c_starts` reads off the exe.
const PLAYING: u8 = 0;
const A_STAGES_FIRST_FRAMES: u32 = 30 + 0xf0;

/// And a second of frames on top of that as the wall this test puts up, rather than the count itself: what
/// is under test here is the word a frame is played with and not which frame the walk-in ends on.
const A_SECOND: u32 = 60;

#[test]
fn a_frame_of_th06c_is_played_with_the_keys_that_frame_was_given() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-frame-it-acts-on",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);
        // **Past the frames a stage begins with**, in which the player is walking back in and moves on no
        // word at all — see `the_player_a_stage_of_th06c_starts`, which is how long that is and why.
        game.frames_until(
            "the player playing",
            A_STAGES_FIRST_FRAMES + A_SECOND,
            || game.image().the_players_state() == PLAYING,
        );
        let was = game.image().where_the_player_is();

        // Up held, and the one frame it is held for run.
        game.keyboard().set(keys::UP, true);
        game.frame();
        assert_eq!(
            game.image().the_word_the_run_acts_on() as u16 & UP,
            UP,
            "the game's own read did not leave the key that was held",
        );
        // **And the player has moved on it on that same frame**, which is the half a frame of lag would
        // break: the word is read at priority 0 and the player's own job is behind it in the same walk.
        let now = game.image().where_the_player_is();
        assert!(
            now.1 < was.1,
            "the player is still where they were, so the run acted on a word the key is not in yet: \
             {was:?} against {now:?}",
        );

        // **And the pair an edge is taken from is still a pair**: the word beside the one the run acts on
        // holds the frame before's, so the second frame of a held key is a hold and not another press.
        game.frame();
        assert_eq!(
            (
                game.image().the_word_the_run_acts_on() as u16 & UP,
                game.image().the_word_before() as u16 & UP,
            ),
            (UP, UP),
            "the word beside the one the run acts on is not the frame before's",
        );

        // And the key let go of leaves the word on its own frame, which is the same thing said the other way
        // round: a word that lags going up is a shot that keeps firing after the key is released.
        game.keyboard().set(keys::UP, false);
        game.frame();
        assert_eq!(
            game.image().the_word_the_run_acts_on() as u16 & UP,
            0,
            "the run acts on a word the key is still in, a frame after it was let go of",
        );
        assert_eq!(
            game.image().the_word_before() as u16 & UP,
            UP,
            "the frame before's word does not hold the key that was down on it",
        );
    });
}

/// **The field a pause asks for is captured on its frame**: the pause writes its request into the renderer
/// where the configuration has it show the field behind it (0x1400093b8), and the game's frame serves it in
/// the tail of its wait (0x140025da4 through 0x140025dd8) — a wait orb's own frame leaves out, so orb serves
/// the request itself and puts it back.
#[test]
fn the_field_a_pause_of_th06c_asks_for_is_captured() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-field-behind-a-pause",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.image().the_pause_shows_the_field();
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);
        game.frames_until(
            "the player playing",
            A_STAGES_FIRST_FRAMES + A_SECOND,
            || game.image().the_players_state() == PLAYING,
        );
        game.keyboard().set(keys::ESCAPE, true);
        game.frames_until("the pause up", A_SECOND, || game.image().is_it_paused());
        game.frame();
        assert_eq!(
            game.image().the_capture_asked_for(),
            -1,
            "the field the pause asked for was never captured",
        );
    });
}

/// **Up held with down moves the player up**, and right held with left moves them right: the player's walk at
/// 0x14004a4b0 asks the word for its directions in an order and takes the first it finds, so the two of a pair
/// held together are one direction and not none — which a word that is the keyboard's and a pad's at once can
/// be.
#[test]
fn up_and_down_held_together_move_a_player_of_th06c_up() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-up-and-down-at-once",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);
        game.frames_until(
            "the player playing",
            A_STAGES_FIRST_FRAMES + A_SECOND,
            || game.image().the_players_state() == PLAYING,
        );
        let was = game.image().where_the_player_is();
        game.keyboard().set(keys::UP, true);
        game.keyboard().set(keys::DOWN, true);
        game.keyboard().set(keys::LEFT, true);
        game.keyboard().set(keys::RIGHT, true);
        game.frame();
        let now = game.image().where_the_player_is();
        assert!(
            now.1 < was.1 && now.0 > was.0,
            "the four directions at once moved the player from {was:?} to {now:?} and not up and to the right",
        );
    });
}

/// **A focus a recipe presses is in the word the run acts on**: this game's table of keys at `0x1402bc0b0`
/// holds the shifts as `0xa0 -> 0x2a` and `0xa1 -> 0x36` and no `VK_SHIFT` at all, so a focus that goes in
/// under `0x10` alone is a key the game's read copies nothing for.
#[test]
fn a_focus_a_recipe_presses_reaches_the_word_a_run_of_th06c_acts_on() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-a-focus-pressed",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
                config.autoplay = Some("- press: [focus]\n  frames: 100000\n".to_owned());
            },
        );
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);
        game.frames_until(
            "the player playing",
            A_STAGES_FIRST_FRAMES + A_SECOND,
            || game.image().the_players_state() == PLAYING,
        );
        game.frames_until("the focus in the word the run acts on", A_SECOND, || {
            game.image().the_word_the_run_acts_on() as u16 & FOCUS == FOCUS
        });
    });
}
