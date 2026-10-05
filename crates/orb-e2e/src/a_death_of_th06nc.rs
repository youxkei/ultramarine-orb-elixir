//! **A death of th06nc as 0x140069f30 plays it**, th06c's death with this build's numbers: the hit leaves the
//! player in state 2 with the count put back to nothing, the eight updates of the window at `+0x773c` take a
//! bomb, and once it is out the state stands until the count reaches thirty (`0x14006a4b7`): then state 1,
//! the player put back at the field's middle and 64 up from its foot, the life taken — none in a Challenge run
//! — and the bombs put to the run's starting count; and thirty more of state 1 (`0x14006a6a9`) before the 240
//! of state 3. Nothing of it holds the rest of the game: the byte that does (`0x14053d409`) has one writer, a
//! script's (0x140034d3d). And the player moves in neither state 2 nor 1.
//!
//! **A bomb goes off on the bomb's edge in any state the window is still open in** (0x14006a0b8 through
//! 0x14006a1a4: none going off, `[0x140abae28]`'s `+0x36b0` below nought, the window not nought, a bomb to spend), and its
//! callback leaves the player untouchable (the state 3 at 0x14000dd26 and its kin, the count at 0x14000da5f).
//!
//! Without chapters throughout, so that nothing of orb's holds the game at the death.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

const PLAYING: u8 = 0;
const COMING_BACK: u8 = 1;
const HIT: u8 = 2;
const UNTOUCHABLE: u8 = 3;
const PUT_BACK_AT: (f32, f32) = (192.0, 384.0);

/// The updates after the hit the state stands for with the life still there, and the ones by which it has
/// gone on to the walk-in.
const STANDS_FOR: u32 = 29;
const GONE_BY: u32 = 33;
/// And the walk-in's thirty.
const THE_WALK_IN_TAKES: u32 = 30;

fn in_a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.chapters = false;
        },
    );
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game.frames(300);
    game
}

#[test]
fn a_death_of_th06nc_takes_its_life_once_its_count_has_reached_thirty_and_holds_nothing() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-a-death");
        let (lives, _) = game.image().lives_and_bombs();
        game.hit();
        game.frame();
        assert_eq!(
            game.image().the_players_state(),
            HIT,
            "the hit did not land"
        );
        for update in 0..STANDS_FOR {
            game.frame();
            assert_eq!(
                (
                    game.image().the_players_state(),
                    game.image().lives_and_bombs().0
                ),
                (HIT, lives),
                "the death went on before its count reached thirty, {update} update(s) after the hit",
            );
            assert!(!game.state().paused, "the death held the game");
        }
        game.frames_until("the walk-in", GONE_BY - STANDS_FOR, || {
            game.image().the_players_state() == COMING_BACK
        });
        assert_eq!(
            game.image().lives_and_bombs().0,
            lives - 1,
            "the life was not taken"
        );
        assert_eq!(
            game.image().where_the_player_is(),
            PUT_BACK_AT,
            "the walk-in did not start the player where the death puts it",
        );
        for update in 1..THE_WALK_IN_TAKES {
            game.frame();
            assert_eq!(
                game.image().the_players_state(),
                COMING_BACK,
                "the walk-in ended {update} update(s) in",
            );
        }
        game.frames_until("the invulnerability", 2, || {
            game.image().the_players_state() == UNTOUCHABLE
        });
    });
}

#[test]
fn the_player_of_th06nc_does_not_move_while_it_dies() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-a-death-stands");
        game.hit();
        game.frame();
        let at = game.image().where_the_player_is();
        game.keyboard().set(keys::LEFT, true);
        game.frames(10);
        game.keyboard().set(keys::LEFT, false);
        assert_eq!(
            game.image().where_the_player_is(),
            at,
            "the player moved while it was dying"
        );
    });
}

#[test]
fn a_bomb_in_th06ncs_window_takes_the_hit_back_into_the_untouchable_state() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-a-deathbomb");
        game.hit();
        game.frame();
        game.keyboard().set(keys::X, true);
        game.frame();
        game.keyboard().set(keys::X, false);
        assert_eq!(
            game.image().the_players_state(),
            UNTOUCHABLE,
            "the bomb did not leave the player in the bomb's own state",
        );
    });
}

/// And a bomb is taken on its edge, so one held down from before goes off once.
#[test]
fn a_bomb_of_th06nc_held_down_goes_off_once() {
    in_its_own_process(|| {
        let game = in_a_run("th06nc-a-bomb-held");
        assert_eq!(
            game.image().the_players_state(),
            PLAYING,
            "the run is not being played"
        );
        let (_, bombs) = game.image().lives_and_bombs();
        game.keyboard().set(keys::X, true);
        game.frames(200);
        game.keyboard().set(keys::X, false);
        assert_eq!(
            game.image().lives_and_bombs().1,
            bombs - 1,
            "a bomb held down went off other than once",
        );
    });
}
