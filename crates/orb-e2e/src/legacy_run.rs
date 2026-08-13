//! One レガシーモード run, which is the same game with none of orb's own work happening in it.
//!
//! The e2e test beside this one — `pointdevice_run` — is what the mode *does*; this is what answering
//! the other way costs, and it is worth an e2e test of its own because every one of those things is
//! something orb has to *not* do. A run whose chapters were kept anyway would look exactly like this
//! one until somebody died.
//!
//! The same game, driven the same way: it presses keys and reads back the game's own memory, the
//! game's own record of the card, and what orb put in the log.

use crate::fake::th06::{ATTACK_CHANGES, CARD, CARD_STARTS, Fake, lives_row};
use crate::fake::{LANGUAGE, Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen, item};
use orb_core::game::{Menu, RunStart};
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_core::mode::{Mode, aside, title};
use orb_sim::keys;

/// The run this e2e test plays, which is the other e2e test's: Normal, Reimu A, from stage one.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

#[test]
fn a_legacy_run_keeps_no_chapters_offers_no_retry_and_leaves_nothing_behind() {
    in_its_own_process(|| {
        let game = Fake::attach("legacy", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();

        // ── 1. レガシーモード. The same question over the same title menu, answered the other way: one
        // press down the list of two, after the frames the question reads nothing over.
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.frames_until("the title menu ready to act on a press", 60, || {
            game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "the press did not put the mode question up: {:?}",
            log.lines(),
        );
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        // The cursor is on the other one now, which is a colour on the screen and not a number anywhere:
        // `menu_ui` draws the item under it in `SELECTED`. And the lines under the items say what *this*
        // choice means, which is not what the other one's said.
        game.forget();
        game.frame();
        let pointdevice = game.says("完全無欠モード");
        let legacy = game.says("レガシーモード");
        assert_eq!(
            (pointdevice.len(), legacy.len()),
            (1, 1),
            "the question is not the two modes: {:?}",
            log.lines(),
        );
        assert_eq!(
            legacy[0].color, SELECTED,
            "the cursor is not on レガシーモード"
        );
        assert_eq!(pointdevice[0].color, NORMAL);
        assert_eq!(
            game.says(aside(Menu::Run, Mode::Normal, LANGUAGE)[0]).len(),
            1,
            "what レガシーモード means is not said under it",
        );
        assert!(
            game.says(aside(Menu::Run, Mode::Pointdevice, LANGUAGE)[0])
                .is_empty(),
            "the lines under the other choice are still on the screen",
        );
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        assert!(
            log.said("mode: normal, was pointdevice"),
            "the other mode was not chosen: {:?}",
            log.lines(),
        );

        // ── 2. The run starts, and the one thing orb does to it is write down that it is doing nothing:
        // the hook over the moment a stage's numbers are put in place is still in the path, and it keeps
        // nothing.
        //
        // Which file the score goes to is not something an e2e test can read: what decides it is an import
        // hook on `CreateFileW` that a test cannot install, and the only thing an e2e test could hold it to
        // would be an observer written for the e2e test. `score.rs`'s own tests cover the decision; the
        // line above is what says the mode this run is in, which is the whole of what it turns on.
        game.frames_until("the shot type select ready to act on a press", 90, || {
            game.image().front_end_now().screen == Screen::ShotType
                && game.image().front_end_now().acts_on_a_press()
        });
        game.press(keys::Z);
        game.frames_until("the stage built", 8, || game.state().playing);
        game.one_frame_to_drain_the_log();
        assert!(
            log.said("resume: stage 1 of a run orb is not keeping; nothing of it is written down"),
            "the run was written down after all: {:?}",
            log.lines(),
        );

        // ── 3. The stage is played past where the other e2e test's first three chapters were, and none of
        // them is here: no snapshot is taken, so there is nothing to go back to and nothing to mark the
        // lives with.
        //
        // What the questions drew on the way in is forgotten first: those cover the whole output, the
        // count of lives with it, and what is being asked about here is the frames of the stage.
        game.forget();
        game.frames_until(
            "the stage past where a card would have been a chapter",
            900,
            || game.state().stage_frames > CARD_STARTS + 60,
        );
        assert!(
            !log.said("chapter 1 (stage start)"),
            "a chapter was taken in a run that cannot be rewound: {:?}",
            log.lines(),
        );
        assert!(
            !log.said("lives: the brush is"),
            "the lives were marked as disabled in a run that loses them: {:?}",
            log.lines(),
        );
        assert!(
            !game
                .drawn()
                .quads
                .iter()
                .any(|quad| quad.overlaps(&lives_row())),
            "something was drawn over the count of lives",
        );
        assert!(
            game.says("DISABLE").is_empty(),
            "the lives were called disabled in a run that loses them",
        );
        // The game counted its own attempt at the card where the card started, which is what it does in
        // either mode: that number is the game's and orb only ever adds to it.
        assert_eq!(game.image().card_attempts(CARD), 1);

        // ── 4. 被弾. A life, and no menu: the game takes the death, respawns, and carries on.
        let before = game.state();
        game.hit();
        game.frame();
        assert_eq!(
            game.state().lives,
            before.lives - 1,
            "the death cost a life"
        );
        assert_eq!(game.state().deaths, 1, "and is counted against the run");
        assert!(
            !log.said("died in chapter"),
            "the retry menu was offered in a run with nothing to retry: {:?}",
            log.lines(),
        );
        let died_at = game.state().stage_frames;
        game.frames(10);
        assert!(
            game.state().stage_frames > died_at,
            "the game stopped updating, which is what a retry menu would do to it",
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            1,
            "an attempt was counted for a death that cost a life",
        );

        // ── 5. And nothing is left behind for a later launch to offer: the run goes on past where the
        // other e2e test's chapter was written down, and no file appears.
        game.frames_until(
            "the stage past where a chapter was written down",
            900,
            || game.state().stage_frames > ATTACK_CHANGES,
        );
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "a legacy run was left to be picked up: {:?}",
            orb_core::resume::left(game.dir()),
        );
        assert!(
            !log.said("frame(s) of buttons"),
            "the buttons of a legacy run were written down: {:?}",
            log.lines(),
        );

        // ── 6. Out of lives, which is where the game ends a run: the run finishes at its own result screen,
        // and that screen is what writes the record — nothing of orb's is built for it, and the count in it
        // is the game's own with nothing of orb's added to it.
        //
        // **Which is the difference the two e2e tests are about.** The other one dies on a spell card,
        // goes back to the chapter it was in, and the record ends up holding that attempt as well; here
        // the same death costs a life, nothing is retried, and the only number against the card is the one
        // the card's own start put there. Which of the two files it is written to — `pointdevice_score.dat`
        // or the game's own — is decided by an import hook on `CreateFileW` that an e2e test cannot install,
        // and `score.rs`'s own tests are what hold it to that. The screen itself, and what it wrote, is
        // `the_screen_a_finished_run_ends_at.rs`'s.
        for _ in 0..2 {
            game.hit();
            game.frame();
        }
        assert_eq!(game.state().lives, -1, "the run is out of lives");
        game.frames_until("the run over", 60, || !game.state().in_run);
        game.frames_until("the record kept for the screen to write", 60, || {
            log.said("score: the run finished, and the screen it finished at is what writes")
        });
        assert!(
            !log.said("score: the captures in memory cleared for the ranking about to be read"),
            "the record was cleared on the way into the screen that is about to write it: {:?}",
            log.lines(),
        );
        assert_eq!(
            game.image().card_attempts(CARD),
            1,
            "the record a legacy run is written from counts an attempt orb added",
        );

        // ── 7. レガシーのスコア画面, read the way somebody reads it: the title menu's `Score`, the same
        // question about which of the two rankings, and then the row for that card. What it shows is the
        // one attempt the card's own start counted — nothing of orb's, there having been no retry to count.
        //
        // Past the screen the run finished at first, which is a screen somebody reads.
        game.frames_until(
            "the title menu after the run's own result screen",
            600,
            || {
                game.image().front_end_now().screen == Screen::Title
                    && game.image().front_end_now().acts_on_a_press()
            },
        );
        for _ in 0..item::SCORE {
            game.press(keys::DOWN);
        }
        game.press(keys::Z);
        assert!(
            log.said("menu: Scores is under the cursor, asking which mode"),
            "the press did not put the ranking's question up: {:?}",
            log.lines(),
        );
        game.forget();
        game.frame();
        assert_eq!(game.says(title(Menu::Scores, LANGUAGE)).len(), 1);
        game.press_until(keys::Z, "レガシーのスコア画面", || {
            game.image().scene() == Scene::Ranking
        });

        game.forget();
        game.frame();
        let card = game.says(&format!("CARD {CARD}"));
        let attempts = game.says("1");
        assert_eq!(
            (card.len(), attempts.len()),
            (1, 1),
            "the ranking is not one row for the one card there is a record of",
        );
        assert_eq!(
            attempts[0].y, card[0].y,
            "the count is not on that card's own row",
        );
        assert!(
            game.says("2").is_empty(),
            "the ranking counts an attempt for a death that cost a life",
        );
    });
}
