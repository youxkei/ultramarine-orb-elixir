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
        assert_eq!(game.sim().tones(), 0, "a tone before any judgement");
        // And power collected after the chapter began, which a retry gives back to the chapter: the game
        // drops 16 on the frame the death is made final, and the restore puts the chapter's own back.
        game.image().set_power(128);

        // ── 3. A hit with the button pressed just before it: in time, nothing owed — and the one press set no
        // bomb off, the game never having been handed it.
        let bombs = game.state().bombs;
        game.press(keys::X);
        game.hit();
        game.frames_until("the death settled", A_BOMB_IS_TAKEN_FOR as u32 + 2, || {
            log.said("died in chapter 1")
        });
        assert!(
            log.said(": pressed in time; 0/12 owed, lives 2"),
            "the press before the hit was not taken as in time: {:?}",
            log.lines(),
        );
        // A tone for it, which is what somebody at the keyboard has to notice it by: no star is shaved
        // either way, and the panel is read after a retry at the earliest.
        assert_eq!(
            game.sim().tones(),
            1,
            "a press in time made no sound: {:?}",
            log.lines(),
        );
        assert_eq!(game.state().bombs, bombs, "one press set a bomb off");
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(game.state().lives, 2);
        // **The chapter's own power**, which is what a rewind gives back: a chapter begun at nothing and
        // brought to 128 by its own items comes back to nothing whichever way the hit was judged.
        assert_eq!(
            game.state().power,
            0,
            "the retry kept power the chapter did not begin with",
        );
        // And the panel asked to paint itself again, every row of it — the picture on the screen being
        // the frame before the restore rather than the restored one.
        let flags = game.image().gui_flags();
        let field = |at: u32| (flags >> (at * 2)) & 0b11;
        assert!(
            (0..5).all(|at| field(at) != 0),
            "the panel was not asked to paint itself after the restore: {flags:#07b}",
        );
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
            log.said_since(mark, ": not pressed in time; 6/12 owed, lives 2"),
            "a hit without the press was not judged so: {:?}",
            log.lines(),
        );
        assert_eq!(
            game.sim().tones(),
            1,
            "a hit the press did not save made a sound",
        );
        game.press_until(keys::Z, "the retry menu answered again", || {
            log.said_since(mark, "retry: the chapter again on the keyboard")
        });
        assert_eq!(game.state().lives, 2, "half a life is not a life");
        // And the chapter's power again, as on any retry.
        assert_eq!(
            game.state().power,
            0,
            "the retry kept power the chapter did not begin with",
        );
        game.frames(AFTER_A_RETRY);
        // And what is owed kept beside the chapter written down after the retry, for a launch that picks the
        // run up to owe it too.
        let owed_kept = game
            .dir()
            .join("pointdevice_bomb_resume")
            .join("normal-reimu-a.owed");
        assert_eq!(
            game.sim().files().text(&owed_kept).as_deref(),
            Some("6"),
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

        // ── And an extend while the part is owed: **the shaved star moves to the one gained**, which is the
        // last of the row — the run's count follows the game's up, and the part it owes is unchanged. The life
        // is left standing rather than put back, the steps after this one being read against it.
        game.image().set_lives(game.state().lives + 1);
        game.frames(2);
        game.forget();
        game.frame();
        let lives_now = u8::try_from(game.state().lives).expect("a run's lives");
        let last = Quad {
            x: row.x + 16.0 * f32::from(lives_now - 1),
            y: row.y,
            width: 16.0,
            height: 16.0,
            ..row
        };
        assert!(
            game.drawn()
                .quads
                .iter()
                .any(|quad| last.overlaps(quad) && quad.x >= last.x + 4.0),
            "the star the part is gone from is not the last of the row after an extend",
        );
        let before = Quad {
            x: last.x - 16.0,
            ..last
        };
        assert!(
            !game.drawn().quads.iter().any(|quad| before.overlaps(quad)),
            "a star before the last is still shaved after an extend",
        );

        // ── 5. And another: the halves come to a life, which the run loses, and which the chapter put back
        // does not give back.
        let mark = log.written();
        game.hit();
        game.frames_until(
            "the life-taker settled",
            A_BOMB_IS_TAKEN_FOR as u32 + 2,
            || log.said_since(mark, "a life is taken"),
        );
        assert!(
            log.said_since(mark, "died in chapter 1"),
            "the third death was not settled: {:?}",
            &log.lines()[mark..],
        );
        assert!(
            log.said_since(
                mark,
                ": not pressed in time, and a life is taken; 0/12 owed, lives 2"
            ),
            "the parts coming to a life did not take one: {:?}",
            log.lines(),
        );
        // **And the row asked for again on the frame the shave stopped**, which nothing else does: the
        // part of a star cut out last frame is in the picture on the screen, and a row the game is not
        // asked for is a row it does not paint. Which was a star with a wedge gone out of it and nothing
        // owed, and the count the attempt before had, for as long as nothing else changed it.
        let flags = game.image().gui_flags();
        assert_ne!(
            flags & 0b11,
            0,
            "the lives' row was not asked for after the shave stopped: {flags:#07b}",
        );
        game.press_until(keys::Z, "the retry menu answered a third time", || {
            log.said_since(mark, "retry: the chapter again on the keyboard")
        });
        assert!(log.said_since(mark, "bomb: the lives put back to 2 over the chapter's own"));
        assert_eq!(game.state().lives, 2, "the restore gave the life back");
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
            bombs,
            "the game's own bomb was taken for a bomb that answered the hit",
        );
        assert!(
            !log.said_since(mark, "died in chapter"),
            "the deathbomb did not take the hit back: {:?}",
            log.lines(),
        );
        assert!(!log.said_since(mark, "bomb: a hit settled"));
        // **And nothing owed for it**: the bomb answered the hit, which is the press working and not a bomb
        // with nothing to answer.
        game.frames_until("the bomb's window over", 600, || {
            log.said_since(mark, "bomb: the bomb answered a hit; it costs nothing")
        });
        assert!(
            !log.said_since(mark, "bomb: a bomb answered nothing"),
            "the deathbomb was counted as a bomb with nothing to answer: {:?}",
            &log.lines()[mark..],
        );
        assert_eq!(game.state().lives, 2);
        // And no tone: the game's own deathbomb is one the game took, and what orb has to say with a tone
        // is its own judgement of a press.
        assert_eq!(
            game.sim().tones(),
            1,
            "the game's own deathbomb made a tone of orb's",
        );

        // ── 7. And a bomb with nothing to answer: a quarter of a bomb, and a tone for it — a bomb that costs
        // nothing is one to bash, and the judgement an errand.
        let mark_first = log.written();
        let bombs_before = game.state().bombs;
        game.press(keys::X);
        game.press(keys::X);
        game.frames_until("the bomb's window over", 600, || {
            log.said_since(mark_first, "bomb: a bomb answered nothing")
        });
        assert!(
            log.said_since(mark_first, "1/4 of a bomb owed"),
            "the bomb did not cost a quarter: {:?}",
            &log.lines()[mark_first..],
        );
        // And the game's own bomb put back: the mode charges the quarter and not a whole bomb for one.
        assert_eq!(
            game.state().bombs,
            bombs_before,
            "the game's own bomb was not put back",
        );
        // ── 8. **Three more quarters come to a bomb**, which the run gives: the panel shows one fewer and
        // the game's own count is what is written.
        for quarter in 2..=4 {
            let mark = log.written();
            game.press(keys::X);
            game.press(keys::X);
            let what = format!("quarter {quarter} of a bomb");
            game.frames_until(&what, 600, || {
                log.said_since(mark, "bomb: a bomb answered nothing")
            });
        }
        assert!(
            log.said_since(mark_first, "bomb: the quarters came to a bomb"),
            "four quarters did not come to a bomb: {:?}",
            &log.lines()[mark_first..],
        );
        assert_eq!(
            game.state().bombs,
            bombs_before - 1,
            "the bomb the quarters came to was not taken",
        );
    });
}
