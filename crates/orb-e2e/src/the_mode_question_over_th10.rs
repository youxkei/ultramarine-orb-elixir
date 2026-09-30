//! Pointdevice mode, chosen at the question orb puts over th10's own title menu.
//!
//! What the question *decides* — which of the two a run is — driven the way somebody meets it: the game at its
//! title menu, keys pressed at it, and what came of that read off the screen and out of orb's log. The same
//! claims `the_mode_question_over_th08.rs` makes, held to this game's own read and menu:
//!
//! - **what is on the screen is how the answer is read**: the two modes drawn with the one under the cursor in
//!   `SELECTED`;
//! - **and what says the press was handed over is this game's own read**: orb holds the decide back for the
//!   question and feeds it on the read after an answer, so the word the pad's half of 0x44a190 answers is where
//!   a press that was kept and then given back shows up;
//! - **and a cancel leaves the title where it was** — which on this game is worth saying, the main menu's own
//!   cancel being the one that puts its cursor on the item that ends the game (0x42d5c0 through 0x42d5dc).

use crate::fake::th10::{Fake, THE_ITEM_FOR_A_RUN};
use crate::fake::{Display, LANGUAGE, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Menu;
use orb_core::game::th10::image::Screen;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_core::mode::{Mode, title};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The bit of this game's own input word the shot button is, which is the press orb holds back and hands
/// over: the keyboard's read folds `Z` into it (0x44a63c through 0x44a8b1).
const SHOOT: u16 = 0x1;

/// A launch with the game at its title menu, taking presses, and with an overlay for orb to draw a question with.
fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.frames_until("an overlay", 120, || log.said("overlay: ready"));
    at_the_title_menu(&game);
    game
}

/// The title menu up with the cursor on the item that starts a run, which is the item orb has a question about.
///
/// A frame before it is put up, where a question was answered before this: the press orb handed on for that
/// answer lands on the read after it, and landing on the frame before the title is put back it walks nothing.
/// And a frame of it after, orb deciding whether to hold a press back at the *end* of an update.
fn at_the_title_menu(game: &Fake) {
    game.frame();
    game.its_title_menu_is_up();
    game.one_frame();
    game.one_frame_to_drain_the_log();
}

/// The question put up, on the press that would have chosen the item under the cursor.
///
/// # Panics
/// If it is not on the screen afterwards, which is the only way anything below means anything.
fn ask(game: &Fake) {
    let log = game.log();
    let from = log.written();
    game.press_until(keys::Z, "the question", || {
        log.said_since(from, "menu: Run is under the cursor, asking which mode")
    });
    game.one_frame();
    assert!(
        !game.says(title(Menu::Run, LANGUAGE)).is_empty(),
        "the press did not put the question up:\n  {}",
        log.lines().join("\n  ")
    );
    game.frames(READS_KEYS_AFTER);
}

/// Which mode the question would answer with now — the one drawn in `SELECTED`.
fn under_the_cursor(game: &Fake) -> Mode {
    game.one_frame();
    let mut on = None;
    for mode in orb_core::mode::CHOICES {
        let text = mode.name(LANGUAGE);
        let drawn = game.says(text);
        assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
        if drawn[0].color == SELECTED {
            assert!(on.replace(mode).is_none(), "both modes are lit");
        } else {
            assert_eq!(drawn[0].color, NORMAL, "{text} is neither lit nor not");
        }
    }
    on.expect("neither mode is lit")
}

/// Whether the question is still on the screen, read off a frame that drew.
fn still_asking(game: &Fake) -> bool {
    !game
        .a_frame_that_says(title(Menu::Run, LANGUAGE))
        .is_empty()
}

/// How many times orb has reported an outcome for this question — a mode chosen, or neither.
fn outcomes(game: &Fake) -> usize {
    game.log()
        .lines()
        .iter()
        .filter(|line| {
            line.contains("mode: answered on the") || line.contains("mode: not chosen on the")
        })
        .count()
}

/// Answers it, whichever way `key` answers.
fn answer(game: &Fake, key: u8) {
    let before = outcomes(game);
    game.press_until(key, "the question answered", || outcomes(game) > before);
}

/// The cursor starts on the mode orb is already in, so one press answers — and the press it answers with is the
/// one orb held back, which the game's own read gets on the frame after. Both keys the title decides with.
#[test]
fn one_press_chooses_the_mode_the_cursor_is_on_and_th10s_own_read_gets_it() {
    in_its_own_process(|| {
        let game = game("th10-mode-chosen");
        let log = game.log();
        for key in [keys::Z, keys::RETURN] {
            at_the_title_menu(&game);
            ask(&game);
            assert_eq!(
                under_the_cursor(&game),
                Mode::Pointdevice,
                "the cursor is not on the mode orb is in",
            );
            answer(&game, key);
            assert!(
                log.said("mode: pointdevice, was pointdevice"),
                "the mode is not what was chosen:\n  {}",
                log.lines().join("\n  ")
            );
            game.frames_until("the press handed to the game's own read", 8, || {
                game.word_the_input_read_answered() & SHOOT != 0
            });
        }
    });
}

