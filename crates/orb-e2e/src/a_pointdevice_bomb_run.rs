//! One run of 完全無欠ボムモード, from the question that starts it through each way a hit can be judged.
//!
//! What the mode adds to Pointdevice mode is all at the hit — see `orb_core::bomb` — so this takes one of
//! each: a hit with the bomb button pressed just before it, two without, and two presses inside the game's
//! own deathbomb window. Read back the way every e2e test reads: the game's memory, what it drew, and the log.

use crate::fake::th06::{A_BOMB_IS_TAKEN_FOR, Fake, lives_row};
use crate::fake::{LANGUAGE, Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_core::game::RunStart;
use orb_core::menu_ui::SELECTED;
use orb_core::mode::difficulty_title;
use orb_sim::{Quad, keys};

/// Frames played after a retry before the next hit: the frame a chapter is put back on is one orb holds
/// nothing of the frame before against, so a hit on it is a count moving that nothing compares.
const AFTER_A_RETRY: u32 = 4;

/// The run this e2e test plays: Normal, Reimu A, from stage one.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
        mode: 0,
    }
}

#[test]
fn a_bomb_mode_run_judges_each_hit_by_the_press_before_it() {
    in_its_own_process(|| {
        let game = Fake::attach("pointdevice-bomb", the_run(), |_| {});
        let log = game.log();

        // ── 1. 完全無欠ボムモード, the question's third answer, and the difficulty asked on the page after it.
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.at_the_title_menu();
        game.press(keys::Z);
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press(keys::DOWN);
        game.forget();
        game.frame();
        assert_eq!(
            game.says("完全無欠ボムモード")[0].color,
            SELECTED,
            "two presses down is not the bomb mode",
        );
        game.press(keys::Z);
        game.forget();
        game.frame();
        assert_eq!(
            game.says(difficulty_title(LANGUAGE)).len(),
            1,
            "the difficulty was not asked next",
        );
        assert_eq!(
            game.says("Normal")[0].color,
            SELECTED,
            "the difficulty does not come up on Normal",
        );
        assert!(
            !log.said("mode: answered"),
            "the mode was answered before its difficulty"
        );
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the difficulty answered", || {
            log.said("mode: answered on the keyboard")
        });
        assert!(log.said("bomb: the difficulty is hard"));
        assert!(log.said("mode: pointdevice-bomb, was pointdevice"));
        game.started_at_the_games_own_screens();
        game.frames_until("the stage's first chapter", 400, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // ── 2. The lives are the run's, so nothing is painted over them.
        game.forget();
        game.frame();
        assert!(
            game.says("DISABLE").is_empty(),
            "the lives were painted over in a run that can lose them",
        );
        assert_eq!(game.state().lives, 2, "the lives a run starts with");

        // ── 3. A hit with the button pressed just before it: in time, nothing owed — and the one press set no
        // bomb off, the game never having been handed it.
        let bombs = game.state().bombs;
        game.press(keys::X);
        game.hit();
        game.frames_until("the death settled", A_BOMB_IS_TAKEN_FOR as u32 + 2, || {
            log.said("died in chapter 1")
        });
        assert!(
            log.said(": pressed in time; 0/8 owed, lives 2"),
            "the press before the hit was not taken as in time: {:?}",
            log.lines(),
        );
        assert_eq!(game.state().bombs, bombs, "one press set a bomb off");
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(game.state().lives, 2);
        game.frames(AFTER_A_RETRY);

        // ── 4. A hit without it: half a life owed on Hard, the lives as they were, and the half drawn beside
        // the game's own count.
        let mark = log.written();
        game.hit();
        game.frames(A_BOMB_IS_TAKEN_FOR as u32 + 2);
        assert!(
            log.said_since(mark, "died in chapter 1"),
            "the second death was not settled: {:?}",
            &log.lines()[mark..],
        );
        assert!(
            log.said_since(mark, ": not pressed in time; 4/8 owed, lives 2"),
            "a hit without the press was not judged so: {:?}",
            log.lines(),
        );
        game.press_until(keys::Z, "the retry menu answered again", || {
            log.said_since(mark, "retry: the chapter again on the keyboard")
        });
        assert_eq!(game.state().lives, 2, "half a life is not a life");
        game.frames(AFTER_A_RETRY);
        // And what is owed kept beside the chapter written down after the retry, for a launch that picks the
        // run up to owe it too.
        let owed_kept = game
            .dir()
            .join("pointdevice_bomb_resume")
            .join("normal-reimu-a.owed");
        assert_eq!(
            game.sim().files().text(&owed_kept).as_deref(),
            Some("4"),
            "what the run owes was not kept beside its chapter",
        );
        // And the half shown on the game's own stars: the last of the two, the second from the row's left,
        // with its right half painted over — clockwise from twelve o'clock, half a turn — and its left half and
        // the first star left as the game drew them.
        game.forget();
        game.frame();
        let row = lives_row();
        let star = Quad {
            x: row.x + 16.0,
            y: row.y,
            width: 16.0,
            height: 16.0,
            ..row
        };
        let over_the_star: Vec<Quad> = game
            .drawn()
            .quads
            .into_iter()
            .filter(|quad| star.overlaps(quad))
            .collect();
        let painted: f32 = over_the_star
            .iter()
            .map(|quad| quad.width * quad.height)
            .sum();
        assert_eq!(
            painted, 128.0,
            "half of the last star is not what is painted over: {over_the_star:?}"
        );
        assert!(
            over_the_star.iter().all(|quad| quad.x >= star.x + 8.0),
            "the left half of the last star was painted over: {over_the_star:?}",
        );
        assert!(
            !game
                .drawn()
                .quads
                .iter()
                .any(|quad| quad.x < row.x + 16.0 && row.overlaps(quad)),
            "the first star was painted over",
        );

        // ── 5. And another: the halves come to a life, which the run loses, and which the chapter put back
        // does not give back.
        let mark = log.written();
        game.hit();
        game.frames(A_BOMB_IS_TAKEN_FOR as u32 + 2);
        assert!(
            log.said_since(mark, "died in chapter 1"),
            "the third death was not settled: {:?}",
            &log.lines()[mark..],
        );
        assert!(
            log.said_since(
                mark,
                ": not pressed in time, and a life is taken; 0/8 owed, lives 1"
            ),
            "the parts coming to a life did not take one: {:?}",
            log.lines(),
        );
        game.press_until(keys::Z, "the retry menu answered a third time", || {
            log.said_since(mark, "retry: the chapter again on the keyboard")
        });
        assert!(log.said_since(mark, "bomb: the lives put back to 1 over the chapter's own"));
        assert_eq!(game.state().lives, 1, "the restore gave the life back");
        game.frames(AFTER_A_RETRY);

        // ── 6. Two presses inside the deathbomb window: the second is the bomb, the game's own deathbomb takes
        // the hit back, and there is no death to judge.
        let mark = log.written();
        let bombs = game.state().bombs;
        game.hit();
        game.press(keys::X);
        game.press(keys::X);
        game.frames(A_BOMB_IS_TAKEN_FOR as u32 + 2);
        assert_eq!(
            game.state().bombs,
            bombs - 1,
            "the second press did not set the bomb off",
        );
        assert!(
            !log.said_since(mark, "died in chapter"),
            "the deathbomb did not take the hit back: {:?}",
            log.lines(),
        );
        assert!(!log.said_since(mark, "bomb: a hit settled"));
        assert_eq!(game.state().lives, 1);
    });
}
