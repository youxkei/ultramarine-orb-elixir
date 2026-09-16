//! **The same launch of th07 on a machine whose own windows are English.**
//!
//! Every other e2e test over this game reads orb's screens in Japanese, that being the language the game is in
//! and the one a simulated machine is set to — see [`LANGUAGE`](crate::fake::LANGUAGE). This one declares the
//! machine instead, with nothing in `orb.yaml` about the language, so the answer comes from the only place left:
//! the language Windows is showing its own windows in.
//!
//! **What is asserted is the words on the screen and not that a function was called.** A screen is localised when
//! what it drew is the English wording and none of the Japanese is anywhere on it, so both are read back — the
//! Japanese by name, because a table with one cell in the wrong language is the mistake two languages make and it
//! draws a screen that looks finished.
//!
//! And it is asked of this game because the *face* is this game's own: th07 ships no `font.ttf`, so every screen
//! here is drawn in a face the machine lent — and a
//! face lent for one language is not evidence about the other.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::{Language, LogLevel};
use orb_core::game::Menu;
use orb_core::game::th07::Th07;
use orb_core::menu_ui::SELECTED;
use orb_core::mode::{self, Mode};
use orb_sim::{keys, langid};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The name orb gives the run these e2e tests play, which is what the replay of one is kept under, and the date
/// the simulated host's calendar is set to, which the kept replay's own line carries.
const SLOT: &str = "normal-reimu-a";
const DATED: &str = "20040815-214233";

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// A launch on a machine whose windows are English, with `orb.yaml` saying nothing about the language.
///
/// # Panics
/// If orb did not say which language it settled on and where the answer came from. That line is what a screenshot
/// of a screen somebody could not read is held against, so a launch without it is one this e2e test cannot claim
/// anything about either.
fn on_an_english_machine(name: &str) -> Box<Fake> {
    let game = Fake::attach_on_a_machine_in(
        langid::ENGLISH,
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            // Said rather than left as it comes: what this e2e test is about is the answer coming from the
            // machine, and a launch that had set the language would be asking a different question.
            assert_eq!(config.language, None, "the file says which language");
        },
    );
    assert!(
        game.log()
            .said("language: english, which is what this machine's own windows are in"),
        "nothing said which language orb settled on:\n  {}",
        game.log().lines().join("\n  ")
    );
    game
}

/// Every line of a screen, held against both languages at once: the English on it once, and none of the Japanese
/// anywhere.
///
/// `lit` names the one line that has to be under the cursor, which is what makes the screen answerable — a menu
/// with nothing lit is one nobody could choose from. `None` where the line under the cursor is not one of these:
/// the retry menu's is a chapter's name, which is the same in both languages.
///
/// # Panics
/// Naming whichever line is missing, doubled, or in the language this machine is not in.
fn reads(game: &Fake, said: &[(&str, &str)], lit: Option<&str>) {
    game.forget();
    game.one_frame();
    for (english, japanese) in said {
        let drawn = game.says(english);
        assert_eq!(drawn.len(), 1, "{english} is not on the screen once");
        assert!(
            game.says(japanese).is_empty(),
            "{japanese} is on a screen this machine reads in English",
        );
        if lit == Some(english) {
            assert_eq!(drawn[0].color, SELECTED, "the cursor is not on {english}");
        }
    }
}

/// The question over the game's own title menu: its title, the two modes, and what the mode under the cursor
/// means.
#[test]
fn the_question_over_the_title_menu_of_th07_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("th07-english-mode-question");
        let log = game.log();
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        // A frame with the cursor there before the press, for the reason `Fake::answers_the_mode` gives.
        game.frame();
        game.press_until(keys::Z, "the mode question", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });

        let mut said = vec![
            (
                mode::title(Menu::Run, Language::English),
                mode::title(Menu::Run, Language::Japanese),
            ),
            (
                Mode::Pointdevice.name(Language::English),
                Mode::Pointdevice.name(Language::Japanese),
            ),
            (
                Mode::Normal.name(Language::English),
                Mode::Normal.name(Language::Japanese),
            ),
        ];
        // The cursor starts on the mode orb is already in, which for a launch nobody has answered yet is the one
        // with chapters — so what is described under the two is that one's.
        let lines = mode::aside(Menu::Run, Mode::Pointdevice, Language::English)
            .iter()
            .zip(mode::aside(
                Menu::Run,
                Mode::Pointdevice,
                Language::Japanese,
            ))
            .map(|(english, japanese)| (*english, *japanese));
        said.extend(lines);
        reads(
            &game,
            &said,
            Some(Mode::Pointdevice.name(Language::English)),
        );
    });
}

