//! Pointdevice mode over a laid-out th11: the question that starts it, a chapter, a death and a retry.
//!
//! **This is what `Th11::rewinds` rests on**, the way `a_pointdevice_run_of_th10` is what that game's rests on:
//! the chapter a snapshot is taken as covers this game's `.data` and the heap blocks beside it, the ranges a
//! restore must leave alone are its sound, its supervisor's locks and surfaces and its `.anm` files, the retry menu
//! is drawn through its Direct3D 9 device in a face the machine lent, and the numbers come back through its own
//! replay record.
//!
//! **The game plays the game's part and this presses keys**: what moves the run along is the laid-out game, and
//! what this file does is press, run frames, and read back the game's own memory and what orb put in the log.

use std::path::PathBuf;

use crate::fake::th11::{DIFFICULTY_NORMAL, EXE, FIRST_STAGE, Fake, TO_A_RUN, TO_THE_SHOT};
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th11::image::{STEP_READING, Screen};
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a screen or a line of the log is given to arrive.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// Two of the retry menu's own ways on, in the language a launch on this machine comes out in.
const AGAIN: &str = "チャプターをやり直す";
const GIVEN_UP: &str = "タイトルに戻る";

/// The name orb gives the run this plays, which names the file it is left in: the difficulty, the character and
/// the partner — see `Th11::run_slot`. **Written out rather than asked of `run_slot`**, which would be this
/// agreeing with the code it is about.
const SLOT: &str = "normal-reimua";

/// How long the laid-out stage lasts here, which is longer than this test plays.
const A_STAGE: u32 = 100_000;

/// Whether the title is on `screen` and reading a key.
fn reading(game: &Fake, screen: Screen) -> bool {
    let (on, step, _, _) = game.image().where_the_title_is();
    game.image().the_title_is_up() && on == screen as i32 && step == STEP_READING
}

#[test]
fn a_pointdevice_run_of_th11_is_chosen_chaptered_lost_and_retried() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-pointdevice",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();

        // ── 1. The mode. A game orb can rewind starts in Pointdevice mode and is asked which at its own main menu,
        // which is the first screen of its title.
        assert!(
            log.said(&format!(
                "mode: {} to start with; the menu asks which",
                Mode::Pointdevice
            )),
            "the launch did not start in the mode orb is for: {:?}",
            log.lines(),
        );
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.frames_until("the main menu", FRAMES_FOR_A_SCREEN, || {
            reading(&game, Screen::MainMenu)
        });
        game.press_until(keys::Z, "the question", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        game.one_frame();
        assert_eq!(
            game.says(title(Menu::Run, LANGUAGE)).len(),
            1,
            "the mode question is not on the screen",
        );
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });

        // ── 2. The run, walked into: the decide orb held back handed to the main menu, and the three selects.
        for screen in [Screen::Difficulty, Screen::Character, Screen::Partner] {
            game.frames_until(&format!("{screen:?}"), FRAMES_FOR_A_SCREEN, || {
                reading(&game, screen)
            });
            game.press(keys::Z);
        }
        game.frames_until("the run", FRAMES_FOR_A_SCREEN, || game.state().in_game);
        game.frames_until("the stage built", FRAMES_FOR_A_SCREEN, || {
            !game.image().the_stage_is_being_built()
        });
        let run = game.state();
        assert_eq!(
            (run.stage, run.difficulty, run.lives),
            (FIRST_STAGE, DIFFICULTY_NORMAL, 2),
            "the run the walk started is not the one this game's own partner select starts",
        );

        // ── 3. The stage's first chapter, which is its own start, and the chapter written down.
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        game.frames_until_a_thread("the chapter written down", || {
            log.said("chapter 1 (") && log.said("resume: stage 1 chapter 1 (")
        });
        assert_eq!(
            orb_core::resume::left(game.dir()),
            vec![SLOT.to_owned()],
            "the run was not left behind under the name of its own difficulty and partner",
        );

        // ── 4. The death: the hit, the frames a bomb could take it back in, and the life gone.
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 1"));
        game.one_frame();
        assert_eq!(
            (game.says(AGAIN).len(), game.says(GIVEN_UP).len()),
            (1, 1),
            "the menu over the frozen game is not the one the mode puts there: {:?}",
            log.lines(),
        );

        // ── 5. The retry, answered on the keyboard: the chapter's own numbers back, through this game's own
        // replay record.
        game.press_until(keys::Z, "the chapter put back", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let state = game.state();
        assert_eq!(
            state.lives, 2,
            "the chapter did not put the run's life back"
        );
        assert!(state.playing, "the run was not left playing");

        // ── 6. The next stage, built after the one won and faded in from it, with a chapter of its own at its start.
        game.moves_on_to_the_next_stage();
        game.frames_until("the next stage's own first chapter", 600, || {
            log.said("stage 2 chapter 1 (stage start)")
        });
        let written = || {
            log.lines()
                .into_iter()
                .find(|line| line.contains("resume: stage 2 chapter 1 ("))
        };
        game.frames_until_a_thread("the next stage's first chapter written down", || {
            written().is_some()
        });
        let next = game.state();
        assert_eq!(
            (next.stage, next.lives),
            (FIRST_STAGE + 1, 2),
            "the stage after the one won did not carry the run on",
        );
    });
}