/// And somebody in legacy mode reaches Pointdevice mode by moving the cursor once, either way — which is also what
/// says the answer is remembered, the question starting a second time on what the first one chose.
#[test]
fn the_cursor_of_th10s_question_moves_either_way_and_the_mode_it_lands_on_is_kept() {
    in_its_own_process(|| {
        let game = game("th10-mode-moved");
        let log = game.log();
        ask(&game);
        game.press(keys::DOWN);
        assert_eq!(under_the_cursor(&game), Mode::Normal);
        answer(&game, keys::Z);
        assert!(
            log.said("mode: normal, was pointdevice"),
            "レガシーモード was not chosen:\n  {}",
            log.lines().join("\n  ")
        );

        at_the_title_menu(&game);
        ask(&game);
        assert_eq!(
            under_the_cursor(&game),
            Mode::Normal,
            "the question does not start on the mode the last one chose",
        );
        game.press(keys::UP);
        assert_eq!(under_the_cursor(&game), Mode::Pointdevice);
        answer(&game, keys::Z);
        assert!(
            log.said("mode: pointdevice, was normal"),
            "完全無欠モード was not chosen the second time:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **Cancelling leaves neither chosen and the title where it was**, on the bomb key and on escape — both of them
/// in this game's own cancel, which at the main menu would have put the cursor on the item that ends the game.
#[test]
fn cancelling_th10s_question_starts_no_run_and_leaves_the_title_where_it_was() {
    in_its_own_process(|| {
        let game = game("th10-mode-cancelled");
        let log = game.log();
        for key in [keys::X, keys::ESCAPE] {
            at_the_title_menu(&game);
            ask(&game);
            answer(&game, key);
            assert!(
                log.said("mode: not chosen on the keyboard"),
                "{key:#04x} did not cancel:\n  {}",
                log.lines().join("\n  ")
            );
            assert!(
                !still_asking(&game),
                "the question is still up after being cancelled:\n  {}",
                log.lines().join("\n  ")
            );
            assert!(
                !log.said("mode: answered on the"),
                "cancelling answered the question:\n  {}",
                log.lines().join("\n  ")
            );
            game.frames(READS_KEYS_AFTER);
            let (screen, _, cursor, _) = game.image().where_the_title_is();
            assert_eq!(
                (screen, cursor),
                (Screen::MainMenu as i32, THE_ITEM_FOR_A_RUN),
                "the key that cancelled reached the title underneath",
            );
            assert!(!game.state().in_run, "a run was started by a cancel");
        }
    });
}

/// **The score object after an answer is the chosen mode's file, but for what the front end offers**: each shot
/// type's record out of the mode's file — the ranking and the cards every screen draws from the object —
/// and where that file has none, the record the game's own code makes empty (0x42acb0); and out of the game's
/// own file, whichever mode, each practice stage's being offered and the clears the Extra is lit from. This game
/// reads `scoreth10.dat` once, at its startup (0x42ae7d), so without this a Pointdevice session's ranking was the
/// game's own, and a fork read in its place came up with the Extra gone from the menu.
#[test]
fn the_answer_at_th10s_question_reads_the_modes_records_and_keeps_the_games_offers() {
    use orb_core::game::th10::score_file::{Records, settings, shot_types_record};
    in_its_own_process(|| {
        let game = game("th10-the-modes-records");
        let log = game.log();
        let mut own = Records::default();
        own.shot_types[0] = Some(shot_types_record(0, THE_GAMES));
        own.shot_types[1] = Some(shot_types_record(1, THE_GAMES));
        own.settings = Some(settings(THE_GAMES));
        let mut modes = Records::default();
        modes.shot_types[0] = Some(shot_types_record(0, THE_MODES));
        modes.settings = Some(settings(0));
        game.its_score_files_hold(&own, &modes);

        at_the_title_menu(&game);
        ask(&game);
        answer(&game, keys::Z);
        assert!(
            log.said("score: 1 of 7 shot types' records read out of pointdevice_scoreth10.dat"),
            "the mode's file was not read on the answer:\n  {}",
            log.lines().join("\n  ")
        );
        let image = game.image();
        let first = image.the_score_objects_record(0);
        assert!(
            outside_the_offers(&first).all(|byte| byte == THE_MODES),
            "the first shot type's record is not the mode's file's",
        );
        for number in [0, 1] {
            assert_eq!(
                offers(&image.the_score_objects_record(number)),
                [THE_GAMES; PRACTICES],
                "shot type {number}'s practices are not offered as the game's own file offers them",
            );
        }
        for number in [1, 2] {
            let record = image.the_score_objects_record(number);
            assert_eq!(
                (
                    u32::from_le_bytes(record[0x10..0x14].try_into().unwrap()),
                    &record[0x16..0x1e],
                    u32::from_le_bytes(record[A_CARD + 0x88..A_CARD + 0x8c].try_into().unwrap()),
                ),
                (1_000_000, &b"--------"[..], 5),
                "shot type {number}'s record, held by no file of the mode's, is not one the game made empty",
            );
        }
        assert_eq!(
            offers(&image.the_score_objects_record(2)),
            [0; PRACTICES],
            "a shot type neither file holds offers a practice",
        );
        assert!(
            image.the_extra_is_open(),
            "the clears were not kept out of the game's own file, which is what lights the Extra",
        );

        // **And the other mode's file is the game's own**, whose records come back whole.
        at_the_title_menu(&game);
        ask(&game);
        game.press(keys::DOWN);
        answer(&game, keys::Z);
        assert!(
            log.said("score: 2 of 7 shot types' records read out of scoreth10.dat"),
            "the game's own file was not read on the answer for レガシーモード:\n  {}",
            log.lines().join("\n  ")
        );
        for number in [0, 1] {
            assert_eq!(
                game.image().the_score_objects_record(number),
                shot_types_record(number as u32, THE_GAMES),
                "shot type {number}'s record is not the game's own file's",
            );
        }
    });
}

/// What every byte of a record each file holds is, so that where a byte of the object came from reads off it.
const THE_GAMES: u8 = 0x01;
const THE_MODES: u8 = 0x22;

/// **Where a record keeps whether each practice stage is offered**: at `+0x5` of 8-byte entries from `+0x4dc`, one
/// for each difficulty but the Extra and each stage counted from 1 — set as a run clears the stage (0x416c08) and
/// read by the practice select (0x4314be).
const PRACTICE_AT: usize = 0x4dc;
const PRACTICE_BYTES: usize = 8;
const PRACTICES: usize = 4 * 6 + 1;
const OFFERED: usize = 0x5;

/// The sixth card's entry, `0x90` bytes each from `+0x59c` (0x432767), whose number 0x42acb0 writes at `+0x88`.
const A_CARD: usize = 0x59c + 5 * 0x90;

fn offers(record: &[u8]) -> [u8; PRACTICES] {
    std::array::from_fn(|practice| record[PRACTICE_AT + practice * PRACTICE_BYTES + OFFERED])
}

/// A record's bytes past its header, less the ones that say a practice is offered.
fn outside_the_offers(record: &[u8]) -> impl Iterator<Item = u8> + '_ {
    let offered: Vec<usize> = (0..PRACTICES)
        .map(|practice| PRACTICE_AT + practice * PRACTICE_BYTES + OFFERED)
        .collect();
    record
        .iter()
        .enumerate()
        .skip(0x10)
        .filter(move |(at, _)| !offered.contains(at))
        .map(|(_, byte)| *byte)
}

/// Nothing is answered while the question holds its keys off with the key that put it up still down, and a key
/// held down answers once and not once a frame.
#[test]
fn the_press_th10s_question_went_up_on_does_not_answer_it_and_a_held_key_answers_once() {
    in_its_own_process(|| {
        let game = game("th10-mode-grace");
        let log = game.log();
        game.keyboard().set(keys::Z, true);
        game.frames_until("the question", 8, || {
            !game.says(title(Menu::Run, LANGUAGE)).is_empty()
        });
        for _ in 0..90 {
            game.one_frame();
            assert_eq!(
                outcomes(&game),
                0,
                "a key held from before the question answered it:\n  {}",
                log.lines().join("\n  ")
            );
        }
        assert!(still_asking(&game), "the question came down by itself");

        game.keyboard().set(keys::Z, false);
        game.one_frame();
        game.keyboard().set(keys::Z, true);
        game.frames_until("the mode chosen", 8, || outcomes(&game) == 1);
        let once = log.written();
        game.frames(30);
        assert!(
            !log.said_since(once, "mode: answered on the"),
            "the question answered again with the key still down:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// Alt-tabbed away, nothing is pressed however hard, and a key held while the game goes to the back is not a
/// press when it comes forward; a host that will not say what is down reads as nothing down.
#[test]
fn keys_are_not_read_at_th10s_question_with_another_window_in_front() {
    in_its_own_process(|| {
        let game = game("th10-mode-unread");
        let log = game.log();
        ask(&game);

        game.sim()
            .display()
            .set_foreground(orb_api::Hwnd(crate::fake::WINDOW.0 + 1));
        game.keyboard().set(keys::Z, true);
        game.frames(60);
        assert_eq!(
            outcomes(&game),
            0,
            "a key held into another program's window answered the question:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(still_asking(&game), "the question came down by itself");

        game.sim().display().set_foreground(crate::fake::WINDOW);
        game.frames(30);
        assert_eq!(
            outcomes(&game),
            0,
            "the key held while the window was behind answered as it came forward:\n  {}",
            log.lines().join("\n  ")
        );

        game.keyboard().refuse(true);
        game.frames(30);
        assert_eq!(
            outcomes(&game),
            0,
            "a host that would not say what was down answered the question:\n  {}",
            log.lines().join("\n  ")
        );
        game.keyboard().refuse(false);

        game.keyboard().set(keys::Z, false);
        game.one_frame();
        answer(&game, keys::Z);
    });
}
