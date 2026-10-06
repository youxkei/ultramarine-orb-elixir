//! **A launch of th06c pressing through a pad** — `--through-a-pad` — whose recipe reaches the game the way a
//! person's pad does: onto the virtual pad, through orb's sample of the pads, and into the `XInputGetState` orb
//! answers this game with (`runtime::xinput_get_state`), the word read off that sample as th06's and th07's pad
//! half reads one.
//!
//! **And the machine's own pad reaches neither**: a launch that hands the bot's word to the read answers this
//! game's pad with that word alone, and one pressing through the pads with the virtual pad alone — a run pressed
//! through it is orb's, and a pad somebody had in hand pressing into one bombed th06nc's no-hit Extra on 917
//! frames (orb-real 1791701497).

use crate::fake::th06c::{Fake, TO_A_RUN};
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

/// A recipe holding the shot for as long as the test runs.
const HOLDING_THE_SHOT: &str = "- press: [shot]\n  frames: 100000\n";

/// A launch of `recipe`, through the pad or not, in a run in the first stage.
fn in_a_run(name: &str, through_a_pad: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(HOLDING_THE_SHOT.to_owned());
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

/// **Through the pad, the recipe's shot reaches the run and the machine's own pad's bomb does not.**
#[test]
fn a_recipes_press_reaches_th06c_through_the_pad_in_place_of_the_machines_own() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-through-a-pad", true);
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
            SHOT,
            "the machine's own pad's bomb reached a run pressed through the virtual pad",
        );
    });
}

/// **And where the launch presses through no pad, the game's pad is the bot's word alone**, as it always was:
/// the machine's own pad does not reach the run.
#[test]
fn a_recipes_press_is_th06cs_whole_pad_where_the_launch_presses_through_no_pad() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-through-no-pad", false);
        the_machines_pad_holds_the_bomb(&game);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & (SHOT | BOMB),
            SHOT,
            "the run did not act on the recipe's word alone",
        );
    });
}

/// **And the front end walked through the pad**: the title's screens and selects, orb's own question of the mode
/// answered off what the pads are doing, and the run entered.
#[test]
fn th06cs_front_end_and_orbs_question_are_walked_through_a_pad() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-front-end-through-a-pad",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "{TO_A_RUN}\
- press: [decide]
  frames: 6
  until: {{ stage: 1, chapter: 1 }}
  give_up_after: 1800
- play_until: {{ frames: 100000 }}
"
                ));
                config.through_a_pad = true;
                config.log_level = LogLevel::Verbose;
            },
        );
        let log = game.log();
        for _ in 0..40_000 {
            if game.state().in_game {
                break;
            }
            game.frame();
        }
        assert!(
            game.state().in_game,
            "the run never started:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("mode: answered on the pad"),
            "orb's question was not answered on the pad:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("autoplay: this game takes no word as a pad"),
            "the word was handed to the read:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
