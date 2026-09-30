//! **A pad playing th10**: its buttons and its stick through the game's own read, and its d-pad through orb's.
//!
//! th10's pad half of the read, `0x44a190`, is called through and not done instead — `game::Read::PadHalf` — so
//! the buttons the game's mapping names and the stick past a quarter of its travel reach the word the game
//! stores as the game makes them: `joyGetPosEx` on joystick 0 where no DirectInput device was found (0x44a1f7),
//! the mapping's five buttons (0x44a205 through 0x44a291) and the two axes (0x44a293 through 0x44a315).
//!
//! **What that read never looks at is the hat**, where a d-pad reports: its `JOYINFOEX` asks for every field and
//! reads the buttons and the two axes, and its DirectInput branch reads `lX`, `lY` and the buttons. So a d-pad
//! does nothing in the game while driving orb's own menus, and `dpad_moves` is what closes that — the direction
//! the hat points added to the word the pad half answered, in the bits that word names the directions by.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A pad on the host, as th07's file declares one.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// Where a hat points, in the hundredths of a degree clockwise from straight up that a pad reports one in.
const HAT_LEFT: u32 = 27_000;
const HAT_RIGHT: u32 = 9_000;

/// The bits of th10's own word a direction and the buttons are (0x44a205 through 0x44a315).
const SHOT: u16 = 0x1;
const BOMB: u16 = 0x2;
const SKIP: u16 = 0x100;
const DOWN: u16 = 0x20;
const LEFT: u16 = 0x40;
const RIGHT: u16 = 0x80;
const DIRECTIONS: u16 = 0xf0;

/// **Which pad button is which with no `th10.cfg` read over the mapping**: `0, 1, 2, 3, -1, -1, -1, -1, 4` in the
/// exe's own `.data` at `0x474e88` — the shot, the bomb, the focus, the pause, four unused and the skip.
const SHOT_BUTTON: u32 = 0;
const BOMB_BUTTON: u32 = 1;
const SKIP_BUTTON: u32 = 4;

fn centred() -> u32 {
    (PAD_TRAVEL.0 + PAD_TRAVEL.1) / 2
}

/// A run with a pad on the host and orb's own thread having read it once.
fn playing(name: &str, settings: impl FnOnce(&mut orb_config::Config)) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            settings(config);
        },
    );
    let log = game.log();
    game.sim()
        .joystick()
        .attach(PAD_BUTTONS, PAD_TRAVEL.0, PAD_TRAVEL.1);
    game.frames_until_a_thread("the pad named", || log.said("joystick: winmm 0 is"));
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.one_frame();
    game
}

/// Frames until the word the game's own read answered is exactly `word`, waited for rather than counted: orb's
/// sample is taken on a thread nothing here drives.
fn the_read_answers(game: &Fake, what: &str, word: u16) {
    game.frames_until_a_thread(what, || game.word_the_input_read_answered() == word);
}

/// The buttons the game's mapping names reach the word, each as the bit the mapping makes it, and the stick past
/// a quarter of its travel is a direction — all of it the game's own read.
#[test]
fn the_pads_buttons_and_stick_play_th10_through_its_own_read() {
    in_its_own_process(|| {
        let game = playing("th10-the-pad-plays", |_| {});
        game.frames(10);
        assert_eq!(
            game.word_the_input_read_answered(),
            0,
            "the word held something with the pad at rest",
        );

        game.sim().joystick().pushes(1 << SHOT_BUTTON, centred());
        game.frames(1);
        assert_eq!(
            game.word_the_input_read_answered(),
            SHOT,
            "the shot button did not shoot"
        );
        game.sim()
            .joystick()
            .pushes(1 << BOMB_BUTTON | 1 << SKIP_BUTTON, centred());
        game.frames(1);
        assert_eq!(
            game.word_the_input_read_answered(),
            BOMB | SKIP,
            "the bomb and skip buttons are not the bits the mapping makes them",
        );

        // Past a quarter of the travel, and short of it: the dead zone is the game's own.
        game.sim()
            .joystick()
            .pushes(0, centred() + PAD_TRAVEL.1 / 4 - 1);
        game.frames(1);
        assert_eq!(
            game.word_the_input_read_answered(),
            0,
            "the stick inside its dead zone was read as a direction",
        );
        game.sim()
            .joystick()
            .pushes(0, centred() + PAD_TRAVEL.1 / 4 + 1);
        game.frames(1);
        assert_eq!(
            game.word_the_input_read_answered(),
            DOWN,
            "the stick down was not down"
        );
    });
}

/// The d-pad moves the player, in the direction it points and every frame it is held — orb's addition to the word
/// the game's own read answered.
#[test]
fn the_dpad_moves_th10s_player_in_the_direction_it_points() {
    in_its_own_process(|| {
        let game = playing("th10-dpad-moves", |_| {});
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);
        game.frames(30);
        assert_eq!(
            game.word_the_input_read_answered() & DIRECTIONS,
            0,
            "the player was moved with the pad pushed nowhere",
        );

        game.sim().joystick().pushes_the_hat(HAT_LEFT);
        the_read_answers(&game, "the d-pad moving the player left", LEFT);
        for _ in 0..3 {
            game.frame();
            assert_eq!(
                game.word_the_input_read_answered(),
                LEFT,
                "the player was not moved left on every frame the d-pad was held",
            );
        }
        game.sim().joystick().pushes_the_hat(HAT_RIGHT);
        the_read_answers(&game, "the d-pad moving the player right", RIGHT);
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);
        the_read_answers(&game, "the player standing still", 0);
    });
}

/// `dpad_moves: false` hands the game the word its own read produced, which is a d-pad that does nothing — the
/// game as it is without orb.
#[test]
fn the_setting_off_leaves_th10s_pad_as_the_game_has_it() {
    in_its_own_process(|| {
        let game = playing("th10-dpad-off", |config| config.dpad_moves = false);
        game.sim().joystick().pushes(1 << SHOT_BUTTON, centred());
        game.sim().joystick().pushes_the_hat(HAT_LEFT);
        game.frames(1);
        for _ in 0..30 {
            game.frame();
            assert_eq!(
                game.word_the_input_read_answered(),
                SHOT,
                "the d-pad moved the player in a launch that turned it off",
            );
        }
    });
}