/// And the menu that appears where a chapter was lost, with the question one of its items asks.
#[test]
fn the_menu_where_a_chapter_of_th07_was_lost_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("th07-english-retry-menu");
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 3"));

        reads(
            &game,
            &[
                ("Retry the chapter", "チャプターをやり直す"),
                ("Retry from further back", "更に前からやり直す"),
                ("Retry the stage", "ステージをやり直す"),
                ("Back to the title screen", "タイトルに戻る"),
            ],
            Some("Retry the chapter"),
        );

        // The screen behind the second item, which is a screen of its own with a question of its own above what it
        // lists. The chapters it lists are named by the boundary detector — the names the status line and the log
        // call them by — and those stay as they are in either language.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapters behind it", || {
            log.said("retry: asking which chapter further back")
        });
        reads(&game, &[("Where to start again", "どこからやり直す")], None);
        assert_eq!(
            game.says("MIDSTAGE 1").len(),
            1,
            "the stage's own start is not listed",
        );

        // And the question one of them asks, which names the chapter in whichever language the rest of the line is
        // in, and says under it what going back leaves behind.
        game.frames(READS_KEYS_AFTER);
        game.press_until(keys::Z, "the chapter asked about", || {
            log.said("retry: asking about a chapter further back")
        });
        reads(
            &game,
            &[
                (
                    "Start again from MIDBOSS NONSPELL 1?",
                    "MIDBOSS NONSPELL 1 からやり直す？",
                ),
                (
                    "There is no going back to the chapter you are in",
                    "今のチャプターには戻れません",
                ),
                ("Yes", "はい"),
                ("No", "いいえ"),
            ],
            // The cursor starts on no, which is what makes a press on the frame the question begins reading keys
            // on cost nothing.
            Some("No"),
        );
    });
}

/// And the question about a run left unfinished, with the mark that goes up before it is asked — over this
/// game's shot select, which is the screen whose own decide chooses the run.
#[test]
fn the_question_about_a_run_of_th07_left_unfinished_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("th07-english-resume-question");
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames_until_a_thread("the chapter written down", || {
            log.said("chapter 1 (MIDSTAGE 1) at frame")
        });
        // タイトルに戻る, which is the one way out of a run that leaves its chapter written down: the run did
        // not finish, so nothing takes the file away. The game is left at its title menu with the mode answered.
        game.gives_the_run_up();

        // The shot select with the run under its cursor, where the mark says that run has a chapter written
        // down, and the press asks about it.
        let from = log.written();
        game.at_the_shot_select();
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("A run left unfinished").len(),
            1,
            "the mark is not on the shot select of a machine whose windows are English:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.says("中断データあり").is_empty(),
            "the mark is in Japanese on a machine whose windows are English",
        );

        game.press_until(keys::Z, "the question", || {
            log.said_since(
                from,
                &format!("resume: {SLOT} was left; asking where to start"),
            )
        });
        reads(
            &game,
            &[
                ("Where to start", "どこから始める"),
                ("Continue", "つづきから"),
                ("From the beginning", "はじめから"),
            ],
            // The cursor starts on the chapter that was left: a run picked up by accident is a run put back
            // where it was, where a fresh one writes over what was left.
            Some("Continue"),
        );
        // And what the other item costs, which is said only while the cursor is on it.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("The run left unfinished will be written over")
                .len(),
            1,
            "what starting again costs is not said in English",
        );
        assert!(
            game.says("中断データは上書きされます").is_empty(),
            "what starting again costs is said in Japanese",
        );
    });
}

