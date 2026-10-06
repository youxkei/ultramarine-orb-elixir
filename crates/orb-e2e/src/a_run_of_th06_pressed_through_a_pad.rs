//! **A launch of th06 pressing through a pad** — `--through-a-pad` — whose recipe reaches the game through the
//! pad half orb does for it (`Th06::pad_word`, over the controller the game holds and `joystick::reading`), the
//! virtual pad being one of the pads that reading has.
//!
//! **What says a press went through the pad is a button the mapping has none for**, as th07's file has it: a word
//! handed to the read has it, and a word pressed on the pad cannot.

use crate::fake::th06::{Fake, MAPPING, the_run};
use crate::fake::{Launched, WINDOW, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_core::game::th06::image::Mapping;

/// The bits of th06's word the shot and the bomb are.
const SHOT: u16 = 0x1;
const BOMB: u16 = 0x2;

/// Another program's window in front.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// **A run of a recipe holding the shot and the bomb, the bomb on no button**, pressing through the pad or not —
/// the front end pressed through by the recipe's own first step, as `the_bot_playing_th06`'s runs are.
fn playing(name: &str, through_a_pad: bool) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.autoplay = Some(
            "- press: [decide]\n  frames: 2\n  until: { stage: 1, chapter: 1 }\n  give_up_after: 1800\n\
             - press: [shot, bomb]\n  frames: 100000\n"
                .to_owned(),
        );
        config.through_a_pad = through_a_pad;
        config.log_level = LogLevel::Verbose;
    });
    game.image().maps_the_pad(Mapping {
        bomb: -1,
        ..MAPPING
    });
    game.in_a_pointdevice_run();
    game.frames(10);
    game
}

/// **Through the pad, the shot reaches the run and the bomb does not**, there being no button to press it on —
/// and behind another program's window as well, where the keyboard is not read.
#[test]
fn a_recipes_press_reaches_th06_through_the_pad_by_its_mapping() {
    in_its_own_process(|| {
        let game = playing("th06-through-a-pad", true);
        let log = game.log();
        assert!(
            log.said("autoplay: the run is pressed through the virtual pad"),
            "the run was not pressed through the pad:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().input_now() & (SHOT | BOMB),
            SHOT,
            "the run did not read the shot alone off a pad with no bomb button",
        );
        game.sim().display().set_foreground(ANOTHER);
        game.frames(10);
        assert_eq!(
            game.image().input_now() & SHOT,
            SHOT,
            "the pad's shot did not reach the run with the window behind",
        );
    });
}

/// **And the host's own pad reaches no run pressed through the virtual pad**: its bomb held on winmm's joystick 0,
/// read by orb's sampling, and the run acting on the recipe's shot alone. A run pressed through the virtual pad is
/// orb's, and a pad somebody had in hand pressing into one bombed th06nc's no-hit Extra on 917 frames (orb-real
/// 1791701497).
#[test]
fn the_hosts_own_pad_reaches_no_run_of_th06_pressed_through_a_pad() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-the-hosts-pad-unread", the_run(), |config| {
            config.autoplay = Some(
                "- press: [decide]\n  frames: 2\n  until: { stage: 1, chapter: 1 }\n  give_up_after: 1800\n\
                 - press: [shot]\n  frames: 100000\n"
                    .to_owned(),
            );
            config.through_a_pad = true;
            config.log_level = LogLevel::Verbose;
        });
        game.image().no_controller();
        game.sim()
            .joystick()
            .attach(HOSTS_BUTTONS, HOSTS_TRAVEL.0, HOSTS_TRAVEL.1);
        game.in_a_pointdevice_run();
        let bomb = u32::try_from(MAPPING.bomb).expect("the mapping has a bomb button");
        game.sim().joystick().pushes(1 << bomb, HOSTS_CENTRE);
        for _ in 0..WAITS {
            if game.log().said(HOSTS_PAD_READ) {
                break;
            }
            game.frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(
            game.log().said(HOSTS_PAD_READ),
            "orb's sampling never read the host's pad:\n  {}",
            game.log().lines().join("\n  ")
        );
        game.frames(10);
        assert_eq!(
            game.image().input_now() & (SHOT | BOMB),
            SHOT,
            "the host's own pad's bomb reached a run pressed through the virtual pad",
        );
    });
}

/// The host's pad, as `mode_on_a_winmm_pad` plugs one in: sixteen buttons over a whole 16-bit travel, resting in
/// the middle of it.
const HOSTS_BUTTONS: u32 = 16;
const HOSTS_TRAVEL: (u32, u32) = (0, 65535);
const HOSTS_CENTRE: u32 = HOSTS_TRAVEL.1 / 2;

/// The line orb's sampling thread writes once it has read that pad.
const HOSTS_PAD_READ: &str = "joystick: winmm 0 is";

/// How long that thread is given, in frames with a millisecond of real time each.
const WAITS: u32 = 3000;

/// **And handed to the read where the launch presses through no pad**, both bits reaching the run.
#[test]
fn a_recipes_press_is_handed_to_th06s_read_where_the_launch_presses_through_no_pad() {
    in_its_own_process(|| {
        let game = playing("th06-through-no-pad", false);
        assert_eq!(
            game.image().input_now() & (SHOT | BOMB),
            SHOT | BOMB,
            "the recipe's word did not reach the run whole",
        );
    });
}