/// **A run placed in the laid-out stage of `lasts` frames, firing nothing**, played up to its chapter `chapter`.
fn in_the_stage_up_to(name: &str, lasts: u32, chapter: u32) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.stages_last(lasts);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    let log = game.log();
    game.frames_until(&format!("chapter {chapter}"), lasts, || {
        log.said(&format!("stage 1 chapter {chapter} "))
    });
    game
}

/// A stage with room for the midboss before the boss: stage 1's row at 1005 its second chapter, her two attacks its
/// third and fourth, the waves after her its fifth, and the boss's nonspell, card and nonspell its sixth to eighth.
const A_STAGE_WITH_BOTH_FIGHTS: u32 = 3_600;

/// **A death in the nonspell after a card goes back to that nonspell and not to the card**: the retry puts the
/// run back on the frame the nonspell's chapter began on, with no card up.
#[test]
fn a_death_in_the_nonspell_after_a_card_of_th11_goes_back_to_that_nonspell() {
    in_its_own_process(|| {
        let game = in_the_stage_up_to("th11-a-death-after-a-card", A_STAGE_WITH_BOTH_FIGHTS, 8);
        let log = game.log();
        assert!(
            log.said("stage 1 chapter 7 (BOSS SPELL 1)")
                && log.said("stage 1 chapter 8 (BOSS NONSPELL 2)"),
            "the boss's card and the nonspell after it took no chapters of their own:\n  {}",
            log.lines().join("\n  "),
        );
        let at = crate::fake::the_frame_chapter_began_on(&log.lines(), 1, 8);
        game.frames(30);
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 8"));
        game.press_until(keys::Z, "the retry answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let back = game.state();
        assert!(
            (at..=at + 1).contains(&back.stage_frames),
            "the retry put the run on frame {} of the stage, where the nonspell's chapter began on frame {at}",
            back.stage_frames,
        );
        assert_eq!(
            back.spellcard, None,
            "the retry put the run back with a card up, which is the card's chapter and not the nonspell's",
        );
    });
}

/// **A chapter taken again puts back the point items' value and the number at `0x4a5744`** as the chapter began
/// with them — two numbers this game keeps that a stage is played for, and words of `.data`, which the snapshot is
/// a copy of.
#[test]
fn a_chapter_of_th11_taken_again_puts_back_the_point_value_and_the_number_at_4a5744() {
    in_its_own_process(|| {
        let game = in_the_stage_up_to("th11-the-point-value-put-back", A_STAGE_WITH_BOTH_FIGHTS, 1);
        let log = game.log();
        game.image()
            .the_run_also_has(A_POINT_VALUE, 0, A_NUMBER, 0, 0, 0);
        game.frames_until("the midboss's chapter", A_STAGE_WITH_BOTH_FIGHTS, || {
            log.said("stage 1 chapter 2 ")
        });
        game.image()
            .the_run_also_has(A_POINT_VALUE + 1_000, 0, A_NUMBER + 64, 0, 0, 0);
        game.frames(30);
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 2"));
        game.press_until(keys::Z, "the retry answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.image().the_point_value_and_the_number_at_4a5744(),
            (A_POINT_VALUE, A_NUMBER),
            "the chapter came back with the numbers the run had reached since it began",
        );
    });
}

/// The point items' value and the number the chapter begins with — any numbers, what the test asks of them being
/// that they differ from what the run has when it dies.
const A_POINT_VALUE: i32 = 17_000;
const A_NUMBER: i32 = 128;

/// The frame of a file's buttons they are taken out from: past the stage's opening and before the laid-out
/// stage's first bullet at 480.
const STANDING_FROM: usize = 400;

/// How long a stage of the laid-out game lasts for the launches below, which is what the chains of the cases give
/// one: its midboss's two attacks are its second and third chapters.
const A_SHORT_STAGE: u32 = 4_500;

/// A launch of this game played by a recipe, over a stage of its own.
fn launched(name: &str, recipe: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(A_SHORT_STAGE);
    game
}

/// **Played until its recipe quits it**, and failed where it never does.
fn runs_out(game: &Fake) {
    const FRAMES: u32 = 40_000;
    for _ in 0..FRAMES {
        if game.sim().exited().is_some() {
            return;
        }
        game.frame();
    }
    let lines = game.log().lines();
    panic!(
        "the launch never quit in {FRAMES} frames; the last of what it said:\n  {}",
        lines[lines.len().saturating_sub(80)..].join("\n  "),
    );
}

/// What a launch left under its directory, re-rooted for the next.
fn left_by(game: &Fake) -> Vec<(PathBuf, Vec<u8>)> {
    let dir = game.dir().to_path_buf();
    let files = game.sim().files();
    files
        .paths()
        .into_iter()
        .filter_map(|path| {
            Some((
                path.strip_prefix(&dir).ok()?.to_path_buf(),
                files.get(&path)?,
            ))
        })
        .collect()
}

/// **The walk into a run and the run picked up**: the partner select's decide asking the resume question, which is
/// the first frame orb knows which run it is, and its first item taken.
fn picked_up_then(rest: &str) -> String {
    format!(
        "{TO_THE_SHOT}\
- press: [decide]
  frames: 10
  until: {{ question: resume }}
  give_up_after: 900
- answer: {{ question: resume, item: 0 }}
  give_up_after: 1800
{rest}"
    )
}

/// **And the run left for the title**, which `quit` cannot be: closing the process leaves a run nobody left, where
/// this is the interruption a resume is for.
const LEFT: &str = "\
- leave: true
- wait_until: { frames: 300 }
- quit: true
";

/// **A run picked up in a stage it entered out of the last one's fade lands with its shot where the run's was**:
/// the stage the run came into from the one before fired its first shots on the update it began on, where the
/// stage a pick-up builds from the menu fires them an update later — so the landing has the player's shot timer
/// at the count the run's had on the same frame, and not one short of it.
#[test]
fn a_run_of_th11_picked_up_in_a_stage_entered_out_of_a_fade_lands_with_the_runs_shot() {
    in_its_own_process(|| {
        let first = launched(
            "th11-a-run-left-in-a-stage-entered-out-of-a-fade",
            &format!(
                "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 18000
- play_until: {{ frames: 120 }}
{LEFT}"
            ),
        );
        first.fires_at_the_player(false);
        let mut the_runs = std::collections::BTreeMap::new();
        for _ in 0..40_000 {
            if first.sim().exited().is_some() {
                break;
            }
            first.frame();
            let state = first.state();
            // The last read of each frame, the fade before the stage counting the same frames first.
            if state.in_run && state.stage == 1 {
                the_runs.insert(state.stage_frames, first.the_shot_timer());
            }
        }
        assert!(
            first.sim().exited().is_some(),
            "the first launch never quit"
        );
        let left = left_by(&first);

        let second = launched(
            "th11-a-run-picked-up-in-a-stage-entered-out-of-a-fade",
            &picked_up_then(&format!("- play_until: {{ frames: 120 }}\n{LEFT}")),
        );
        second.fires_at_the_player(false);
        for (inside, bytes) in &left {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        let log = second.log();
        second.frames_until("the run landed", 40_000, || {
            log.said("resume: the landing is")
        });
        let landed_at = second.state().stage_frames;
        assert_eq!(
            second.times_orb_stepped_the_shot(),
            1,
            "the landing did not step the shot through the game's own call, once",
        );
        assert_eq!(
            second.the_shot_timer(),
            the_runs[&landed_at],
            "the run picked up landed at frame {landed_at} with its shot elsewhere than the run's on that frame",
        );
    });
}

/// **A run picked up whose playback is hit short of its chapter is the rules' to play after the retry**, and not
/// the record's: the file is the one a run left in its second chapter with its buttons taken out from a frame on,
/// a record the laid-out stage's bullets, aimed at where the player stands, find standing still.
#[test]
fn a_run_of_th11_whose_playback_is_hit_short_of_its_chapter_is_the_rules_to_play_after_the_retry() {
    in_its_own_process(|| {
        let first = launched(
            "th11-a-run-left-for-its-playback-to-be-hit",
            &format!(
                "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 1, chapter: 2 }}
  give_up_after: 18000
{LEFT}"
            ),
        );
        runs_out(&first);
        let left = left_by(&first);
        let kept = PathBuf::from("pointdevice_resume").join(format!("{SLOT}.msgpack"));
        let game = orb_core::game::found(EXE)
            .expect("th11 is a game orb knows")
            .game;
        let bytes = left
            .iter()
            .find(|(inside, _)| *inside == kept)
            .map(|(_, bytes)| bytes)
            .expect("the run was left behind");
        let mut saved = game
            .decode_resume(bytes)
            .expect("the run left behind reads back");
        assert!(
            saved.buttons.len() > STANDING_FROM,
            "the run was left before the frame its buttons are taken out from: {} frame(s)",
            saved.buttons.len(),
        );
        for word in saved.buttons.iter_mut().skip(STANDING_FROM) {
            *word = 0;
        }
        let standing = game.encode_resume(&saved);

        let second = launched(
            "th11-a-run-picked-up-to-be-hit-in-its-playback",
            &picked_up_then(&format!(
                "- play_until: {{ stage: 1, chapter: 3 }}\n  give_up_after: 18000\n{LEFT}"
            )),
        );
        for (inside, bytes) in &left {
            let bytes = if *inside == kept { &standing } else { bytes };
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let log = second.log();
        assert!(
            log.said("resume: the playback stopped because the player was hit at frame"),
            "the playback was not hit short of its chapter:\n  {}",
            log.lines().join("\n  "),
        );
        let deaths = log
            .lines()
            .iter()
            .filter(|line| line.contains("died in chapter"))
            .count();
        assert_eq!(
            deaths,
            1,
            "the chapter taken again was played from the record and not by the rules:\n  {}",
            log.lines().join("\n  "),
        );
        assert!(
            log.said("stage 1 chapter 3"),
            "the rules did not play on past the chapter the run was picked up at:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **A run picked up in a stage after its first plays the run it was written from back into place**, where its
/// numbers are not the ones a build starts a run with — the stage after one won being built with the numbers
/// carried (`+0x564` at nought, 0x42b279, the build's 0x41f932) and through 0x436f30 rather than the seam a fresh
/// build goes through (0x41fc98, 0x41fc2b).
#[test]
fn a_run_of_th11_picked_up_after_its_first_stage_lands_where_it_was_written() {
    in_its_own_process(|| {
        let first = launched(
            "th11-a-run-left-after-its-first-stage",
            &format!(
                "{TO_THE_SHOT}{TO_A_RUN}\
- play_until: {{ stage: 2, chapter: 1 }}
  give_up_after: 18000
- play_until: {{ frames: 120 }}
{LEFT}"
            ),
        );
        first.fires_at_the_player(false);
        runs_out(&first);
        assert!(
            first.log().said("resume: stage 2 chapter 1 ("),
            "the run was not left in its second stage:\n  {}",
            first.log().lines().join("\n  "),
        );
        let left = left_by(&first);

        let second = launched(
            "th11-a-run-picked-up-after-its-first-stage",
            &picked_up_then(&format!("- play_until: {{ frames: 120 }}\n{LEFT}")),
        );
        second.fires_at_the_player(false);
        for (inside, bytes) in &left {
            second.sim().files().put(second.dir().join(inside), bytes);
        }
        runs_out(&second);
        let log = second.log();
        let said: Vec<String> = log
            .lines()
            .into_iter()
            .filter(|line| line.contains("resume: ") && !line.contains("resume: frame "))
            .collect();
        assert!(
            said.iter()
                .any(|line| line.contains("the run is being built at that stage")),
            "the run was not picked up in a stage after its first:\n  {}",
            said.join("\n  "),
        );
        assert!(
            log.said("resume: the landing is the frame that was written down, field for field"),
            "the run picked up after its first stage did not land where it was written:\n  {}",
            said.join("\n  "),
        );
    });
}