/// **The screens a replay is kept and watched on**, which is the offer where a run is written over and the four
/// lists a replay is chosen out of.
///
/// Every one of them is orb's own — the game has no screen for a replay of a run that was rewound — so all of it
/// is a table of two languages, which is exactly where a cell in the wrong one hides.
#[test]
fn the_screens_a_replay_of_th07_is_kept_and_watched_on_are_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("th07-english-replay-screens");
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        game.gives_the_run_up();
        game.frames_until_a_thread("the replay written down", || {
            orb_core::replay::read_being_played(game.dir(), SLOT, &Th07).is_some()
        });

        // ── The offer, which はじめから is the second of the two moments of: the run left unfinished is about
        // to be written over, so its replay is offered before that happens.
        let from = log.written();
        game.at_the_shot_select();
        game.press_until(keys::Z, "the question about the run left behind", || {
            log.said_since(from, "asking where to start")
        });
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "はじめから answered", || {
            log.said_since(from, "resume: from the beginning, answered on the keyboard")
        });
        game.frames_until("the offer", 60, || {
            log.said_since(from, "asking whether to keep it")
        });
        reads(
            &game,
            &[
                ("Keep this replay", "リプレイを保存する"),
                ("Yes", "はい"),
                ("No", "いいえ"),
                (
                    "Watchable by stage, chapter and attempt",
                    "ステージ・チャプター・トライを選んで見られます",
                ),
            ],
            // Keeping is what the cursor starts on: a replay declined by accident is a run nobody can watch
            // again.
            Some("Yes"),
        );
        // And what the other answer costs, said only while the cursor is on it.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.forget();
        game.one_frame();
        assert_eq!(game.says("This replay is lost").len(), 1);
        assert!(game.says("このリプレイは失われます").is_empty());

        // Kept, and the run it was written over started and given up in turn: back up to はい and answered,
        // the game building the run the answer let through, and the title menu after it.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::UP);
        game.press_until(keys::Z, "the replay kept", || {
            log.said_since(from, "replay: kept on the keyboard")
        });
        game.image().a_run_has_been_chosen();
        game.builds_a_stage();
        game.frames_until("its own first chapter", 600, || {
            log.said_since(from, "stage 1 chapter 1 (stage start)")
        });
        game.gives_the_run_up();

        // ── And the lists it is watched out of, reached the way anybody reaches them: the replay item on the
        // title menu, and Pointdevice Mode over it.
        let from = log.written();
        game.image().the_title_menu_points_at_the_replays();
        game.frames(2);
        game.press(keys::Z);
        assert!(
            log.said_since(from, "menu: Replays is under the cursor, asking which mode"),
            "the press did not put the question up:\n  {}",
            log.lines().join("\n  ")
        );
        reads(
            &game,
            &[
                ("Which replays to watch", "どちらのリプレイを見る"),
                ("Pointdevice Mode", "完全無欠モード"),
                ("Legacy Mode", "レガシーモード"),
            ],
            Some("Pointdevice Mode"),
        );
        game.press_until(keys::Z, "the list of replays", || {
            log.said_since(from, "replay(s) to watch")
        });
        // And what its one line says where a name would be: this run was left rather than played out. The
        // whole line, that being what an item is — the word alone is a substring of it and nothing draws it.
        reads(
            &game,
            &[
                ("Choose a replay", "リプレイを選ぶ"),
                (
                    &format!("Interrupted  {SLOT}  {DATED}"),
                    &format!("中断  {SLOT}  {DATED}"),
                ),
            ],
            None,
        );

        game.press_until(keys::Z, "the stages", || {
            log.said_since(from, "stage(s) in it")
        });
        reads(&game, &[("Choose a stage", "ステージを選ぶ")], None);

        game.press_until(keys::Z, "the chapters", || {
            log.said_since(from, "chapter(s) in it")
        });
        reads(&game, &[("Choose a chapter", "チャプターを選ぶ")], None);

        game.press_until(keys::Z, "the attempts", || {
            log.said_since(from, "attempt(s) at it")
        });
        reads(
            &game,
            &[
                ("Choose an attempt", "トライを選ぶ"),
                ("Attempt 1", "トライ 1"),
                (
                    "The attempt the run went on from",
                    "このトライで先に進みました",
                ),
            ],
            Some("Attempt 1"),
        );
    });
}
