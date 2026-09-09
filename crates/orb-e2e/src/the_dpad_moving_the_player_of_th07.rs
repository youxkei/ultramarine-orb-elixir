//! **The d-pad moving th07's player**, which the game itself cannot do.
//!
//! th07's own pad read at 0x430f20 reads a pad's two axes and its buttons and nothing else: the field a d-pad
//! reports in is one it has no look at, so a d-pad does nothing at all in the game while driving orb's own
//! menus, which read one. `dpad_moves` is what closes that: orb adds the direction the hat is pushed to the
//! word the game's own read handed back, in the bits that word names the four directions by — 0x10, 0x20,
//! 0x40 and 0x80, read off the same exe.
//!
//! **What a laid-out th07 has for a player is the word its player's job reads.** This game's own jobs read
//! the keys at chain priority 0, move the player at 8 and copy the read's word into the run's at 0x10, so the
//! word at 8 is the whole of what a frame's play is decided by — see `Fake::the_player_read` — and a d-pad
//! that moves the player is a direction in that word and nothing else. Which is what is asserted here: the
//! frame of input lag between the read and the player is this game's, so the word the player reads is the
//! word the read left the frame before.
//!
//! Legacy mode throughout: what a direction does to the player is the same in either mode, and a pointdevice
//! run would put a chapter's frames between the push and the player.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A pad on the host, as an e2e test declares one — the same declaration `th07.rs`'s own pad test makes.
const PAD_BUTTONS: u32 = 16;
const PAD_TRAVEL: (u32, u32) = (0, 65535);

/// Where a hat points, in the hundredths of a degree clockwise from straight up that a pad reports one in.
const HAT_LEFT: u32 = 27000;
const HAT_RIGHT: u32 = 9000;

/// The bits of th07's own input word a direction and the shot button are — see `Th07::pad_word`.
const SHOOT: u16 = 0x1;
const LEFT: u16 = 0x40;
const RIGHT: u16 = 0x80;
const DIRECTIONS: u16 = 0xf0;

/// And which pad button shoot is with nobody's `th07.cfg` in the directory: the first.
const SHOOT_BUTTON: u32 = 0;

fn centred() -> u32 {
    (PAD_TRAVEL.0 + PAD_TRAVEL.1) / 2
}

/// A legacy run with a pad on the host and orb's own thread having read it once.
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
    game.streams_its_song(0);
    game.in_a_legacy_run();
    game
}

/// Frames until the word the player's own job read is exactly `word`, waited for rather than counted: the
/// sample is taken on a thread nothing here drives, so what says a push has arrived is the player reading it.
fn the_player_reads(game: &Fake, what: &str, word: u16) {
    game.frames_until_a_thread(what, || game.the_player_read() == word);
}

/// The d-pad moves the player, in the direction it points and every frame it is held.
#[test]
fn the_dpad_moves_th07s_player_in_the_direction_it_points() {
    in_its_own_process(|| {
        let game = playing("th07-dpad-moves", |_| {});

        // Nothing pushed first, the hat where a device that has one leaves it: a player that drifts with the
        // pad at rest is an added direction nobody asked for, and every assertion below would pass.
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);
        game.frames(30);
        assert_eq!(
            game.the_player_read() & DIRECTIONS,
            0,
            "the player was moved with the pad pushed nowhere",
        );

        game.sim().joystick().pushes_the_hat(HAT_LEFT);
        the_player_reads(&game, "the d-pad moving the player left", LEFT);
        // And held, which is the player moving every frame rather than once on the edge.
        for _ in 0..3 {
            game.frame();
            assert_eq!(
                game.the_player_read(),
                LEFT,
                "the player was not moved left on every frame the d-pad was held",
            );
        }

        // And back, which is the assertion that what reached the game was the direction pushed rather than a
        // direction: the hat is one field, and up-for-left is a mapping that works until it does not.
        game.sim().joystick().pushes_the_hat(HAT_RIGHT);
        the_player_reads(&game, "the d-pad moving the player right", RIGHT);

        // Let go, and the player stands still: what is added is what the hat says now, not what it last said.
        game.sim().joystick().pushes_the_hat(orb_sim::POV_CENTERED);
        the_player_reads(&game, "the player standing still", 0);
    });
}

/// `dpad_moves: false` hands the game the word its own read produced, which is a d-pad that does nothing —
/// the game as it is without orb.
#[test]
fn the_setting_off_leaves_th07s_pad_as_the_game_has_it() {
    in_its_own_process(|| {
        let game = playing("th07-dpad-off", |config| config.dpad_moves = false);

        // The hat pushed with the shot button held: the button reaches the player and the hat does not. The
        // button is what says the sample arrived, a word that is *not* something being nothing to wait for.
        game.sim().joystick().pushes(1 << SHOOT_BUTTON, centred());
        game.sim().joystick().pushes_the_hat(HAT_LEFT);
        the_player_reads(&game, "the shot button reaching the player", SHOOT);
        for _ in 0..30 {
            game.frame();
            assert_eq!(
                game.the_player_read(),
                SHOOT,
                "the d-pad moved the player in a launch that turned it off",
            );
        }
    });
}
