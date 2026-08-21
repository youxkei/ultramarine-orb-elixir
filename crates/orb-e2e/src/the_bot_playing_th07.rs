//! **orb playing 妖々夢 itself, so that a run to be picked up out of a file costs nobody an afternoon.**
//!
//! Every measurement of a resume so far has cost somebody a run of stage 1 and a stretch of stage 2, and then
//! another one every time a field was added to what the file carries. A run the machine plays is a run that can
//! be had again on demand — and the same buttons every launch, which is what makes two launches comparable at
//! all.
//!
//! **It plays the way somebody does and reads what they read.** No invulnerability and no keys sent from
//! outside: the word goes in where a playback's own words go in — see `resume::fed` and the read hook in
//! `runtime` — and what it is made of is the game's own numbers, read where the game keeps them.
//!
//! The rules are 妖々夢 on Normal, where the bullets are thin enough that a run does not need to dodge:
//!
//! - the shot held, always
//! - the bomb the frame the player is hit, which is 妖々夢's own 決死結界 — [`PLAYER_STATE`] is 2 for the
//!   updates between the hit test finding a bullet and the death handler running, so a bomb pressed there is a
//!   death taken back
//! - a power item gone to while a stage is running, and the boss's own x held under a boss with the player low
//!
//! What no laid-out game can say is whether the rules clear the game. That is what a launch says, and the
//! reason this file holds the rules one at a time instead: a run that dies in stage 3 is a rule that needs
//! another look, and which rule it was is what these say.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// 妖々夢's own first two buttons, as `Th07` has them: shoot and bomb.
const SHOT: u16 = 0x001;
const BOMB: u16 = 0x002;

fn playing(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game
}

/// The shot is held on every frame of a stage, which is the whole of what a run of 妖々夢 does with it.
#[test]
fn the_bot_holds_the_shot_down() {
    in_its_own_process(|| {
        let game = playing("th07-the-bot-shoots");
        game.image().the_player_is_playing();

        game.frames(30);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & SHOT,
            SHOT,
            "the bot played a frame of a stage without the shot held:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And the bomb on the frame the player is hit, which is what takes the death back.
///
/// **The state and not the deaths**, because a death counted is a death already taken: [`PLAYER_STATE`] is 2
/// from the update the hit test finds a bullet on until the one the death handler runs on, and those are the
/// updates a bomb is still a 決死結界.
#[test]
fn the_bot_bombs_the_frame_the_player_is_hit() {
    in_its_own_process(|| {
        let game = playing("th07-the-bot-death-bombs");
        game.image().the_player_is_playing();
        game.frames(30);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            0,
            "the bot bombed a frame nothing had hit the player on",
        );

        game.image().the_player_has_been_hit();
        game.frames(1);
        assert_eq!(
            game.image().the_word_the_run_acts_on() & BOMB,
            BOMB,
            "the bot did not bomb the frame the player was hit:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}
