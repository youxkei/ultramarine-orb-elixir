//! **A conversation of th06nc carried by the shot, because this game's skip is a key orb cannot press.**
//!
//! Every other game orb plays hands its skip button over: `orb_bot::steer` puts `Keys::skip` into the
//! word the frame a line is up, and th07's Ctrl ends a conversation in the frames it takes to draw one.
//! th06nc has no such button among `Th06nc::BUTTONS` — its own skip bit is `0x8` of the word at
//! `0x140abae80`, built at `0x1400136f9` out of a binding this build fills per keyboard layout and not
//! out of any key orb knows to hold — so `Game::button_bit("skip")` finds nothing and `Keys::skip` is
//! nought.
//!
//! **So what moves a line on is the shot going down**, `0x140040cfa` masking that word with `0x301` —
//! the shot's `0x1` among the three — against the frame before's, and `0x140040d17` wanting
//! `[dialogue + 0x3420]`, the characters of the line that are out, at eight or more.
//!
//! **Which at one release a second is a line a second.** A Normal run the rules drove read the line up at
//! stage frames 5,125 and 5,900 with the script clock standing still throughout, and each of its first
//! two stages took about twice the frames its own script did — `orb_bot::steer`'s
//! `SHOT_LET_GO_EVERY_IN_A_CONVERSATION` is what this file is about, and it is the whole of the fix: the
//! same release, ten frames apart instead of sixty, for the frames a line is up and no others.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run is played in, counted from nought the way `State::stage` is.
const THE_FIRST_STAGE: i32 = 0;

/// The bit this game's own read leaves for the shot. Written out here rather than taken from `Th06nc`'s
/// own `button` module, which keeps the same number: a test that read its bits off the code under test
/// would agree with it whatever either said.
const SHOT: u16 = 0x0001;

/// How often the shot is let go for a frame with no line up and with one, as `orb_bot::steer`'s
/// `SHOT_LET_GO_EVERY` and `SHOT_LET_GO_EVERY_IN_A_CONVERSATION` have them.
const LET_GO_EVERY: u32 = 60;
const LET_GO_EVERY_IN_A_CONVERSATION: u32 = 10;

/// The frames a stage begins with, in which the player is coming back: the walk-in's one update and the
/// invulnerability behind it, as `the_frame_a_run_of_th06nc_acts_on` counts them.
const A_STAGES_FIRST_FRAMES: u32 = 1 + 0xf0;

/// The line a conversation is said to be at. Nought rather than the `-1` a stage with no conversation
/// carries — see `Image::no_line_of_dialogue_is_up`.
const A_LINE: i32 = 0;

/// A run of this game in one of its stages with the rules playing it, and nothing fired at the player:
/// what is under test is the word, and a hit would move the player on nobody's.
fn playing(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some("- play_until: { frames: 3000 }\n".to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(THE_FIRST_STAGE);
    game.frames(A_STAGES_FIRST_FRAMES);
    game
}

/// The frames of the next `over` on which the shot was not in the word, and how many runs of such frames
/// there were: one release of a frame apiece is the rule, so the two agreeing is what says a release was
/// a release and not the shot going quiet.
fn released_over(game: &Fake, over: u32) -> (u32, u32) {
    let (mut frames, mut runs) = (0, 0);
    let mut holding = true;
    for _ in 0..over {
        game.frame();
        let held = game.image().the_word_the_run_acts_on() as u16 & SHOT != 0;
        if !held {
            frames += 1;
            if holding {
                runs += 1;
            }
        }
        holding = held;
    }
    (frames, runs)
}

/// **With no line up, once a second**, which is the cadence a screen waiting for a press is given one at.
#[test]
fn the_shot_of_a_run_of_th06nc_is_let_go_once_a_second() {
    in_its_own_process(|| {
        let game = playing("th06nc-the-shot-let-go");
        assert_eq!(
            released_over(&game, LET_GO_EVERY * 3),
            (3, 3),
            "the shot was not let go for one frame three times over {} frames:\n  {}",
            LET_GO_EVERY * 3,
            game.log().lines().join("\n  "),
        );
    });
}

/// **And with a line up, six times as often**, which is what carries a conversation this game gives orb
/// no key to skip.
#[test]
fn the_shot_of_a_run_of_th06nc_is_let_go_six_times_as_often_while_a_line_is_up() {
    in_its_own_process(|| {
        let game = playing("th06nc-the-shot-let-go-in-a-conversation");
        game.image().a_line_of_dialogue_is_up(A_LINE);
        assert_eq!(
            released_over(&game, LET_GO_EVERY * 3),
            (
                LET_GO_EVERY * 3 / LET_GO_EVERY_IN_A_CONVERSATION,
                LET_GO_EVERY * 3 / LET_GO_EVERY_IN_A_CONVERSATION,
            ),
            "the shot was not let go every {LET_GO_EVERY_IN_A_CONVERSATION} frames over {} of them:\n  {}",
            LET_GO_EVERY * 3,
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the line taken down puts it back**, which is what keeps the faster cadence to the frames it is
/// for: a shot let go six times a second for a whole stage is six frames a second of no fire.
#[test]
fn the_shot_of_a_run_of_th06nc_goes_back_to_once_a_second_after_the_line() {
    in_its_own_process(|| {
        let game = playing("th06nc-the-shot-let-go-after-a-conversation");
        game.image().a_line_of_dialogue_is_up(A_LINE);
        released_over(&game, LET_GO_EVERY);
        game.image().no_line_of_dialogue_is_up();
        assert_eq!(
            released_over(&game, LET_GO_EVERY * 3),
            (3, 3),
            "the shot kept the conversation's own cadence with no line up:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
