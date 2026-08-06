//! What orb's own frame loop asks of the game, in what order, and the ways it declines to run one.
//!
//! The rate is `pacing_*.rs`'s subject. This is the loop's shape: the update before the draw, the sounds
//! between them, the present at the end, and the frame handed back to the game's own loop where there is
//! nothing to pace or draw with. None of it could be driven before the two calls into the game became
//! addresses the game hands over — see `docs/adr/0002`.

mod fake;
mod pacing;

use fake::{
    CHAIN_FAILED, CHAIN_LEFT, DRAW, FRAME_FAILED, FRAME_KEPT_RUNNING, FRAME_LEFT, Fake, PRESENT,
    SOUND, UPDATE, WINDOW, in_its_own_process,
};
use orb_api::Hwnd;
use orb_core::game::RunStart;

/// The run a launch is started for. None of it is played: what these are about is the frame, and the
/// game sits on its title menu throughout.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// A launch, run far enough in that the frames below are ordinary ones — the overlay built, the title
/// menu up — and then asked to forget what it was asked for while getting there.
fn settled(name: &str, settings: impl FnOnce(&mut orb_config::Config)) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), settings);
    game.at_the_title_menu();
    game.forget_asked();
    game
}

/// The same, with the frame loop's own log on so that what it says about the pacing can be read back.
fn settled_with_the_pacing_log(
    name: &str,
    settings: impl FnOnce(&mut orb_config::Config),
) -> Box<Fake> {
    settled(name, |config| {
        config.pacing_log = true;
        settings(config);
    })
}

#[test]
fn the_frame_loop_asks_for_the_update_first_and_the_draw_after_the_sounds() {
    in_its_own_process(|| {
        let game = settled("frame-loop-order", |_| {});
        game.frame();
        // The whole of what a frame asks of the game, in orb's order. The update before the draw is the
        // frame of input lag removed; the sounds go where the game's own loop put them, after the update.
        assert_eq!(game.asked(), [UPDATE, SOUND, DRAW, PRESENT]);
    });
}

/// And a launch that turned it off gets 紅魔郷's own order back, which is what `--no-frame-loop` is for.
#[test]
fn the_games_own_loop_asks_for_the_draw_first() {
    in_its_own_process(|| {
        let game = settled("frame-loop-off", |config| config.own_frame_loop = false);
        game.frame();
        assert_eq!(game.asked(), [DRAW, UPDATE, SOUND, PRESENT]);
    });
}

/// The chain's two exits are the frame's two, and a frame that is leaving is not drawn or handed over.
///
/// Which is what the game's own loop above `Render` reads to stop: a walk that answered nothing has asked
/// the game to stop, and one that answered `-1` has failed. Nothing of orb's decides that — it is passed
/// on, and the frame stops where it was told to.
#[test]
fn the_chains_two_exits_become_the_frames_two() {
    in_its_own_process(|| {
        let game = settled("frame-loop-exits", |_| {});
        for (walked, answered) in [(CHAIN_LEFT, FRAME_LEFT), (CHAIN_FAILED, FRAME_FAILED)] {
            game.chain_answers(walked);
            game.forget_asked();
            assert_eq!(game.frame(), answered, "the chain answered {walked}");
            assert_eq!(
                game.asked(),
                [UPDATE, SOUND],
                "the chain answered {walked} and the frame carried on",
            );
        }
    });
}

/// With no runtime the frame is the game's own again, which is the state a process is in before
/// `DllMain` has left one behind and the state a closed game is left in.
#[test]
fn a_frame_with_no_runtime_is_handed_back_to_the_game() {
    in_its_own_process(|| {
        let game = settled("frame-loop-no-runtime", |_| {});
        unsafe { orb::detached() };
        assert_eq!(game.frame(), FRAME_KEPT_RUNNING);
        assert_eq!(game.asked(), [DRAW, UPDATE, SOUND, PRESENT]);
    });
}

/// And with no device, which is every frame before the game has finished setting Direct3D up: there is
/// nothing to draw the overlay through and nothing to pace against.
#[test]
fn a_frame_with_no_device_is_handed_back_to_the_game() {
    in_its_own_process(|| {
        let game = settled("frame-loop-no-device", |_| {});
        game.image().shows_through(std::ptr::null_mut(), WINDOW);
        assert_eq!(game.frame(), FRAME_KEPT_RUNNING);
        assert_eq!(game.asked(), [DRAW, UPDATE, SOUND, PRESENT]);
    });
}

/// The window behind is the one way out that paces instead of returning.
///
/// The game's own loop calls straight back, so a frame that returned without waiting would spin a core
/// for as long as the window stayed behind. Nothing is asked of the game — that is what `always_draw` off
/// means — and the turn is still taken.
///
/// **On the blanks, and not by the clock.** That is measured rather than assumed: a window behind, one
/// covered by a full-screen window and one minimised all flush at the compositor's own rate with every
/// gap one refresh, and the lead a frame needs to make its blank is the same whether anybody can see it —
/// `scripts/background-flush-probe.c`. So a background frame is paced like any other, which is what the
/// run below reads back out of the log.
#[test]
fn a_frame_with_the_window_behind_takes_its_turn_on_the_blanks() {
    in_its_own_process(|| {
        let game =
            settled_with_the_pacing_log("frame-loop-behind", |config| config.always_draw = false);
        let before = game.sim().clock().peek();
        game.sim().display().set_foreground(Hwnd(WINDOW.0 + 1));

        assert_eq!(game.frame(), FRAME_KEPT_RUNNING);
        assert_eq!(game.asked(), Vec::<&str>::new());

        // A whole turn of the clock went by all the same, which is the difference between pacing and
        // returning: a sixtieth of a second is 16666µs, and the wait is the only thing here that takes
        // any.
        let took = orb_sim::Clock::micros_for_ticks(game.sim().clock().peek() - before);
        assert!(
            took > 10_000,
            "the frame took {took}us, which is no turn at all"
        );

        // And it was the compositor's blank it waited for, which orb's own count of the other kind is
        // what says: a frame paced by the clock is counted there and never taken back.
        //
        // Read after the window comes forward again, because that count is written from inside the
        // update — and a frame behind asks the game for nothing, so a run that stayed behind writes
        // nothing at all to read.
        const BEHIND: u32 = 300;
        game.frames(BEHIND);
        game.sim().display().set_foreground(WINDOW);
        pacing::until_reported(&game);

        assert!(
            game.log().said("0 frame(s) paced by the clock"),
            "the {} frame(s) behind were paced by the clock — {}",
            BEHIND + 1,
            pacing::last_said(&game)
        );
    });
}
