//! **A launch of th07 pressing through a pad** — `--through-a-pad` — whose recipe reaches the game through the
//! pad half orb does for it (`Th07::pad_word` over `joystick::reading`), the virtual pad being one of the pads that
//! reading has.
//!
//! **What says a press went through the pad is a button the mapping has none for**: a pad cannot press it, so a
//! word handed to the read has it and a word pressed on the pad does not.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, WINDOW, Work, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bits of th07's word the shot and the bomb are.
const SHOT: u16 = 0x1;
const BOMB: u16 = 0x2;

/// **A mapping of nobody's defaults, the bomb on no button at all**: the shot on the seventh button, the focus on
/// the third, the pause on the fifth and the skip on the fourth.
const MAPPED: [i16; 9] = [6, -1, 2, 4, -1, -1, -1, -1, 3];

/// Another program's window in front.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// **A run of a recipe holding the shot and the bomb**, pressing through the pad or not — the front end pressed
/// through by the recipe's own first step, the bot answering every read of the game's keys.
fn playing(name: &str, through_a_pad: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(
                "- press: [decide]\n  frames: 2\n  until: { stage: 1, chapter: 1 }\n  give_up_after: 1800\n\
                 - press: [shot, bomb]\n  frames: 100000\n"
                    .to_owned(),
            );
            config.through_a_pad = through_a_pad;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.image().the_pad_is_mapped(MAPPED);
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game.frames(10);
    game
}

/// **Through the pad, the shot reaches the player on the button the mapping names and the bomb does not**, there
/// being no button to press it on.
#[test]
fn a_recipes_press_reaches_th07_through_the_pad_by_its_mapping() {
    in_its_own_process(|| {
        let game = playing("th07-through-a-pad", true);
        let log = game.log();
        assert!(
            log.said("autoplay: the run is pressed through the virtual pad"),
            "the run was not pressed through the pad:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.the_player_read() & (SHOT | BOMB),
            SHOT,
            "the player did not read the shot alone off a pad with no bomb button",
        );

        // And behind another program's window, where the keyboard is not read and the pad is.
        game.sim().display().set_foreground(ANOTHER);
        game.frames(10);
        assert_eq!(
            game.the_player_read() & SHOT,
            SHOT,
            "the pad's shot did not reach the player with the window behind",
        );
    });
}

/// **And handed to the read where the launch presses through no pad**, both bits reaching the player.
#[test]
fn a_recipes_press_is_handed_to_th07s_read_where_the_launch_presses_through_no_pad() {
    in_its_own_process(|| {
        let game = playing("th07-through-no-pad", false);
        assert_eq!(
            game.the_player_read() & (SHOT | BOMB),
            SHOT | BOMB,
            "the recipe's word did not reach the player whole",
        );
    });
}
