//! **A launch of th06nc pressing through a pad** — `--through-a-pad` — the same claims
//! `a_run_of_th06c_pressed_through_a_pad` makes, over the game whose word and pad are th06c's bit for bit: the
//! recipe's press onto the virtual pad, through orb's sample of the pads, and into the `XInputGetState` orb
//! answers this game with — and the machine's own pad reaching the run beside it. The front end walked through
//! the pad is `the_cases_of_th06nc_the_real_game_is_tested_with`'s, every chain there pressing through it.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::button as xinput;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// The bits of the word the run acts on the shot and the bomb are, as the game's pad read sets them for `A` and
/// `B`.
const SHOT: u32 = 0x1;
const BOMB: u32 = 0x2;

/// The slot the machine's own pad is in, which is where the development machine's is.
const SLOT: u32 = 0;

/// How long orb's own sampling thread is given to read the machine's pad, in frames with a millisecond of real
/// time each.
const WAITS: u32 = 3000;

/// A launch of a recipe holding the shot, through the pad or not, in a run in the first stage.
fn in_a_run(name: &str, through_a_pad: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some("- press: [shot]\n  frames: 100000\n".to_owned());
            config.through_a_pad = through_a_pad;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.in_a_run_in_the_stage(1);
    game
}

/// The machine's own pad plugged into XInput's first slot with `B` held, once orb's sampling has read it.
fn the_machines_pad_holds_the_bomb(game: &Fake) {
    game.sim().xinput().attach(SLOT);
    game.sim().xinput().pushes(SLOT, xinput::B);
    for _ in 0..WAITS {
        if game
            .log()
            .said(&format!("joystick: xinput {SLOT} has a pad"))
        {
            game.frames(10);
            return;
        }
        game.frame();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!(
        "orb's sampling never read the pad in xinput {SLOT}:\n  {}",
        game.log().lines().join("\n  ")
    );
}

/// **Through the pad, the recipe's shot and the machine's own pad's bomb reach the run together.**
#[test]
fn a_recipes_press_reaches_th06nc_through_the_pads_beside_the_machines_own() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-through-a-pad", true);
        game.frames(10);
        let log = game.log();
        assert!(
            log.said("autoplay: the run is pressed through the virtual pad"),
            "the run was not pressed through the pad:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_word_the_run_acts_on() & SHOT,
            SHOT,
            "the recipe's shot did not reach the run through the pad",
        );
        the_machines_pad_holds_the_bomb(&game);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & (SHOT | BOMB),
            SHOT | BOMB,
            "the machine's own pad's bomb did not reach the run beside the virtual pad's shot",
        );
    });
}

/// **And where the launch presses through no pad, the game's pad is the bot's word alone**, as it always was.
#[test]
fn a_recipes_press_is_th06ncs_whole_pad_where_the_launch_presses_through_no_pad() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-through-no-pad", false);
        the_machines_pad_holds_the_bomb(&game);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & (SHOT | BOMB),
            SHOT,
            "the run did not act on the recipe's word alone",
        );
    });
}
