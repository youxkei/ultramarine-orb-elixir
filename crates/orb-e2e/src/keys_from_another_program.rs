//! **`--sent-keys`: the game reading keys another program pressed, which is what drives an unwatched run.**
//!
//! What each scenario holds is the measurement it has to reproduce, taken off 東方紅魔郷 1.02h on this
//! machine.
//!
//! **A key another program sent is not a key a hand is holding, and the difference is which read sees it.**
//! `orb_sim::Keyboard::sends` is the first — `GetKeyboardState` reports it and
//! `orb_sim::Keyboard::held` does not — and the laid-out 紅魔郷 reads its keyboard through whichever of the
//! two the game is on: the device it took `DISCL_EXCLUSIVE | DISCL_FOREGROUND` while it still holds one,
//! and `GetKeyboardState` once orb has let that device go. Which of those the game is on is the pointer at
//! `g_Supervisor + 0x10`, because that is what the game's own read branches on and what orb clears.
//!
//! Why it matters past automation: every measurement of a front end that nobody was there to press keys
//! at was taken through this, so a launch that cannot be driven is a launch that cannot be measured.

use crate::fake::th06::{DEMO_AFTER, Fake, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen};
use orb_sim::keys;

/// How long a scenario gives the title screen to fall into its attract demo, in frames.
const INTO_THE_DEMO: u32 = DEMO_AFTER as u32 * 2;

/// Runs frames until the attract demo is not only asked for but *running*.
///
/// Two waits and not one, because the frame the demo's scene is asked for is a frame the supervisor spends
/// building it — the update returns before any of the scene's own jobs — so a key pressed on it is a key
/// nothing was there to read. Which is the same one-frame gap every scene change here has, and not one of
/// the two moments this file is about.
///
/// # Panics
/// Naming whichever of the two did not happen.
fn into_the_demo(game: &Fake) {
    game.frames_until("the attract demo asked for", INTO_THE_DEMO, || {
        game.image().scene() == Scene::Playing
    });
    game.frames_until("the attract demo running", 8, || {
        game.state().stage_frames > 0
    });
}

/// Keys another program sends reach the system and do not reach the game, until orb lets the device go.
///
/// Measured: keys injected with `SendInput` — tried carrying the virtual key with its scancode, and as
/// the scancode alone with `KEYEVENTF_SCANCODE` — are accepted by the system (`SendInput` returns **1**)
/// and not seen by the game, which sat idle into its attract demo twice. `Controller::GetInput` takes the
/// keyboard `DISCL_EXCLUSIVE | DISCL_FOREGROUND` and such a device does not see them.
///
/// `--sent-keys` has orb let that device go — `Unacquire`, `Release`, the pointer cleared, which is what
/// `Supervisor::RegisterChain` does with a device it cannot set up — and the game then reads
/// `GetKeyboardState`, its own other way, which does see them. The first press after that ended the
/// attract demo, which is what proved it.
#[test]
fn an_injected_key_reaches_the_game_only_once_orb_has_released_its_device() {
    in_its_own_process(|| {
        // Without `--sent-keys` first, which is the launch the measurement was taken against: the game
        // keeps the device it took exclusively for its whole life.
        {
            let game = Fake::attach("sent-keys-refused", the_run(), |config| {
                config.log_level = LogLevel::Verbose;
            });
            game.demos_when_idle();
            game.at_the_title_menu();
            assert!(
                game.image().holds_a_keyboard_device(),
                "the game has no device for an injected key to be refused by",
            );

            // Sent rather than pressed, and the system took it: `GetKeyboardState` says it is down.
            game.keyboard().sends(keys::Z, true);
            assert!(
                orb_api::keyboard::state().is_some_and(|state| state[keys::Z as usize] & 0x80 != 0),
                "the host did not take the key another program sent",
            );

            // And the game read nothing: it sat idle into its attract demo with that key sent throughout,
            // which is what happened twice on the machine.
            game.frames_until("the attract demo", INTO_THE_DEMO, || {
                game.image().scene() == Scene::Playing
            });
            assert!(
                game.state().demo,
                "the run the title screen started is not its attract demo",
            );
            // And goes on not reading it: the demo is not left by a key it cannot see.
            game.frames(INTO_THE_DEMO);
            assert!(
                game.state().demo,
                "the injected key ended the attract demo through a device that cannot see it",
            );
        }

        // And with `--sent-keys`: orb lets the device go, which sends the game's own read down its
        // `GetKeyboardState` branch — the read that does see a sent key.
        let game = Fake::attach("sent-keys-taken", the_run(), |config| {
            config.sent_keys = true;
            config.log_level = LogLevel::Verbose;
        });
        game.demos_when_idle();
        game.frames_until("the device let go of", 8, || {
            !game.image().holds_a_keyboard_device()
        });
        assert!(
            game.log().said(
                "input: the game's own keyboard device let go of; keys sent to it are read the other way"
            ),
            "orb did not say it had let the device go:\n  {}",
            game.log().lines().join("\n  ")
        );

        // The same key sent the same way, and now the game is driven by it: the title screen falls into its
        // demo with nothing sent, and the first press after that ends it.
        game.at_the_title_menu();
        into_the_demo(&game);
        assert!(game.state().demo);
        game.keyboard().sends(keys::Z, true);
        game.frames_until("the attract demo ended by a sent key", 8, || {
            game.image().scene() == Scene::FrontEnd
        });
        // The flag goes with the run when the front end is built, a frame later: it belongs to the run and
        // not to the screen, which is why `read_state` reads it out of the game manager.
        game.frames_until("the front end built over it", 8, || !game.state().demo);
    });
}

/// A press has to be repeated, because two moments in the front end spend one on nothing.
///
/// Measured, two things: a press inside the title's own opening animation is spent on nothing, and the
/// attract demo eats one to leave. So what works is **pressing again until the log says the screen
/// moved**, rather than one press per screen and hoping.
#[test]
fn a_press_is_repeated_until_the_screen_moves_because_two_moments_swallow_one() {
    in_its_own_process(|| {
        let game = Fake::attach("sent-keys-repeated", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.demos_when_idle();

        // The first of the two: the title's own opening animation, which the screen ignores a press over —
        // `MENU_TITLE_GRACE_FRAMES`, and `FrontEnd::acts_on_a_press` is the game's own answer about it. A
        // press inside those frames moves nothing.
        game.frames_until("an overlay", 8, || game.log().said("overlay: ready"));
        assert!(
            !game.image().front_end_now().acts_on_a_press(),
            "the title screen is already past its opening animation, so there is nothing here to \
             swallow a press",
        );
        game.press(keys::Z);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Title,
            "a press inside the opening animation moved the screen",
        );

        // The second: the attract demo, which eats the press that leaves it. So the press that ends the
        // demo is not the press that starts a run — the menu underneath never sees it.
        game.at_the_title_menu();
        into_the_demo(&game);
        assert!(game.state().demo);
        game.press(keys::Z);
        assert_eq!(
            game.image().scene(),
            Scene::FrontEnd,
            "the press did not leave the attract demo:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Title,
            "the press that left the demo went on to start a run as well",
        );

        // Which is why one press per screen does not work and pressing until the screen moves does: from
        // here the same key, repeated, gets to the shot type select — through the frames the title screen
        // ignores a press over on the way back, which is a third helping of the first moment.
        game.press_until(keys::Z, "the run started", || {
            game.image().front_end_now().screen == Screen::ShotType
        });
    });
}
